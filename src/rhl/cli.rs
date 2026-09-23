//! The `.rhl` side of the existing `ruchy check` and `ruchy fmt` verbs (spec
//! RHL-001 §5: extend by file extension; plan D8), for both surfaces: RHL
//! text (`.rhl`) and its YAML (`.rhl.yaml`, row RHL-3). The new verbs `ruchy
//! fix` and `ruchy vocab` (RHL-2) live in [`super::fix`] and
//! [`super::vocab_cli`]; `ruchy convert` (RHL-3) is [`convert_file`].
//!
//! Everything a verb decides lives here, in the library, so the required
//! check (`cargo test --lib`, binding B4) covers it. The binary only prints an
//! [`Outcome`] and exits with its code.

use super::check::{check, check_yaml, Report};
use super::diag::{self, render_text};
use super::fmt::{format, format_source};
use super::lower::{lower_source, lower_yaml, to_rust, LowerFailure};
use super::vocab::find_root;
use super::yaml::{self, is_rhl_yaml};
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

/// True for a path RHL owns: the `.rhl` extension, or a `.rhl.yaml` name.
#[must_use]
pub fn is_rhl(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "rhl") || is_rhl_yaml(path)
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
    let file = path.display().to_string();
    if is_rhl_yaml(path) {
        check_yaml(&file, source, root.as_deref())
    } else {
        check(&file, source, root.as_deref())
    }
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
/// as text when the source is not a program. A `.rhl.yaml` file is re-emitted
/// as canonical YAML.
///
/// # Errors
///
/// Returns the rendered diagnostic when `source` does not parse.
pub fn format_file_source(file: &str, source: &str) -> Result<String, String> {
    if is_rhl_yaml(Path::new(file)) {
        return yaml::format_yaml(source).map_err(|e| yaml_error(file, &e));
    }
    format_source(source)
        .map_err(|failure| render_text(&[diag::from_parse_failure(file, source, &failure)]))
}

fn yaml_error(file: &str, e: &yaml::YamlError) -> String {
    render_text(&[yaml::diagnostic(file, e)])
}

/// Which surface `ruchy convert` writes; the input is the other one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// RHL text in `fmt`'s normal form, read from YAML.
    Rhl,
    /// Canonical `.rhl.yaml`, read from RHL text.
    Yaml,
}

impl Target {
    /// Parse `--to`. Only `rhl` and `yaml` exist.
    ///
    /// # Errors
    ///
    /// Returns the message to print for any other value.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "rhl" => Ok(Self::Rhl),
            "yaml" => Ok(Self::Yaml),
            other => Err(format!("--to must be `rhl` or `yaml`, not `{other}`\n")),
        }
    }

    /// The direction a file name implies: `.rhl.yaml` converts to RHL,
    /// `.rhl` to YAML, anything else to nothing.
    #[must_use]
    pub fn for_input(path: &Path) -> Option<Self> {
        if is_rhl_yaml(path) {
            Some(Self::Rhl)
        } else if is_rhl(path) {
            Some(Self::Yaml)
        } else {
            None
        }
    }
}

/// Convert `source` (named `file` in diagnostics) to `target`.
///
/// # Errors
///
/// Returns the rendered `RHL-P…` diagnostic when the source is not a tree:
/// RHL text that does not parse, or YAML that [`yaml::from_yaml`] refuses.
pub fn convert_source(file: &str, source: &str, target: Target) -> Result<String, String> {
    match target {
        Target::Yaml => super::parse(source)
            .map(|p| yaml::to_yaml(&p))
            .map_err(|f| render_text(&[diag::from_parse_failure(file, source, &f)])),
        Target::Rhl => yaml::from_yaml(source)
            .map(|p| format(&p))
            .map_err(|e| yaml_error(file, &e)),
    }
}

/// `ruchy convert <input> [-o <output>] [--to rhl|yaml]`. The result goes to
/// `output`, or to standard output. Exit 0 on success; 2 when the direction
/// is unknown or the input is not a tree (convert declines, diagnostic on
/// standard error); 1 when a file cannot be read or written.
#[must_use]
pub fn convert_file(input: &Path, output: Option<&Path>, to: Option<&str>) -> Outcome {
    let target = match resolve_target(input, to) {
        Ok(t) => t,
        Err(message) => return failed(message, 2),
    };
    let source = match std::fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => return failed(format!("{}: cannot read: {e}\n", input.display()), 1),
    };
    match convert_source(&input.display().to_string(), &source, target) {
        Ok(text) => deliver(output, text),
        Err(rendered) => failed(rendered, 2),
    }
}

fn resolve_target(input: &Path, to: Option<&str>) -> Result<Target, String> {
    match to {
        Some(value) => Target::parse(value),
        None => Target::for_input(input).ok_or_else(|| {
            format!(
                "{}: cannot tell the direction: name the file .rhl or .rhl.yaml, or pass --to rhl|yaml\n",
                input.display()
            )
        }),
    }
}

/// Write `text` to `output`, or return it as standard output.
fn deliver(output: Option<&Path>, text: String) -> Outcome {
    let Some(path) = output else {
        return Outcome {
            stdout: text,
            stderr: String::new(),
            exit: 0,
        };
    };
    match std::fs::write(path, text) {
        Ok(()) => Outcome {
            stdout: String::new(),
            stderr: String::new(),
            exit: 0,
        },
        Err(e) => failed(format!("{}: cannot write: {e}\n", path.display()), 1),
    }
}

fn failed(stderr: String, exit: i32) -> Outcome {
    Outcome {
        stdout: String::new(),
        stderr,
        exit,
    }
}

/// Which stage `ruchy transpile` stops at on an RHL file (spec RHL-001 §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emit {
    /// The lowered ruchy source (`--emit ruchy`).
    Ruchy,
    /// Rust, through ruchy's own parser and transpiler (the default).
    Rust,
}

impl Emit {
    /// Parse `--emit`; absent means Rust.
    ///
    /// # Errors
    ///
    /// Returns the message to print for a value other than `ruchy` or `rust`.
    pub fn parse(value: Option<&str>) -> Result<Self, String> {
        match value {
            None | Some("rust") => Ok(Self::Rust),
            Some("ruchy") => Ok(Self::Ruchy),
            Some(other) => Err(format!("--emit must be `ruchy` or `rust`, not `{other}`\n")),
        }
    }
}

/// `ruchy transpile <file.rhl | file.rhl.yaml> [--emit ruchy|rust] [-o out]`
/// (RHL-4): check, lower to ruchy, and by default transpile that ruchy to
/// Rust with the calls `ruchy transpile x.ruchy` makes. Exit 0 on success;
/// the check's exit code when the program does not check clean; 2 for a
/// lowering refusal (`RHL-L001`) or an unknown `--emit`; 1 when a file cannot
/// be read or written or ruchy rejects the lowered source. Diagnostics go to
/// standard error, so standard output is only code.
#[must_use]
pub fn transpile_file(input: &Path, output: Option<&Path>, emit: Option<&str>) -> Outcome {
    let emit = match Emit::parse(emit) {
        Ok(e) => e,
        Err(message) => return failed(message, 2),
    };
    let source = match std::fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => return failed(format!("{}: cannot read: {e}\n", input.display()), 1),
    };
    let file = input.display().to_string();
    let root = find_root(input);
    let lowered = if is_rhl_yaml(input) {
        lower_yaml(&file, &source, root.as_deref())
    } else {
        lower_source(&file, &source, root.as_deref())
    };
    let ruchy = match lowered {
        Ok(r) => r,
        Err(f) => return failed(render_lower_failure(&f), f.exit_code()),
    };
    let text = match emit {
        Emit::Ruchy => ruchy,
        Emit::Rust => match to_rust(&ruchy) {
            Ok(rust) => rust,
            Err(m) => return failed(format!("{file}: {m}\n"), 1),
        },
    };
    deliver(output.filter(|p| p.as_os_str() != "-"), text)
}

fn render_lower_failure(f: &LowerFailure) -> String {
    match f {
        LowerFailure::Check(report) => text(report),
        LowerFailure::Refused(d) => render_text(std::slice::from_ref(d)),
    }
}

/// `ruchy compile` on an RHL file: declined in RHL-4a, exit 2. A binary
/// needs `observe` and `apply`, whose bindings are RHL-4b's.
#[must_use]
pub fn compile_refusal(input: &Path) -> Outcome {
    failed(
        format!(
            "{}: `ruchy compile` does not take RHL yet: the observe/apply bindings land in RHL-4b; \
             `ruchy transpile` gives the Rust of `decide`\n",
            input.display()
        ),
        2,
    )
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
    #[test]
    fn test_rhl4_cli_emit_accepts_ruchy_and_rust_and_defaults_to_rust() {
        assert_eq!(Emit::parse(None), Ok(Emit::Rust));
        assert_eq!(Emit::parse(Some("rust")), Ok(Emit::Rust));
        assert_eq!(Emit::parse(Some("ruchy")), Ok(Emit::Ruchy));
        assert!(Emit::parse(Some("c")).is_err());
    }

    #[test]
    fn test_rhl4_cli_transpile_emits_ruchy_or_rust_and_declines_a_typo() {
        let path = corpus("docs/rhl/breaks/v2/valid/01-gx10-disk-watch.rhl");
        let ruchy = transpile_file(&path, None, Some("ruchy"));
        assert_eq!(ruchy.exit, 0, "{}", ruchy.stderr);
        assert!(ruchy
            .stdout
            .contains("fun decide(facts: Facts) -> Vec<Action>"));
        let rust = transpile_file(&path, None, None);
        assert_eq!(rust.exit, 0, "{}", rust.stderr);
        assert!(rust
            .stdout
            .contains("fn decide(facts: Facts) -> Vec<Action>"));
        assert_eq!(transpile_file(&path, None, Some("c")).exit, 2);
        let typo = corpus("docs/rhl/breaks/planted/typo-term/01-gx10-disk-watch/broken.rhl");
        let out = transpile_file(&typo, None, None);
        assert_eq!(out.exit, 2);
        assert!(
            out.stdout.is_empty() && out.stderr.contains("RHL-V002"),
            "{out:?}"
        );
    }

    #[test]
    fn test_rhl4_cli_compile_of_rhl_is_declined_with_exit_2() {
        let out = compile_refusal(Path::new("j.rhl"));
        assert_eq!(out.exit, 2);
        assert!(out.stderr.contains("RHL-4b"), "{}", out.stderr);
    }
}
