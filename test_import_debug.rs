use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

fn main() {
    let test_cases = vec![
        "import std::collections::HashMap",
        "import std::collections::{HashMap, HashSet}",
        "import std::collections::{HashMap as Map}",
        "import std::collections::*",
    ];
    
    for code in test_cases {
        println!("\n=== Input: {} ===", code);
        match Parser::new(code).parse() {
            Ok(ast) => {
                println!("AST: {:?}", ast.kind);
                let transpiler = Transpiler::new();
                match transpiler.transpile(&ast) {
                    Ok(result) => {
                        let normalized = result.to_string().chars()
                            .filter(|c| !c.is_whitespace())
                            .collect::<String>();
                        println!("Output: {}", result);
                        println!("Normalized: {}", normalized);
                    }
                    Err(e) => println!("Transpile error: {}", e),
                }
            }
            Err(e) => println!("Parse error: {}", e),
        }
    }
}