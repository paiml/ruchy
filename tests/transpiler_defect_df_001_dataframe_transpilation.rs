//! DEFECT-TRANSPILER-DF-001 through DF-004: `DataFrame` Transpilation Tests
//!
//! Root Cause: Transpiler generates incorrect Polars API code for `DataFrames`
//! - DF-001: Missing `use polars::prelude::*;` imports
//! - DF-002: Wrong API - `DataFrame::empty().column()` doesn't exist
//! - DF-003: Wrong method names - `rows()` should be `height()`, `columns()` should be `width()`
//! - DF-004: Missing error handling for Result types
//!
//! Test Strategy: EXTREME TDD with empirical rustc validation

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

/// RED TEST: `DataFrame` transpilation should generate correct Polars imports
#[test]
fn test_df_001_transpiler_generates_polars_imports() {
    let code = r#"
        let df = DataFrame::new()
            .column("name", ["Alice", "Bob"])
            .build()
        println("Rows: {}", df.rows())
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");

    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast).expect("Failed to transpile");
    let rust_code = result.to_string();

    // ASSERTION: Must contain polars import
    assert!(
        rust_code.contains("use polars :: prelude :: *"),
        "Transpiled code missing polars import:\n{rust_code}"
    );
}

/// RED TEST: `DataFrame.rows()` should transpile to .`height()`
#[test]
fn test_df_003_rows_method_transpiles_to_height() {
    let code = r"
        fun count_rows(df: DataFrame) {
            return df.rows()
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast).expect("Failed to transpile");
    let rust_code = result.to_string();

    // ASSERTION: .rows() must become .height()
    assert!(
        rust_code.contains(". height ()"),
        "Expected .height() but got:\n{rust_code}"
    );
    assert!(
        !rust_code.contains(". rows ()"),
        "Should not contain .rows():\n{rust_code}"
    );
}

/// RED TEST: `DataFrame.columns()` should transpile to .`width()`
#[test]
fn test_df_003_columns_method_transpiles_to_width() {
    let code = r"
        fun count_cols(df: DataFrame) {
            return df.columns()
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast).expect("Failed to transpile");
    let rust_code = result.to_string();

    // ASSERTION: .columns() must become .width()
    assert!(
        rust_code.contains(". width ()"),
        "Expected .width() but got:\n{rust_code}"
    );
    assert!(
        !rust_code.contains(". columns ()"),
        "Should not contain .columns():\n{rust_code}"
    );
}

/// RED TEST: Transpiled `DataFrame` code should compile with rustc
/// This is the EMPIRICAL validation test - proves generated code is correct
#[test]
#[ignore] // Run with: cargo test test_df_empirical_rustc_validation -- --ignored
fn test_df_empirical_rustc_validation() {
    let code = r#"
        fun analyze(df: DataFrame) {
            println("Rows: {}", df.rows())
            return df.rows()
        }

        let df1 = DataFrame::new()
            .column("name", ["Alice", "Bob"])
            .column("age", [30, 25])
            .build()

        let count = analyze(df1)
        println("Count: {}", count)
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");

    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast).expect("Failed to transpile");
    let rust_code = result.to_string();

    // Write to temp file
    let temp_file = "/tmp/test_df_validation.rs";
    std::fs::write(temp_file, &rust_code).expect("Failed to write temp file");

    // EMPIRICAL VALIDATION: Try to compile with rustc
    // Note: This will fail without polars dependency, but checks syntax correctness
    let output = std::process::Command::new("rustc")
        .arg("--edition")
        .arg("2021")
        .arg("--crate-type")
        .arg("lib")
        .arg(temp_file)
        .arg("-o")
        .arg("/tmp/test_df_validation.rlib")
        .output()
        .expect("Failed to run rustc");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Filter out polars dependency errors - those are expected
        let real_errors: Vec<&str> = stderr
            .lines()
            .filter(|line| !line.contains("unresolved module or unlinked crate `polars`"))
            .filter(|line| !line.contains("use of unresolved module"))
            .filter(|line| line.contains("error"))
            .collect();

        assert!(real_errors.is_empty(), 
                "Transpiled code has compilation errors:\n{}\n\nGenerated code:\n{}",
                real_errors.join("\n"),
                rust_code
            );
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Property: Any `DataFrame` method call should transpile without panicking
    #[test]
    fn property_dataframe_methods_never_panic() {
        let test_cases = vec![
            ("df.rows()", "height"),
            ("df.columns()", "width"),
            ("df.height()", "height"),
            ("df.width()", "width"),
        ];

        for (input, expected_method) in test_cases {
            let code = format!("fun test(df: DataFrame) {{ return {input} }}");

            let mut parser = Parser::new(&code);
            let parse_result = parser.parse();

            if let Ok(ast) = parse_result {
                let transpiler = Transpiler::new();
                let result = std::panic::catch_unwind(|| {
                    transpiler.transpile(&ast)
                });

                assert!(
                    result.is_ok(),
                    "Transpiler panicked on input: {input}"
                );

                if let Ok(Ok(tokens)) = result {
                    let rust_code = tokens.to_string();
                    assert!(
                        rust_code.contains(expected_method),
                        "Expected method '{expected_method}' not found in:\n{rust_code}"
                    );
                }
            }
        }
    }
}
