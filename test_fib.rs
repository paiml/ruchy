use ruchy::Parser;

fn main() {
    let code = std::fs::read_to_string("examples/fibonacci.ruchy").unwrap();
    let mut parser = Parser::new(&code);
    let ast = parser.parse().unwrap();
    println!("{:#?}", ast);
}
