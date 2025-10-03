//! `DataFrame` aggregation method tests (TDD for DF-006)
//!
//! Tests for `DataFrame` aggregation operations: .`sum()`, .`mean()`, .`max()`, .`min()`
//! Following Toyota Way: Write tests FIRST, then implementation

use ruchy::frontend::Parser;
use ruchy::runtime::{Interpreter, Value};

/// Helper: Evaluate Ruchy code and return result
fn eval(code: &str) -> Result<Value, String> {
    let mut interp = Interpreter::new();
    let mut parser = Parser::new(code);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    interp.eval_expr(&expr).map_err(|e| e.to_string())
}

// ============================================================================
// .sum() Tests (already implemented, verify it works)
// ============================================================================

#[test]
fn test_sum_basic_integers() {
    let code = r#"
        let df = DataFrame::new()
            .column("a", [1, 2, 3])
            .column("b", [4, 5, 6])
            .build();
        df.sum()
    "#;

    let result = eval(code).expect("Should compute sum");
    assert_eq!(result.to_string(), "21"); // 1+2+3+4+5+6 = 21
}

#[test]
fn test_sum_mixed_numeric() {
    let code = r#"
        let df = DataFrame::new()
            .column("a", [1, 2])
            .column("b", [1.5, 2.5])
            .build();
        df.sum()
    "#;

    let result = eval(code).expect("Should compute sum");
    assert_eq!(result.to_string(), "7"); // 1+2+1.5+2.5 = 7.0 (whole number)
}

#[test]
fn test_sum_with_non_numeric() {
    let code = r#"
        let df = DataFrame::new()
            .column("age", [25, 30, 35])
            .column("name", ["Alice", "Bob", "Charlie"])
            .build();
        df.sum()
    "#;

    let result = eval(code).expect("Should skip non-numeric values");
    assert_eq!(result.to_string(), "90"); // 25+30+35 = 90 (skips strings)
}

// ============================================================================
// .mean() Tests (TDD - implement this)
// ============================================================================

#[test]
fn test_mean_basic_integers() {
    let code = r#"
        let df = DataFrame::new()
            .column("values", [10, 20, 30])
            .build();
        df.mean()
    "#;

    let result = eval(code).expect("Should compute mean");
    assert_eq!(result.to_string(), "20"); // (10+20+30)/3 = 20
}

#[test]
fn test_mean_with_decimals() {
    let code = r#"
        let df = DataFrame::new()
            .column("scores", [85.5, 90.0, 92.5])
            .build();
        df.mean()
    "#;

    let result = eval(code).expect("Should compute mean");
    assert_eq!(result.to_string(), "89.33333333333333"); // (85.5+90+92.5)/3
}

#[test]
fn test_mean_multiple_columns() {
    let code = r#"
        let df = DataFrame::new()
            .column("a", [1, 2, 3])
            .column("b", [4, 5, 6])
            .build();
        df.mean()
    "#;

    let result = eval(code).expect("Should compute mean across all columns");
    assert_eq!(result.to_string(), "3.5"); // (1+2+3+4+5+6)/6 = 21/6 = 3.5
}

#[test]
fn test_mean_skips_non_numeric() {
    let code = r#"
        let df = DataFrame::new()
            .column("age", [20, 30, 40])
            .column("name", ["Alice", "Bob", "Charlie"])
            .build();
        df.mean()
    "#;

    let result = eval(code).expect("Should skip non-numeric");
    assert_eq!(result.to_string(), "30"); // (20+30+40)/3 = 30
}

#[test]
fn test_mean_empty_dataframe() {
    let code = r"
        let df = DataFrame::new().build();
        df.mean()
    ";

    let result = eval(code).expect("Should handle empty DataFrame");
    assert_eq!(result.to_string(), "0"); // No values = 0
}

// ============================================================================
// .max() Tests (TDD - implement this)
// ============================================================================

#[test]
fn test_max_basic_integers() {
    let code = r#"
        let df = DataFrame::new()
            .column("values", [5, 10, 3, 8])
            .build();
        df.max()
    "#;

    let result = eval(code).expect("Should find max");
    assert_eq!(result.to_string(), "10");
}

#[test]
fn test_max_with_floats() {
    let code = r#"
        let df = DataFrame::new()
            .column("temps", [98.6, 99.1, 97.8, 100.2])
            .build();
        df.max()
    "#;

    let result = eval(code).expect("Should find max float");
    assert_eq!(result.to_string(), "100.2");
}

#[test]
fn test_max_multiple_columns() {
    let code = r#"
        let df = DataFrame::new()
            .column("a", [1, 5, 2])
            .column("b", [8, 3, 4])
            .build();
        df.max()
    "#;

    let result = eval(code).expect("Should find max across all columns");
    assert_eq!(result.to_string(), "8");
}

#[test]
fn test_max_negative_numbers() {
    let code = r#"
        let df = DataFrame::new()
            .column("values", [-5, -1, -10, -3])
            .build();
        df.max()
    "#;

    let result = eval(code).expect("Should find max negative");
    assert_eq!(result.to_string(), "-1");
}

#[test]
fn test_max_skips_non_numeric() {
    let code = r#"
        let df = DataFrame::new()
            .column("score", [85, 92, 78])
            .column("grade", ["B", "A", "C"])
            .build();
        df.max()
    "#;

    let result = eval(code).expect("Should skip non-numeric");
    assert_eq!(result.to_string(), "92");
}

// ============================================================================
// .min() Tests (TDD - implement this)
// ============================================================================

#[test]
fn test_min_basic_integers() {
    let code = r#"
        let df = DataFrame::new()
            .column("values", [5, 10, 3, 8])
            .build();
        df.min()
    "#;

    let result = eval(code).expect("Should find min");
    assert_eq!(result.to_string(), "3");
}

#[test]
fn test_min_with_floats() {
    let code = r#"
        let df = DataFrame::new()
            .column("temps", [98.6, 99.1, 97.8, 100.2])
            .build();
        df.min()
    "#;

    let result = eval(code).expect("Should find min float");
    assert_eq!(result.to_string(), "97.8");
}

#[test]
fn test_min_multiple_columns() {
    let code = r#"
        let df = DataFrame::new()
            .column("a", [5, 2, 8])
            .column("b", [3, 9, 1])
            .build();
        df.min()
    "#;

    let result = eval(code).expect("Should find min across all columns");
    assert_eq!(result.to_string(), "1");
}

#[test]
fn test_min_negative_numbers() {
    let code = r#"
        let df = DataFrame::new()
            .column("values", [-5, -1, -10, -3])
            .build();
        df.min()
    "#;

    let result = eval(code).expect("Should find min negative");
    assert_eq!(result.to_string(), "-10");
}

#[test]
fn test_min_skips_non_numeric() {
    let code = r#"
        let df = DataFrame::new()
            .column("score", [85, 92, 78])
            .column("grade", ["B", "A", "C"])
            .build();
        df.min()
    "#;

    let result = eval(code).expect("Should skip non-numeric");
    assert_eq!(result.to_string(), "78");
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_aggregations_single_value() {
    let code = r#"
        let df = DataFrame::new()
            .column("value", [42])
            .build();
        [df.sum(), df.mean(), df.max(), df.min()]
    "#;

    let result = eval(code).expect("Should handle single value");
    assert_eq!(result.to_string(), "[42, 42, 42, 42]");
}

#[test]
fn test_aggregations_all_same_values() {
    let code = r#"
        let df = DataFrame::new()
            .column("values", [5, 5, 5, 5])
            .build();
        [df.sum(), df.mean(), df.max(), df.min()]
    "#;

    let result = eval(code).expect("Should handle identical values");
    assert_eq!(result.to_string(), "[20, 5, 5, 5]");
}
