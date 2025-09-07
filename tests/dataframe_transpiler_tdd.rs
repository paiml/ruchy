/// TDD Tests for DataFrame Transpilation
/// 
/// These tests drive the implementation of DataFrame transpiler support
/// Following strict TDD: Red -> Green -> Refactor

#[cfg(test)]
mod dataframe_transpiler_tests {
    use ruchy::{Parser, Transpiler};

    #[test]
    fn test_transpile_empty_dataframe_literal() {
        // Simplest test - empty DataFrame literal should transpile
        let code = "df![]";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        
        assert!(result.is_ok(), "Should transpile empty DataFrame literal");
        let transpiled = result.unwrap().to_string();
        assert!(transpiled.contains("DataFrame") && transpiled.contains("empty"),
            "Should generate DataFrame::empty() - got: {}", transpiled);
    }

    #[test]
    fn test_transpile_dataframe_with_single_column() {
        let code = r#"df![
            "name" => ["Alice", "Bob", "Charlie"]
        ]"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        
        assert!(result.is_ok(), "Should transpile DataFrame with column");
        let transpiled = result.unwrap().to_string();
        assert!(transpiled.contains("DataFrame"), "Should have DataFrame");
        assert!(transpiled.contains("name"), "Should have column name");
    }

    #[test]
    fn test_transpile_dataframe_with_multiple_columns() {
        let code = r#"df![
            "name" => ["Alice", "Bob"],
            "age" => [25, 30],
            "city" => ["NYC", "LA"]
        ]"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        
        assert!(result.is_ok(), "Should transpile DataFrame with multiple columns");
        let transpiled = result.unwrap().to_string();
        assert!(transpiled.contains("DataFrame"), "Should have DataFrame");
        assert!(transpiled.contains("name"), "Should have name column");
        assert!(transpiled.contains("age"), "Should have age column");
        assert!(transpiled.contains("city"), "Should have city column");
    }

    #[test]
    fn test_transpile_dataframe_new() {
        let code = "DataFrame::new()";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        
        assert!(result.is_ok(), "Should transpile DataFrame::new()");
        let transpiled = result.unwrap().to_string();
        assert!(transpiled.contains("DataFrame") && transpiled.contains("new"), 
            "Should generate DataFrame::new() - got: {}", transpiled);
    }

    #[test]
    fn test_transpile_dataframe_from_csv() {
        let code = r#"DataFrame::from_csv("data.csv")"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        
        assert!(result.is_ok(), "Should transpile DataFrame::from_csv()");
        let transpiled = result.unwrap().to_string();
        assert!(transpiled.contains("read_csv") || transpiled.contains("from_csv"),
            "Should generate CSV reading code");
    }

    #[test]
    fn test_transpile_dataframe_select() {
        let code = r#"df!["name" => ["Alice"]].select(["name"])"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        
        assert!(result.is_ok(), "Should transpile DataFrame select");
        let transpiled = result.unwrap().to_string();
        assert!(transpiled.contains("select"), "Should have select method");
    }

    #[test]
    fn test_transpile_dataframe_filter() {
        let code = r#"df!["age" => [25, 30]].filter(age > 25)"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        
        assert!(result.is_ok(), "Should transpile DataFrame filter");
        let transpiled = result.unwrap().to_string();
        assert!(transpiled.contains("filter"), "Should have filter method");
    }

    #[test]
    fn test_transpile_dataframe_group_by() {
        let code = r#"df!["category" => ["A", "B"], "value" => [1, 2]].group_by("category")"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        
        assert!(result.is_ok(), "Should transpile DataFrame group_by");
        let transpiled = result.unwrap().to_string();
        assert!(transpiled.contains("group_by") || transpiled.contains("groupby"),
            "Should have group_by method - got: {}", transpiled);
    }

    #[test]
    fn test_transpile_dataframe_chain() {
        let code = r#"df!["age" => [25, 30], "name" => ["Alice", "Bob"], "city" => ["NYC", "LA"]]
            .filter(age > 25)
            .select(["name", "city"])
            .sort_by("name")"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        
        assert!(result.is_ok(), "Should transpile DataFrame method chain");
        let transpiled = result.unwrap().to_string();
        assert!(transpiled.contains("filter"), "Should have filter");
        assert!(transpiled.contains("select"), "Should have select");
        assert!(transpiled.contains("sort"), "Should have sort");
    }

    #[test]
    #[ignore] // Import syntax not yet implemented in parser
    fn test_transpile_import_dataframe() {
        let code = "import dataframe as df";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        
        assert!(result.is_ok(), "Should transpile import dataframe");
        let transpiled = result.unwrap().to_string();
        assert!(transpiled.contains("use polars") || transpiled.contains("use datafusion"),
            "Should import a DataFrame library");
    }
}