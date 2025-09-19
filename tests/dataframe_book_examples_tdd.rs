#[cfg(test)]
mod dataframe_book_examples_tdd {
    use ruchy::runtime::Repl;
    
    #[test]
    fn test_dataframe_new_constructor() {
        // RED: Test that DataFrame::new() works as shown in book Ch18
        let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
        
        // Book example from Ch18-00-dataframes-data-processing.md
        let code = r#"
            let df = DataFrame::new()
                .column("id", [1, 2, 3])
                .column("name", ["Alice", "Bob", "Charlie"])
                .build();
            df
        "#;
        
        let result = repl.eval(code);
        
        // RED: This should fail because DataFrame::new() doesn't work yet
        assert!(result.is_ok(), "DataFrame::new() should work, got error: {:?}", result);
        
        // Verify it returns a DataFrame value
        let output = result.unwrap();
        assert!(output.contains("DataFrame") || output.contains("column"), 
            "Should return DataFrame, got: {}", output);
    }
    
    #[test]
    fn test_dataframe_rows_method() {
        // RED: Test that df.rows() works as shown in book
        let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
        
        let code = r#"
            let df = DataFrame::new()
                .column("id", [1, 2, 3])
                .build();
            df.rows()
        "#;
        
        let result = repl.eval(code);
        
        // RED: This should fail because .rows() method doesn't exist
        assert!(result.is_ok(), "df.rows() should work, got error: {:?}", result);
        
        // Should return 3
        let output = result.unwrap();
        assert!(output.contains("3"), "df.rows() should return 3, got: {}", output);
    }
    
    #[test]
    fn test_dataframe_columns_method() {
        // RED: Test that df.columns() works as shown in book
        let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
        
        let code = r#"
            let df = DataFrame::new()
                .column("id", [1, 2])
                .column("name", ["A", "B"])
                .build();
            df.columns()
        "#;
        
        let result = repl.eval(code);
        
        // RED: This should fail because .columns() method doesn't exist
        assert!(result.is_ok(), "df.columns() should work, got error: {:?}", result);
        
        // Should return 2
        let output = result.unwrap();
        assert!(output.contains("2"), "df.columns() should return 2, got: {}", output);
    }
    
    #[test]
    fn test_dataframe_display() {
        // RED: Test that println(df) displays DataFrame nicely
        let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
        
        let code = r#"
            let df = DataFrame::new()
                .column("product", ["Widget", "Gadget"])
                .column("price", [10, 20])
                .build();
            df
        "#;
        
        let result = repl.eval(code);
        
        // RED: This should fail because DataFrame display doesn't work
        assert!(result.is_ok(), "DataFrame creation should work, got error: {:?}", result);
        
        let output = result.unwrap();
        
        // Should display as a table
        assert!(output.contains("product") || output.contains("DataFrame") || output.contains("Widget"), 
            "DataFrame should display nicely, got: {}", output);
    }
}