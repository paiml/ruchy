use ruchy::Parser;

#[allow(clippy::print_stdout)]
fn main() {
    let code = r#"println("Result: {}", x)"#;
    let mut parser = Parser::new(code);
    match parser.parse() {
        Ok(ast) => {
            println!("AST: {ast:#?}");
        }
        Err(e) => {
            println!("Parse error: {e}");
        }
    }
}