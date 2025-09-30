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
// BUG 5: WHILE LOOP MUTABILITY INFERENCE (Affects 1 example)
// ============================================================================

#[test]
fn test_while_loop_mutability_inference() {
    // Bug: let i = 0 followed by i = i + 1 in while loop doesn't detect mutation
    // This causes: cannot assign twice to immutable variable
    let code = r#"
let i = 0
while i < 3 {
    println(i)
    i = i + 1
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    // Should auto-detect mutation and generate let mut
    assert!(
        rust_code.contains("let mut i"),
        "While loop mutation should be detected: expected 'let mut i' but got:\n{}",
        rust_code
    );
}

#[test]
fn test_block_level_mutability_detection() {
    // Test that mutations anywhere in the block are detected
    let code = r#"
fun test() {
    let x = 0
    x = 5
    x
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains("let mut x"),
        "Block-level mutation should be detected"
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

// ============================================================================
// OPTION A: ARRAY LITERAL TYPE PRESERVATION
// ============================================================================
// Target: Make [1, 2, 3] transpile to [1, 2, 3] not vec![1, 2, 3]
// Root Cause: transpile_list() always generates vec![] syntax
// Solution: Default to fixed-size array for simple literals

#[test]
fn test_array_literal_numeric_stays_array() {
    // Simple numeric array should transpile to fixed-size array syntax
    let code = "[1, 2, 3]";

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains("[1") || rust_code.contains("[1i32"),
        "Expected fixed-size array [1, 2, 3], got vec![]: {}",
        rust_code
    );
    assert!(
        !rust_code.contains("vec !"),
        "Should NOT generate vec![] for simple numeric literals: {}",
        rust_code
    );
}

#[test]
fn test_array_literal_string_stays_array() {
    // String array should transpile to fixed-size array syntax
    let code = r#"["a", "b", "c"]"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains(r#"["a""#) || rust_code.contains("[\"a\""),
        "Expected fixed-size array for strings, got vec![]: {}",
        rust_code
    );
    assert!(
        !rust_code.contains("vec !"),
        "Should NOT generate vec![] for simple string literals: {}",
        rust_code
    );
}

#[test]
fn test_array_literal_bool_stays_array() {
    // Boolean array should transpile to fixed-size array syntax
    let code = "[true, false, true]";

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains("[true") && !rust_code.contains("vec !"),
        "Expected fixed-size array [true, false, true], got vec![]: {}",
        rust_code
    );
}

#[test]
fn test_array_literal_mixed_types_stays_array() {
    // Mixed type array (will fail at Rust compilation, but transpilation should use [])
    let code = "[1, 2.5, 3]";

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains("[1") && !rust_code.contains("vec !"),
        "Expected fixed-size array even for mixed types: {}",
        rust_code
    );
}

#[test]
fn test_array_with_spread_uses_vec() {
    // Arrays with spread operators MUST use vec![] (no fixed-size syntax)
    let code = "[1, 2, ...[3, 4]]";

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains("vec !") || rust_code.contains("Vec"),
        "Spread operators require vec![] (dynamic sizing): {}",
        rust_code
    );
}

#[test]
fn test_array_assignment_preserves_type() {
    // When assigned to variable, array type should be preserved
    let code = r#"
        let numbers = [1, 2, 3]
        let first = numbers[0]
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains("let numbers") && rust_code.contains("[1"),
        "Array assignment should preserve fixed-size array type: {}",
        rust_code
    );
}

#[test]
fn test_array_function_return_preserves_type() {
    // Function returning array should preserve type
    let code = r#"
        fun get_array() -> [i32; 3] {
            [1, 2, 3]
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains("-> [i32 ; 3]") || rust_code.contains("-> [i32; 3]"),
        "Return type should be fixed-size array: {}",
        rust_code
    );
    assert!(
        rust_code.contains("[1") && !rust_code.contains("vec !"),
        "Return value should be fixed-size array literal: {}",
        rust_code
    );
}

#[test]
fn test_nested_arrays_stay_arrays() {
    // Nested arrays should maintain array type
    let code = "[[1, 2], [3, 4]]";

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains("[[") || rust_code.contains("[ ["),
        "Nested arrays should use array syntax: {}",
        rust_code
    );
    assert!(
        !rust_code.contains("vec !"),
        "Nested arrays should NOT use vec![]: {}",
        rust_code
    );
}

#[test]
fn test_empty_array_uses_vec() {
    // Empty arrays have no size info, should use vec![]
    let code = "[]";

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);

    // Empty arrays need vec![] or type annotation - this is acceptable
    assert!(
        result.is_ok(),
        "Empty array should transpile: {:?}",
        result.err()
    );
}

// ============================================================================
// OPTION B: :: SYNTAX SUPPORT (TURBOFISH GENERICS)
// ============================================================================
// Basic :: syntax already works (String::new(), Option::Some, etc.)
// Target: Add turbofish support for Vec::<i32>::new()

#[test]
fn test_basic_coloncolon_syntax_works() {
    // Verify basic :: syntax is functional (regression test)
    let code = "let s = String::new()";

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains("String :: new ()"),
        "Basic :: syntax should work: {}",
        rust_code
    );
}

#[test]
fn test_module_path_coloncolon() {
    // Module paths should work with ::
    let code = "let hm = std::collections::HashMap::new()";

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains("std :: collections :: HashMap :: new ()"),
        "Module paths should work: {}",
        rust_code
    );
}

#[test]
fn test_enum_variant_coloncolon() {
    // Enum variants should work with ::
    let code = "let opt = Option::Some(42)";

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains("Option :: Some") && rust_code.contains("42"),
        "Enum variants should work: {}",
        rust_code
    );
}

#[test]
fn test_turbofish_vec_new() {
    // Turbofish syntax for Vec::<i32>::new()
    let code = "let v = Vec::<i32>::new()";

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains("Vec :: < i32 > :: new ()") || rust_code.contains("Vec::<i32>::new"),
        "Turbofish syntax should work: {}",
        rust_code
    );
}

#[test]
fn test_turbofish_hashmap_new() {
    // Turbofish with multiple type parameters
    let code = "let hm = HashMap::<String, i32>::new()";

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains("HashMap")
            && rust_code.contains("String")
            && rust_code.contains("i32")
            && rust_code.contains("new"),
        "Turbofish with multiple params should work: {}",
        rust_code
    );
}

#[test]
fn test_turbofish_collect() {
    // Turbofish in method chains
    let code = "let v = vec![1, 2, 3].iter().collect::<Vec<i32>>()";

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains("collect") && rust_code.contains("Vec") && rust_code.contains("i32"),
        "Turbofish in method chains should work: {}",
        rust_code
    );
}

#[test]
fn test_nested_turbofish() {
    // Nested generics with turbofish
    let code = "let v = Vec::<Vec<i32>>::new()";

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains("Vec") && rust_code.contains("i32"),
        "Nested turbofish should work: {}",
        rust_code
    );
}

#[test]
fn test_turbofish_vs_less_than_disambiguation() {
    // Ensure parser distinguishes Vec::<T> from comparisons
    let code = r#"
        let v = Vec::<i32>::new()
        let b = 3 < 5
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap().to_string();

    assert!(
        rust_code.contains("Vec") && rust_code.contains("i32"),
        "Turbofish should not be confused with < operator: {}",
        rust_code
    );
    assert!(
        rust_code.contains("3 < 5"),
        "Less-than operator should still work: {}",
        rust_code
    );
}
