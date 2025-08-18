use ruchy::frontend::parser::Parser;

fn main() {
    let code = "import std::collections::HashMap";
    println!("Parsing: {}", code);
    
    match Parser::new(code).parse() {
        Ok(expr) => {
            println!("Result: {:?}", expr.kind);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
