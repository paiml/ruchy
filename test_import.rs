use ruchy::frontend::parser::Parser;

fn main() {
    let code = "import std::collections::{HashMap, HashSet}";
    match Parser::new(code).parse() {
        Ok(expr) => println!("Parsed successfully: {:?}", expr),
        Err(e) => println!("Parse error: {}", e),
    }
}
