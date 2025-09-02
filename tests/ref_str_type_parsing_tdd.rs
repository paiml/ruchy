use ruchy::Parser;

#[test]
fn test_parse_ref_str_type_annotation() {
    let input = r#"fn greet(name: &str) { println(name) }"#;
    let mut parser = Parser::new(input);
    let result = parser.parse();
    
    println!("Parse result: {:?}", result);
    
    // This test will show us if &str is being parsed correctly
    assert!(result.is_ok(), "Failed to parse &str type annotation: {:?}", result);
}

#[test]
fn test_parse_string_type_annotation() {
    let input = r#"fn greet(name: String) { println(name) }"#;
    let mut parser = Parser::new(input);
    let result = parser.parse();
    
    println!("Parse result: {:?}", result);
    
    assert!(result.is_ok(), "Failed to parse String type annotation");
}