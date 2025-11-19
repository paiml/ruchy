#![allow(missing_docs)]
// CLI-UNIFY-002: Test that 'ruchy run' interprets (not compiles)
// EXTREME TDD: RED phase - these tests will FAIL initially

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::time::Instant;
use tempfile::NamedTempFile;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
fn test_ruchy_run_interprets_under_2_seconds() {
    // CRITICAL: `ruchy run` should interpret (<2s), not compile (10-60s)
    // This is the SMOKING GUN test that proves the defect

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"
fun main() {
    println("Hello from interpreter!")
}
"#,
    )
    .unwrap();

    let start = Instant::now();

    ruchy_cmd()
        .arg("run")
        .arg(temp_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from interpreter!"));

    let duration = start.elapsed();

    // ASSERT: Should complete in <2 seconds (interpret)
    // NOT 10-60 seconds (compile)
    assert!(
        duration.as_secs() < 2,
        "ruchy run took {}s - it's compiling instead of interpreting!",
        duration.as_secs()
    );
}

#[test]
fn test_ruchy_run_no_binary_artifact() {
    // CRITICAL: `ruchy run` should NOT create binary artifacts
    // Currently it creates binaries in /tmp (compilation evidence)

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"
fun main() {
    println("No binary should be created")
}
"#,
    )
    .unwrap();

    // Get list of files in /tmp before
    let before_files = fs::read_dir("/tmp")
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("ruchy_compile_")
        })
        .count();

    ruchy_cmd()
        .arg("run")
        .arg(temp_file.path())
        .assert()
        .success();

    // Get list of files in /tmp after
    let after_files = fs::read_dir("/tmp")
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("ruchy_compile_")
        })
        .count();

    // ASSERT: No new binary artifacts created
    assert_eq!(
        before_files, after_files,
        "ruchy run created binary artifacts - it's compiling instead of interpreting!"
    );
}

#[test]
fn test_ruchy_run_output_correct() {
    // Test that `ruchy run` produces correct output

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r#"
fun main() {
    println("Line 1")
    println("Line 2")
    let x = 42
    println(x)
}
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(temp_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Line 1"))
        .stdout(predicate::str::contains("Line 2"))
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_ruchy_run_handles_errors() {
    // Test that `ruchy run` handles syntax errors gracefully

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r"
fun main() {
    let x = 1 +
}
",
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(temp_file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));
}

#[test]
fn test_ruchy_run_same_output_as_direct() {
    // CRITICAL: `ruchy run script.ruchy` should produce same output as `ruchy script.ruchy`
    // This tests interpreter/transpiler parity

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(
        &temp_file,
        r"
fun main() {
    let x = 10
    let y = 20
    println(x + y)
}
",
    )
    .unwrap();

    // Get output from direct execution
    let direct_output = ruchy_cmd()
        .arg(temp_file.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Get output from `ruchy run`
    let run_output = ruchy_cmd()
        .arg("run")
        .arg(temp_file.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // ASSERT: Same output
    assert_eq!(
        direct_output, run_output,
        "ruchy run produces different output than direct execution!"
    );
}
