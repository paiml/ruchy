#![no_main]

use libfuzzer_sys::fuzz_target;
use ruchy::parser::Parser;
use ruchy::transpiler::Transpiler;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Limit input size
        if s.len() > 5_000 {
            return;
        }
        
        // Try to parse
        let mut parser = Parser::new(s);
        
        // If parsing succeeds, try to transpile
        if let Ok(ast) = parser.parse_module() {
            let transpiler = Transpiler::new();
            let _ = transpiler.transpile_module(&ast);
        }
        
        // Also try expression transpilation
        let mut parser2 = Parser::new(s);
        if let Ok(expr) = parser2.parse_expression() {
            let transpiler = Transpiler::new();
            let _ = transpiler.transpile_expr(&expr);
        }
    }
});