//! The `.rhl` side of the existing `ruchy check` and `ruchy fmt` verbs (spec
//! RHL-001 §5: extend by file extension; plan D8). The new verbs `ruchy fix`
//! and `ruchy vocab` (RHL-2) live in [`super::fix`] and [`super::vocab_cli`].
//!
//! Everything a verb decides lives here, in the library, so the required
//! check (`cargo test --lib`, binding B4) covers it. The binary only prints an
//! [`Outcome`] and exits with its code.

use super::check::{check, Report};
use super::diag::{self, render_text};
use super::fmt::format_source;
use super::vocab::find_root;
use std::path::Path;

/// How `ruchy check` prints its result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Diagnostics for a person.
    Text,
    /// The report object of plan D8, for agents.
    Json,
}

impl OutputFormat {
    /// Parse `--format`. Only `text` and `json` exist.
    ///
    /// # Errors
    ///
    /// Returns the message to print for any other value.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            other => Err(format!(
                "--format must be `text` or `json` for .rhl files, not `{other}`"
            )),
        }
    }
}

/// What a verb prints, and the process exit code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// Text for standard output.
    pub stdout: String,
    /// Text for standard error.
    pub stderr: String,
    /// 0 pass, 1 error, 2 refusal (plan D8).
    pub exit: i32,
}

/// True for a path RHL owns: the `.rhl` extension.
#[must_use]
pub fn is_rhl(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "rhl")
}

/// `ruchy check` over `.rhl` files. One file prints one report; several print
/// a JSON array of reports (or the text of each). The exit code is the worst.
#[must_use]
pub fn check_files(paths: &[&Path], format: OutputFormat) -> Outcome {
    let mut reports = Vec::new();
    let mut stderr = String::new();
    for path in paths {
        match std::fs::read_to_string(path) {
            Ok(source) => reports.push(check_one(path, &source)),
            Err(e) => stderr.push_str(&format!("{}: cannot read: {e}\n", path.display())),
        }
    }
    let io_exit = i32::from(!stderr.is_empty());
    let exit = reports
        .iter()
        .map(Report::exit_code)
        .fold(io_exit, i32::max);
    Outcome {
        stdout: render(&reports, format),
        stderr,
        exit,
    }
}

fn check_one(path: &Path, source: &str) -> Report {
    let root = find_root(path);
    check(&path.display().to_string(), source, root.as_deref())
}

fn render(reports: &[Report], format: OutputFormat) -> String {
    match (format, reports) {
        (OutputFormat::Json, [one]) => json(one),
        (OutputFormat::Json, many) => json(&many),
        (OutputFormat::Text, many) => many.iter().map(text).collect(),
    }
}

/// `value` as pretty JSON and a newline.
pub(crate) fn json<T: serde::Serialize>(value: &T) -> String {
    match serde_json::to_string_pretty(value) {
        Ok(s) => s + "\n",
        Err(e) => format!("{{\"error\": \"cannot serialize the report: {e}\"}}\n"),
    }
}

/// A report for a person: its diagnostics, its unverified instances, a verdict line.
pub(crate) fn text(report: &Report) -> String {
    let mut out = render_text(&report.diagnostics);
    for u in &report.unverified {
        out.push_str(&format!(
            "{}: note: {} {} unverified: no file matches {}\n",
            report.file, u.entity, u.instance, u.source_glob
        ));
    }
    out.push_str(&format!("{}: {}\n", report.file, report.verdict.word()));
    out
}

/// `ruchy fmt` on one `.rhl` source: the normal form, or the parse diagnostic
/// as text when the source is not a program.
///
/// # Errors
///
/// Returns the rendered diagnostic when `source` does not parse.
pub fn format_file_source(file: &str, source: &str) -> Result<String, String> {
    format_source(source)
        .map_err(|failure| render_text(&[diag::from_parse_failure(file, source, &failure)]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn corpus(rel: &str) -> PathBuf {
        crate::rhl::repo_root().join(rel)
    }

    fn check_json(rel: &str) -> (serde_json::Value, i32) {
        let path = corpus(rel);
        let out = check_files(&[path.as_path()], OutputFormat::Json);
        let value = serde_json::from_str(&out.stdout).expect("stdout is one JSON object");
        (value, out.exit)
    }

    #[test]
    fn test_rhl_1_cli_is_rhl_by_extension_only() {
        assert!(is_rhl(Path::new("a/job.rhl")));
        assert!(!is_rhl(Path::new("a/job.ruchy")));
        assert!(!is_rhl(Path::new("a/rhl")));
    }

    #[test]
    fn test_rhl_1_cli_format_flag_accepts_text_and_json_only() {
        assert_eq!(OutputFormat::parse("json"), Ok(OutputFormat::Json));
        assert_eq!(OutputFormat::parse("text"), Ok(OutputFormat::Text));
        assert!(OutputFormat::parse("yaml").is_err());
    }

    #[test]
    fn test_rhl_1_cli_check_json_of_a_typo_break_is_the_d8_object_exit_2() {
        let (v, exit) =
            check_json("docs/rhl/breaks/planted/typo-term/01-gx10-disk-watch/broken.rhl");
        for key in ["file", "verdict", "diagnostics", "unverified"] {
            assert!(v.get(key).is_some(), "missing `{key}`");
        }
        assert_eq!(v["verdict"], "refused");
        let codes: Vec<&str> = v["diagnostics"]
            .as_array()
            .expect("array")
            .iter()
            .filter_map(|d| d["code"].as_str())
            .collect();
        assert!(codes.contains(&"RHL-V002"), "{codes:?}");
        assert_eq!(exit, 2);
    }

    #[test]
    fn test_rhl_1_cli_check_of_a_missing_end_is_p001_exit_1() {
        let (v, exit) =
            check_json("docs/rhl/breaks/planted/missing-end/01-gx10-disk-watch/broken.rhl");
        assert_eq!(v["diagnostics"][0]["code"], "RHL-P001");
        assert_eq!(v["verdict"], "fail");
        assert_eq!(exit, 1);
    }

    #[test]
    fn test_rhl_1_cli_check_of_several_files_prints_an_array_and_the_worst_exit() {
        let a = corpus("docs/rhl/breaks/planted/missing-end/01-gx10-disk-watch/broken.rhl");
        let b = corpus("docs/rhl/breaks/planted/typo-term/01-gx10-disk-watch/broken.rhl");
        let out = check_files(&[a.as_path(), b.as_path()], OutputFormat::Json);
        let v: serde_json::Value = serde_json::from_str(&out.stdout).expect("JSON");
        assert_eq!(v.as_array().map(Vec::len), Some(2));
        assert_eq!(out.exit, 2);
    }

    #[test]
    fn test_rhl_1_cli_check_text_names_the_code_and_the_verdict() {
        let path = corpus("docs/rhl/breaks/planted/typo-term/01-gx10-disk-watch/broken.rhl");
        let out = check_files(&[path.as_path()], OutputFormat::Text);
        assert!(out.stdout.contains("error[RHL-V002]"), "{}", out.stdout);
        assert!(
            out.stdout.contains("did you mean `disk free of`"),
            "{}",
            out.stdout
        );
        assert!(
            out.stdout.trim_end().ends_with(": refused"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn test_rhl_1_cli_check_of_an_unreadable_file_is_exit_1_on_stderr() {
        let out = check_files(&[Path::new("/nonexistent/x.rhl")], OutputFormat::Text);
        assert_eq!(out.exit, 1);
        assert!(out.stderr.contains("cannot read"));
    }

    #[test]
    fn test_rhl_1_cli_fmt_gives_the_normal_form_or_the_parse_diagnostic() {
        let src = "job \"x\"\n      every 1 hour\nend\n";
        assert_eq!(
            format_file_source("x.rhl", src),
            Ok("job \"x\"\n  every 1 hour\nend\n".to_string())
        );
        let err = format_file_source("x.rhl", "job \"x\"\n  every 1 hour\n").expect_err("no end");
        assert!(err.contains("error[RHL-P001]"), "{err}");
    }
}
