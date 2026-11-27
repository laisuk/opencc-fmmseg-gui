// epub_helper.rs
//
// Rough Rust port of your Java EpubHelper.java:
// - zip-based EPUB detection
// - container.xml -> OPF
// - OPF manifest + spine
// - XHTML/HTML -> plain text (block breaks + whitespace normalize)

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read, Seek};
use std::path::Path;

use crate::office_converter::zip::ZipArchive;
use crate::utils;
use kuchiki::traits::*;
use markup5ever::data::NAMED_ENTITIES;
use quick_xml::events::Event;
use quick_xml::Reader;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EpubError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] crate::office_converter::zip::result::ZipError),

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
    extract_epub_all_text_from_reader(File::open(epub_path)?, options)
}

/// Extract spine text from a seekable EPUB source, such as a file or a byte cursor.
pub fn extract_epub_all_text_from_reader<R: Read + Seek>(
    source: R,
    options: ExtractOptions,
) -> Result<String, EpubError> {
    let mut zip = ZipArchive::new(source)?;

    let opf_path = find_opf_path(&mut zip)?.ok_or_else(|| {
        EpubError::Invalid("container.xml has no OPF rootfile. Not a valid .epub?".into())
    })?;

    let opf_dir = get_dir(&opf_path);
    let opf = load_opf(&mut zip, &opf_path)?;

    let mut out = String::with_capacity(256 * 1024);
    let mut events = Vec::new();
    let mut elements = Vec::new();
    let mut chapter = String::new();

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

        let entry = match zip.by_name(&full_name) {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        if options.include_part_headings {
            if !out.is_empty() && !out.ends_with('\n') {
                push_newline(&mut out);
            }
            out.push_str("=== ");
            append_output(&mut out, &full_name, options.normalize_newlines, false);
            append_output(&mut out, " ===\n", false, false);
        }

        chapter.clear();
        if !stream_xhtml(entry, &mut chapter, &mut events, &mut elements)? {
            // Reopen this chapter after discarding any partial fast-path output.
            chapter.clear();
            extract_html_fallback(zip.by_name(&full_name)?, &mut chapter)?;
        }
        append_output(&mut out, &chapter, false, true);

        if !out.ends_with('\n') {
            push_newline(&mut out);
        }
        push_newline(&mut out);
    }

    Ok(out)
}

// ---------------------------- container.xml ----------------------------

fn find_opf_path<R: Read + Seek>(zip: &mut ZipArchive<R>) -> Result<Option<String>, EpubError> {
    let entry = match zip.by_name("META-INF/container.xml") {
        Ok(f) => f,
        Err(_) => return Ok(None),
    };

    let mut r = Reader::from_reader(BufReader::new(entry));
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
                        // Finish reading so the ZIP reader still checks the CRC.
                        io::copy(&mut r.into_inner(), &mut io::sink())?;
                        return Ok(Some(full));
                    }
                }
            }

            Err(quick_xml::Error::Io(e)) => return Err(io::Error::new(e.kind(), e).into()),
            Err(e) => return Err(EpubError::Xml(e.to_string())),
            _ => {}
        }

        buf.clear();
    }

    Ok(None)
}

// ---------------------------- OPF parsing ----------------------------

fn load_opf<R: Read + Seek>(zip: &mut ZipArchive<R>, opf_path: &str) -> Result<OpfData, EpubError> {
    let entry = zip
        .by_name(opf_path)
        .map_err(|_| EpubError::Invalid(format!("OPF not found: {opf_path}")))?;

    let mut r = Reader::from_reader(BufReader::new(entry));
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

            Err(quick_xml::Error::Io(e)) => return Err(io::Error::new(e.kind(), e).into()),
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

// Only use the fast path for structures whose HTML5 text order and whitespace
// are reproduced by walking XML events. HTML tree repair, foreign content,
// table foster parenting and raw-text edge cases go through Kuchiki.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Element {
    Html,
    Head,
    Body,
    Block,
    Paragraph,
    Heading,
    List,
    Item,
    Inline,
    Link,
    HiddenText,
}

fn stream_xhtml(
    entry: impl Read,
    chapter: &mut String,
    buf: &mut Vec<u8>,
    stack: &mut Vec<Element>,
) -> Result<bool, EpubError> {
    use Element::*;

    let mut reader = Reader::from_reader(BufReader::new(entry));
    reader.config_mut().trim_text(false);
    reader.config_mut().check_comments = true;
    buf.clear();
    stack.clear();

    let mut seen_html = false;
    let mut seen_head = false;
    let mut seen_body = false;

    loop {
        let parent = stack.last().copied();
        let hidden = parent == Some(HiddenText);
        let visible = seen_body && !hidden;
        match reader.read_event_into(buf) {
            Ok(event @ (Event::Start(_) | Event::Empty(_))) => {
                let (tag, empty) = match event {
                    Event::Start(tag) => (tag, false),
                    Event::Empty(tag) => (tag, true),
                    _ => unreachable!(),
                };
                let name = tag.name();
                let name = name.as_ref();
                // HTML ignores the XML self-closing flag on non-void elements.
                if hidden || tag.attributes().any(|attr| attr.is_err()) {
                    return Ok(false);
                }
                if empty {
                    match (parent, name) {
                        (Some(Head), "meta" | "link" | "base") => (),
                        (Some(Body | Block | Paragraph | Heading | Item | Inline | Link), "br") => {
                            chapter.push('\n');
                        }
                        (Some(Body | Block | Heading | Item | Inline | Link), "hr")
                            if !stack.contains(&Paragraph) =>
                        {
                            ensure_paragraph_break(chapter);
                        }
                        (
                            Some(Body | Block | Paragraph | Heading | Item | Inline | Link),
                            "img" | "wbr",
                        ) => (),
                        _ => return Ok(false),
                    }
                } else {
                    let element = match (parent, name) {
                        (None, "html") if !seen_html => {
                            seen_html = true;
                            Html
                        }
                        (Some(Html), "head") if !seen_head && !seen_body => {
                            seen_head = true;
                            Head
                        }
                        (Some(Html), "body") if !seen_body => {
                            seen_body = true;
                            Body
                        }
                        (Some(Head), "title" | "style" | "script" | "noscript") => HiddenText,
                        (Some(List), "li") => Item,
                        (Some(Body | Block | Paragraph | Heading | Item | Inline | Link), _) => {
                            match name {
                                "p" => Paragraph,
                                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Heading,
                                "div" | "section" | "article" | "blockquote" => Block,
                                "ol" | "ul" => List,
                                "a" if !stack.contains(&Link) => Link,
                                "span" | "em" | "strong" | "b" | "i" | "u" | "s" | "small"
                                | "sub" | "sup" | "code" | "mark" | "abbr" | "cite" | "q"
                                | "dfn" | "kbd" | "samp" | "var" | "bdi" | "bdo" => Inline,
                                "script" | "style" | "noscript" => HiddenText,
                                _ => return Ok(false),
                            }
                        }
                        _ => return Ok(false),
                    };
                    if (matches!(element, Block | Paragraph | Heading | List)
                        && stack.contains(&Paragraph))
                        || (element == Heading && stack.contains(&Heading))
                    {
                        return Ok(false);
                    }
                    if matches!(element, Block | Paragraph | Heading | Item) {
                        ensure_paragraph_break(chapter);
                    }
                    stack.push(element);
                }
            }
            Ok(Event::End(_)) => {
                // quick-xml checks the matching qualified name.
                if matches!(stack.pop(), Some(Block | Paragraph | Heading | Item)) {
                    ensure_paragraph_break(chapter);
                }
            }
            Ok(Event::Text(text)) => {
                let text = text.as_ref();
                let whitespace = text.bytes().all(is_html_space);
                if text.contains('\0')
                    || (!whitespace
                        && !hidden
                        && (!seen_body || matches!(parent, None | Some(Html | List))))
                {
                    return Ok(false);
                }
                if visible {
                    append_normalized_text(chapter, text);
                }
            }
            Ok(Event::GeneralRef(reference))
                if visible && !matches!(parent, None | Some(Html | List)) =>
            {
                match reference.resolve_char_ref() {
                    Ok(Some(c))
                        if !c.is_control() || matches!(c, '\t' | '\n' | '\r' | '\u{000C}') =>
                    {
                        append_normalized_text(chapter, c.encode_utf8(&mut [0; 4]));
                    }
                    Ok(None) => {
                        if !append_named_entity(chapter, reference.as_ref()) {
                            return Ok(false);
                        }
                    }
                    // HTML remaps some numeric control references (including C1).
                    _ => return Ok(false),
                }
            }
            Ok(Event::GeneralRef(_)) if hidden => (),
            // In an HTML body this is a bogus comment, ending at the
            // first '>'. Let Kuchiki handle any early terminator.
            Ok(Event::CData(text)) if !hidden && !text.contains('>') => (),
            Ok(Event::Comment(text)) if !hidden && !text.contains('>') => (),
            Ok(Event::Decl(decl)) if !seen_html && !decl.contains('>') => (),
            Ok(Event::DocType(doctype)) if !seen_html && doctype.eq_ignore_ascii_case("html") => (),
            Ok(Event::Eof) => return Ok(seen_body && stack.is_empty()),
            Err(quick_xml::Error::Io(error)) => {
                return Err(io::Error::new(error.kind(), error).into());
            }
            _ => return Ok(false),
        }
        buf.clear();
    }
}

fn append_named_entity(chapter: &mut String, name: &str) -> bool {
    // quick-xml separates the reference name from its semicolon. The HTML
    // tokenizer's table keys for complete references include the semicolon.
    let mut key = [0; 32];
    let Some(key) = key.get_mut(..=name.len()) else {
        return false;
    };
    key[..name.len()].copy_from_slice(name.as_bytes());
    key[name.len()] = b';';
    let Ok(key) = std::str::from_utf8(key) else {
        return false;
    };
    let Some(&(first, second)) = NAMED_ENTITIES.get(key) else {
        return false;
    };
    for codepoint in [first, second] {
        if let Some(c) = char::from_u32(codepoint).filter(|&c| c != '\0') {
            append_normalized_text(chapter, c.encode_utf8(&mut [0; 4]));
        }
    }
    true
}

fn is_html_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0C)
}

fn extract_html_fallback(xhtml_stream: impl Read, sb: &mut String) -> Result<(), EpubError> {
    let html = read_to_string_lossy_utf8(xhtml_stream)?;

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

    use kuchiki::iter::NodeEdge;
    for edge in body.traverse() {
        match edge {
            NodeEdge::Start(node) => {
                if let Some(el) = node.as_element() {
                    let name = el.name.local.as_ref();
                    if is_block_element(name) {
                        ensure_paragraph_break(sb);
                    } else if name.eq_ignore_ascii_case("br") {
                        sb.push('\n');
                    }
                } else if let Some(text) = node.as_text() {
                    append_normalized_text(sb, &text.borrow());
                }
            }
            NodeEdge::End(node) => {
                if let Some(el) = node.as_element() {
                    if is_block_element(el.name.local.as_ref()) {
                        ensure_paragraph_break(sb);
                    }
                }
            }
        }
    }

    Ok(())
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
            if !sb.is_empty() && !sb.ends_with([' ', '\n', '\r', '\t']) {
                sb.push(' ');
            }
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

fn push_newline(out: &mut String) {
    if !out.ends_with("\n\n") {
        out.push('\n');
    }
}

fn append_output(out: &mut String, text: &str, normalize_newlines: bool, chapter: bool) {
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' if normalize_newlines => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                push_newline(out);
            }
            '\n' => push_newline(out),
            '\u{00AD}' if chapter => (),
            _ => out.push(c),
        }
    }
}

fn read_to_string_lossy_utf8<R: Read>(mut r: R) -> Result<String, EpubError> {
    let mut bytes = Vec::with_capacity(32 * 1024);
    r.read_to_end(&mut bytes)?;
    Ok(String::from_utf8(bytes)
        .unwrap_or_else(|error| String::from_utf8_lossy(error.as_bytes()).into_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streams_xhtml_with_html5_text_semantics() {
        let html = r#"<?xml version="1.0"?><!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><meta charset="utf-8"/><title>Hidden</title><style>a { color: red }</style></head>
<body><h1>Heading</h1><section><div><p>漢<span> 字</span> 𠀀 &#x20000; &amp;
&nbsp;&shy; &NotEqualTilde;<![CDATA[hidden]]><br/>next</p></div></section>
<ul><li>First<ol><li>Nested</li></ol></li><li>Last</li></ul>
<p>before<script>hidden</script><noscript>hidden</noscript>after</p>
<span>tail</span></body>
</html>"#;
        let mut streamed = String::new();
        assert!(stream_xhtml(
            html.as_bytes(),
            &mut streamed,
            &mut Vec::new(),
            &mut Vec::new(),
        )
        .unwrap());
        let mut fallback = String::new();
        extract_html_fallback(html.as_bytes(), &mut fallback).unwrap();
        assert_eq!(streamed, fallback);
    }

    #[test]
    fn defers_html_tree_repair_to_kuchiki() {
        for body in [
            "<p>start<div>automatically closes the paragraph</div></p>",
            "<p>start<br>end",
            "<p>start &copy without a semicolon</p>",
            "<p>start &#128;</p>",
            "<p>start<span/>end</p>",
            "<a>start<a>nested</a>end</a>",
            "<table>outside<tr><td>inside</td></tr></table>",
            "<svg><foreignObject><p>foreign content</p></foreignObject></svg>",
            "<p>a<![CDATA[early>visible]]>b</p>",
        ] {
            let html = format!("<html><body>{body}</body></html>");
            assert!(
                !stream_xhtml(
                    html.as_bytes(),
                    &mut String::new(),
                    &mut Vec::new(),
                    &mut Vec::new(),
                )
                .unwrap(),
                "{body}"
            );
        }
    }
}
