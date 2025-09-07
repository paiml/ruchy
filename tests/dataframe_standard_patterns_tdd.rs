/// TDD Tests for Standard DataFrame Patterns
/// 
/// Tests that standard data science patterns work correctly
/// Everyone expects "df = ..." to work

#[cfg(test)]
mod dataframe_standard_tests {
    use ruchy::{Parser, Transpiler};

    #[test]
    fn test_df_as_variable_name() {
        // Most common pattern in data science
        let code = r#"let df = df!["x" => [1, 2, 3]]"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        
        assert!(result.is_ok(), "Should parse df as variable name");
        let transpiled = result.unwrap().to_string();
        assert!(transpiled.contains("let df ="), "Should have df variable");
    }

    #[test]
    fn test_standard_data_science_workflow() {
        // Standard data science workflow
        let code = r#"let df = df!["x" => [1, 2, 3], "y" => [4, 5, 6]]"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        
        assert!(result.is_ok(), "Should transpile standard workflow");
    }

    #[test]
    fn test_dataframe_from_csv_pattern() {
        // Common CSV loading pattern
        let code = r#"let df = DataFrame::from_csv("data.csv")"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        
        assert!(result.is_ok(), "Should transpile from_csv");
        let transpiled = result.unwrap().to_string();
        assert!(transpiled.contains("from_csv") || transpiled.contains("read_csv"),
            "Should generate CSV reading code");
    }

    #[test]
    fn test_multiple_df_variables() {
        // Multiple DataFrames with standard naming
        let code = r#"
            let df1 = df!["a" => [1, 2]]
            let df2 = df!["b" => [3, 4]]
            let df_merged = df1.join(df2)
        "#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        
        assert!(result.is_ok(), "Should handle multiple df variables");
    }

    #[test]
    fn test_mutable_df() {
        // Mutable DataFrame
        let code = r#"let mut df = df!["x" => [1, 2, 3]]"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        
        assert!(result.is_ok(), "Should handle mutable df");
        let transpiled = result.unwrap().to_string();
        assert!(transpiled.contains("let mut df"), "Should have mutable df");
    }

    #[test]
    fn test_df_method_chaining() {
        // Method chaining - the pandas/polars way
        let code = r#"
            let df = df!["x" => [1, 2, 3], "y" => [4, 5, 6]]
                .filter(x > 1)
                .select(["x", "y"])
                .sort_by("x")
        "#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        
        assert!(result.is_ok(), "Should handle method chaining");
    }
}