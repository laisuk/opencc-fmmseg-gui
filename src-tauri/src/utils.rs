use quick_xml::events::BytesStart;
use quick_xml::{Reader, XmlVersion};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

// ---------------------------- Format detection ----------------------------
pub fn is_docx(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    if !path.is_file() {
        return false;
    }
    let name = match path.file_name().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return false,
    };
    if !ends_with_ignore_case(name, ".docx") {
        return false;
    }

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut zip = match ZipArchive::new(file) {
        Ok(z) => z,
        Err(_) => return false,
    };

    let ok1 = zip.by_name("word/document.xml").is_ok();
    let ok2 = zip.by_name("[Content_Types].xml").is_ok();
    ok1 && ok2
}

pub fn is_odt(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    if !path.is_file() {
        return false;
    }
    let name = match path.file_name().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return false,
    };
    if !ends_with_ignore_case(name, ".odt") {
        return false;
    }

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut zip = match ZipArchive::new(file) {
        Ok(z) => z,
        Err(_) => return false,
    };

    // required
    if zip.by_name("content.xml").is_err() {
        return false;
    }

    // optional mimetype verification (the best effort)
    if let Ok(mut mt) = zip.by_name("mimetype") {
        let mut buf = Vec::new();
        if mt.read_to_end(&mut buf).is_ok() {
            let s = String::from_utf8_lossy(&buf);
            let mt = s.trim();
            if mt != "application/vnd.oasis.opendocument.text" {
                return false;
            }
        }
    }

    true
}

pub fn is_epub(path: impl AsRef<Path>) -> bool {
    use std::fs::File;
    use zip::ZipArchive;
    let path = path.as_ref();
    if !path.is_file() {
        return false;
    }

    let name = match path.file_name().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return false,
    };
    if !ends_with_ignore_case(name, ".epub") {
        return false;
    }

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut zip = match ZipArchive::new(file) {
        Ok(z) => z,
        Err(_) => return false,
    };

    // ✅ force the borrowed temporary (ZipFile) to be dropped before returning
    let ok = zip.by_name("META-INF/container.xml").is_ok();
    ok
}

pub fn ends_with_ignore_case(s: &str, suffix: &str) -> bool {
    s.to_lowercase().ends_with(&suffix.to_lowercase())
}

pub fn local_name_bytes(full: &[u8]) -> &[u8] {
    match full.iter().rposition(|&b| b == b':') {
        Some(pos) => &full[pos + 1..],
        None => full,
    }
}

pub fn attr_any_string<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &BytesStart<'_>,
    local: &[u8],
) -> Option<String> {
    for a in e.attributes().with_checks(false) {
        let a = a.ok()?;
        let k_local = local_name_bytes(a.key.as_ref());

        if k_local == local {
            return a
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
                .ok()
                .map(|v| v.into_owned());
        }
    }

    None
}
