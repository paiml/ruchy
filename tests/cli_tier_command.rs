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
fn test_tier_list_marks_pub_functions() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("a.ruchy"),
        "pub fun exposed() { 1 }\nfun hidden() { 2 }",
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
    // `pub` marker must appear on the `exposed` line
    let exposed_line = stdout
        .lines()
        .find(|l| l.contains("exposed"))
        .expect("exposed should be listed");
    assert!(exposed_line.contains("pub"), "exposed line: {exposed_line}");
    let hidden_line = stdout
        .lines()
        .find(|l| l.contains("hidden"))
        .expect("hidden should be listed");
    assert!(!hidden_line.contains("pub"), "hidden line: {hidden_line}");
}

#[test]
fn test_tier_json_list_emits_functions_array() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("a.ruchy"),
        "pub fun exposed() { 1 }\nfun hidden() { 2 }",
    )
    .unwrap();

    let output = ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--json")
        .arg("--list")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // First line: aggregate JSON. Second line: functions array.
    let mut lines = stdout.lines();
    let aggr = lines.next().expect("aggregate line");
    let funcs = lines.next().expect("functions line");
    assert!(aggr.starts_with('{') && aggr.contains("\"functions\":2"));
    assert!(funcs.starts_with('[') && funcs.ends_with(']'));
    assert!(funcs.contains("\"name\":\"exposed\""));
    assert!(funcs.contains("\"name\":\"hidden\""));
    assert!(funcs.contains("\"pub\":true"));
    assert!(funcs.contains("\"pub\":false"));
}

#[test]
fn test_tier_json_without_list_emits_only_aggregate() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "pub fun a() { 1 }").unwrap();
    let output = ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--json")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Only one line of output (no functions array).
    assert_eq!(stdout.trim().lines().count(), 1);
    assert!(!stdout.contains("\"name\":"));
}

#[test]
fn test_tier_summary_contains_pub_bronze_line() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "pub fun a() { 1 }").unwrap();
    let output = ruchy_cmd().arg("tier").arg(tmp.path()).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("public API (F4 proxy)"));
    assert!(stdout.contains("pub Bronze: 1"));
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
fn test_tier_fail_under_f1_triggers_on_trivial_contracts() {
    let tmp = TempDir::new().unwrap();
    // Two functions: one trivial, one non-trivial -> F1 = 50%.
    fs::write(
        tmp.path().join("a.ruchy"),
        "fun trivial() requires true ensures true { 1 }\n\
         fun real(x: i32) requires x > 0 ensures x > 0 { x }",
    )
    .unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-under-f1")
        .arg("80")
        .assert()
        .failure()
        .stderr(predicate::str::contains("§14.5 F1 breach"));
}

#[test]
fn test_tier_fail_under_f1_passes_when_all_non_trivial() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("a.ruchy"),
        "fun real(x: i32) requires x > 0 ensures x > 0 { x }",
    )
    .unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-under-f1")
        .arg("100")
        .assert()
        .success();
}

#[test]
fn test_tier_fail_under_f1_skipped_when_no_contracts() {
    // No contract-bearing functions -> F1 not applicable, must not fail.
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "fun f() { 1 }").unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-under-f1")
        .arg("100")
        .assert()
        .success();
}

#[test]
fn test_tier_fail_exempt_density_above_triggers_on_high_density() {
    let tmp = TempDir::new().unwrap();
    // 2 exemptions in ~5 LoC -> density ~400/KLoC (way above 1.0).
    fs::write(
        tmp.path().join("a.ruchy"),
        "#[contract_exempt]\nfun a() { 1 }\n#[contract_exempt]\nfun b() { 2 }",
    )
    .unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-exempt-density-above")
        .arg("1.0")
        .assert()
        .failure()
        .stderr(predicate::str::contains("§14.5 F2 breach"));
}

#[test]
fn test_tier_fail_exempt_density_passes_when_no_exemptions() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("a.ruchy"),
        "fun a() { 1 }\nfun b() { 2 }\nfun c() { 3 }",
    )
    .unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-exempt-density-above")
        .arg("0.5")
        .assert()
        .success();
}

#[test]
fn test_tier_fail_exempt_density_skipped_on_empty_loc() {
    // No .ruchy files -> total_loc = 0 -> gate skipped (not applicable).
    let tmp = TempDir::new().unwrap();
    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-exempt-density-above")
        .arg("0.0")
        .assert()
        .success();
}

#[test]
fn test_tier_public_only_filters_to_pub_functions() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("a.ruchy"),
        "pub fun exposed() { 1 }\nfun internal() { 2 }",
    )
    .unwrap();

    let output = ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--public-only")
        .arg("--json")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Only 1 function reported (the pub one).
    assert!(stdout.contains("\"functions\":1"));
    assert!(stdout.contains("\"bronze\":1"));
}

#[test]
fn test_tier_fail_pub_bronze_above_triggers_on_breach() {
    let tmp = TempDir::new().unwrap();
    // 3 pub Bronze (no requires/ensures) → breach at ceiling 1.
    fs::write(
        tmp.path().join("a.ruchy"),
        "pub fun a() { 1 }\npub fun b() { 2 }\npub fun c() { 3 }",
    )
    .unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-pub-bronze-above")
        .arg("1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("§14.5 F4 breach"));
}

#[test]
fn test_tier_fail_pub_bronze_above_passes_when_within_ceiling() {
    let tmp = TempDir::new().unwrap();
    // 1 pub Bronze → within ceiling 2.
    fs::write(
        tmp.path().join("a.ruchy"),
        "pub fun a() { 1 }\nfun internal() { 2 }",
    )
    .unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-pub-bronze-above")
        .arg("2")
        .assert()
        .success();
}

#[test]
fn test_tier_fail_pub_bronze_above_zero_blocks_any_pub_bronze() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "pub fun a() { 1 }").unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-pub-bronze-above")
        .arg("0")
        .assert()
        .failure()
        .stderr(predicate::str::contains("§14.5 F4 breach"));
}

#[test]
fn test_tier_public_only_without_pub_reports_zero() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("a.ruchy"),
        "fun a() { 1 }\nfun b() { 2 }",
    )
    .unwrap();
    let output = ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--public-only")
        .arg("--json")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"functions\":0"));
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
