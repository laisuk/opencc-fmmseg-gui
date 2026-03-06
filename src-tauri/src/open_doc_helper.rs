// open_doc_helper.rs
//
// Rust port of OpenDocumentHelper.java (ODT content.xml text extraction)
//
// - Reads ODT as zip
// - Parses content.xml using quick-xml
// - Extracts:
//   * text:p / text:h => newline on end
//   * tables => TSV rows (tab-separated, newline at row end)
//   * text:tab => '\t'
//   * text:line-break => '\n'
//   * text:s text:c="N" => repeated spaces
//
// Dependencies:
//   quick-xml = "0.36"
//   zip       = "2"
//   thiserror = "2"

use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;
use thiserror::Error;
use zip::ZipArchive;
#[derive(Debug, Error)]
pub enum OdtError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Invalid ODT: {0}")]
    Invalid(String),

    #[error("XML error: {0}")]
    Xml(String),
}

/// Extract all user-visible text from an ODT file (content.xml).
/// - Paragraphs/headings end with '\n'
/// - Table rows are tab-separated, ending with '\n'
pub fn extract_odt_all_text(odt_path: impl AsRef<Path>) -> Result<String, OdtError> {
    let odt_path = odt_path.as_ref();

    let file = File::open(odt_path)?;
    let mut zip = ZipArchive::new(file)?;

    let mut entry = zip
        .by_name("content.xml")
        .map_err(|_| OdtError::Invalid("Invalid ODT: missing content.xml".into()))?;

    let mut xml = Vec::with_capacity(256 * 1024);
    entry.read_to_end(&mut xml)?;

    extract_odf_content_xml_bytes(&xml)
}

fn extract_odf_content_xml_bytes(xml: &[u8]) -> Result<String, OdtError> {
    let mut r = Reader::from_reader(Cursor::new(xml));
    r.config_mut().trim_text(false);
    r.config_mut().expand_empty_elements = true;

    let mut out = String::with_capacity(64 * 1024);

    let mut in_row = false;
    let mut in_cell = false;

    let mut current_row_cells: Option<Vec<String>> = None;
    let mut current_cell: Option<String> = None;

    let mut in_paragraph_like = false;

    let mut buf = Vec::<u8>::new();

    loop {
        match r.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => {
                let e_name = e.name();
                let name_full = e_name.as_ref();

                if eq_name(name_full, b"table", b"table-row") {
                    in_row = true;
                    current_row_cells = Some(Vec::with_capacity(8));
                    buf.clear();
                    continue;
                }

                if eq_name(name_full, b"table", b"table-cell") {
                    if in_row {
                        in_cell = true;
                        current_cell = Some(String::with_capacity(256));
                    }
                    buf.clear();
                    continue;
                }

                if eq_name(name_full, b"text", b"p") || eq_name(name_full, b"text", b"h") {
                    in_paragraph_like = true;
                    buf.clear();
                    continue;
                }

                if eq_name(name_full, b"text", b"tab") {
                    current_target(in_cell, current_cell.as_mut(), &mut out).push('\t');
                    buf.clear();
                    continue;
                }

                if eq_name(name_full, b"text", b"line-break") {
                    current_target(in_cell, current_cell.as_mut(), &mut out).push('\n');
                    buf.clear();
                    continue;
                }

                if eq_name(name_full, b"text", b"s") {
                    let mut count = 1usize;

                    if let Some(c) = attr_any_string_bytes(&e, b"c") {
                        if let Ok(v) = c.trim().parse::<usize>() {
                            count = v.max(1);
                        }
                    }

                    let tgt = current_target(in_cell, current_cell.as_mut(), &mut out);
                    for _ in 0..count {
                        tgt.push(' ');
                    }

                    buf.clear();
                    continue;
                }
            }

            Ok(Event::Text(t)) => {
                let raw = t.decode().map_err(|e| OdtError::Xml(e.to_string()))?;
                let unescaped = quick_xml::escape::unescape(&raw)
                    .map_err(|e| OdtError::Xml(e.to_string()))?;

                if !unescaped.is_empty() {
                    current_target(in_cell, current_cell.as_mut(), &mut out)
                        .push_str(&unescaped);
                }
            }

            Ok(Event::CData(t)) => {
                let s = String::from_utf8_lossy(t.as_ref());
                if !s.is_empty() {
                    current_target(in_cell, current_cell.as_mut(), &mut out).push_str(&s);
                }
            }

            Ok(Event::End(e)) => {
                let e_name = e.name();
                let name_full = e_name.as_ref();

                if eq_name(name_full, b"text", b"p") || eq_name(name_full, b"text", b"h") {
                    if in_paragraph_like {
                        current_target(in_cell, current_cell.as_mut(), &mut out).push('\n');
                        in_paragraph_like = false;
                    }
                    buf.clear();
                    continue;
                }

                if eq_name(name_full, b"table", b"table-cell") {
                    if in_cell {
                        if let (Some(row), Some(cell)) =
                            (current_row_cells.as_mut(), current_cell.take())
                        {
                            row.push(trim_trailing_newlines(&cell));
                        }
                        in_cell = false;
                    }
                    buf.clear();
                    continue;
                }

                if eq_name(name_full, b"table", b"table-row") {
                    if in_row {
                        if let Some(row) = current_row_cells.take() {
                            append_row_as_tsv_line(&mut out, &row);
                        }
                        in_row = false;
                    }
                    buf.clear();
                    continue;
                }
            }

            Err(e) => return Err(OdtError::Xml(e.to_string())),
            _ => {}
        }

        buf.clear();
    }

    Ok(out)
}

fn current_target<'a>(
    in_cell: bool,
    current_cell: Option<&'a mut String>,
    out: &'a mut String,
) -> &'a mut String {
    if in_cell {
        if let Some(cell) = current_cell {
            return cell;
        }
    }
    out
}

fn trim_trailing_newlines(s: &str) -> String {
    let mut end = s.len();
    let bytes = s.as_bytes();
    while end > 0 {
        let b = bytes[end - 1];
        if b == b'\n' || b == b'\r' {
            end -= 1;
        } else {
            break;
        }
    }
    s[..end].to_string()
}

fn append_row_as_tsv_line(out: &mut String, cells: &[String]) {
    for (i, c) in cells.iter().enumerate() {
        if i > 0 {
            out.push('\t');
        }
        out.push_str(c);
    }
    out.push('\n');
}

fn split_prefix_local(full: &[u8]) -> (&[u8], &[u8]) {
    match full.iter().position(|&b| b == b':') {
        Some(pos) => (&full[..pos], &full[pos + 1..]),
        None => (&[][..], full),
    }
}

fn eq_name(full: &[u8], prefix: &[u8], local: &[u8]) -> bool {
    let (p, l) = split_prefix_local(full);
    p == prefix && l == local
}

fn attr_any_string_bytes(
    e: &quick_xml::events::BytesStart,
    key_local: &[u8],
) -> Option<String> {
    for a in e.attributes().with_checks(false) {
        let a = a.ok()?;
        let k = a.key.as_ref();
        let (_, k_local) = split_prefix_local(k);

        if k_local == key_local {
            return a.unescape_value().ok().map(|v| v.into_owned());
        }
    }
    None
}

