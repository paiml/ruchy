#![allow(missing_docs)]
// Tests for DEBUGGER-014: --trace flag support (Issue #84 Phase 1.1)
// GitHub Issue: https://github.com/paiml/ruchy/issues/84
//
// Test naming convention: test_debugger_014_<scenario>

/// Test #1: Verify --trace flag is recognized (doesn't error)
#[test]
fn test_debugger_014_trace_flag_recognized() {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("--trace")
        .arg("--help")
        .assert()
        .success();
}

/// Test #2: Verify --trace flag works with eval
#[test]
fn test_debugger_014_trace_with_eval() {
    let code = r#"
fun main() {
    println("Hello");
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}

/// Test #3: Verify --trace flag works with run command
#[test]
fn test_debugger_014_trace_with_run() {
    // Create temporary test file
    let temp_file = std::env::temp_dir().join("test_trace.ruchy");
    std::fs::write(&temp_file, "fun main() { println(\"test\"); }").unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("--trace")
        .arg("run")
        .arg(&temp_file)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();

    // Cleanup
    std::fs::remove_file(temp_file).ok();
}
