// LANG-COMP-003: Control Flow - RED PHASE TESTS
// Tests written FIRST before examples exist
// EXTREME TDD Protocol: These tests MUST fail until examples are created

use std::process::Command;

/// Helper function to run a Ruchy example file and capture output
fn run_ruchy_file(file_path: &str) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "run", file_path])
        .output()
        .expect("Failed to execute ruchy command")
}

/// Helper function to evaluate Ruchy code directly using REPL
/// Returns output with REPL banners stripped - only the evaluation result
fn eval_ruchy_code(code: &str) -> std::process::Output {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "repl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn ruchy repl");

    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "{}", code).expect("Failed to write to stdin");
        writeln!(stdin, ":quit").expect("Failed to write quit command");
    }

    let mut output = child.wait_with_output().expect("Failed to read output");

    // Strip REPL banners - keep only the last non-empty line (the result)
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let result = stdout_str
        .lines()
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with("Welcome")
                && !line.starts_with("Type")
                && !line.starts_with("🚀")
                && !line.starts_with("✨")
        })
        .last()
        .unwrap_or("")
        .to_string();

    output.stdout = result.into_bytes();
    output
}

// ============================================================================
// IF EXPRESSION TESTS
// ============================================================================

#[test]
fn test_if_true_branch() {
    let output = eval_ruchy_code("if true { 1 } else { 2 }");
    assert!(output.status.success(), "If true should execute");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "1");
}

#[test]
fn test_if_false_branch() {
    let output = eval_ruchy_code("if false { 1 } else { 2 }");
    assert!(output.status.success(), "If false should execute else");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "2");
}

#[test]
fn test_if_without_else() {
    let output = eval_ruchy_code("if true { 42 }");
    assert!(output.status.success(), "If without else should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "42");
}

#[test]
fn test_if_expression_example() {
    let output = run_ruchy_file("examples/lang_comp/03-control-flow/01_if.ruchy");
    assert!(
        output.status.success(),
        "If expression example should execute successfully"
    );
}

// ============================================================================
// MATCH EXPRESSION TESTS
// ============================================================================

#[test]
fn test_match_literal() {
    let output = eval_ruchy_code("match 1 { 1 => 100, 2 => 200, _ => 999 }");
    assert!(output.status.success(), "Match literal should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "100");
}

#[test]
fn test_match_wildcard() {
    let output = eval_ruchy_code("match 99 { 1 => 100, 2 => 200, _ => 999 }");
    assert!(output.status.success(), "Match wildcard should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "999");
}

#[test]
fn test_match_expression_example() {
    let output = run_ruchy_file("examples/lang_comp/03-control-flow/02_match.ruchy");
    assert!(
        output.status.success(),
        "Match expression example should execute successfully"
    );
}

// ============================================================================
// FOR LOOP TESTS
// ============================================================================

#[test]
fn test_for_loop_range() {
    // Use file execution for multi-statement code (REPL is for single expressions)
    use std::fs;
    let test_file = "/tmp/test_for_loop.ruchy";
    fs::write(
        test_file,
        r#"
let sum = 0
for i in 0..3 {
    sum = sum + i
}
sum
"#,
    )
    .expect("Failed to write test file");

    let output = run_ruchy_file(test_file);
    assert!(output.status.success(), "For loop with range should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "3");
}

#[test]
fn test_for_loop_example() {
    let output = run_ruchy_file("examples/lang_comp/03-control-flow/03_for.ruchy");
    assert!(
        output.status.success(),
        "For loop example should execute successfully"
    );
}

// ============================================================================
// WHILE LOOP TESTS
// ============================================================================

#[test]
fn test_while_loop() {
    // Use file execution for multi-statement code (REPL is for single expressions)
    use std::fs;
    let test_file = "/tmp/test_while_loop.ruchy";
    fs::write(
        test_file,
        r#"
let count = 0
while count < 3 {
    count = count + 1
}
count
"#,
    )
    .expect("Failed to write test file");

    let output = run_ruchy_file(test_file);
    assert!(output.status.success(), "While loop should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "3");
}

#[test]
fn test_while_loop_example() {
    let output = run_ruchy_file("examples/lang_comp/03-control-flow/04_while.ruchy");
    assert!(
        output.status.success(),
        "While loop example should execute successfully"
    );
}

// ============================================================================
// LOOP CONTROL TESTS (break, continue)
// ============================================================================

#[test]
fn test_break_statement() {
    // Use file execution for multi-statement code (REPL is for single expressions)
    use std::fs;
    let test_file = "/tmp/test_break.ruchy";
    fs::write(
        test_file,
        r#"
let i = 0
while true {
    if i == 3 {
        break
    }
    i = i + 1
}
i
"#,
    )
    .expect("Failed to write test file");

    let output = run_ruchy_file(test_file);
    assert!(output.status.success(), "Break statement should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "3");
}

#[test]
fn test_loop_control_example() {
    let output = run_ruchy_file("examples/lang_comp/03-control-flow/05_break_continue.ruchy");
    assert!(
        output.status.success(),
        "Loop control example should execute successfully"
    );
}

// ============================================================================
// PROPERTY TESTS (Currently ignored - will enable after basic tests pass)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;

    #[test]
    #[ignore]
    fn if_else_covers_all_cases() {
        use std::fs;
        // Property: if-else always returns a value
        for i in 0..100 {
            let code = format!("if {} > 50 {{ 1 }} else {{ 0 }}", i);
            let test_file = format!("/tmp/prop_if_{}.ruchy", i);
            fs::write(&test_file, &code).expect("Failed to write test file");
            let output = run_ruchy_file(&test_file);
            assert!(output.status.success());
            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
            assert!(result == "1" || result == "0", "Got: {}", result);
        }
    }

    #[test]
    #[ignore]
    fn match_wildcard_always_matches() {
        use std::fs;
        // Property: match with wildcard never fails
        for i in 0..100 {
            let code = format!("match {} {{ 1 => 100, _ => 999 }}", i);
            let test_file = format!("/tmp/prop_match_{}.ruchy", i);
            fs::write(&test_file, &code).expect("Failed to write test file");
            let output = run_ruchy_file(&test_file);
            assert!(output.status.success(), "Failed for i={}", i);
        }
    }

    #[test]
    #[ignore]
    fn for_loop_iterations_match_range() {
        use std::fs;
        // Property: for loop runs exactly range.len() times
        for n in 1..10 {
            let code = format!(
                r#"
let count = 0
for i in 0..{} {{
    count = count + 1
}}
count
"#,
                n
            );
            let test_file = format!("/tmp/prop_for_{}.ruchy", n);
            fs::write(&test_file, &code).expect("Failed to write test file");
            let output = run_ruchy_file(&test_file);
            assert!(output.status.success(), "Failed for n={}", n);
            assert_eq!(
                String::from_utf8_lossy(&output.stdout).trim(),
                n.to_string(),
                "For n={}, expected {} iterations",
                n,
                n
            );
        }
    }
}
