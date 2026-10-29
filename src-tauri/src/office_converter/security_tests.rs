// Included in converter::tests to exercise the private converter and temp lifecycle.
#[test]
fn all_document_formats_rebuild_and_validate_with_raw_assets() {
    for (format, part) in [
        ("docx", "word/document.xml"),
        ("xlsx", "xl/sharedStrings.xml"),
        ("pptx", "ppt/slides/slide1.xml"),
        ("odt", "content.xml"),
        ("ods", "content.xml"),
        ("odp", "content.xml"),
        ("epub", "OEBPS/chapter.xhtml"),
    ] {
        let mut entries = vec![
            (part, "<t>汉语</t>".as_bytes()),
            ("assets/image.bin", &b"inert binary\0data"[..]),
        ];
        if format == "epub" {
            entries.push(("mimetype", b"application/epub+zip"));
        }
        let input = make_zip(&entries);
        let converter = TextConverter::new(|text: &str| text.replace("汉语", "漢語"));
        let (output, count) =
            OfficeConverter::convert_bytes(&input, format, true, &converter).unwrap();
        assert_eq!(count, 1, "{format}");
        assert_eq!(read_zip_entry(&output, part), "<t>漢語</t>");
        assert_eq!(
            read_zip_entry(&output, "assets/image.bin"),
            "inert binary\0data"
        );
        let mut rebuilt = ZipArchive::new(Cursor::new(&output)).unwrap();
        for i in 0..rebuilt.len() {
            io::copy(&mut rebuilt.by_index(i).unwrap(), &mut io::sink()).unwrap();
        }
        if format == "epub" {
            let first = rebuilt.by_index(0).unwrap();
            assert_eq!(first.name(), "mimetype");
            assert_eq!(first.compression(), CompressionMethod::Stored);
        }
    }
}

#[test]
fn portable_unsafe_paths_are_omitted_from_reconstructed_packages() {
    let unsafe_names = [
        "../outside",
        "a/../../outside",
        "a\\..\\outside",
        "/rooted",
        "\\rooted",
        "C:/absolute",
        "C:relative",
        "safe:stream",
        "nul\0suffix",
        "//server/share",
    ];
    let mut entries = vec![("word/document.xml", &b"<t>text</t>"[..])];
    for name in &unsafe_names {
        entries.push((name, b"inert"));
    }
    let input = make_zip(&entries);
    let converter = TextConverter::new(|text: &str| text.to_owned());
    let (output, _) = OfficeConverter::convert_bytes(&input, "docx", false, &converter).unwrap();
    let archive = ZipArchive::new(Cursor::new(output)).unwrap();
    assert_eq!(archive.len(), 1);
    for name in unsafe_names {
        assert!(OfficeConverter::is_unsafe_zip_name(name), "{name:?}");
    }
}

#[test]
fn header_mismatch_in_target_or_raw_asset_preserves_output() {
    let dir = TestDirectory::new();
    let input_path = dir.0.join("input.docx");
    let output_path = dir.0.join("output.docx");
    let converter = TextConverter::new(|text: &str| text.to_owned());
    for malformed_name in [
        b"word/document.xml".as_slice(),
        b"assets/image.bin".as_slice(),
    ] {
        let mut input = make_zip(&[
            ("word/document.xml", b"<t>text</t>"),
            ("assets/image.bin", b"inert"),
        ]);
        let name = input
            .windows(malformed_name.len())
            .position(|v| v == malformed_name)
            .unwrap();
        input[name] = b'x';
        assert!(OfficeConverter::convert_bytes(&input, "docx", false, &converter).is_err());
        fs::write(&input_path, &input).unwrap();
        for existing in [false, true] {
            if existing {
                fs::write(&output_path, b"original destination").unwrap();
            }
            assert!(OfficeConverter::convert_path_stream(
                input_path.to_str().unwrap(),
                output_path.to_str().unwrap(),
                "docx",
                false,
                &converter
            )
            .is_err());
            if existing {
                assert_eq!(fs::read(&output_path).unwrap(), b"original destination");
                fs::remove_file(&output_path).unwrap();
            } else {
                assert!(!output_path.exists());
            }
            dir.assert_entries(1);
        }
    }
}

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        use std::collections::hash_map::RandomState;
        use std::hash::BuildHasher;
        loop {
            let path = std::env::temp_dir().join(format!(
                "opencc-gui-zip-test-{:016x}",
                RandomState::new().hash_one(std::process::id())
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create test directory: {error}"),
            }
        }
    }

    fn assert_entries(&self, expected: usize) {
        assert_eq!(fs::read_dir(&self.0).unwrap().count(), expected);
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn detection_and_extraction_reject_inconsistent_headers() {
    let dir = TestDirectory::new();
    for (format, part) in [
        ("docx", "word/document.xml"),
        ("odt", "content.xml"),
        ("epub", "META-INF/container.xml"),
    ] {
        let path = dir.0.join(format!("input.{format}"));
        let mut input = make_zip(&[(part, b"<t>text</t>"), ("[Content_Types].xml", b"<Types/>")]);
        fs::write(&path, &input).unwrap();
        let detect = |path: &Path| match format {
            "docx" => crate::utils::is_docx(path),
            "odt" => crate::utils::is_odt(path),
            "epub" => crate::utils::is_epub(path),
            _ => unreachable!(),
        };
        assert!(detect(&path), "valid headers for {format}");
        let offset = input
            .windows(part.len())
            .position(|v| v == part.as_bytes())
            .unwrap();
        input[offset] = b'x';
        fs::write(&path, &input).unwrap();
        assert!(!detect(&path), "inconsistent headers for {format}");
        match format {
            "docx" => assert!(matches!(
                crate::open_xml_helper::extract_docx_all_text(&path, false, true),
                Err(crate::open_xml_helper::OpenXmlError::Zip(_))
            )),
            "odt" => assert!(matches!(
                crate::open_doc_helper::extract_odt_all_text(&path),
                Err(crate::open_doc_helper::OdtError::Zip(_))
            )),
            "epub" => assert!(matches!(
                crate::epub_helper::extract_epub_all_text(&path, Default::default()),
                Err(crate::epub_helper::EpubError::Zip(_))
            )),
            _ => unreachable!(),
        }
    }
}
