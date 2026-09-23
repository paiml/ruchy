//! RHL-2 fix engine tests: `ruchy fix --safe` applies safe fixes only,
//! reverts a fix that raises the error count (plan ruling Q5), is idempotent,
//! and runs a bounded number of rounds.

use super::*;
use crate::rhl::check::{check, Report};
use crate::rhl::codes;
use crate::rhl::diag::{Candidate, Diagnostic, Edit, Fix, LineSpan};
use crate::rhl::tree::Span;
use std::cell::Cell;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    crate::rhl::repo_root()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Every `broken.rhl` of a v2 planted-break class, with its base program.
fn v2_breaks(class: &str) -> Vec<(PathBuf, PathBuf)> {
    let dir = root().join("docs/rhl/breaks/v2/planted").join(class);
    let mut out: Vec<(PathBuf, PathBuf)> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .map(|d| {
            let meta = read(&d.join("break.yaml"));
            let base = meta
                .lines()
                .find_map(|l| l.strip_prefix("base: "))
                .expect("break.yaml names its base")
                .trim()
                .to_string();
            (d.join("broken.rhl"), root().join(base))
        })
        .collect();
    out.sort();
    assert_eq!(out.len(), 12, "{class}: 12 breaks");
    out
}

fn fix_file(path: &Path) -> Fixed {
    let source = read(path);
    fix_safe(&path.display().to_string(), &source, Some(&root()))
}

/// A diagnostic at `line` whose one fix replaces the whole line with `text`.
fn diag(line: usize, text: &str, safe: bool, candidates: usize) -> Diagnostic {
    let mut d = Diagnostic::error(codes::V002, "t.rhl", "", Span::new(0, 0), "m".into());
    d.span = LineSpan {
        line,
        col: 1,
        len: 1,
    };
    d.candidates = (0..candidates)
        .map(|i| Candidate {
            term: format!("c{i}"),
            vocabulary: "t v1".into(),
            distance: 1,
        })
        .collect();
    d.fixes.push(Fix {
        title: format!("replace with `{text}`"),
        safe,
        edits: vec![Edit {
            span: d.span,
            text: text.to_string(),
        }],
    });
    d
}

/// A report with one error per lower-case letter left in `text`, each with a
/// safe fix that upper-cases it — unless `worse` names a letter whose fix
/// is followed by two new errors (`!` marks them).
fn letters_report(text: &str, worse: char) -> Report {
    let mut diags = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let c = line.chars().next().unwrap_or(' ');
        if c.is_ascii_lowercase() {
            diags.push(diag(i + 1, &c.to_ascii_uppercase().to_string(), true, 1));
        }
        if c == worse.to_ascii_uppercase() {
            diags.push(diag(i + 1, "!", false, 0));
            diags.push(diag(i + 1, "!", false, 0));
        }
    }
    Report::for_test("t.rhl", diags)
}

// ---------------------------------------------------------------- the engine, on synthetic reports

#[test]
fn test_rhl2_fix_applies_every_safe_fix() {
    let fixed = fix_with("a\nb\nc\n", |t| letters_report(t, ' '));
    assert_eq!(fixed.source, "A\nB\nC\n");
    assert_eq!(fixed.report.applied.len(), 3);
    assert!(fixed.report.reverted.is_empty());
}

#[test]
fn test_rhl2_fix_leaves_an_unsafe_fix_untouched() {
    let check = |t: &str| {
        let mut d = diag(1, "X", false, 1);
        if t.starts_with('X') {
            d.fixes.clear();
        }
        Report::for_test("t.rhl", vec![d])
    };
    let fixed = fix_with("a\n", check);
    assert_eq!(fixed.source, "a\n");
    assert!(fixed.report.applied.is_empty());
}

#[test]
fn test_rhl2_fix_never_applies_a_fix_with_two_candidates() {
    let fixed = fix_with("a\n", |_| {
        Report::for_test("t.rhl", vec![diag(1, "X", true, 2)])
    });
    assert_eq!(fixed.source, "a\n", "two candidates: the fix is a guess");
    assert!(fixed.report.applied.is_empty());
}

#[test]
fn test_rhl2_fix_reverts_a_fix_after_which_the_error_count_rose() {
    let fixed = fix_with("a\nb\nc\n", |t| letters_report(t, 'b'));
    assert_eq!(fixed.source, "A\nb\nC\n", "the fix of `b` adds two errors");
    assert_eq!(fixed.report.reverted.len(), 1);
    assert_eq!(fixed.report.reverted[0].line, 2);
    assert_eq!(fixed.report.applied.len(), 2);
}

#[test]
fn test_rhl2_fix_is_idempotent() {
    let once = fix_with("a\nb\nc\n", |t| letters_report(t, 'b'));
    let twice = fix_with(&once.source, |t| letters_report(t, 'b'));
    assert_eq!(twice.source, once.source);
    assert!(twice.report.applied.is_empty());
}

#[test]
fn test_rhl2_fix_rounds_are_bounded_by_the_initial_safe_fixes() {
    // Every fix proposes another: without a bound this never ends.
    let calls = Cell::new(0usize);
    let check = |t: &str| {
        calls.set(calls.get() + 1);
        let n = t.lines().count();
        Report::for_test("t.rhl", vec![diag(n, &format!("x\n{n}"), true, 1)])
    };
    let fixed = fix_with("x\n", check);
    assert_eq!(fixed.report.rounds, 1, "one initial safe fix, one round");
    assert_eq!(fixed.report.applied.len(), 1);
    assert!(calls.get() < 10, "{} checks", calls.get());
}

#[test]
fn test_rhl2_fix_with_no_safe_fix_runs_no_round() {
    let fixed = fix_with("a\n", |_| Report::for_test("t.rhl", Vec::new()));
    assert_eq!(fixed.report.rounds, 0);
    assert_eq!(fixed.source, "a\n");
}

// ---------------------------------------------------------------- the engine, on real programs

#[test]
fn test_rhl2_fix_repairs_every_v2_typo_term_break_to_its_base() {
    for (broken, base) in v2_breaks("typo-term") {
        let fixed = fix_file(&broken);
        assert_eq!(fixed.source, read(&base), "{}", broken.display());
        assert_eq!(fixed.report.report.exit_code(), 0, "{}", broken.display());
        assert_eq!(fixed.report.applied.len(), 1, "{}", broken.display());
    }
}

#[test]
fn test_rhl2_fix_leaves_every_v2_two_candidate_break_unchanged() {
    for (broken, _) in v2_breaks("two-candidate-typo") {
        let fixed = fix_file(&broken);
        assert_eq!(fixed.source, read(&broken), "{}", broken.display());
        assert!(fixed.report.applied.is_empty());
        assert!(fixed
            .report
            .report
            .diagnostics
            .iter()
            .any(|d| d.code == codes::V003));
    }
}

#[test]
fn test_rhl2_fix_reverts_a_real_fix_that_exposes_two_unit_errors() {
    // `hou` → `hour` is safe at the edit site (RHL-1's local rule), but the
    // Duration it gives is compared with a Size twice: two new errors for one.
    let src = "use vocabulary fleet v2\n\njob \"d\"\n  every 1 hour\n  may read disk\n\n  let t be 10 hou\n\n  expect disk free of \"/\" is at least t\n  expect disk free of \"/\" is above t\nend\n";
    let before = check("t.rhl", src, Some(&root()));
    assert!(
        before
            .diagnostics
            .iter()
            .any(|d| d.fixes.iter().any(|f| f.safe)),
        "check proposes the fix as safe: {before:#?}"
    );
    let fixed = fix_safe("t.rhl", src, Some(&root()));
    assert_eq!(fixed.source, src, "the fix raised the error count");
    assert_eq!(fixed.report.reverted.len(), 1, "{:#?}", fixed.report);
}

#[test]
fn test_rhl2_fix_json_is_the_report_plus_applied() {
    let broken = &v2_breaks("typo-term")[0].0;
    let fixed = fix_file(broken);
    let v = serde_json::to_value(&fixed.report).expect("serializable");
    for key in ["file", "verdict", "diagnostics", "unverified", "applied"] {
        assert!(v.get(key).is_some(), "missing `{key}` in {v}");
    }
    assert_eq!(v["verdict"], "pass");
    assert_eq!(v["applied"][0]["code"], "RHL-V002");
    assert_eq!(v["applied"][0]["edits"][0]["text"], "disk free of");
}

// ---------------------------------------------------------------- the verb

#[test]
fn test_rhl2_fix_files_rewrites_in_place_and_exits_as_check() {
    let tmp = tempfile::tempdir().expect("tempdir");
    for sub in ["vocab", "contracts"] {
        copy_dir(&root().join(sub), &tmp.path().join(sub));
    }
    let (broken, base) = &v2_breaks("typo-term")[1];
    let file = tmp.path().join("x.rhl");
    std::fs::copy(broken, &file).expect("copy");
    let out = fix_files(&[file.as_path()], true, OutputFormat::Text);
    assert_eq!(out.exit, 0, "{out:?}");
    assert!(out.stdout.contains("fixed[RHL-V002]"), "{}", out.stdout);
    assert_eq!(read(&file), read(base));
    let again = fix_files(&[file.as_path()], true, OutputFormat::Json);
    assert_eq!(again.exit, 0);
    let v: serde_json::Value = serde_json::from_str(&again.stdout).expect("JSON");
    assert_eq!(v["applied"].as_array().map(Vec::len), Some(0));
    assert_eq!(read(&file), read(base), "idempotent");
}

#[test]
fn test_rhl2_fix_without_safe_is_refused_and_writes_nothing() {
    let (broken, _) = &v2_breaks("typo-term")[0];
    let out = fix_files(&[broken.as_path()], false, OutputFormat::Text);
    assert_eq!(out.exit, 2);
    assert!(out.stderr.contains("--safe"), "{}", out.stderr);
}

#[test]
fn test_rhl2_fix_of_a_non_rhl_file_is_exit_1() {
    let out = fix_files(&[Path::new("a.ruchy")], true, OutputFormat::Text);
    assert_eq!(out.exit, 1);
    assert!(out.stderr.contains("not an .rhl file"), "{}", out.stderr);
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("mkdir");
    for entry in std::fs::read_dir(from)
        .expect("read_dir")
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.is_file() {
            std::fs::copy(&path, to.join(entry.file_name())).expect("copy");
        }
    }
}
