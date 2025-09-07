//! TDD Tests for DataFrame Builder Pattern Implementation
//! 
//! These tests drive the implementation of builder patterns needed for Ch18 DataFrames
//! Following strict TDD: Red -> Green -> Refactor

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

#[test]
fn test_dataframe_new_builder() {
    let code = "let df = DataFrame::new();";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse DataFrame::new(): {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile DataFrame::new(): {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("DataFrame") && (rust_code.contains("empty") || rust_code.contains("new")), 
        "Should generate DataFrame constructor: {}", rust_code);
}

#[test]
fn test_dataframe_column_builder() {
    let code = r#"let df = DataFrame::new().column("name", ["Alice", "Bob", "Charlie"]);"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse column builder: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile column builder: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("column") && rust_code.contains("name"), 
        "Should generate column method call: {}", rust_code);
}

#[test]
fn test_dataframe_build_method() {
    let code = r#"let df = DataFrame::new()
        .column("id", [1, 2, 3])
        .column("name", ["Alice", "Bob", "Charlie"])
        .build();"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse build method: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile build method: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("build"), "Should generate build method call: {}", rust_code);
}

#[test]
fn test_dataframe_from_csv_string() {
    let code = r#"let df = DataFrame::from_csv_string("id,name\n1,Alice\n2,Bob");"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse from_csv_string: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile from_csv_string: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("from_csv_string") || rust_code.contains("read_csv"), 
        "Should generate CSV loading: {}", rust_code);
}

#[test]
fn test_dataframe_from_json() {
    let code = r#"let df = DataFrame::from_json([
        {"id": 1, "name": "Alice"},
        {"id": 2, "name": "Bob"}
    ]);"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse from_json: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile from_json: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("from_json") || rust_code.contains("from_rows"), 
        "Should generate JSON loading: {}", rust_code);
}

#[test]
fn test_dataframe_rows_method() {
    let code = r#"let row_count = df.rows();"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    // Debug output to understand the failure
    match &result {
        Ok(ast) => println!("DEBUG: Successfully parsed: {:?}", ast),
        Err(e) => println!("DEBUG: Parse error for 'df.rows()': {}", e),
    }
    
    assert!(result.is_ok(), "Should parse rows method: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile rows method: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("rows") || rust_code.contains("height"), 
        "Should generate rows method: {}", rust_code);
}

#[test]
fn test_dataframe_columns_method() {
    let code = r#"let col_names = df.columns();"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse columns method: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile columns method: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("columns") || rust_code.contains("get_column_names"), 
        "Should generate columns method: {}", rust_code);
}

#[test]
fn test_dataframe_get_method() {
    let code = r#"let column = df.get("name");"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse get method: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile get method: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("get") || rust_code.contains("column"), 
        "Should generate get method: {}", rust_code);
}

#[test]
fn test_complex_builder_chain() {
    let code = r#"
        let df = DataFrame::new()
            .column("id", [1, 2, 3, 4])
            .column("name", ["Alice", "Bob", "Charlie", "David"])
            .column("age", [25, 30, 35, 28])
            .column("salary", [50000.0, 60000.0, 55000.0, 52000.0])
            .build();
        
        let total_rows = df.rows();
        let column_names = df.columns();
        let names = df.get("name");
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse complex builder chain: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile complex builder chain: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("column") && rust_code.contains("build"), 
        "Should generate builder pattern: {}", rust_code);
    assert!(rust_code.contains("rows") && rust_code.contains("columns") && rust_code.contains("get"), 
        "Should generate analysis methods: {}", rust_code);
}

#[test]
fn test_dataframe_method_chaining_complexity() {
    // Test various combinations to ensure builder pattern works comprehensively
    let test_cases = vec![
        ("DataFrame::new().build()", "basic builder"),
        ("DataFrame::new().column(\"x\", [1]).build()", "single column"),
        ("DataFrame::from_csv_string(\"a,b\\n1,2\")", "CSV loading"),
        ("df.rows()", "row count"),
        ("df.columns()", "column names"),
        ("df.get(\"column_name\")", "column access"),
    ];
    
    for (code, description) in test_cases {
        let mut parser = Parser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse {}: {:?}", description, result.err());
        
        let ast = result.unwrap();
        let transpiler = Transpiler::new();
        let transpiled = transpiler.transpile(&ast);
        assert!(transpiled.is_ok(), "Should transpile {}: {:?}", description, transpiled.err());
    }
}

#[test]
fn test_dataframe_transpiler_integration() {
    // Test that builder pattern integrates correctly with existing DataFrame support
    let code = r#"
        fun create_sample_data() -> DataFrame {
            DataFrame::new()
                .column("product", ["Widget", "Gadget", "Tool"])
                .column("price", [10.99, 25.50, 15.00])
                .column("quantity", [100, 50, 75])
                .build()
        }
        
        fun analyze_data(df: DataFrame) -> i32 {
            df.rows()
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse DataFrame functions: {:?}", result.err());
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile DataFrame functions: {:?}", transpiled.err());
    
    let rust_code = transpiled.unwrap().to_string();
    assert!(rust_code.contains("fn create_sample_data") && rust_code.contains("fn analyze_data"), 
        "Should generate functions: {}", rust_code);
    assert!(rust_code.contains("DataFrame") && rust_code.contains("column"), 
        "Should preserve DataFrame builder pattern: {}", rust_code);
}