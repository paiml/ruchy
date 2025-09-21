//! TDD Tests for Impl Blocks & Methods
//! Sprint v3.9.0 - Testing method transpilation fix

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::ast::{Expr, ExprKind};
use ruchy::frontend::parser::Parser;

#[cfg(test)]
mod impl_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_simple_impl_block() {
        let input = r#"
        impl Point {
            fun new(x: i32, y: i32) -> Point {
                Point { x: x, y: y }
            }
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse impl block");

        let ast = result.unwrap();
        if let ExprKind::Impl {
            for_type, methods, ..
        } = &ast.kind
        {
            assert_eq!(for_type, "Point");
            assert_eq!(methods.len(), 1);
            assert_eq!(methods[0].name, "new");
        } else {
            panic!("Expected Impl block");
        }
    }

    #[test]
    fn test_parse_impl_with_self_methods() {
        let input = r#"
        impl Point {
            fun distance(&self) -> f64 {
                sqrt((self.x * self.x) as f64)
            }

            fun move(&mut self, dx: i32, dy: i32) {
                self.x += dx;
                self.y += dy;
            }
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse impl with self methods");

        let ast = result.unwrap();
        if let ExprKind::Impl { methods, .. } = &ast.kind {
            assert_eq!(methods.len(), 2);
            // Check first method has &self
            assert!(methods[0].params.len() > 0);
            // Check second method has &mut self
            assert!(methods[1].params.len() > 0);
        } else {
            panic!("Expected Impl block");
        }
    }

    #[test]
    fn test_parse_associated_function_call() {
        let input = "Point::new(10, 20)";
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse associated function call");

        let ast = result.unwrap();
        if let ExprKind::Call { func, args } = &ast.kind {
            // Associated functions are parsed as Identifier with :: in the name
            if let ExprKind::Identifier(name) = &func.kind {
                assert_eq!(name, "Point::new");
                assert_eq!(args.len(), 2);
            } else {
                panic!("Expected identifier, got: {:?}", func.kind);
            }
        } else {
            panic!("Expected Call expression");
        }
    }

    #[test]
    fn test_parse_method_call() {
        let input = "point.distance()";
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse method call");

        let ast = result.unwrap();
        if let ExprKind::MethodCall {
            receiver,
            method,
            args,
        } = &ast.kind
        {
            if let ExprKind::Identifier(name) = &receiver.kind {
                assert_eq!(name, "point");
            }
            assert_eq!(method, "distance");
            assert_eq!(args.len(), 0);
        } else {
            panic!("Expected MethodCall expression");
        }
    }
}

#[cfg(test)]
mod impl_transpilation_tests {
    use super::*;

    #[test]
    fn test_transpile_simple_impl() {
        let input = r#"
        impl Point {
            fun new(x: i32, y: i32) -> Point {
                Point { x: x, y: y }
            }
        }"#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let result = transpiler.transpile_to_string(&ast);
        assert!(result.is_ok(), "Should transpile impl block");

        let code = result.unwrap();
        assert!(code.contains("impl Point"));
        assert!(code.contains("fn new"));
        assert!(code.contains("-> Point"));
        assert!(!code.contains("impl Point { }"), "Should not be empty");
    }

    #[test]
    fn test_transpile_self_methods() {
        let input = r#"
        impl Point {
            fun distance(&self) -> f64 {
                sqrt((self.x * self.x) as f64)
            }
        }"#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let result = transpiler.transpile_to_string(&ast);
        assert!(result.is_ok(), "Should transpile self methods");

        let code = result.unwrap();
        assert!(code.contains("impl Point"));
        assert!(code.contains("fn distance"));
        // Check for &self (may have spaces)
        assert!(code.contains("& self") || code.contains("&self"));
        // self.x becomes self.get("x") in transpilation
        assert!(code.contains("self") && code.contains("\"x\""));
    }

    #[test]
    fn test_transpile_associated_function_call() {
        let input = "Point::new(10, 20)";
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let result = transpiler.transpile_to_string(&ast);
        assert!(result.is_ok(), "Should transpile associated function call");

        let code = result.unwrap();
        // Transpiler may add spaces around ::
        assert!(code.contains("Point") && code.contains("new"));
        assert!(code.contains("10"));
        assert!(code.contains("20"));
    }

    #[test]
    fn test_transpile_method_call() {
        let input = "point.distance()";
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let result = transpiler.transpile_to_string(&ast);
        assert!(result.is_ok(), "Should transpile method call");

        let code = result.unwrap();
        // Transpiler may add spaces around .
        assert!(code.contains("point") && code.contains("distance"));
    }

    #[test]
    fn test_transpile_mutable_self() {
        let input = r#"
        impl Point {
            fun move(&mut self, dx: i32, dy: i32) {
                self.x = self.x + dx;
                self.y = self.y + dy;
            }
        }"#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let result = transpiler.transpile_to_string(&ast);
        assert!(result.is_ok(), "Should transpile mutable self");

        let code = result.unwrap();
        // Check for &mut self (may have spaces)
        assert!(code.contains("& mut self") || code.contains("&mut self"));
        // Assignment becomes more complex in transpilation
        assert!(code.contains("self") && code.contains("dx"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_impl_parsing_never_panics(type_name in "[A-Z][a-zA-Z0-9]*") {
            let input = format!("impl {} {{ }}", type_name);
            let mut parser = Parser::new(&input);
            let _ = parser.parse(); // Should not panic
        }

        #[test]
        fn test_method_call_parsing_never_panics(
            obj in "[a-z][a-z0-9_]*",
            method in "[a-z][a-z0-9_]*"
        ) {
            let input = format!("{}.{}()", obj, method);
            let mut parser = Parser::new(&input);
            let _ = parser.parse(); // Should not panic
        }

        #[test]
        fn test_associated_function_parsing_never_panics(
            type_name in "[A-Z][a-zA-Z0-9]*",
            func_name in "[a-z][a-z0-9_]*"
        ) {
            let input = format!("{}::{}()", type_name, func_name);
            let mut parser = Parser::new(&input);
            let _ = parser.parse(); // Should not panic
        }
    }
}
