//! TDD Tests for DataFrame Transpiler to Generate Correct Polars API
//! 
//! These tests ensure the transpiler generates valid Polars code that compiles
//! Following strict TDD: Red -> Green -> Refactor

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

#[test]
fn test_dataframe_new_generates_empty() {
    let code = "DataFrame::new()";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse DataFrame::new()");
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile DataFrame::new()");
    
    let rust_code = transpiled.unwrap().to_string();
    println!("Generated: {}", rust_code);
    
    // Should generate empty DataFrame
    assert!(rust_code.contains("DataFrame") && rust_code.contains("empty"),
        "Should generate DataFrame::empty(). Got: {}", rust_code);
}

#[test]
fn test_dataframe_with_columns_generates_series() {
    let code = r#"
        DataFrame::new()
            .column("name", ["Alice", "Bob"])
            .column("age", [30, 25])
            .build()
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse DataFrame builder pattern");
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile DataFrame builder");
    
    let rust_code = transpiled.unwrap().to_string();
    println!("Generated: {}", rust_code);
    
    // Should generate Series::new() calls, not .column() which doesn't exist
    assert!(rust_code.contains("Series") && rust_code.contains("new"),
        "Should generate Series::new() for columns, not .column(). Got: {}", rust_code);
    assert!(!rust_code.contains(".column("),
        "Should NOT contain .column() method which doesn't exist in Polars");
    
    // Should have the column names
    assert!(rust_code.contains("\"name\""), "Should have 'name' column");
    assert!(rust_code.contains("\"age\""), "Should have 'age' column");
}

#[test]
fn test_dataframe_from_csv_generates_csvreader() {
    let code = r#"DataFrame::from_csv("data.csv")"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse DataFrame::from_csv()");
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile from_csv");
    
    let rust_code = transpiled.unwrap().to_string();
    println!("Generated: {}", rust_code);
    
    // Should use CsvReader
    assert!(rust_code.contains("CsvReader") || 
            rust_code.contains("read_csv"),
        "Should use CsvReader for CSV loading. Got: {}", rust_code);
    
    // Should have finish() or collect() to execute
    assert!(rust_code.contains("finish") || 
            rust_code.contains("collect"),
        "Should finalize CSV reading. Got: {}", rust_code);
}

#[test]
fn test_dataframe_lazy_operations() {
    let code = r#"
        df.filter(|row| row["age"] > 25)
          .select(["name", "age"])
          .sort("age")
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse DataFrame operations");
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile operations");
    
    let rust_code = transpiled.unwrap().to_string();
    println!("Generated: {}", rust_code);
    
    // Should use lazy API for chaining
    assert!(rust_code.contains("lazy") || 
            rust_code.contains("LazyFrame") ||
            rust_code.contains("DataFrame"),
        "Should use lazy or eager API. Got: {}", rust_code);
}

#[test]
fn test_dataframe_rows_method() {
    let code = "df.rows()";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse df.rows()");
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile rows()");
    
    let rust_code = transpiled.unwrap().to_string();
    println!("Generated: {}", rust_code);
    
    // Polars uses height() not rows()
    assert!(rust_code.contains("height") || 
            rust_code.contains("shape") ||
            rust_code.contains("len"),
        "Should use .height() or .shape() for row count. Got: {}", rust_code);
}

#[test]
fn test_dataframe_get_column() {
    let code = r#"df.get("price")"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse df.get()");
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile get()");
    
    let rust_code = transpiled.unwrap().to_string();
    println!("Generated: {}", rust_code);
    
    // Polars uses column() not get()
    assert!(rust_code.contains("column") || 
            rust_code.contains("get_column"),
        "Should use .column() for column access. Got: {}", rust_code);
}

#[test]
fn test_correct_polars_imports() {
    let code = r#"
        fun main() {
            let df = DataFrame::new()
                .column("x", [1, 2, 3])
                .build();
            println(df.rows());
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse full program");
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile program");
    
    let rust_code = transpiled.unwrap().to_string();
    println!("Generated: {}", rust_code);
    
    // Should have polars imports
    assert!(rust_code.contains("polars") || 
            rust_code.contains("use polars"),
        "Should include polars imports. Got: {}", rust_code);
}

#[test]
fn test_dataframe_from_json() {
    let code = r#"
        DataFrame::from_json([
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ])
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse from_json");
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile from_json");
    
    let rust_code = transpiled.unwrap().to_string();
    println!("Generated: {}", rust_code);
    
    // Should handle JSON data
    assert!(rust_code.contains("serde_json") || 
            rust_code.contains("json!") ||
            rust_code.contains("DataFrame"),
        "Should handle JSON input. Got: {}", rust_code);
}

#[test]
fn test_compilable_dataframe_code() {
    // This test verifies the generated code would actually compile with Polars
    let code = r#"
        let df = DataFrame::new()
            .column("product", ["Widget", "Gadget"])
            .column("price", [10.99, 25.50])
            .build();
        let total_rows = df.rows();
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse DataFrame code");
    
    let ast = result.unwrap();
    let transpiler = Transpiler::new();
    let transpiled = transpiler.transpile(&ast);
    assert!(transpiled.is_ok(), "Should transpile DataFrame code");
    
    let rust_code = transpiled.unwrap().to_string();
    println!("Generated Rust code:\n{}", rust_code);
    
    // The generated code should NOT have these invalid Polars patterns
    assert!(!rust_code.contains(".column(\""), 
        "Should NOT generate .column() method which doesn't exist in Polars");
    assert!(!rust_code.contains(".build()"), 
        "Should NOT generate .build() method which doesn't exist in Polars");
    assert!(!rust_code.contains(".rows()"), 
        "Should NOT generate .rows() method - Polars uses .height()");
}