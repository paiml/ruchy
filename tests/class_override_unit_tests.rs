//! EXTREME TDD: Unit tests for method override keyword
//! Tests the override keyword for explicit method overriding

use ruchy::{Parser, Transpiler};

#[test]
fn test_simple_override() {
    let code = r"
        class Shape {
            fn area(&self) -> f64 {
                0.0
            }
        }

        class Circle : Shape {
            radius: f64,

            override fn area(&self) -> f64 {
                3.14159 * self.radius * self.radius
            }
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should generate both structs and the overridden method
    assert!(result_str.contains("struct Shape"));
    assert!(result_str.contains("struct Circle"));
    assert!(result_str.contains("fn area"));
}

#[test]
fn test_override_ast_structure() {
    let code = r"
        class Derived : Base {
            override fn process(&self) -> i32 {
                42
            }
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    // Verify AST structure for override
    if let ruchy::frontend::ast::ExprKind::Class { methods, .. } = &ast.kind {
        assert_eq!(methods.len(), 1, "Should have 1 method");
        let method = &methods[0];
        assert_eq!(method.name, "process");
        assert!(method.is_override, "Method should be marked as override");
    } else {
        panic!("AST should be a class");
    }
}

#[test]
fn test_multiple_overrides() {
    let code = r#"
        class Base {
            fn method1(&self) -> i32 { 0 }
            fn method2(&self) -> String { "base".to_string() }
            fn method3(&self) -> bool { false }
        }

        class Derived : Base {
            override fn method1(&self) -> i32 { 1 }
            override fn method2(&self) -> String { "derived".to_string() }
            fn method3(&self) -> bool { true }  // Not marked as override
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    // Verify multiple overrides
    if let ruchy::frontend::ast::ExprKind::Block(exprs) = &ast.kind {
        if let ruchy::frontend::ast::ExprKind::Class { methods, .. } = &exprs[1].kind {
            assert_eq!(methods.len(), 3, "Should have 3 methods");
            assert!(methods[0].is_override, "method1 should be override");
            assert!(methods[1].is_override, "method2 should be override");
            assert!(!methods[2].is_override, "method3 should not be override");
        }
    }
}

#[test]
fn test_override_with_visibility() {
    let code = r"
        class Base {
            pub fn visible(&self) -> i32 { 0 }
        }

        class Derived : Base {
            pub override fn visible(&self) -> i32 { 1 }
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    // Verify override with visibility modifier
    if let ruchy::frontend::ast::ExprKind::Block(exprs) = &ast.kind {
        if let ruchy::frontend::ast::ExprKind::Class { methods, .. } = &exprs[1].kind {
            let method = &methods[0];
            assert!(method.is_override, "Should be marked as override");
            assert!(method.is_pub, "Should be public");
        }
    }
}

#[test]
fn test_override_with_different_self_types() {
    let code = r"
        class Collection {
            fn add(&mut self, item: i32) {}
            fn get(&self, index: usize) -> i32 { 0 }
            fn clear(mut self) {}
        }

        class CustomCollection : Collection {
            override fn add(&mut self, item: i32) {}
            override fn get(&self, index: usize) -> i32 { 42 }
            override fn clear(mut self) {}
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    // Verify overrides with different self types
    if let ruchy::frontend::ast::ExprKind::Block(exprs) = &ast.kind {
        if let ruchy::frontend::ast::ExprKind::Class { methods, .. } = &exprs[1].kind {
            assert_eq!(methods.len(), 3, "Should have 3 methods");
            for method in methods {
                assert!(method.is_override, "All methods should be override");
            }
        }
    }
}
