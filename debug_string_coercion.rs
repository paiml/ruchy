use ruchy::{Parser, Transpiler};

fn main() {
    let input = r#"
fn greet(name: String) {
    println("Hello, " + name)
}
greet("Alice")
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    println!("Generated code: {}", rust_code);
    
    if rust_code.contains("greet(\"Alice\".to_string())") {
        println!("✅ String coercion working correctly!");
    } else if rust_code.contains("greet(String::from(\"Alice\"))") {
        println!("✅ String coercion working correctly with String::from!");
    } else {
        println!("❌ String coercion NOT working. Raw output:");
        println!("{}", rust_code);
    }
}
