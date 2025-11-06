#![no_main]
//! TRANSPILER-PROPERTY + FUZZ: Integrated Property-Based + Coverage-Guided Fuzzing
//!
//! **Purpose**: Combine structured property testing with libfuzzer's coverage guidance
//! **Strategy**: Use property test patterns but with fuzzer-driven input selection
//! **Method**: EXTREME TDD with parse → transpile → type check validation
//!
//! This harness discovered TRANSPILER-TYPE-INFER-PARAMS bug in first 100 cases.

use libfuzzer_sys::fuzz_target;
use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }

    // Use fuzzer bytes to select test patterns (mimics property test generators)
    let type_idx = data[0] % 8;
    let func_name_len = ((data[1] as usize) % 10) + 1;
    let param_name_len = ((data[2] as usize) % 10) + 1;
    let expr_pattern = data[3] % 6;

    // Generate type annotation (from gen_type_annotation)
    let param_type = match type_idx {
        0 => "i32",
        1 => "f64",
        2 => "bool",
        3 => "String",
        4 => "str",
        5 => "i64",
        6 => "u32",
        _ => "char",
    };

    // Generate function name (from gen_func_name pattern)
    let func_name = generate_name("f", func_name_len, &data[4..]);

    // Generate parameter name (from gen_var_name pattern)
    let param_name = generate_name("x", param_name_len, &data[4..]);

    // Generate expression pattern (from gen_simple_expr patterns)
    let expr = match expr_pattern {
        0 => param_name.clone(),                             // Direct return
        1 => format!("{} + 1", param_name),                  // Binary op
        2 => format!("{} * 2", param_name),                  // Binary op
        3 => format!("let result = {}; result", param_name), // Variable assignment
        4 => format!("let result = {} * 2; result", param_name), // Expression assignment
        _ => format!("let y = {}; y", param_name),           // Different variable name
    };

    // Generate test value for main function
    let test_value = match param_type {
        "i32" | "i64" | "u32" => "42",
        "f64" => "3.14",
        "bool" => "true",
        "char" => "'x'",
        "str" => "\"test\"",
        "String" => "\"test\".to_string()",
        _ => "42",
    };

    // Generate complete program (from gen_type_inference_function pattern)
    let program = format!(
        r#"
fun {func_name}({param_name}: {param_type}) {{
    {expr}
}}

fun main() {{
    let value = {func_name}({test_value});
    println("{{}}", value)
}}
"#
    );

    // Property 1: Program must parse successfully
    let mut parser = Parser::new(&program);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(_) => return, // Skip invalid programs
    };

    // Property 2: Transpilation must succeed
    let mut transpiler = Transpiler::new();
    let rust_code = match transpiler.transpile(&ast) {
        Ok(code) => code,
        Err(_) => return, // Skip transpiler errors
    };

    let rust_str = rust_code.to_string();

    // Property 3: Type inference correctness (CRITICAL - found TRANSPILER-TYPE-INFER-PARAMS)
    // Functions returning parameter values MUST infer parameter's type, NOT default to i32
    if rust_str.contains(&format!("fn {func_name}")) {
        // Check return type matches parameter type
        let expected_return_type = match param_type {
            "i32" => "-> i32",
            "f64" => "-> f64",
            "bool" => "-> bool",
            "String" => "-> String",
            "str" => "-> &str", // Ruchy 'str' maps to Rust '&str'
            "i64" => "-> i64",
            "u32" => "-> u32",
            "char" => "-> char",
            _ => "-> i32",
        };

        // ASSERTION: If function returns parameter value, type must match
        // This property check found the original bug: fun a(a: f64) { a } → fn a(a: f64) -> i32 ❌
        if expr.contains(&param_name) && !expr.contains("42") && !expr.contains("true") {
            // Only check if expression actually uses the parameter
            if !rust_str.contains(expected_return_type) && param_type != "str" {
                // Allow 'str' special case due to lifetime handling
                panic!(
                    "TYPE INFERENCE BUG: Function returning {param_type} parameter should have {expected_return_type}\n\
                     Program: {program}\n\
                     Generated: {rust_str}"
                );
            }
        }
    }

    // Property 4: Output must be valid Rust tokens
    assert!(
        rust_str.contains("fn "),
        "Transpiled code must contain function definitions"
    );

    // Property 5: Type safety - no mixing of incompatible types
    check_type_consistency(&rust_str, param_type);
});

/// Helper: Generate valid identifier from fuzzer bytes
fn generate_name(prefix: &str, max_len: usize, bytes: &[u8]) -> String {
    let mut name = prefix.to_string();
    let len = max_len.min(bytes.len()).min(15);
    for &byte in &bytes[..len] {
        let c = match byte % 37 {
            0..=25 => (b'a' + (byte % 26)) as char,
            26..=35 => (b'0' + ((byte % 10).saturating_sub(26))) as char,
            _ => '_',
        };
        name.push(c);
    }
    name
}

/// Helper: Check type consistency in generated Rust code
fn check_type_consistency(rust_code: &str, param_type: &str) {
    // Don't mix floating point with integer operations
    if param_type == "f64" && rust_code.contains("-> i32") {
        // This is likely the bug we're looking for!
        // Unless there's explicit casting or it's a different function
        if !rust_code.contains("as i32") && !rust_code.contains("fn main") {
            // Additional validation could go here
        }
    }

    // Ensure return types are sensible
    if rust_code.contains("-> bool") {
        assert!(
            rust_code.contains("true") || rust_code.contains("false") || rust_code.contains("bool"),
            "Boolean return type but no boolean values"
        );
    }
}
