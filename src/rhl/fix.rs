//! `ruchy fix --safe` (spec RHL-001 §3.5, §5; roadmap row RHL-2).
//!
//! The checker marks a fix `safe` by RHL-1's local rule: it is the unique
//! candidate and the fixed program has no type or unit error on an edited
//! line. `fix --safe` is the global guard (plan `docs/rhl/rhlga-1-plan.md`
//! ruling Q5): it applies the safe fixes one at a time, re-checks the whole
//! file after each, and reverts any fix after which the file's error count
//! rose. It repeats until no safe fix applies, for at most as many rounds as
//! the file had safe fixes to begin with.

use super::check::{check, Report};
use super::cli::{is_rhl, json, text, Outcome, OutputFormat};
use super::diag::{apply_edits, Diagnostic, Edit, Fix, Severity};
use super::vocab::find_root;
use serde::Serialize;
use std::path::Path;

/// One fix `fix --safe` applied or reverted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Applied {
    /// The code of the diagnostic the fix repairs.
    pub code: String,
    /// The diagnostic's line.
    pub line: usize,
    /// The diagnostic's column.
    pub col: usize,
    /// What the fix does.
    pub title: String,
    /// Its edits, against the text as it was when the fix was tried.
    pub edits: Vec<Edit>,
}

/// What `fix --safe` did to one file: the check report of the result, plus
/// the fixes it applied and the ones it reverted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FixReport {
    /// The check of the fixed file.
    #[serde(flatten)]
    pub report: Report,
    /// The fixes that stayed, in the order applied.
    pub applied: Vec<Applied>,
    /// The fixes undone because the error count rose after them.
    pub reverted: Vec<Applied>,
    /// Rounds run; never more than the number of initial safe fixes.
    pub rounds: usize,
}

/// The fixed text and its report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fixed {
    /// The source after every fix that stayed.
    pub source: String,
    /// What was done.
    pub report: FixReport,
}

/// `fix --safe` on `source`, named `file`, with vocabularies under `root`.
#[must_use]
pub fn fix_safe(file: &str, source: &str, root: Option<&Path>) -> Fixed {
    fix_with(source, |text| check(file, text, root))
}

/// The fix engine over any checker: `check` maps a text to its report.
#[must_use]
pub fn fix_with<F: Fn(&str) -> Report>(source: &str, check: F) -> Fixed {
    let mut engine = Engine {
        text: source.to_string(),
        applied: Vec::new(),
        reverted: Vec::new(),
        check: &check,
    };
    let bound = safe_fixes(&check(source)).len();
    let mut rounds = 0;
    while rounds < bound && engine.round() {
        rounds += 1;
    }
    let report = check(&engine.text);
    Fixed {
        source: engine.text,
        report: FixReport {
            report,
            applied: engine.applied,
            reverted: engine.reverted,
            rounds,
        },
    }
}

/// The number of errors in `report`.
#[must_use]
pub fn error_count(report: &Report) -> usize {
    report
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count()
}

/// Every fix of `report` that `fix --safe` may try: marked safe, and on a
/// diagnostic with at most one candidate (two candidates are a choice, §3.5,
/// whatever the fix says). Last position first, so applying one leaves the
/// positions of the others valid.
fn safe_fixes(report: &Report) -> Vec<Applied> {
    let mut out: Vec<Applied> = report
        .diagnostics
        .iter()
        .filter(|d| d.candidates.len() <= 1)
        .flat_map(|d| {
            d.fixes
                .iter()
                .filter(|f| f.safe)
                .map(move |f| applied(d, f))
        })
        .collect();
    out.sort_by_key(|a| std::cmp::Reverse(last_position(&a.edits)));
    out
}

fn applied(d: &Diagnostic, f: &Fix) -> Applied {
    Applied {
        code: d.code.clone(),
        line: d.span.line,
        col: d.span.col,
        title: f.title.clone(),
        edits: f.edits.clone(),
    }
}

fn last_position(edits: &[Edit]) -> (usize, usize) {
    edits
        .iter()
        .map(|e| (e.span.line, e.span.col))
        .max()
        .unwrap_or((0, 0))
}

struct Engine<'c, F> {
    text: String,
    applied: Vec<Applied>,
    reverted: Vec<Applied>,
    check: &'c F,
}

impl<F: Fn(&str) -> Report> Engine<'_, F> {
    /// One round: try every safe fix of the current text that was not
    /// reverted before. True when at least one fix stayed.
    fn round(&mut self) -> bool {
        let report = (self.check)(&self.text);
        let mut errors = error_count(&report);
        let mut progress = false;
        for fix in safe_fixes(&report) {
            if self.reverted.iter().any(|r| r.edits == fix.edits) {
                continue;
            }
            let next = apply_edits(&self.text, &fix.edits);
            let after = error_count(&(self.check)(&next));
            if after > errors {
                self.reverted.push(fix);
            } else {
                self.text = next;
                errors = after;
                self.applied.push(fix);
                progress = true;
            }
        }
        progress
    }
}

/// `ruchy fix` over `.rhl` files. Only `--safe` exists (v0); without it the
/// verb is refused (exit 2). Each changed file is written in place. The exit
/// code is `ruchy check`'s on the result: the worst of the files.
#[must_use]
pub fn fix_files(paths: &[&Path], safe: bool, format: OutputFormat) -> Outcome {
    if !safe {
        let msg = "ruchy fix: only `--safe` exists for .rhl files: it applies the fixes `ruchy check` marks safe\n";
        return failure(msg.to_string(), 2);
    }
    if let Some(other) = paths.iter().find(|p| !is_rhl(p)) {
        let msg = format!(
            "{} is not an .rhl file; ruchy fix applies to .rhl files\n",
            other.display()
        );
        return failure(msg, 1);
    }
    let mut reports = Vec::new();
    let mut stderr = String::new();
    for path in paths {
        match fix_one(path) {
            Ok(r) => reports.push(r),
            Err(e) => stderr.push_str(&e),
        }
    }
    let io_exit = i32::from(!stderr.is_empty());
    let exit = reports
        .iter()
        .map(|r| r.report.exit_code())
        .fold(io_exit, i32::max);
    Outcome {
        stdout: render(&reports, format),
        stderr,
        exit,
    }
}

fn failure(stderr: String, exit: i32) -> Outcome {
    Outcome {
        stdout: String::new(),
        stderr,
        exit,
    }
}

/// Fix one file and write it back when it changed.
fn fix_one(path: &Path) -> Result<FixReport, String> {
    let shown = path.display();
    let source =
        std::fs::read_to_string(path).map_err(|e| format!("{shown}: cannot read: {e}\n"))?;
    let root = find_root(path);
    let fixed = fix_safe(&shown.to_string(), &source, root.as_deref());
    if fixed.source != source {
        std::fs::write(path, &fixed.source).map_err(|e| format!("{shown}: cannot write: {e}\n"))?;
    }
    Ok(fixed.report)
}

fn render(reports: &[FixReport], format: OutputFormat) -> String {
    match (format, reports) {
        (OutputFormat::Json, [one]) => json(one),
        (OutputFormat::Json, many) => json(&many),
        (OutputFormat::Text, many) => many.iter().map(render_text).collect(),
    }
}

/// For a person: one line per fix applied or reverted, then the check.
fn render_text(r: &FixReport) -> String {
    let file = &r.report.file;
    let mut out = String::new();
    for a in &r.applied {
        out.push_str(&format!(
            "{file}:{}:{}: fixed[{}]: {}\n",
            a.line, a.col, a.code, a.title
        ));
    }
    for a in &r.reverted {
        out.push_str(&format!(
            "{file}:{}:{}: reverted[{}]: {} (the file's error count rose)\n",
            a.line, a.col, a.code, a.title
        ));
    }
    out + &text(&r.report)
}

#[cfg(test)]
#[path = "fix_tests.rs"]
mod fix_tests;
