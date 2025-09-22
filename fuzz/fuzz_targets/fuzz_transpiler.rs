#![no_main]

use libfuzzer_sys::fuzz_target;
use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        let mut parser = Parser::new(input);
        if let Ok(expr) = parser.parse() {
            // If parsing succeeds, transpilation should not panic
            let transpiler = Transpiler::new();
            let _ = transpiler.transpile_expr(&expr);
        }
    }
});
