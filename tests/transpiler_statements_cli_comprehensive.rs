//! CLI-based comprehensive tests for transpiler/statements.rs (7,191 lines → TDG target)
//!
//! EXTREME TDD: Complement existing 25 API-based tests with CLI-based coverage
//! Target: src/backend/transpiler/statements.rs (7,191 lines, 287.6 lines/test)
//! Strategy: Test ALL statement types via `ruchy transpile` CLI
//! Goal: Improve from 25 tests (287.6 lines/test) → 100+ tests (<100 lines/test)
//!
//! Coverage: let variants, if-let, while-let, loop, try-catch, pipelines, lambdas,
//! list comprehensions, pattern matching, edge cases

use assert_cmd::Command;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// Let Statement Variants (transpile_let, transpile_let_pattern, transpile_let_with_type)
// ============================================================================

#[test]
fn test_let_destructure_tuple() {
    let code = r"
        let (a, b) = (10, 20);
        a + b
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("let") && output.contains("(a, b)"));
}

#[test]
fn test_let_destructure_list() {
    let code = r"
        let [x, y, z] = [1, 2, 3];
        x + y + z
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("let"));
}

#[test]
fn test_let_with_explicit_type() {
    let code = r"
        let x: i32 = 42;
        x
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("i32"));
}

#[test]
#[ignore = "Parser feature gap: let-else syntax not yet implemented (let Some(x) = opt else { return })"]
fn test_let_else_pattern() {
    let code = r"
        let Some(x) = Some(42) else {
            return 0
        };
        x
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("else"));
}

#[test]
fn test_let_multiple_bindings() {
    let code = r"
        let x = 10;
        let y = 20;
        let z = 30;
        x + y + z
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.matches("let").count() >= 3);
}

// ============================================================================
// If-Let Statements (transpile_if_let)
// ============================================================================

#[test]
fn test_if_let_some() {
    let code = r"
        let opt = Some(42);
        if let Some(x) = opt {
            x
        } else {
            0
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("if let"));
}

#[test]
fn test_if_let_ok() {
    let code = r"
        let res = Ok(100);
        if let Ok(val) = res {
            val
        } else {
            0
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("if let"));
}

#[test]
fn test_if_let_tuple() {
    let code = r"
        let pair = (10, 20);
        if let (a, b) = pair {
            a + b
        } else {
            0
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("if let"));
}

// ============================================================================
// While-Let Statements (transpile_while_let)
// ============================================================================

#[test]
fn test_while_let_some() {
    let code = r"
        let mut opt = Some(5);
        while let Some(x) = opt {
            if x == 0 {
                opt = None
            } else {
                opt = Some(x - 1)
            }
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("while let"));
}

// ============================================================================
// Loop Statements (transpile_loop)
// ============================================================================

#[test]
fn test_loop_infinite() {
    let code = r"
        loop {
            break
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("loop"));
}

#[test]
fn test_loop_with_break_value() {
    let code = r"
        let result = loop {
            break 42
        };
        result
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("loop") && output.contains("break"));
}

#[test]
fn test_loop_with_continue() {
    let code = r"
        let mut count = 0;
        loop {
            count = count + 1;
            if count < 5 {
                continue
            };
            break
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("continue"));
}

// ============================================================================
// For Loop Variants (transpile_for)
// ============================================================================

#[test]
fn test_for_range_exclusive() {
    let code = r"
        for i in 0..10 {
            println(i)
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("for"));
}

#[test]
fn test_for_range_inclusive() {
    let code = r"
        for i in 0..=10 {
            println(i)
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("for"));
}

#[test]
fn test_for_array_iteration() {
    let code = r"
        for item in [1, 2, 3, 4, 5] {
            println(item)
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("for"));
}

#[test]
fn test_for_with_break() {
    let code = r"
        for i in 0..100 {
            if i == 10 {
                break
            }
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("break"));
}

#[test]
fn test_for_with_continue() {
    let code = r"
        for i in 0..10 {
            if i % 2 == 0 {
                continue
            };
            println(i)
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("continue"));
}

// ============================================================================
// While Loop Variants (transpile_while)
// ============================================================================

#[test]
fn test_while_condition() {
    let code = r"
        let mut x = 0;
        while x < 10 {
            x = x + 1
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("while"));
}

#[test]
fn test_while_with_break() {
    let code = r"
        let mut x = 0;
        while true {
            x = x + 1;
            if x > 5 {
                break
            }
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("while") && output.contains("break"));
}

// ============================================================================
// Try-Catch Statements (transpile_try_catch)
// ============================================================================

#[test]
#[ignore = "Parser feature gap: try-catch syntax not yet implemented"]
fn test_try_catch_basic() {
    let code = r#"
        try {
            let x = parse_int("42");
            x
        } catch e {
            0
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("catch"));
}

// ============================================================================
// Lambda Functions (transpile_lambda)
// ============================================================================

#[test]
fn test_lambda_simple() {
    let code = r"
        let add = |a, b| a + b;
        add(10, 20)
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains('|'));
}

#[test]
fn test_lambda_with_type_annotations() {
    let code = r"
        let multiply: fn(i32, i32) -> i32 = |x, y| x * y;
        multiply(5, 6)
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains('|'));
}

#[test]
fn test_lambda_passed_to_function() {
    let code = r"
        fun apply(f: fn(i32) -> i32, x: i32) -> i32 {
            f(x)
        }
        apply(|n| n * 2, 21)
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains('|'));
}

// ============================================================================
// Pipeline Operators (transpile_pipeline)
// ============================================================================

#[test]
#[ignore = "Parser feature gap: Pipeline operator (|>) not yet implemented"]
fn test_pipeline_simple() {
    let code = r"
        42 |> double |> add_ten
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("double"));
}

// ============================================================================
// List Comprehensions (transpile_list_comprehension_new)
// ============================================================================

#[test]
#[ignore = "Parser feature gap: List comprehension syntax not yet implemented ([x * 2 for x in items])"]
fn test_list_comprehension_simple() {
    let code = r"
        [x * 2 for x in [1, 2, 3, 4, 5]]
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("map"));
}

#[test]
#[ignore = "Parser feature gap: List comprehension with filter not yet implemented"]
fn test_list_comprehension_with_filter() {
    let code = r"
        [x for x in [1, 2, 3, 4, 5] if x > 2]
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("filter"));
}

// ============================================================================
// Method Calls (transpile_method_call)
// ============================================================================

#[test]
fn test_method_call_simple() {
    let code = r#"
        let s = "hello";
        s.len()
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("len"));
}

#[test]
fn test_method_call_chained() {
    let code = r#"
        let s = "hello world";
        s.to_uppercase().len()
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("to_uppercase"));
}

#[test]
fn test_method_call_with_args() {
    let code = r"
        let vec = vec![1, 2, 3];
        vec.push(4)
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("push"));
}

// ============================================================================
// Block Expressions (transpile_block)
// ============================================================================

#[test]
fn test_block_simple() {
    let code = r"
        {
            let x = 10;
            let y = 20;
            x + y
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains('{'));
}

#[test]
fn test_block_nested() {
    let code = r"
        {
            let x = {
                let y = 10;
                y * 2
            };
            x + 5
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.matches('{').count() >= 2);
}

#[test]
fn test_block_with_early_return() {
    let code = r"
        fun test() -> i32 {
            {
                if true {
                    return 42
                };
                100
            }
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("return"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn edge_case_nested_loops() {
    let code = r"
        for i in 0..5 {
            for j in 0..5 {
                println(i * j)
            }
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.matches("for").count() >= 2);
}

#[test]
fn edge_case_complex_if_elif_else() {
    // Use function parameter to prevent compile-time constant folding
    let code = r#"
        fun classify(x: i32) -> &str {
            if x < 10 {
                "small"
            } else if x < 50 {
                "medium"
            } else if x < 100 {
                "large"
            } else {
                "huge"
            }
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    // Transpiler generates nested if statements, just verify structure exists
    assert!(output.contains("if") && output.contains("else"));
}

#[test]
fn edge_case_loop_labels() {
    let code = r"
        'outer: for i in 0..5 {
            for j in 0..5 {
                if i * j > 10 {
                    break 'outer
                }
            }
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    // Transpiler generates "break outer;" - verify label name appears
    assert!(output.contains("outer") && output.contains("break"));
}

#[test]
fn edge_case_match_with_guards() {
    let code = r#"
        let x = 42;
        match x {
            n if n < 10 => "small",
            n if n < 50 => "medium",
            _ => "large"
        }
    "#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("match"));
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn property_for_loop_depth_1_to_5() {
    // Property: Nested for loops transpile correctly at any depth 1-5
    for depth in 1..=5 {
        let mut code = "println(1)".to_string();
        for i in (0..depth).rev() {
            code = format!("for x{i} in 0..2 {{ {code} }}");
        }

        ruchy_cmd()
            .arg("transpile")
            .arg("-")
            .write_stdin(code.as_str())
            .assert()
            .success();
    }
}

#[test]
fn property_let_bindings_1_to_10() {
    // Property: Multiple sequential let bindings transpile correctly
    for count in 1..=10 {
        let bindings = (0..count)
            .map(|i| format!("let x{i} = {i};"))
            .collect::<Vec<_>>()
            .join("\n");
        let sum = (0..count)
            .map(|i| format!("x{i}"))
            .collect::<Vec<_>>()
            .join(" + ");
        let code = format!("{bindings}\n{sum}");

        ruchy_cmd()
            .arg("transpile")
            .arg("-")
            .write_stdin(code.as_str())
            .assert()
            .success();
    }
}

#[test]
fn property_if_elif_chains_1_to_10() {
    // Property: If-elif chains of any length transpile correctly
    for count in 1..=10 {
        let mut code = "let x = 5;\n".to_string();
        code.push_str("if x < 1 {\n    \"a\"\n");
        for i in 1..count {
            code.push_str(&format!("}} else if x < {} {{\n    \"b\"\n", i + 1));
        }
        code.push_str("} else {\n    \"c\"\n}");

        ruchy_cmd()
            .arg("transpile")
            .arg("-")
            .write_stdin(code.as_str())
            .assert()
            .success();
    }
}

// ============================================================================
// Integration: Full Transpile → Compile
// ============================================================================

#[test]
fn integration_statements_full_pipeline() {
    let code = r#"
        fun fibonacci(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        }

        fun main() {
            for i in 0..10 {
                let result = fibonacci(i);
                println!("fib({}) = {}", i, result);
            }
        }
    "#;

    // Transpile
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let rust_code = String::from_utf8_lossy(&result.get_output().stdout);

    // Verify contains expected elements
    assert!(rust_code.contains("fn fibonacci"));
    assert!(rust_code.contains("fn main"));
    assert!(rust_code.contains("for"));

    // Write to temp file and compile
    std::fs::write("/tmp/statements_integration_test.rs", rust_code.as_ref()).unwrap();
    let compile = std::process::Command::new("rustc")
        .args([
            "--crate-type",
            "bin",
            "/tmp/statements_integration_test.rs",
            "-o",
            "/tmp/statements_integration_test",
        ])
        .output()
        .unwrap();

    assert!(
        compile.status.success(),
        "Compilation failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
}
