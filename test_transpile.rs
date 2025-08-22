use ruchy::{Parser, Transpiler};

fn main() {
    let code = r#"let greeting = "Hello, Ruchy!" in greeting"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap();
    println!("{}", rust_code);
}
