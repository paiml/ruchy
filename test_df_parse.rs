fn main() {
    let input = "df![col => [1, 2, 3]]";
    println!("Parsing: {}", input);
    
    let mut parser = ruchy::Parser::new(input);
    match parser.parse() {
        Ok(ast) => println!("Success! AST: {:#?}", ast),
        Err(e) => println!("Parse error: {}", e),
    }
}