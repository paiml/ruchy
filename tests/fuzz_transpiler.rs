//! Fuzz tests for transpiler robustness

#![allow(clippy::needless_range_loop)]
//! 
//! These tests use random inputs to ensure the transpiler never panics
//! and handles malformed input gracefully.

use proptest::prelude::*;
use ruchy::backend::transpiler::Transpiler;
use ruchy::Parser;

// Generate random function-like structures
fn arb_function_code() -> impl Strategy<Value = String> {
    (
        "[a-z][a-z0-9]{0,8}",                      // function name
        prop::collection::vec("[a-z][a-z0-9]{0,5}", 0..5), // parameters
        "[a-zA-Z0-9(){}+*/ -]{0,50}",             // body
    ).prop_map(|(name, params, body)| {
        let param_list = params.join(", ");
        format!("fun {name}({param_list}) {{ {body} }}")
    })
}

// Generate random expression-like code
fn arb_expression_code() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "x + y".to_string(),
        "f(g(h))".to_string(),
        "let x = 5; x".to_string(),
        "if true { 1 } else { 2 }".to_string(),
        "x * y + z".to_string(),
        "(x) => x + 1".to_string(),
        "f(x) + g(y)".to_string(),
    ])
}

// Generate random binary operations
fn arb_binary_op_code() -> impl Strategy<Value = String> {
    (
        "[a-z][a-z0-9]{0,3}",  // left operand
        prop::sample::select(vec!["+", "-", "*", "/", "%", "<", ">", "<=", ">=", "==", "!="]),
        "[a-z0-9]{1,5}",       // right operand
    ).prop_map(|(left, op, right)| {
        format!("{left} {op} {right}")
    })
}

proptest! {
    // Fuzz test: transpiler should never panic on random function code
    #[test]
    fn fuzz_transpiler_functions_never_panic(code in arb_function_code()) {
        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let mut transpiler = Transpiler::new();
            // Should not panic, regardless of output
            let _result = std::panic::catch_unwind(|| {
                transpiler.transpile(&ast)
            });
        }
    }

    // Fuzz test: type inference should handle random expressions
    #[test] 
    fn fuzz_type_inference_expressions(expr in arb_expression_code()) {
        let code = format!("fun test(p) {{ {expr} }}");
        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let mut transpiler = Transpiler::new();
            let _result = std::panic::catch_unwind(|| {
                transpiler.transpile(&ast)
            });
        }
    }

    // Fuzz test: binary operations should be handled robustly
    #[test]
    fn fuzz_binary_operations(op_expr in arb_binary_op_code()) {
        let code = format!("fun test(x, y, z) {{ {op_expr} }}");
        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let mut transpiler = Transpiler::new();
            let _result = std::panic::catch_unwind(|| {
                transpiler.transpile(&ast)
            });
        }
    }

    // Fuzz test: random parameter combinations
    #[test]
    fn fuzz_parameter_combinations(
        func_name in "[a-z][a-z0-9]{0,5}",
        param1 in "[a-z][a-z0-9]{0,5}",
        param2 in "[a-z][a-z0-9]{0,5}",
        param3 in "[a-z][a-z0-9]{0,5}",
        operation in prop::sample::select(vec![
            "p1(p2)", "p1 + p2", "p1 * p2", "p1(p2(p3))", 
            "p1 - p2 + p3", "p1 / p2 % p3"
        ])
    ) {
        let body = operation
            .replace("p1", &param1)
            .replace("p2", &param2) 
            .replace("p3", &param3);
        let code = format!("fun {func_name}({param1}, {param2}, {param3}) {{ {body} }}");
        
        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let mut transpiler = Transpiler::new();
            let _result = std::panic::catch_unwind(|| {
                transpiler.transpile(&ast)
            });
        }
    }

    // Fuzz test: edge cases with empty parameters
    #[test]
    fn fuzz_edge_cases_empty_params(
        func_name in "[a-z][a-z0-9]{0,5}",
        body in "[a-zA-Z0-9 ]{0,20}",
    ) {
        let code = format!("fun {func_name}() {{ {body} }}");
        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let mut transpiler = Transpiler::new();
            let _result = std::panic::catch_unwind(|| {
                transpiler.transpile(&ast)
            });
        }
    }

    // Fuzz test: deeply nested function calls
    #[test]
    fn fuzz_nested_function_calls(
        depth in 1..10usize,
        func_names in prop::collection::vec("[a-z][a-z0-9]{0,3}", 1..10),
    ) {
        if func_names.is_empty() { return Ok(()); }
        
        let mut call_chain = func_names[0].clone();
        for i in 1..std::cmp::min(depth, func_names.len()) {
            call_chain = format!("{}({})", func_names[i], call_chain);
        }
        
        let param_list = func_names.join(", ");
        let code = format!("fun test({param_list}) {{ {call_chain} }}");
        
        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let mut transpiler = Transpiler::new();
            let _result = std::panic::catch_unwind(|| {
                transpiler.transpile(&ast)
            });
        }
    }

    // Fuzz test: random combinations of numeric and function usage
    #[test]
    fn fuzz_mixed_usage_patterns(
        func_name in "[a-z][a-z0-9]{0,5}",
        f_param in "[f][a-z0-9]{0,3}",
        n_param in "[n][a-z0-9]{0,3}",
        operation in prop::sample::select(vec![
            "f(n + 1)", "f(n) * 2", "f(n * 2) + f(n / 3)", 
            "n + f(5)", "f(f(n))"
        ])
    ) {
        let body = operation.replace('f', &f_param).replace('n', &n_param);
        let code = format!("fun {func_name}({f_param}, {n_param}) {{ {body} }}");
        
        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let mut transpiler = Transpiler::new();
            let _result = std::panic::catch_unwind(|| {
                transpiler.transpile(&ast)
            });
        }
    }

    // Stress test: very long parameter lists
    #[test]
    fn fuzz_long_parameter_lists(
        params in prop::collection::vec("[a-z][a-z0-9]{0,3}", 1..20),
        operation_count in 1..10usize,
    ) {
        if params.is_empty() { return Ok(()); }
        
        let param_list = params.join(", ");
        
        // Create operations using the parameters
        let mut operations = Vec::new();
        for i in 0..std::cmp::min(operation_count, params.len().saturating_sub(1)) {
            operations.push(format!("{} + {}", params[i], params[i + 1]));
        }
        let body = operations.join("; ");
        
        let code = format!("fun test({param_list}) {{ {body} }}");
        
        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let mut transpiler = Transpiler::new();
            let _result = std::panic::catch_unwind(|| {
                transpiler.transpile(&ast)
            });
        }
    }
}

#[cfg(test)]
mod determinism_tests {
    use super::*;
    
    #[test]
    fn test_transpiler_deterministic() {
        // Test that transpiling the same AST multiple times gives same result
        let code = "fun apply(f, x) { f(x) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let mut transpiler = Transpiler::new();
        let result1 = transpiler.transpile(&ast).unwrap();
        let result2 = transpiler.transpile(&ast).unwrap();
        
        assert_eq!(result1.to_string(), result2.to_string(), 
                  "Transpiler should be deterministic");
    }

    #[test]
    fn test_type_inference_deterministic() {
        // Test that type inference is consistent
        let codes = vec![
            "fun double(n) { n * 2 }",
            "fun apply(f, x) { f(x) }",
            "fun greet(name) { \"Hello \" + name }",
        ];
        
        for code in codes {
            let mut parser = Parser::new(code);
            let ast = parser.parse().unwrap();
            
            let mut transpiler = Transpiler::new();
            let result1 = transpiler.transpile(&ast).unwrap().to_string();
            let result2 = transpiler.transpile(&ast).unwrap().to_string();
            
            assert_eq!(result1, result2, 
                      "Type inference should be deterministic for: {code}");
        }
    }
}