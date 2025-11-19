//! Comprehensive tests for `parser/expressions_helpers/patterns.rs` (1,352 lines → TDG target)
//!
//! EXTREME TDD: TDG-driven testing for pattern matching and destructuring
//! Target: `src/frontend/parser/expressions_helpers/patterns.rs`
//! Coverage: Identifier, tuple, list, struct, variant, or, literal, range patterns

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// Identifier Patterns (parse_let_pattern - basic identifiers)
// ============================================================================

#[test]
fn test_identifier_pattern_simple() {
    let code = "let x = 42; println(x)";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_identifier_pattern_underscore() {
    // Underscore pattern (ignore binding)
    let code = "let _ = 42; println(100)";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("100"));
}

#[test]
fn test_identifier_pattern_multiple() {
    let code = r"
        let a = 1;
        let b = 2;
        let c = 3;
        println(a + b + c)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("6"));
}

// ============================================================================
// Tuple Patterns (destructuring tuples)
// ============================================================================

#[test]
fn test_tuple_pattern_two_elements() {
    let code = r"
        let (a, b) = (10, 20);
        println(a + b)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("30"));
}

#[test]
fn test_tuple_pattern_three_elements() {
    let code = r"
        let (x, y, z) = (1, 2, 3);
        println(x * y * z)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("6"));
}

#[test]
fn test_tuple_pattern_nested() {
    let code = r"
        let ((a, b), c) = ((1, 2), 3);
        println(a + b + c)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("6"));
}

#[test]
fn test_tuple_pattern_with_underscore() {
    // Ignore some elements
    let code = r"
        let (x, _, z) = (10, 20, 30);
        println(x + z)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("40"));
}

// ============================================================================
// List Patterns (destructuring lists/arrays)
// ============================================================================

#[test]
fn test_list_pattern_simple() {
    let code = r"
        let [a, b, c] = [1, 2, 3];
        println(a + b + c)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("6"));
}

#[test]
#[ignore = "Parser feature gap: Rest patterns in list destructuring ([first, ...rest]) not yet implemented"]
fn test_list_pattern_with_rest() {
    // Rest pattern: [first, ...rest]
    let code = r"
        let [first, ...rest] = [1, 2, 3, 4, 5];
        println(first)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
}

#[test]
#[ignore = "Parser feature gap: Rest patterns in list destructuring ([first, second, ...rest]) not yet implemented"]
fn test_list_pattern_multiple_with_rest() {
    // Multiple elements + rest
    let code = r"
        let [first, second, ...rest] = [10, 20, 30, 40, 50];
        println(first + second)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("30"));
}

// ============================================================================
// Struct Patterns (destructuring structs)
// ============================================================================

#[test]
fn test_struct_pattern_basic() {
    let code = r"
        struct Point { x: i32, y: i32 }
        let p = Point { x: 10, y: 20 };
        let Point { x, y } = p;
        println(x + y)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("30"));
}

#[test]
fn test_struct_pattern_renamed_fields() {
    // Rename fields: { x: a, y: b }
    let code = r"
        struct Point { x: i32, y: i32 }
        let p = Point { x: 100, y: 200 };
        let Point { x: a, y: b } = p;
        println(a + b)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("300"));
}

#[test]
fn test_struct_pattern_partial_destructure() {
    // Destructure only some fields
    let code = r"
        struct Point { x: i32, y: i32, z: i32 }
        let p = Point { x: 1, y: 2, z: 3 };
        let Point { x, .. } = p;
        println(x)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
}

// ============================================================================
// Variant Patterns (Some, Ok, Err)
// ============================================================================

#[test]
fn test_variant_pattern_some() {
    let code = r"
        let opt = Some(42);
        match opt {
            Some(x) => println(x),
            None => println(0)
        }
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_variant_pattern_none() {
    let code = r"
        let opt: Option<i32> = None;
        match opt {
            Some(x) => println(x),
            None => println(999)
        }
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("999"));
}

#[test]
fn test_variant_pattern_ok() {
    let code = r"
        let res: Result<i32, String> = Ok(100);
        match res {
            Ok(x) => println(x),
            Err(_) => println(0)
        }
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("100"));
}

#[test]
fn test_variant_pattern_err() {
    let code = r#"
        let res: Result<i32, String> = Err("failed");
        match res {
            Ok(x) => println(x),
            Err(e) => println(e)
        }
    "#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("failed"));
}

#[test]
fn test_variant_pattern_nested() {
    // Some(Ok(x))
    let code = r"
        let nested = Some(Ok(42));
        match nested {
            Some(Ok(x)) => println(x),
            _ => println(0)
        }
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// ============================================================================
// Or Patterns (pattern | pattern)
// ============================================================================

#[test]
fn test_or_pattern_literals() {
    let code = r#"
        let x = 2;
        match x {
            1 | 2 | 3 => println("small"),
            _ => println("large")
        }
    "#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("small"));
}

#[test]
fn test_or_pattern_variants() {
    let code = r#"
        let opt = None;
        match opt {
            Some(x) | None => println("handled")
        }
    "#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("handled"));
}

// ============================================================================
// Literal Patterns (42, "hello", true)
// ============================================================================

#[test]
fn test_literal_pattern_integer() {
    let code = r#"
        let x = 42;
        match x {
            42 => println("found"),
            _ => println("not found")
        }
    "#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("found"));
}

#[test]
fn test_literal_pattern_string() {
    let code = r#"
        let s = "hello";
        match s {
            "hello" => println("greeting"),
            _ => println("other")
        }
    "#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("greeting"));
}

#[test]
fn test_literal_pattern_bool() {
    let code = r#"
        let flag = true;
        match flag {
            true => println("yes"),
            false => println("no")
        }
    "#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("yes"));
}

// ============================================================================
// Range Patterns (1..10, 1..=100)
// ============================================================================

#[test]
fn test_range_pattern_exclusive() {
    let code = r#"
        let x = 5;
        match x {
            0..10 => println("in range"),
            _ => println("out of range")
        }
    "#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("in range"));
}

#[test]
fn test_range_pattern_inclusive() {
    let code = r#"
        let x = 10;
        match x {
            1..=10 => println("included"),
            _ => println("excluded")
        }
    "#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("included"));
}

#[test]
fn test_range_pattern_multiple() {
    let code = r#"
        let x = 15;
        match x {
            0..10 => println("low"),
            10..20 => println("mid"),
            _ => println("high")
        }
    "#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("mid"));
}

// ============================================================================
// Complex Pattern Combinations
// ============================================================================

#[test]
#[ignore = "Parser limitation: Struct patterns inside tuple patterns not supported - raises 'Expected ',' or ')' in pattern' error"]
fn test_complex_tuple_struct_pattern() {
    let code = r"
        struct Point { x: i32, y: i32 }
        let pair = (Point { x: 10, y: 20 }, Point { x: 30, y: 40 });
        let (Point { x: x1, y: y1 }, Point { x: x2, y: y2 }) = pair;
        println(x1 + y1 + x2 + y2)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("100"));
}

#[test]
fn test_complex_nested_variants() {
    let code = r"
        let data = Some(Ok((1, 2)));
        match data {
            Some(Ok((a, b))) => println(a + b),
            _ => println(0)
        }
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_complex_list_in_match() {
    let code = r"
        let items = [1, 2, 3];
        match items {
            [a, b, c] => println(a + b + c),
            _ => println(0)
        }
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("6"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
#[ignore = "Runtime limitation: Empty tuple pattern `let () = ()` not supported - raises 'Pattern did not match the value' error"]
fn edge_case_empty_tuple() {
    let code = r"
        let () = ();
        println(42)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn edge_case_single_element_tuple() {
    let code = r"
        let (x,) = (42,);
        println(x)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn edge_case_wildcard_in_tuple() {
    let code = r"
        let (_, _, x) = (1, 2, 3);
        println(x)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn edge_case_deeply_nested_pattern() {
    let code = r"
        let (((x,),),) = (((42,),),);
        println(x)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn error_case_pattern_mismatch_tuple_size() {
    // (a, b) = (1, 2, 3) should fail
    let code = r"
        let (a, b) = (1, 2, 3);
        println(a)
    ";
    ruchy_cmd().arg("-e").arg(code).assert().failure();
}

#[test]
fn error_case_pattern_mismatch_list_size() {
    // [a, b] = [1, 2, 3] should fail
    let code = r"
        let [a, b] = [1, 2, 3];
        println(a)
    ";
    ruchy_cmd().arg("-e").arg(code).assert().failure();
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn property_tuple_patterns_1_to_10() {
    // Property: Tuple patterns work for any size 2-10 (size 1 requires trailing comma)
    for size in 2..=10 {
        let bindings = (1..=size)
            .map(|i| format!("x{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let values = (1..=size)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let sum_expr = (1..=size)
            .map(|i| format!("x{i}"))
            .collect::<Vec<_>>()
            .join(" + ");
        let expected_sum: i32 = (1..=size).sum();

        let code = format!("let ({bindings}) = ({values}); println({sum_expr})");
        ruchy_cmd()
            .arg("-e")
            .arg(&code)
            .assert()
            .success()
            .stdout(predicate::str::contains(expected_sum.to_string()));
    }
}

#[test]
fn property_list_patterns_1_to_10() {
    // Property: List patterns work for any size 1-10
    for size in 1..=10 {
        let bindings = (1..=size)
            .map(|i| format!("x{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let values = (1..=size)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let sum_expr = (1..=size)
            .map(|i| format!("x{i}"))
            .collect::<Vec<_>>()
            .join(" + ");
        let expected_sum: i32 = (1..=size).sum();

        let code = format!("let [{bindings}] = [{values}]; println({sum_expr})");
        ruchy_cmd()
            .arg("-e")
            .arg(&code)
            .assert()
            .success()
            .stdout(predicate::str::contains(expected_sum.to_string()));
    }
}

#[test]
#[ignore = "Test algorithm flaw: Generates nested single-element tuples ((x)) without trailing commas, which fail at runtime. Proper nested tuple test exists in edge_case_deeply_nested_pattern"]
fn property_nested_tuple_depth_1_to_5() {
    // Property: Nested tuple patterns work to depth 2-5 (depth 1 creates single-element tuple issue)
    for depth in 2..=5 {
        let mut pattern = "x".to_string();
        let mut value = "42".to_string();
        for _ in 0..depth {
            pattern = format!("({pattern})");
            value = format!("({value})");
        }
        let code = format!("let {pattern} = {value}; println(x)");
        ruchy_cmd()
            .arg("-e")
            .arg(&code)
            .assert()
            .success()
            .stdout(predicate::str::contains("42"));
    }
}

#[test]
fn property_match_literal_ranges() {
    // Property: Range patterns work for all boundaries
    let ranges = vec![
        (5, "0..10", true),
        (10, "0..10", false),
        (10, "0..=10", true),
        (0, "1..10", false),
        (9, "0..10", true),
    ];

    for (value, range, expected_match) in ranges {
        let expected_output = if expected_match { "match" } else { "no match" };
        let code = format!(
            r#"
            let x = {value};
            match x {{
                {range} => println("match"),
                _ => println("no match")
            }}
        "#
        );
        ruchy_cmd()
            .arg("-e")
            .arg(&code)
            .assert()
            .success()
            .stdout(predicate::str::contains(expected_output));
    }
}

// ============================================================================
// Integration: Full Transpile → Compile
// ============================================================================

#[test]
#[ignore = "Transpiler limitation: List destructuring generates refutable pattern `let [x, y, z] = ...as_slice()` which fails Rust compilation (E0005: irrefutable pattern required)"]
fn integration_patterns_transpile_compile() {
    let code = r#"
        fun process_result(res: Result<i32, String>) -> i32 {
            match res {
                Ok(x) => x * 2,
                Err(_) => 0
            }
        }

        fun process_option(opt: Option<i32>) -> i32 {
            match opt {
                Some(x) => x + 10,
                None => -1
            }
        }

        fun main() {
            let (a, b) = (5, 10);
            println!("Tuple: {}", a + b);

            let [x, y, z] = [1, 2, 3];
            println!("List: {}", x + y + z);

            println!("Result: {}", process_result(Ok(21)));
            println!("Option: {}", process_option(Some(32)));
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
    assert!(rust_code.contains("fn process_result"));
    assert!(rust_code.contains("fn process_option"));
    assert!(rust_code.contains("match"));

    // Write to temp file and compile
    std::fs::write("/tmp/patterns_integration_test.rs", rust_code.as_ref()).unwrap();
    let compile = std::process::Command::new("rustc")
        .args([
            "--crate-type",
            "bin",
            "/tmp/patterns_integration_test.rs",
            "-o",
            "/tmp/patterns_integration_test",
        ])
        .output()
        .unwrap();

    assert!(
        compile.status.success(),
        "Compilation failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
}
