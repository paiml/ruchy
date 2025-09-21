use ruchy::{Parser, Transpiler};

#[test]
fn debug_string_coercion_output() {
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

    println!("Generated Rust code:");
    println!("{}", rust_code);

    println!(
        "Function signatures found: {:?}",
        transpiler.function_signatures
    );

    if rust_code.contains("greet(\"Alice\".to_string())") {
        println!("✅ String coercion working!");
    } else {
        println!("❌ String coercion NOT working");
    }
}
