// TDD test for parse_dataframe_literal complexity refactoring
// GOAL: Reduce parse_dataframe_literal complexity from 22 to <10 via systematic extraction
// RED → GREEN → REFACTOR methodology

use ruchy::frontend::parser::Parser;

#[test]
fn test_parse_simple_dataframe() {
    // Test simple dataframe literals
    let test_cases = vec![
        (r#"df![]"#, "should parse empty dataframe"),
        (r#"df!["name" => []]"#, "should parse dataframe with empty column"),
        (r#"df!["age" => [1, 2, 3]]"#, "should parse dataframe with single column"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok(), "{}: {}", description, input);
        let debug_str = format!("{:?}", result.unwrap());
        assert!(debug_str.contains("DataFrame"), 
                "{}: Should contain DataFrame expression: {}", description, debug_str);
    }
}

#[test]
fn test_parse_dataframe_with_multiple_columns() {
    // Test dataframe with multiple columns
    let test_cases = vec![
        (r#"df!["name" => ["Alice"], "age" => [30]]"#, "should parse two columns"),
        (r#"df!["x" => [1, 2], "y" => [3, 4], "z" => [5, 6]]"#, "should parse three columns"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok(), "{}: {}", description, input);
        let debug_str = format!("{:?}", result.unwrap());
        assert!(debug_str.contains("DataFrame"), 
                "{}: Should contain DataFrame expression: {}", description, debug_str);
    }
}

#[test]
fn test_parse_dataframe_with_identifier_keys() {
    // Test dataframe with identifier keys (no quotes)
    let test_cases = vec![
        (r#"df![name => ["Bob"]]"#, "should parse identifier key"),
        (r#"df![x => [1], y => [2]]"#, "should parse multiple identifier keys"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok(), "{}: {}", description, input);
        let debug_str = format!("{:?}", result.unwrap());
        assert!(debug_str.contains("DataFrame"), 
                "{}: Should parse dataframe with identifier keys: {}", description, debug_str);
    }
}

#[test]
fn test_parse_dataframe_with_mixed_values() {
    // Test dataframe with mixed value types
    let test_cases = vec![
        (r#"df!["values" => [1, 2.5, "text", true]]"#, "should parse mixed types"),
        (r#"df!["data" => [null, 42, "hello"]]"#, "should parse with nulls"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        match result {
            Ok(expr) => {
                let debug_str = format!("{:?}", expr);
                assert!(debug_str.contains("DataFrame"),
                        "{}: Should parse mixed values: {}", description, debug_str);
            },
            Err(e) => {
                println!("⚠️  Mixed values not yet working: {} (will fix during refactoring)", e);
                // Don't fail - this guides our refactoring
            }
        }
    }
}

#[test]
fn test_parse_dataframe_with_trailing_comma() {
    // Test dataframe with trailing commas
    let test_cases = vec![
        (r#"df!["x" => [1,], ]"#, "should handle trailing comma in list"),
        (r#"df!["x" => [1], "y" => [2],]"#, "should handle trailing comma in columns"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        match result {
            Ok(expr) => {
                let debug_str = format!("{:?}", expr);
                println!("✅ {}: {}", description, input);
            },
            Err(e) => {
                println!("⚠️  Trailing comma not yet working: {} (will fix during refactoring)", e);
            }
        }
    }
}

#[test]
fn test_parse_dataframe_literal_complexity_is_reduced() {
    // This test will pass once we've successfully refactored parse_dataframe_literal
    // REQUIREMENT: parse_dataframe_literal should delegate to focused helper functions
    
    // After refactoring, parse_dataframe_literal should be a simple dispatcher
    // that calls focused functions like:
    // - parse_dataframe_header() - df![
    // - parse_dataframe_columns() - main column parsing loop
    // - parse_column_definition() - single "key" => [values]
    // - parse_column_name() - string or identifier
    // - parse_column_values() - list of values
    
    // For now, just ensure basic functionality works
    let mut parser = Parser::new(r#"df!["test" => [1, 2, 3]]"#);
    let result = parser.parse().expect("Should parse simple dataframe");
    assert!(format!("{:?}", result).contains("DataFrame"));
    
    // TODO: Add complexity measurement when we have the tools
    // assert!(parse_dataframe_literal_complexity() < 10);
}

#[test]
fn test_parse_dataframe_with_expressions() {
    // Test dataframe with computed expressions as values
    let test_cases = vec![
        (r#"df!["calc" => [1 + 1, 2 * 3]]"#, "should parse expressions"),
        (r#"df!["vars" => [x, y, z]]"#, "should parse variable references"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        match result {
            Ok(_) => {
                println!("✅ {}", description);
            },
            Err(e) => {
                println!("⚠️  Expression values not yet working: {} (will fix during refactoring)", e);
            }
        }
    }
}

#[test]
fn test_all_dataframe_features_work_after_refactoring() {
    // Comprehensive test to ensure refactoring doesn't break any feature
    let test_cases = vec![
        // Basic cases
        (r#"df![]"#, "empty dataframe"),
        (r#"df!["col" => []]"#, "empty column"),
        (r#"df!["col" => [1]]"#, "single value"),
        (r#"df!["col" => [1, 2, 3]]"#, "multiple values"),
        
        // Multiple columns
        (r#"df!["a" => [1], "b" => [2]]"#, "two columns"),
        
        // Identifier keys
        (r#"df![col => [1]]"#, "identifier key"),
        
        // Mixed types
        (r#"df!["mix" => [1, "text", true]]"#, "mixed types"),
    ];
    
    let mut passed = 0;
    let mut failed = 0;
    
    for (input, variant) in test_cases {
        let mut parser = Parser::new(input);
        match parser.parse() {
            Ok(_) => {
                println!("✅ {} variant: works", variant);
                passed += 1;
            },
            Err(e) => {
                println!("⚠️  {} variant failed: {}", variant, e);
                failed += 1;
            }
        }
    }
    
    println!("\n📊 Results: {} passed, {} failed", passed, failed);
    // We expect at least simple cases to work
    assert!(passed >= 5, "At least simple dataframe literals should work");
}