use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

fn main() {
    let code = "import std::collections::{HashMap as Map}";
    
    match Parser::new(code).parse() {
        Ok(ast) => {
            let transpiler = Transpiler::new();
            match transpiler.transpile(&ast) {
                Ok(rust_code) => {
                    println!("Transpiled: '{}'", rust_code);
                    println!("Expected contains: 'use std::collections::{{HashMap as Map}}'");
                    println!("Contains check: {}", rust_code.to_string().contains("use std::collections::{HashMap as Map}"));
                }
                Err(e) => println!("Transpile error: {}", e),
            }
        }
        Err(e) => println!("Parse error: {}", e),
    }
}
