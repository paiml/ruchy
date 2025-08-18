use ruchy::frontend::parser::Parser;

fn main() {
    let code = "import std::collections::{HashMap as Map}";
    println!("Parsing: {}", code);
    
    let mut parser = Parser::new(code);
    match parser.parse() {
        Ok(expr) => {
            println!("Success! Parsed expression: {:?}", expr.kind);
        }
        Err(e) => {
            println!("Error: {}", e);
            println!("Error details: {:?}", e);
        }
    }
}
