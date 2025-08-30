/// QUALITY-009: Score Tool Poor Actionability - TDD Test Suite
/// 
/// This test file demonstrates and fixes the poor discrimination of quality scores.
/// The current scoring algorithm is too lenient, giving high scores to terrible code.
/// Following TDD methodology: Write failing tests first, then fix the algorithm.

use std::process::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_score_excellent_code_should_be_high() {
    // TDD: Excellent code should score 0.90+ 
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("excellent.ruchy");
    
    // This is genuinely excellent code - simple, clear, well-structured
    let excellent_code = r#"
/// Calculate the factorial of a number
fn factorial(n: i32) -> i32 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

fn main() {
    let result = factorial(5);
    println("5! = {}", result);
}
"#;
    
    fs::write(&file_path, excellent_code).unwrap();
    
    let output = Command::new("./target/debug/ruchy")
        .args(&["score", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    
    assert!(output.status.success(), "Should successfully score excellent code");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Extract score from output
    let score = extract_score_from_output(&stdout);
    
    // Excellent code should score 0.90 or higher
    assert!(score >= 0.90, 
        "Excellent code should score ≥0.90, got {:.2}. Output: {}", 
        score, stdout);
}

#[test]
fn test_score_terrible_code_should_be_low() {
    // TDD: This test should demonstrate the current problem - terrible code gets good scores
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("terrible.ruchy");
    
    // This is genuinely terrible code - 26 parameters, 8-level nesting, no structure
    let terrible_code = r#"
fn terrible_function(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32, h: i32, i: i32, j: i32, k: i32, l: i32, m: i32, n: i32, o: i32, p: i32, q: i32, r: i32, s: i32, t: i32, u: i32, v: i32, w: i32, x: i32, y: i32, z: i32) -> i32 {
    if a > 0 {
        if b > 0 {
            if c > 0 {
                if d > 0 {
                    if e > 0 {
                        if f > 0 {
                            if g > 0 {
                                if h > 0 {
                                    let mut result = 0;
                                    for i in 0..100 {
                                        for j in 0..50 {
                                            for k in 0..25 {
                                                result += a + b + c + d + e + f + g + h + i + j + k + l + m + n + o + p + q + r + s + t + u + v + w + x + y + z;
                                            }
                                        }
                                    }
                                    return result;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    0
}

fn main() {
    let result = terrible_function(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26);
    println("Result: {}", result);
}
"#;
    
    fs::write(&file_path, terrible_code).unwrap();
    
    let output = Command::new("./target/debug/ruchy")
        .args(&["score", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    
    assert!(output.status.success(), "Should successfully score terrible code");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let score = extract_score_from_output(&stdout);
    
    // This test will initially fail because terrible code currently gets ~0.84
    // After fixing, terrible code should score 0.30 or lower
    assert!(score <= 0.30, 
        "Terrible code (26 params, 8-level nesting) should score ≤0.30, got {:.2}. Output: {}", 
        score, stdout);
}

#[test]
fn test_score_good_vs_terrible_discrimination() {
    // TDD: Test that we can discriminate between good and terrible code
    let temp_dir = tempdir().unwrap();
    
    // Good code
    let good_file = temp_dir.path().join("good.ruchy");
    let good_code = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

fn main() {
    let sum = add(2, 3);
    let product = multiply(sum, 4);
    println("Result: {}", product);
}
"#;
    fs::write(&good_file, good_code).unwrap();
    
    // Terrible code
    let terrible_file = temp_dir.path().join("terrible.ruchy");
    let terrible_code = r#"
fn bad(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32, h: i32, i: i32, j: i32) -> i32 {
    if a > 0 {
        if b > 0 {
            if c > 0 {
                if d > 0 {
                    if e > 0 {
                        if f > 0 {
                            return a + b + c + d + e + f + g + h + i + j;
                        }
                    }
                }
            }
        }
    }
    0
}
"#;
    fs::write(&terrible_file, terrible_code).unwrap();
    
    // Score both
    let good_output = Command::new("./target/debug/ruchy")
        .args(&["score", good_file.to_str().unwrap()])
        .output()
        .unwrap();
    
    let terrible_output = Command::new("./target/debug/ruchy")
        .args(&["score", terrible_file.to_str().unwrap()])
        .output()
        .unwrap();
    
    let good_score = extract_score_from_output(&String::from_utf8_lossy(&good_output.stdout));
    let terrible_score = extract_score_from_output(&String::from_utf8_lossy(&terrible_output.stdout));
    
    // There should be at least 0.40 difference between good and terrible code
    // Currently there's only ~0.11 difference, which is not actionable
    let difference = good_score - terrible_score;
    assert!(difference >= 0.40, 
        "Good code ({:.2}) should score at least 0.40 points higher than terrible code ({:.2}). Current difference: {:.2}", 
        good_score, terrible_score, difference);
}

#[test]
fn test_score_complexity_penalty() {
    // TDD: Test that high complexity gets penalized appropriately
    let temp_dir = tempdir().unwrap();
    
    let simple_file = temp_dir.path().join("simple.ruchy");
    fs::write(&simple_file, "fn simple() -> i32 { 42 }").unwrap();
    
    let complex_file = temp_dir.path().join("complex.ruchy");
    let complex_code = r#"
fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                if x > 1000 {
                    return x * 4;
                } else {
                    return x * 3;
                }
            } else {
                return x * 2;
            }
        } else {
            return x + 1;
        }
    } else {
        return 0;
    }
}
"#;
    fs::write(&complex_file, complex_code).unwrap();
    
    let simple_output = Command::new("./target/debug/ruchy")
        .args(&["score", simple_file.to_str().unwrap()])
        .output()
        .unwrap();
    
    let complex_output = Command::new("./target/debug/ruchy")
        .args(&["score", complex_file.to_str().unwrap()])
        .output()
        .unwrap();
    
    let simple_score = extract_score_from_output(&String::from_utf8_lossy(&simple_output.stdout));
    let complex_score = extract_score_from_output(&String::from_utf8_lossy(&complex_output.stdout));
    
    // Complex nested code should score significantly lower
    assert!(simple_score > complex_score + 0.25,
        "Simple code ({:.2}) should score at least 0.25 higher than complex nested code ({:.2})",
        simple_score, complex_score);
}

/// Helper function to extract score from ruchy score output
fn extract_score_from_output(output: &str) -> f64 {
    // Look for pattern like "Score: 0.84/1.0"
    for line in output.lines() {
        if line.contains("Score:") {
            if let Some(score_part) = line.split("Score:").nth(1) {
                if let Some(score_str) = score_part.trim().split('/').next() {
                    if let Ok(score) = score_str.trim().parse::<f64>() {
                        return score;
                    }
                }
            }
        }
    }
    panic!("Could not extract score from output: {}", output);
}