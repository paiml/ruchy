#![allow(missing_docs)]
//! BUG-034: Linter Reports False Errors for Built-in Functions
//!
//! **Problem**: `ruchy lint` reports "undefined variable" errors for built-in functions
//! **Discovered**: GitHub Issue #34
//! **Severity**: MEDIUM - Makes linter output unusable due to false positives
//!
//! **Expected**: Linter should recognize built-in functions and not report errors
//! **Actual**: Reports "undefined variable" for `fs_read`, `env_args`, range, etc.
//!
//! **Root Cause**: Linter only recognizes println/print/eprintln as built-ins
//!
//! This test follows EXTREME TDD (RED → GREEN → REFACTOR)

use ruchy::frontend::parser::Parser;
use ruchy::quality::linter::Linter;

/// Helper to lint code
fn lint_code(code: &str) -> Vec<ruchy::quality::linter::LintIssue> {
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");

    let mut linter = Linter::new();
    linter.set_rules("undefined");
    linter.lint(&ast, code).expect("Failed to lint")
}

// ==================== RED PHASE: Failing Tests ====================

/// Test 1: fs_ functions should not be reported as undefined
#[test]
fn test_bug_034_red_fs_functions() {
    let code = r#"
fun read_config() {
    fs_read("config.txt")
}
"#;

    let issues = lint_code(code);

    // RED: This will FAIL - fs_read reported as undefined
    let undefined_issues: Vec<_> = issues.iter().filter(|i| i.rule == "undefined").collect();

    assert_eq!(
        undefined_issues.len(),
        0,
        "fs_read should be recognized as built-in, found: {undefined_issues:?}"
    );
}

/// Test 2: env_ functions should not be reported as undefined
#[test]
fn test_bug_034_red_env_functions() {
    let code = r"
fun get_args() {
    env_args()
}
";

    let issues = lint_code(code);

    let undefined_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == "undefined" && i.name.starts_with("env_"))
        .collect();

    assert_eq!(
        undefined_issues.len(),
        0,
        "env_args should be recognized as built-in, found: {undefined_issues:?}"
    );
}

/// Test 3: `range()` function should not be reported as undefined
#[test]
fn test_bug_034_red_range_function() {
    let code = r"
fun test_loop() {
    for i in range(0, 10) {
        println(i)
    }
}
";

    let issues = lint_code(code);

    let undefined_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == "undefined" && i.name == "range")
        .collect();

    assert_eq!(
        undefined_issues.len(),
        0,
        "range should be recognized as built-in, found: {undefined_issues:?}"
    );
}

/// Test 4: http_ functions should not be reported as undefined
#[test]
fn test_bug_034_red_http_functions() {
    let code = r#"
fun fetch_data() {
    http_get("https://api.example.com/data")
}
"#;

    let issues = lint_code(code);

    let undefined_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == "undefined" && i.name.starts_with("http_"))
        .collect();

    assert_eq!(
        undefined_issues.len(),
        0,
        "http_get should be recognized as built-in, found: {undefined_issues:?}"
    );
}

/// Test 5: json_ functions should not be reported as undefined
#[test]
fn test_bug_034_red_json_functions() {
    let code = r#"
fun parse_json() {
    json_parse("{\"key\": \"value\"}")
}
"#;

    let issues = lint_code(code);

    let undefined_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == "undefined" && i.name.starts_with("json_"))
        .collect();

    assert_eq!(
        undefined_issues.len(),
        0,
        "json_parse should be recognized as built-in, found: {undefined_issues:?}"
    );
}

/// Test 6: time_ functions should not be reported as undefined
#[test]
fn test_bug_034_red_time_functions() {
    let code = r"
fun get_timestamp() {
    time_now()
}
";

    let issues = lint_code(code);

    let undefined_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == "undefined" && i.name.starts_with("time_"))
        .collect();

    assert_eq!(
        undefined_issues.len(),
        0,
        "time_now should be recognized as built-in, found: {undefined_issues:?}"
    );
}

/// Test 7: path_ functions should not be reported as undefined
#[test]
fn test_bug_034_red_path_functions() {
    let code = r#"
fun get_extension() {
    path_extension("file.txt")
}
"#;

    let issues = lint_code(code);

    let undefined_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == "undefined" && i.name.starts_with("path_"))
        .collect();

    assert_eq!(
        undefined_issues.len(),
        0,
        "path_extension should be recognized as built-in, found: {undefined_issues:?}"
    );
}

/// Test 8: Baseline - println already works
#[test]
fn test_bug_034_baseline_println() {
    let code = r#"
fun test() {
    println("Hello")
}
"#;

    let issues = lint_code(code);

    let undefined_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == "undefined" && i.name == "println")
        .collect();

    // This should already pass (baseline test)
    assert_eq!(
        undefined_issues.len(),
        0,
        "println should be recognized as built-in"
    );
}

/// Test 9: Baseline - undefined variables should still be reported
#[test]
fn test_bug_034_baseline_real_undefined() {
    let code = r"
fun test() {
    undefined_function_xyz()
}
";

    let issues = lint_code(code);

    let undefined_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.rule == "undefined" && i.name == "undefined_function_xyz")
        .collect();

    // This should fail - we WANT to detect real undefined variables
    assert_eq!(
        undefined_issues.len(),
        1,
        "Real undefined variables should still be reported"
    );
}

/// Test 10: Multiple built-ins in one file
#[test]
fn test_bug_034_red_multiple_builtins() {
    let code = r#"
fun main() {
    let args = env_args()
    let config = fs_read("config.json")
    let data = json_parse(config)
    let url = "https://api.example.com"
    let response = http_get(url)
    println(response)
}
"#;

    let issues = lint_code(code);

    // Should only report undefined issues for non-built-ins
    let undefined_issues: Vec<_> = issues.iter().filter(|i| i.rule == "undefined").collect();

    assert_eq!(
        undefined_issues.len(),
        0,
        "No built-ins should be reported as undefined, found: {undefined_issues:?}"
    );
}

// ==================== RED PHASE SUMMARY ====================

/// Summary test to document the RED phase
#[test]
fn test_bug_034_red_phase_summary() {
    println!("BUG-034 RED Phase: Linter False Positives for Built-ins");
    println!();
    println!("Problem: Linter only recognizes println/print/eprintln as built-ins");
    println!("Impact: Makes linter output unusable due to excessive false positives");
    println!();
    println!("Test Suite Created:");
    println!("1. fs_ functions (fs_read, fs_write, etc.)");
    println!("2. env_ functions (env_args, env_var, etc.)");
    println!("3. range() function");
    println!("4. http_ functions (http_get, http_post, etc.)");
    println!("5. json_ functions (json_parse, json_stringify, etc.)");
    println!("6. time_ functions (time_now, etc.)");
    println!("7. path_ functions (path_extension, etc.)");
    println!("8. Baseline: println already works");
    println!("9. Baseline: real undefined variables still reported");
    println!("10. Multiple built-ins in one file");
    println!();
    println!("Expected Results:");
    println!("- RED Phase: Tests 1-7, 10 FAIL (false positives)");
    println!("- RED Phase: Tests 8-9 PASS (baseline validation)");
    println!("- GREEN Phase: ALL tests PASS after fix");
    println!();
    println!("Next Step: Add is_builtin() function to linter");
}
