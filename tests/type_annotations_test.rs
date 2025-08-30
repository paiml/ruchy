// BOOK-001: Type Annotations Parser Support Tests
// Following Toyota Way TDD - RED phase first

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

#[test]
fn test_parse_basic_type_annotation() {
    let code = "fun add(x: i32, y: i32) -> i32 { x + y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Should parse function with type annotations");
}

#[test]
fn test_parse_string_type_annotation() {
    let code = "fun greet(name: String) { println(\"Hello \" + name) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Should parse String type annotation");
}

#[test]
fn test_parse_bool_type_annotation() {
    let code = "fun check(flag: bool) -> bool { !flag }";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Should parse bool type annotation");
}

#[test]
fn test_parse_float_type_annotation() {
    let code = "fun calculate(x: f64, y: f64) -> f64 { x * y + 3.14 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Should parse f64 type annotation");
}

#[test]
fn test_parse_function_type_annotation() {
    let code = "fun apply(f: fn(i32) -> i32, x: i32) -> i32 { f(x) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Should parse function type annotation");
}

#[test]
fn test_parse_array_type_annotation() {
    let code = "fun sum(nums: [i32]) -> i32 { nums.reduce(|a, b| a + b, 0) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Should parse array type annotation");
}

#[test]
fn test_parse_generic_type_annotation() {
    let code = "fun identity<T>(x: T) -> T { x }";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Should parse generic type annotation");
}

#[test]
fn test_parse_option_type_annotation() {
    let code = "fun find(items: [i32], target: i32) -> Option<i32> { None }";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Should parse Option type annotation");
}

#[test]
fn test_parse_result_type_annotation() {
    let code = "fun safe_divide(x: i32, y: i32) -> Result<i32, String> { Ok(x / y) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Should parse Result type annotation");
}

#[test]
fn test_parse_tuple_type_annotation() {
    let code = "fun pair(x: i32, y: String) -> (i32, String) { (x, y) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Should parse tuple type annotation");
}

#[test]
fn test_parse_hashmap_type_annotation() {
    let code = "fun create_map() -> HashMap<String, i32> { HashMap::new() }";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Should parse HashMap type annotation");
}

#[test]
fn test_transpile_ignores_type_annotations() {
    // For phase 1, transpiler can ignore type annotations
    let code = "fun add(x: i32, y: i32) -> i32 { x + y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast);
    assert!(rust_code.is_ok(), "Should transpile even with type annotations");
    
    let rust_str = rust_code.unwrap().to_string();
    assert!(rust_str.contains("fn add"), "Should generate function");
}

#[test]
fn test_mixed_typed_untyped_params() {
    let code = "fun process(x: i32, y, z: String) { println(x + y) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Should parse mixed typed/untyped parameters");
}

#[test]
fn test_let_binding_type_annotation() {
    let code = "let x: i32 = 42";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Should parse let binding with type annotation");
}

#[test]
fn test_closure_type_annotation() {
    let code = "let add = |x: i32, y: i32| -> i32 { x + y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Should parse closure with type annotations");
}

// Property tests for complexity validation
#[test]
fn test_parse_type_complexity_under_10() {
    // Ensure parsing functions have complexity < 10
    // This will be validated by PMAT after implementation
}

// Book example tests
#[test]
fn test_book_example_typed_add() {
    let code = r"
        fun add(x: i32, y: i32) -> i32 {
            x + y
        }
        
        fun main() {
            let result = add(5, 3);
            println(result);
        }
    ";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Book example should parse");
}

#[test]
fn test_book_example_string_processing() {
    let code = r#"
        fun process_name(name: String) -> String {
            "Hello, " + name
        }
        
        fun main() {
            let greeting = process_name("Alice");
            println(greeting);
        }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Book string example should parse");
}

#[test]
fn test_book_example_array_operations() {
    let code = r"
        fun sum_array(nums: [i32]) -> i32 {
            let mut total = 0;
            for n in nums {
                total += n;
            }
            total
        }
    ";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Book array example should parse");
}