use ruchy::frontend::parser::Parser;
fn main() {
    let input = "import df as a";
    let result = Parser::new(input).parse();
    match result {
        Ok(_) => println!("Parsed successfully"),
        Err(e) => println!("Parse error: {:?}", e),
    }
}
