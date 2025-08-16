use ruchy::{Parser, Transpiler};

fn main() {
    let input = "let x = 10";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    println!("AST: {:?}", ast);
    
    let transpiler = Transpiler::new();
    let tokens = transpiler.transpile(&ast).unwrap();
    let code = tokens.to_string();
    println!("Transpiled: '{}'", code);
    println!("Ends with semicolon: {}", code.ends_with(';'));
}
