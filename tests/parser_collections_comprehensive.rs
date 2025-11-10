//! Comprehensive tests for parser/collections.rs (1,979 lines, 45 tests → TDG target)
//!
//! EXTREME TDD: TDG-driven testing for under-tested module
//! Target: src/frontend/parser/collections.rs (44 lines/test ratio)
//! Coverage: Blocks, objects, arrays, tuples, comprehensions, `DataFrames`

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// Block Expressions
// ============================================================================

#[test]
fn test_block_empty() {
    ruchy_cmd().arg("-e").arg("let x = {}; println(x)")
        .assert().success();
}

#[test]
fn test_block_single_expression() {
    ruchy_cmd().arg("-e").arg("let x = { 42 }; println(x)")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_block_multiple_expressions() {
    ruchy_cmd().arg("-e").arg(r"
        let x = {
            let a = 10;
            let b = 20;
            a + b
        };
        println(x)
    ").assert().success().stdout(predicate::str::contains("30"));
}

#[test]
fn test_block_nested() {
    ruchy_cmd().arg("-e").arg(r"
        let x = {
            let y = { 10 };
            y + 5
        };
        println(x)
    ").assert().success().stdout(predicate::str::contains("15"));
}

#[test]
fn test_block_with_comments() {
    ruchy_cmd().arg("-e").arg(r"
        let x = {
            // Comment before expression
            42
            // Comment after expression
        };
        println(x)
    ").assert().success().stdout(predicate::str::contains("42"));
}

// ============================================================================
// Object Literals
// ============================================================================

#[test]
fn test_object_literal_empty() {
    ruchy_cmd().arg("-e").arg(r#"let obj = {}; println("OK")"#)
        .assert().success().stdout(predicate::str::contains("OK"));
}

#[test]
fn test_object_literal_single_field() {
    ruchy_cmd().arg("-e").arg(r#"
        let obj = { x: 42 };
        println(obj["x"])
    "#).assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_object_literal_multiple_fields() {
    ruchy_cmd().arg("-e").arg(r#"
        let obj = { x: 10, y: 20, z: 30 };
        println(obj["x"], obj["y"], obj["z"])
    "#).assert().success()
        .stdout(predicate::str::contains("10"))
        .stdout(predicate::str::contains("20"))
        .stdout(predicate::str::contains("30"));
}

#[test]
fn test_object_literal_string_keys() {
    ruchy_cmd().arg("-e").arg(r#"
        let obj = { "name": "test", "count": 5 };
        println(obj["name"])
    "#).assert().success().stdout(predicate::str::contains("test"));
}

#[test]
fn test_object_literal_nested() {
    ruchy_cmd().arg("-e").arg(r#"
        let obj = {
            outer: {
                inner: 42
            }
        };
        println(obj["outer"]["inner"])
    "#).assert().success().stdout(predicate::str::contains("42"));
}

// ============================================================================
// Array Literals
// ============================================================================

#[test]
fn test_array_empty() {
    ruchy_cmd().arg("-e").arg("let arr = []; println(len(arr))")
        .assert().success().stdout(predicate::str::contains("0"));
}

#[test]
fn test_array_single_element() {
    ruchy_cmd().arg("-e").arg("let arr = [42]; println(arr[0])")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_array_multiple_elements() {
    ruchy_cmd().arg("-e").arg("let arr = [1, 2, 3, 4, 5]; println(len(arr))")
        .assert().success().stdout(predicate::str::contains("5"));
}

#[test]
fn test_array_heterogeneous() {
    ruchy_cmd().arg("-e").arg(r#"let arr = [1, "two", 3.0, true]; println(len(arr))"#)
        .assert().success().stdout(predicate::str::contains("4"));
}

#[test]
fn test_array_nested() {
    ruchy_cmd().arg("-e").arg("let arr = [[1, 2], [3, 4], [5, 6]]; println(len(arr))")
        .assert().success().stdout(predicate::str::contains("3"));
}

#[test]
fn test_array_with_trailing_comma() {
    ruchy_cmd().arg("-e").arg("let arr = [1, 2, 3,]; println(len(arr))")
        .assert().success().stdout(predicate::str::contains("3"));
}

#[test]
fn test_array_multiline() {
    ruchy_cmd().arg("-e").arg(r"
        let arr = [
            1,
            2,
            3
        ];
        println(len(arr))
    ").assert().success().stdout(predicate::str::contains("3"));
}

// ============================================================================
// Tuple Literals
// ============================================================================

#[test]
fn test_tuple_empty() {
    ruchy_cmd().arg("-e").arg("let t = (); println(\"OK\")")
        .assert().success().stdout(predicate::str::contains("OK"));
}

#[test]
fn test_tuple_single_element() {
    ruchy_cmd().arg("-e").arg("let t = (42,); println(\"OK\")")
        .assert().success();
}

#[test]
fn test_tuple_pair() {
    ruchy_cmd().arg("-e").arg("let t = (1, 2); println(t.0, t.1)")
        .assert().success()
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"));
}

#[test]
fn test_tuple_triple() {
    ruchy_cmd().arg("-e").arg("let t = (1, 2, 3); println(t.0, t.1, t.2)")
        .assert().success()
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_tuple_heterogeneous() {
    ruchy_cmd().arg("-e").arg(r#"let t = (42, "hello", 3.14, true); println("OK")"#)
        .assert().success().stdout(predicate::str::contains("OK"));
}

#[test]
#[ignore = "Nested tuple access (t.0.0) not yet implemented - parser error"]
fn test_tuple_nested() {
    ruchy_cmd().arg("-e").arg("let t = ((1, 2), (3, 4)); println(t.0.0)")
        .assert().success().stdout(predicate::str::contains("1"));
}

// ============================================================================
// Comprehensions
// ============================================================================

#[test]
#[ignore = "List comprehensions not yet fully implemented in runtime"]
fn test_list_comprehension_basic() {
    ruchy_cmd().arg("-e").arg(r"
        let squares = [x * x for x in range(5)];
        println(len(squares))
    ").assert().success().stdout(predicate::str::contains("5"));
}

#[test]
#[ignore = "List comprehensions with filters not yet implemented"]
fn test_list_comprehension_with_filter() {
    ruchy_cmd().arg("-e").arg(r"
        let evens = [x for x in range(10) if x % 2 == 0];
        println(len(evens))
    ").assert().success().stdout(predicate::str::contains("5"));
}

#[test]
#[ignore = "Set comprehensions not yet implemented"]
fn test_set_comprehension_basic() {
    ruchy_cmd().arg("-e").arg(r#"
        let s = {x * x for x in range(5)};
        println("OK")
    "#).assert().success();
}

#[test]
#[ignore = "Dict comprehensions not yet implemented"]
fn test_dict_comprehension_basic() {
    ruchy_cmd().arg("-e").arg(r#"
        let d = {x: x * x for x in range(5)};
        println("OK")
    "#).assert().success();
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn edge_case_deeply_nested_blocks() {
    ruchy_cmd().arg("-e").arg(r"
        let x = {
            {
                {
                    42
                }
            }
        };
        println(x)
    ").assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn edge_case_array_with_expressions() {
    ruchy_cmd().arg("-e").arg("let arr = [1 + 1, 2 * 2, 3 - 1]; println(arr[0])")
        .assert().success().stdout(predicate::str::contains("2"));
}

#[test]
fn edge_case_object_with_computed_values() {
    ruchy_cmd().arg("-e").arg(r#"
        let obj = {
            a: 10 + 5,
            b: 20 * 2
        };
        println(obj["a"])
    "#).assert().success().stdout(predicate::str::contains("15"));
}

#[test]
fn edge_case_array_of_arrays_of_arrays() {
    ruchy_cmd().arg("-e").arg("let arr = [[[1, 2]], [[3, 4]]]; println(arr[0][0][0])")
        .assert().success().stdout(predicate::str::contains("1"));
}

#[test]
fn edge_case_tuple_with_trailing_comma() {
    ruchy_cmd().arg("-e").arg("let t = (1, 2, 3,); println(\"OK\")")
        .assert().success();
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn property_array_lengths_0_to_20() {
    // Property: Arrays of any length parse correctly
    for n in 0..=20 {
        let elements = (0..n).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
        ruchy_cmd().arg("-e")
            .arg(format!("let arr = [{elements}]; assert_eq(len(arr), {n})"))
            .assert().success();
    }
}

#[test]
fn property_nested_block_depth_1_to_5() {
    // Property: Nested blocks work to arbitrary depth
    for depth in 1..=5 {
        let mut code = "42".to_string();
        for _ in 0..depth {
            code = format!("{{ {code} }}");
        }
        code = format!("let x = {code}; println(x)");

        ruchy_cmd().arg("-e").arg(&code)
            .assert().success().stdout(predicate::str::contains("42"));
    }
}

#[test]
fn property_tuple_sizes_0_to_10() {
    // Property: Tuples of any size parse correctly
    for n in 0..=10 {
        let elements = (0..n).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
        let code = if n == 0 {
            "let t = (); println(\"OK\")".to_string()
        } else if n == 1 {
            format!("let t = ({elements},); println(\"OK\")")
        } else {
            format!("let t = ({elements}); println(\"OK\")")
        };

        ruchy_cmd().arg("-e").arg(&code)
            .assert().success();
    }
}
