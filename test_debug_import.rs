use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

fn main() {
    let normalize = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    
    let test_cases = vec![
        "import std::collections::HashMap",
        "import std::collections::{HashMap, HashSet}",
        "import std::collections::{HashMap as Map}",
        "import std::collections::*",
    ];
    
    for code in test_cases {
        println!("\nInput: {}", code);
        match Parser::new(code).parse() {
            Ok(ast) => {
                println!("Parsed: {:?}", ast.kind);
                let transpiler = Transpiler::new();
                match transpiler.transpile(&ast) {
                    Ok(rust_code) => {
                        println!("Raw output: '{}'", rust_code);
                        println!("Normalized: '{}'", normalize(&rust_code.to_string()));
                    }
                    Err(e) => println!("Transpile error: {}", e),
                }
            }
            Err(e) => println!("Parse error: {}", e),
        }
    }
}
