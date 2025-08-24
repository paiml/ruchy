#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::unwrap_used, clippy::uninlined_format_args, clippy::useless_vec)]
//! Fuzz tests for Ruchy compiler - find edge cases and crashes

use ruchy::{Parser, Transpiler};
use ruchy::runtime::Repl;
use std::time::{Duration, Instant};

/// Generate random valid Ruchy code
fn generate_random_code(seed: usize) -> String {
    let operations = vec![
        "+", "-", "*", "/", "%", 
        "==", "!=", "<", ">", "<=", ">=",
        "&&", "||"
    ];
    
    let values = vec![
        "1", "2", "42", "0", "-1",
        "true", "false",
        r#""hello""#, r#""world""#,
        "[1, 2, 3]", "[]",
        "(1, 2)", "()",
    ];
    
    let templates = vec![
        "VAL OP VAL",
        "let x = VAL",
        "if VAL { VAL } else { VAL }",
        "match VAL { _ => VAL }",
        "{ VAL }",
        "[VAL, VAL, VAL]",
        "fun f() -> i32 { VAL }",
        "for x in [VAL] { VAL }",
        "while false { VAL }",
    ];
    
    // Use seed to pick template and values
    let template = &templates[seed % templates.len()];
    let val = values[seed % values.len()];
    let op = operations[seed % operations.len()];
    
    template
        .replace("VAL", val)
        .replace("OP", op)
}

/// Fuzz test: Parser should never panic
#[test]
fn fuzz_parser_no_panic() {
    for seed in 0..1000 {
        let code = generate_random_code(seed);
        let mut parser = Parser::new(&code);
        let _ = parser.parse(); // Should not panic
    }
}

/// Fuzz test: Transpiler should handle any valid AST
#[test]
fn fuzz_transpiler_valid_ast() {
    for seed in 0..1000 {
        let code = generate_random_code(seed);
        let mut parser = Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let _ = transpiler.transpile(&ast); // Should not panic
        }
    }
}

/// Fuzz test: REPL should handle any input with timeout
#[test]
fn fuzz_repl_timeout() {
    for seed in 0..100 {
        let code = generate_random_code(seed);
        let mut repl = Repl::new().unwrap();
        let deadline = Some(Instant::now() + Duration::from_millis(100));
        let _ = repl.evaluate_expr_str(&code, deadline); // Should not hang
    }
}

/// Fuzz test: String escaping edge cases
#[test]
fn fuzz_string_escaping() {
    let edge_cases = vec![
        r#""""#,
        r#""\""#,
        r#""\n""#,
        r#""\r""#,
        r#""\t""#,
        r#""\0""#,
        r#""\\""#,
        r#""\x41""#,
        r#""\u{1F600}""#,
        r#""'"'"'"#,
        r#""```""#,
        r#""${}"#,
        r#""{{}}""#,
    ];
    
    for case in edge_cases {
        let code = format!("let x = {}", case);
        let mut parser = Parser::new(&code);
        let _ = parser.parse(); // Should handle gracefully
    }
}

/// Fuzz test: Number edge cases
#[test]
fn fuzz_number_parsing() {
    let numbers = vec![
        "0", "-0", "+0",
        "1", "-1", "+1",
        "999999999999999999999999999999",
        "-999999999999999999999999999999",
        "0.0", "0.000000001", "1e10", "1e-10",
        "1_000_000", "0b1010", "0o755", "0xFF",
        "NaN", "Infinity", "-Infinity",
    ];
    
    for num in numbers {
        let code = format!("let x = {}", num);
        let mut parser = Parser::new(&code);
        let _ = parser.parse(); // Should handle gracefully
    }
}

/// Fuzz test: Deeply nested structures
#[test]
fn fuzz_deep_nesting() {
    // Test deeply nested blocks
    let mut code = String::new();
    for _ in 0..100 {
        code.push('{');
    }
    code.push_str("42");
    for _ in 0..100 {
        code.push('}');
    }
    
    let mut parser = Parser::new(&code);
    let _ = parser.parse(); // Should handle deep nesting
    
    // Test deeply nested lists
    let mut list_code = String::new();
    for _ in 0..50 {
        list_code.push('[');
    }
    list_code.push('1');
    for _ in 0..50 {
        list_code.push(']');
    }
    
    let mut parser2 = Parser::new(&list_code);
    let _ = parser2.parse(); // Should handle deep nesting
}

/// Fuzz test: Unicode handling
#[test]
fn fuzz_unicode() {
    let unicode_strings = vec![
        "let x = \"Hello, 世界\"",
        "let ℵ = 42",
        "let 🦀 = \"rust\"",
        "let x = \"\\u{1F600}\"",
        "let x = \"א״ב״ג״\"",
        "let x = \"🏳️‍🌈\"",
        "let x = \"\\u{0}\\u{FFFF}\"",
    ];
    
    for code in unicode_strings {
        let mut parser = Parser::new(code);
        let _ = parser.parse(); // Should handle Unicode
    }
}

/// Fuzz test: Operator precedence combinations
#[test]
fn fuzz_operator_precedence() {
    let expressions = vec![
        "1 + 2 * 3",
        "1 * 2 + 3",
        "1 + 2 + 3 + 4",
        "1 * 2 * 3 * 4",
        "1 + 2 * 3 - 4 / 5",
        "1 == 2 && 3 < 4 || 5 > 6",
        "!true && false || true",
        "1 < 2 == 3 > 4",
    ];
    
    for expr in expressions {
        let mut repl = Repl::new().unwrap();
        let _ = repl.eval(expr); // Should evaluate correctly
    }
}

/// Fuzz test: Invalid syntax recovery
#[test]
fn fuzz_syntax_errors() {
    let invalid = vec![
        "let",
        "let x",
        "let x =",
        "if",
        "if true",
        "if true {",
        "fun",
        "fun f",
        "fun f(",
        "fun f()",
        "match",
        "match x",
        "match x {",
        "[",
        "[1",
        "[1,",
        "{",
        "{x",
        "{x:",
        "(",
        "(1",
        "(1,",
    ];
    
    for code in invalid {
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_err(), "Should reject invalid syntax: {}", code);
    }
}

/// Fuzz test: Memory-intensive operations
#[test]
fn fuzz_memory_limits() {
    let mut repl = Repl::new().unwrap();
    
    // Large list
    let large_list = format!("[{}]", (0..1000).map(|i| i.to_string()).collect::<Vec<_>>().join(", "));
    let _ = repl.eval(&large_list); // Should handle large lists
    
    // Long string
    let long_string = format!(r#""{}""#, "x".repeat(10000));
    let _ = repl.eval(&format!("let x = {}", long_string)); // Should handle long strings
    
    // Many variables
    for i in 0..1000 {
        let _ = repl.eval(&format!("let var_{} = {}", i, i));
    }
}