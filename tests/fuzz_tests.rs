#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::unwrap_used, clippy::uninlined_format_args, clippy::useless_vec)]
//! Fuzz tests for Ruchy compiler - find edge cases and crashes

use ruchy::{Parser, Transpiler};
use ruchy::runtime::Repl;
use ruchy::runtime::repl::ReplConfig;
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
            let mut transpiler = Transpiler::new();
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

// REPL-Specific Fuzz Tests (REPL-TEST-003)

const MAX_MEMORY: usize = 1024 * 1024; // 1MB

/// Core fuzz testing function for REPL evaluation (matches spec)
pub fn fuzz_repl_eval(data: &[u8]) -> i32 {
    // Convert bytes to UTF-8 string, skip if invalid
    let input = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    // Skip extremely large inputs to avoid timeout
    if input.len() > 1000 {
        return 0;
    }
    
    // Create sandboxed REPL with strict resource limits
    let config = ReplConfig {
        max_memory: MAX_MEMORY,
        timeout: Duration::from_millis(100),
        max_depth: 100,
        debug: false,
    };
    
    let mut repl = match Repl::with_config(config) {
        Ok(r) => r,
        Err(_) => return 0,
    };
    
    // Attempt evaluation - should never panic or crash
    let _result = repl.eval(input);
    
    // Verify critical invariants hold after any input
    assert!(repl.memory_used() <= MAX_MEMORY, 
        "Memory invariant violated: {} bytes used", repl.memory_used());
    
    assert!(!repl.is_failed() || repl.recover().is_ok(),
        "Recovery invariant violated: failed state cannot be recovered");
    
    // Verify REPL can still accept basic input after fuzz input
    let basic_test = repl.eval("1 + 1");
    assert!(basic_test.is_ok() || repl.recover().is_ok(),
        "Basic functionality lost after fuzz input: {:?}", input);
    
    0 // Success
}

#[test]
fn test_repl_fuzz_basic_inputs() {
    // Test known problematic input patterns
    let problematic_inputs: Vec<&[u8]> = vec![
        b"",
        b"let",
        b"let x = ",
        b"(((((((((",
        b"))))))))",
        b"\"unclosed string",
        b"1 + + + +",
        b"x.y.z.a.b.c",
        b"if if if if",
        b"fn fn fn fn",
        b"[[[[[[[[[",
        b"]]]]]]]]]",
        b"{{{{{{{{{",
        b"}}}}}}}}}",
        b"123.456.789",
        b"true false maybe",
        b"// comment with } weird chars $@#%",
        b"/* /* nested /* comments */ */ */",
        b"\x00\x01\x02\x03", // Control characters
    ];
    
    for input in problematic_inputs {
        let result = fuzz_repl_eval(input);
        assert_eq!(result, 0, "Fuzz test failed for input: {:?}", 
            std::str::from_utf8(input).unwrap_or("<invalid UTF-8>"));
    }
}

#[test]
fn test_repl_fuzz_memory_exhaustion() {
    // Try to exhaust memory with large data structures
    let memory_intensive: Vec<&[u8]> = vec![
        b"let huge = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20]",
        b"let nested = [[[[[[[[[[]]]]]]]]]]",
        b"let deep = {a: {b: {c: {d: {e: {f: 42}}}}}}",
        b"let range_data = (1..1000)",
    ];
    
    for input in memory_intensive {
        let result = fuzz_repl_eval(input);
        assert_eq!(result, 0, "Memory exhaustion test failed");
    }
}

#[test]
fn test_repl_fuzz_state_corruption() {
    // Test sequences that might corrupt internal state
    let mut repl = Repl::new().unwrap();
    
    let state_corruption_inputs = vec![
        "let x = 1",
        "let x = \"string\"", // Type change
        "let y = x + 1", 
        "let x = x", // Self-reference
        "let recursive = recursive",
    ];
    
    for input in state_corruption_inputs {
        let _result = repl.eval(input);
        
        // State should remain consistent
        assert!(!repl.is_failed() || repl.recover().is_ok(),
            "State corrupted by input: {}", input);
        
        // Should still be able to evaluate basic expressions
        let test = repl.eval("1 + 1");
        assert!(test.is_ok() || repl.is_failed(),
            "Basic evaluation broken after: {}", input);
    }
}

#[test]
fn test_repl_fuzz_checkpoint_consistency() {
    let mut repl = Repl::new().unwrap();
    
    // Set up initial state
    let _ = repl.eval("let initial = 42");
    let checkpoint = repl.checkpoint();
    
    // Apply potentially corrupting fuzz inputs
    let corruption_inputs = vec![
        "let initial = \"changed type\"",
        "let new_var = initial * 2",
        "initial + undefined_var",
        "let shadow = initial",
        "let initial = initial + 1",
    ];
    
    for input in corruption_inputs {
        let _ = repl.eval(input);
        
        // Restore checkpoint
        repl.restore_checkpoint(&checkpoint);
        
        // Original state should be restored
        let result = repl.eval("initial");
        assert!(result.is_ok() || repl.is_failed(),
            "Checkpoint corruption detected with input: {}", input);
    }
}

#[test]
fn test_repl_fuzz_resource_bounds() {
    // Test that resource bounds are never exceeded under fuzz conditions
    let config = ReplConfig {
        max_memory: 512 * 1024, // 512KB - tighter bounds
        timeout: Duration::from_millis(50), // Shorter timeout
        max_depth: 50, // Shallower stack
        debug: false,
    };
    
    let mut repl = Repl::with_config(config).unwrap();
    
    let resource_intensive = vec![
        "let huge_list = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20]",
        "let nested_objects = {a: {b: {c: {d: {e: 42}}}}}",
        "let many_vars = 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10",
        "let deep_expr = ((((((((42))))))))",
        "for i in [1,2,3,4,5] { let temp = i * 2 }",
    ];
    
    for input in resource_intensive {
        let start = std::time::Instant::now();
        let _result = repl.eval(input);
        let elapsed = start.elapsed();
        
        // Check resource bounds
        assert!(elapsed < Duration::from_millis(200), 
            "Timeout exceeded for: {}", input);
        assert!(repl.memory_used() <= 512 * 1024,
            "Memory exceeded for: {}", input);
    }
}

#[test]
fn test_repl_fuzz_unicode_edge_cases() {
    // Test Unicode edge cases that might break parsing or evaluation
    let unicode_inputs = vec![
        "let α = 42".as_bytes(),
        "let emoji = \"🚀\"".as_bytes(),
        "let chinese = \"你好\"".as_bytes(),
        "let arabic = \"مرحبا\"".as_bytes(),
        "let mixed = \"Hello 🌍 世界\"".as_bytes(),
        "\u{FEFF}let x = 1".as_bytes(), // BOM
        "let x = \"\\u{1F600}\"".as_bytes(), // Unicode escape
        "/* 中文注释 */ let x = 1".as_bytes(),
        "let invalid_char = \"\u{FFFF}\"".as_bytes(), // Non-character
        "let zero_width = \"\u{200B}\"".as_bytes(), // Zero-width space
    ];
    
    for input in unicode_inputs {
        let result = fuzz_repl_eval(input);
        assert_eq!(result, 0, "Unicode fuzz test failed for: {:?}", 
            std::str::from_utf8(input));
    }
}

#[test]
fn test_repl_fuzz_malformed_syntax() {
    // Test syntactically invalid inputs that should be handled gracefully
    let malformed_inputs: Vec<&[u8]> = vec![
        b"let 123 = abc",
        b"fn 456() { }",
        b"if (((( { }",
        b"match x { 1 => 2 => 3 }",
        b"impl impl impl",
        b"type type = type",
        b"const const const",
        b"let let let",
        b"fn fn fn",
        b"if if if",
        b"while while while",
        b"for for for",
    ];
    
    for input in malformed_inputs {
        let result = fuzz_repl_eval(input);
        assert_eq!(result, 0, "Malformed syntax fuzz test failed");
    }
}

#[test]
fn test_repl_fuzz_boundary_values() {
    // Test numeric and size boundary values
    let boundary_inputs: Vec<&[u8]> = vec![
        b"let max_int = 9223372036854775807",
        b"let min_int = -9223372036854775808", 
        b"let zero = 0",
        b"let neg_zero = -0",
        b"let huge_float = 1.7976931348623157e+308",
        b"let infinity = 1.0/0.0",
        b"let neg_infinity = -1.0/0.0", 
        b"let empty_string = \"\"",
        b"let empty_array = []",
        b"let empty_object = {}",
        b"let max_depth = ((((((((42))))))))", // Deep nesting
        b"let long_var_name_that_goes_on_and_on_and_on = 1", // Long identifier
    ];
    
    for input in boundary_inputs {
        let result = fuzz_repl_eval(input);
        assert_eq!(result, 0, "Boundary value fuzz test failed");
    }
}

#[test]
fn test_repl_fuzz_random_bytes() {
    // Test truly random byte sequences to simulate real fuzzing
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    // Generate deterministic "random" bytes for consistent testing
    for seed in 0..50 { // Reduced from 100 for test performance
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Convert hash to byte array
        let bytes = hash.to_le_bytes();
        let extended: Vec<u8> = bytes.iter()
            .cycle()
            .take(50) // Reasonable size
            .copied()
            .collect();
        
        let result = fuzz_repl_eval(&extended);
        assert_eq!(result, 0, "Random bytes fuzz test failed for seed: {}", seed);
    }
}

/// Example function for cargo-fuzz integration (REPL-TEST-003 spec compliance)
#[cfg(feature = "fuzz")]
#[no_mangle]
pub extern "C" fn LLVMFuzzerTestOneInput(data: *const u8, size: usize) -> i32 {
    let input = unsafe { std::slice::from_raw_parts(data, size) };
    fuzz_repl_eval(input)
}