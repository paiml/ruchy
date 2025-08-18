use ruchy::frontend::parser::Parser;

fn main() {
    // Test without block wrapper
    let code1 = "import std::collections::{HashMap, HashSet}";
    match Parser::new(code1).parse() {
        Ok(expr) => println!("Code1 parsed successfully"),
        Err(e) => println!("Code1 error: {}", e),
    }
    
    // Test with block wrapper (what transpile_str does)
    let code2 = "{ import std::collections::{HashMap, HashSet} }";
    match Parser::new(code2).parse() {
        Ok(expr) => println!("Code2 parsed successfully"),
        Err(e) => println!("Code2 error: {}", e),
    }
}
