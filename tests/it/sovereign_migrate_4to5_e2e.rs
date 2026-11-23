#![allow(missing_docs)]
//! End-to-end migration suite for `ruchy migrate-4to5`.
//!
//! Verifies Section 10 criterion #12: the migrate-4to5 tool handles 100% of
//! keyword-conflict cases in synthetic 4.x code. A single synthetic project
//! contains one file per new-5.0 keyword, each using the keyword as an
//! identifier. After migration the scan must report zero remaining conflicts.
//!
//! Ticket: [EMBED-002] migrate-4to5 synthetic 4.x acceptance suite.

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// 4.x identifiers that become reserved in 5.0.
const RESERVED_KEYWORDS: &[&str] = &[
    "requires",
    "ensures",
    "invariant",
    "decreases",
    "infra",
    "signal",
    "yield",
];

fn build_synthetic_4x_project(dir: &std::path::Path) {
    for kw in RESERVED_KEYWORDS {
        // Each file assigns `let <keyword> = N` as a value. These parse in 4.x
        // but are rejected by the 5.0 parser until renamed.
        let src = format!("let {kw} = 1\n");
        fs::write(dir.join(format!("{kw}.ruchy")), src).unwrap();
    }
}

#[test]
fn test_migrate_dry_run_reports_all_seven_conflicts() {
    let tmp = TempDir::new().unwrap();
    build_synthetic_4x_project(tmp.path());

    let output = ruchy_cmd()
        .arg("migrate-4to5")
        .arg("--dry-run")
        .arg(tmp.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "migrate dry-run must succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    for kw in RESERVED_KEYWORDS {
        assert!(
            stdout.contains(kw),
            "dry-run output must mention `{kw}`, got: {stdout}"
        );
    }
}

#[test]
fn test_migrate_dry_run_does_not_modify_files() {
    let tmp = TempDir::new().unwrap();
    build_synthetic_4x_project(tmp.path());

    let before: Vec<(String, String)> = RESERVED_KEYWORDS
        .iter()
        .map(|kw| {
            let path = tmp.path().join(format!("{kw}.ruchy"));
            (kw.to_string(), fs::read_to_string(&path).unwrap())
        })
        .collect();

    ruchy_cmd()
        .arg("migrate-4to5")
        .arg("--dry-run")
        .arg(tmp.path())
        .assert()
        .success();

    for (kw, original) in &before {
        let path = tmp.path().join(format!("{kw}.ruchy"));
        let after = fs::read_to_string(&path).unwrap();
        assert_eq!(&after, original, "dry-run must not modify {kw}.ruchy");
    }
}

#[test]
fn test_migrate_apply_removes_all_seven_conflicts() {
    let tmp = TempDir::new().unwrap();
    build_synthetic_4x_project(tmp.path());

    // Apply migration (no --dry-run).
    ruchy_cmd()
        .arg("migrate-4to5")
        .arg(tmp.path())
        .assert()
        .success();

    // Verify each file no longer contains the bare keyword as an identifier.
    for kw in RESERVED_KEYWORDS {
        let path = tmp.path().join(format!("{kw}.ruchy"));
        let contents = fs::read_to_string(&path).unwrap();
        // The keyword must have been replaced by a longer identifier with a
        // suffix (e.g. `requires_val`). The bare `let <kw> =` is gone.
        assert!(
            !contents.contains(&format!("let {kw} =")),
            "post-migration file {kw}.ruchy still contains bare keyword: {contents}"
        );
    }

    // Running a second dry-run scan must now report zero renames.
    let output = ruchy_cmd()
        .arg("migrate-4to5")
        .arg("--dry-run")
        .arg(tmp.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("0 identifiers renamed") || stdout.contains("0 files modified"),
        "second-pass scan must report zero conflicts, got: {stdout}"
    );
}

#[test]
fn test_migrate_apply_leaves_non_conflicting_code_untouched() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("clean.ruchy");
    let original = "let x = 1\nlet y = 2\n";
    fs::write(&path, original).unwrap();

    ruchy_cmd()
        .arg("migrate-4to5")
        .arg(tmp.path())
        .assert()
        .success();

    let after = fs::read_to_string(&path).unwrap();
    assert_eq!(after, original, "non-conflicting file must be untouched");
}
