//! CITL export (ruchy#165): rustc diagnostics on transpiled Ruchy output as
//! `CitlExport` records for organizational-intelligence-plugin (OIP).
//!
//! Each record is one rustc diagnostic (`error` or `warning`) from compiling the
//! generated Rust. Spans point into the generated Rust, not the Ruchy source.

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;
use std::process::Command;

/// One compiler diagnostic as an OIP training record.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CitlExport {
    pub source_file: String,
    pub error_code: Option<String>,
    pub clippy_lint: Option<String>,
    pub level: String,
    pub message: String,
    pub oip_category: Option<String>,
    /// Compiler diagnostics are ground truth, so this is always 1.0.
    pub confidence: f32,
    pub span: Option<SpanInfo>,
    pub suggestion: Option<SuggestionInfo>,
    pub timestamp: i64,
}

/// Primary span of a diagnostic, in the generated Rust file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpanInfo {
    pub file_name: String,
    pub line_start: u64,
    pub line_end: u64,
    pub column_start: u64,
    pub column_end: u64,
}

/// A replacement the compiler suggested (first one found, any applicability).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SuggestionInfo {
    pub message: String,
    pub replacement: String,
}

/// Turn rustc `--error-format=json` lines into records. Non-diagnostic lines
/// (artifact notices, notes, help) are skipped.
pub fn records_from_rustc_json(stderr: &str, source_file: &str, timestamp: i64) -> Vec<CitlExport> {
    stderr
        .lines()
        .filter_map(|line| record_from_line(line, source_file, timestamp))
        .collect()
}

fn record_from_line(line: &str, source_file: &str, timestamp: i64) -> Option<CitlExport> {
    let v: Value = serde_json::from_str(line).ok()?;
    if v["$message_type"] != "diagnostic" {
        return None;
    }
    let level = v["level"].as_str()?;
    if level != "error" && level != "warning" {
        return None;
    }
    let code = v["code"]["code"].as_str().map(str::to_string);
    let message = v["message"].as_str()?;
    if is_rustc_summary(message, code.is_none(), &v) {
        return None;
    }
    let is_clippy = code.as_deref().is_some_and(|c| c.starts_with("clippy::"));
    Some(CitlExport {
        source_file: source_file.to_string(),
        error_code: code.clone().filter(|_| !is_clippy),
        clippy_lint: code.filter(|_| is_clippy),
        level: level.to_string(),
        message: message.to_string(),
        oip_category: None,
        confidence: 1.0,
        span: primary_span(&v),
        suggestion: suggestion(&v),
        timestamp,
    })
}

/// rustc's trailing totals ("aborting due to N previous error(s)", "N warning(s)
/// emitted") carry no code and no span. They summarise the other diagnostics,
/// so they are not records. Any other codeless, spanless error is kept.
fn is_rustc_summary(message: &str, no_code: bool, v: &Value) -> bool {
    if message.starts_with("aborting due to ") {
        return true;
    }
    let no_spans = v["spans"].as_array().is_none_or(Vec::is_empty);
    no_code
        && no_spans
        && (message.ends_with(" warning emitted") || message.ends_with(" warnings emitted"))
}

fn primary_span(v: &Value) -> Option<SpanInfo> {
    let span = v["spans"]
        .as_array()?
        .iter()
        .find(|s| s["is_primary"] == true)?;
    Some(SpanInfo {
        file_name: span["file_name"].as_str()?.to_string(),
        line_start: span["line_start"].as_u64()?,
        line_end: span["line_end"].as_u64()?,
        column_start: span["column_start"].as_u64()?,
        column_end: span["column_end"].as_u64()?,
    })
}

fn suggestion(v: &Value) -> Option<SuggestionInfo> {
    v["children"].as_array()?.iter().find_map(|child| {
        let replacement = child["spans"]
            .as_array()?
            .iter()
            .find_map(|s| s["suggested_replacement"].as_str())?;
        Some(SuggestionInfo {
            message: child["message"].as_str()?.to_string(),
            replacement: replacement.to_string(),
        })
    })
}

/// Compile `rust_code` with rustc and collect its diagnostics as records.
///
/// # Errors
///
/// Returns an error when rustc cannot be started or the scratch dir fails.
pub fn export_rustc_diagnostics(rust_code: &str, source_file: &str) -> Result<Vec<CitlExport>> {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("ruchy_citl_{}_{seq}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let result = run_rustc(&dir, rust_code, source_file);
    let _ = std::fs::remove_dir_all(&dir);
    result
}

fn run_rustc(dir: &Path, rust_code: &str, source_file: &str) -> Result<Vec<CitlExport>> {
    let src = dir.join("citl.rs");
    std::fs::write(&src, rust_code)?;
    let out = Command::new("rustc")
        .arg("--crate-type=bin")
        .arg("--edition=2021")
        .arg("--emit=metadata")
        .arg("--error-format=json")
        .arg("--out-dir")
        .arg(dir)
        .arg(&src)
        .output()
        .context("could not run rustc for CITL export")?;
    let stderr = String::from_utf8_lossy(&out.stderr);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(0))
        .unwrap_or(0);
    Ok(records_from_rustc_json(&stderr, source_file, timestamp))
}

#[cfg(test)]
mod tests {
    use super::*;

    const E0308_LINE: &str = r#"{"$message_type":"diagnostic","message":"mismatched types","code":{"code":"E0308","explanation":null},"level":"error","spans":[{"file_name":"citl.rs","line_start":2,"line_end":2,"column_start":18,"column_end":21,"is_primary":true,"suggested_replacement":null}],"children":[{"message":"use an integer","level":"help","spans":[{"file_name":"citl.rs","line_start":2,"line_end":2,"column_start":18,"column_end":21,"is_primary":true,"suggested_replacement":"42"}]}],"rendered":"x"}"#;

    #[test]
    fn test_ruchy_165_record_maps_error_code_span_and_suggestion() {
        let recs = records_from_rustc_json(E0308_LINE, "input.rb", 7);
        assert_eq!(recs.len(), 1);
        let r = &recs[0];
        assert_eq!(r.error_code.as_deref(), Some("E0308"));
        assert_eq!(r.clippy_lint, None);
        assert_eq!(r.level, "error");
        assert_eq!(r.source_file, "input.rb");
        assert_eq!(r.timestamp, 7);
        assert_eq!(r.confidence, 1.0);
        assert_eq!(r.oip_category, None);
        let span = r.span.as_ref().expect("primary span");
        assert_eq!(span.line_start, 2);
        assert_eq!(span.column_start, 18);
        assert_eq!(span.column_end, 21);
        let sug = r.suggestion.as_ref().expect("suggestion");
        assert_eq!(sug.replacement, "42");
        assert_eq!(sug.message, "use an integer");
    }

    #[test]
    fn test_ruchy_165_clippy_code_goes_to_clippy_lint() {
        let line = r#"{"$message_type":"diagnostic","message":"used unwrap","code":{"code":"clippy::unwrap_used","explanation":null},"level":"warning","spans":[],"children":[]}"#;
        let r = &records_from_rustc_json(line, "a.rb", 0)[0];
        assert_eq!(r.clippy_lint.as_deref(), Some("clippy::unwrap_used"));
        assert_eq!(r.error_code, None);
        assert_eq!(r.level, "warning");
        assert!(r.span.is_none());
        assert!(r.suggestion.is_none());
    }

    #[test]
    fn test_ruchy_165_non_diagnostic_lines_are_skipped() {
        let artifact = r#"{"$message_type":"artifact","artifact":"x.rmeta","emit":"metadata"}"#;
        let note = r#"{"$message_type":"diagnostic","message":"n","code":null,"level":"note","spans":[],"children":[]}"#;
        let stderr = format!("not json\n{artifact}\n{note}\n");
        assert!(records_from_rustc_json(&stderr, "a.rb", 0).is_empty());
    }

    #[test]
    fn test_ruchy_165_rustc_summary_line_is_not_a_record() {
        let summary = r#"{"$message_type":"diagnostic","message":"aborting due to 1 previous error","code":null,"level":"error","spans":[],"children":[]}"#;
        let stderr = format!("{E0308_LINE}\n{summary}\n");
        let recs = records_from_rustc_json(&stderr, "a.rb", 0);
        assert_eq!(recs.len(), 1, "{recs:?}");
        assert_eq!(recs[0].error_code.as_deref(), Some("E0308"));
    }

    #[test]
    fn test_ruchy_165_codeless_real_error_is_kept() {
        let line = r#"{"$message_type":"diagnostic","message":"unwinding panics are not supported without std","code":null,"level":"error","spans":[],"children":[]}"#;
        let recs = records_from_rustc_json(line, "a.rb", 0);
        assert_eq!(recs.len(), 1, "{recs:?}");
        assert_eq!(recs[0].error_code, None);
    }

    #[test]
    fn test_ruchy_165_export_rustc_clean_program_has_no_records() {
        let recs = export_rustc_diagnostics("fn main() { println!(\"ok\"); }\n", "a.rb")
            .expect("rustc runs");
        assert!(recs.is_empty(), "{recs:?}");
    }

    #[test]
    fn test_ruchy_165_export_rustc_type_error_is_e0308() {
        let recs = export_rustc_diagnostics("fn main() { let x: i32 = \"a\"; }\n", "a.rb")
            .expect("rustc runs");
        assert!(recs
            .iter()
            .any(|r| r.error_code.as_deref() == Some("E0308")));
    }
}
