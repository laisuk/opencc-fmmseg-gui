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
//   ZIP       = private office_converter::zip module (see VENDOR_NOTES.md)
//   thiserror = "2"

use std::fs::File;
use std::io::{self, BufReader, Read, Seek};
use std::path::Path;

use crate::office_converter::zip::ZipArchive;
use quick_xml::events::Event;
use quick_xml::{Reader, XmlVersion};
use thiserror::Error;
#[derive(Debug, Error)]
pub enum OdtError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] crate::office_converter::zip::result::ZipError),

    #[error("Invalid ODT: {0}")]
    Invalid(String),

    #[error("XML error: {0}")]
    Xml(String),
}

/// Extract all user-visible text from an ODT file (content.xml).
/// - Paragraphs/headings end with '\n'
/// - Table rows are tab-separated, ending with '\n'
pub fn extract_odt_all_text(odt_path: impl AsRef<Path>) -> Result<String, OdtError> {
    extract_odt_all_text_from_reader(File::open(odt_path)?)
}

/// Extract text from a seekable source containing an ODT archive.
pub fn extract_odt_all_text_from_reader(source: impl Read + Seek) -> Result<String, OdtError> {
    let mut zip = ZipArchive::new(source)?;

    let entry = zip
        .by_name("content.xml")
        .map_err(|_| OdtError::Invalid("Invalid ODT: missing content.xml".into()))?;

    extract_odf_content_xml(entry)
}

fn extract_odf_content_xml(xml: impl Read) -> Result<String, OdtError> {
    let mut r = Reader::from_reader(BufReader::new(xml));
    r.config_mut().trim_text(false);

    let mut out = String::with_capacity(64 * 1024);

    let mut in_row = false;
    let mut in_cell = false;

    let mut row = String::new();
    let mut cell = String::new();
    let mut row_has_cell = false;

    let mut in_paragraph_like = false;

    let mut buf = Vec::<u8>::new();

    loop {
        let event = match r.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(event) => event,
            Err(quick_xml::Error::Io(e)) => return Err(io::Error::new(e.kind(), e).into()),
            Err(e) => return Err(OdtError::Xml(e.to_string())),
        };

        let end = match &event {
            Event::Start(e) | Event::Empty(e) => {
                let name = e.name();

                match name.as_ref() {
                    "table:table-row" => {
                        in_row = true;
                        row.clear();
                        row_has_cell = false;
                    }

                    "table:table-cell" if in_row => {
                        in_cell = true;
                        cell.clear();
                    }

                    "text:p" | "text:h" => {
                        in_paragraph_like = true;
                    }

                    "text:tab" => {
                        current_target(in_cell, &mut cell, &mut out).push('\t');
                    }

                    "text:line-break" => {
                        current_target(in_cell, &mut cell, &mut out).push('\n');
                    }

                    "text:s" => {
                        let count = e
                            .attributes()
                            .with_checks(false)
                            .map_while(Result::ok)
                            .find(|attr| attr.key.local_name().as_ref() == "c")
                            .and_then(|attr| attr.normalized_value(XmlVersion::Implicit1_0).ok())
                            .and_then(|value| value.trim().parse::<usize>().ok())
                            .unwrap_or(1)
                            .max(1);
                        current_target(in_cell, &mut cell, &mut out)
                            .extend(std::iter::repeat_n(' ', count));
                    }

                    _ => {}
                }
                matches!(event, Event::Empty(_)).then_some(name)
            }

            Event::Text(t) => {
                let text = t.xml10_content();

                if !text.is_empty() {
                    current_target(in_cell, &mut cell, &mut out).push_str(&text);
                }
                None
            }

            Event::CData(t) => {
                let text = t.xml10_content();

                if !text.is_empty() {
                    current_target(in_cell, &mut cell, &mut out).push_str(&text);
                }
                None
            }

            Event::End(e) => Some(e.name()),
            // Match the existing extractor, which only emits Text and CData.
            _ => None,
        };

        if let Some(name) = end {
            match name.as_ref() {
                "text:p" | "text:h" if in_paragraph_like => {
                    current_target(in_cell, &mut cell, &mut out).push('\n');
                    in_paragraph_like = false;
                }

                "table:table-cell" if in_cell => {
                    if in_row {
                        if row_has_cell {
                            row.push('\t');
                        }
                        row.push_str(cell.trim_end_matches(['\n', '\r']));
                        row_has_cell = true;
                    }
                    in_cell = false;
                }

                "table:table-row" if in_row => {
                    out.push_str(&row);
                    out.push('\n');
                    in_row = false;
                }

                _ => {}
            }
        }

        buf.clear();
    }

    Ok(out)
}

fn current_target<'a>(in_cell: bool, cell: &'a mut String, out: &'a mut String) -> &'a mut String {
    if in_cell {
        cell
    } else {
        out
    }
}
