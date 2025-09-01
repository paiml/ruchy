/// QUALITY-008: Score Tool Directory Support - TDD Test Suite
/// 
/// This test file implements comprehensive testing for directory support in the score tool.
/// Following TDD methodology: Write failing tests first, then implement functionality.

use std::process::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_score_directory_basic_failure_case() {
    // TDD Red: This test should fail initially, demonstrating the current bug
    let temp_dir = tempdir().unwrap();
    let dir_path = temp_dir.path();
    
    // Create a simple .ruchy file in the directory
    let file_path = dir_path.join("hello.ruchy");
    fs::write(&file_path, "fn main() { println(\"Hello\"); }").unwrap();
    
    // This should work but currently fails with "Is a directory" error
    let output = Command::new("./target/debug/ruchy")
        .args(["score", dir_path.to_str().unwrap()])
        .output()
        .unwrap();
    
    // Initially this will fail, proving the bug exists
    assert!(output.status.success(), 
        "Score tool should support directory analysis. stderr: {}", 
        String::from_utf8_lossy(&output.stderr));
    
    // Output should contain aggregated score information
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Score:"), "Output should contain score information");
}

#[test] 
fn test_score_single_file_still_works() {
    // TDD Red: Ensure single file behavior still works after directory support
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("single.ruchy");
    fs::write(&file_path, "fn test() -> i32 { 1 + 1 }").unwrap();
    
    // Single file scoring should still work
    let output = Command::new("./target/debug/ruchy")
        .args(["score", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    
    assert!(output.status.success(), "Single file scoring should still work");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Score:"), "Should contain score");
}