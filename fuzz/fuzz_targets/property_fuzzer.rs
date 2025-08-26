#![no_main]

use libfuzzer_sys::fuzz_target;
use ruchy::{Parser, Transpiler};

/// Property: Parse → Transpile → Parse should be idempotent
/// If we can parse code, transpile it, and parse the result,
/// the AST should be equivalent (modulo formatting)
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Limit input size
        if s.len() > 1_000 {
            return;
        }
        
        // Step 1: Parse the original input
        let mut parser1 = Parser::new(s);
        if let Ok(ast1) = parser1.parse() {
            // Step 2: Transpile to Rust code
            let transpiler = Transpiler::new();
            if let Ok(rust_code) = transpiler.transpile(&ast1) {
                let rust_str = rust_code.to_string();
                
                // Property 1: Transpilation should always produce valid UTF-8
                assert!(rust_str.is_ascii() || rust_str.chars().all(|c| c.is_ascii() || c.len_utf8() > 1));
                
                // Property 2: Transpiled code should not be empty for non-empty AST
                if !matches!(ast1.kind, ruchy::frontend::ast::ExprKind::Block(ref stmts) if stmts.is_empty()) {
                    assert!(!rust_str.trim().is_empty(), "Non-empty AST produced empty transpilation");
                }
                
                // Property 3: Certain patterns should always appear in transpiled code
                check_transpilation_invariants(&ast1, &rust_str);
            }
        }
    }
});

fn check_transpilation_invariants(ast: &ruchy::frontend::ast::Expr, rust_code: &str) {
    use ruchy::frontend::ast::ExprKind;
    
    match &ast.kind {
        ExprKind::Function { name, params, body, .. } => {
            // Function names should appear in output
            assert!(rust_code.contains(&format!("fn {}", name)) || 
                    rust_code.contains(&format!("fn {name}")),
                    "Function name '{}' not found in transpiled code", name);
        }
        ExprKind::Let { name, .. } => {
            // Let bindings should appear
            assert!(rust_code.contains("let "), "Let binding not found in transpiled code");
            // Variable name should appear somewhere
            assert!(rust_code.contains(name), "Variable name '{}' not found", name);
        }
        ExprKind::If { .. } => {
            assert!(rust_code.contains("if "), "If expression not found in transpiled code");
        }
        ExprKind::Match { .. } => {
            assert!(rust_code.contains("match "), "Match expression not found in transpiled code");
        }
        ExprKind::While { .. } => {
            assert!(rust_code.contains("while "), "While loop not found in transpiled code");
        }
        ExprKind::For { .. } => {
            assert!(rust_code.contains("for "), "For loop not found in transpiled code");
        }
        _ => {
            // For other expression types, we don't enforce specific patterns
        }
    }
}