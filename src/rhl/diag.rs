//! Diagnostics are data (spec RHL-001 §3.5): the agent-repair contract.
//!
//! Every diagnostic carries a stable code, a span, the expected set,
//! candidates and fix edits. Agents program against the JSON; the text a
//! human reads is rendered from the same value, never the other way round.

use super::codes::{self, CodeInfo};
use super::parse::ParseFailure;
use super::tree::Span;
use serde::{Deserialize, Serialize};

/// How serious a diagnostic is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// The program does not compile.
    Error,
    /// The program compiles, but something is worth fixing.
    Warning,
}

/// A position a person can find: 1-based line and column, length in characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineSpan {
    /// 1-based line number.
    pub line: usize,
    /// 1-based column, counted in characters.
    pub col: usize,
    /// Length in characters.
    pub len: usize,
}

impl LineSpan {
    /// Convert a byte span of `source` to a line span. Out-of-range or
    /// mid-character offsets are clamped to the nearest character boundary.
    #[must_use]
    pub fn from_span(source: &str, span: Span) -> Self {
        let start = floor_char_boundary(source, span.start);
        let end = floor_char_boundary(source, span.end.max(start));
        let before = &source[..start];
        let line = before.matches('\n').count() + 1;
        let line_start = before.rfind('\n').map_or(0, |i| i + 1);
        Self {
            line,
            col: source[line_start..start].chars().count() + 1,
            len: source[start..end].chars().count(),
        }
    }
}

fn floor_char_boundary(s: &str, at: usize) -> usize {
    let mut i = at.min(s.len());
    while !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// What the checker wanted at the diagnostic's position.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expected {
    /// The term kind or namespace expected, for example `measure` or `unit`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub kind: Option<String>,
    /// The type expected, for example `Size`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub gives: Option<String>,
    /// The tokens the parser would have accepted.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tokens: Vec<String>,
}

/// A known word close to an unknown one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Candidate {
    /// The known spelling.
    pub term: String,
    /// Where it comes from, for example `fleet v1`, or `let` for a local name.
    pub vocabulary: String,
    /// Edit distance from the written spelling.
    pub distance: usize,
}

/// One text replacement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edit {
    /// The text to replace.
    pub span: LineSpan,
    /// The replacement.
    pub text: String,
}

/// A proposed repair. `safe` only when it is the unique candidate and keeps
/// types (§3.5); `ruchy fix --safe` (row RHL-2) applies only those.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fix {
    /// What the fix does.
    pub title: String,
    /// Whether it may be applied without a choice.
    pub safe: bool,
    /// The edits, applied together.
    pub edits: Vec<Edit>,
}

/// One diagnostic, in the §3.5 shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Stable code from [`codes::CATALOGUE`].
    pub code: String,
    /// Error or warning.
    pub severity: Severity,
    /// The file the span refers to.
    pub file: String,
    /// Where the problem is.
    pub span: LineSpan,
    /// A one-line human message.
    pub message: String,
    /// What was expected at this position.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub expected: Option<Expected>,
    /// Known spellings close to what was written.
    pub candidates: Vec<Candidate>,
    /// Proposed repairs.
    pub fixes: Vec<Fix>,
    /// The §3.5 refusal this diagnostic is, if any.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub refusal: Option<String>,
}

impl Diagnostic {
    /// An error with `code` at `span` of `source`, with no candidates or fixes.
    ///
    /// # Panics
    ///
    /// Panics if `code` is not in the catalogue: an uncatalogued code is a
    /// programming error, and every emitting path is covered by a test.
    #[must_use]
    pub fn error(code: &str, file: &str, source: &str, span: Span, message: String) -> Self {
        let info: &CodeInfo = codes::lookup(code)
            .unwrap_or_else(|| panic!("RHL-1: diagnostic code {code} is not in the catalogue"));
        Self {
            code: info.code.to_string(),
            severity: Severity::Error,
            file: file.to_string(),
            span: LineSpan::from_span(source, span),
            message,
            expected: None,
            candidates: Vec::new(),
            fixes: Vec::new(),
            refusal: info.refusal.map(str::to_string),
        }
    }

    /// The same diagnostic with `expected` set.
    #[must_use]
    pub fn with_expected(mut self, expected: Expected) -> Self {
        self.expected = Some(expected);
        self
    }
}

/// `source` with `edits` applied. Edits are applied last-position first, so
/// an earlier edit's line and column stay valid; an edit whose line or column
/// lies past the text is ignored.
#[must_use]
pub fn apply_edits(source: &str, edits: &[Edit]) -> String {
    let mut sorted: Vec<&Edit> = edits.iter().collect();
    sorted.sort_by_key(|e| std::cmp::Reverse((e.span.line, e.span.col)));
    sorted.into_iter().fold(source.to_string(), |text, e| {
        match byte_range(&text, e.span) {
            Some((start, end)) => format!("{}{}{}", &text[..start], e.text, &text[end..]),
            None => text,
        }
    })
}

/// The byte range of a line span in `text`, if the line and column exist.
fn byte_range(text: &str, span: LineSpan) -> Option<(usize, usize)> {
    let line_start = if span.line == 1 {
        0
    } else {
        text.match_indices('\n').nth(span.line - 2)?.0 + 1
    };
    let mut offsets = text[line_start..]
        .char_indices()
        .map(|(i, _)| line_start + i)
        .chain(std::iter::once(text.len()));
    let start = offsets.nth(span.col - 1)?;
    let end = if span.len == 0 {
        start
    } else {
        offsets.nth(span.len - 1)?
    };
    Some((start, end))
}

/// The diagnostic for a parse failure of `source`.
#[must_use]
pub fn from_parse_failure(file: &str, source: &str, failure: &ParseFailure) -> Diagnostic {
    match failure {
        ParseFailure::InvalidToken { at } => invalid_token(file, source, *at),
        ParseFailure::UnrecognizedEof { at, expected } => eof(file, source, *at, expected),
        ParseFailure::UnrecognizedToken {
            span,
            found,
            expected,
        } => unexpected(file, source, *span, found, expected),
        ParseFailure::ExtraToken { span, found } => unexpected(file, source, *span, found, &[]),
    }
}

fn invalid_token(file: &str, source: &str, at: usize) -> Diagnostic {
    let at = floor_char_boundary(source, at);
    let len = source[at..].chars().next().map_or(0, char::len_utf8);
    let shown = source[at..at + len].escape_debug().to_string();
    let message = format!("`{shown}` starts no RHL token");
    Diagnostic::error(codes::P003, file, source, Span::new(at, at + len), message)
}

fn eof(file: &str, source: &str, at: usize, expected: &[String]) -> Diagnostic {
    let tokens = token_names(expected);
    let (code, message) = if tokens.iter().any(|t| t == "end") {
        (
            codes::P001,
            "missing `end`: the input ends inside a block".to_string(),
        )
    } else {
        (codes::P002, "unexpected end of input".to_string())
    };
    Diagnostic::error(code, file, source, Span::new(at, at), message).with_expected(Expected {
        tokens,
        ..Expected::default()
    })
}

fn unexpected(
    file: &str,
    source: &str,
    span: Span,
    found: &str,
    expected: &[String],
) -> Diagnostic {
    let shown = if found.starts_with('\n') {
        "end of line"
    } else {
        found
    };
    let message = format!("unexpected `{shown}`");
    Diagnostic::error(codes::P002, file, source, span, message).with_expected(Expected {
        tokens: token_names(expected),
        ..Expected::default()
    })
}

/// LALRPOP spells a keyword `"\"end\""` and a named terminal `"NEWLINE"`;
/// a person reads `end` and `end of line`.
fn token_names(expected: &[String]) -> Vec<String> {
    expected
        .iter()
        .map(|t| match t.as_str() {
            "NEWLINE" => "end of line".to_string(),
            other => other.trim_matches('"').to_string(),
        })
        .collect()
}

/// Render diagnostics for a person: `file:line:col: error[CODE]: message`,
/// then one `help:` line per candidate.
#[must_use]
pub fn render_text(diagnostics: &[Diagnostic]) -> String {
    let mut out = String::new();
    for d in diagnostics {
        let sev = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        out.push_str(&format!(
            "{}:{}:{}: {sev}[{}]: {}\n",
            d.file, d.span.line, d.span.col, d.code, d.message
        ));
        for c in &d.candidates {
            out.push_str(&format!(
                "  help: did you mean `{}` ({})?\n",
                c.term, c.vocabulary
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rhl::parse;

    fn diag_of(src: &str) -> Diagnostic {
        let failure = parse(src).expect_err("the test source must not parse");
        from_parse_failure("t.rhl", src, &failure)
    }

    #[test]
    fn test_rhl_1_diag_missing_end_is_p001_at_end_of_input() {
        let src = "job \"x\"\n  every 1 hour\n";
        let d = diag_of(src);
        assert_eq!(d.code, codes::P001);
        assert_eq!(
            d.span,
            LineSpan {
                line: 3,
                col: 1,
                len: 0
            }
        );
        assert!(d
            .expected
            .expect("expected set")
            .tokens
            .contains(&"end".to_string()));
    }

    #[test]
    fn test_rhl_1_diag_misplaced_token_is_p002_with_the_expected_set() {
        let src = "job \"x\"\n  every hour\nend\n";
        let d = diag_of(src);
        assert_eq!(d.code, codes::P002);
        assert_eq!(
            d.span,
            LineSpan {
                line: 2,
                col: 9,
                len: 4
            }
        );
        assert_eq!(d.expected.expect("expected set").tokens, vec!["INT"]);
    }

    #[test]
    fn test_rhl_1_diag_unknown_character_is_p003_one_char_long() {
        let d = diag_of("job \"x\"\n  every 1 hour !\nend\n");
        assert_eq!(d.code, codes::P003);
        assert_eq!(
            d.span,
            LineSpan {
                line: 2,
                col: 16,
                len: 1
            }
        );
    }

    #[test]
    fn test_rhl_1_diag_eof_without_end_in_the_expected_set_is_p002() {
        let d = diag_of("use vocabulary fleet v1");
        assert_eq!(d.code, codes::P002);
        assert_eq!(d.message, "unexpected end of input");
    }

    #[test]
    fn test_rhl_1_diag_line_span_counts_characters_not_bytes() {
        let src = "\"é\" x";
        let s = LineSpan::from_span(src, Span::new(5, 6));
        assert_eq!(
            s,
            LineSpan {
                line: 1,
                col: 5,
                len: 1
            }
        );
    }

    #[test]
    fn test_rhl_1_diag_json_has_the_section_3_5_shape() {
        let d = diag_of("job \"x\"\n  every hour\nend\n");
        let v = serde_json::to_value(&d).expect("serializable");
        for key in [
            "code",
            "severity",
            "file",
            "span",
            "message",
            "candidates",
            "fixes",
        ] {
            assert!(v.get(key).is_some(), "missing `{key}` in {v}");
        }
        assert_eq!(v["severity"], "error");
        assert_eq!(v["span"]["line"], 2);
        let back: Diagnostic = serde_json::from_value(v).expect("round-trips");
        assert_eq!(back, d);
    }

    #[test]
    fn test_rhl_1_diag_refusal_class_is_carried_from_the_catalogue() {
        let d = Diagnostic::error(codes::V003, "f", "abc", Span::new(0, 1), String::new());
        assert_eq!(d.refusal.as_deref(), Some("Ambiguous"));
        let p = Diagnostic::error(codes::P002, "f", "abc", Span::new(0, 1), String::new());
        assert_eq!(p.refusal, None);
    }

    #[test]
    fn test_rhl_1_diag_apply_edits_counts_characters_and_lines() {
        let src = "a\n  é fre of\nz";
        let edit = |line, col, len, text: &str| Edit {
            span: LineSpan { line, col, len },
            text: text.to_string(),
        };
        assert_eq!(
            apply_edits(src, &[edit(2, 5, 3, "free")]),
            "a\n  é free of\nz"
        );
        assert_eq!(apply_edits(src, &[edit(1, 2, 0, "b")]), "ab\n  é fre of\nz");
        assert_eq!(
            apply_edits(src, &[edit(9, 1, 1, "x")]),
            src,
            "past the text: ignored"
        );
        let two = [edit(1, 1, 1, "A"), edit(3, 1, 1, "Z")];
        assert_eq!(apply_edits(src, &two), "A\n  é fre of\nZ");
    }

    #[test]
    fn test_rhl_1_diag_render_text_names_file_line_code_and_candidates() {
        let mut d = Diagnostic::error(codes::V002, "a.rhl", "x\nabc", Span::new(2, 5), "m".into());
        d.candidates.push(Candidate {
            term: "abd".into(),
            vocabulary: "fleet v1".into(),
            distance: 1,
        });
        let text = render_text(&[d]);
        assert_eq!(
            text,
            "a.rhl:2:1: error[RHL-V002]: m\n  help: did you mean `abd` (fleet v1)?\n"
        );
    }
}
