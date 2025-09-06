use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

fn main() {
    let code = r#"
mod math {
    pub fun add(a: i32, b: i32) -> i32 {
        a + b
    }
}

fun main() {
    let result = math::add(5, 3);
    println(result);
}
"#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    println!("AST has module: {}", format!("{:?}", ast).contains("Module"));
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    println!("Transpiled code:\n{}", rust_code);
}