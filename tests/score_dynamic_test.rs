// TDD Test Suite for Dynamic Score Calculation
// Ensures score is not hardcoded but calculated based on code quality

use std::process::Command;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_score_varies_with_quality() {
    // Test that different quality code gets different scores
    
    // High quality code - should score high
    let high_quality = r"
fn calculate_sum(numbers: Vec<i32>) -> i32 {
    numbers.iter().sum()
}

fn calculate_average(numbers: Vec<i32>) -> f64 {
    if numbers.is_empty() {
        return 0.0
    }
    let sum = numbers.iter().sum()
    sum / numbers.len()
}
";
    
    // Low quality code - should score lower
    let low_quality = r"
fn f(x: Vec<i32>) -> i32 {
    let mut s = 0;
    for i in 0..x.len() {
        s = s + x[i];
    }
    s
}

// TODO: Fix this mess
fn g(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32 {
    if a > 0 {
        if b > 0 {
            if c > 0 {
                if d > 0 {
                    if e > 0 {
                        a + b + c + d + e
                    } else { 0 }
                } else { 0 }
            } else { 0 }
        } else { 0 }
    } else { 0 }
}
";
    
    // Get scores for both
    let high_score = get_score(high_quality);
    let low_score = get_score(low_quality);
    
    // Scores should be different
    assert_ne!(high_score, low_score, "Scores should vary based on code quality");
    
    // High quality should score better than low quality
    assert!(high_score > low_score, 
        "High quality code ({high_score}) should score better than low quality ({low_score})");
    
    // Neither should be exactly 0.85 (the hardcoded value)
    assert_ne!(high_score, 0.85, "High quality score should not be hardcoded 0.85");
    assert_ne!(low_score, 0.85, "Low quality score should not be hardcoded 0.85");
}

#[test]
fn test_score_detects_satd_comments() {
    // Code with SATD should score lower
    let with_satd = r"
fn process_data(data: Vec<i32>) -> i32 {
    // TODO: This is a hack, fix later
    // FIXME: This doesn't handle edge cases
    data.iter().sum()
}
";
    
    let without_satd = r"
fn process_data(data: Vec<i32>) -> i32 {
    // Calculate the sum of all elements
    data.iter().sum()
}
";
    
    let score_with_satd = get_score(with_satd);
    let score_without_satd = get_score(without_satd);
    
    assert!(score_without_satd > score_with_satd,
        "Code without SATD ({score_without_satd}) should score higher than with SATD ({score_with_satd})");
}

#[test]
fn test_score_considers_complexity() {
    // Simple function - low complexity
    let simple = r"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
";
    
    // Complex function - high complexity
    let complex = r"
fn complex_logic(x: i32) -> i32 {
    if x > 10 {
        if x > 20 {
            if x > 30 {
                if x > 40 {
                    return x * 2;
                } else {
                    return x + 10;
                }
            } else {
                return x - 5;
            }
        } else {
            return x + 1;
        }
    } else {
        if x < 0 {
            return -x;
        } else {
            return x;
        }
    }
}
";
    
    let simple_score = get_score(simple);
    let complex_score = get_score(complex);
    
    assert!(simple_score > complex_score,
        "Simple code ({simple_score}) should score higher than complex code ({complex_score})");
}

#[test]
fn test_score_json_output() {
    let code = r#"fn main() { println("Hello"); }"#;
    
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(code.as_bytes()).unwrap();
    temp_file.flush().unwrap();
    
    let output = Command::new("./target/release/ruchy")
        .args(["score", temp_file.path().to_str().unwrap(), "--format", "json"])
        .output()
        .expect("Failed to run ruchy score");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"score\":"), "JSON output should contain score field");
    
    // Parse JSON to verify it's valid and has expected structure
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Output should be valid JSON");
    
    assert!(json["score"].is_number(), "Score should be a number");
    let score = json["score"].as_f64().unwrap();
    assert!((0.0..=1.0).contains(&score), "Score should be between 0 and 1");
}

#[test]
fn test_score_threshold_enforcement() {
    let code = r#"fn main() { println("Hello"); }"#;
    
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(code.as_bytes()).unwrap();
    temp_file.flush().unwrap();
    
    // Test with threshold that should pass (very low)
    let output = Command::new("./target/release/ruchy")
        .args(["score", temp_file.path().to_str().unwrap(), "--min", "0.1"])
        .output()
        .expect("Failed to run ruchy score");
    
    assert!(output.status.success(), "Should pass with low threshold");
    
    // Test with threshold that should fail (impossibly high)
    let output = Command::new("./target/release/ruchy")
        .args(["score", temp_file.path().to_str().unwrap(), "--min", "0.99"])
        .output()
        .expect("Failed to run ruchy score");
    
    assert!(!output.status.success(), "Should fail with high threshold");
}

// Helper function to get score from code
fn get_score(code: &str) -> f64 {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(code.as_bytes()).unwrap();
    temp_file.flush().unwrap();
    
    let output = Command::new("./target/release/ruchy")
        .args(["score", temp_file.path().to_str().unwrap()])
        .output()
        .expect("Failed to run ruchy score");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse the score from output
    // Format is: "Score: X.XX/1.0"
    if let Some(line) = stdout.lines().find(|l| l.contains("Score:")) {
        if let Some(score_str) = line.split("Score:").nth(1) {
            if let Some(score_part) = score_str.split('/').next() {
                return score_part.trim().parse::<f64>().unwrap_or(0.0);
            }
        }
    }
    
    0.0
}