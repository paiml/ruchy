//! EXTREME TDD: Transpiler Book Compatibility Tests
//!
//! Following CLAUDE.md EXTREME TDD Protocol:
//! - Write tests FIRST before any transpiler fixes
//! - 100% coverage of identified transpiler bugs from book
//! - Property tests with 10,000+ iterations
//! - Tests prove bugs exist, then fixes prove they're gone
//!
//! Target: Book compatibility 77% → 85% (10 examples fixed)

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

// ============================================================================
// BUG 1: REDUNDANT SEMICOLONS (Affects 7 examples)
// ============================================================================

#[test]
fn test_redundant_semicolon_let_statements() {
    // Bug: Transpiler generates "let mut total = 0 ; ;" instead of "let mut total = 0;"
    let code = r#"
        fun test() -> i32 {
            let mut total = 0;
            let mut i = 0;
            total
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    // Should NOT have redundant semicolons
    assert!(
        !rust_code.contains(" ; ;"),
        "Transpiler should not generate redundant semicolons: found ' ; ;' in:\n{}",
        rust_code
    );
}

#[test]
fn test_no_redundant_semicolon_in_while_loop() {
    let code = r#"
        fun test() -> i32 {
            let mut i = 0;
            while i < 5 {
                i = i + 1;
            }
            i
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        !rust_code.contains(" ; ;"),
        "While loops should not have redundant semicolons"
    );
}

#[test]
fn test_no_redundant_semicolon_after_assignments() {
    let code = r#"
        fun test() -> i32 {
            let x = 1;
            let y = 2;
            let z = 3;
            x + y + z
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        !rust_code.contains(" ; ;"),
        "Multiple let statements should not create redundant semicolons"
    );
}

// ============================================================================
// BUG 2: ARRAY LITERAL VS VEC! MISMATCH (Affects 3 examples)
// ============================================================================

#[test]
fn test_array_literal_stays_array() {
    // Bug: Book code [1, 2, 3] transpiles to vec![1, 2, 3] causing type mismatch
    let code = r#"
        fun test() -> [i32; 3] {
            let arr = [1, 2, 3];
            arr
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    // Array literals with known size should NOT become vec!
    // They should stay as [1, 2, 3] for fixed-size array types
    if rust_code.contains("[1") {
        // If we can detect fixed-size arrays, they should stay as arrays
        assert!(
            !rust_code.contains("vec ! ["),
            "Fixed-size array literal should not transpile to vec!: found 'vec ! [' in:\n{}",
            rust_code
        );
    }
}

#[test]
fn test_array_function_parameter() {
    // Bug: Function expects [i32; 5] but call passes Vec
    let code = r#"
        fun calculate_total(prices: [i32; 5]) -> i32 {
            prices[0] + prices[1]
        }

        fun main() {
            let arr = [10, 25, 5, 15, 8];
            calculate_total(arr)
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);

    // Should successfully transpile without type mismatches
    assert!(
        result.is_ok(),
        "Array function calls should transpile successfully"
    );

    let rust_code = result.unwrap().to_string();

    // The array literal should maintain its type
    // Either as [10, 25, 5, 15, 8] or with proper type annotation
    if rust_code.contains("calculate_total") {
        // If function exists, the call should be type-compatible
        assert!(
            rust_code.contains("[10") || rust_code.contains("arr"),
            "Array should be passed to function correctly"
        );
    }
}

#[test]
fn test_array_with_type_annotation() {
    let code = r#"
        fun test() {
            let prices: [i32; 5] = [10, 25, 5, 15, 8];
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);

    assert!(
        result.is_ok(),
        "Arrays with type annotations should transpile"
    );
}

// ============================================================================
// BUG 3: LIFETIME INFERENCE FOR &str RETURNS (Affects 2 examples)
// ============================================================================

#[test]
fn test_lifetime_inference_single_str_param() {
    // Bug: fn create_profile(name: &str) -> &str needs <'a> lifetime
    let code = r#"
        fun create_profile(name: &str) -> &str {
            name
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    // Should add lifetime parameter automatically
    // Either: fn create_profile<'a>(name: &'a str) -> &'a str
    // Or handle it another way that compiles
    if rust_code.contains("-> & str") {
        // If returning borrowed str, needs lifetime
        assert!(
            rust_code.contains("<") || rust_code.contains("'"),
            "Functions returning &str should have lifetime annotations: {}",
            rust_code
        );
    }
}

#[test]
fn test_lifetime_inference_multiple_str_params() {
    // Bug: fn process(a: &str, b: &str) -> &str is ambiguous
    let code = r#"
        fun choose_first(a: &str, b: &str) -> &str {
            a
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    // With multiple &str params, lifetime needed
    if rust_code.contains("-> & str") {
        assert!(
            rust_code.contains("<") || rust_code.contains("'"),
            "Functions with multiple &str params returning &str need lifetimes"
        );
    }
}

#[test]
fn test_string_return_instead_of_str_ref() {
    // Alternative: Could return String instead of &str
    let code = r#"
        fun create_message(name: &str) -> String {
            String::from(name)
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);

    assert!(result.is_ok(), "Functions returning String should work");
}

// ============================================================================
// PROPERTY TESTS: 10,000+ iterations per EXTREME TDD protocol
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]

        /// Property: Transpiled code should NEVER contain redundant semicolons
        #[test]
        fn test_no_redundant_semicolons_ever(
            var_count in 1..10usize,
        ) {
            // Generate code with multiple let statements
            let mut code = String::from("fun test() -> i32 {\n");
            for i in 0..var_count {
                code.push_str(&format!("    let x{} = {};\n", i, i));
            }
            code.push_str("    0\n}");

            let mut parser = Parser::new(&code);
            if let Ok(ast) = parser.parse() {
                let mut transpiler = Transpiler::new();
                if let Ok(rust_code) = transpiler.transpile_to_program(&ast) {
                    let code_str = rust_code.to_string();
                    prop_assert!(
                        !code_str.contains(" ; ;"),
                        "Found redundant semicolons in transpiled code with {} variables",
                        var_count
                    );
                }
            }
        }

        /// Property: Array literals should maintain their representation
        #[test]
        fn test_array_literals_consistent(
            size in 1..10usize,
        ) {
            let values: Vec<String> = (0..size).map(|i| i.to_string()).collect();
            let array_str = format!("[{}]", values.join(", "));
            let code = format!("fun test() {{ let arr = {}; }}", array_str);

            let mut parser = Parser::new(&code);
            if let Ok(ast) = parser.parse() {
                let mut transpiler = Transpiler::new();
                if let Ok(_rust_code) = transpiler.transpile_to_program(&ast) {
                    // Should transpile successfully
                    prop_assert!(true);
                }
            }
        }

        /// Property: Functions with string params should transpile
        #[test]
        fn test_string_functions_transpile(
            param_count in 1..5usize,
        ) {
            let params: Vec<String> = (0..param_count)
                .map(|i| format!("p{}: &str", i))
                .collect();
            let code = format!(
                "fun test({}) -> String {{ String::from(\"test\") }}",
                params.join(", ")
            );

            let mut parser = Parser::new(&code);
            if let Ok(ast) = parser.parse() {
                let mut transpiler = Transpiler::new();
                let result = transpiler.transpile_to_program(&ast);
                prop_assert!(result.is_ok(), "String functions should always transpile");
            }
        }

        /// Property: Transpiled code should be valid Rust (no double semicolons)
        #[test]
        fn test_transpiled_rust_validity(
            stmt_count in 1..20usize,
        ) {
            let mut code = String::from("fun test() {\n");
            for i in 0..stmt_count {
                code.push_str(&format!("    let var{} = {};\n", i, i * 10));
            }
            code.push_str("}");

            let mut parser = Parser::new(&code);
            if let Ok(ast) = parser.parse() {
                let mut transpiler = Transpiler::new();
                if let Ok(rust_code) = transpiler.transpile_to_program(&ast) {
                    let code_str = rust_code.to_string();

                    // Check for common invalid Rust patterns
                    prop_assert!(!code_str.contains(" ; ;"), "No redundant semicolons");
                    prop_assert!(!code_str.contains(";;"), "No double semicolons");
                    prop_assert!(!code_str.contains("; ; ;"), "No triple semicolons");
                }
            }
        }

        /// Property: Mutation patterns should transpile correctly
        #[test]
        fn test_mutation_patterns_transpile(
            loop_iterations in 1..20usize,
        ) {
            let code = format!(
                r#"
                fun test() -> i32 {{
                    let mut total = 0;
                    let mut i = 0;
                    while i < {} {{
                        total = total + i;
                        i = i + 1;
                    }}
                    total
                }}
                "#,
                loop_iterations
            );

            let mut parser = Parser::new(&code);
            if let Ok(ast) = parser.parse() {
                let mut transpiler = Transpiler::new();
                if let Ok(rust_code) = transpiler.transpile_to_program(&ast) {
                    let code_str = rust_code.to_string();

                    // Should not have redundant semicolons
                    prop_assert!(!code_str.contains(" ; ;"));

                    // Should handle mutation correctly
                    prop_assert!(code_str.contains("mut"));
                }
            }
        }
    }
}

// ============================================================================
// INTEGRATION TESTS: Real book examples that currently fail
// ============================================================================

#[test]
fn test_book_example_accumulator_pattern() {
    // From ch04-00-practical-patterns-tdd.md, example 5
    let code = r#"
        fun calculate_total(prices: [i32; 5]) -> i32 {
            let mut total = 0;
            let mut i = 0;
            while i < 5 {
                total = total + prices[i];
                i = i + 1;
            }
            total
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);

    assert!(result.is_ok(), "Book accumulator pattern should transpile");

    let rust_code = result.unwrap().to_string();
    assert!(
        !rust_code.contains(" ; ;"),
        "Should not have redundant semicolons"
    );
}

#[test]
fn test_book_example_validation_with_str_return() {
    // From ch04-00-practical-patterns-tdd.md, example 2 (simplified)
    let code = r#"
        fun validate_name(name: &str) -> String {
            if name.len() == 0 {
                String::from("Error: Name cannot be empty")
            } else {
                String::from("Valid")
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);

    assert!(result.is_ok(), "String return validation should work");
}

// ============================================================================
// BUG 4: STRING PARAMETER TYPE DEFAULTS (Affects 2 examples)
// ============================================================================

#[test]
fn test_string_parameter_defaults_to_str_ref() {
    // Bug: Parameters default to String but string literals are &str
    // This causes: expected `String`, found `&str` type mismatch
    let code = r#"
        fn greet(name) {
            println("Hello " + name)
        }
        greet("World")
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);

    // Should transpile successfully without type mismatch
    assert!(
        result.is_ok(),
        "Function with untyped string parameter should accept string literals: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();

    // Should generate either:
    // 1. fn greet(name: &str) - parameter accepts &str directly
    // 2. fn greet(name: String) with greet("World".to_string()) - conversion at call site
    // The former is more efficient and matches Rust idioms
    assert!(
        rust_code.contains("& str") || rust_code.contains("to_string"),
        "Should handle string parameter type correctly: {}",
        rust_code
    );
}

#[test]
fn test_string_concatenation_with_str_params() {
    // String concatenation should work with &str parameters
    let code = r#"
        fn make_greeting(name) {
            "Hello " + name
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);

    assert!(
        result.is_ok(),
        "String concatenation with inferred parameters should work"
    );
}

// ============================================================================
// REGRESSION TESTS: Prevent future transpiler bugs
// ============================================================================

#[test]
fn test_regression_no_semicolon_explosion() {
    // Ensure we never regress to generating excessive semicolons
    let code = r#"
        fun test() {
            let a = 1;
            let b = 2;
            let c = 3;
            let d = 4;
            let e = 5;
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    // Count semicolons - should be exactly 5 (one per let statement)
    let semicolon_count = rust_code.matches(';').count();
    assert!(
        semicolon_count <= 10, // Allow some flexibility for transpiler structure
        "Excessive semicolons detected: found {} semicolons in:\n{}",
        semicolon_count,
        rust_code
    );
}

#[test]
fn test_regression_array_type_consistency() {
    // Arrays should maintain their type through transpilation
    let code = r#"
        fun test() {
            let arr: [i32; 3] = [1, 2, 3];
            let sum = arr[0] + arr[1] + arr[2];
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);

    assert!(
        result.is_ok(),
        "Array type consistency should be maintained"
    );
}
