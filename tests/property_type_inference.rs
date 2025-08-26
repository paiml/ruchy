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
        !matches!(s.as_str(), "if" | "else" | "while" | "for" | "let" | "fun" | 
                  "return" | "break" | "continue" | "match" | "struct" | "enum" |
                  "impl" | "trait" | "type" | "const" | "static" | "mut" | "fn" | "main")
    })
}

/// Generate valid parameter names  
fn param_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,10}".prop_filter("Not a reserved word", |s| {
        !matches!(s.as_str(), "if" | "else" | "while" | "for" | "let" | "fun" | 
                  "return" | "break" | "continue" | "match" | "struct" | "enum" |
                  "impl" | "trait" | "type" | "const" | "static" | "mut" | "fn")
    })
}

proptest! {
    /// Property: main() function NEVER generates with a return type annotation
    #[test]
    fn test_main_never_has_return_type(body in "[a-z0-9+* ]+") {
        let code = format!("fun main() {{ {} }}", body);
        
        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            if let Ok(rust_code) = transpiler.transpile(&ast) {
                let rust_str = rust_code.to_string();
                // main should never have -> return type annotation
                prop_assert!(!rust_str.contains("fn main () ->"), 
                    "main() should not have return type annotation, got: {}", rust_str);
                prop_assert!(!rust_str.contains("fn main() ->"),
                    "main() should not have return type annotation, got: {}", rust_str);
            }
        }
    }

    /// Property: Function parameters used as functions should NOT be typed as String
    #[test]
    fn test_function_params_not_string(
        fname in function_name(),
        (fparam, xparam) in (param_name(), param_name())
            .prop_filter("Different params", |(f, x)| f != x),
    ) {
        // Generate code where fparam is used as a function
        let code = format!("fun {}({}, {}) {{ {}({}) }}", fname, fparam, xparam, fparam, xparam);
        
        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            if let Ok(rust_code) = transpiler.transpile(&ast) {
                let rust_str = rust_code.to_string();
                // Function parameter should NOT be typed as String
                prop_assert!(!rust_str.contains(&format!("{} : String", fparam)),
                    "Function parameter {} should not be typed as String when used as function. Got: {}", 
                    fparam, rust_str);
            }
        }
    }

    /// Property: Type inference is deterministic - same input produces same output
    #[test]
    fn test_type_inference_deterministic(
        fname in function_name(),
        param in param_name(),
        n in 0i32..100i32
    ) {
        let code = format!("fun {}({}) {{ {} + {} }}", fname, param, param, n);
        
        // Parse and transpile twice
        let mut parser1 = Parser::new(&code);
        let mut parser2 = Parser::new(&code);
        
        if let (Ok(ast1), Ok(ast2)) = (parser1.parse(), parser2.parse()) {
            let transpiler1 = Transpiler::new();
            let transpiler2 = Transpiler::new();
            
            if let (Ok(rust1), Ok(rust2)) = (transpiler1.transpile(&ast1), transpiler2.transpile(&ast2)) {
                prop_assert_eq!(rust1.to_string(), rust2.to_string(),
                    "Type inference should be deterministic for same input");
            }
        }
    }
}

/// Specific regression tests as properties
mod regression_properties {
    use super::*;
    
    proptest! {
        /// BUG-002 Regression: Higher-order functions must work
        #[test]
        fn test_higher_order_apply_pattern(
            (f, x) in (param_name(), param_name())
                .prop_filter("Different params", |(f, x)| f != x)
        ) {
            // The apply(f, x) pattern
            let code = format!("fun apply({}, {}) {{ {}({}) }}", f, x, f, x);
            
            let mut parser = Parser::new(&code);
            prop_assume!(parser.parse().is_ok());
            let ast = parser.parse().unwrap();
            
            let transpiler = Transpiler::new();
            let result = transpiler.transpile(&ast);
            
            // Should transpile successfully
            prop_assert!(result.is_ok(), 
                "Higher-order function should transpile: {:?}", result);
            
            let rust_str = result.unwrap().to_string();
            // f should NOT be String
            prop_assert!(!rust_str.contains(&format!("{} : String", f)),
                "Function parameter should not be String in apply pattern");
        }
        
        /// String parameters should still work when not used as functions
        #[test]
        fn test_string_params_work(
            fname in function_name(),
            param in param_name(),
        ) {
            // Function that uses parameter as string
            let code = format!(r#"fun {}({}) {{ println({}) }}"#, fname, param, param);
            
            let mut parser = Parser::new(&code);
            if let Ok(ast) = parser.parse() {
                let transpiler = Transpiler::new();
                let result = transpiler.transpile(&ast);
                
                // Should transpile successfully
                prop_assert!(result.is_ok(),
                    "String parameter function should transpile: {:?}", result);
            }
        }
    }
}