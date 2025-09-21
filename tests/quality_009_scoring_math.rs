/// TDD: Mathematical scoring tests for quality tool
/// These tests define the EXACT scoring behavior we expect
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_score_mathematical_model() {
    // Define our mathematical scoring model:
    // Base score = 1.0
    // Score = base * complexity_penalty * param_penalty * nesting_penalty * length_penalty

    // Complexity penalties (based on cyclomatic complexity)
    // Note: Functions with good names get doc bonus, poor names get penalty
    assert_score_for_complexity(1, 0.95, 1.00); // No branches
    assert_score_for_complexity(5, 0.95, 1.00); // Low complexity
    assert_score_for_complexity(10, 0.60, 0.85); // Medium complexity (with doc penalty)
    assert_score_for_complexity(20, 0.20, 0.65); // High complexity
    assert_score_for_complexity(40, 0.05, 0.25); // Very high complexity
}

fn assert_score_for_complexity(complexity: usize, min_expected: f64, max_expected: f64) {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    // Generate code with specific cyclomatic complexity
    let mut code = String::from("fn test_complexity() {\n");
    code.push_str("    let mut x = 0;\n");

    // Each if statement adds 1 to cyclomatic complexity
    for i in 0..complexity.saturating_sub(1) {
        code.push_str(&format!("    if x > {i} {{\n"));
        code.push_str(&format!("        x = x + {i};\n"));
        code.push_str("    }\n");
    }

    code.push_str("}\n");

    fs::write(&file_path, code).unwrap();

    let output = Command::new("./target/debug/ruchy")
        .args(["score", file_path.to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let score = json["score"].as_f64().unwrap();

    assert!(
        score >= min_expected && score <= max_expected,
        "Complexity {complexity} should score between {min_expected:.2} and {max_expected:.2}, got {score:.2}"
    );
}

#[test]
fn test_parameter_count_mathematical_model() {
    // Parameter count penalties (multiplicative)
    assert_score_for_params(0, 1.00, 1.00); // No params - perfect
    assert_score_for_params(3, 0.95, 1.00); // Few params - good
    assert_score_for_params(5, 0.85, 0.95); // Moderate params
    assert_score_for_params(7, 0.70, 0.85); // Many params
    assert_score_for_params(10, 0.40, 0.60); // Too many params
    assert_score_for_params(20, 0.05, 0.20); // Way too many params
}

fn assert_score_for_params(param_count: usize, min_expected: f64, max_expected: f64) {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    // Generate function with specific number of parameters
    let mut code = String::from("fn test_params(");
    for i in 0..param_count {
        if i > 0 {
            code.push_str(", ");
        }
        code.push_str(&format!("p{i}: i32"));
    }
    code.push_str(") -> i32 {\n");

    // Simple body - just sum the params
    if param_count > 0 {
        code.push_str("    p0");
        for i in 1..param_count {
            code.push_str(&format!(" + p{i}"));
        }
        code.push('\n');
    } else {
        code.push_str("    0\n");
    }
    code.push_str("}\n");

    fs::write(&file_path, code).unwrap();

    let output = Command::new("./target/debug/ruchy")
        .args(["score", file_path.to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let score = json["score"].as_f64().unwrap();

    assert!(
        score >= min_expected && score <= max_expected,
        "Function with {param_count} params should score between {min_expected:.2} and {max_expected:.2}, got {score:.2}"
    );
}

#[test]
fn test_nesting_depth_mathematical_model() {
    // Nesting depth penalties (multiplicative)
    assert_score_for_nesting(0, 0.95, 1.00); // No nesting - perfect
    assert_score_for_nesting(2, 0.95, 1.00); // Shallow nesting - good
    assert_score_for_nesting(3, 0.85, 0.95); // Moderate nesting
    assert_score_for_nesting(4, 0.70, 0.80); // Deep nesting
    assert_score_for_nesting(5, 0.45, 0.55); // Too deep
    assert_score_for_nesting(7, 0.10, 0.20); // Way too deep
    assert_score_for_nesting(10, 0.01, 0.10); // Catastrophic nesting
}

fn assert_score_for_nesting(depth: usize, min_expected: f64, max_expected: f64) {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    // Generate deeply nested code
    let mut code = String::from("fn test_nesting() {\n");

    // Create nested if statements
    for i in 0..depth {
        code.push_str(&"    ".repeat(i + 1));
        code.push_str("if true {\n");
    }

    // Add a statement at the deepest level
    code.push_str(&"    ".repeat(depth + 1));
    code.push_str("let x = 1;\n");

    // Close all the if statements
    for i in (0..depth).rev() {
        code.push_str(&"    ".repeat(i + 1));
        code.push_str("}\n");
    }

    code.push_str("}\n");

    fs::write(&file_path, code).unwrap();

    let output = Command::new("./target/debug/ruchy")
        .args(["score", file_path.to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let score = json["score"].as_f64().unwrap();

    assert!(
        score >= min_expected && score <= max_expected,
        "Nesting depth {depth} should score between {min_expected:.2} and {max_expected:.2}, got {score:.2}"
    );
}

#[test]
fn test_combined_penalties_multiply() {
    // Test that penalties multiply correctly
    // Bad complexity (10) * bad params (10) * bad nesting (5) should give very low score

    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    let code = r"
fn combined_bad(p0: i32, p1: i32, p2: i32, p3: i32, p4: i32,
                 p5: i32, p6: i32, p7: i32, p8: i32, p9: i32) -> i32 {
    let mut result = 0;
    
    // Add complexity with multiple if statements
    if p0 > 0 { result += 1; }
    if p1 > 0 { result += 2; }
    if p2 > 0 { result += 3; }
    if p3 > 0 { result += 4; }
    if p4 > 0 { result += 5; }
    if p5 > 0 { result += 6; }
    if p6 > 0 { result += 7; }
    if p7 > 0 { result += 8; }
    if p8 > 0 { result += 9; }
    
    // Add deep nesting
    if p0 > 0 {
        if p1 > 0 {
            if p2 > 0 {
                if p3 > 0 {
                    if p4 > 0 {
                        result = result * 2;
                    }
                }
            }
        }
    }
    
    result
}
";

    fs::write(&file_path, code).unwrap();

    let output = Command::new("./target/debug/ruchy")
        .args(["score", file_path.to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let score = json["score"].as_f64().unwrap();

    // With 10 params (penalty ~0.5), complexity ~10 (penalty ~0.85), nesting 5 (penalty ~0.5)
    // Combined score should be approximately: 1.0 * 0.5 * 0.85 * 0.5 = 0.21
    assert!(
        (0.10..=0.35).contains(&score),
        "Combined bad metrics should score between 0.10 and 0.35, got {score:.2}"
    );
}

#[test]
fn test_perfect_code_scores_high() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    let code = r"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
";

    fs::write(&file_path, code).unwrap();

    let output = Command::new("./target/debug/ruchy")
        .args(["score", file_path.to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let score = json["score"].as_f64().unwrap();

    assert!(
        score >= 0.95,
        "Perfect simple code should score >= 0.95, got {score:.2}"
    );
}
