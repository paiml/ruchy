//! TDD tests for refactored parser modules
//! Comprehensive coverage for parser expression modules

#[cfg(test)]
mod literals_tests {
    use ruchy::frontend::parser::{Parser, ParserState};
    use ruchy::frontend::ast::{Expr, ExprKind, Literal};

    fn parse_expr(input: &str) -> Result<Expr, String> {
        let mut parser = Parser::new(input);
        parser.parse_expr().map_err(|e| e.to_string())
    }

    #[test]
    fn test_parse_integer_literal() {
        let expr = parse_expr("42").unwrap();
        match expr.kind {
            ExprKind::Literal(Literal::Integer(val)) => assert_eq!(val, 42),
            _ => panic!("Expected integer literal"),
        }
    }

    #[test]
    fn test_parse_float_literal() {
        let expr = parse_expr("3.14").unwrap();
        match expr.kind {
            ExprKind::Literal(Literal::Float(val)) => assert!((val - 3.14).abs() < 0.001),
            _ => panic!("Expected float literal"),
        }
    }

    #[test]
    fn test_parse_string_literal() {
        let expr = parse_expr(r#""hello world""#).unwrap();
        match expr.kind {
            ExprKind::Literal(Literal::String(val)) => assert_eq!(val, "hello world"),
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_parse_bool_literals() {
        let true_expr = parse_expr("true").unwrap();
        match true_expr.kind {
            ExprKind::Literal(Literal::Bool(val)) => assert!(val),
            _ => panic!("Expected boolean literal"),
        }

        let false_expr = parse_expr("false").unwrap();
        match false_expr.kind {
            ExprKind::Literal(Literal::Bool(val)) => assert!(!val),
            _ => panic!("Expected boolean literal"),
        }
    }

    #[test]
    fn test_parse_none_literal() {
        let expr = parse_expr("None").unwrap();
        match expr.kind {
            ExprKind::Literal(Literal::None) => {},
            _ => panic!("Expected None literal"),
        }
    }

    #[test]
    fn test_parse_char_literal() {
        let expr = parse_expr("'a'").unwrap();
        match expr.kind {
            ExprKind::Literal(Literal::Char(val)) => assert_eq!(val, 'a'),
            _ => panic!("Expected char literal"),
        }
    }
}

#[cfg(test)]
mod control_flow_tests {
    use ruchy::frontend::parser::Parser;
    use ruchy::frontend::ast::{Expr, ExprKind};

    fn parse_expr(input: &str) -> Result<Expr, String> {
        let mut parser = Parser::new(input);
        parser.parse_expr().map_err(|e| e.to_string())
    }

    #[test]
    fn test_parse_if_expression() {
        let expr = parse_expr("if x > 0 then 1 else 0").unwrap();
        assert!(matches!(expr.kind, ExprKind::If { .. }));
    }

    #[test]
    fn test_parse_if_block() {
        let expr = parse_expr("if x > 0 { print(x) }").unwrap();
        assert!(matches!(expr.kind, ExprKind::If { .. }));
    }

    #[test]
    fn test_parse_if_else_if() {
        let expr = parse_expr("if x > 0 { 1 } else if x < 0 { -1 } else { 0 }").unwrap();
        assert!(matches!(expr.kind, ExprKind::If { .. }));
    }

    #[test]
    fn test_parse_while_loop() {
        let expr = parse_expr("while x > 0 { x = x - 1 }").unwrap();
        assert!(matches!(expr.kind, ExprKind::While { .. }));
    }

    #[test]
    fn test_parse_for_loop() {
        let expr = parse_expr("for i in 0..10 { print(i) }").unwrap();
        assert!(matches!(expr.kind, ExprKind::For { .. }));
    }

    #[test]
    fn test_parse_loop() {
        let expr = parse_expr("loop { break }").unwrap();
        assert!(matches!(expr.kind, ExprKind::Loop { .. }));
    }

    #[test]
    fn test_parse_match() {
        let expr = parse_expr(r#"
            match x {
                0 => "zero",
                1 => "one",
                _ => "other"
            }
        "#).unwrap();
        assert!(matches!(expr.kind, ExprKind::Match { .. }));
    }

    #[test]
    fn test_parse_break() {
        let expr = parse_expr("break").unwrap();
        assert!(matches!(expr.kind, ExprKind::Break { value: None }));

        let expr = parse_expr("break 42").unwrap();
        assert!(matches!(expr.kind, ExprKind::Break { value: Some(_) }));
    }

    #[test]
    fn test_parse_continue() {
        let expr = parse_expr("continue").unwrap();
        assert!(matches!(expr.kind, ExprKind::Continue));
    }

    #[test]
    fn test_parse_return() {
        let expr = parse_expr("return").unwrap();
        assert!(matches!(expr.kind, ExprKind::Return { value: None }));

        let expr = parse_expr("return 42").unwrap();
        assert!(matches!(expr.kind, ExprKind::Return { value: Some(_) }));
    }
}

#[cfg(test)]
mod pattern_tests {
    use ruchy::frontend::parser::Parser;
    use ruchy::frontend::ast::{Pattern, Literal};

    fn parse_pattern(input: &str) -> Result<Pattern, String> {
        let mut parser = Parser::new(input);
        // Parse as part of a let statement
        let expr = parser.parse_expr().map_err(|e| e.to_string())?;
        
        // Extract pattern from let statement
        match expr.kind {
            ruchy::frontend::ast::ExprKind::Statement(stmt) => {
                match stmt.kind {
                    ruchy::frontend::parser::StmtKind::Let { pattern, .. } => Ok(pattern),
                    _ => Err("Not a let statement".to_string()),
                }
            }
            _ => Err("Not a statement".to_string()),
        }
    }

    #[test]
    fn test_wildcard_pattern() {
        let pattern = parse_pattern("let _ = 42").unwrap();
        assert!(matches!(pattern, Pattern::Wildcard));
    }

    #[test]
    fn test_identifier_pattern() {
        let pattern = parse_pattern("let x = 42").unwrap();
        match pattern {
            Pattern::Identifier(name) => assert_eq!(name, "x"),
            _ => panic!("Expected identifier pattern"),
        }
    }

    #[test]
    fn test_literal_patterns() {
        let pattern = parse_pattern("let 42 = x").unwrap();
        match pattern {
            Pattern::Literal(Literal::Integer(val)) => assert_eq!(val, 42),
            _ => panic!("Expected literal pattern"),
        }
    }

    #[test]
    fn test_tuple_pattern() {
        let pattern = parse_pattern("let (x, y) = point").unwrap();
        match pattern {
            Pattern::Tuple(patterns) => assert_eq!(patterns.len(), 2),
            _ => panic!("Expected tuple pattern"),
        }
    }

    #[test]
    fn test_list_pattern() {
        let pattern = parse_pattern("let [a, b, c] = list").unwrap();
        match pattern {
            Pattern::List(patterns) => assert_eq!(patterns.len(), 3),
            _ => panic!("Expected list pattern"),
        }
    }

    #[test]
    fn test_or_pattern() {
        // OR patterns in match expressions
        let input = r#"
            match x {
                1 | 2 | 3 => "small",
                _ => "other"
            }
        "#;
        let mut parser = Parser::new(input);
        let expr = parser.parse_expr().unwrap();
        assert!(matches!(expr.kind, ruchy::frontend::ast::ExprKind::Match { .. }));
    }
}

#[cfg(test)]
mod variables_tests {
    use ruchy::frontend::parser::Parser;
    use ruchy::frontend::ast::{Expr, ExprKind};
    use ruchy::frontend::parser::StmtKind;

    fn parse_expr(input: &str) -> Result<Expr, String> {
        let mut parser = Parser::new(input);
        parser.parse_expr().map_err(|e| e.to_string())
    }

    #[test]
    fn test_parse_let() {
        let expr = parse_expr("let x = 42").unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                assert!(matches!(stmt.kind, StmtKind::Let { .. }));
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_parse_let_mut() {
        let expr = parse_expr("let mut x = 42").unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                match stmt.kind {
                    StmtKind::Let { is_mut, .. } => assert!(is_mut),
                    _ => panic!("Expected let statement"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_parse_let_with_type() {
        let expr = parse_expr("let x: i32 = 42").unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                match stmt.kind {
                    StmtKind::Let { type_annotation, .. } => {
                        assert!(type_annotation.is_some());
                    }
                    _ => panic!("Expected let statement"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_parse_var() {
        let expr = parse_expr("var x = 42").unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                match stmt.kind {
                    StmtKind::Let { is_mut, .. } => assert!(is_mut),
                    _ => panic!("Expected var statement"),
                }
            }
            _ => panic!("Expected var statement"),
        }
    }

    #[test]
    fn test_parse_assignment() {
        let expr = parse_expr("x = 42").unwrap();
        assert!(matches!(expr.kind, ExprKind::Assign { .. }));
    }

    #[test]
    fn test_parse_augmented_assignment() {
        let expr = parse_expr("x += 1").unwrap();
        assert!(matches!(expr.kind, ExprKind::AugAssign { .. }));
    }
}

#[cfg(test)]
mod function_tests {
    use ruchy::frontend::parser::Parser;
    use ruchy::frontend::ast::{Expr, ExprKind};
    use ruchy::frontend::parser::StmtKind;

    fn parse_expr(input: &str) -> Result<Expr, String> {
        let mut parser = Parser::new(input);
        parser.parse_expr().map_err(|e| e.to_string())
    }

    #[test]
    fn test_parse_function() {
        let expr = parse_expr("fn add(x: i32, y: i32) -> i32 { x + y }").unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                assert!(matches!(stmt.kind, StmtKind::Function { .. }));
            }
            _ => panic!("Expected function statement"),
        }
    }

    #[test]
    fn test_parse_async_function() {
        let expr = parse_expr("async fn fetch() { await get_data() }").unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                match stmt.kind {
                    StmtKind::Function { is_async, .. } => assert!(is_async),
                    _ => panic!("Expected async function"),
                }
            }
            _ => panic!("Expected function statement"),
        }
    }

    #[test]
    fn test_parse_lambda() {
        let expr = parse_expr("|x| x + 1").unwrap();
        assert!(matches!(expr.kind, ExprKind::Lambda { .. }));

        let expr = parse_expr("|x, y| x + y").unwrap();
        assert!(matches!(expr.kind, ExprKind::Lambda { .. }));

        let expr = parse_expr("|| 42").unwrap();
        assert!(matches!(expr.kind, ExprKind::Lambda { .. }));
    }

    #[test]
    fn test_parse_lambda_with_arrow() {
        let expr = parse_expr("|x| -> x * 2").unwrap();
        assert!(matches!(expr.kind, ExprKind::Lambda { .. }));

        let expr = parse_expr("|x| => x * 2").unwrap();
        assert!(matches!(expr.kind, ExprKind::Lambda { .. }));
    }

    #[test]
    fn test_parse_generic_function() {
        let expr = parse_expr("fn identity<T>(x: T) -> T { x }").unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                match stmt.kind {
                    StmtKind::Function { generics, .. } => {
                        assert!(generics.is_some());
                    }
                    _ => panic!("Expected function with generics"),
                }
            }
            _ => panic!("Expected function statement"),
        }
    }
}

#[cfg(test)]
mod operators_tests {
    use ruchy::frontend::parser::Parser;
    use ruchy::frontend::ast::{Expr, ExprKind, BinaryOp, UnaryOp};

    fn parse_expr(input: &str) -> Result<Expr, String> {
        let mut parser = Parser::new(input);
        parser.parse_expr().map_err(|e| e.to_string())
    }

    #[test]
    fn test_parse_binary_operators() {
        let expr = parse_expr("1 + 2").unwrap();
        assert!(matches!(expr.kind, ExprKind::Binary { op: BinaryOp::Add, .. }));
        
        let expr = parse_expr("3 * 4").unwrap();
        assert!(matches!(expr.kind, ExprKind::Binary { op: BinaryOp::Mul, .. }));
        
        let expr = parse_expr("5 - 6").unwrap();
        assert!(matches!(expr.kind, ExprKind::Binary { op: BinaryOp::Sub, .. }));
    }

    #[test]
    fn test_parse_comparison_operators() {
        let expr = parse_expr("x > 5").unwrap();
        assert!(matches!(expr.kind, ExprKind::Binary { op: BinaryOp::Gt, .. }));
        
        let expr = parse_expr("y <= 10").unwrap();
        assert!(matches!(expr.kind, ExprKind::Binary { op: BinaryOp::LtEq, .. }));
        
        let expr = parse_expr("z == 0").unwrap();
        assert!(matches!(expr.kind, ExprKind::Binary { op: BinaryOp::Eq, .. }));
    }

    #[test]
    fn test_parse_logical_operators() {
        let expr = parse_expr("true && false").unwrap();
        assert!(matches!(expr.kind, ExprKind::Binary { op: BinaryOp::And, .. }));
        
        let expr = parse_expr("x || y").unwrap();
        assert!(matches!(expr.kind, ExprKind::Binary { op: BinaryOp::Or, .. }));
    }

    #[test]
    fn test_parse_unary_operators() {
        let expr = parse_expr("-42").unwrap();
        assert!(matches!(expr.kind, ExprKind::Unary { op: UnaryOp::Neg, .. }));
        
        let expr = parse_expr("!true").unwrap();
        assert!(matches!(expr.kind, ExprKind::Unary { op: UnaryOp::Not, .. }));
    }

    #[test]
    fn test_parse_range_operator() {
        let expr = parse_expr("0..10").unwrap();
        assert!(matches!(expr.kind, ExprKind::Range { inclusive: false, .. }));
        
        let expr = parse_expr("1..=5").unwrap();
        assert!(matches!(expr.kind, ExprKind::Range { inclusive: true, .. }));
    }

    #[test]
    fn test_parse_pipeline_operator() {
        let expr = parse_expr("x |> print").unwrap();
        assert!(matches!(expr.kind, ExprKind::Pipeline { .. }));
    }

    #[test]
    fn test_parse_in_operator() {
        let expr = parse_expr("x in [1, 2, 3]").unwrap();
        assert!(matches!(expr.kind, ExprKind::Binary { op: BinaryOp::In, .. }));
    }

    #[test]
    fn test_parse_is_operator() {
        let expr = parse_expr("x is None").unwrap();
        assert!(matches!(expr.kind, ExprKind::Binary { op: BinaryOp::Is, .. }));
        
        let expr = parse_expr("y is not None").unwrap();
        assert!(matches!(expr.kind, ExprKind::Binary { op: BinaryOp::IsNot, .. }));
    }

    #[test]
    fn test_parse_as_operator() {
        let expr = parse_expr("x as i32").unwrap();
        assert!(matches!(expr.kind, ExprKind::Cast { .. }));
    }
}

#[cfg(test)]
mod collections_tests {
    use ruchy::frontend::parser::Parser;
    use ruchy::frontend::ast::{Expr, ExprKind};

    fn parse_expr(input: &str) -> Result<Expr, String> {
        let mut parser = Parser::new(input);
        parser.parse_expr().map_err(|e| e.to_string())
    }

    #[test]
    fn test_parse_list_literal() {
        let expr = parse_expr("[1, 2, 3]").unwrap();
        match expr.kind {
            ExprKind::List { ref elements } => assert_eq!(elements.len(), 3),
            _ => panic!("Expected list literal"),
        }
    }

    #[test]
    fn test_parse_empty_list() {
        let expr = parse_expr("[]").unwrap();
        match expr.kind {
            ExprKind::List { ref elements } => assert_eq!(elements.len(), 0),
            _ => panic!("Expected empty list"),
        }
    }

    #[test]
    fn test_parse_list_comprehension() {
        let expr = parse_expr("[x * 2 for x in range(10)]").unwrap();
        assert!(matches!(expr.kind, ExprKind::ListComp { .. }));
        
        let expr = parse_expr("[x for x in items if x > 0]").unwrap();
        assert!(matches!(expr.kind, ExprKind::ListComp { .. }));
    }

    #[test]
    fn test_parse_tuple_literal() {
        let expr = parse_expr("(1, 2, 3)").unwrap();
        match expr.kind {
            ExprKind::Tuple { ref elements } => assert_eq!(elements.len(), 3),
            _ => panic!("Expected tuple literal"),
        }
    }

    #[test]
    fn test_parse_unit_tuple() {
        let expr = parse_expr("()").unwrap();
        match expr.kind {
            ExprKind::Tuple { ref elements } => assert_eq!(elements.len(), 0),
            _ => panic!("Expected unit tuple"),
        }
    }

    #[test]
    fn test_parse_set_literal() {
        let expr = parse_expr("{1, 2, 3}").unwrap();
        match expr.kind {
            ExprKind::Set { ref elements } => assert_eq!(elements.len(), 3),
            _ => panic!("Expected set literal"),
        }
    }

    #[test]
    fn test_parse_dict_literal() {
        let expr = parse_expr(r#"{"a": 1, "b": 2}"#).unwrap();
        match expr.kind {
            ExprKind::Dict { ref keys, ref values } => {
                assert_eq!(keys.len(), 2);
                assert_eq!(values.len(), 2);
            }
            _ => panic!("Expected dict literal"),
        }
    }

    #[test]
    fn test_parse_empty_dict() {
        let expr = parse_expr("{}").unwrap();
        match expr.kind {
            ExprKind::Dict { ref keys, ref values } => {
                assert_eq!(keys.len(), 0);
                assert_eq!(values.len(), 0);
            }
            _ => panic!("Expected empty dict"),
        }
    }

    #[test]
    fn test_parse_set_comprehension() {
        let expr = parse_expr("{x * 2 for x in range(10)}").unwrap();
        assert!(matches!(expr.kind, ExprKind::SetComp { .. }));
    }

    #[test]
    fn test_parse_dict_comprehension() {
        let expr = parse_expr("{x: x * 2 for x in range(10)}").unwrap();
        assert!(matches!(expr.kind, ExprKind::DictComp { .. }));
    }
}

#[cfg(test)]
mod data_structures_tests {
    use ruchy::frontend::parser::Parser;
    use ruchy::frontend::ast::{Expr, ExprKind};
    use ruchy::frontend::parser::StmtKind;

    fn parse_expr(input: &str) -> Result<Expr, String> {
        let mut parser = Parser::new(input);
        parser.parse_expr().map_err(|e| e.to_string())
    }

    #[test]
    fn test_parse_struct() {
        let expr = parse_expr("struct Point { x: i32, y: i32 }").unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                assert!(matches!(stmt.kind, StmtKind::Struct { .. }));
            }
            _ => panic!("Expected struct statement"),
        }
    }

    #[test]
    fn test_parse_tuple_struct() {
        let expr = parse_expr("struct Color(u8, u8, u8)").unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                assert!(matches!(stmt.kind, StmtKind::Struct { .. }));
            }
            _ => panic!("Expected tuple struct"),
        }
    }

    #[test]
    fn test_parse_enum() {
        let expr = parse_expr(r#"
            enum Option<T> {
                Some(T),
                None
            }
        "#).unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                assert!(matches!(stmt.kind, StmtKind::Enum { .. }));
            }
            _ => panic!("Expected enum statement"),
        }
    }

    #[test]
    fn test_parse_trait() {
        let expr = parse_expr(r#"
            trait Display {
                fn fmt(&self) -> String
            }
        "#).unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                assert!(matches!(stmt.kind, StmtKind::Trait { .. }));
            }
            _ => panic!("Expected trait statement"),
        }
    }

    #[test]
    fn test_parse_impl() {
        let expr = parse_expr(r#"
            impl Display for Point {
                fn fmt(&self) -> String {
                    format!("({}, {})", self.x, self.y)
                }
            }
        "#).unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                assert!(matches!(stmt.kind, StmtKind::Impl { .. }));
            }
            _ => panic!("Expected impl statement"),
        }
    }
}

#[cfg(test)]
mod imports_tests {
    use ruchy::frontend::parser::Parser;
    use ruchy::frontend::ast::{Expr, ExprKind};
    use ruchy::frontend::parser::StmtKind;

    fn parse_expr(input: &str) -> Result<Expr, String> {
        let mut parser = Parser::new(input);
        parser.parse_expr().map_err(|e| e.to_string())
    }

    #[test]
    fn test_parse_import() {
        let expr = parse_expr("import std.io").unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                assert!(matches!(stmt.kind, StmtKind::Import { .. }));
            }
            _ => panic!("Expected import statement"),
        }
    }

    #[test]
    fn test_parse_from_import() {
        let expr = parse_expr("from std.io import println").unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                assert!(matches!(stmt.kind, StmtKind::FromImport { .. }));
            }
            _ => panic!("Expected from-import statement"),
        }
    }

    #[test]
    fn test_parse_import_wildcard() {
        let expr = parse_expr("from math import *").unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                match stmt.kind {
                    StmtKind::FromImport { ref items, .. } => {
                        assert_eq!(items[0], "*");
                    }
                    _ => panic!("Expected from-import with wildcard"),
                }
            }
            _ => panic!("Expected from-import statement"),
        }
    }

    #[test]
    fn test_parse_import_alias() {
        let expr = parse_expr("import numpy as np").unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                match stmt.kind {
                    StmtKind::Import { ref alias, .. } => {
                        assert_eq!(alias.as_ref().unwrap(), "np");
                    }
                    _ => panic!("Expected import with alias"),
                }
            }
            _ => panic!("Expected import statement"),
        }
    }

    #[test]
    fn test_parse_export() {
        let expr = parse_expr("export { foo, bar }").unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                assert!(matches!(stmt.kind, StmtKind::Export { .. }));
            }
            _ => panic!("Expected export statement"),
        }
    }

    #[test]
    fn test_parse_use_statement() {
        let expr = parse_expr("use std::io::prelude::*;").unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                assert!(matches!(stmt.kind, StmtKind::Use { .. }));
            }
            _ => panic!("Expected use statement"),
        }
    }
}

#[cfg(test)]
mod actors_tests {
    use ruchy::frontend::parser::Parser;
    use ruchy::frontend::ast::{Expr, ExprKind};
    use ruchy::frontend::parser::StmtKind;

    fn parse_expr(input: &str) -> Result<Expr, String> {
        let mut parser = Parser::new(input);
        parser.parse_expr().map_err(|e| e.to_string())
    }

    #[test]
    fn test_parse_actor() {
        let expr = parse_expr(r#"
            actor Counter {
                receive {
                    Inc => self.count += 1,
                    Get => reply(self.count)
                }
            }
        "#).unwrap();
        match expr.kind {
            ExprKind::Statement(stmt) => {
                assert!(matches!(stmt.kind, StmtKind::Actor { .. }));
            }
            _ => panic!("Expected actor statement"),
        }
    }

    #[test]
    fn test_parse_spawn() {
        let expr = parse_expr("spawn Counter()").unwrap();
        assert!(matches!(expr.kind, ExprKind::Spawn { .. }));
    }

    #[test]
    fn test_parse_send() {
        let expr = parse_expr("actor ! message").unwrap();
        assert!(matches!(expr.kind, ExprKind::Send { .. }));
    }

    #[test]
    fn test_parse_receive() {
        let expr = parse_expr(r#"
            receive {
                Message(x) => handle(x)
            }
        "#).unwrap();
        assert!(matches!(expr.kind, ExprKind::Receive { .. }));
    }

    #[test]
    fn test_parse_select() {
        let expr = parse_expr(r#"
            select {
                receive Pattern1 => action1(),
                receive Pattern2 => action2(),
                default => timeout()
            }
        "#).unwrap();
        assert!(matches!(expr.kind, ExprKind::Select { .. }));
    }

    #[test]
    fn test_parse_async_block() {
        let expr = parse_expr("async { do_work().await }").unwrap();
        assert!(matches!(expr.kind, ExprKind::AsyncBlock { .. }));
    }

    #[test]
    fn test_parse_await() {
        let expr = parse_expr("future.await").unwrap();
        assert!(matches!(expr.kind, ExprKind::Await { .. }));
    }

    #[test]
    fn test_parse_yield() {
        let expr = parse_expr("yield 42").unwrap();
        assert!(matches!(expr.kind, ExprKind::Yield { .. }));
    }
}

#[cfg(test)]
mod integration_tests {
    use ruchy::frontend::parser::Parser;

    #[test]
    fn test_complex_expression() {
        let input = r#"
            let result = if x > 0 {
                for i in 0..x {
                    print(i)
                }
                x * 2
            } else {
                -x
            }
        "#;

        let mut parser = Parser::new(input);
        let expr = parser.parse_expr();
        assert!(expr.is_ok());
    }

    #[test]
    fn test_nested_match() {
        let input = r#"
            match x {
                Some(y) => match y {
                    0 => "zero",
                    _ => "non-zero"
                },
                None => "nothing"
            }
        "#;

        let mut parser = Parser::new(input);
        let expr = parser.parse_expr();
        assert!(expr.is_ok());
    }

    #[test]
    fn test_list_comprehension() {
        let input = "[x * 2 for x in 0..10 if x % 2 == 0]";
        let mut parser = Parser::new(input);
        let expr = parser.parse_expr();
        assert!(expr.is_ok());
    }

    #[test]
    fn test_function_with_patterns() {
        let input = r#"
            fn process((x, y): (i32, i32)) -> i32 {
                x + y
            }
        "#;

        let mut parser = Parser::new(input);
        let expr = parser.parse_expr();
        assert!(expr.is_ok());
    }
}