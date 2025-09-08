#[cfg(test)]
mod dataframe_parsing_tdd {
    use std::time::Instant;
    
    #[test]
    fn test_df_empty_brackets_should_parse() {
        // RED: This test should fail initially
        let mut repl = ruchy::runtime::Repl::new().expect("Failed to create REPL");
        let deadline = Instant::now() + std::time::Duration::from_secs(1);
        
        // Test that df![] parses and executes without error
        let result = repl.eval("df![]");
        
        assert!(result.is_ok(), 
               "df![] should parse and execute successfully, but got error: {:?}", 
               result.err());
    }
    
    #[test]
    fn test_df_empty_in_assignment() {
        // RED: This test should fail initially  
        let mut repl = ruchy::runtime::Repl::new().expect("Failed to create REPL");
        
        // Test that df![] can be assigned to a variable
        let result = repl.eval("let df = df![]; df");
        
        assert!(result.is_ok(), 
               "let df = df![]; should work, but got error: {:?}", 
               result.err());
    }
}