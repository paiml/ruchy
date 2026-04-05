#![allow(missing_docs)]
//! CLI integration tests for `ruchy tier` (PROVABILITY-001).
//!
//! Exercises the tier-distribution reporter on synthetic projects.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
fn test_tier_help_available() {
    ruchy_cmd()
        .arg("tier")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("tier distribution"));
}

#[test]
fn test_tier_single_bronze_file() {
    let tmp = TempDir::new().unwrap();
    let f = tmp.path().join("a.ruchy");
    fs::write(&f, "fun f() { 1 }").unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(&f)
        .assert()
        .success()
        .stdout(predicate::str::contains("functions: 1"))
        .stdout(predicate::str::contains("bronze:"));
}

#[test]
fn test_tier_json_output() {
    let tmp = TempDir::new().unwrap();
    let f = tmp.path().join("a.ruchy");
    fs::write(&f, "fun f() { 1 }\nfun g() { 2 }").unwrap();

    let output = ruchy_cmd()
        .arg("tier")
        .arg(&f)
        .arg("--json")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"functions\":2"));
    assert!(stdout.contains("\"bronze\":2"));
    assert!(stdout.contains("\"non_bronze_pct\":0.00"));
}

#[test]
fn test_tier_scans_directory_recursively() {
    let tmp = TempDir::new().unwrap();
    let sub = tmp.path().join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(tmp.path().join("top.ruchy"), "fun t() { 0 }").unwrap();
    fs::write(sub.join("deep.ruchy"), "fun d() { 0 }").unwrap();

    let output = ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--json")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"files\":2"));
    assert!(stdout.contains("\"functions\":2"));
}

#[test]
fn test_tier_ignores_non_ruchy_files() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "fun a() { 1 }").unwrap();
    fs::write(tmp.path().join("notes.txt"), "ignore me").unwrap();
    fs::write(tmp.path().join("script.py"), "print('skip')").unwrap();

    let output = ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--json")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"files\":1"));
}

#[test]
fn test_tier_list_flag_enumerates_functions() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("a.ruchy"),
        "fun alpha() { 1 }\n#[bronze]\nfun beta() { 2 }",
    )
    .unwrap();

    let output = ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--list")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("alpha"), "must list `alpha`: {stdout}");
    assert!(stdout.contains("beta"), "must list `beta`: {stdout}");
}

#[test]
fn test_tier_fail_under_breach_exits_nonzero() {
    let tmp = TempDir::new().unwrap();
    // Three Bronze functions -> non_bronze_pct = 0.0
    fs::write(
        tmp.path().join("a.ruchy"),
        "fun a() { 1 }\nfun b() { 2 }\nfun c() { 3 }",
    )
    .unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-under")
        .arg("50")
        .assert()
        .failure()
        .stderr(predicate::str::contains("F1 falsifier breach"));
}

#[test]
fn test_tier_fail_under_threshold_met_exits_zero() {
    let tmp = TempDir::new().unwrap();
    // All Silver -> non_bronze_pct = 100.0
    fs::write(
        tmp.path().join("a.ruchy"),
        "fun a() requires true ensures true { 1 }\nfun b() requires true { 2 }",
    )
    .unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-under")
        .arg("50")
        .assert()
        .success();
}

#[test]
fn test_tier_fail_under_zero_always_passes() {
    // Even 0% non-bronze should pass with threshold 0.0
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "fun a() { 1 }").unwrap();
    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-under")
        .arg("0")
        .assert()
        .success();
}

#[test]
fn test_tier_fail_on_totality_violation_triggers_on_gold_without_total() {
    let tmp = TempDir::new().unwrap();
    // @gold decorator + contracts -> Gold tier; no @total -> violation.
    fs::write(
        tmp.path().join("a.ruchy"),
        "#[gold]\nfun compute() requires true ensures true { 1 }",
    )
    .unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-on-totality-violation")
        .assert()
        .failure()
        .stderr(predicate::str::contains("§14.10.6 breach"));
}

#[test]
fn test_tier_fail_on_totality_violation_passes_when_gold_has_total() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("a.ruchy"),
        "#[gold]\n#[total]\nfun compute() requires true ensures true { 1 }",
    )
    .unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-on-totality-violation")
        .assert()
        .success();
}

#[test]
fn test_tier_fail_on_totality_violation_passes_on_silver_unmarked() {
    // Silver functions without @total are NOT violations — §14.10.6 only
    // applies to Gold/Platinum.
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("a.ruchy"),
        "fun f() requires true { 1 }",
    )
    .unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-on-totality-violation")
        .assert()
        .success();
}

#[test]
fn test_tier_empty_directory() {
    let tmp = TempDir::new().unwrap();
    let output = ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--json")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"files\":0"));
    assert!(stdout.contains("\"functions\":0"));
}
