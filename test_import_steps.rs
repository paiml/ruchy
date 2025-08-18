fn main() {
    use ruchy::frontend::parser::Parser;
    
    let code = "import std";
    match Parser::new(code).parse() {
        Ok(_) => println!("Simple import works"),
        Err(e) => println!("Simple import error: {}", e),
    }
    
    let code = "import std::collections";
    match Parser::new(code).parse() {
        Ok(_) => println!("Path import works"),
        Err(e) => println!("Path import error: {}", e),
    }
    
    let code = "import std::collections::HashMap";
    match Parser::new(code).parse() {
        Ok(_) => println!("Full path import works"),
        Err(e) => println!("Full path import error: {}", e),
    }
    
    let code = "import std::collections::*";
    match Parser::new(code).parse() {
        Ok(_) => println!("Wildcard import works"),
        Err(e) => println!("Wildcard import error: {}", e),
    }
    
    let code = "import std::collections::{HashMap}";
    match Parser::new(code).parse() {
        Ok(_) => println!("Single item brace import works"),
        Err(e) => println!("Single item brace import error: {}", e),
    }
    
    let code = "import std::collections::{HashMap, HashSet}";
    match Parser::new(code).parse() {
        Ok(_) => println!("Multi item brace import works"),
        Err(e) => println!("Multi item brace import error: {}", e),
    }
}
