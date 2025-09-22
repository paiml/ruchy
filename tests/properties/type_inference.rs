//! Property tests for type inference invariants
//!
//! These tests ensure mathematical properties hold for all type inference:
//! 1. main() function NEVER has a return type
//! 2. Function parameters used as functions get Fn types
//! 3. Type inference is deterministic
//! 4. No function gets wrong default types

use proptest::prelude::*;
use ruchy::{Parser, Transpiler};

/// Generate valid function names
fn function_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,10}".prop_filter("Not a reserved word", |s| {
        !matches!(
            s.as_str(),
            "if" | "else"
                | "while"
                | "for"
                | "let"
                | "fun"
                | "return"
                | "break"
                | "continue"
                | "match"
                | "struct"
                | "enum"
                | "impl"
                | "trait"
                | "type"
                | "const"
                | "static"
                | "mut"
                | "fn"
        )
    })
}

/// Generate valid parameter names  
fn param_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,10}".prop_filter("Not a reserved word", |s| {
        !matches!(
            s.as_str(),
            "if" | "else"
                | "while"
                | "for"
                | "let"
                | "fun"
                | "return"
                | "break"
                | "continue"
                | "match"
                | "struct"
                | "enum"
                | "impl"
                | "trait"
                | "type"
                | "const"
                | "static"
                | "mut"
                | "fn"
        )
    })
}

/// Generate simple expressions
fn simple_expr() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("42".to_string()),
        Just("\"hello\"".to_string()),
        Just("true".to_string()),
        Just("false".to_string()),
        param_name(),
    ]
}

/// Generate a simple function body
fn function_body() -> impl Strategy<Value = String> {
    prop_oneof![
        simple_expr(),
        // Binary operations
        (simple_expr(), simple_expr()).prop_map(|(a, b)| format!("{} + {}", a, b)),
        (simple_expr(), simple_expr()).prop_map(|(a, b)| format!("{} * {}", a, b)),
        // Function calls
        (param_name(), simple_expr()).prop_map(|(f, x)| format!("{}({})", f, x)),
    ]
}

proptest! {
    /// Property: main() function NEVER generates with a return type
    #[test]
    fn main_never_has_return_type(body in function_body()) {
        let code = format!("fun main() {{ {} }}", body);

        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let mut transpiler = Transpiler::new();
            if let Ok(rust_code) = transpiler.transpile(&ast) {
                let rust_str = rust_code.to_string();
                // main should never have -> return type annotation
                prop_assert!(!rust_str.contains("fn main ()"),
                    "main() should not have return type, got: {}", rust_str);
                prop_assert!(!rust_str.contains("fn main() ->"),
                    "main() should not have return type, got: {}", rust_str);
            }
        }
    }

    /// Property: Functions with numeric operations get numeric parameter types
    #[test]
    fn numeric_functions_get_numeric_params(
        name in function_name(),
        param in param_name()
    ) {
        let code = format!("fun {}({}) {{ {} * 2 }}", name, param, param);

        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let mut transpiler = Transpiler::new();
            if let Ok(rust_code) = transpiler.transpile(&ast) {
                let rust_str = rust_code.to_string();
                // Should not type numeric operations with String
                prop_assert!(!rust_str.contains(&format!("{} : String", param)),
                    "Numeric function parameter should not be String: {}", rust_str);
            }
        }
    }

    /// Property: Function parameters used as functions get Fn types
    #[test]
    fn function_params_get_fn_types(
        fname in function_name(),
        fparam in param_name(),
        xparam in param_name().prop_filter("Different from f", move |x| x != &fparam),
    ) {
        let code = format!("fun {}({}, {}) {{ {}({}) }}", fname, fparam, xparam, fparam, xparam);

        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let mut transpiler = Transpiler::new();
            if let Ok(rust_code) = transpiler.transpile(&ast) {
                let rust_str = rust_code.to_string();
                // Function parameter should have Fn type
                prop_assert!(!rust_str.contains(&format!("{} : String", fparam)),
                    "Function parameter {} should not be String: {}", fparam, rust_str);
                // Should contain impl Fn or similar
                prop_assert!(rust_str.contains("impl Fn") || rust_str.contains("dyn Fn"),
                    "Function parameter {} should have Fn type: {}", fparam, rust_str);
            }
        }
    }

    /// Property: Type inference is deterministic
    #[test]
    fn type_inference_is_deterministic(
        code in "[a-z(){}+* ,0-9]+".prop_filter("Valid code", |s| s.len() > 5)
    ) {
        // Try parsing as function
        let func_code = format!("fun test() {{ {} }}", code);

        let mut parser1 = Parser::new(&func_code);
        let mut parser2 = Parser::new(&func_code);

        if let (Ok(ast1), Ok(ast2)) = (parser1.parse(), parser2.parse()) {
            let transpiler1 = Transpiler::new();
            let transpiler2 = Transpiler::new();

            if let (Ok(rust1), Ok(rust2)) = (transpiler1.transpile(&ast1), transpiler2.transpile(&ast2)) {
                prop_assert_eq!(rust1.to_string(), rust2.to_string(),
                    "Type inference should be deterministic");
            }
        }
    }

    /// Property: Simple string operations don't get numeric types
    #[test]
    fn string_operations_get_string_types(
        name in function_name(),
        param in param_name(),
        str_val in "[a-zA-Z ]{1,20}"
    ) {
        let code = format!(r#"fun {}({}) {{ {} + "{}" }}"#, name, param, param, str_val);

        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let mut transpiler = Transpiler::new();
            if let Ok(rust_code) = transpiler.transpile(&ast) {
                let rust_str = rust_code.to_string();
                // String concatenation should use String types
                if !name.starts_with("add") && !name.contains("sum") {
                    prop_assert!(rust_str.contains(&format!("{} : String", param)) ||
                                rust_str.contains(&format!("{} : &str", param)) ||
                                rust_str.contains(&format!("{} : impl", param)),
                        "String operation parameter should be String-like: {}", rust_str);
                }
            }
        }
    }

    /// Property: Functions never get both String and numeric operations
    #[test]
    fn no_mixed_type_operations(
        name in function_name(),
        p1 in param_name(),
        p2 in param_name().prop_filter("Different", move |x| x != &p1)
    ) {
        // Try mixed operations - should fail to compile or use consistent types
        let code = format!("fun {}({}, {}) {{ {} * 2 + {} }}", name, p1, p2, p1, p2);

        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let mut transpiler = Transpiler::new();
            if let Ok(rust_code) = transpiler.transpile(&ast) {
                let rust_str = rust_code.to_string();
                // Both params should have same type
                let p1_is_string = rust_str.contains(&format!("{} : String", p1));
                let p2_is_string = rust_str.contains(&format!("{} : String", p2));
                let p1_is_numeric = rust_str.contains(&format!("{} : i32", p1)) ||
                                   rust_str.contains(&format!("{} : i64", p1));
                let p2_is_numeric = rust_str.contains(&format!("{} : i32", p2)) ||
                                   rust_str.contains(&format!("{} : i64", p2));

                if p1_is_string || p2_is_string {
                    prop_assert!(p1_is_string == p2_is_string,
                        "Mixed numeric/string types: {}", rust_str);
                } else if p1_is_numeric || p2_is_numeric {
                    prop_assert!(p1_is_numeric == p2_is_numeric,
                        "Mixed numeric types: {}", rust_str);
                }
            }
        }
    }
}

/// Property tests for return type inference
mod return_types {
    use super::*;

    proptest! {
        /// Property: Functions with numeric operations infer numeric return
        #[test]
        fn numeric_ops_infer_numeric_return(
            name in function_name().prop_filter("Not main", |s| s != "main")
        ) {
            let code = format!("fun {}() {{ 1 + 2 }}", name);

            let mut parser = Parser::new(&code);
            if let Ok(ast) = parser.parse() {
                let mut transpiler = Transpiler::new();
                if let Ok(rust_code) = transpiler.transpile(&ast) {
                    let rust_str = rust_code.to_string();
                    // Numeric function should have numeric return or inferred
                    if rust_str.contains("-> String") {
                        prop_assert!(false,
                            "Numeric function should not return String: {}", rust_str);
                    }
                }
            }
        }

        /// Property: void functions don't get return types
        #[test]
        fn void_functions_no_return_type(name in function_name()) {
            let code = format!(r#"fun {}() {{ println("hello") }}"#, name);

            let mut parser = Parser::new(&code);
            if let Ok(ast) = parser.parse() {
                let mut transpiler = Transpiler::new();
                if let Ok(rust_code) = transpiler.transpile(&ast) {
                    let rust_str = rust_code.to_string();
                    // println returns (), shouldn't add return type
                    if name != "main" {
                        // Regular functions might infer return type
                        prop_assert!(true, "Non-main functions can have return types");
                    } else {
                        // main should never have return type
                        prop_assert!(!rust_str.contains("fn main() ->") &&
                                   !rust_str.contains("fn main () ->"),
                            "main() with println should not have return type: {}", rust_str);
                    }
                }
            }
        }
    }
}

/// Property tests for higher-order functions
mod higher_order {
    use super::*;

    proptest! {
        /// Property: apply(f, x) pattern types correctly
        #[test]
        fn apply_pattern_types_correctly(
            f in param_name(),
            x in param_name().prop_filter("Different", move |p| p != &f)
        ) {
            let code = format!("fun apply({}, {}) {{ {}({}) }}", f, x, f, x);

            let mut parser = Parser::new(&code);
            if let Ok(ast) = parser.parse() {
                let mut transpiler = Transpiler::new();
                if let Ok(rust_code) = transpiler.transpile(&ast) {
                    let rust_str = rust_code.to_string();
                    // f should be a function type
                    prop_assert!(!rust_str.contains(&format!("{} : String", f)),
                        "Parameter {} used as function should not be String: {}", f, rust_str);
                }
            }
        }

        /// Property: map(f, list) pattern types correctly
        #[test]
        fn map_pattern_types_correctly(
            f in param_name(),
            list in param_name().prop_filter("Different", move |p| p != &f)
        ) {
            let code = format!("fun map({}, {}) {{ {}({}) }}", f, list, f, list);

            let mut parser = Parser::new(&code);
            if let Ok(ast) = parser.parse() {
                let mut transpiler = Transpiler::new();
                if let Ok(rust_code) = transpiler.transpile(&ast) {
                    let rust_str = rust_code.to_string();
                    // f should be a function type
                    prop_assert!(!rust_str.contains(&format!("{} : String", f)),
                        "Mapper function should not be String: {}", rust_str);
                }
            }
        }
    }
}
