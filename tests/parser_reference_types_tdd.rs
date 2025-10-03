//! TDD Test for BOOK-CH15-003: Reference type parsing bug
//!
//! Root Cause: Parser fails when function parameters use reference types (&T)
//! after a function with multiple let statements
//!
//! Minimal reproduction:
//! ```ruchy
//! fun main() {
//!     let a = 1;
//!     let b = helper(&a);
//! }
//! fun helper(x: &i32) -> i32 { 42 }
//! ```

use ruchy::Parser;

#[test]
fn test_reference_type_after_multilet_function() {
    // Minimal case: function with 2 lets, followed by function with &i32 param
    let code = r#"
        fun main() {
            let data = vec![1, 2, 3];
            let sum = calc(&data);
            println("Sum: {}", sum);
        }

        fun calc(data: &Vec<i32>) -> i32 {
            42
        }
    "#;

    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Failed to parse function with reference type parameter: {:?}",
        result.err()
    );
}

#[test]
fn test_simple_reference_type_parameter() {
    // Simple case: single function with &i32 parameter
    let code = r#"
        fun helper(x: &i32) -> i32 {
            42
        }
    "#;

    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Failed to parse simple reference type: {:?}",
        result.err()
    );
}

#[test]
fn test_reference_type_after_simple_function() {
    // Case: simple function followed by function with &i32 param
    let code = r#"
        fun main() {
            println("Hello");
        }

        fun helper(x: &i32) -> i32 {
            42
        }
    "#;

    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Failed to parse reference type after simple function: {:?}",
        result.err()
    );
}

#[test]
fn test_reference_vec_type() {
    // Specific case from Chapter 15: &Vec<i32>
    let code = r#"
        fun calculate_sum(data: &Vec<i32>) -> i32 {
            let mut total = 0;
            let mut i = 0;
            while i < data.len() {
                total = total + data[i];
                i = i + 1;
            }
            total
        }
    "#;

    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Failed to parse &Vec<i32> parameter: {:?}",
        result.err()
    );
}

#[test]
fn test_data_processor_example() {
    // Full example from Chapter 15
    let code = r#"
        fun main() {
            let data = vec![1, 2, 3, 4, 5];
            let sum = calculate_sum(&data);
            let avg = calculate_average(&data);
            println("Sum: {}, Avg: {}", sum, avg);
        }

        fun calculate_sum(data: &Vec<i32>) -> i32 {
            let mut total = 0;
            let mut i = 0;
            while i < data.len() {
                total = total + data[i];
                i = i + 1;
            }
            total
        }

        fun calculate_average(data: &Vec<i32>) -> f64 {
            let sum = calculate_sum(data);
            (sum as f64) / (data.len() as f64)
        }
    "#;

    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Failed to parse data processor example: {:?}",
        result.err()
    );
}
