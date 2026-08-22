// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod cjk_text_normalize;
mod epub_helper;
mod office_converter;
mod open_doc_helper;
mod open_xml_helper;
mod utils;

use crate::cjk_text_normalize::DialogQuoteValidationResult;
use crate::epub_helper::ExtractOptions;
use crate::office_converter::OfficeConverter;
use opencc_fmmseg::{DetofuLevel, OpenCC};
use pdfium_helper::{
    extract_pdf_pages_with_callback_pdfium, reflow_cjk_paragraphs_with_heading_regex, PdfiumLibrary,
};
use regex::Regex;
use rfd::AsyncFileDialog;
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{fs, io};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_clipboard_manager::ClipboardExt;

pub struct AppState {
    opencc: Arc<OpenCC>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            opencc: Arc::new(OpenCC::new()),
        }
    }
}

#[derive(Serialize, Clone)]
struct BatchProgress {
    index: usize,
    total: usize,
    input: String,
    output: String,
    ok: bool,
    message: String,
    progress: String, // ✅ short status text for lblStatusBar
}

// #[derive(serde::Serialize, Clone)]
// pub struct OpenProgress {
//     pub stage: String, // "start" | "extract" | "reflow" | "convert" | "done" | "error"
//     pub path: String,
//     pub ok: bool,
//     pub message: String,  // short label
//     pub progress: String, // the fancy line from format_progress_with_prefix(...)
// }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // .plugin(tauri_plugin_dialog::init())
        // .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_app_version,
            paste_text2,
            copy_text2,
            convert_text,
            open_file,
            save_file,
            zho_check,
            normalize_compat,
            normalize_dialog_quotes,
            validate_dialog_quotes,
            detofu,
            read_text_file,
            reflow_text,
            open_path_to_editor,
            pick_paths_batch,
            pick_output_dir,
            run_batch_convert,
            open_output_dir,
            open_in_file_manager,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn paste_text2(app: AppHandle) -> String {
    app.clipboard().read_text().unwrap_or_default()
}

#[tauri::command]
fn copy_text2(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}

#[tauri::command]
fn convert_text(
    state: State<'_, AppState>,
    text: String,
    config: String,
    punctuation: bool,
) -> String {
    state.opencc.convert(&text, &config, punctuation)
}

#[tauri::command]
fn zho_check(state: State<'_, AppState>, text: String) -> i32 {
    state.opencc.zho_check(&text)
}

#[tauri::command]
fn normalize_compat(state: State<'_, AppState>, text: String, extended: bool) -> String {
    if extended {
        state.opencc.normalize_compat_extended(&text)
    } else {
        state.opencc.normalize_compat(&text)
    }
}

#[tauri::command]
fn normalize_dialog_quotes(text: String) -> String {
    cjk_text_normalize::normalize_cjk_text_dialog_quotes(&text, true)
}

#[tauri::command]
fn validate_dialog_quotes(text: String) -> DialogQuoteValidationResult {
    let result = cjk_text_normalize::validate_cjk_text_dialog_quotes(&text);
    DialogQuoteValidationResult::from(result)
}

#[tauri::command]
fn detofu(state: State<'_, AppState>, text: String, level: String) -> Result<String, String> {
    let level = match level.as_str() {
        "ExtB" => DetofuLevel::ExtB,
        "ExtC" => DetofuLevel::ExtC,
        "ExtD" => DetofuLevel::ExtD,
        "ExtE" => DetofuLevel::ExtE,
        "ExtF" => DetofuLevel::ExtF,
        "ExtG" => DetofuLevel::ExtG,
        "ExtH" => DetofuLevel::ExtH,
        "ExtI" => DetofuLevel::ExtI,
        _ => return Err(format!("Invalid DeToFu level: {level}")),
    };

    Ok(state.opencc.detofu(&text, level))
}

// ----- Open File to Editor ------

// fn emit_open_progress(app: &AppHandle, p: OpenProgress) {
//     if let Err(e) = app.emit_to("main", "open-progress", p) {
//         eprintln!("emit(open-progress) failed: {e}");
//     }
// }

fn emit_open_as_batch(app: &AppHandle, p: BatchProgress) {
    if let Err(e) = app.emit_to("main", "open-progress", p) {
        eprintln!("emit(open-progress) failed: {e}");
    }
}

#[tauri::command]
async fn open_file(
    app: AppHandle,
    state: State<'_, AppState>,
    is_reflow: bool,
    page_header: bool,
    compact: bool,
    config: String,
    punctuation: bool,
    custom_heading_regex: Option<String>, // NEW
) -> Result<(String, String), String> {
    let Some(file_handle) = AsyncFileDialog::new()
        .add_filter("Text Files", &["txt", "md"])
        .add_filter("Subtitles Files", &["srt", "vtt", "ass", "ttml2", "xml"])
        .add_filter("Word Documents", &["docx", "odt"])
        .add_filter("Epub Files", &["epub"])
        .add_filter("PDF Files", &["pdf"])
        .add_filter("All Files", &["*.*"])
        .pick_file()
        .await
    else {
        return Ok((String::new(), String::new()));
    };

    let path = file_handle.path().display().to_string();

    // Delegate all logic (PDF/TXT, emits, BOM strip, spawn_blocking) here:
    open_path_to_editor(
        app,
        state,
        path,
        is_reflow,
        page_header,
        compact,
        config,
        punctuation,
        custom_heading_regex,
    )
    .await
}

#[tauri::command]
fn reflow_text(
    text: String,
    page_header: bool,
    compact: bool,
    custom_heading_regex: Option<String>,
) -> Result<String, String> {
    let heading_re = compile_optional_regex(custom_heading_regex.as_deref())?;

    Ok(reflow_cjk_paragraphs_with_heading_regex(
        &text,
        page_header,
        compact,
        heading_re.as_ref(),
    ))
}

fn compile_optional_regex(pattern: Option<&str>) -> Result<Option<Regex>, String> {
    match pattern.map(str::trim) {
        Some("") | None => Ok(None),
        Some(p) => Regex::new(p)
            .map(Some)
            .map_err(|e| format!("Invalid regex: {e}")),
    }
}

// small inner helpers (monomorphized, no heap, no extra module needed)
fn emit_start(app: &AppHandle) {
    emit_open_as_batch(
        app,
        BatchProgress {
            index: 0,
            total: 1,
            input: String::new(), // marker
            output: String::new(),
            ok: true,
            message: "Open start".into(),
            progress: "[1/1] starting...".into(),
        },
    );
}

fn emit_done(app: &AppHandle, input_path: &str) {
    emit_open_as_batch(
        app,
        BatchProgress {
            index: 1,
            total: 1,
            input: input_path.to_string(),
            output: "Editor".into(),
            ok: true,
            message: "opened".into(),
            progress: "[1/1] ✅ done".into(),
        },
    );
}

#[tauri::command]
async fn open_path_to_editor(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    is_reflow: bool,
    page_header: bool,
    compact: bool,
    config: String,
    punctuation: bool,
    custom_heading_regex: Option<String>, // NEW
) -> Result<(String, String), String> {
    let path_buf = PathBuf::from(&path);
    let path_str = path_buf.display().to_string();

    if !path_buf.is_file() {
        return Err(format!("Invalid file path: {}", path_str));
    }

    let ext = path_buf
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    // -------------------- PDF --------------------
    if ext == "pdf" {
        let opencc_arc = state.opencc.clone();
        let app2 = app.clone();
        let path2 = path_str.clone();
        let cfg2 = config.clone();

        let text = tauri::async_runtime::spawn_blocking(move || {
            open_pdf_extract_text_with_progress(
                &app2,
                &opencc_arc,
                &path2,
                is_reflow,
                page_header,
                compact,
                &cfg2,
                punctuation,
                false, // do_convert on open
                custom_heading_regex.as_deref(),
            )
        })
        .await
        .map_err(|e| e.to_string())??;

        return Ok((path_str, text));
    }

    // -------------------- EPUB --------------------
    // Fast gate by extension; robust confirm by container.xml
    let looks_epub = ext == "epub" || utils::is_epub(&path_buf);
    if looks_epub {
        emit_start(&app);

        let app2 = app.clone();
        let path2 = path_buf.clone(); // move into blocking thread

        let text = tauri::async_runtime::spawn_blocking(move || {
            // optional: you can emit a “parsing epub...” event here too if you want
            let opts = ExtractOptions {
                include_part_headings: false, // disable markers
                normalize_newlines: true,
                skip_nav_documents: true,
            };

            // returns Result<String, EpubError>
            epub_helper::extract_epub_all_text(&path2, opts)
                .map_err(|e| format!("extract epub {}: {}", path2.display(), e))
        })
        .await
        .map_err(|e| e.to_string())??;

        emit_done(&app2, &path_str);

        return Ok((path_str, text));
    }

    // -------------------- DOCX --------------------
    // Fast gate by extension; robust confirm by inspecting zip entries
    let looks_docx = ext == "docx" || utils::is_docx(&path_buf);
    if looks_docx {
        emit_start(&app);

        let app2 = app.clone();
        let path2 = path_buf.clone();

        let text = tauri::async_runtime::spawn_blocking(move || {
            // match your Java defaults:
            // include_part_headings: false (no "=== word/document.xml ===")
            // normalize_newlines: true
            open_xml_helper::extract_docx_all_text(&path2, false, true)
                .map_err(|e| format!("extract docx {}: {}", path2.display(), e))
        })
        .await
        .map_err(|e| e.to_string())??;

        emit_done(&app2, &path_str);
        return Ok((path_str, text));
    }

    // -------------------- ODT --------------------
    // Fast gate by extension; robust confirm by inspecting zip entries
    let looks_odt = ext == "odt" || utils::is_odt(&path_buf);
    if looks_odt {
        emit_start(&app);

        let app2 = app.clone();
        let path2 = path_buf.clone();

        let text = tauri::async_runtime::spawn_blocking(move || {
            // match your Java defaults:
            // include_part_headings: false (no "=== word/document.xml ===")
            // normalize_newlines: true
            open_doc_helper::extract_odt_all_text(&path2)
                .map_err(|e| format!("extract odt {}: {}", path2.display(), e))
        })
        .await
        .map_err(|e| e.to_string())??;

        emit_done(&app2, &path_str);
        return Ok((path_str, text));
    }

    // -------------------- Plain text / others --------------------
    emit_start(&app);

    let data = fs::read(&path_buf).map_err(|e| format!("read {}: {}", path_str, e))?;

    let mut contents = String::from_utf8(data)
        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).to_string());

    if contents.starts_with('\u{FEFF}') {
        contents.remove(0);
    }

    emit_done(&app, &path_str);

    Ok((path_str, contents))
}

fn open_pdf_extract_text_with_progress(
    app: &AppHandle,
    opencc_arc: &Arc<OpenCC>,
    input_path: &str,
    is_reflow: bool,
    page_header: bool,
    compact: bool,
    config: &str,
    punctuation: bool,
    do_convert: bool,
    custom_heading_regex: Option<&str>, // NEW (borrow, no clone)
) -> Result<String, String> {
    let path_display = input_path.to_string();

    // ---- marker: open start ----
    emit_start(&app);

    // ---- ensure pdfium is loaded once per process ----
    let (pdfium, _lib_path) = PdfiumLibrary::global_with_fallbacks()
        .map_err(|e| format!("[1/1] pdfium load failed: {e}"))?;

    // ---- extract pages ----
    let input_norm = normalize_input_path_for_os(input_path);

    let mut pages: Vec<String> = Vec::new();
    let mut last_emit_page: i32 = 0;
    let mut block: i32 = 1;

    extract_pdf_pages_with_callback_pdfium(
        pdfium,
        &input_norm,
        page_header,
        |page, total_pages, text| {
            pages.push(text.to_owned());

            if page == 1 {
                block = progress_block(total_pages);
            }

            let should_emit = page == 1 || page == total_pages || (page % block == 0);
            if should_emit && page != last_emit_page {
                last_emit_page = page;

                let progress_line = format_progress_with_prefix(1, 1, page, total_pages, text);

                emit_open_as_batch(
                    app,
                    BatchProgress {
                        index: 1,
                        total: 1,
                        input: path_display.clone(),
                        output: String::new(),
                        ok: true,
                        message: "pdf extracting".into(),
                        progress: progress_line,
                    },
                );
            }
        },
    )
    .map_err(|e| format!("[1/1] pdf extract failed: {e}"))?;

    let mut extracted = pages.concat();

    // ---- reflow (optional) ----
    if is_reflow {
        emit_open_as_batch(
            app,
            BatchProgress {
                index: 1,
                total: 1,
                input: path_display.clone(),
                output: "Editor".into(),
                ok: true,
                message: "reflowing".into(),
                progress: "[1/1] 🧩 reflow...".into(),
            },
        );

        let heading_regex = compile_optional_regex(custom_heading_regex.as_deref())?;

        extracted = reflow_cjk_paragraphs_with_heading_regex(
            &extracted,
            page_header,
            compact,
            heading_regex.as_ref(),
        );
    }

    // ---- convert (optional) ----
    if do_convert {
        emit_open_as_batch(
            app,
            BatchProgress {
                index: 1,
                total: 1,
                input: path_display.clone(),
                output: "Editor".into(),
                ok: true,
                message: "converting".into(),
                progress: format!("[1/1] 🔁 opencc ({}) ...", config),
            },
        );

        extracted = opencc_arc.convert(&extracted, config, punctuation);
    }

    // ---- done ----
    emit_open_as_batch(
        app,
        BatchProgress {
            index: 1,
            total: 1,
            input: path_display,
            output: "Editor".into(),
            ok: true,
            message: "opened".into(),
            progress: "[1/1] ✅ done".into(),
        },
    );

    Ok(extracted)
}

#[tauri::command]
async fn save_file(content: String) -> Result<String, String> {
    let Some(file_handle) = AsyncFileDialog::new()
        .add_filter("Text Files", &["txt"])
        .set_file_name("File.txt")
        .save_file()
        .await
    else {
        return Err("No file selected".to_string());
    };

    file_handle
        .write(content.as_bytes())
        .await
        .map_err(|e| format!("Failed to save file: {e}"))?;

    Ok(file_handle.path().display().to_string())
}

#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    use std::{fs, path::PathBuf};

    let path_buf = PathBuf::from(&path);
    if !path_buf.is_file() {
        return Err(format!("Invalid file path or not a file: {path}"));
    }

    let data = fs::read(&path_buf).map_err(|e| format!("read {path}: {e}"))?;

    let mut text = String::from_utf8(data)
        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).to_string());

    // strip UTF-8 BOM if present (fixes the red dot)
    if text.starts_with('\u{FEFF}') {
        text.remove(0);
    }

    Ok(text)
}

#[tauri::command]
async fn pick_paths_batch() -> Vec<String> {
    let Some(handles) = AsyncFileDialog::new()
        .add_filter("Text Files", &["txt", "md"])
        .add_filter("Subtitles Files", &["srt", "vtt", "ass", "ttml2", "xml"])
        .add_filter(
            "Office Documents",
            &["docx", "xlsx", "pptx", "odt", "ods", "odp"],
        )
        .add_filter("EPUB Files", &["epub"])
        .add_filter("PDF Files", &["pdf"])
        .add_filter(
            "Supported Files",
            &[
                "txt", "md", "srt", "vtt", "ass", "docx", "xlsx", "pptx", "odt", "ods", "odp",
                "epub", "pdf",
            ],
        )
        .add_filter("ALL Files", &["*.*"])
        .pick_files()
        .await
    else {
        return Vec::new();
    };

    handles
        .into_iter()
        .map(|h| h.path().display().to_string())
        .collect()
}

#[tauri::command]
async fn pick_output_dir() -> String {
    let Some(dir) = AsyncFileDialog::new().pick_folder().await else {
        return String::new();
    };
    dir.path().display().to_string()
}

// Batch Conversion

// --- helpers ---
fn office_extensions() -> HashSet<&'static str> {
    ["docx", "xlsx", "pptx", "odt", "ods", "odp", "epub"].into()
}

/// Return lowercase extension (no dot), e.g. "docx"
fn lower_ext(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
}

/// Standardized output rule:
///   {stem}_{config}.{ext}
///   (if no ext) {stem}_{config}
fn make_output_path(input_path: &str, out_dir: &Path, config: &str) -> Result<PathBuf, String> {
    let p = Path::new(input_path);

    let stem = p
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("Invalid filename: {input_path}"))?;

    let ext = p.extension().and_then(|e| e.to_str());

    let filename = match ext {
        Some(e) => format!("{stem}_{config}.{e}"),
        None => format!("{stem}_{config}"),
    };

    Ok(out_dir.join(filename))
}

fn convert_filename_only(
    opencc: &OpenCC,
    input_path: &str,
    config: &str,
    punctuation: bool,
) -> Result<String, String> {
    let p = Path::new(input_path);

    // If we can't get a filename, just return original.
    let file_name = match p.file_name().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return Ok(input_path.to_string()),
    };

    // Split into stem + ext (ext includes the dot, e.g. ".pdf")
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or(file_name);
    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");

    // Convert only the stem
    let stem_converted = opencc.convert(stem, config, punctuation);

    // Rebuild filename with original extension
    let new_file_name = if ext.is_empty() {
        stem_converted
    } else {
        format!("{stem_converted}.{ext}")
    };

    // Rebuild full path: same parent, new filename
    let new_path: PathBuf = match p.parent() {
        Some(parent) => parent.join(new_file_name),
        None => PathBuf::from(new_file_name),
    };

    Ok(new_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn run_batch_convert(
    app: AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
    output_dir: String,
    config: String,
    punctuation: bool,
    convert_filename: bool,
    overwrite_output: bool,
    custom_heading_regex: Option<String>, // NEW
) -> Result<(), String> {
    let opencc_arc = state.opencc.clone();

    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        let out_dir = Path::new(&output_dir);
        fs::create_dir_all(out_dir)
            .map_err(|e| format!("create_dir_all {}: {}", out_dir.display(), e))?;

        let total = paths.len();
        let office_extensions = office_extensions();
        let mut converted_count = 0usize;
        let mut error_skip_count = 0usize;
        let mut existing_skip_count = 0usize;

        app.emit_to(
            "main",
            "batch-progress",
            BatchProgress {
                index: 0,
                total,
                input: String::new(),
                output: output_dir.clone(),
                ok: true,
                message: "Batch start".into(),
                progress: format!("[0/{total}] starting..."),
            },
        )
        .map_err(|e| format!("emit(start) failed: {e}"))?;

        let heading_regex = match custom_heading_regex
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            Some(pattern) => match Regex::new(pattern) {
                Ok(re) => Some(re),
                Err(err) => {
                    app.emit_to(
                        "main",
                        "batch-progress",
                        BatchProgress {
                            index: 0,
                            total,
                            input: String::new(),
                            output: output_dir.clone(),
                            ok: true,
                            message: format!("Custom heading regex rejected by Rust: {err}"),
                            progress: format!("[0/{total}] warning"),
                        },
                    )
                    .map_err(|e| format!("emit(regex warning) failed: {e}"))?;

                    None
                }
            },
            None => None,
        };

        for (i, path) in paths.into_iter().enumerate() {
            let idx = i + 1;

            let ext_lower = lower_ext(&path);
            let is_pdf = ext_lower.as_deref() == Some("pdf");
            let is_office = ext_lower
                .as_deref()
                .map(|e| office_extensions.contains(e))
                .unwrap_or(false);

            // compute path used for output naming
            let path_for_output = if convert_filename {
                convert_filename_only(&*opencc_arc, &path, &config, punctuation)
                    .map_err(|e| format!("[{idx}/{total}] convert_filename failed: {e}"))?
            } else {
                path.clone()
            };

            // output path
            let output_path = if is_pdf {
                make_output_path_pdf(&path_for_output, out_dir, &config)
                    .map_err(|e| format!("[{idx}/{total}] {e}"))?
            } else {
                make_output_path(&path_for_output, out_dir, &config)
                    .map_err(|e| format!("[{idx}/{total}] {e}"))?
            };

            // ---- output exists handling ----
            if output_path.exists() {
                if !overwrite_output {
                    app.emit_to(
                        "main",
                        "batch-progress",
                        BatchProgress {
                            index: idx,
                            total,
                            input: path.clone(),
                            output: output_path.display().to_string(),
                            ok: true,
                            message: "skipped (output exists)".into(),
                            progress: format!("[{idx}/{total}] ⏭ skipped (exists)"),
                        },
                    )
                    .map_err(|e| format!("emit(skip {idx}/{total}) failed: {e}"))?;

                    existing_skip_count += 1;
                    continue;
                }

                // ✅ overwrite enabled
                fs::remove_file(&output_path).map_err(|e| {
                    format!(
                        "[{idx}/{total}] cannot overwrite {}: {e}",
                        output_path.display()
                    )
                })?;

                // optional: emit overwrite info
                app.emit_to(
                    "main",
                    "batch-progress",
                    BatchProgress {
                        index: idx,
                        total,
                        input: path.clone(),
                        output: output_path.display().to_string(),
                        ok: true,
                        message: "overwriting existing file".into(),
                        progress: format!("[{idx}/{total}] ♻ overwriting..."),
                    },
                )
                .map_err(|e| format!("emit(overwrite {idx}/{total}) failed: {e}"))?;
            }

            // per-file "start" emit (nice for UI)
            app.emit_to(
                "main",
                "batch-progress",
                BatchProgress {
                    index: idx,
                    total,
                    input: path.clone(),
                    output: output_path.display().to_string(),
                    ok: true,
                    message: "processing".into(),
                    progress: format!("[{idx}/{total}] processing..."),
                },
            )
            .map_err(|e| format!("emit(file start {idx}/{total}) failed: {e}"))?;

            let result: Result<(), String> = if is_pdf {
                convert_pdf_to_txt_with_progress(
                    &app,
                    &opencc_arc,
                    idx,
                    total,
                    &path,
                    &output_path,
                    &config,
                    punctuation,
                    heading_regex.as_ref(),
                )
            } else if is_office {
                match ext_lower.clone() {
                    Some(office_format) => OfficeConverter::convert(
                        &path,
                        &output_path.to_string_lossy(),
                        &office_format,
                        &*opencc_arc,
                        &config,
                        punctuation,
                        true, // keep_font
                    )
                    .map_err(|e| format!("[{idx}/{total}] office convert failed: {e}"))
                    .and_then(|r| {
                        if r.success {
                            Ok(())
                        } else {
                            Err(format!(
                                "[{idx}/{total}] office convert failed: {}",
                                r.message
                            ))
                        }
                    }),
                    None => Err(format!("[{idx}/{total}] Cannot infer extension: {path}")),
                }
            } else {
                (|| -> Result<(), String> {
                    // Text path
                    let data =
                        fs::read(&path).map_err(|e| format!("[{idx}/{total}] read {path}: {e}"))?;

                    let contents = String::from_utf8(data)
                        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).to_string());

                    let converted = { opencc_arc.convert(&contents, &config, punctuation) };

                    // keep your current behavior; or switch to write_text_unix_newlines if you want consistent \n
                    fs::write(&output_path, converted).map_err(|e| {
                        format!("[{idx}/{total}] write {}: {e}", output_path.display())
                    })?;

                    Ok(())
                })()
            };

            match result {
                Ok(()) => {
                    converted_count += 1;
                    app.emit_to(
                        "main",
                        "batch-progress",
                        BatchProgress {
                            index: idx,
                            total,
                            input: path,
                            output: output_path.display().to_string(),
                            ok: true,
                            message: if is_pdf {
                                "pdf converted"
                            } else if is_office {
                                "office converted"
                            } else {
                                "converted"
                            }
                            .into(),
                            progress: format!("[{idx}/{total}] ✅ done"),
                        },
                    )
                    .map_err(|e| format!("emit(ok {idx}/{total}) failed: {e}"))?;
                }
                Err(msg) => {
                    error_skip_count += 1;
                    let item_prefix = format!("[{idx}/{total}] ");
                    let reason = msg.strip_prefix(&item_prefix).unwrap_or(&msg);

                    app.emit_to(
                        "main",
                        "batch-progress",
                        BatchProgress {
                            index: idx,
                            total,
                            input: path,
                            output: output_path.display().to_string(),
                            ok: false,
                            message: format!("skipped: {reason}"),
                            progress: format!("[{idx}/{total}] ⚠ skipped — {reason}"),
                        },
                    )
                    .map_err(|e| format!("emit(err {idx}/{total}) failed: {e}"))?;
                }
            }
        }

        app.emit_to(
            "main",
            "batch-progress",
            BatchProgress {
                index: total,
                total,
                input: String::new(),
                output: output_dir,
                ok: true,
                message: format!(
                    "Batch done ({config}): {converted_count} converted, {error_skip_count} skipped with errors, {existing_skip_count} skipped (output exists)"
                ),
                progress: format!(
                    "[{total}/{total}] done — {converted_count} converted, {error_skip_count} errors"
                ),
            },
        )
        .map_err(|e| format!("emit(done) failed: {e}"))?;

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn open_output_dir(app: AppHandle, output_dir: String) -> Result<(), String> {
    let output_dir = output_dir.trim();
    if output_dir.is_empty() {
        return Err("No output directory specified".into());
    }

    let mut path = PathBuf::from(output_dir);

    if path.is_relative() {
        let base = app
            .path()
            .executable_dir()
            .map_err(|e| format!("Failed to resolve executable directory: {e}"))?;
        path = base.join(path);
    }

    if !path.exists() {
        return Err(format!("Output directory not found: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
    }

    opener::open(&path).map_err(|e| format!("Failed to open directory {}: {e}", path.display()))?;

    Ok(())
}

#[tauri::command]
fn open_in_file_manager(path: String) -> Result<(), String> {
    let p = PathBuf::from(path);

    if !p.exists() {
        return Err(format!("Path not found: {}", p.display()));
    }

    // If it's a directory: open it.
    // If it's a file: reveal it in the directory.
    if p.is_dir() {
        open_dir(&p)
    } else {
        reveal_file(&p)
    }
}

fn open_dir(dir: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(dir)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(dir)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(dir)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
}

fn reveal_file(file: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Explorer: select the file
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(file)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        // Finder: reveal the file
        std::process::Command::new("open")
            .arg("-R")
            .arg(file)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        // Many Linux file managers support this via DBus / xdg-open doesn't "select".
        // Fallback: open the parent directory.
        let parent = file.parent().ok_or("No parent directory")?;
        return open_dir(parent);
    }
}

// ------ PDF Helpers ------
// Write UTF-8 text using Unix newlines (`\n`) on all platforms
fn write_text_unix_newlines<P: AsRef<Path>>(path: P, s: &str) -> io::Result<()> {
    let normalized = s.replace("\r\n", "\n").replace('\r', "\n");
    fs::write(path, normalized.as_bytes())
}

// Output: <out_dir>/<stem>_<config>.txt   (for PDF always txt)
fn make_output_path_pdf(input: &str, out_dir: &Path, config: &str) -> Result<PathBuf, String> {
    let p = Path::new(input);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("input");
    Ok(out_dir.join(format!("{stem}_{config}.txt")))
}

// Optional: normalize for Windows separators (matches your CLI)
fn normalize_input_path_for_os(input: &str) -> String {
    if cfg!(windows) {
        input.replace(['/', '\\'], &std::path::MAIN_SEPARATOR.to_string())
    } else {
        input.to_string()
    }
}

fn convert_pdf_to_txt_with_progress(
    app: &AppHandle,
    opencc_arc: &Arc<OpenCC>,
    idx: usize,
    total: usize,
    input_path: &str,
    output_path: &Path,
    config: &str,
    punctuation: bool,
    custom_heading_regex: Option<&Regex>, // NEW (borrow, no clone)
) -> Result<(), String> {
    // ---- ensure pdfium is loaded once per process ----
    let (pdfium, _lib_path) = PdfiumLibrary::global_with_fallbacks()
        .map_err(|e| format!("[{idx}/{total}] pdfium load failed: {e}"))?;

    // ---- extract pages (your throttled progress + map_err) ----
    let input_norm = normalize_input_path_for_os(input_path);

    let mut pages: Vec<String> = Vec::new();
    let mut last_emit_page: i32 = 0;
    let mut block: i32 = 1;

    extract_pdf_pages_with_callback_pdfium(
        pdfium,
        &input_norm,
        false,
        |page, total_pages, text| {
            pages.push(text.to_owned());

            if page == 1 {
                block = progress_block(total_pages);
            }

            let should_emit = page == 1 || page == total_pages || (page % block == 0);
            if should_emit && page != last_emit_page {
                last_emit_page = page;

                let progress_line =
                    format_progress_with_prefix(idx, total, page, total_pages, text);

                if let Err(e) = app.emit_to(
                    "main",
                    "batch-progress",
                    BatchProgress {
                        index: idx,
                        total,
                        input: input_path.to_string(),
                        output: output_path.display().to_string(),
                        ok: true,
                        message: "pdf extracting".into(),
                        progress: progress_line,
                    },
                ) {
                    eprintln!("emit(progress) failed: {e}");
                }
            }
        },
    )
    .map_err(|e| format!("[{idx}/{total}] pdf extract failed: {e}"))?;

    // drop(guard)?  <-- keep it or drop it, both fine; key is pdfium stays in AppState

    let mut extracted = pages.concat();

    extracted =
        reflow_cjk_paragraphs_with_heading_regex(&extracted, false, false, custom_heading_regex);

    let converted = { opencc_arc.convert(&extracted, config, punctuation) };

    write_text_unix_newlines(output_path, &converted)
        .map_err(|e| format!("[{idx}/{total}] write {}: {e}", output_path.display()))?;

    Ok(())
}

/// Return formatted progress line (GUI-safe, no \r printing)
fn progress_block(total_pages: i32) -> i32 {
    if total_pages <= 20 {
        1
    } else if total_pages <= 100 {
        3
    } else if total_pages <= 300 {
        5
    } else {
        (total_pages / 20).max(1) // ~5% intervals
    }
}

fn build_progress_bar_percent(percent: i32, width: usize) -> String {
    let percent = percent.clamp(0, 100);
    let filled = (percent as usize * width) / 100;

    let mut s = String::with_capacity(width * 4 + 2);
    s.push('[');
    for _ in 0..filled {
        s.push_str("🟩");
    }
    for _ in filled..width {
        s.push_str("🟨"); // or ⬜
    }
    s.push(']');
    s
}

pub fn format_progress_with_prefix(
    idx: usize,
    total_files: usize,
    page: i32,
    total_pages: i32,
    text: &str,
) -> String {
    let percent = page * 100 / total_pages.max(1);
    let bar = build_progress_bar_percent(percent, 10);

    format!(
        "[{}/{}] 📄 {} {:3}% ({} chars)",
        idx,
        total_files,
        bar,
        percent,
        text.chars().count()
    )
}
