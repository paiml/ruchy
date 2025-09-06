// TDD tests for module system support
// This test captures the requirement for basic module declarations

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;
use ruchy::runtime::interpreter::Interpreter;

#[test]
fn test_parse_basic_module() {
    let code = r#"
        mod math {
            pub fun add(a: i32, b: i32) -> i32 {
                a + b
            }
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse module declaration");
    
    // Check that it parses correctly
    assert!(format!("{:?}", ast).contains("Module") || format!("{:?}", ast).contains("mod"));
}

#[test]
fn test_module_with_multiple_functions() {
    let code = r#"
        mod utils {
            pub fun square(x: i32) -> i32 {
                x * x
            }
            
            pub fun double(x: i32) -> i32 {
                x * 2
            }
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse module with multiple functions");
    
    // Verify parsing succeeds
    assert!(format!("{:?}", ast).contains("square"));
    assert!(format!("{:?}", ast).contains("double"));
}

#[test]
fn test_module_function_call() {
    let code = r#"
        mod math {
            pub fun add(a: i32, b: i32) -> i32 {
                a + b
            }
        }
        
        fun main() {
            let result = math::add(5, 3);
            println(result);
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse module and function call");
    
    // Check for module path access
    assert!(format!("{:?}", ast).contains("math") || format!("{:?}", ast).contains("::"));
}

#[test]
fn test_transpile_module() {
    let code = r#"
        mod math {
            pub fun add(a: i32, b: i32) -> i32 {
                a + b
            }
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    
    // Should generate Rust module
    let rust_str = rust_code.to_string();
    assert!(rust_str.contains("mod math") || rust_str.contains("pub mod math"),
            "Should transpile to Rust module. Got: {}", rust_str);
}

#[test]
fn test_use_statement() {
    let code = r#"
        mod math {
            pub fun add(a: i32, b: i32) -> i32 {
                a + b
            }
        }
        
        use math::add;
        
        fun main() {
            let result = add(5, 3);
            println(result);
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse use statement");
    
    // Verify use statement is parsed
    assert!(format!("{:?}", ast).contains("use") || format!("{:?}", ast).contains("Import"));
}

#[test]
fn test_nested_modules() {
    let code = r#"
        mod outer {
            mod inner {
                pub fun helper() -> i32 {
                    42
                }
            }
            
            pub fun get_value() -> i32 {
                inner::helper()
            }
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse nested modules");
    
    // Verify nested structure
    let ast_str = format!("{:?}", ast);
    assert!(ast_str.contains("outer"));
    assert!(ast_str.contains("inner"));
}

#[test]
fn test_module_visibility() {
    let code = r#"
        mod utils {
            // Private function (no pub)
            fun internal_helper(x) {
                x + 1
            }
            
            // Public function
            pub fun public_api(x) {
                internal_helper(x) * 2
            }
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse public/private functions");
    
    let ast_str = format!("{:?}", ast);
    assert!(ast_str.contains("pub") || ast_str.contains("is_pub"));
}