use ruchy::{Parser, Transpiler};

fn main() {
    let code = r#"let greeting = "Hello, World!" in greeting"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap();
    std::fs::write("/tmp/test_hw_output.rs", &rust_code.to_string()).unwrap();
}
