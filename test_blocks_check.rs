use ruchy::{Parser, Transpiler};

fn main() {
    let code = std::fs::read_to_string("examples/test_blocks.ruchy").unwrap();
    let mut parser = Parser::new(&code);
    let ast = parser.parse().unwrap();
    println!("AST kind: {:?}", std::mem::discriminant(&ast.kind));
    
    let transpiler = Transpiler::new();
    match transpiler.transpile_to_program(&ast) {
        Ok(rust_code) => {
            std::fs::write("/tmp/test_blocks_output.rs", &rust_code.to_string()).unwrap();
            println!("Transpiled successfully");
        }
        Err(e) => println!("Error: {}", e)
    }
}
