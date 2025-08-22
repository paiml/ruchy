use ruchy::{Parser, Transpiler};

fn main() {
    let code = std::fs::read_to_string("examples/fibonacci.ruchy").unwrap();
    let mut parser = Parser::new(&code);
    let ast = parser.parse().unwrap();
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast).unwrap();
    std::fs::write("/tmp/fib_output.rs", rust_code.to_string()).unwrap();
}
