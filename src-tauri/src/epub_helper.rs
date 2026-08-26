// epub_helper.rs
//
// Rough Rust port of your Java EpubHelper.java:
// - zip-based EPUB detection
// - container.xml -> OPF
// - OPF manifest + spine
// - XHTML/HTML -> plain text (block breaks + whitespace normalize)

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;

use crate::utils;
use kuchiki::traits::*;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Cursor;
use thiserror::Error;
use zip::ZipArchive;

#[derive(Debug, Error)]
pub enum EpubError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Invalid EPUB: {0}")]
    Invalid(String),

    #[error("XML parse error: {0}")]
    Xml(String),
}

#[derive(Debug, Clone)]
struct ManifestItem {
    href: String,
    media_type: String,
    is_nav: bool,
}

#[derive(Debug)]
struct OpfData {
    manifest: HashMap<String, ManifestItem>,
    spine: Vec<String>,
}

// ---------------------------- EPUB extraction ----------------------------

#[derive(Debug, Clone, Copy)]
pub struct ExtractOptions {
    pub include_part_headings: bool,
    pub normalize_newlines: bool,
    pub skip_nav_documents: bool,
}

impl Default for ExtractOptions {
    fn default() -> Self {
        Self {
            include_part_headings: false,
            normalize_newlines: true,
            skip_nav_documents: true,
        }
    }
}

pub fn extract_epub_all_text(
    epub_path: impl AsRef<Path>,
    options: ExtractOptions,
) -> Result<String, EpubError> {
    let epub_path = epub_path.as_ref();

    let file = File::open(epub_path)?;
    let mut zip = ZipArchive::new(file)?;

    let opf_path = find_opf_path(&mut zip)?.ok_or_else(|| {
        EpubError::Invalid("container.xml has no OPF rootfile. Not a valid .epub?".into())
    })?;

    let opf_dir = get_dir(&opf_path);
    let opf = load_opf(&mut zip, &opf_path)?;

    let mut sb = String::with_capacity(256 * 1024);

    for idref in opf.spine.iter() {
        let item = match opf.manifest.get(idref) {
            Some(it) => it,
            None => continue,
        };

        if !looks_like_html(&item.media_type, &item.href) {
            continue;
        }
        if options.skip_nav_documents && item.is_nav {
            continue;
        }

        let full_name = combine_zip_path(&opf_dir, &item.href);

        let mut zf = match zip.by_name(&full_name) {
            Ok(f) => f,
            Err(_) => continue,
        };

        if options.include_part_headings {
            if !sb.is_empty() && !ends_with_newline_str(&sb) {
                sb.push('\n');
            }
            sb.push_str("=== ");
            sb.push_str(&full_name);
            sb.push_str(" ===\n");
        }

        let chapter_text = extract_xhtml_text(&mut zf)?;
        sb.push_str(&chapter_text);

        if !ends_with_newline_str(&sb) {
            sb.push('\n');
        }
        sb.push('\n'); // blank line between spine docs
    }

    let mut text = sb;

    if options.normalize_newlines {
        // In Rust strings are already '\n' typically; but keep parity with Java:
        text = text.replace("\r\n", "\n").replace('\r', "\n");
    }

    text = normalize_excess_blank_lines(&text);
    Ok(text)
}

// ---------------------------- container.xml ----------------------------

fn find_opf_path<R: Read + Seek>(zip: &mut ZipArchive<R>) -> Result<Option<String>, EpubError> {
    let mut entry = match zip.by_name("META-INF/container.xml") {
        Ok(f) => f,
        Err(_) => return Ok(None),
    };

    let mut bytes = Vec::with_capacity(8 * 1024);
    entry.read_to_end(&mut bytes)?;

    let mut r = Reader::from_reader(Cursor::new(bytes));
    r.config_mut().trim_text(true);
    r.config_mut().expand_empty_elements = true;

    let mut buf = Vec::new();

    loop {
        match r.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => {
                let name = e.local_name();

                if name.as_ref() == "rootfile" {
                    // EPUB container.xml: <rootfile full-path="..."/>
                    let mut full = utils::attr_any_string(&e, "full-path")
                        .unwrap_or_default()
                        .trim()
                        .to_string();

                    if full.is_empty() {
                        // fallback seen in some broken files
                        full = utils::attr_any_string(&e, "fullpath")
                            .unwrap_or_default()
                            .trim()
                            .to_string();
                    }

                    if !full.is_empty() {
                        return Ok(Some(full));
                    }
                }
            }

            Err(e) => return Err(EpubError::Xml(e.to_string())),
            _ => {}
        }

        buf.clear();
    }

    Ok(None)
}

// ---------------------------- OPF parsing ----------------------------

fn load_opf<R: Read + Seek>(zip: &mut ZipArchive<R>, opf_path: &str) -> Result<OpfData, EpubError> {
    let mut entry = zip
        .by_name(opf_path)
        .map_err(|_| EpubError::Invalid(format!("OPF not found: {opf_path}")))?;

    let mut bytes = Vec::with_capacity(64 * 1024);
    entry.read_to_end(&mut bytes)?;

    let mut r = Reader::from_reader(Cursor::new(bytes));
    r.config_mut().trim_text(true);
    r.config_mut().expand_empty_elements = true;

    let mut manifest: HashMap<String, ManifestItem> = HashMap::with_capacity(1024);
    let mut spine: Vec<String> = Vec::with_capacity(256);

    let mut buf = Vec::new();

    loop {
        match r.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => {
                let name = e.local_name();

                // manifest: <item id="x" href="y" media-type="..." properties="nav"/>
                if name.as_ref() == "item" {
                    let id = utils::attr_any_string(&e, "id").unwrap_or_default();
                    let href = utils::attr_any_string(&e, "href").unwrap_or_default();
                    let id = id.trim();
                    let href = href.trim();

                    if !id.is_empty() && !href.is_empty() {
                        let media_type =
                            utils::attr_any_string(&e, "media-type").unwrap_or_default();
                        let props = utils::attr_any_string(&e, "properties").unwrap_or_default();
                        let is_nav = contains_token_ignore_case(&props, "nav");

                        manifest.insert(
                            id.to_string(),
                            ManifestItem {
                                href: href.to_string(),
                                media_type,
                                is_nav,
                            },
                        );
                    }
                }
                // spine: <itemref idref="..."/>
                else if name.as_ref() == "itemref" {
                    let idref = utils::attr_any_string(&e, "idref").unwrap_or_default();
                    let idref = idref.trim();
                    if !idref.is_empty() {
                        spine.push(idref.to_string());
                    }
                }
            }

            Err(e) => return Err(EpubError::Xml(e.to_string())),
            _ => {}
        }

        buf.clear();
    }

    Ok(OpfData { manifest, spine })
}

fn contains_token_ignore_case(space_separated: &str, token: &str) -> bool {
    let s = space_separated.trim();
    if s.is_empty() {
        return false;
    }
    s.split_whitespace().any(|p| p.eq_ignore_ascii_case(token))
}

fn looks_like_html(media_type: &str, href: &str) -> bool {
    let mt = media_type.trim();
    let h = href.trim();

    if mt.is_empty() {
        return utils::ends_with_ignore_case(h, ".xhtml")
            || utils::ends_with_ignore_case(h, ".html")
            || utils::ends_with_ignore_case(h, ".htm");
    }

    if mt.eq_ignore_ascii_case("application/xhtml+xml") {
        return true;
    }
    if mt.eq_ignore_ascii_case("text/html") {
        return true;
    }

    if mt.to_ascii_lowercase().contains("html") {
        return true;
    }

    utils::ends_with_ignore_case(h, ".xhtml")
        || utils::ends_with_ignore_case(h, ".html")
        || utils::ends_with_ignore_case(h, ".htm")
}

// ---------------------------- XHTML/HTML -> plain text ----------------------------

fn extract_xhtml_text<R: Read>(mut xhtml_stream: R) -> Result<String, EpubError> {
    let html = read_to_string_lossy_utf8(&mut xhtml_stream)?;

    let document = kuchiki::parse_html().one(html);

    for sel in ["script", "style", "head", "svg", "math", "noscript"] {
        if let Ok(nodes) = document.select(sel) {
            for n in nodes {
                n.as_node().detach();
            }
        }
    }

    let body: kuchiki::NodeRef = match document.select_first("body") {
        Ok(b) => b.as_node().clone(),
        Err(_) => document.clone(),
    };

    let mut sb = String::with_capacity(32 * 1024);
    walk_dom(&body, &mut sb, 0);

    let mut text = sb;
    text = text.replace('\u{00AD}', ""); // soft hyphen
    text = text.replace('\u{00A0}', " "); // NBSP
    Ok(text)
}

fn walk_dom(node: &kuchiki::NodeRef, sb: &mut String, depth: usize) {
    // head() behavior
    if let Some(el) = node.as_element() {
        let name = el.name.local.to_string();
        if is_block_element(&name) {
            ensure_paragraph_break(sb);
        } else if name.eq_ignore_ascii_case("br") {
            sb.push('\n');
        }
    } else if let Some(text) = node.as_text() {
        let t = text.borrow();
        if !t.is_empty() {
            append_normalized_text(sb, &t);
        }
    }

    // children
    for child in node.children() {
        walk_dom(&child, sb, depth + 1);
    }

    // tail() behavior
    if let Some(el) = node.as_element() {
        let name = el.name.local.to_string();
        if is_block_element(&name) {
            ensure_paragraph_break(sb);
        }
    }
}

fn is_block_element(local_name: &str) -> bool {
    matches_ignore_case(
        local_name,
        &[
            "p",
            "div",
            "section",
            "article",
            "blockquote",
            "li",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "hr",
        ],
    )
}

fn matches_ignore_case(s: &str, items: &[&str]) -> bool {
    items.iter().any(|x| s.eq_ignore_ascii_case(x))
}

fn append_normalized_text(sb: &mut String, t: &str) {
    // Same logic as Java: collapse whitespace to single spaces,
    // but don't add leading space.
    for c in t.chars() {
        if c.is_whitespace() {
            if sb.is_empty() {
                continue;
            }
            let last = sb.chars().last().unwrap_or('\0');
            if last == ' ' || last == '\n' || last == '\r' || last == '\t' {
                continue;
            }
            sb.push(' ');
        } else {
            sb.push(c);
        }
    }
}

fn ensure_paragraph_break(sb: &mut String) {
    trim_trailing_spaces(sb);
    if sb.is_empty() {
        return;
    }

    // already ends with "\n\n"?
    if sb.ends_with("\n\n") {
        return;
    }

    if !sb.ends_with('\n') {
        sb.push('\n');
    }
    sb.push('\n');
}

fn trim_trailing_spaces(sb: &mut String) {
    while sb.ends_with(' ') || sb.ends_with('\t') {
        sb.pop();
    }
}

// ---------------------------- Paths / normalize ----------------------------

fn get_dir(path: &str) -> String {
    match path.rfind('/') {
        Some(idx) => path[..=idx].to_string(),
        None => String::new(),
    }
}

fn combine_zip_path(dir: &str, href: &str) -> String {
    let mut raw = String::with_capacity(dir.len() + href.len() + 4);
    raw.push_str(dir);
    raw.push_str(href);

    // normalize slashes
    let raw = raw.replace('\\', "/");

    // Resolve "." and ".." in a zip-ish way, preserving forward slashes
    let mut stack: Vec<&str> = Vec::with_capacity(raw.len() / 4);
    for part in raw.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            stack.pop();
            continue;
        }
        stack.push(part);
    }

    stack.join("/")
}

fn normalize_excess_blank_lines(s: &str) -> String {
    // Keep at most 2 consecutive '\n'
    let mut out = String::with_capacity(s.len());
    let mut nl = 0usize;

    for c in s.chars() {
        if c == '\n' {
            nl += 1;
            if nl <= 2 {
                out.push(c);
            }
        } else {
            nl = 0;
            out.push(c);
        }
    }
    out
}

// ---------------------------- misc ----------------------------

fn ends_with_newline_str(s: &str) -> bool {
    s.ends_with('\n') || s.ends_with("\r\n")
}

fn read_to_string_lossy_utf8<R: Read>(mut r: R) -> Result<String, EpubError> {
    let mut bytes = Vec::with_capacity(32 * 1024);
    r.read_to_end(&mut bytes)?;
    // Java forces UTF-8; keep same, but tolerate invalid sequences
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}
