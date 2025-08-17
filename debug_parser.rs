use ruchy::frontend::parser::Parser;

fn main() {
    println!("=== Testing parser precedence ===");
    
    // Test with single expression
    let mut parser = Parser::new("myactor");
    if let Ok(ast) = parser.parse() {
        println!("'myactor' -> {:?}", ast.kind);
    }
    
    // Test with question mark  
    let mut parser = Parser::new("myactor ?");
    if let Ok(ast) = parser.parse() {
        println!("'myactor ?' -> {:?}", ast.kind);
    }
    
    // Test with question + identifier
    let mut parser = Parser::new("myactor ? request");
    if let Ok(ast) = parser.parse() {
        println!("'myactor ? request' -> {:?}", ast.kind);
    }
}