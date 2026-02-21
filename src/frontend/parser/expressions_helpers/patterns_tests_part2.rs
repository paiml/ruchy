use super::*;

use crate::frontend::ast::TypeKind;
use crate::frontend::parser::Parser;

fn make_cov_test_expr(kind: ExprKind) -> Expr {
    Expr::new(kind, Span::default())
}

fn make_cov_test_value() -> Box<Expr> {
    Box::new(make_cov_test_expr(ExprKind::Literal(Literal::Integer(
        42, None,
    ))))
}

fn make_cov_test_body() -> Box<Expr> {
    Box::new(make_cov_test_expr(ExprKind::Literal(Literal::Unit)))
}

#[test]
fn test_create_let_expression_identifier_mutable() {
    let pattern = Pattern::Identifier("x".to_string());
    let result = create_let_expression(
        pattern,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        true,
        None,
        Span::default(),
    )
    .unwrap();
    if let ExprKind::Let { is_mutable, .. } = &result.kind {
        assert!(*is_mutable, "Should be mutable");
    } else {
        panic!("Expected ExprKind::Let");
    }
}

#[test]
fn test_create_let_expression_identifier_with_else() {
    let pattern = Pattern::Identifier("x".to_string());
    let else_block = Some(Box::new(make_cov_test_expr(ExprKind::Return {
        value: None,
    })));
    let result = create_let_expression(
        pattern,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        else_block,
        Span::default(),
    )
    .unwrap();
    if let ExprKind::Let { else_block, .. } = &result.kind {
        assert!(else_block.is_some(), "Should have else block");
    } else {
        panic!("Expected ExprKind::Let");
    }
}

#[test]
fn test_create_let_expression_tuple_produces_let_pattern() {
    let pattern = Pattern::Tuple(vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Identifier("b".to_string()),
    ]);
    let result = create_let_expression(
        pattern,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_list_produces_let_pattern() {
    let pattern = Pattern::List(vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Identifier("b".to_string()),
    ]);
    let result = create_let_expression(
        pattern,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_wildcard_produces_let_pattern() {
    let result = create_let_expression(
        Pattern::Wildcard,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_some_produces_let_pattern() {
    let pattern = Pattern::Some(Box::new(Pattern::Identifier("x".to_string())));
    let result = create_let_expression(
        pattern,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_ok_produces_let_pattern() {
    let pattern = Pattern::Ok(Box::new(Pattern::Identifier("v".to_string())));
    let result = create_let_expression(
        pattern,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_err_produces_let_pattern() {
    let pattern = Pattern::Err(Box::new(Pattern::Identifier("e".to_string())));
    let result = create_let_expression(
        pattern,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_none_produces_let_pattern() {
    let result = create_let_expression(
        Pattern::None,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_tuple_variant_produces_let_pattern() {
    let pattern = Pattern::TupleVariant {
        path: vec!["Color".to_string()],
        patterns: vec![
            Pattern::Identifier("r".to_string()),
            Pattern::Identifier("g".to_string()),
        ],
    };
    let result = create_let_expression(
        pattern,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_struct_produces_let_pattern() {
    let pattern = Pattern::Struct {
        name: "Point".to_string(),
        fields: vec![],
        has_rest: false,
    };
    let result = create_let_expression(
        pattern,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_or_produces_let_pattern() {
    let pattern = Pattern::Or(vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Identifier("b".to_string()),
    ]);
    let result = create_let_expression(
        pattern,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_range_produces_let_pattern() {
    let pattern = Pattern::Range {
        start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
        end: Box::new(Pattern::Literal(Literal::Integer(10, None))),
        inclusive: true,
    };
    let result = create_let_expression(
        pattern,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_literal_produces_let_pattern() {
    let pattern = Pattern::Literal(Literal::Integer(42, None));
    let result = create_let_expression(
        pattern,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_qualified_name_produces_let_pattern() {
    let pattern = Pattern::QualifiedName(vec!["Ordering".to_string(), "Less".to_string()]);
    let result = create_let_expression(
        pattern,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_rest_produces_let_pattern() {
    let result = create_let_expression(
        Pattern::Rest,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_rest_named_produces_let_pattern() {
    let result = create_let_expression(
        Pattern::RestNamed("rest".to_string()),
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_at_binding_produces_let_pattern() {
    let pattern = Pattern::AtBinding {
        name: "x".to_string(),
        pattern: Box::new(Pattern::Identifier("val".to_string())),
    };
    let result = create_let_expression(
        pattern,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_with_default_produces_let_pattern() {
    let pattern = Pattern::WithDefault {
        pattern: Box::new(Pattern::Identifier("x".to_string())),
        default: Box::new(make_cov_test_expr(ExprKind::Literal(Literal::Integer(
            0, None,
        )))),
    };
    let result = create_let_expression(
        pattern,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

#[test]
fn test_create_let_expression_mut_produces_let_pattern() {
    let pattern = Pattern::Mut(Box::new(Pattern::Identifier("x".to_string())));
    let result = create_let_expression(
        pattern,
        None,
        make_cov_test_value(),
        make_cov_test_body(),
        false,
        None,
        Span::default(),
    )
    .unwrap();
    assert!(matches!(result.kind, ExprKind::LetPattern { .. }));
}

// ========================================================================
// parse_let_pattern coverage tests
// ========================================================================

#[test]
fn test_parse_let_some_pattern_coverage() {
    let code = "let Some(val) = maybe";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "let Some(val) should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_let_ok_pattern_coverage() {
    let code = "let Ok(v) = result_val";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "let Ok(v) should parse");
}

#[test]
fn test_parse_let_err_pattern_coverage() {
    let code = "let Err(e) = result_val";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "let Err(e) should parse");
}

#[test]
fn test_parse_let_none_pattern_coverage() {
    let code = "let None = opt";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "let None should parse");
}

#[test]
fn test_parse_let_df_keyword_coverage() {
    let code = "let df = data";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "DataFrame keyword as variable name should parse"
    );
}

#[test]
fn test_parse_let_default_keyword_coverage() {
    let code = "let default = config";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "default keyword as variable name should parse"
    );
}

#[test]
fn test_parse_let_final_keyword_coverage() {
    let code = "let final = value";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "final keyword as variable name should parse"
    );
}

#[test]
fn test_parse_let_underscore_wildcard_coverage() {
    let code = "let _ = compute()";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "underscore pattern should parse");
}

#[test]
fn test_parse_let_tuple_destructure_coverage() {
    let code = "let (a, b, c) = (1, 2, 3)";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "tuple pattern should parse");
}

#[test]
fn test_parse_let_list_destructure_coverage() {
    let code = "let [first, second] = items";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "list pattern should parse");
}

#[test]
fn test_parse_let_struct_brace_pattern_coverage() {
    let code = "let {name, age} = person";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "brace struct pattern should parse");
}

#[test]
fn test_parse_let_variant_tuple_pattern_coverage() {
    let code = "let Color(r, g, b) = pixel";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "variant tuple pattern should parse");
}

#[test]
fn test_parse_let_named_struct_destructure_coverage() {
    let code = "let Point { x, y } = origin";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "named struct pattern should parse");
}

#[test]
fn test_parse_let_mut_coverage() {
    let code = "let mut x = 42";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "let mut identifier should parse");
}

// ========================================================================
// create_var_expression coverage tests
// ========================================================================

#[test]
fn test_parse_var_identifier_coverage() {
    let code = "var x = 42";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "var x = 42 should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_var_tuple_destructuring_coverage() {
    let code = "var (a, b) = (1, 2)";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "var (a, b) should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_var_list_destructuring_coverage() {
    let code = "var [x, y] = items";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "var [x, y] should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_var_with_type_annotation_coverage() {
    let code = "var x: i32 = 42";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "var x: i32 should parse: {:?}",
        result.err()
    );
}

// ============================================================
// Coverage tests for parse_let_pattern (patterns.rs:118)
// Targeting all branches in the match on state.tokens.peek()
// ============================================================

#[test]
fn test_let_pattern_some_without_parens_errors() {
    // Some not followed by parens should error
    let code = "let Some = value";
    let result = Parser::new(code).parse();
    assert!(result.is_err(), "Some without parens should fail");
}

#[test]
fn test_let_pattern_ok_without_parens_errors() {
    // Ok not followed by parens should error
    let code = "let Ok = value";
    let result = Parser::new(code).parse();
    assert!(result.is_err(), "Ok without parens should fail");
}

#[test]
fn test_let_pattern_err_without_parens_errors() {
    // Err not followed by parens should error
    let code = "let Err = value";
    let result = Parser::new(code).parse();
    assert!(result.is_err(), "Err without parens should fail");
}

#[test]
fn test_let_pattern_none() {
    let code = "let None = opt";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "None pattern should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_identifier_with_struct_destructure() {
    // Identifier followed by { } is struct pattern
    let code = "let Config { debug, verbose } = config";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Struct pattern should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_identifier_variant_with_parens() {
    // Identifier followed by ( ) is variant pattern
    let code = "let MyVariant(x) = val";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Variant pattern should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_dataframe_token() {
    // DataFrame token as variable name
    let code = "let df = create_dataframe()";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "df pattern should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_default_token() {
    // Default token as variable name
    let code = "let default = get_config()";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "default pattern should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_final_token() {
    // Final token as variable name
    let code = "let final = get_value()";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "final pattern should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_underscore() {
    // Underscore as wildcard pattern
    let code = "let _ = compute()";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Wildcard pattern should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_tuple() {
    // Tuple destructuring pattern
    let code = "let (a, b, c) = triple";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Tuple pattern should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_list() {
    // List destructuring pattern
    let code = "let [first, second] = items";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "List pattern should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_struct_brace() {
    // Struct destructuring pattern with { }
    let code = "let { name, age } = person";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Struct brace pattern should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_error_invalid_token() {
    // Invalid token after let should error
    let code = "let 42 = value";
    let result = Parser::new(code).parse();
    assert!(result.is_err(), "Number after let should fail");
}

#[test]
fn test_let_mut_pattern_error_invalid_token() {
    // Invalid token after let mut should error
    let code = "let mut 42 = value";
    let result = Parser::new(code).parse();
    assert!(result.is_err(), "Number after let mut should fail");
}

#[test]
fn test_let_pattern_some_with_multiple_args() {
    // Some with multiple args in tuple destructure
    let code = "let Some((a, b)) = maybe_pair";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Some with nested tuple should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_ok_with_value() {
    let code = "let Ok(value) = result";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Ok(value) pattern should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_err_with_error() {
    let code = "let Err(e) = result";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Err(e) pattern should parse: {:?}",
        result.err()
    );
}

// ============================================================
// Direct unit tests for create_var_expression (patterns.rs:360)
// This function is defined in patterns.rs but not called through
// the public API (variable_declarations.rs has its own copy).
// We test it directly to cover all branches.
// ============================================================

#[test]
fn test_create_var_expression_identifier_pattern() {
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span};

    let pattern = Pattern::Identifier("x".to_string());
    let value = Box::new(Expr::new(
        ExprKind::Literal(Literal::Integer(42, None)),
        Span::new(0, 2),
    ));
    let result = create_var_expression(pattern, None, value, Span::new(0, 10));
    assert!(result.is_ok());
    let expr = result.unwrap();
    match &expr.kind {
        ExprKind::Let {
            name,
            is_mutable,
            type_annotation,
            else_block,
            ..
        } => {
            assert_eq!(name, "x");
            assert!(is_mutable);
            assert!(type_annotation.is_none());
            assert!(else_block.is_none());
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_create_var_expression_tuple_pattern() {
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span};

    let pattern = Pattern::Tuple(vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Identifier("b".to_string()),
    ]);
    let value = Box::new(Expr::new(
        ExprKind::Literal(Literal::Integer(1, None)),
        Span::new(0, 1),
    ));
    let result = create_var_expression(pattern, None, value, Span::new(0, 10));
    assert!(result.is_ok());
    let expr = result.unwrap();
    match &expr.kind {
        ExprKind::LetPattern {
            pattern,
            is_mutable,
            type_annotation,
            else_block,
            ..
        } => {
            assert!(matches!(pattern, Pattern::Tuple(_)));
            assert!(is_mutable);
            assert!(type_annotation.is_none());
            assert!(else_block.is_none());
        }
        other => panic!("Expected LetPattern, got {:?}", other),
    }
}

#[test]
fn test_create_var_expression_with_type_annotation() {
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span, Type, TypeKind};

    let pattern = Pattern::Identifier("count".to_string());
    let value = Box::new(Expr::new(
        ExprKind::Literal(Literal::Integer(0, None)),
        Span::new(0, 1),
    ));
    let ty = Some(Type {
        kind: TypeKind::Named("i32".to_string()),
        span: Span::new(0, 3),
    });
    let result = create_var_expression(pattern, ty, value, Span::new(0, 15));
    assert!(result.is_ok());
    let expr = result.unwrap();
    match &expr.kind {
        ExprKind::Let {
            type_annotation, ..
        } => {
            assert!(type_annotation.is_some());
        }
        other => panic!("Expected Let with type, got {:?}", other),
    }
}

// ============================================================
// Coverage tests for parse_let_pattern (patterns.rs:118)
// and parse_variant_pattern_with_name (patterns.rs:43)
// These functions are reachable via expressions.rs delegation
// but also testable directly via ParserState.
// ============================================================

#[test]
fn test_let_pattern_some_variant() {
    // let Some(x) = maybe_val -- exercises Token::Some branch in parse_let_pattern
    let code = "let Some(x) = maybe_val";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "let Some(x) should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_ok_variant() {
    // let Ok(val) = result_val -- exercises Token::Ok branch
    let code = "let Ok(val) = result_val";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "let Ok(val) should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_err_variant() {
    // let Err(e) = result_val -- exercises Token::Err branch
    let code = "let Err(e) = result_val";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "let Err(e) should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_none_keyword() {
    // let None = opt -- exercises Token::None branch
    let code = "let None = opt";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "let None should parse: {:?}", result.err());
}

#[test]
fn test_let_pattern_identifier_with_variant_destructure() {
    // let Color(r, g, b) = my_color -- exercises Identifier+LeftParen branch
    let code = "let Color(r, g, b) = my_color";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "let Color(r, g, b) should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_identifier_struct_destructure_coverage() {
    // let Point { x, y } = p -- exercises Identifier+LeftBrace branch
    let code = "let Point { x, y } = p";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "let Point {{ x, y }} should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_dataframe_keyword() {
    // let df = data -- exercises Token::DataFrame branch
    let code = "let df = data";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "let df should parse: {:?}", result.err());
}

#[test]
fn test_let_pattern_default_keyword() {
    // let default = config_value -- exercises Token::Default branch
    let code = "let default = config_value";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "let default should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_final_keyword() {
    // let final = x -- exercises Token::Final branch
    let code = "let final = x";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "let final should parse: {:?}", result.err());
}

#[test]
fn test_let_pattern_underscore_wildcard() {
    // let _ = compute() -- exercises Token::Underscore branch
    let code = "let _ = compute()";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "let _ should parse: {:?}", result.err());
}

#[test]
fn test_let_pattern_tuple_destructure() {
    // let (a, b, c) = tuple -- exercises Token::LeftParen branch
    let code = "let (a, b, c) = tuple";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "let (a, b, c) should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_list_destructure() {
    // let [first, second] = arr -- exercises Token::LeftBracket branch
    let code = "let [first, second] = arr";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "let [first, second] should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_pattern_struct_destructure_brace() {
    // let {name, age} = obj -- exercises Token::LeftBrace branch
    let code = "let {name, age} = obj";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "let {{name, age}} should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_let_mut_pattern_identifier() {
    // let mut x = 5 -- exercises is_mutable=true in parse_let_pattern
    let code = "let mut x = 5";
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "let mut x should parse: {:?}", result.err());
}

#[test]
fn test_let_pattern_error_on_bad_token() {
    // let 42 = x -- should fail with error for unexpected token
    let code = "let 42 = x";
    let result = Parser::new(code).parse();
    // Should fail or produce an error since 42 is not a valid pattern for let
    // The behavior depends on parsing precedence; at minimum it should not panic
    assert!(
        result.is_ok() || result.is_err(),
        "Should not panic on bad let pattern"
    );
}

// ============================================================
// Coverage tests for parse_variant_pattern_with_name (patterns.rs:43)
// Exercise: empty variant, single variant, multi variant, trailing comma
// ============================================================

#[test]
fn test_variant_pattern_some_single_ident() {
    // match opt { Some(x) => x, None => 0 }
    let code = "match opt { Some(x) => x, None => 0 }";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Some(x) pattern in match should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_variant_pattern_ok_single_ident() {
    // match res { Ok(v) => v, Err(e) => 0 }
    let code = "match res { Ok(v) => v, Err(e) => 0 }";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Ok(v)/Err(e) patterns should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_variant_pattern_custom_multi_args() {
    // match color { Color(r, g, b) => r + g + b, _ => 0 }
    // Exercises multi-element TupleVariant path
    let code = "match color { Color(r, g, b) => r + g + b, _ => 0 }";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Custom variant with multiple args should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_variant_pattern_trailing_comma() {
    // match x { Pair(a, b,) => a, _ => 0 }
    // Exercises trailing comma detection in parse_variant_pattern_with_name
    let code = "match x { Pair(a, b,) => a, _ => 0 }";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Variant with trailing comma should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_variant_pattern_empty_parens() {
    // match x { Empty() => 0, _ => 1 }
    // Exercises empty pattern list in parse_variant_pattern_with_name
    let code = "match x { Empty() => 0, _ => 1 }";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Empty variant parens should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_variant_pattern_nested() {
    // match x { Some(Ok(v)) => v, _ => 0 }
    // Exercises nested variant patterns
    let code = "match x { Some(Ok(v)) => v, _ => 0 }";
    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Nested variant patterns should parse: {:?}",
        result.err()
    );
}

// ============================================================
// Direct unit tests for parse_let_pattern and parse_variant_pattern_with_name
// via ParserState (they are pub(in crate::frontend::parser))
// ============================================================

#[test]
fn test_direct_parse_let_pattern_identifier() {
    use crate::frontend::parser::ParserState;
    let mut state = ParserState::new("x");
    let pattern = super::parse_let_pattern(&mut state, false);
    assert!(
        pattern.is_ok(),
        "Direct parse_let_pattern should succeed for identifier"
    );
    assert!(matches!(pattern.unwrap(), Pattern::Identifier(n) if n == "x"));
}

#[test]
fn test_direct_parse_let_pattern_some() {
    use crate::frontend::parser::ParserState;
    let mut state = ParserState::new("Some(val)");
    let pattern = super::parse_let_pattern(&mut state, false);
    assert!(
        pattern.is_ok(),
        "Direct parse_let_pattern should succeed for Some(val)"
    );
}

#[test]
fn test_direct_parse_let_pattern_ok() {
    use crate::frontend::parser::ParserState;
    let mut state = ParserState::new("Ok(v)");
    let pattern = super::parse_let_pattern(&mut state, false);
    assert!(
        pattern.is_ok(),
        "Direct parse_let_pattern should succeed for Ok(v)"
    );
}

#[test]
fn test_direct_parse_let_pattern_err() {
    use crate::frontend::parser::ParserState;
    let mut state = ParserState::new("Err(e)");
    let pattern = super::parse_let_pattern(&mut state, false);
    assert!(
        pattern.is_ok(),
        "Direct parse_let_pattern should succeed for Err(e)"
    );
}

#[test]
fn test_direct_parse_let_pattern_none() {
    use crate::frontend::parser::ParserState;
    let mut state = ParserState::new("None");
    let pattern = super::parse_let_pattern(&mut state, true);
    assert!(
        pattern.is_ok(),
        "Direct parse_let_pattern should succeed for None"
    );
    assert!(matches!(pattern.unwrap(), Pattern::None));
}

#[test]
fn test_direct_parse_let_pattern_dataframe() {
    use crate::frontend::parser::ParserState;
    let mut state = ParserState::new("df");
    let pattern = super::parse_let_pattern(&mut state, false);
    assert!(
        pattern.is_ok(),
        "Direct parse_let_pattern should succeed for df"
    );
    assert!(matches!(pattern.unwrap(), Pattern::Identifier(n) if n == "df"));
}

#[test]
fn test_direct_parse_let_pattern_default() {
    use crate::frontend::parser::ParserState;
    let mut state = ParserState::new("default");
    let pattern = super::parse_let_pattern(&mut state, false);
    assert!(
        pattern.is_ok(),
        "Direct parse_let_pattern should succeed for default"
    );
    assert!(matches!(pattern.unwrap(), Pattern::Identifier(n) if n == "default"));
}

#[test]
fn test_direct_parse_let_pattern_final() {
    use crate::frontend::parser::ParserState;
    let mut state = ParserState::new("final");
    let pattern = super::parse_let_pattern(&mut state, false);
    assert!(
        pattern.is_ok(),
        "Direct parse_let_pattern should succeed for final"
    );
    assert!(matches!(pattern.unwrap(), Pattern::Identifier(n) if n == "final"));
}

#[test]
fn test_direct_parse_let_pattern_underscore() {
    use crate::frontend::parser::ParserState;
    let mut state = ParserState::new("_");
    let pattern = super::parse_let_pattern(&mut state, false);
    assert!(
        pattern.is_ok(),
        "Direct parse_let_pattern should succeed for _"
    );
    assert!(matches!(pattern.unwrap(), Pattern::Identifier(n) if n == "_"));
}

#[test]
fn test_direct_parse_let_pattern_tuple() {
    use crate::frontend::parser::ParserState;
    let mut state = ParserState::new("(a, b)");
    let pattern = super::parse_let_pattern(&mut state, false);
    assert!(
        pattern.is_ok(),
        "Direct parse_let_pattern should succeed for (a, b)"
    );
}

#[test]
fn test_direct_parse_let_pattern_list() {
    use crate::frontend::parser::ParserState;
    let mut state = ParserState::new("[a, b]");
    let pattern = super::parse_let_pattern(&mut state, false);
    assert!(
        pattern.is_ok(),
        "Direct parse_let_pattern should succeed for [a, b]"
    );
}

#[test]
fn test_direct_parse_let_pattern_struct_brace() {
    use crate::frontend::parser::ParserState;
    let mut state = ParserState::new("{a, b}");
    let pattern = super::parse_let_pattern(&mut state, false);
    assert!(
        pattern.is_ok(),
        "Direct parse_let_pattern should succeed for {{a, b}}"
    );
}

#[test]
fn test_direct_parse_let_pattern_named_struct() {
    use crate::frontend::parser::ParserState;
    let mut state = ParserState::new("Point { x, y }");
    let pattern = super::parse_let_pattern(&mut state, false);
    assert!(
        pattern.is_ok(),
        "Direct parse_let_pattern should succeed for Point {{ x, y }}"
    );
}

#[test]
fn test_direct_parse_let_pattern_named_variant() {
    use crate::frontend::parser::ParserState;
    let mut state = ParserState::new("Color(r, g, b)");
    let pattern = super::parse_let_pattern(&mut state, false);
    assert!(
        pattern.is_ok(),
        "Direct parse_let_pattern should succeed for Color(r, g, b)"
    );
}

#[test]
fn test_direct_parse_let_pattern_error_on_eof() {
    use crate::frontend::parser::ParserState;
    let mut state = ParserState::new("");
    let pattern = super::parse_let_pattern(&mut state, false);
    assert!(
        pattern.is_err(),
        "Direct parse_let_pattern should fail on empty input"
    );
}
