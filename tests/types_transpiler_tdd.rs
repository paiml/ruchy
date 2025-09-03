//! Comprehensive TDD test suite for types.rs transpiler module
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every type system path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::{Transpiler, Parser};

// ==================== BASIC TYPE TESTS ====================

#[test]
fn test_transpile_type_int() {
    let transpiler = Transpiler::new();
    let code = "let x: int = 42";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("i64"));
}

#[test]
fn test_transpile_type_float() {
    let transpiler = Transpiler::new();
    let code = "let x: float = 3.14";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("f64"));
}

#[test]
fn test_transpile_type_bool() {
    let transpiler = Transpiler::new();
    let code = "let x: bool = true";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("bool"));
}

#[test]
fn test_transpile_type_string() {
    let transpiler = Transpiler::new();
    let code = r#"let x: String = "hello""#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("String"));
}

#[test]
fn test_transpile_type_char() {
    let transpiler = Transpiler::new();
    let code = "let x: char = 'a'";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("char"));
}

#[test]
fn test_transpile_type_any() {
    let transpiler = Transpiler::new();
    let code = "let x: Any = 42";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    // Should use type inference (_) instead of Any
}

#[test]
fn test_transpile_type_underscore() {
    let transpiler = Transpiler::new();
    let code = "let x: _ = 42";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    // Should use type inference
}

// ==================== GENERIC TYPE TESTS ====================

#[test]
fn test_transpile_type_vec() {
    let transpiler = Transpiler::new();
    let code = "let x: Vec<i32> = vec![1, 2, 3]";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("Vec"));
    assert!(transpiled.contains("i32"));
}

#[test]
fn test_transpile_type_option() {
    let transpiler = Transpiler::new();
    let code = "let x: Option<i32> = Some(42)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("Option"));
}

#[test]
fn test_transpile_type_result() {
    let transpiler = Transpiler::new();
    let code = "let x: Result<i32, String> = Ok(42)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("Result"));
}

#[test]
fn test_transpile_type_hashmap() {
    let transpiler = Transpiler::new();
    let code = "let x: HashMap<String, i32> = HashMap::new()";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("HashMap"));
}

// ==================== COLLECTION TYPE TESTS ====================

#[test]
fn test_transpile_type_list() {
    let transpiler = Transpiler::new();
    let code = "let x: [i32] = [1, 2, 3]";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    // List types should become Vec
    assert!(transpiled.contains("vec!") || transpiled.contains("Vec"));
}

#[test]
fn test_transpile_type_tuple() {
    let transpiler = Transpiler::new();
    let code = "let x: (i32, String, bool) = (42, \"hello\", true)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("("));
    assert!(transpiled.contains(")"));
}

#[test]
fn test_transpile_type_nested_generics() {
    let transpiler = Transpiler::new();
    let code = "let x: Vec<Option<i32>> = vec![Some(1), None]";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("Vec"));
    assert!(transpiled.contains("Option"));
}

// ==================== REFERENCE TYPE TESTS ====================

#[test]
fn test_transpile_type_ref() {
    let transpiler = Transpiler::new();
    let code = "let x: &i32 = &42";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("&"));
}

#[test]
fn test_transpile_type_mut_ref() {
    let transpiler = Transpiler::new();
    let code = "let x: &mut i32 = &mut 42";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("&mut"));
}

#[test]
fn test_transpile_type_str_ref() {
    let transpiler = Transpiler::new();
    let code = r#"let x: &str = "hello""#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("&str") || transpiled.contains("str"));
}

// ==================== FUNCTION TYPE TESTS ====================

#[test]
fn test_transpile_type_function() {
    let transpiler = Transpiler::new();
    let code = "let f: fn(i32, i32) -> i32 = add";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("fn"));
}

#[test]
fn test_transpile_type_closure() {
    let transpiler = Transpiler::new();
    let code = "let f: |i32| -> i32 = |x| x + 1";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

// ==================== STRUCT DEFINITION TESTS ====================

#[test]
fn test_transpile_simple_struct() {
    let transpiler = Transpiler::new();
    let code = "struct Point { x: i32, y: i32 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("struct Point"));
    assert!(transpiled.contains("x :"));
    assert!(transpiled.contains("y :"));
}

#[test]
fn test_transpile_public_struct() {
    let transpiler = Transpiler::new();
    let code = "pub struct Person { name: String, age: i32 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("pub struct Person"));
}

#[test]
fn test_transpile_generic_struct() {
    let transpiler = Transpiler::new();
    let code = "struct Container<T> { value: T }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("struct Container"));
    assert!(transpiled.contains("<T>") || transpiled.contains("< T >"));
}

#[test]
fn test_transpile_struct_with_default() {
    let transpiler = Transpiler::new();
    let code = "struct Config { timeout: i32 = 30, retries: i32 = 3 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("struct Config"));
}

// ==================== TRAIT DEFINITION TESTS ====================

#[test]
fn test_transpile_simple_trait() {
    let transpiler = Transpiler::new();
    let code = "trait Display { fun show(self); }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("trait Display"));
}

#[test]
fn test_transpile_trait_with_default() {
    let transpiler = Transpiler::new();
    let code = r#"
    trait Printable {
        fun print(self) {
            println("Default print")
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("trait Printable"));
}

#[test]
fn test_transpile_generic_trait() {
    let transpiler = Transpiler::new();
    let code = "trait Container<T> { fun get(self) -> T; }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("trait Container"));
}

// ==================== IMPL BLOCK TESTS ====================

#[test]
fn test_transpile_impl_block() {
    let transpiler = Transpiler::new();
    let code = r#"
    impl Point {
        fun new(x: i32, y: i32) -> Point {
            Point { x: x, y: y }
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("impl Point"));
}

#[test]
fn test_transpile_trait_impl() {
    let transpiler = Transpiler::new();
    let code = r#"
    impl Display for Point {
        fun show(self) {
            println(f"Point({self.x}, {self.y})")
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("impl Display for Point"));
}

#[test]
fn test_transpile_generic_impl() {
    let transpiler = Transpiler::new();
    let code = r#"
    impl<T> Container<T> {
        fun new(value: T) -> Container<T> {
            Container { value: value }
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("impl"));
    assert!(transpiled.contains("Container"));
}

// ==================== ENUM DEFINITION TESTS ====================

#[test]
fn test_transpile_simple_enum() {
    let transpiler = Transpiler::new();
    let code = "enum Color { Red, Green, Blue }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("enum Color"));
    assert!(transpiled.contains("Red"));
    assert!(transpiled.contains("Green"));
    assert!(transpiled.contains("Blue"));
}

#[test]
fn test_transpile_enum_with_values() {
    let transpiler = Transpiler::new();
    let code = "enum Option<T> { Some(T), None }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("enum Option"));
    assert!(transpiled.contains("Some"));
    assert!(transpiled.contains("None"));
}

#[test]
fn test_transpile_enum_with_struct_variants() {
    let transpiler = Transpiler::new();
    let code = r#"
    enum Message {
        Text(String),
        Move { x: i32, y: i32 },
        Quit
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("enum Message"));
    assert!(transpiled.contains("Text"));
    assert!(transpiled.contains("Move"));
    assert!(transpiled.contains("Quit"));
}

// ==================== DATAFRAME TYPE TESTS ====================

#[test]
fn test_transpile_type_dataframe() {
    let transpiler = Transpiler::new();
    let code = "let df: DataFrame = DataFrame::new()";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("DataFrame"));
}

#[test]
fn test_transpile_type_series() {
    let transpiler = Transpiler::new();
    let code = "let s: Series = Series::new(\"col\", vec![1, 2, 3])";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("Series"));
}

// Run all tests with: cargo test types_transpiler_tdd --test types_transpiler_tdd