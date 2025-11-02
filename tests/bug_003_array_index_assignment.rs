// BUG-003: Array Index Assignment Not Supported
// EXTREME TDD Protocol: RED → GREEN → REFACTOR → VALIDATE
// GitHub Issue: https://github.com/paiml/ruchy-book/blob/main/docs/bugs/ruchy-runtime-bugs.md#bug-003

use assert_cmd::Command;
use predicates::prelude::*;

/// RED PHASE: These tests WILL FAIL until array index assignment is implemented
/// Current Error: "Invalid assignment target" for matrix[i][j] = value
/// Expected: Support nested array index assignment like Rust

// ============================================================================
// TEST GROUP 1: Simple Array Index Assignment
// ============================================================================

#[test]
fn test_bug_003_simple_array_assignment() {
    // Pattern: arr[0] = value
    let code = r#"
        let mut arr = vec![1, 2, 3];
        arr[0] = 99;
        println(arr[0]);
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("99"));
}

#[test]
fn test_bug_003_array_assignment_transpile() {
    // Verify transpiler generates valid Rust code
    let code = r#"
        let mut arr = vec![1, 2, 3];
        arr[0] = 99;
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("arr[0]").and(predicate::str::contains("= 99")));
}

// ============================================================================
// TEST GROUP 2: Nested Array Index Assignment (Matrix Operations)
// ============================================================================

#[test]
fn test_bug_003_nested_array_assignment() {
    // Pattern: matrix[i][j] = value (CRITICAL for BENCH-002)
    let code = r#"
        let mut matrix = vec![vec![1, 2], vec![3, 4]];
        matrix[0][1] = 99;
        println(matrix[0][1]);
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("99"));
}

#[test]
fn test_bug_003_matrix_update_loop() {
    // Pattern: Update all matrix elements in loop (BENCH-002 pattern)
    let code = r#"
        let mut matrix = vec![vec![0, 0], vec![0, 0]];
        let mut i = 0;
        while i < 2 {
            let mut j = 0;
            while j < 2 {
                matrix[i][j] = i * 2 + j;
                j = j + 1;
            }
            i = i + 1;
        }
        println(matrix[1][1]);
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("3")); // matrix[1][1] should be 1*2 + 1 = 3
}

// ============================================================================
// TEST GROUP 3: Assignment with Expressions
// ============================================================================

#[test]
fn test_bug_003_index_assignment_with_expression() {
    // Pattern: arr[i] = expr (not just literal)
    let code = r#"
        let mut arr = vec![1, 2, 3];
        arr[1] = arr[0] + arr[2];
        println(arr[1]);
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("4")); // 1 + 3 = 4
}

// ============================================================================
// TEST GROUP 4: BENCH-002 Validation (Matrix Multiplication)
// ============================================================================

#[test]
fn test_bug_003_bench_002_matrix_multiplication() {
    // Simplified BENCH-002: 2x2 matrix multiplication
    let code = r#"
        let mut a = vec![vec![1, 2], vec![3, 4]];
        let mut b = vec![vec![2, 0], vec![1, 2]];
        let mut result = vec![vec![0, 0], vec![0, 0]];

        let mut i = 0;
        while i < 2 {
            let mut j = 0;
            while j < 2 {
                let mut k = 0;
                let mut sum = 0;
                while k < 2 {
                    sum = sum + a[i][k] * b[k][j];
                    k = k + 1;
                }
                result[i][j] = sum;
                j = j + 1;
            }
            i = i + 1;
        }

        println(result[0][0]);
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("4")); // result[0][0] = 1*2 + 2*1 = 4
}

// ============================================================================
// PROPERTY TEST: Array assignment preserves other elements
// ============================================================================

#[test]
#[ignore] // Run with: cargo test --test bug_003_array_index_assignment -- --ignored
fn property_array_assignment_preserves_others() {
    use proptest::prelude::*;

    proptest!(|(val1 in 0..100i32, val2 in 0..100i32, new_val in 0..100i32)| {
        let code = format!(r#"
            let mut arr = vec![{}, {}, 999];
            arr[1] = {};
            println(arr[0]);
            println(arr[1]);
            println(arr[2]);
        "#, val1, val2, new_val);

        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        let output = cmd.arg("-e").arg(&code).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);

        // arr[0] should be unchanged
        assert!(stdout.contains(&val1.to_string()));
        // arr[1] should be new value
        assert!(stdout.contains(&new_val.to_string()));
        // arr[2] should be unchanged
        assert!(stdout.contains("999"));
    });
}
