use crate::utils;
use quick_xml::events::Event;
use quick_xml::{Reader, XmlVersion};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, Read, Seek};
use std::path::Path;
use thiserror::Error;
use zip::ZipArchive;

#[derive(Debug, Error)]
pub enum OpenXmlError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("XML error: {0}")]
    Xml(String),

    #[allow(dead_code)]
    #[error("Invalid file: {0}")]
    Invalid(String),
}

// ---------------------------- DOCX extraction ----------------------------

pub fn extract_docx_all_text(
    docx_path: impl AsRef<Path>,
    include_part_headings: bool,
    normalize_newlines: bool,
) -> Result<String, OpenXmlError> {
    let file = File::open(docx_path)?;
    let mut zip = ZipArchive::new(file)?;

    // numbering context (simplified / disabled)
    let mut ctx = NumberingContext::load(&mut zip)?;

    // base parts
    let mut parts: Vec<String> = vec![
        "word/document.xml".into(),
        "word/footnotes.xml".into(),
        "word/endnotes.xml".into(),
        "word/comments.xml".into(),
    ];

    // headers/footers
    let mut headers: Vec<String> = Vec::new();
    let mut footers: Vec<String> = Vec::new();

    for i in 0..zip.len() {
        let name = zip.by_index(i)?.name().to_string();
        let lower = name.to_ascii_lowercase();
        if lower.starts_with("word/header") && lower.ends_with(".xml") {
            headers.push(name);
        } else if lower.starts_with("word/footer") && lower.ends_with(".xml") {
            footers.push(name);
        }
    }

    headers.sort_by(|a, b| a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()));
    footers.sort_by(|a, b| a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()));

    parts.extend(headers);
    parts.extend(footers);

    let distinct_parts = distinct_ignore_case_preserve_order(parts);

    let mut out = String::with_capacity(128 * 1024);

    for part in distinct_parts {
        if zip.by_name(&part).is_err() {
            continue;
        }

        if include_part_headings {
            if !out.is_empty() && !ends_with_newline_str(&out) {
                out.push('\n');
            }
            out.push_str("=== ");
            out.push_str(&part);
            out.push_str(" ===\n");
        }

        // read entry bytes
        let mut entry = zip.by_name(&part)?;
        let mut xml = Vec::new();
        entry.read_to_end(&mut xml)?;

        // Reset counters per part (kept for parity)
        ctx.reset_counters_for_part();

        let text = extract_wordprocessingml_text(&xml, &mut ctx)?;
        out.push_str(&text);

        if !ends_with_newline_str(&out) {
            out.push('\n');
        }
    }

    let mut result = out;
    if normalize_newlines {
        result = normalize_newlines_to_lf(&result);
    }
    Ok(result)
}

// ---------------------------- WordprocessingML parsing ----------------------------
// NOTE: we simplify namespace handling: we match local names ignoring prefix.
// In DOCX, that’s almost always `w:*`, so this is safe enough in practice.

fn extract_wordprocessingml_text(
    xml: &[u8],
    ctx: &mut NumberingContext,
) -> Result<String, OpenXmlError> {
    fn set_opt_i32_from_val(
        r: &Reader<&[u8]>,
        in_para: bool,
        e: &quick_xml::events::BytesStart,
        slot: &mut Option<i32>,
    ) {
        if !in_para {
            return;
        }
        if let Some(v) =
            utils::attr_any_string(r, e, b"val").and_then(|s| s.trim().parse::<i32>().ok())
        {
            *slot = Some(v);
        }
    }
    let mut r = Reader::from_reader(xml);
    r.config_mut().trim_text(false);
    r.config_mut().expand_empty_elements = true;

    let mut buf = Vec::new();
    let mut sb = String::with_capacity(64 * 1024);

    // table handling
    let mut in_table = false;
    let mut in_row = false;
    let mut in_cell = false;
    let mut current_row_cells: Vec<String> = Vec::new();
    let mut current_cell = String::with_capacity(256);

    // paragraph/list prefix
    let mut in_paragraph = false;
    let mut para_prefix_emitted = false;
    let mut para_num_id: Option<i32> = None;
    let mut para_ilvl: Option<i32> = None;
    let mut para_style_id: Option<String> = None;

    // notes
    let mut in_footnote = false;
    let mut in_endnote = false;
    let mut skip_this_note = false;

    loop {
        match r.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => {
                let e_name = e.name();
                let name = local_name(e_name.as_ref());

                if name == "footnote" {
                    in_footnote = true;
                    skip_this_note = should_skip_note_element(&r, &e);
                } else if name == "endnote" {
                    in_endnote = true;
                    skip_this_note = should_skip_note_element(&r, &e);
                } else if name == "tbl" {
                    in_table = true;
                } else if name == "tr" {
                    if in_table {
                        in_row = true;
                        current_row_cells.clear();
                    }
                } else if name == "tc" {
                    if in_row {
                        in_cell = true;
                        current_cell.clear();
                    }
                } else if name == "p" {
                    in_paragraph = true;
                    para_prefix_emitted = false;
                    para_num_id = None;
                    para_ilvl = None;
                    para_style_id = None;
                } else if name == "pStyle" {
                    if in_paragraph {
                        if let Some(val) = utils::attr_any_string(&r, &e, b"val") {
                            if !val.is_empty() {
                                para_style_id = Some(val);
                            }
                        }
                    }
                } else if name == "numId" {
                    set_opt_i32_from_val(&r, in_paragraph, &e, &mut para_num_id);
                } else if name == "ilvl" {
                    set_opt_i32_from_val(&r, in_paragraph, &e, &mut para_ilvl);
                } else if name == "t" {
                    if skip_this_note && (in_footnote || in_endnote) {
                        skip_element(&mut r, &mut buf)?;
                    } else {
                        para_prefix_emitted = write_list_prefix(
                            ctx,
                            in_paragraph,
                            para_prefix_emitted,
                            para_num_id,
                            para_ilvl,
                            para_style_id.as_deref(),
                            in_cell,
                            &mut current_cell,
                            &mut sb,
                        );

                        let txt = read_text_until_end(&mut r, b"t", &mut buf)?;
                        current_target(in_cell, &mut current_cell, &mut sb).push_str(&txt);
                    }
                } else if name == "tab" {
                    if !(skip_this_note && (in_footnote || in_endnote)) {
                        para_prefix_emitted = write_list_prefix(
                            ctx,
                            in_paragraph,
                            para_prefix_emitted,
                            para_num_id,
                            para_ilvl,
                            para_style_id.as_deref(),
                            in_cell,
                            &mut current_cell,
                            &mut sb,
                        );
                        current_target(in_cell, &mut current_cell, &mut sb).push('\t');
                    }
                } else if name == "br" || name == "cr" {
                    if !(skip_this_note && (in_footnote || in_endnote)) {
                        para_prefix_emitted = write_list_prefix(
                            ctx,
                            in_paragraph,
                            para_prefix_emitted,
                            para_num_id,
                            para_ilvl,
                            para_style_id.as_deref(),
                            in_cell,
                            &mut current_cell,
                            &mut sb,
                        );
                        current_target(in_cell, &mut current_cell, &mut sb).push('\n');
                    }
                }
            }

            Ok(Event::End(e)) => {
                let e_name = e.name();
                let name = local_name(e_name.as_ref());

                if name == "p" {
                    if !(skip_this_note && (in_footnote || in_endnote)) {
                        current_target(in_cell, &mut current_cell, &mut sb).push('\n');
                    }
                    in_paragraph = false;
                    para_prefix_emitted = false;
                } else if name == "tc" {
                    if in_cell {
                        let cell = trim_trailing_newlines(&current_cell);
                        current_row_cells.push(cell);
                        in_cell = false;
                        current_cell.clear();
                    }
                } else if name == "tr" {
                    if in_row {
                        append_row_as_tsv_line(&mut sb, &current_row_cells);
                        in_row = false;
                        current_row_cells.clear();
                    }
                } else if name == "tbl" {
                    if in_table {
                        if !ends_with_newline_str(&sb) {
                            sb.push('\n');
                        }
                        in_table = false;
                    }
                } else if name == "footnote" {
                    in_footnote = false;
                    skip_this_note = false;
                } else if name == "endnote" {
                    in_endnote = false;
                    skip_this_note = false;
                }
            }

            // Text events: Word text usually comes via <w:t>, but keep other text safe.
            Ok(Event::Text(_)) => {}

            Err(e) => return Err(OpenXmlError::Xml(e.to_string())),
            _ => {}
        }

        buf.clear();
    }

    Ok(sb)
}

// ---------------------------- Note skipping ----------------------------

fn should_skip_note_element(r: &Reader<&[u8]>, e: &quick_xml::events::BytesStart) -> bool {
    // In WordprocessingML: <w:footnote w:type="separator" w:id="0"> ...
    if let Some(t) = utils::attr_any_string(r, e, b"type") {
        let tt = t.trim();
        if tt.eq_ignore_ascii_case("separator") || tt.eq_ignore_ascii_case("continuationSeparator")
        {
            return true;
        }
    }

    if let Some(id) = utils::attr_any_string(r, e, b"id") {
        if let Ok(v) = id.trim().parse::<i32>() {
            if v <= 0 {
                return true;
            }
        }
    }

    false
}

// ---------------------------- Numbering prefix (REAL) ----------------------------

fn write_list_prefix(
    ctx: &mut NumberingContext,
    in_paragraph: bool,
    list_prefix_written: bool,
    para_num_id: Option<i32>,
    para_ilvl: Option<i32>,
    para_style_id: Option<&str>,
    in_cell: bool,
    current_cell: &mut String,
    sb: &mut String,
) -> bool {
    if !in_paragraph || list_prefix_written {
        return list_prefix_written;
    }

    let Some(rn) = ctx.resolve_num(para_num_id, para_ilvl, para_style_id) else {
        return false;
    };

    let prefix = ctx.next_prefix(rn.num_id, rn.ilvl);
    if prefix.is_empty() {
        return false;
    }

    current_target(in_cell, current_cell, sb).push_str(&prefix);
    true
}

#[derive(Clone, Debug, Default)]
struct LevelDef {
    num_fmt: String,   // decimal, lowerLetter, bullet, ...
    lvl_text: String,  // %1) or "" etc
    font_hint: String, // Symbol / Wingdings / Courier New ...
}

pub struct NumberingContext {
    // numId -> abstractNumId
    num_to_abs: HashMap<i32, i32>,
    // (abstractNumId, ilvl) -> LevelDef
    abs_lvl: HashMap<(i32, i32), LevelDef>,
    // styleId -> (numId, ilvl)
    style_num: HashMap<String, (i32, i32)>,
    // numId -> counters[0..9)
    counters: HashMap<i32, [i32; 9]>,
}

impl NumberingContext {
    pub fn load<R: Read + Seek>(zip: &mut ZipArchive<R>) -> Result<Self, OpenXmlError> {
        let mut ctx = Self {
            num_to_abs: HashMap::new(),
            abs_lvl: HashMap::new(),
            style_num: HashMap::new(),
            counters: HashMap::new(),
        };

        ctx.load_numbering(zip)?;
        ctx.load_styles(zip)?;
        Ok(ctx)
    }

    pub fn reset_counters_for_part(&mut self) {
        // same behavior as your C# version
        self.counters.clear();
    }

    pub fn resolve_num(
        &self,
        direct_num_id: Option<i32>,
        direct_ilvl: Option<i32>,
        style_id: Option<&str>,
    ) -> Option<ResolvedNum> {
        // Word behavior: if numId exists but ilvl missing, treat as 0
        if let Some(num_id) = direct_num_id {
            return Some(ResolvedNum {
                num_id,
                ilvl: direct_ilvl.unwrap_or(0),
                source: NumSource::Direct,
            });
        }

        if let Some(sid) = style_id {
            if let Some((num_id, ilvl)) = self.style_num.get(sid).copied() {
                return Some(ResolvedNum {
                    num_id,
                    ilvl,
                    source: NumSource::Style,
                });
            }
        }

        None
    }

    pub fn next_prefix(&mut self, num_id: i32, ilvl: i32) -> String {
        let mut ilvl = ilvl;
        if ilvl < 0 {
            ilvl = 0;
        }
        if ilvl > 8 {
            ilvl = 8;
        }

        let abs = match self.num_to_abs.get(&num_id).copied() {
            Some(v) => v,
            None => return String::new(),
        };

        let def = match self.abs_lvl.get(&(abs, ilvl)).cloned() {
            Some(d) => d,
            None => return String::new(),
        };

        // update counters
        let counters = self.counters.entry(num_id).or_insert([0; 9]);
        let u = ilvl as usize;
        counters[u] += 1;
        for d in (u + 1)..counters.len() {
            counters[d] = 0;
        }

        // bullets
        if def.num_fmt.eq_ignore_ascii_case("bullet") {
            let b = resolve_bullet_glyph(&def.lvl_text, &def.font_hint, ilvl);
            if b.is_empty() {
                return String::new();
            }
            return format!("{b} ");
        }

        // numbering template
        let lvl_text = if def.lvl_text.is_empty() {
            "%1.".to_string()
        } else {
            def.lvl_text
        };

        // substitute %1..%9
        let mut out = lvl_text;
        for n in 1..=9 {
            let idx = (n - 1) as usize;
            let v = counters[idx];
            let ref_fmt = self
                .abs_lvl
                .get(&(abs, idx as i32))
                .map(|d| d.num_fmt.as_str())
                .unwrap_or("decimal");
            let rep = format_counter(v, ref_fmt);
            out = out.replace(&format!("%{n}"), &rep);
        }

        // normalize whitespace (like your C#)
        out = out.replace('\t', " ").replace('\u{00A0}', " ");
        if !out.is_empty() && !out.chars().last().unwrap().is_whitespace() {
            out.push(' ');
        }
        out
    }

    fn load_numbering<R: Read + Seek>(
        &mut self,
        zip: &mut ZipArchive<R>,
    ) -> Result<(), OpenXmlError> {
        let mut entry = match zip.by_name("word/numbering.xml") {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        let mut xml = Vec::new();
        entry.read_to_end(&mut xml)?;

        let mut r = Reader::from_reader(xml.as_slice());
        r.config_mut().trim_text(true);
        r.config_mut().expand_empty_elements = true;

        let mut buf = Vec::new();

        let mut cur_abs: Option<i32> = None;
        let mut cur_ilvl: Option<i32> = None;
        let mut cur_num: Option<i32> = None;

        loop {
            match r.read_event_into(&mut buf) {
                Ok(Event::Eof) => break,

                Ok(Event::Start(e)) => {
                    let e_name = e.name();
                    let name = local_name(e_name.as_ref());

                    match name {
                        "num" => {
                            cur_num = utils::attr_any_string(&r, &e, b"numId")
                                .and_then(|s| s.parse().ok());
                        }
                        "abstractNumId" => {
                            if let Some(num_id) = cur_num {
                                if let Some(abs) = utils::attr_any_string(&r, &e, b"val")
                                    .and_then(|s| s.parse().ok())
                                {
                                    self.num_to_abs.insert(num_id, abs);
                                }
                            }
                        }
                        "abstractNum" => {
                            cur_abs = utils::attr_any_string(&r, &e, b"abstractNumId")
                                .and_then(|s| s.parse().ok());
                        }
                        "lvl" => {
                            cur_ilvl = utils::attr_any_string(&r, &e, b"ilvl")
                                .and_then(|s| s.parse().ok());
                            if let (Some(abs), Some(ilvl)) = (cur_abs, cur_ilvl) {
                                self.abs_lvl.entry((abs, ilvl)).or_default();
                            }
                        }
                        "numFmt" => {
                            if let (Some(abs), Some(ilvl)) = (cur_abs, cur_ilvl) {
                                let v = utils::attr_any_string(&r, &e, b"val").unwrap_or_default();
                                self.abs_lvl.entry((abs, ilvl)).or_default().num_fmt = v;
                            }
                        }
                        "lvlText" => {
                            if let (Some(abs), Some(ilvl)) = (cur_abs, cur_ilvl) {
                                let v = utils::attr_any_string(&r, &e, b"val").unwrap_or_default();
                                self.abs_lvl.entry((abs, ilvl)).or_default().lvl_text = v;
                            }
                        }
                        "rFonts" => {
                            if let (Some(abs), Some(ilvl)) = (cur_abs, cur_ilvl) {
                                // prefer ascii then hAnsi (like your C#)
                                let ascii =
                                    utils::attr_any_string(&r, &e, b"ascii").unwrap_or_default();
                                let hansi =
                                    utils::attr_any_string(&r, &e, b"hAnsi").unwrap_or_default();
                                let font = if !ascii.is_empty() { ascii } else { hansi };
                                if !font.is_empty() {
                                    self.abs_lvl.entry((abs, ilvl)).or_default().font_hint = font;
                                }
                            }
                        }
                        _ => {}
                    }
                }

                Ok(Event::End(e)) => {
                    let e_name = e.name();
                    let name = local_name(e_name.as_ref());
                    match name {
                        "num" => cur_num = None,
                        "abstractNum" => {
                            cur_abs = None;
                            cur_ilvl = None;
                        }
                        "lvl" => cur_ilvl = None,
                        _ => {}
                    }
                }

                Err(e) => return Err(OpenXmlError::Xml(e.to_string())),
                _ => {}
            }

            buf.clear();
        }

        Ok(())
    }

    fn load_styles<R: Read + Seek>(&mut self, zip: &mut ZipArchive<R>) -> Result<(), OpenXmlError> {
        let mut entry = match zip.by_name("word/styles.xml") {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        let mut xml = Vec::new();
        entry.read_to_end(&mut xml)?;

        let mut r = Reader::from_reader(xml.as_slice());
        r.config_mut().trim_text(true);
        r.config_mut().expand_empty_elements = true;

        let mut buf = Vec::new();

        let mut cur_style_id: Option<String> = None;
        let mut style_num_id: Option<i32> = None;
        let mut style_ilvl: Option<i32> = None;

        loop {
            match r.read_event_into(&mut buf) {
                Ok(Event::Eof) => break,

                Ok(Event::Start(e)) => {
                    let e_name = e.name();
                    let name = local_name(e_name.as_ref());
                    match name {
                        "style" => {
                            cur_style_id = utils::attr_any_string(&r, &e, b"styleId");
                            style_num_id = None;
                            style_ilvl = None;
                        }
                        "numId" => {
                            if cur_style_id.is_some() {
                                style_num_id = utils::attr_any_string(&r, &e, b"val")
                                    .and_then(|s| s.parse().ok());
                            }
                        }
                        "ilvl" => {
                            if cur_style_id.is_some() {
                                style_ilvl = utils::attr_any_string(&r, &e, b"val")
                                    .and_then(|s| s.parse().ok());
                            }
                        }
                        _ => {}
                    }
                }

                Ok(Event::End(e)) => {
                    let e_name = e.name();
                    let name = local_name(e_name.as_ref());
                    if name == "style" {
                        if let (Some(sid), Some(num_id), Some(ilvl)) =
                            (cur_style_id.take(), style_num_id, style_ilvl)
                        {
                            self.style_num.insert(sid, (num_id, ilvl));
                        } else {
                            cur_style_id = None;
                        }
                        style_num_id = None;
                        style_ilvl = None;
                    }
                }

                Err(e) => return Err(OpenXmlError::Xml(e.to_string())),
                _ => {}
            }

            buf.clear();
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NumSource {
    Direct,
    Style,
}

#[derive(Clone, Copy, Debug)]
pub struct ResolvedNum {
    pub num_id: i32,
    pub ilvl: i32,
    #[allow(dead_code)]
    pub source: NumSource,
}

// ---------------------------- format helpers ----------------------------

fn format_counter(v: i32, fmt: &str) -> String {
    let v = if v <= 0 { 1 } else { v };
    match fmt {
        "lowerLetter" => to_letters(v, false),
        "upperLetter" => to_letters(v, true),
        "lowerRoman" => roman(v).to_lowercase(),
        "upperRoman" => roman(v),
        "decimalZero" => {
            if v < 10 {
                format!("0{v}")
            } else {
                v.to_string()
            }
        }
        _ => v.to_string(), // decimal + fallback
    }
}

fn to_letters(mut v: i32, upper: bool) -> String {
    // 1->a, 26->z, 27->aa...
    if v <= 0 {
        v = 1;
    }
    let mut s = String::new();
    while v > 0 {
        v -= 1;
        let c = ((v % 26) as u8 + b'a') as char;
        s.insert(0, c);
        v /= 26;
    }
    if upper {
        s.to_uppercase()
    } else {
        s
    }
}

fn roman(mut v: i32) -> String {
    if v <= 0 {
        return "I".into();
    }
    let map = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut s = String::new();
    for (n, sym) in map {
        while v >= n {
            s.push_str(sym);
            v -= n;
        }
    }
    s
}

// ---------------------------- bullet helpers ----------------------------

fn resolve_bullet_glyph(lvl_text: &str, font_hint: &str, ilvl: i32) -> String {
    let t = lvl_text
        .replace('\t', "")
        .replace('\u{00A0}', " ")
        .trim()
        .to_string();
    if t.is_empty() {
        return fallback_bullet(ilvl).to_string();
    }

    let ch = t.chars().next().unwrap();
    if is_good_unicode_bullet(ch) {
        return ch.to_string();
    }

    let font = font_hint.trim();

    if font.eq_ignore_ascii_case("Symbol") {
        // Word default: '' in Symbol font means •
        if ch == '' {
            return "•".into();
        }
        return "•".into();
    }

    if font.to_ascii_lowercase().starts_with("wingdings") {
        return match ch {
            '' => "✓".into(),
            '' => "➤".into(),
            '' => "▪".into(),
            _ => "•".into(),
        };
    }

    if font.eq_ignore_ascii_case("Courier New") {
        if ch == 'o' {
            return "○".into();
        }
    }

    // last resort: if it looks like 'o', it's often an open circle bullet
    if ch == 'o' {
        return "○".into();
    }

    fallback_bullet(ilvl).to_string()
}

fn fallback_bullet(ilvl: i32) -> &'static str {
    // simple 3-cycle
    match (if ilvl < 0 { 0 } else { ilvl }) % 3 {
        0 => "•",
        1 => "○",
        _ => "▪",
    }
}

fn is_good_unicode_bullet(c: char) -> bool {
    matches!(
        c,
        '•' | '●' | '○' | '■' | '▪' | '◆' | '◇' | '✓' | '✔' | '➤' | '➔' | '➢'
    )
}

// ---------------------------- small utils ----------------------------

fn distinct_ignore_case_preserve_order(items: Vec<String>) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<String> = Vec::with_capacity(items.len());
    for s in items {
        let k = s.to_ascii_lowercase();
        if seen.insert(k) {
            out.push(s);
        }
    }
    out
}

fn normalize_newlines_to_lf(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

fn ends_with_newline_str(s: &str) -> bool {
    s.ends_with('\n') || s.ends_with("\r\n")
}

fn trim_trailing_newlines(s: &str) -> String {
    let mut end = s.len();
    let bytes = s.as_bytes();
    while end > 0 && (bytes[end - 1] == b'\n' || bytes[end - 1] == b'\r') {
        end -= 1;
    }
    s[..end].to_string()
}

fn append_row_as_tsv_line(sb: &mut String, cells: &[String]) {
    for (i, c) in cells.iter().enumerate() {
        if i > 0 {
            sb.push('\t');
        }
        sb.push_str(c);
    }
    if !sb.ends_with('\n') {
        sb.push('\n');
    }
}

fn current_target<'a>(
    in_cell: bool,
    current_cell: &'a mut String,
    sb: &'a mut String,
) -> &'a mut String {
    if in_cell {
        current_cell
    } else {
        sb
    }
}

fn local_name(full: &[u8]) -> &str {
    // strip prefix like "w:t" -> "t"
    let s = std::str::from_utf8(full).unwrap_or("");
    match s.rsplit_once(':') {
        Some((_, local)) => local,
        None => s,
    }
}

fn read_text_until_end<R: BufRead>(
    r: &mut Reader<R>,
    end_local: &[u8],
    buf: &mut Vec<u8>,
) -> Result<String, OpenXmlError> {
    let mut out = String::new();

    loop {
        match r.read_event_into(buf) {
            Ok(Event::Text(t)) => {
                let text = t
                    .xml_content(XmlVersion::Implicit1_0)
                    .map_err(|e| OpenXmlError::Xml(e.to_string()))?;
                out.push_str(&text);
            }
            Ok(Event::CData(t)) => out.push_str(&String::from_utf8_lossy(t.as_ref())),
            Ok(Event::End(e)) => {
                let e_name = e.name();
                let name = utils::local_name_bytes(e_name.as_ref());
                if name == end_local {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OpenXmlError::Xml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    Ok(out)
}

fn skip_element<R: BufRead>(r: &mut Reader<R>, buf: &mut Vec<u8>) -> Result<(), OpenXmlError> {
    let mut depth: usize = 1;
    buf.clear();

    while depth > 0 {
        match r.read_event_into(buf) {
            Ok(Event::Start(_)) => depth += 1,
            Ok(Event::End(_)) => depth -= 1,
            Ok(Event::Eof) => break,
            Err(e) => return Err(OpenXmlError::Xml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

#[allow(dead_code)]
fn skip_element_named<R: BufRead>(
    r: &mut Reader<R>,
    start_local: &[u8],
    buf: &mut Vec<u8>,
) -> Result<(), OpenXmlError> {
    let mut depth: usize = 1;
    buf.clear();

    while depth > 0 {
        match r.read_event_into(buf) {
            Ok(Event::Start(_)) => depth += 1,
            Ok(Event::End(e)) => {
                let e_name = e.name();
                let end_name = utils::local_name_bytes(e_name.as_ref());
                depth -= 1;

                if depth == 0 && end_name != start_local {
                    // Optional strict mode:
                    // return Err(OpenXmlError::Xml("skip_element_named: mismatched end tag".into()));
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OpenXmlError::Xml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}
