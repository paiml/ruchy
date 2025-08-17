//! Additional parser tests to improve coverage

use anyhow::Result;
use ruchy::{ExprKind, Parser};

#[test]
fn test_parse_actor_system() -> Result<()> {
    let input = r#"
        actor Counter {
            state {
                count: i32
            }
            receive {
                Increment => self.count + 1,
                Get => self.count
            }
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    match &ast.kind {
        ExprKind::Actor {
            name,
            state,
            handlers,
        } => {
            assert_eq!(name, "Counter");
            assert_eq!(state.len(), 1);
            assert_eq!(handlers.len(), 2);
        }
        _ => panic!("Expected actor, got {:?}", ast.kind),
    }

    Ok(())
}

#[test]
fn test_parse_dataframe_operations() -> Result<()> {
    let input = "df.filter(col(\"age\") > 18).groupby(\"city\").mean()";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    // Should parse as method call chain
    assert!(matches!(ast.kind, ExprKind::MethodCall { .. }));

    Ok(())
}

#[test]
fn test_parse_impl_block() -> Result<()> {
    let input = r#"
        impl Point {
            fun distance(&self, other: Point) -> f64 {
                ((self.x - other.x).pow(2) + (self.y - other.y).pow(2)).sqrt()
            }
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    match &ast.kind {
        ExprKind::Impl { methods, .. } => {
            // Check methods
            assert_eq!(methods.len(), 1);
        }
        _ => panic!("Expected impl block, got {:?}", ast.kind),
    }

    Ok(())
}

#[test]
fn test_parse_trait_definition() -> Result<()> {
    let input = r#"
        trait Display {
            fun fmt(&self) -> String
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    match &ast.kind {
        ExprKind::Trait { name, methods, .. } => {
            assert_eq!(name, "Display");
            assert_eq!(methods.len(), 1);
        }
        _ => panic!("Expected trait, got {:?}", ast.kind),
    }

    Ok(())
}

#[test]
fn test_parse_generic_function() -> Result<()> {
    let input = "fun identity<T>(x: T) -> T { x }";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    match &ast.kind {
        ExprKind::Function {
            name, type_params, ..
        } => {
            assert_eq!(name, "identity");
            assert_eq!(type_params.len(), 1);
            assert_eq!(type_params[0], "T");
        }
        _ => panic!("Expected function, got {:?}", ast.kind),
    }

    Ok(())
}

#[test]
fn test_parse_generic_struct() -> Result<()> {
    let input = "struct Box<T> { value: T }";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    match &ast.kind {
        ExprKind::Struct {
            name,
            type_params,
            fields,
        } => {
            assert_eq!(name, "Box");
            assert_eq!(type_params.len(), 1);
            assert_eq!(type_params[0], "T");
            assert_eq!(fields.len(), 1);
        }
        _ => panic!("Expected struct, got {:?}", ast.kind),
    }

    Ok(())
}

#[test]
fn test_parse_list_comprehension() -> Result<()> {
    let input = "[x * 2 for x in range(10) if x % 2 == 0]";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    match &ast.kind {
        ExprKind::ListComprehension {
            element: _,
            variable: _,
            iterable: _,
            condition,
        } => {
            // Check condition exists
            assert!(condition.is_some());
        }
        _ => panic!("Expected list comprehension, got {:?}", ast.kind),
    }

    Ok(())
}

#[test]
fn test_parse_try_operator() -> Result<()> {
    let input = "file.read()?";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    assert!(matches!(ast.kind, ExprKind::Try { .. }));

    Ok(())
}

#[test]
fn test_parse_pipeline_operator() -> Result<()> {
    let input = "data |> filter(x > 5) |> map(x * 2)";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    assert!(matches!(ast.kind, ExprKind::Pipeline { .. }));

    Ok(())
}

#[test]
fn test_parse_string_interpolation() -> Result<()> {
    let input = r#""Hello, {name}! You are {age} years old.""#;

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    match &ast.kind {
        ExprKind::StringInterpolation { parts } => {
            assert!(parts.len() > 1);
        }
        _ => panic!("Expected string interpolation, got {:?}", ast.kind),
    }

    Ok(())
}

#[test]
fn test_parse_for_loop() -> Result<()> {
    let input = "for x in [1, 2, 3] { print(x) }";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    match &ast.kind {
        ExprKind::For { var, .. } => {
            assert_eq!(var, "x");
        }
        _ => panic!("Expected for loop, got {:?}", ast.kind),
    }

    Ok(())
}

#[test]
fn test_parse_while_loop() -> Result<()> {
    let input = "while x < 10 { x = x + 1 }";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    assert!(matches!(ast.kind, ExprKind::While { .. }));

    Ok(())
}

#[test]
fn test_parse_match_expression() -> Result<()> {
    let input = r#"
        match value {
            0 => "zero",
            1 => "one",
            _ => "many"
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    match &ast.kind {
        ExprKind::Match { arms, .. } => {
            assert_eq!(arms.len(), 3);
        }
        _ => panic!("Expected match expression, got {:?}", ast.kind),
    }

    Ok(())
}

#[test]
fn test_parse_struct_literal() -> Result<()> {
    let input = "Point { x: 10, y: 20 }";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    match &ast.kind {
        ExprKind::StructLiteral { name, fields } => {
            assert_eq!(name, "Point");
            assert_eq!(fields.len(), 2);
        }
        _ => panic!("Expected struct literal, got {:?}", ast.kind),
    }

    Ok(())
}

// Index operation not yet implemented in AST

#[test]
fn test_parse_attribute() -> Result<()> {
    let input = "#[test]\nfun test_foo() { assert(true) }";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    match &ast.kind {
        ExprKind::Function { .. } => {
            assert_eq!(ast.attributes.len(), 1);
            assert_eq!(ast.attributes[0].name, "test");
        }
        _ => panic!("Expected function with attribute, got {:?}", ast.kind),
    }

    Ok(())
}

#[test]
fn test_parse_async_function() -> Result<()> {
    let input = "async fun fetch(url: String) -> String { http.get(url).await }";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    match &ast.kind {
        ExprKind::Function { name, is_async, .. } => {
            assert_eq!(name, "fetch");
            assert!(*is_async);
        }
        _ => panic!("Expected async function, got {:?}", ast.kind),
    }

    Ok(())
}

#[test]
fn test_parse_await_expression() -> Result<()> {
    let input = "fetch(url).await";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    assert!(matches!(ast.kind, ExprKind::Await { .. }));

    Ok(())
}

#[test]
fn test_parse_import_statement() -> Result<()> {
    let input = "import std.collections.HashMap";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    match &ast.kind {
        ExprKind::Import { path, items } => {
            assert_eq!(path, "std.collections");
            assert!(!items.is_empty());
        }
        _ => panic!("Expected import, got {:?}", ast.kind),
    }

    Ok(())
}

#[test]
fn test_parse_col_function() -> Result<()> {
    let input = "col(\"name\")";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    match &ast.kind {
        ExprKind::Call { func, args } => {
            if let ExprKind::Identifier(name) = &func.kind {
                assert_eq!(name, "col");
                assert_eq!(args.len(), 1);
            } else {
                panic!("Expected col function call");
            }
        }
        _ => panic!("Expected function call, got {:?}", ast.kind),
    }

    Ok(())
}

#[test]
fn test_parse_send_operation() -> Result<()> {
    let input = "actor ! Message";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    assert!(matches!(ast.kind, ExprKind::Send { .. }));

    Ok(())
}

#[test]
fn test_parse_ask_operation() -> Result<()> {
    let input = "actor ? Request";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    assert!(matches!(ast.kind, ExprKind::Ask { .. }));

    Ok(())
}

// Bitwise operations test - simplified without BinaryOp enum
#[test]
fn test_parse_bitwise_operations() -> Result<()> {
    let cases = vec!["a & b", "a | b", "a ^ b", "a << 2", "a >> 2"];

    for input in cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse()?;

        // Just verify it parses as a binary operation
        assert!(
            matches!(ast.kind, ExprKind::Binary { .. }),
            "Failed for input: {}",
            input
        );
    }

    Ok(())
}

#[test]
fn test_parse_power_operator() -> Result<()> {
    let input = "2 ** 8";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    // Just verify it parses as a binary operation
    assert!(matches!(ast.kind, ExprKind::Binary { .. }));

    Ok(())
}

#[test]
fn test_parse_multiple_type_parameters() -> Result<()> {
    let input = "fun map<T, U>(list: List<T>, f: T -> U) -> List<U> { }";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    match &ast.kind {
        ExprKind::Function { type_params, .. } => {
            assert_eq!(type_params.len(), 2);
            assert_eq!(type_params[0], "T");
            assert_eq!(type_params[1], "U");
        }
        _ => panic!("Expected generic function, got {:?}", ast.kind),
    }

    Ok(())
}
