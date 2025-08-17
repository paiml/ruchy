use ruchy::frontend::parser::Parser;

fn main() {
    println!("=== Parsing 'myactor ! message' ===");
    let mut parser = Parser::new("myactor ! message");
    match parser.parse() {
        Ok(ast) => println!("AST: {:#?}", ast),
        Err(e) => println!("Parse error: {}", e),
    }
    
    println!("\n=== Parsing 'myactor ? request' ===");
    let mut parser = Parser::new("myactor ? request");
    match parser.parse() {
        Ok(ast) => println!("AST: {:#?}", ast),
        Err(e) => println!("Parse error: {}", e),
    }
}