use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

fn main() {
    let code = "(raw_score as f64) / (max_score as f64) * 100.0";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast).expect("Should transpile");
    
    println!("Generated code:\n{}", result);
}
