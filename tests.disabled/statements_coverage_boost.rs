//! Comprehensive tests to boost statements.rs coverage to 100%
//! Sprint 75: Systematic coverage improvement

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::ast::{
    Expr, ExprKind, ImportItem, Literal, Param, Pattern, Span, Type, TypeKind,
};
use ruchy::frontend::parser::Parser;

#[test]
fn test_print_macros_comprehensive() {
    let transpiler = Transpiler::new();

    // Test println with single string
    let mut parser = Parser::new("println(\"hello\")");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("println!"));

    // Test println with integer (should add format string)
    let mut parser = Parser::new("println(42)");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("{:?}"));

    // Test print without newline
    let mut parser = Parser::new("print(\"test\")");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("print!"));

    // Test dbg macro
    let mut parser = Parser::new("dbg(x + 1)");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("dbg!"));

    // Test panic macro
    let mut parser = Parser::new("panic(\"error message\")");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("panic!"));
}

#[test]
fn test_print_with_multiple_args() {
    let transpiler = Transpiler::new();

    // Test println with multiple arguments
    let mut parser = Parser::new("println(\"x = {}, y = {}\", x, y)");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("println!"));

    // Test print with format string
    let mut parser = Parser::new("print(\"value: {}\", 42)");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("print!"));
}

#[test]
fn test_string_interpolation_in_print() {
    let transpiler = Transpiler::new();

    // Test println with string interpolation
    let mut parser = Parser::new("println(f\"Hello {name}!\")");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("format!") || result.contains("println!"));

    // Test print with complex interpolation
    let mut parser = Parser::new("print(f\"Result: {x + y}\")");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("print!"));
}

#[test]
fn test_import_statements_comprehensive() {
    let transpiler = Transpiler::new();

    // Simple import
    let import_expr = Expr::new(
        ExprKind::Import {
            module: "std::collections".to_string(),
            items: Some(vec!["HashMap".to_string()]),
        },
        Span::new(0, 0),
    );
    let result = transpiler.transpile_expr(&import_expr).unwrap().to_string();
    assert!(result.contains("use"));
    assert!(result.contains("std::collections"));

    // Import with multiple items
    let import_expr = Expr::new(
        ExprKind::Import {
            module: "std::sync".to_string(),
            items: Some(vec!["Arc".to_string(), "Mutex".to_string()]),
        },
        Span::new(0, 0),
    );
    let result = transpiler.transpile_expr(&import_expr).unwrap().to_string();
    assert!(result.contains("Arc") && result.contains("Mutex"));

    // Import all (wildcard)
    let import_expr = Expr::new(
        ExprKind::Import {
            module: "std::prelude".to_string(),
            items: None,
        },
        Span::new(0, 0),
    );
    let result = transpiler.transpile_expr(&import_expr).unwrap().to_string();
    assert!(result.contains("*") || result.contains("std::prelude"));
}

#[test]
fn test_export_statements() {
    let transpiler = Transpiler::new();

    // Export a simple expression
    let inner_expr = Box::new(Expr::new(
        ExprKind::Identifier("my_function".to_string()),
        Span::new(0, 0),
    ));
    let export_expr = Expr::new(
        ExprKind::Export {
            expr: inner_expr,
            is_default: false,
        },
        Span::new(0, 0),
    );
    let result = transpiler.transpile_expr(&export_expr).unwrap().to_string();
    assert!(result.contains("pub"));

    // Default export
    let inner_expr = Box::new(Expr::new(
        ExprKind::Identifier("main_export".to_string()),
        Span::new(0, 0),
    ));
    let export_expr = Expr::new(
        ExprKind::Export {
            expr: inner_expr,
            is_default: true,
        },
        Span::new(0, 0),
    );
    let result = transpiler.transpile_expr(&export_expr).unwrap().to_string();
    assert!(result.contains("pub"));
}

#[test]
fn test_for_loop_variations() {
    let transpiler = Transpiler::new();

    // For loop with range
    let mut parser = Parser::new("for i in 0..10 { println(i) }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("for"));

    // For loop with array
    let mut parser = Parser::new("for x in [1, 2, 3] { print(x) }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("for"));

    // For loop with iterator method
    let mut parser = Parser::new("for item in list.iter() { process(item) }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("for"));
}

#[test]
fn test_while_loop_variations() {
    let transpiler = Transpiler::new();

    // Simple while
    let mut parser = Parser::new("while x < 10 { x = x + 1 }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("while"));

    // While with complex condition
    let mut parser = Parser::new("while x > 0 && y < 100 { process() }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("while"));
}

#[test]
fn test_loop_control_flow() {
    let transpiler = Transpiler::new();

    // Loop with break
    let mut parser = Parser::new("loop { if done { break } }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("loop"));
    assert!(result.contains("break"));

    // Loop with continue
    let mut parser = Parser::new("loop { if skip { continue } process() }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("continue"));

    // Break with value
    let mut parser = Parser::new("let x = loop { break 42 }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("break"));
}

#[test]
fn test_pattern_matching_comprehensive() {
    let transpiler = Transpiler::new();

    // Match with literals
    let mut parser = Parser::new("match x { 1 => \"one\", 2 => \"two\", _ => \"other\" }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("match"));

    // Match with patterns
    let mut parser = Parser::new("match opt { Some(x) => x, None => 0 }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("Some"));
    assert!(result.contains("None"));

    // Match with guards
    let mut parser = Parser::new("match x { n if n > 0 => \"positive\", _ => \"non-positive\" }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("if"));
}

#[test]
fn test_try_catch_error_handling() {
    let transpiler = Transpiler::new();

    // Try-catch with simple error
    let mut parser = Parser::new("try { risky_operation() } catch e { handle_error(e) }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    // Try-catch transpiles to match on Result
    assert!(result.contains("match") || result.contains("Result"));

    // Try with question mark operator
    let mut parser = Parser::new("fn test() { let x = risky()?; x }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("?"));
}

#[test]
fn test_destructuring_patterns() {
    let transpiler = Transpiler::new();

    // Tuple destructuring
    let mut parser = Parser::new("let (x, y) = point");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("let"));

    // Array destructuring with rest
    let mut parser = Parser::new("let [first, ...rest] = items");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("[") || result.contains("first"));

    // Struct destructuring
    let mut parser = Parser::new("let Point { x, y } = p");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("Point") || result.contains("x"));
}

#[test]
fn test_type_annotations() {
    let transpiler = Transpiler::new();

    // Let with type annotation
    let mut parser = Parser::new("let x: i32 = 42");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("i32"));

    // Function with return type
    let mut parser = Parser::new("fn add(a: i32, b: i32) -> i32 { a + b }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("-> i32"));

    // Generic types
    let mut parser = Parser::new("let v: Vec<String> = Vec::new()");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("Vec"));
}

#[test]
fn test_async_await() {
    let transpiler = Transpiler::new();

    // Async function
    let mut parser = Parser::new("async fn fetch_data() { await get_remote() }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("async"));

    // Await expression
    let mut parser = Parser::new("let data = await fetch()");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("await"));
}

#[test]
fn test_special_function_calls() {
    let transpiler = Transpiler::new();

    // DataFrame operations
    let mut parser = Parser::new("df.select(\"column\")");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("select"));

    // Method chaining
    let mut parser = Parser::new("list.map(f).filter(g).collect()");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("map"));
    assert!(result.contains("filter"));
    assert!(result.contains("collect"));
}

#[test]
fn test_edge_cases() {
    let transpiler = Transpiler::new();

    // Empty block
    let mut parser = Parser::new("{ }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("{"));

    // Single expression block
    let mut parser = Parser::new("{ 42 }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("42"));

    // Nested blocks
    let mut parser = Parser::new("{ { { 1 } } }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("1"));
}

#[test]
fn test_compound_assignments() {
    let transpiler = Transpiler::new();

    // Add-assign
    let mut parser = Parser::new("x += 5");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("+="));

    // Multiply-assign
    let mut parser = Parser::new("y *= 2");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("*="));

    // Bitwise operations
    let mut parser = Parser::new("flags |= FLAG_BIT");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("|="));
}

#[test]
fn test_increment_decrement() {
    let transpiler = Transpiler::new();

    // Post-increment
    let mut parser = Parser::new("x++");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("+="));

    // Pre-decrement
    let mut parser = Parser::new("--y");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("-="));
}

#[test]
fn test_enum_and_struct_definitions() {
    let transpiler = Transpiler::new();

    // Enum definition
    let mut parser = Parser::new("enum Color { Red, Green, Blue }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("enum"));

    // Struct definition
    let mut parser = Parser::new("struct Point { x: f64, y: f64 }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("struct"));
}

#[test]
fn test_module_definitions() {
    let transpiler = Transpiler::new();

    // Module with items
    let mut parser = Parser::new("mod utils { fn helper() { } }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("mod"));
}

#[test]
fn test_closures_and_lambdas() {
    let transpiler = Transpiler::new();

    // Simple closure
    let mut parser = Parser::new("let add = |x, y| x + y");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("|"));

    // Closure with move
    let mut parser = Parser::new("let f = move |x| x * 2");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("move"));

    // Complex closure
    let mut parser = Parser::new("list.map(|x| { let y = x + 1; y * y })");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("|"));
}

#[test]
fn test_return_statements() {
    let transpiler = Transpiler::new();

    // Explicit return
    let mut parser = Parser::new("fn test() { return 42 }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("return"));

    // Early return
    let mut parser = Parser::new("fn check(x) { if x < 0 { return false } true }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("return"));

    // Return with expression
    let mut parser = Parser::new("fn compute() { return x * y + z }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("return"));
}

#[test]
fn test_reserved_keywords() {
    let transpiler = Transpiler::new();

    // Using reserved keywords with r#
    let mut parser = Parser::new("let r#type = \"test\"");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("r#type"));

    // Function with reserved name
    let mut parser = Parser::new("fn r#match() { }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("r#match"));
}
