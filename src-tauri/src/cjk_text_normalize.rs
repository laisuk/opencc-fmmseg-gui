use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DialogQuoteValidationResult {
    pub is_valid: bool,
    pub summary: String,
    pub suspicious_lines: Vec<DialogQuoteIssue>,
}

impl DialogQuoteValidationResult {
    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DialogQuoteIssue {
    pub line_number: usize,
    pub text: String,
}

#[derive(Default)]
struct DialogQuoteState {
    inside_double: bool,
    inside_single: bool,
}

impl DialogQuoteState {
    #[inline]
    fn normalize_char(&mut self, ch: char) -> char {
        match ch {
            '“' => {
                self.inside_double = true;
                ch
            }
            '”' => {
                self.inside_double = false;
                ch
            }
            '"' => {
                if self.inside_double {
                    self.inside_double = false;
                    '”'
                } else {
                    self.inside_double = true;
                    '“'
                }
            }

            '‘' => {
                self.inside_single = true;
                ch
            }
            '’' => {
                self.inside_single = false;
                ch
            }
            '\'' => {
                if self.inside_single {
                    self.inside_single = false;
                    '’'
                } else {
                    self.inside_single = true;
                    '‘'
                }
            }

            _ => ch,
        }
    }
}

const MASKED_LATIN_SINGLE_QUOTE: char = '\u{E000}';

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

fn mask_latin_single_quotes(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());

    for i in 0..chars.len() {
        let ch = chars[i];

        if ch == '\''
            && i > 0
            && i + 1 < chars.len()
            && is_ascii_letter(chars[i - 1])
            && is_ascii_letter(chars[i + 1])
        {
            out.push(MASKED_LATIN_SINGLE_QUOTE);
        } else {
            out.push(ch);
        }
    }

    out
}

#[inline]
fn is_ascii_letter(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

pub fn validate_cjk_text_dialog_quotes(text: &str) -> DialogQuoteValidationResult {
    let mut suspicious_lines = Vec::new();

    for (idx, line) in text.replace("\r\n", "\n").split('\n').enumerate() {
        let stripped = line.trim();

        if stripped.is_empty() {
            continue;
        }

        if (stripped.starts_with('”') && stripped.ends_with('“'))
            || (stripped.starts_with('’') && stripped.ends_with('‘'))
        {
            suspicious_lines.push(DialogQuoteIssue {
                line_number: idx + 1,
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

fn build_dialog_quote_summary(
    is_valid: bool,
    suspicious_lines: &[DialogQuoteIssue],
) -> String {
    if is_valid {
        return "No suspicious dialog quote issues found.".to_string();
    }

    let mut s = String::new();

    s.push_str(&format!(
        "Found {} suspicious dialog quote line(s).\n\n",
        suspicious_lines.len()
    ));
    s.push_str("Hint:\n");
    s.push_str("The actual typo is often a missing or extra dialog quote\n");
    s.push_str("a few lines above the first reported line.\n");
    s.push_str("Fix the source text and validate again.");

    s
}