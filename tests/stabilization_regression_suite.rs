//! Stabilization Regression Test Suite
//!
//! Comprehensive tests to ensure beta readiness and prevent regressions.
//! Covers parser, transpiler, and runtime edge cases.

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// PARSER: Edge Cases
// ============================================================================

mod parser_edge_cases {
    use super::*;

    #[test]
    fn test_parser_001_nested_blocks() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let result = {
        let x = {
            let y = 10;
            y * 2
        };
        x + 5
    };
    println!("{}", result)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("25"), "Expected 25: {stdout}");
    }

    #[test]
    fn test_parser_002_chained_method_calls() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let s = "hello world";
    let result = s.to_uppercase().len();
    println!("{}", result)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("11"), "Expected 11: {stdout}");
    }

    #[test]
    fn test_parser_003_complex_if_else_chain() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun classify(n: i64) -> i64 {
    if n < 0 {
        -1
    } else if n == 0 {
        0
    } else if n < 10 {
        1
    } else if n < 100 {
        2
    } else {
        3
    }
}

fun main() {
    println!("{}", classify(-5));
    println!("{}", classify(0));
    println!("{}", classify(5));
    println!("{}", classify(50));
    println!("{}", classify(500))
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("-1"), "Should have -1: {stdout}");
        assert!(stdout.contains('0'), "Should have 0: {stdout}");
    }

    #[test]
    fn test_parser_004_match_with_guards() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let x = 15;
    let result = match x {
        n if n < 10 => 1,
        n if n < 20 => 2,
        _ => 3,
    };
    println!("{}", result)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains('2'), "Expected 2: {stdout}");
    }

    #[test]
    fn test_parser_005_array_operations() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let arr = [1, 2, 3, 4, 5];
    let first = arr[0];
    let last = arr[4];
    println!("{}", first + last)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains('6'), "Expected 6: {stdout}");
    }

    #[test]
    fn test_parser_006_closure_captures() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let x = 10;
    let y = 20;
    let add_xy = |z| x + y + z;
    println!("{}", add_xy(5))
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("35"), "Expected 35: {stdout}");
    }
}

// ============================================================================
// TRANSPILER: Code Generation Quality
// ============================================================================

mod transpiler_quality {
    use super::*;

    #[test]
    fn test_transpiler_001_function_with_multiple_params() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun calc(a: i64, b: i64, c: i64, d: i64) -> i64 {
    (a + b) * (c - d)
}

fun main() {
    println!("{}", calc(1, 2, 10, 5))
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        // (1+2) * (10-5) = 3 * 5 = 15
        assert!(stdout.contains("15"), "Expected 15: {stdout}");
    }

    #[test]
    fn test_transpiler_002_recursive_function() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun factorial(n: i64) -> i64 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

fun main() {
    println!("{}", factorial(5))
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("120"), "Expected 120: {stdout}");
    }

    #[test]
    fn test_transpiler_003_string_operations() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let greeting = "Hello";
    let name = "World";
    println!("{} {}!", greeting, name)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Hello World!"),
            "Expected greeting: {stdout}"
        );
    }

    #[test]
    fn test_transpiler_004_boolean_operations() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let a = true;
    let b = false;
    let and_result = a && b;
    let or_result = a || b;
    println!("{} {}", and_result, or_result)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("false true"),
            "Expected 'false true': {stdout}"
        );
    }

    #[test]
    fn test_transpiler_005_while_loop() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let mut sum = 0;
    let mut i = 1;
    while i <= 10 {
        sum = sum + i;
        i = i + 1;
    }
    println!("{}", sum)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        // 1+2+...+10 = 55
        assert!(stdout.contains("55"), "Expected 55: {stdout}");
    }

    #[test]
    fn test_transpiler_006_for_loop_with_range() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let mut product = 1;
    for i in [1, 2, 3, 4] {
        product = product * i;
    }
    println!("{}", product)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        // 1*2*3*4 = 24
        assert!(stdout.contains("24"), "Expected 24: {stdout}");
    }
}

// ============================================================================
// RUNTIME: Edge Cases and Error Handling
// ============================================================================

mod runtime_edge_cases {
    use super::*;

    #[test]
    fn test_runtime_001_negative_numbers() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let a = -10;
    let b = 5;
    let result = a + b;
    println!("{}", result)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("-5"), "Expected -5: {stdout}");
    }

    #[test]
    fn test_runtime_002_float_operations() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let a = 3.14;
    let b = 2.0;
    let result = a * b;
    println!("{}", result)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("6.28"), "Expected ~6.28: {stdout}");
    }

    #[test]
    fn test_runtime_003_empty_function() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun do_nothing() {
}

fun main() {
    do_nothing();
    println!("done")
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("done"), "Expected 'done': {stdout}");
    }

    #[test]
    fn test_runtime_004_early_return() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun find_first_positive(arr: [i64]) -> i64 {
    for x in arr {
        if x > 0 {
            return x;
        }
    }
    return -1;
}

fun main() {
    let result = find_first_positive([-3, -1, 0, 5, 10]);
    println!("{}", result)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains('5'), "Expected 5: {stdout}");
    }

    #[test]
    fn test_runtime_005_nested_function_calls() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun double(x: i64) -> i64 {
    x * 2
}

fun triple(x: i64) -> i64 {
    x * 3
}

fun add(a: i64, b: i64) -> i64 {
    a + b
}

fun main() {
    let result = add(double(5), triple(3));
    println!("{}", result)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        // double(5)=10, triple(3)=9, add(10,9)=19
        assert!(stdout.contains("19"), "Expected 19: {stdout}");
    }
}

// ============================================================================
// LITERALS: All supported literal types
// ============================================================================

mod literal_types {
    use super::*;

    #[test]
    fn test_literal_001_hex_lowercase() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let x = 0xff;
    println!("{}", x)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("255"), "Expected 255: {stdout}");
    }

    #[test]
    fn test_literal_002_hex_uppercase() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let x = 0xABCD;
    println!("{}", x)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("43981"), "Expected 43981: {stdout}");
    }

    #[test]
    fn test_literal_003_char() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let c = 'A';
    println!("{}", c)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains('A'), "Expected A: {stdout}");
    }

    #[test]
    fn test_literal_004_escaped_string() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let s = "line1\nline2";
    println!("{}", s)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("line1"), "Expected line1: {stdout}");
        assert!(stdout.contains("line2"), "Expected line2: {stdout}");
    }

    #[test]
    fn test_literal_005_vec_repeat_transpile() {
        // Test vec![value; count] transpiles correctly to Rust
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let arr = vec![0; 5];
    println!("{:?}", arr)
}
"#,
        )
        .unwrap();

        let output_path = dir.path().join("output.rs");
        ruchy_cmd()
            .arg("transpile")
            .arg(&file_path)
            .arg("-o")
            .arg(&output_path)
            .assert()
            .success();

        let output = fs::read_to_string(&output_path).unwrap();
        // Should have semicolon, not comma
        assert!(
            output.contains("vec![0; 5]") || output.contains("vec![0i64; 5]"),
            "Should have vec repeat with semicolon: {output}"
        );
    }
}

// ============================================================================
// INTEGRATION: Complex scenarios
// ============================================================================

mod integration_scenarios {
    use super::*;

    #[test]
    fn test_integration_001_fibonacci() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun fib(n: i64) -> i64 {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

fun main() {
    println!("{}", fib(10))
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("55"), "Expected 55: {stdout}");
    }

    #[test]
    fn test_integration_002_higher_order_function() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun main() {
    let apply_twice = |f, x| f(f(x));
    let double = |n| n * 2;
    let result = apply_twice(double, 3);
    println!("{}", result)
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        // double(double(3)) = double(6) = 12
        assert!(stdout.contains("12"), "Expected 12: {stdout}");
    }

    #[test]
    fn test_integration_003_multiple_functions() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(
            &file_path,
            r#"
fun is_even(n: i64) -> bool {
    n % 2 == 0
}

fun is_positive(n: i64) -> bool {
    n > 0
}

fun is_valid(n: i64) -> bool {
    is_even(n) && is_positive(n)
}

fun main() {
    println!("{}", is_valid(4));
    println!("{}", is_valid(3));
    println!("{}", is_valid(-2))
}
"#,
        )
        .unwrap();

        let output = ruchy_cmd()
            .arg("run")
            .arg(&file_path)
            .output()
            .expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("true"), "Should have true: {stdout}");
        assert!(stdout.contains("false"), "Should have false: {stdout}");
    }
}
