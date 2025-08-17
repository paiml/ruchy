#![allow(clippy::uninlined_format_args, clippy::print_stdout)]
use ruchy::{Parser, Transpiler};

fn main() {
    let inputs = vec!["let x = 10", "x", "let y = 20", "x + y"];

    for input in inputs {
        println!("\n=== Testing: {} ===", input);
        let mut parser = Parser::new(input);
        match parser.parse() {
            Ok(ast) => {
                println!("AST kind: {:?}", ast.kind);

                let transpiler = Transpiler::new();
                match transpiler.transpile(&ast) {
                    Ok(tokens) => {
                        let code = tokens.to_string();
                        println!("Transpiled: '{}'", code);
                        println!("Length: {}", code.len());
                        println!("Ends with semicolon: {}", code.ends_with(';'));

                        // Check if it's a let statement
                        if matches!(ast.kind, ruchy::ExprKind::Let { .. }) {
                            println!("This is a let statement, needs semicolon!");
                            if !code.ends_with(';') {
                                let fixed = format!("{};", code);
                                println!("Fixed: '{}'", fixed);
                            }
                        }
                    }
                    Err(e) => println!("Transpile error: {}", e),
                }
            }
            Err(e) => println!("Parse error: {}", e),
        }
    }
}
