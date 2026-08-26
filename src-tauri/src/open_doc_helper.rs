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
//   quick-xml = "0.42"
//   zip       = "2"
//   thiserror = "2"

use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

use crate::utils;
use quick_xml::events::Event;
use quick_xml::{Reader, XmlVersion};
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
                let name = e.name();

                match name.as_ref() {
                    "table:table-row" => {
                        in_row = true;
                        current_row_cells = Some(Vec::with_capacity(8));
                        buf.clear();
                        continue;
                    }

                    "table:table-cell" => {
                        if in_row {
                            in_cell = true;
                            current_cell = Some(String::with_capacity(256));
                        }
                        buf.clear();
                        continue;
                    }

                    "text:p" | "text:h" => {
                        in_paragraph_like = true;
                        buf.clear();
                        continue;
                    }

                    "text:tab" => {
                        current_target(in_cell, current_cell.as_mut(), &mut out).push('\t');
                        buf.clear();
                        continue;
                    }

                    "text:line-break" => {
                        current_target(in_cell, current_cell.as_mut(), &mut out).push('\n');
                        buf.clear();
                        continue;
                    }

                    "text:s" => {
                        let mut count = 1usize;

                        if let Some(c) = utils::attr_any_string(&e, "c") {
                            if let Ok(v) = c.trim().parse::<usize>() {
                                count = v.max(1);
                            }
                        }

                        let target = current_target(in_cell, current_cell.as_mut(), &mut out);
                        for _ in 0..count {
                            target.push(' ');
                        }

                        buf.clear();
                        continue;
                    }

                    _ => {}
                }
            }

            Ok(Event::Text(t)) => {
                let text = t.xml_content(XmlVersion::Implicit1_0);

                if !text.is_empty() {
                    current_target(in_cell, current_cell.as_mut(), &mut out).push_str(&text);
                }
            }

            Ok(Event::CData(t)) => {
                let text = t.xml_content(XmlVersion::Implicit1_0);

                if !text.is_empty() {
                    current_target(in_cell, current_cell.as_mut(), &mut out).push_str(&text);
                }
            }

            Ok(Event::End(e)) => {
                let name = e.name();

                match name.as_ref() {
                    "text:p" | "text:h" => {
                        if in_paragraph_like {
                            current_target(in_cell, current_cell.as_mut(), &mut out).push('\n');
                            in_paragraph_like = false;
                        }
                        buf.clear();
                        continue;
                    }

                    "table:table-cell" => {
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

                    "table:table-row" => {
                        if in_row {
                            if let Some(row) = current_row_cells.take() {
                                append_row_as_tsv_line(&mut out, &row);
                            }
                            in_row = false;
                        }
                        buf.clear();
                        continue;
                    }

                    _ => {}
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
