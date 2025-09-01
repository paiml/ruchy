/// Comprehensive CLI tests for the score command
/// These tests verify all CLI flags and options work correctly

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_score_basic_file() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    fs::write(&file_path, "fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["score", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Score:"))
        .stdout(predicate::str::contains("Quality Score"));
}

#[test]
fn test_score_json_format() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    fs::write(&file_path, "fn test() { let x = 1; }").unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["score", file_path.to_str().unwrap(), "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"score\""))
        .stdout(predicate::str::contains("\"file\""))
        .stdout(predicate::str::contains("\"depth\""));
}

#[test]
fn test_score_min_threshold_pass() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Simple function should score high
    fs::write(&file_path, "fn simple() -> i32 { 42 }").unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["score", file_path.to_str().unwrap(), "--min", "0.5"])
        .assert()
        .success();
}

#[test]
fn test_score_min_threshold_fail() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Very complex function should score low
    let complex_code = r"
fn terrible(a1: i32, a2: i32, a3: i32, a4: i32, a5: i32,
            b1: i32, b2: i32, b3: i32, b4: i32, b5: i32,
            c1: i32, c2: i32, c3: i32, c4: i32, c5: i32,
            d1: i32, d2: i32, d3: i32, d4: i32, d5: i32) -> i32 {
    let mut x = 0;
    if a1 > 0 {
        if a2 > 0 {
            if a3 > 0 {
                if a4 > 0 {
                    if a5 > 0 {
                        if b1 > 0 {
                            if b2 > 0 {
                                x = 1;
                            }
                        }
                    }
                }
            }
        }
    }
    x
}
";
    
    fs::write(&file_path, complex_code).unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["score", file_path.to_str().unwrap(), "--min", "0.9"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("below threshold"));
}

#[test]
fn test_score_directory() {
    let dir = TempDir::new().unwrap();
    
    // Create multiple .ruchy files
    fs::write(dir.path().join("file1.ruchy"), "fn test1() { 1 }").unwrap();
    fs::write(dir.path().join("file2.ruchy"), "fn test2() { 2 }").unwrap();
    
    let subdir = dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("file3.ruchy"), "fn test3() { 3 }").unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["score", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Project Quality Score"))
        .stdout(predicate::str::contains("Files analyzed: 3"));
}

#[test]
fn test_score_directory_json() {
    let dir = TempDir::new().unwrap();
    
    fs::write(dir.path().join("test.ruchy"), "fn main() {}").unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["score", dir.path().to_str().unwrap(), "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"files_analyzed\""))
        .stdout(predicate::str::contains("\"average_score\""));
}

#[test]
fn test_score_output_to_file() {
    let dir = TempDir::new().unwrap();
    let input_file = dir.path().join("test.ruchy");
    let output_file = dir.path().join("score_report.txt");
    
    fs::write(&input_file, "fn test() { 42 }").unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args([
            "score",
            input_file.to_str().unwrap(),
            "--output",
            output_file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Score report written to"));
    
    // Verify output file was created and contains score
    assert!(output_file.exists());
    let content = fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("Score:"));
}

#[test]
fn test_score_depth_options() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    fs::write(&file_path, "fn test() { 1 + 1 }").unwrap();
    
    // Test different depth options
    for depth in &["shallow", "standard", "deep"] {
        Command::cargo_bin("ruchy")
            .unwrap()
            .args(["score", file_path.to_str().unwrap(), "--depth", depth])
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("Analysis Depth: {depth}")));
    }
}

#[test]
fn test_score_empty_directory() {
    let dir = TempDir::new().unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["score", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No .ruchy files found"));
}

#[test]
fn test_score_nonexistent_file() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["score", "/nonexistent/file.ruchy"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to read file"));
}

#[test]
fn test_score_invalid_ruchy_syntax() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("invalid.ruchy");
    
    fs::write(&file_path, "fn test( { invalid syntax }").unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["score", file_path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("parse"));
}

#[test]
fn test_score_help() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["score", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--min"))
        .stdout(predicate::str::contains("--output"));
}

#[test]
fn test_score_verbose_output() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    fs::write(&file_path, "fn test() { 1 }").unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["score", file_path.to_str().unwrap(), "--verbose"])
        .assert()
        .success();
}

#[test]
fn test_score_perfect_code() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("perfect.ruchy");
    
    // Write perfect code
    fs::write(&file_path, r"
/// Adds two numbers together
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Multiplies two numbers
fn multiply(x: i32, y: i32) -> i32 {
    x * y
}
").unwrap();
    
    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .args(["score", file_path.to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let score = json["score"].as_f64().unwrap();
    
    assert!(score >= 0.95, "Perfect code should score >= 0.95, got {score}");
}

#[test]
fn test_score_terrible_code() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("terrible.ruchy");
    
    // Write terrible code
    fs::write(&file_path, r"
fn x(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32, h: i32,
     i: i32, j: i32, k: i32, l: i32, m: i32, n: i32, o: i32, p: i32,
     q: i32, r: i32, s: i32, t: i32, u: i32, v: i32, w: i32, y: i32,
     z: i32, aa: i32) -> i32 {
    // TODO: Fix this mess
    let mut x = 0;
    if a > 0 {
        if b > 0 {
            if c > 0 {
                if d > 0 {
                    if e > 0 {
                        if f > 0 {
                            if g > 0 {
                                if h > 0 {
                                    if i > 0 {
                                        x = 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    x
}
").unwrap();
    
    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .args(["score", file_path.to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let score = json["score"].as_f64().unwrap();
    
    assert!(score <= 0.20, "Terrible code should score <= 0.20, got {score}");
}