use ruchy::frontend::parser::Parser;

fn main() {
    let source = r#"
        fun double(x) { x * 2 }
        double(21)
    "#;
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse");
    println!("AST: {:#?}", ast);
}
