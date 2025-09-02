use ruchy::Parser;

#[test]
fn test_object_items_parsing() {
    let input = r#"
let person = {"name" => "Alice", "age" => 30}
for (key, value) in person.items() {
    println(key + ": " + value)
}
"#;
    
    let mut parser = Parser::new(input);
    let result = parser.parse();
    
    println!("Parse result: {:?}", result);
    
    match result {
        Ok(ast) => println!("Successfully parsed: {:?}", ast),
        Err(e) => println!("Parse error: {}", e),
    }
}