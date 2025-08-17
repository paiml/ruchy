//! Edge case tests to improve coverage

use anyhow::Result;
use ruchy::{compile, get_parse_error, is_valid_syntax, Parser};

#[test]
fn test_empty_list() -> Result<()> {
    let mut parser = Parser::new("[]");
    let ast = parser.parse()?;
    assert!(matches!(ast.kind, ruchy::ExprKind::List(ref v) if v.is_empty()));
    Ok(())
}

#[test]
fn test_nested_lists() -> Result<()> {
    let mut parser = Parser::new("[[1, 2], [3, 4]]");
    let ast = parser.parse()?;
    assert!(matches!(ast.kind, ruchy::ExprKind::List(_)));
    Ok(())
}

#[test]
fn test_deeply_nested_expressions() -> Result<()> {
    let mut parser = Parser::new("((((1))))");
    let ast = parser.parse()?;
    assert!(matches!(ast.kind, ruchy::ExprKind::Literal(_)));
    Ok(())
}

#[test]
fn test_complex_precedence() -> Result<()> {
    let mut parser = Parser::new("1 + 2 * 3 - 4 / 2");
    let ast = parser.parse()?;
    assert!(matches!(ast.kind, ruchy::ExprKind::Binary { .. }));
    Ok(())
}

#[test]
fn test_chained_comparisons() -> Result<()> {
    let mut parser = Parser::new("a < b && b < c");
    let ast = parser.parse()?;
    assert!(matches!(ast.kind, ruchy::ExprKind::Binary { .. }));
    Ok(())
}

#[test]
fn test_multiple_let_bindings() -> Result<()> {
    let mut parser = Parser::new("let x = 1 in let y = 2 in x + y");
    let ast = parser.parse()?;
    assert!(matches!(ast.kind, ruchy::ExprKind::Let { .. }));
    Ok(())
}

#[test]
fn test_empty_block() -> Result<()> {
    let mut parser = Parser::new("{ }");
    let ast = parser.parse()?;
    // Empty block should be unit literal
    assert!(matches!(
        ast.kind,
        ruchy::ExprKind::Literal(ruchy::Literal::Unit)
    ));
    Ok(())
}

#[test]
fn test_single_expression_block() -> Result<()> {
    let mut parser = Parser::new("{ 42 }");
    let ast = parser.parse()?;
    // Single expression block should unwrap to the expression
    assert!(matches!(ast.kind, ruchy::ExprKind::Literal(_)));
    Ok(())
}

#[test]
fn test_multi_expression_block() -> Result<()> {
    let mut parser = Parser::new("{ 1; 2; 3 }");
    let ast = parser.parse()?;
    assert!(matches!(ast.kind, ruchy::ExprKind::Block(_)));
    Ok(())
}

#[test]
fn test_if_without_else() -> Result<()> {
    let mut parser = Parser::new("if true { 42 }");
    let ast = parser.parse()?;
    match ast.kind {
        ruchy::ExprKind::If { else_branch, .. } => {
            assert!(else_branch.is_none());
        }
        _ => panic!("Expected if expression"),
    }
    Ok(())
}

#[test]
fn test_if_else_if_chain() -> Result<()> {
    let mut parser = Parser::new("if x { 1 } else if y { 2 } else { 3 }");
    let ast = parser.parse()?;
    assert!(matches!(ast.kind, ruchy::ExprKind::If { .. }));
    Ok(())
}

#[test]
fn test_match_single_arm() -> Result<()> {
    let mut parser = Parser::new("match x { _ => 42 }");
    let ast = parser.parse()?;
    match ast.kind {
        ruchy::ExprKind::Match { arms, .. } => {
            assert_eq!(arms.len(), 1);
        }
        _ => panic!("Expected match expression"),
    }
    Ok(())
}

#[test]
fn test_function_no_params() -> Result<()> {
    let mut parser = Parser::new("fun test() { 42 }");
    let ast = parser.parse()?;
    match ast.kind {
        ruchy::ExprKind::Function { params, .. } => {
            assert!(params.is_empty());
        }
        _ => panic!("Expected function"),
    }
    Ok(())
}

#[test]
fn test_function_single_param() -> Result<()> {
    let mut parser = Parser::new("fun inc(x: i32) -> i32 { x + 1 }");
    let ast = parser.parse()?;
    match ast.kind {
        ruchy::ExprKind::Function { params, .. } => {
            assert_eq!(params.len(), 1);
        }
        _ => panic!("Expected function"),
    }
    Ok(())
}

#[test]
fn test_lambda_no_params() -> Result<()> {
    let mut parser = Parser::new("fun () { 42 }");
    let ast = parser.parse()?;
    assert!(matches!(ast.kind, ruchy::ExprKind::Lambda { .. }));
    Ok(())
}

#[test]
fn test_call_no_args() -> Result<()> {
    let mut parser = Parser::new("func()");
    let ast = parser.parse()?;
    match ast.kind {
        ruchy::ExprKind::Call { args, .. } => {
            assert!(args.is_empty());
        }
        _ => panic!("Expected call expression"),
    }
    Ok(())
}

#[test]
fn test_call_single_arg() -> Result<()> {
    let mut parser = Parser::new("func(42)");
    let ast = parser.parse()?;
    match ast.kind {
        ruchy::ExprKind::Call { args, .. } => {
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected call expression"),
    }
    Ok(())
}

#[test]
fn test_struct_empty() -> Result<()> {
    let mut parser = Parser::new("struct Empty { }");
    let ast = parser.parse()?;
    match ast.kind {
        ruchy::ExprKind::Struct { fields, .. } => {
            assert!(fields.is_empty());
        }
        _ => panic!("Expected struct"),
    }
    Ok(())
}

#[test]
fn test_struct_single_field() -> Result<()> {
    let mut parser = Parser::new("struct Wrapper { value: i32 }");
    let ast = parser.parse()?;
    match ast.kind {
        ruchy::ExprKind::Struct { fields, .. } => {
            assert_eq!(fields.len(), 1);
        }
        _ => panic!("Expected struct"),
    }
    Ok(())
}

#[test]
fn test_pattern_wildcard() -> Result<()> {
    let mut parser = Parser::new("match x { _ => 0 }");
    let ast = parser.parse()?;
    match ast.kind {
        ruchy::ExprKind::Match { arms, .. } => {
            assert!(matches!(arms[0].pattern, ruchy::Pattern::Wildcard));
        }
        _ => panic!("Expected match"),
    }
    Ok(())
}

#[test]
fn test_pattern_literal() -> Result<()> {
    let mut parser = Parser::new("match x { 42 => \"found\" }");
    let ast = parser.parse()?;
    match ast.kind {
        ruchy::ExprKind::Match { arms, .. } => {
            assert!(matches!(arms[0].pattern, ruchy::Pattern::Literal(_)));
        }
        _ => panic!("Expected match"),
    }
    Ok(())
}

#[test]
fn test_pattern_identifier() -> Result<()> {
    let mut parser = Parser::new("match x { y => y }");
    let ast = parser.parse()?;
    match ast.kind {
        ruchy::ExprKind::Match { arms, .. } => {
            assert!(matches!(arms[0].pattern, ruchy::Pattern::Identifier(_)));
        }
        _ => panic!("Expected match"),
    }
    Ok(())
}

#[test]
fn test_list_with_trailing_comma() -> Result<()> {
    let mut parser = Parser::new("[1, 2, 3,]");
    let ast = parser.parse()?;
    match ast.kind {
        ruchy::ExprKind::List(ref items) => {
            assert_eq!(items.len(), 3);
        }
        _ => panic!("Expected list"),
    }
    Ok(())
}

#[test]
fn test_compile_valid_program() {
    let result = compile("let x = 42 in x");
    assert!(result.is_ok());
}

#[test]
fn test_is_valid_syntax_edge_cases() {
    assert!(is_valid_syntax("42"));
    assert!(is_valid_syntax("()"));
    assert!(is_valid_syntax("{ }"));
    assert!(!is_valid_syntax(""));
    assert!(!is_valid_syntax("   "));
}

#[test]
fn test_get_parse_error_cases() {
    assert!(get_parse_error("let x =").is_some());
    assert!(get_parse_error("if").is_some());
    assert!(get_parse_error("42").is_none());
}

#[test]
fn test_parse_all_literal_types() -> Result<()> {
    let literals = vec![
        ("42", ruchy::Literal::Integer(42)),
        ("3.14", ruchy::Literal::Float(3.14)),
        ("true", ruchy::Literal::Bool(true)),
        ("false", ruchy::Literal::Bool(false)),
    ];

    for (input, expected) in literals {
        let mut parser = Parser::new(input);
        let ast = parser.parse()?;
        match ast.kind {
            ruchy::ExprKind::Literal(ref lit) => {
                assert_eq!(*lit, expected, "Failed for input: {}", input);
            }
            _ => panic!("Expected literal for input: {}", input),
        }
    }

    Ok(())
}

#[test]
fn test_parse_all_binary_operators() -> Result<()> {
    let operators = vec![
        "+", "-", "*", "/", "%", "**", "==", "!=", "<", "<=", ">", ">=", "&&", "||", "&", "|", "^",
        "<<", ">>",
    ];

    for op in operators {
        let input = format!("a {} b", op);
        let mut parser = Parser::new(&input);
        let ast = parser.parse()?;
        assert!(
            matches!(ast.kind, ruchy::ExprKind::Binary { .. }),
            "Failed for operator: {}",
            op
        );
    }

    Ok(())
}

#[test]
fn test_parse_all_unary_operators() -> Result<()> {
    let operators = vec!["-", "!"];

    for op in operators {
        let input = format!("{}x", op);
        let mut parser = Parser::new(&input);
        let ast = parser.parse()?;
        assert!(
            matches!(ast.kind, ruchy::ExprKind::Unary { .. }),
            "Failed for operator: {}",
            op
        );
    }

    Ok(())
}
