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
use super::lower::{lower_source_as, lower_yaml_as, to_rust, Form, LowerFailure};
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
    check_source(&path.display().to_string(), source, root.as_deref())
}

/// The check `ruchy check` runs on `source`, named `file`: the YAML checker
/// for a `.rhl.yaml` name, the RHL text checker otherwise.
#[must_use]
pub fn check_source(file: &str, source: &str, root: Option<&Path>) -> Report {
    if is_rhl_yaml(Path::new(file)) {
        check_yaml(file, source, root)
    } else {
        check(file, source, root)
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
    /// The job's unit contract, a `pv` contract (`--emit contract`, RHL-5b).
    Contract,
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
            Some("contract") => Ok(Self::Contract),
            Some(other) => Err(format!(
                "--emit must be `ruchy`, `rust` or `contract`, not `{other}`\n"
            )),
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
    if emit == Emit::Contract {
        return match contract_of(input) {
            Ok(c) => deliver(output.filter(|p| p.as_os_str() != "-"), c),
            Err(out) => out,
        };
    }
    let ruchy = match lower_file(input, Form::Decide) {
        Ok(r) => r,
        Err(out) => return out,
    };
    let file = input.display().to_string();
    let text = match emit {
        Emit::Ruchy | Emit::Contract => ruchy,
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

/// Read, check and lower `input` to `form`; the outcome to print otherwise.
fn lower_file(input: &Path, form: Form) -> Result<String, Outcome> {
    let source = std::fs::read_to_string(input)
        .map_err(|e| failed(format!("{}: cannot read: {e}\n", input.display()), 1))?;
    let file = input.display().to_string();
    let root = find_root(input);
    lower_named(form, &file, &source, root.as_deref())
        .map_err(|f| failed(render_lower_failure(&f), f.exit_code()))
}

/// Check and lower `source`, named `file`, to `form`: the `.rhl.yaml` path
/// for a YAML name, the RHL text path otherwise. What `ruchy transpile` and
/// `ruchy compile` lower with.
///
/// # Errors
///
/// As [`lower_source_as`] and [`lower_yaml_as`].
pub fn lower_named(
    form: Form,
    file: &str,
    source: &str,
    root: Option<&Path>,
) -> Result<String, LowerFailure> {
    if is_rhl_yaml(Path::new(file)) {
        lower_yaml_as(form, file, source, root)
    } else {
        lower_source_as(form, file, source, root)
    }
}

/// `ruchy compile <file.rhl | file.rhl.yaml> [-o bin]` (RHL-4): lower the job
/// with its runtime bindings, `observe`, `apply` and `main`, and build it with
/// the call `ruchy compile x.ruchy` makes. Exit 0 when built; the check's exit
/// code, 2 for `RHL-L001`/`RHL-L002`, 1 when the file cannot be read or the
/// build fails.
#[must_use]
pub fn compile_file(input: &Path, output: &Path) -> Outcome {
    let ruchy = match lower_file(input, Form::Program) {
        Ok(r) => r,
        Err(out) => return out,
    };
    let contract = match write_contract(input, output) {
        Ok(c) => c,
        Err(out) => return out,
    };
    let built = build_job(input, &ruchy, output);
    if built.exit != 0 {
        return built;
    }
    let receipt = super::receipt::build(input, &ruchy, &contract, run_examples(input));
    match receipt
        .and_then(|r| std::fs::write(receipt_path(output), r.to_json()).map_err(|e| e.to_string()))
    {
        Ok(()) => built,
        Err(e) => failed(
            format!("{}: cannot write the receipt: {e}\n", input.display()),
            1,
        ),
    }
}

/// `<out>.contract.yaml`: where `ruchy compile -o <out>` writes the unit contract.
#[must_use]
pub fn contract_path(output: &Path) -> std::path::PathBuf {
    suffixed(output, ".contract.yaml")
}

/// `<out>.receipt.json`: where `ruchy compile -o <out>` writes the receipt.
#[must_use]
pub fn receipt_path(output: &Path) -> std::path::PathBuf {
    suffixed(output, ".receipt.json")
}

fn suffixed(output: &Path, suffix: &str) -> std::path::PathBuf {
    let mut name = output.as_os_str().to_os_string();
    name.push(suffix);
    name.into()
}

/// The unit contract of `input` (RHL-5b), or the outcome to print.
fn contract_of(input: &Path) -> Result<String, Outcome> {
    let source = std::fs::read_to_string(input)
        .map_err(|e| failed(format!("{}: cannot read: {e}\n", input.display()), 1))?;
    let file = input.display().to_string();
    let root = find_root(input);
    let emitted = if is_rhl_yaml(input) {
        super::contract::unit_contract_yaml(&file, &source, root.as_deref())
    } else {
        super::contract::unit_contract(&file, &source, root.as_deref())
    };
    emitted.map_err(|f| failed(render_lower_failure(&f), f.exit_code()))
}

/// Emit and write `<out>.contract.yaml` before anything is built: a unit
/// without its contract is refused with `RHL-C002` (§9.3).
fn write_contract(input: &Path, output: &Path) -> Result<String, Outcome> {
    let contract = contract_of(input)?;
    let path = contract_path(output);
    std::fs::write(&path, &contract).map_err(|e| {
        let m = format!(
            "{}: error[{}]: unit contract could not be emitted: cannot write {}: {e}\n",
            input.display(),
            super::codes::C002,
            path.display()
        );
        failed(m, 2)
    })?;
    Ok(contract)
}

/// Build the lowered `ruchy` to the binary `output`.
fn build_job(input: &Path, ruchy: &str, output: &Path) -> Outcome {
    let options = crate::backend::CompileOptions {
        output: output.to_path_buf(),
        ..crate::backend::CompileOptions::default()
    };
    match crate::backend::compile_source_to_binary(ruchy, &options) {
        Ok(_) => Outcome {
            stdout: String::new(),
            stderr: format!("{}: built {}\n", input.display(), output.display()),
            exit: 0,
        },
        Err(e) => failed(format!("{}: {e:#}\n", input.display()), 1),
    }
}

/// `ruchy run <file.rhl | file.rhl.yaml> [--apply]` (RHL-4): compile to a
/// temporary directory and execute, because ruchy's interpreter does not run
/// processes. Without `--apply` the job only prints its plan (§9.4). The
/// outcome is the job's own output and exit code.
#[must_use]
pub fn run_file(input: &Path, apply: bool) -> Outcome {
    let dir = match tempfile::tempdir() {
        Ok(d) => d,
        Err(e) => return failed(format!("cannot create a build directory: {e}\n"), 1),
    };
    let bin = dir.path().join("job");
    let ruchy = match lower_file(input, Form::Program) {
        Ok(r) => r,
        Err(out) => return out,
    };
    let built = build_job(input, &ruchy, &bin);
    if built.exit != 0 {
        return built;
    }
    let mut cmd = std::process::Command::new(&bin);
    if apply {
        cmd.arg("--apply");
    }
    match cmd.output() {
        Ok(o) => Outcome {
            stdout: String::from_utf8_lossy(&o.stdout).to_string(),
            stderr: String::from_utf8_lossy(&o.stderr).to_string(),
            exit: o.status.code().unwrap_or(1),
        },
        Err(e) => failed(format!("{}: cannot run the job: {e}\n", input.display()), 1),
    }
}

/// One example's result, as the test program reported it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ExampleResult {
    /// The example's name.
    pub name: String,
    /// True when every `then` and every `expect` held.
    pub ok: bool,
    /// The first `then` or `expect` that did not hold.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed: Option<String>,
}

/// `ruchy test <file.rhl | file.rhl.yaml>` (RHL-5): lower the job with one
/// test per `example` and no `observe`/`apply`, build it with the call `ruchy
/// compile x.ruchy` makes, run it, and report each example. Exit 0 when every
/// example holds (a job with no examples reports `0 examples`), 1 when one
/// fails or the build fails; the check's code, or 2 for a lowering refusal.
#[must_use]
pub fn test_file(input: &Path, format: OutputFormat) -> Outcome {
    let results = match run_examples(input) {
        Ok(r) => r,
        Err(out) => return out,
    };
    let exit = i32::from(results.iter().any(|r| !r.ok));
    let file = input.display().to_string();
    let text = match format {
        OutputFormat::Text => test_text(&file, &results),
        OutputFormat::Json => test_json(&file, &results, exit),
    };
    Outcome {
        stdout: text,
        stderr: String::new(),
        exit,
    }
}

/// Lower `input`'s examples to the test program, build it, run it, and read
/// each example's result: the runner `ruchy test` and the receipt share.
fn run_examples(input: &Path) -> Result<Vec<ExampleResult>, Outcome> {
    let ruchy = lower_file(input, Form::Tests)?;
    let names = example_names_of(input)?;
    let stdout = build_and_run(input, &ruchy)?;
    Ok(example_results(&names, &stdout))
}

fn example_names_of(input: &Path) -> Result<Vec<String>, Outcome> {
    let source = std::fs::read_to_string(input)
        .map_err(|e| failed(format!("{}: cannot read: {e}\n", input.display()), 1))?;
    let program = if is_rhl_yaml(input) {
        yaml::from_yaml(&source).ok()
    } else {
        super::parse(&source).ok()
    };
    program
        .map(|p| super::lower::example_names(&p))
        .ok_or_else(|| failed(format!("{}: does not parse\n", input.display()), 1))
}

/// Build the test program in a temporary directory and run it with an empty
/// environment: its standard output.
fn build_and_run(input: &Path, ruchy: &str) -> Result<String, Outcome> {
    let dir = tempfile::tempdir()
        .map_err(|e| failed(format!("cannot create a build directory: {e}\n"), 1))?;
    let bin = dir.path().join("examples");
    let options = crate::backend::CompileOptions {
        output: bin.clone(),
        ..crate::backend::CompileOptions::default()
    };
    crate::backend::compile_source_to_binary(ruchy, &options)
        .map_err(|e| failed(format!("{}: {e:#}\n", input.display()), 1))?;
    let out = std::process::Command::new(&bin)
        .env_clear()
        .output()
        .map_err(|e| {
            failed(
                format!("{}: cannot run the tests: {e}\n", input.display()),
                1,
            )
        })?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Each example's result from the `rhl-example <n> ok|FAILED <what>` lines;
/// an example with no line failed (the test program stopped before it).
fn example_results(names: &[String], stdout: &str) -> Vec<ExampleResult> {
    names
        .iter()
        .enumerate()
        .map(|(n, name)| {
            let prefix = format!("rhl-example {n} ");
            let line = stdout.lines().find_map(|l| l.strip_prefix(&prefix));
            let failed = match line {
                Some("ok") => None,
                Some(rest) => Some(rest.strip_prefix("FAILED ").unwrap_or(rest).to_string()),
                None => Some("no result: the test program stopped".to_string()),
            };
            ExampleResult {
                name: name.clone(),
                ok: failed.is_none(),
                failed,
            }
        })
        .collect()
}

fn test_text(file: &str, results: &[ExampleResult]) -> String {
    let mut out = String::new();
    for r in results {
        match &r.failed {
            None => out.push_str(&format!("example \"{}\" … ok\n", r.name)),
            Some(what) => out.push_str(&format!("example \"{}\" … FAILED ({what})\n", r.name)),
        }
    }
    let failures = results.iter().filter(|r| !r.ok).count();
    let noun = if results.len() == 1 {
        "example"
    } else {
        "examples"
    };
    out.push_str(&format!(
        "{file}: {} {noun}, {} passed, {failures} failed\n",
        results.len(),
        results.len() - failures
    ));
    out
}

fn test_json(file: &str, results: &[ExampleResult], exit: i32) -> String {
    let failures = results.iter().filter(|r| !r.ok).count();
    json(&serde_json::json!({
        "file": file,
        "examples": results,
        "passed": results.len() - failures,
        "failed": failures,
        "exit": exit,
    }))
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
    fn test_rhl4c_cli_compile_and_run_refuse_an_unbound_term_with_l002_exit_2() {
        let path = corpus("docs/rhl/breaks/v2/valid/03-gx12-runner-load-watch.rhl");
        let dir = tempfile::tempdir().expect("tempdir");
        let out = compile_file(&path, &dir.path().join("j"));
        assert_eq!(out.exit, 2, "{out:?}");
        assert!(out.stderr.contains("RHL-L002"), "{}", out.stderr);
        assert!(out.stderr.contains("`runner load of`"), "{}", out.stderr);
        assert!(!dir.path().join("j").exists());
        let run = run_file(&path, true);
        assert_eq!(run.exit, 2, "{run:?}");
        assert!(run.stderr.contains("RHL-L002"), "{}", run.stderr);
    }

    #[test]
    fn test_rhl4c_cli_compile_declines_a_typo_with_the_checks_exit() {
        let typo = corpus("docs/rhl/breaks/planted/typo-term/01-gx10-disk-watch/broken.rhl");
        let dir = tempfile::tempdir().expect("tempdir");
        let out = compile_file(&typo, &dir.path().join("j"));
        assert_eq!(out.exit, 2);
        assert!(out.stderr.contains("RHL-V002"), "{}", out.stderr);
        let missing = compile_file(Path::new("/nonexistent/x.rhl"), &dir.path().join("j"));
        assert_eq!(missing.exit, 1);
    }
}
