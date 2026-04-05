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
fn test_tier_by_file_human_output() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "fun x() { 1 }\nfun y() { 2 }").unwrap();
    fs::write(
        tmp.path().join("b.ruchy"),
        "fun z() requires i > 0 { 1 }",
    )
    .unwrap();

    let output = ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--by-file")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("per-file tier breakdown"));
    assert!(stdout.contains("a.ruchy"));
    assert!(stdout.contains("b.ruchy"));
}

#[test]
fn test_tier_config_file_applies_gate() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "pub fun a() { 1 }").unwrap();
    let cfg = tmp.path().join("tier.toml");
    fs::write(&cfg, "[gates]\nfail_pub_bronze_above = 0\n").unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--config")
        .arg(&cfg)
        .assert()
        .failure()
        .stderr(predicate::str::contains("§14.5 F4 breach"));
}

#[test]
fn test_tier_cli_flag_overrides_config() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "pub fun a() { 1 }").unwrap();
    let cfg = tmp.path().join("tier.toml");
    // Config says fail if pub_bronze > 5 (would pass)
    fs::write(&cfg, "[gates]\nfail_pub_bronze_above = 5\n").unwrap();

    // CLI flag tightens to 0 → should fail (CLI wins).
    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--config")
        .arg(&cfg)
        .arg("--fail-pub-bronze-above")
        .arg("0")
        .assert()
        .failure()
        .stderr(predicate::str::contains("§14.5 F4 breach"));
}

#[test]
fn test_tier_config_file_missing_errors_cleanly() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "fun a() { 1 }").unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--config")
        .arg("/nonexistent-config.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("reading config"));
}

#[test]
fn test_tier_fail_on_scorecard_warn_fires_on_pub_bronze() {
    let tmp = TempDir::new().unwrap();
    // 1 pub Bronze → F4 WARN → breach at level=warn.
    fs::write(tmp.path().join("a.ruchy"), "pub fun a() { 1 }").unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-on-scorecard")
        .arg("warn")
        .assert()
        .failure()
        .stderr(predicate::str::contains("§14.5 scorecard breach"))
        .stderr(predicate::str::contains("F4:WARN"));
}

#[test]
fn test_tier_fail_on_scorecard_fail_ignores_warn() {
    let tmp = TempDir::new().unwrap();
    // 1 pub Bronze → F4 WARN (not FAIL) → level=fail passes.
    fs::write(tmp.path().join("a.ruchy"), "pub fun a() { 1 }").unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-on-scorecard")
        .arg("fail")
        .assert()
        .success();
}

#[test]
fn test_tier_fail_on_scorecard_invalid_level_errors() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "fun a() { 1 }").unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-on-scorecard")
        .arg("bogus")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid --fail-on-scorecard"));
}

#[test]
fn test_tier_markdown_output_has_headers() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "pub fun a() { 1 }").unwrap();

    let output = ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--markdown")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("## §14.5 Provability Tier Report"));
    assert!(stdout.contains("### Tier Distribution"));
    assert!(stdout.contains("### §14.5 Falsifier Scorecard"));
}

#[test]
fn test_tier_markdown_suppresses_human_summary() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "fun a() { 1 }").unwrap();
    let output = ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--markdown")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Markdown format replaces the human "Provability tier scan:" prefix
    assert!(!stdout.contains("Provability tier scan:"));
}

#[test]
fn test_tier_baseline_creates_file_on_first_run() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "fun x() { 1 }").unwrap();
    let baseline = tmp.path().join("tier-baseline.json");

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--baseline")
        .arg(&baseline)
        .assert()
        .success()
        .stderr(predicate::str::contains("baseline captured"));

    assert!(baseline.exists(), "baseline file should have been written");
    let content = fs::read_to_string(&baseline).unwrap();
    assert!(content.contains("\"bronze\""));
    assert!(content.contains("\"pub_bronze\""));
}

#[test]
fn test_tier_baseline_passes_when_no_regression() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "fun x() { 1 }").unwrap();
    let baseline = tmp.path().join("tier-baseline.json");

    // First run captures baseline.
    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--baseline")
        .arg(&baseline)
        .assert()
        .success();

    // Second run compares — identical scan, no regressions.
    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--baseline")
        .arg(&baseline)
        .assert()
        .success()
        .stderr(predicate::str::contains("baseline OK"));
}

#[test]
fn test_tier_baseline_fails_on_regression() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "fun x() { 1 }").unwrap();
    let baseline = tmp.path().join("tier-baseline.json");

    // Capture baseline with 1 Bronze.
    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--baseline")
        .arg(&baseline)
        .assert()
        .success();

    // Add 2 more Bronze functions → regression.
    fs::write(
        tmp.path().join("a.ruchy"),
        "fun x() { 1 }\nfun y() { 2 }\nfun z() { 3 }",
    )
    .unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--baseline")
        .arg(&baseline)
        .assert()
        .failure()
        .stderr(predicate::str::contains("baseline regression"))
        .stderr(predicate::str::contains("bronze : 1 → 3"));
}

#[test]
fn test_tier_summary_contains_scorecard_line() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "pub fun a() { 1 }").unwrap();
    let output = ruchy_cmd().arg("tier").arg(tmp.path()).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("§14.5 scorecard:"));
    assert!(stdout.contains("F1:"));
    assert!(stdout.contains("F4:WARN")); // 1 pub Bronze → F4 WARN
}

#[test]
fn test_tier_json_contains_scorecard_object() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "fun a() { 1 }").unwrap();
    let output = ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--json")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"scorecard\""));
    assert!(stdout.contains("\"f1\":"));
    assert!(stdout.contains("\"f11\":"));
}

#[test]
fn test_tier_by_file_sort_by_bronze_puts_worst_first() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "fun x() { 1 }").unwrap();
    fs::write(
        tmp.path().join("b.ruchy"),
        "fun p() { 1 }\nfun q() { 2 }\nfun r() { 3 }",
    )
    .unwrap();

    let output = ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--json")
        .arg("--by-file")
        .arg("--sort-by")
        .arg("bronze")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let by_file_line = stdout.lines().nth(1).unwrap();
    // b.ruchy (3 Bronze) should come before a.ruchy (1 Bronze)
    let b_pos = by_file_line.find("b.ruchy").unwrap();
    let a_pos = by_file_line.find("a.ruchy").unwrap();
    assert!(b_pos < a_pos, "worst file should come first: {by_file_line}");
}

#[test]
fn test_tier_by_file_top_truncates() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "fun x() { 1 }").unwrap();
    fs::write(tmp.path().join("b.ruchy"), "fun y() { 1 }").unwrap();
    fs::write(tmp.path().join("c.ruchy"), "fun z() { 1 }").unwrap();

    let output = ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--json")
        .arg("--by-file")
        .arg("--top")
        .arg("2")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let by_file_line = stdout.lines().nth(1).unwrap();
    // Exactly 2 entries → one comma between objects
    assert_eq!(by_file_line.matches("},{").count(), 1);
}

#[test]
fn test_tier_by_file_json_emits_second_array() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "fun x() { 1 }").unwrap();

    let output = ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--json")
        .arg("--by-file")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    let aggr = lines.next().unwrap();
    let by_file = lines.next().unwrap();
    assert!(aggr.starts_with('{'));
    assert!(by_file.starts_with('['));
    assert!(by_file.contains("\"bronze\":1"));
    assert!(by_file.contains("\"total\":1"));
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
fn test_tier_fail_diff_exempt_density_above_triggers_on_high_density() {
    let tmp = TempDir::new().unwrap();
    // 2 #[diff_exempt] in ~5 LoC → density ~400/KLoC (way above 1.0).
    fs::write(
        tmp.path().join("a.ruchy"),
        "#[diff_exempt]\nfun a() { 1 }\n#[diff_exempt]\nfun b() { 2 }",
    )
    .unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-diff-exempt-density-above")
        .arg("1.0")
        .assert()
        .failure()
        .stderr(predicate::str::contains("§14.5 F11 breach"));
}

#[test]
fn test_tier_fail_diff_exempt_density_passes_when_no_exemptions() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("a.ruchy"), "fun a() { 1 }\nfun b() { 2 }").unwrap();

    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-diff-exempt-density-above")
        .arg("0.5")
        .assert()
        .success();
}

#[test]
fn test_tier_fail_diff_exempt_density_skipped_on_empty_loc() {
    let tmp = TempDir::new().unwrap();
    ruchy_cmd()
        .arg("tier")
        .arg(tmp.path())
        .arg("--fail-diff-exempt-density-above")
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
