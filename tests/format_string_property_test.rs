#![allow(clippy::print_stdout)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::needless_raw_string_hashes)]

use proptest::prelude::*;
use ruchy::{Parser, Transpiler};

#[test]
fn test_println_format_string_simple() {
    let mut transpiler = Transpiler::new();
    let mut parser = Parser::new(r#"println("Hello, {}", "world")"#);
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    
    println!("Generated code: {}", result);
    
    // The fix should generate: println!("Hello, {}", "world")
    // NOT: println!("{} {}", format!("Hello: {{}}"), "world")
    assert!(!result.contains("format!"));
    assert!(result.contains(r#"println ! ("Hello, {}" , "world")"#));
}

#[test]
fn test_println_simple_string() {
    let mut transpiler = Transpiler::new();
    let mut parser = Parser::new(r#"println("Hello world")"#);
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    
    println!("Generated code: {}", result);
    assert!(result.contains(r#"println ! ( "Hello world" )"#));
}

#[test]
fn test_regular_function_string_conversion() {
    let mut transpiler = Transpiler::new();
    let mut parser = Parser::new(r#"some_func("test")"#);
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    
    println!("Generated code: {}", result);
    // Regular functions should convert strings to String
    assert!(result.contains(r#""test" . to_string"#));
}

proptest! {
    #[test]
    fn prop_println_never_double_formats(
        format_str in r#"[a-zA-Z0-9 {},!.?-]+"#,
        arg_count in 1..5usize
    ) {
        let args = (0..arg_count).map(|i| format!("arg{}", i)).collect::<Vec<_>>().join(", ");
        let code = format!(r#"println("{}", {})"#, format_str, args);
        
        let mut transpiler = Transpiler::new();
        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            if let Ok(tokens) = transpiler.transpile(&ast) {
                let result = tokens.to_string();
                // Should never have nested format! calls
                prop_assert!(!result.contains("format ! ( "));
                prop_assert!(!result.contains("format!("));
            }
        }
    }
    
    #[test]
    fn prop_macro_vs_function_distinction(
        name in "[a-z]+",
        arg in r#"[a-zA-Z0-9 ]+"#
    ) {
        let code = format!(r#"{}("{}")"#, name, arg);
        let mut transpiler = Transpiler::new();
        
        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            if let Ok(tokens) = transpiler.transpile(&ast) {
                let result = tokens.to_string();
                if name == "println" || name == "print" || name == "dbg" || name == "panic" {
                    // Macros should use ! syntax
                    let expected = format!("{} !", name);
                    prop_assert!(result.contains(&expected));
                } else {
                    // Functions should convert strings
                    prop_assert!(result.contains(". to_string"));
                }
            }
        }
    }
}