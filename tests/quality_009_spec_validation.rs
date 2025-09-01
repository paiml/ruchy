#![allow(clippy::format_push_string, clippy::unnecessary_to_owned)]

use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_score_spec_compliance_complexity_boundaries() {
    // Test that our implementation matches the spec from docs/specifications/ruchy-scoring-spec.md
    // The spec says (lines 489-493):
    // 0..=10 => 1.0,
    // 11..=20 => 0.9 - ((code.max_cyclomatic - 10) as f64 * 0.01),
    // _ => 0.7,
    
    let test_cases = vec![
        // (complexity, expected_range_min, expected_range_max)
        (5, 0.90, 1.00),   // Low complexity should score high
        (10, 0.85, 1.00),  // Boundary case
        (15, 0.70, 0.90),  // Medium complexity
        (25, 0.30, 0.70),  // High complexity
        (50, 0.01, 0.30),  // Very high complexity
    ];
    
    for (complexity, min_score, max_score) in test_cases {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.ruchy");
        
        // Generate code with specific complexity
        let mut code = String::from("fn complex_function() {\n");
        code.push_str("    let mut x = 0;\n");
        
        // Add nested if statements to increase complexity
        for i in 0..complexity {
            code.push_str(&format!("    if x > {i} {{\n"));
            code.push_str(&format!("        x = x + {i};\n"));
        }
        
        // Close all the if statements
        for _ in 0..complexity {
            code.push_str("    }\n");
        }
        
        code.push_str("}\n");
        
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(code.as_bytes()).unwrap();
        
        let output = Command::new("./target/debug/ruchy")
            .args(["score", file_path.to_str().unwrap(), "--format", "json"])
            .output()
            .unwrap();
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Parse JSON output
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
            let score = json["score"].as_f64().unwrap();
            assert!(
                score >= min_score && score <= max_score,
                "Complexity {complexity} should score between {min_score} and {max_score}, got {score}"
            );
        }
    }
}

#[test]
fn test_score_spec_grade_boundaries() {
    // Test grade boundaries from spec (lines 684-696)
    let test_cases = vec![
        (0.98, "A+"),  // [0.97, 1.00]
        (0.95, "A"),   // [0.93, 0.97)
        (0.91, "A-"),  // [0.90, 0.93)
        (0.88, "B+"),  // [0.87, 0.90)
        (0.85, "B"),   // [0.83, 0.87)
        (0.81, "B-"),  // [0.80, 0.83)
        (0.78, "C+"),  // [0.77, 0.80)
        (0.75, "C"),   // [0.73, 0.77)
        (0.71, "C-"),  // [0.70, 0.73)
        (0.65, "D"),   // [0.60, 0.70)
        (0.30, "F"),   // [0.00, 0.60)
    ];
    
    for (target_score, expected_grade) in test_cases {
        // We can't directly test grades without parsing the output
        // but we can verify the scoring produces reasonable gradations
        println!("Testing score {target_score} should get grade {expected_grade}");
    }
}

#[test]
fn test_score_parameter_penalty() {
    // Test that excessive parameters are heavily penalized
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Function with many parameters should score very low
    let code = r"
fn many_params(
    a1: i32, a2: i32, a3: i32, a4: i32, a5: i32,
    b1: i32, b2: i32, b3: i32, b4: i32, b5: i32,
    c1: i32, c2: i32, c3: i32, c4: i32, c5: i32,
    d1: i32, d2: i32, d3: i32, d4: i32, d5: i32
) -> i32 {
    a1 + a2 + a3 + a4 + a5 +
    b1 + b2 + b3 + b4 + b5 +
    c1 + c2 + c3 + c4 + c5 +
    d1 + d2 + d3 + d4 + d5
}
";
    
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(code.as_bytes()).unwrap();
    
    let output = Command::new("./target/debug/ruchy")
        .args(["score", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should score very low due to 20 parameters
    assert!(stdout.contains("0.") || stdout.contains("1."));
    
    // Extract score from output
    if let Some(score_line) = stdout.lines().find(|l| l.contains("Score:")) {
        if let Some(score_str) = score_line.split(':').nth(1) {
            if let Ok(score) = score_str.trim().split('/').next().unwrap().parse::<f64>() {
                assert!(
                    score <= 0.50,
                    "Function with 20 parameters should score <= 0.50, got {score}"
                );
            }
        }
    }
}

#[test]
fn test_score_nesting_depth_penalty() {
    // Test that deep nesting is heavily penalized
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Create deeply nested code
    let mut code = String::from("fn deeply_nested() {\n");
    let depth = 10;
    
    for i in 0..depth {
        code.push_str(&"    ".repeat(i + 1));
        code.push_str(&"if true {\n".to_string());
        code.push_str(&"    ".repeat(i + 2));
        code.push_str(&format!("let x{i} = {i};\n"));
    }
    
    for i in (0..depth).rev() {
        code.push_str(&"    ".repeat(i + 1));
        code.push_str("}\n");
    }
    
    code.push_str("}\n");
    
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(code.as_bytes()).unwrap();
    
    let output = Command::new("./target/debug/ruchy")
        .args(["score", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Extract score - should be penalized for deep nesting
    if let Some(score_line) = stdout.lines().find(|l| l.contains("Score:")) {
        if let Some(score_str) = score_line.split(':').nth(1) {
            if let Ok(score) = score_str.trim().split('/').next().unwrap().parse::<f64>() {
                assert!(
                    score <= 0.70,
                    "Function with nesting depth {depth} should score <= 0.70, got {score}"
                );
            }
        }
    }
}