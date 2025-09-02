use ruchy::{Parser, Transpiler};

#[test]
fn test_var_in_for_loop() {
    let input = r#"
for i in 0..5 {
    var sum = 0
    sum = sum + i
    println(sum)
}
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse var in for loop");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_var_destructuring_tuple() {
    let input = r#"
var (x, y) = (10, 20)
x = x + 5
y = y * 2
println(x + y)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse var with tuple destructuring");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    assert!(rust_code.contains("let mut"));
}
