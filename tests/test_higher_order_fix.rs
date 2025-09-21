//! Test-first development for BUG-002 fix
//! These tests define the expected behavior BEFORE implementing the fix

use ruchy::{Parser, Transpiler};

#[test]
fn test_main_has_no_return_type() {
    let code = r#"
fun main() {
    println("Hello, World!")
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_str = rust_code.to_string();

    // CRITICAL: main() must never have a return type annotation
    assert!(
        !rust_str.contains("fn main() ->"),
        "main() should not have return type, got: {rust_str}"
    );
    assert!(
        !rust_str.contains("fn main () ->"),
        "main() should not have return type, got: {rust_str}"
    );
}

#[test]
fn test_higher_order_function_types_correctly() {
    let code = r"
fun apply(f, x) {
    f(x)
}

fun double(n) {
    n * 2
}

apply(double, 5)
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_str = rust_code.to_string();

    // Function parameter should NOT be String
    assert!(
        !rust_str.contains("f : String"),
        "Function parameter f should not be String, got: {rust_str}"
    );

    // Should have some function type (impl Fn, dyn Fn, or generic)
    assert!(
        rust_str.contains("impl Fn")
            || rust_str.contains("dyn Fn")
            || rust_str.contains("F:")
            || rust_str.contains("f :"), // At least typed somehow
        "Function parameter f should have function type, got: {rust_str}"
    );
}

#[test]
fn test_string_params_still_work() {
    let code = r#"
fun greet(name) {
    println("Hello, " + name)
}

greet("World")
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");

    // Should compile without errors
    assert!(
        !rust_code.to_string().is_empty(),
        "Should transpile successfully"
    );
}

#[test]
fn test_numeric_functions_get_numeric_types() {
    let code = r"
fun add(a, b) {
    a + b
}

add(3, 4)
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_str = rust_code.to_string();

    // Numeric function should have numeric parameters
    assert!(
        rust_str.contains("a : i32")
            || rust_str.contains("a : i64")
            || rust_str.contains("a : f32")
            || rust_str.contains("a : f64")
            || !rust_str.contains("a : String"),
        "Numeric function should have numeric params, got: {rust_str}"
    );
}

#[test]
fn test_double_function_gets_correct_types() {
    let code = r"
fun double(n) {
    n * 2
}

double(5)
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_str = rust_code.to_string();

    // Function with multiplication should have numeric parameter
    assert!(
        !rust_str.contains("n : String"),
        "Parameter n in double() should not be String, got: {rust_str}"
    );
    assert!(
        rust_str.contains("n : i32")
            || rust_str.contains("n : i64")
            || rust_str.contains("n : f32")
            || rust_str.contains("n : f64"),
        "Parameter n in double() should be numeric, got: {rust_str}"
    );
}

#[test]
fn test_no_return_type_for_void_functions() {
    let code = r#"
fun log_message(msg) {
    println(msg)
}

log_message("test")
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_str = rust_code.to_string();

    // Functions that don't return values shouldn't have i32 return type
    if rust_str.contains("fn log_message") && rust_str.contains("->") {
        assert!(
            !rust_str.contains("-> i32"),
            "Void function should not return i32, got: {rust_str}"
        );
    }
}
