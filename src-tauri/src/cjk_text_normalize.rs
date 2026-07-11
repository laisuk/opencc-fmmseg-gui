use serde::Serialize;

/// The result produced by dialog quote validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DialogQuoteValidationResult {
    /// Indicates whether no suspicious dialog quote lines were found.
    pub is_valid: bool,

    /// A short user-facing summary of the validation result.
    pub summary: String,

    /// The suspicious lines found during validation.
    pub suspicious_lines: Vec<DialogQuoteIssue>,
}

impl DialogQuoteValidationResult {
    /// Returns `true` when no suspicious dialog quote lines were found.
    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }
}

/// Describes one suspicious dialog quote line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DialogQuoteIssue {
    /// The one-based source line number.
    pub line_number: usize,

    /// The original source line, including its original indentation.
    pub text: String,
}

/// The currently active quotation-mark family for one logical quote level.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum DialogQuoteFamily {
    /// No quote is currently open.
    #[default]
    None,

    /// Curly quotation marks are active:
    /// `“”` or `‘’`.
    Curly,

    /// Traditional Chinese corner quotation marks are active:
    /// `「」` or `『』`.
    Corner,
}

/// Tracks active outer and inner dialog quotation-mark families while text is
/// normalized.
#[derive(Debug, Default)]
struct DialogQuoteState {
    double_family: DialogQuoteFamily,
    single_family: DialogQuoteFamily,
}

impl DialogQuoteState {
    /// Normalizes one character and updates the current dialog quote state.
    ///
    /// Existing CJK quotation marks are preserved exactly. They only update the
    /// state used to interpret subsequent ASCII quotation marks.
    #[inline]
    fn normalize_char(&mut self, ch: char) -> char {
        match ch {
            // Curly outer quotation marks.
            '“' => {
                self.double_family = DialogQuoteFamily::Curly;
                ch
            }
            '”' => {
                self.double_family = DialogQuoteFamily::None;
                ch
            }

            // Traditional Chinese outer corner quotation marks.
            '「' => {
                self.double_family = DialogQuoteFamily::Corner;
                ch
            }
            '」' => {
                self.double_family = DialogQuoteFamily::None;
                ch
            }

            // ASCII double quote.
            '"' => self.normalize_ascii_double(),

            // Curly inner quotation marks.
            '‘' => {
                self.single_family = DialogQuoteFamily::Curly;
                ch
            }
            '’' => {
                self.single_family = DialogQuoteFamily::None;
                ch
            }

            // Traditional Chinese inner corner quotation marks.
            '『' => {
                self.single_family = DialogQuoteFamily::Corner;
                ch
            }
            '』' => {
                self.single_family = DialogQuoteFamily::None;
                ch
            }

            // ASCII single quote.
            '\'' => self.normalize_ascii_single(),

            _ => ch,
        }
    }

    /// Normalizes one ASCII double quote according to the active outer quote
    /// family.
    ///
    /// When an outer quote is already open, a matching closer is produced.
    /// Otherwise, a curly opening double quote is used by default.
    #[inline]
    fn normalize_ascii_double(&mut self) -> char {
        match self.double_family {
            DialogQuoteFamily::Curly => {
                self.double_family = DialogQuoteFamily::None;
                '”'
            }
            DialogQuoteFamily::Corner => {
                self.double_family = DialogQuoteFamily::None;
                '」'
            }
            DialogQuoteFamily::None => {
                self.double_family = DialogQuoteFamily::Curly;
                '“'
            }
        }
    }

    /// Normalizes one ASCII single quote according to the active inner quote
    /// family.
    ///
    /// When an inner quote is already open, a matching closer is produced.
    /// Otherwise, a curly opening single quote is used by default.
    #[inline]
    fn normalize_ascii_single(&mut self) -> char {
        match self.single_family {
            DialogQuoteFamily::Curly => {
                self.single_family = DialogQuoteFamily::None;
                '’'
            }
            DialogQuoteFamily::Corner => {
                self.single_family = DialogQuoteFamily::None;
                '』'
            }
            DialogQuoteFamily::None => {
                self.single_family = DialogQuoteFamily::Curly;
                '‘'
            }
        }
    }
}

/// Private Use Area marker used temporarily while Latin apostrophes are masked.
const MASKED_LATIN_SINGLE_QUOTE: char = '\u{E000}';

/// Normalizes ASCII dialog quotation marks in CJK text.
///
/// Existing curly and Traditional Chinese corner quotation marks update the
/// current quote state:
///
/// ```text
/// “dialog”
/// ‘nested dialog’
/// 「對話」
/// 『內層對話』
/// ```
///
/// When an ASCII quote closes an existing quotation, the active quote family
/// is preserved:
///
/// ```text
/// “Hello"  -> “Hello”
/// 「Hello"  -> 「Hello」
/// ‘Hello'  -> ‘Hello’
/// 『Hello'  -> 『Hello』
/// ```
///
/// When an ASCII quote opens a new quotation and no quote family is active,
/// curly quotation marks are used by default.
///
/// When `preserve_latin_single_quotes` is `true`, ASCII apostrophes between
/// Latin letters are preserved, including words such as `don't`, `I'm`,
/// `rock'n'roll`, and `O'Brien`.
pub fn normalize_cjk_text_dialog_quotes(text: &str, preserve_latin_single_quotes: bool) -> String {
    if text.is_empty() {
        return String::new();
    }

    let source = if preserve_latin_single_quotes {
        mask_latin_single_quotes(text)
    } else {
        text.to_string()
    };

    let mut state = DialogQuoteState::default();
    let mut out = String::with_capacity(source.len());

    for ch in source.chars() {
        out.push(state.normalize_char(ch));
    }

    if preserve_latin_single_quotes {
        out.replace(MASKED_LATIN_SINGLE_QUOTE, "'")
    } else {
        out
    }
}

/// Masks ASCII apostrophes that appear between Latin letters.
///
/// This prevents apostrophes in words such as `don't`, `I'm`, `rock'n'roll`,
/// and `O'Brien` from being interpreted as dialog quotation marks.
fn mask_latin_single_quotes(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());

    for (index, &ch) in chars.iter().enumerate() {
        if ch == '\''
            && index > 0
            && index + 1 < chars.len()
            && is_ascii_letter(chars[index - 1])
            && is_ascii_letter(chars[index + 1])
        {
            out.push(MASKED_LATIN_SINGLE_QUOTE);
        } else {
            out.push(ch);
        }
    }

    out
}

/// Returns `true` when the character is an ASCII Latin letter.
#[inline]
fn is_ascii_letter(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

/// Validates completed dialog quote pairs found at the beginning and end of
/// individual lines.
///
/// The validator detects:
///
/// - Reversed pairs, such as `”Hello“`, `’Hello‘`, `」你好「`, and `』你好『`.
/// - Mixed quote families, such as `「Hello”` and `“Hello」`.
/// - Mixed quote levels, such as `「Hello』` and `『Hello」`.
///
/// Mixed quote styles are not automatically corrected because the intended
/// quotation style is ambiguous. They are reported so the user can jump to the
/// suspicious line and choose the desired pair manually.
///
/// This function checks only the first and last non-whitespace characters of
/// each line. It does not perform full multi-line quote balancing.
pub fn validate_cjk_text_dialog_quotes(text: &str) -> DialogQuoteValidationResult {
    let mut suspicious_lines = Vec::new();

    let normalized_newlines = text.replace("\r\n", "\n").replace('\r', "\n");

    for (index, line) in normalized_newlines.split('\n').enumerate() {
        let stripped = line.trim();

        if stripped.is_empty() {
            continue;
        }

        if has_suspicious_dialog_quote_pair(stripped) {
            suspicious_lines.push(DialogQuoteIssue {
                line_number: index + 1,
                text: line.to_string(),
            });
        }
    }

    let is_valid = suspicious_lines.is_empty();
    let summary = build_dialog_quote_summary(is_valid, &suspicious_lines);

    DialogQuoteValidationResult {
        is_valid,
        summary,
        suspicious_lines,
    }
}

/// Returns `true` when a line begins and ends with a reversed or mismatched
/// dialog quote pair.
fn has_suspicious_dialog_quote_pair(stripped: &str) -> bool {
    let mut chars = stripped.chars();

    let Some(first) = chars.next() else {
        return false;
    };

    let Some(last) = stripped.chars().next_back() else {
        return false;
    };

    // Reversed pair:
    // ”...“ / ’...‘ / 」...「 / 』...『
    if is_dialog_quote_closer(first) && is_dialog_quote_opener(last) {
        return true;
    }

    // Both ends are quotation marks, but they do not form a valid pair:
    // 「...” / “...」 / 「...』 / 『...」, etc.
    is_dialog_quote_opener(first)
        && is_dialog_quote_closer(last)
        && !is_matching_dialog_quote_pair(first, last)
}

/// Returns `true` for supported opening dialog quotation marks.
#[inline]
fn is_dialog_quote_opener(ch: char) -> bool {
    matches!(ch, '“' | '‘' | '「' | '『')
}

/// Returns `true` for supported closing dialog quotation marks.
#[inline]
fn is_dialog_quote_closer(ch: char) -> bool {
    matches!(ch, '”' | '’' | '」' | '』')
}

/// Returns `true` when the supplied opening and closing quotation marks form a
/// supported matching pair.
#[inline]
fn is_matching_dialog_quote_pair(open: char, close: char) -> bool {
    matches!(
        (open, close),
        ('“', '”') | ('‘', '’') | ('「', '」') | ('『', '』')
    )
}

/// Builds the English fallback validation summary.
///
/// The frontend may replace these messages with localized i18n strings. This
/// summary remains useful for command-line use, tests, logs, or callers that do
/// not provide their own localization.
fn build_dialog_quote_summary(is_valid: bool, suspicious_lines: &[DialogQuoteIssue]) -> String {
    if is_valid {
        return "No suspicious dialog quote issues found.".to_string();
    }

    let mut summary = String::new();

    summary.push_str(&format!(
        "Found {} suspicious dialog quote line(s).\n\n",
        suspicious_lines.len()
    ));

    summary.push_str("Hint:\n");
    summary
        .push_str("The actual typo is often a missing, extra, reversed, or mixed dialog quote.\n");
    summary.push_str("It may appear on the reported line or a few lines above it.\n");
    summary.push_str("Fix the source text and validate again.");

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_ascii_curly_quotes() {
        assert_eq!(
            normalize_cjk_text_dialog_quotes("\"Hello\"", true),
            "“Hello”"
        );

        assert_eq!(normalize_cjk_text_dialog_quotes("'Hello'", true), "‘Hello’");
    }

    #[test]
    fn closes_existing_curly_quote_family() {
        assert_eq!(
            normalize_cjk_text_dialog_quotes("“Hello\"", true),
            "“Hello”"
        );

        assert_eq!(normalize_cjk_text_dialog_quotes("‘Hello'", true), "‘Hello’");
    }

    #[test]
    fn closes_existing_corner_quote_family() {
        assert_eq!(
            normalize_cjk_text_dialog_quotes("「Hello\"", true),
            "「Hello」"
        );

        assert_eq!(
            normalize_cjk_text_dialog_quotes("『Hello'", true),
            "『Hello』"
        );
    }

    #[test]
    fn preserves_existing_corner_quotes() {
        assert_eq!(
            normalize_cjk_text_dialog_quotes("「你好」『世界』", true),
            "「你好」『世界』"
        );
    }

    #[test]
    fn preserves_latin_apostrophes() {
        assert_eq!(
            normalize_cjk_text_dialog_quotes("I'm reading O'Brien's book.", true),
            "I'm reading O'Brien's book."
        );
    }

    #[test]
    fn detects_reversed_curly_pair() {
        let result = validate_cjk_text_dialog_quotes("”Hello“");

        assert!(!result.is_valid());
        assert_eq!(result.suspicious_lines.len(), 1);
        assert_eq!(result.suspicious_lines[0].line_number, 1);
    }

    #[test]
    fn detects_reversed_corner_pair() {
        let result = validate_cjk_text_dialog_quotes("　　」……「");

        assert!(!result.is_valid());
        assert_eq!(result.suspicious_lines.len(), 1);
        assert_eq!(result.suspicious_lines[0].text, "　　」……「");
    }

    #[test]
    fn detects_mixed_quote_family() {
        for text in ["「Hello”", "“Hello」", "『Hello’", "‘Hello』"] {
            let result = validate_cjk_text_dialog_quotes(text);

            assert!(!result.is_valid(), "expected suspicious: {text}");
        }
    }

    #[test]
    fn detects_mixed_quote_level() {
        for text in ["「Hello』", "『Hello」", "“Hello’", "‘Hello”"] {
            let result = validate_cjk_text_dialog_quotes(text);

            assert!(!result.is_valid(), "expected suspicious: {text}");
        }
    }

    #[test]
    fn accepts_matching_quote_pairs() {
        let text = ["“Hello”", "‘Hello’", "「Hello」", "『Hello』"].join("\n");

        let result = validate_cjk_text_dialog_quotes(&text);

        assert!(result.is_valid());
        assert!(result.suspicious_lines.is_empty());
    }
}
