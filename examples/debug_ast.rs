use ruchy::Parser;

#[allow(clippy::print_stdout)]
fn main() {
    let code = r#"fun test() {
    return "hello";
}"#;
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
