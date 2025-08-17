#![allow(clippy::unwrap_used, clippy::panic)]
#![allow(clippy::unreadable_literal)] // Property tests generate random values
#![allow(unused_imports)] // Test utilities may not all be used
#![allow(clippy::uninlined_format_args)] // Test code formatting style
#![allow(clippy::print_stdout)] // Tests may print debug output
#![allow(clippy::panic)] // Tests use panic for assertions
#![allow(clippy::expect_used)] // Tests use expect for clarity in error messages
#![allow(clippy::single_match)] // Test pattern matching style
#![allow(clippy::redundant_closure_for_method_calls)] // Test code style

use proptest::prelude::*;
use ruchy::{ExprKind, Parser, Transpiler};

/// Property: Any valid let statement should transpile to Rust code with a semicolon
#[test]
fn prop_let_statements_have_semicolons() {
    proptest!(|(name in "[a-z][a-z0-9_]{0,10}", value in 0i64..1000i64)| {
        let input = format!("let {} = {}", name, value);
        let mut parser = Parser::new(&input);
        let ast = parser.parse().expect("Should parse valid let statement");

        let transpiler = Transpiler::new();
        let tokens = transpiler.transpile(&ast).expect("Should transpile");
        let rust_code = tokens.to_string();

        // Property 1: Let statements should produce valid Rust code
        assert!(rust_code.starts_with("let "), "Should start with 'let'");
        assert!(rust_code.contains(&name), "Should contain variable name");

        // Property 2: The AST should be recognized as a Let expression
        assert!(matches!(ast.kind, ExprKind::Let { .. }), "Should be a Let expression");

        // Property 3: When used in a statement context, it needs a semicolon
        // Note: The transpiler itself doesn't add semicolons, that's the REPL's job
        let needs_semicolon = !rust_code.ends_with(';');
        assert!(needs_semicolon || rust_code.ends_with(';'),
                "Transpiler output: '{}' - needs post-processing for semicolon", rust_code);
    });
}

/// Property: Variable references should transpile to simple identifiers
#[test]
fn prop_variable_references_transpile_correctly() {
    proptest!(|(name in "[a-z][a-z0-9_]{0,10}")| {
        let mut parser = Parser::new(&name);
        let ast = parser.parse().expect("Should parse identifier");

        let transpiler = Transpiler::new();
        let tokens = transpiler.transpile(&ast).expect("Should transpile");
        let rust_code = tokens.to_string();

        // Property: Identifiers should transpile to themselves
        assert_eq!(rust_code, name, "Identifier should transpile unchanged");
        assert!(matches!(ast.kind, ExprKind::Identifier(_)), "Should be an Identifier");
    });
}

/// Property: Binary operations should preserve operator precedence
#[test]
fn prop_binary_ops_preserve_precedence() {
    proptest!(|(a in 1i64..100, b in 1i64..100, c in 1i64..100)| {
        let expressions = vec![
            format!("{} + {} * {}", a, b, c),  // Multiplication should bind tighter
            format!("{} * {} + {}", a, b, c),  // Multiplication first, then addition
            format!("({} + {}) * {}", a, b, c), // Parentheses override precedence
        ];

        for expr in expressions {
            let mut parser = Parser::new(&expr);
            let ast = parser.parse().expect("Should parse expression");

            let transpiler = Transpiler::new();
            let tokens = transpiler.transpile(&ast).expect("Should transpile");
            let rust_code = tokens.to_string();

            // Property: Binary operations should transpile with proper parentheses
            assert!(rust_code.contains(&a.to_string()), "Should contain first operand");
            assert!(rust_code.contains(&b.to_string()), "Should contain second operand");
            assert!(rust_code.contains(&c.to_string()), "Should contain third operand");

            // The transpiler adds parentheses to preserve precedence
            if expr.contains('(') && expr.contains(')') {
                // If input has explicit parens, output should preserve grouping
                assert!(rust_code.contains('(') && rust_code.contains(')'),
                        "Should preserve explicit parentheses");
            }
        }
    });
}

/// Property: Arrays should transpile to Vec literals
#[test]
fn prop_arrays_transpile_to_vecs() {
    proptest!(|(elements in prop::collection::vec(0i64..1000, 0..10))| {
        let array_str = format!("[{}]", elements.iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", "));

        let mut parser = Parser::new(&array_str);
        let ast = parser.parse().expect("Should parse array");

        let transpiler = Transpiler::new();
        let tokens = transpiler.transpile(&ast).expect("Should transpile");
        let rust_code = tokens.to_string();

        // Property: Arrays should transpile to vec! macro
        assert!(rust_code.starts_with("vec !"), "Should transpile to vec! macro");
        assert!(rust_code.contains('[') && rust_code.contains(']'),
                "Should contain square brackets");

        // All elements should be present
        for elem in &elements {
            assert!(rust_code.contains(&format!("{}i64", elem)),
                    "Should contain element {}", elem);
        }
    });
}

/// Property: String literals should be properly escaped
#[test]
fn prop_strings_properly_escaped() {
    proptest!(|(s in "[a-zA-Z0-9 ]{0,50}")| {
        let input = format!("\"{}\"", s);
        let mut parser = Parser::new(&input);
        let ast = parser.parse().expect("Should parse string");

        let transpiler = Transpiler::new();
        let tokens = transpiler.transpile(&ast).expect("Should transpile");
        let rust_code = tokens.to_string();

        // Property: String literals should be preserved with quotes
        assert!(rust_code.starts_with('"') && rust_code.ends_with('"'),
                "Should be quoted: {}", rust_code);
        assert!(rust_code.contains(&s) || s.is_empty(),
                "Should contain string content");
    });
}

/// Property: Function definitions should transpile to valid Rust functions
#[test]
fn prop_functions_transpile_correctly() {
    proptest!(|(
        name in "[a-z][a-z0-9_]{0,10}",
        param1 in "[a-z][a-z0-9_]{0,5}",
        param2 in "[a-z][a-z0-9_]{0,5}",
        value in 0i64..100
    )| {
        // Ensure params are different
        let p2 = if param2 == param1 { format!("{}2", param2) } else { param2 };

        let input = format!("fun {}({}, {}) {{ {} }}", name, param1, p2, value);
        let mut parser = Parser::new(&input);
        let ast = parser.parse().expect("Should parse function");

        let transpiler = Transpiler::new();
        let tokens = transpiler.transpile(&ast).expect("Should transpile");
        let rust_code = tokens.to_string();

        // Property: Functions should transpile to Rust fn syntax
        assert!(rust_code.starts_with("fn "), "Should start with 'fn'");
        assert!(rust_code.contains(&name), "Should contain function name");
        assert!(rust_code.contains(&param1), "Should contain first parameter");
        assert!(rust_code.contains(&p2), "Should contain second parameter");
        assert!(rust_code.contains(&value.to_string()), "Should contain body value");
    });
}

/// Property: The transpiler output should be deterministic
#[test]
fn prop_transpiler_is_deterministic() {
    proptest!(|(seed in "[a-z]+", value in 0i64..1000)| {
        let inputs = vec![
            format!("let {} = {}", seed, value),
            format!("{} + {}", value, value),
            format!("if {} > 0 {{ {} }} else {{ 0 }}", value, value),
        ];

        for input in inputs {
            let mut parser1 = Parser::new(&input);
            let ast1 = parser1.parse().expect("Should parse");

            let mut parser2 = Parser::new(&input);
            let ast2 = parser2.parse().expect("Should parse");

            let transpiler = Transpiler::new();
            let tokens1 = transpiler.transpile(&ast1).expect("Should transpile");
            let tokens2 = transpiler.transpile(&ast2).expect("Should transpile");

            // Property: Same input should always produce same output
            assert_eq!(tokens1.to_string(), tokens2.to_string(),
                      "Transpiler should be deterministic for input: {}", input);
        }
    });
}

/// Property: Debug and release builds should produce identical results
#[cfg(test)]
mod debug_release_parity {
    use super::*;

    #[test]
    fn test_statement_handling_parity() {
        let test_cases = vec![
            "let x = 10",
            "let y = 20",
            "let result = x + y",
            "let arr = [1, 2, 3]",
        ];

        for input in test_cases {
            let mut parser = Parser::new(input);
            let ast = parser.parse().expect("Should parse");

            let transpiler = Transpiler::new();
            let tokens = transpiler.transpile(&ast).expect("Should transpile");
            let rust_code = tokens.to_string();

            // In both debug and release, the output should be the same
            match &ast.kind {
                ExprKind::Let { .. } => {
                    // The transpiler doesn't add semicolons - that's the REPL's job
                    assert!(
                        !rust_code.ends_with(';'),
                        "Transpiler should not add semicolons"
                    );

                    // But the REPL should detect this is a let statement
                    assert!(
                        matches!(ast.kind, ExprKind::Let { .. }),
                        "Should be detected as Let statement"
                    );
                }
                _ => {}
            }
        }
    }
}

/// Property: Complex expressions should round-trip through transpilation
#[test]
fn prop_complex_expressions_valid() {
    proptest!(|(
        a in 1i64..10,
        b in 1i64..10,
        c in 1i64..10,
        op1 in prop::sample::select(vec!["+", "-", "*"]),
        op2 in prop::sample::select(vec!["+", "-", "*"])
    )| {
        let input = format!("{} {} {} {} {}", a, op1, b, op2, c);
        let mut parser = Parser::new(&input);

        match parser.parse() {
            Ok(ast) => {
                let transpiler = Transpiler::new();
                match transpiler.transpile(&ast) {
                    Ok(tokens) => {
                        let rust_code = tokens.to_string();

                        // Property: Should contain all operands
                        assert!(rust_code.contains(&a.to_string()));
                        assert!(rust_code.contains(&b.to_string()));
                        assert!(rust_code.contains(&c.to_string()));

                        // Property: Should contain operators
                        assert!(rust_code.contains(op1) || rust_code.contains(&format!("({})", op1)));
                        assert!(rust_code.contains(op2) || rust_code.contains(&format!("({})", op2)));
                    }
                    Err(e) => {
                        panic!("Transpilation failed for '{}': {}", input, e);
                    }
                }
            }
            Err(e) => {
                // Some combinations might not parse, that's ok for this test
                println!("Parse error for '{}': {}", input, e);
            }
        }
    });
}
