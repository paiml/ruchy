#![allow(missing_docs)]
//! EXTREME TDD Tests for Titanic `DataFrame` Example
//!
//! Test Strategy: RED → GREEN → REFACTOR
//! - RED: These tests WILL FAIL until example is implemented
//! - GREEN: Implement example to make tests pass
//! - REFACTOR: Apply PMAT quality gates
//!
//! Test Coverage:
//! 1. Example file exists and parses
//! 2. Transpiles to valid Rust with correct `DataFrame` API
//! 3. Method name mapping works (rows→height, columns→width)
//! 4. Polars imports generated correctly

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;
use std::path::Path;

/// RED TEST: Titanic example file must exist
#[test]
fn test_titanic_example_file_exists() {
    let example_path = "examples/titanic_dataframe.ruchy";
    assert!(
        Path::new(example_path).exists(),
        "Example file not found: {example_path}. Create it to make this test pass!"
    );
}

/// RED TEST: Titanic example must parse without errors
#[test]
fn test_titanic_example_parses() {
    let example_path = "examples/titanic_dataframe.ruchy";

    // Skip if file doesn't exist yet (will fail previous test)
    if !Path::new(example_path).exists() {
        eprintln!("Skipping: example file not created yet");
        return;
    }

    let code = std::fs::read_to_string(example_path).expect("Failed to read example file");

    let mut parser = Parser::new(&code);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Example failed to parse: {:?}",
        result.err()
    );
}

/// RED TEST: Titanic example must transpile to valid Rust
#[test]
fn test_titanic_example_transpiles() {
    let example_path = "examples/titanic_dataframe.ruchy";

    // Skip if file doesn't exist yet
    if !Path::new(example_path).exists() {
        eprintln!("Skipping: example file not created yet");
        return;
    }

    let code = std::fs::read_to_string(example_path).expect("Failed to read example file");

    let mut parser = Parser::new(&code);
    let ast = parser.parse().expect("Failed to parse example");

    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);

    assert!(
        result.is_ok(),
        "Failed to transpile example: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();

    // Verify correct DataFrame API is used
    assert!(
        rust_code.contains("DataFrame :: new (vec !"),
        "Transpiled code must use DataFrame::new(vec![...]) API.\nGenerated:\n{}",
        &rust_code[..500.min(rust_code.len())]
    );

    assert!(
        rust_code.contains("Series :: new"),
        "Transpiled code must use Series::new() for columns.\nGenerated:\n{}",
        &rust_code[..500.min(rust_code.len())]
    );

    assert!(
        rust_code.contains("use polars :: prelude :: *"),
        "Transpiled code must include polars import.\nGenerated:\n{}",
        &rust_code[..200.min(rust_code.len())]
    );

    // Should NOT contain wrong API
    assert!(
        !rust_code.contains("DataFrame :: empty () . column"),
        "Transpiled code must NOT use DataFrame::empty().column() (wrong API).\nFound in:\n{}",
        &rust_code[..500.min(rust_code.len())]
    );
}

/// RED TEST: `DataFrame` method names must be mapped correctly
#[test]
fn test_dataframe_method_name_mapping() {
    let code = r"
        fun analyze(df: DataFrame) {
            let row_count = df.rows()
            let col_count = df.columns()
            return row_count + col_count
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");

    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast).expect("Failed to transpile");
    let rust_code = result.to_string();

    // Verify method name mapping
    assert!(
        rust_code.contains(". height ()"),
        "df.rows() must transpile to df.height().\nGenerated:\n{rust_code}"
    );

    assert!(
        rust_code.contains(". width ()"),
        "df.columns() must transpile to df.width().\nGenerated:\n{rust_code}"
    );

    // Should NOT contain old method names
    assert!(
        !rust_code.contains(". rows ()"),
        "Transpiled code should NOT contain .rows() method.\nFound in:\n{rust_code}"
    );

    assert!(
        !rust_code.contains(". columns ()"),
        "Transpiled code should NOT contain .columns() method.\nFound in:\n{rust_code}"
    );
}

/// RED TEST: Builder pattern must transpile correctly
#[test]
fn test_dataframe_builder_pattern() {
    let code = r#"
        fun create_sample() {
            let df = DataFrame::new()
                .column("id", [1, 2, 3])
                .column("name", ["Alice", "Bob", "Charlie"])
                .build()
            return df
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");

    let mut transpiler = Transpiler::new();
    let result = transpiler
        .transpile_to_program(&ast)
        .expect("Failed to transpile");
    let rust_code = result.to_string();

    // Verify correct builder pattern transpilation
    assert!(
        rust_code.contains("DataFrame :: new (vec !"),
        "Builder pattern must transpile to DataFrame::new(vec![...]).\nGenerated:\n{}",
        &rust_code[..800.min(rust_code.len())]
    );

    assert!(
        rust_code.contains("Series :: new (\"id\""),
        "Must create Series for 'id' column.\nGenerated:\n{}",
        &rust_code[..800.min(rust_code.len())]
    );

    assert!(
        rust_code.contains("Series :: new (\"name\""),
        "Must create Series for 'name' column.\nGenerated:\n{}",
        &rust_code[..800.min(rust_code.len())]
    );

    assert!(
        rust_code.contains(". expect (\"Failed to create DataFrame\")"),
        "Must include error handling with .expect().\nGenerated:\n{}",
        &rust_code[..800.min(rust_code.len())]
    );
}

/// RED TEST: Multiple `DataFrames` in same file must work
#[test]
fn test_multiple_dataframes() {
    let code = r#"
        fun main() {
            let df1 = DataFrame::new()
                .column("a", [1, 2])
                .build()

            let df2 = DataFrame::new()
                .column("b", [3, 4])
                .build()

            println("DF1: {} rows", df1.rows())
            println("DF2: {} rows", df2.rows())
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");

    let mut transpiler = Transpiler::new();
    let result = transpiler
        .transpile_to_program(&ast)
        .expect("Failed to transpile");
    let rust_code = result.to_string();

    // Verify both DataFrames are created correctly
    let dataframe_count = rust_code.matches("DataFrame :: new (vec !").count();
    assert_eq!(
        dataframe_count,
        2,
        "Must create 2 DataFrames.\nGenerated:\n{}",
        &rust_code[..1000.min(rust_code.len())]
    );

    // Verify polars import is only included once
    let import_count = rust_code.matches("use polars :: prelude :: *").count();
    assert_eq!(
        import_count,
        1,
        "Polars import should appear exactly once.\nGenerated:\n{}",
        &rust_code[..200.min(rust_code.len())]
    );
}

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Property: Any valid `DataFrame` builder pattern should transpile without panicking
    #[test]
    fn property_dataframe_builder_never_panics() {
        let test_cases = vec![
            // Single column
            r#"let df = DataFrame::new().column("a", [1]).build()"#,
            // Multiple columns
            r#"let df = DataFrame::new().column("a", [1]).column("b", [2]).build()"#,
            // Without .build()
            r#"let df = DataFrame::new().column("a", [1])"#,
            // Empty DataFrame
            r"let df = DataFrame::new().build()",
        ];

        for (i, code) in test_cases.iter().enumerate() {
            let full_code = format!("fun test() {{ {code} }}");

            let mut parser = Parser::new(&full_code);
            let parse_result = parser.parse();

            if let Ok(ast) = parse_result {
                let mut transpiler = Transpiler::new();
                let transpile_result = transpiler.transpile_to_program(&ast);

                assert!(
                    transpile_result.is_ok(),
                    "Test case {} failed: {}. Error: {:?}",
                    i,
                    code,
                    transpile_result.err()
                );

                if let Ok(tokens) = transpile_result {
                    let rust_code = tokens.to_string();
                    // Basic sanity check
                    assert!(
                        rust_code.contains("DataFrame") || rust_code.contains("dataframe"),
                        "Test case {} produced unexpected output: {}",
                        i,
                        &rust_code[..200.min(rust_code.len())]
                    );
                }
            }
        }
    }
}
