use opencc_fmmseg::OpenCC;

/// Configuration for the GUI file-conversion pipeline.
///
/// The options are captured once before conversion begins so downstream
/// consumers only need to call `TextConverter::convert(text)`.
#[derive(Clone, Copy)]
pub struct TextConverterOptions<'a> {
    pub config: &'a str,
    pub punctuation: bool,
}

/// Generic text-to-text transformation.
///
/// Deliberately independent of files, Office/EPUB containers, PDF handling,
/// filenames, and GUI state.
pub struct TextConverter<F> {
    convert: F,
}

impl<F> TextConverter<F>
where
    F: Fn(&str) -> String,
{
    #[inline]
    pub fn new(convert: F) -> Self {
        Self { convert }
    }

    #[inline]
    pub fn convert(&self, text: &str) -> String {
        (self.convert)(text)
    }
}

/// Build the standard GUI conversion pipeline.
///
/// Current GUI batch semantics are intentionally preserved:
/// OpenCC conversion + optional punctuation conversion only.
pub fn create_text_converter<'a>(
    opencc: &'a OpenCC,
    options: TextConverterOptions<'a>,
) -> TextConverter<impl Fn(&str) -> String + 'a> {
    TextConverter::new(move |text| opencc.convert(text, options.config, options.punctuation))
}
