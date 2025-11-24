// ISSUE-116: File object methods - open() builtin function
// ROOT CAUSE: open() standalone function not registered as builtin
//
// EXTREME TDD: RED → GREEN → REFACTOR → VALIDATE


#[test]
fn test_issue_116_open_function_with_file_methods() {
    // RED: This test MUST fail - open() not recognized as builtin

    // Create test file
    std::fs::create_dir_all("/tmp/test-data").unwrap();
    std::fs::write("/tmp/test-data/sample.txt", "Line 1\nLine 2\nLine 3").unwrap();

    let script = r#"
let file = open("/tmp/test-data/sample.txt", "r")
let line1 = file.read_line()
println(line1)
let line2 = file.read_line()
println(line2)
file.close()
"#;

    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Script failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Expected output
    assert!(
        stdout.contains("Line 1"),
        "Expected 'Line 1', got: {stdout}"
    );
    assert!(
        stdout.contains("Line 2"),
        "Expected 'Line 2', got: {stdout}"
    );
}

#[test]
fn test_issue_116_open_function_missing_file() {
    // Test error handling for non-existent file

    let script = r#"
let file = open("/tmp/nonexistent_file_test.txt", "r")
"#;

    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "Should fail for non-existent file"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Failed to open") || stderr.contains("No such file"),
        "Expected file error, got: {stderr}"
    );
}
