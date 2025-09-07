/// TDD Tests for DataFrame REPL Implementation
/// 
/// These tests drive the implementation of DataFrame support in the REPL
/// Following strict TDD: Red -> Green -> Refactor

#[cfg(test)]
mod dataframe_repl_tests {
    use ruchy::runtime::repl::Repl;

    #[test]
    fn test_dataframe_constructor_exists() {
        // Simplest test - DataFrame::new() should be recognized
        let mut repl = Repl::new().unwrap();
        let result = repl.eval("DataFrame::new()");
        // Currently this will fail - driving implementation
        assert!(result.is_ok(), "DataFrame::new() should be recognized");
    }

    #[test]
    fn test_dataframe_builder_column_method() {
        let mut repl = Repl::new().unwrap();
        let result = repl.eval(r#"
            DataFrame::new()
                .column("name", ["Alice", "Bob"])
        "#);
        assert!(result.is_ok(), "DataFrame builder should support .column()");
    }

    #[test]
    fn test_dataframe_builder_build_method() {
        let mut repl = Repl::new().unwrap();
        let result = repl.eval(r#"
            DataFrame::new()
                .column("name", ["Alice", "Bob"])
                .build()
        "#);
        assert!(result.is_ok(), "DataFrame builder should support .build()");
    }

    #[test]
    fn test_dataframe_rows_method() {
        let mut repl = Repl::new().unwrap();
        let result = repl.eval(r#"
            DataFrame::new()
                .column("name", ["Alice", "Bob", "Charlie"])
                .build()
                .rows()
        "#).unwrap();
        assert_eq!(result, "3", "DataFrame should report correct row count");
    }

    #[test]
    fn test_dataframe_columns_method() {
        let mut repl = Repl::new().unwrap();
        let result = repl.eval(r#"
            DataFrame::new()
                .column("name", ["Alice"])
                .column("age", [25])
                .column("city", ["NYC"])
                .build()
                .columns()
        "#).unwrap();
        assert_eq!(result, "3", "DataFrame should report correct column count");
    }

    #[test]
    fn test_dataframe_get_method() {
        let mut repl = Repl::new().unwrap();
        
        let result = repl.eval(r#"
            DataFrame::new()
                .column("name", ["Alice", "Bob"])
                .column("age", [25, 30])
                .build()
                .get("name", 0)
        "#).unwrap();
        assert_eq!(result, "\"Alice\"", "Should get correct value");
        
        let result2 = repl.eval(r#"
            DataFrame::new()
                .column("name", ["Alice", "Bob"])
                .column("age", [25, 30])
                .build()
                .get("age", 1)
        "#).unwrap();
        assert_eq!(result2, "30", "Should get correct numeric value");
    }
}