#[cfg(test)]
mod test_transpiler_method {
    use crate::frontend::parser::Parser;
    use crate::backend::transpiler::Transpiler;
    
    #[test]
    fn test_transpile_to_program_with_test_attr() {
        let input = r#"
#[test]
fun test_simple() {
    assert_eq!(2, 2)
}
"#;
        let mut parser = Parser::new(input);
        let expr = parser.parse().expect("Parse failed");
        
        println!("Parsed expr attributes: {:?}", expr.attributes);
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_to_program(&expr).expect("Transpile failed");
        
        println!("Generated tokens: {}", result);
        
        // Check that test functions don't have return types
        let code_str = result.to_string();
        assert!(!code_str.contains("-> i32"), "Test function should not have return type, got: {}", code_str);
        assert!(code_str.contains("#[test]"), "Should preserve test attribute");
    }
}