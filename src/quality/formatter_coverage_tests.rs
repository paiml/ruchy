use super::*;
use crate::frontend::ast::*;

// ============================================================================
// Coverage tests for format_pattern (56 uncov lines, 18.8% coverage)
// ============================================================================

fn make_formatter() -> Formatter {
    Formatter::new()
}

fn make_default_span() -> Span {
    Span::new(0, 0)
}

#[test]
fn test_format_pattern_wildcard() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Wildcard);
    assert_eq!(result, "_");
}

#[test]
fn test_format_pattern_identifier() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Identifier("foo".to_string()));
    assert_eq!(result, "foo");
}

#[test]
fn test_format_pattern_qualified_name() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::QualifiedName(vec![
        "Ordering".to_string(),
        "Less".to_string(),
    ]));
    assert_eq!(result, "Ordering::Less");
}

#[test]
fn test_format_pattern_literal_integer() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Literal(Literal::Integer(42, None)));
    assert_eq!(result, "42");
}

#[test]
fn test_format_pattern_literal_string() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Literal(Literal::String("hello".to_string())));
    assert_eq!(result, "\"hello\"");
}

#[test]
fn test_format_pattern_literal_bool() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Literal(Literal::Bool(true)));
    assert_eq!(result, "true");
}

#[test]
fn test_format_pattern_tuple_empty() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Tuple(vec![]));
    assert_eq!(result, "()");
}

#[test]
fn test_format_pattern_tuple_single() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Tuple(vec![Pattern::Identifier("x".to_string())]));
    assert_eq!(result, "(x)");
}

#[test]
fn test_format_pattern_tuple_multiple() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Tuple(vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Identifier("b".to_string()),
        Pattern::Identifier("c".to_string()),
    ]));
    assert_eq!(result, "(a, b, c)");
}

#[test]
fn test_format_pattern_list_empty() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::List(vec![]));
    assert_eq!(result, "[]");
}

#[test]
fn test_format_pattern_list_with_elements() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::List(vec![
        Pattern::Literal(Literal::Integer(1, None)),
        Pattern::Literal(Literal::Integer(2, None)),
    ]));
    assert_eq!(result, "[1, 2]");
}

#[test]
fn test_format_pattern_struct_without_rest() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Struct {
        name: "Point".to_string(),
        fields: vec![
            StructPatternField {
                name: "x".to_string(),
                pattern: Some(Pattern::Identifier("a".to_string())),
            },
            StructPatternField {
                name: "y".to_string(),
                pattern: Some(Pattern::Identifier("b".to_string())),
            },
        ],
        has_rest: false,
    });
    assert_eq!(result, "Point { x: a, y: b }");
}

#[test]
fn test_format_pattern_struct_with_rest() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Struct {
        name: "Config".to_string(),
        fields: vec![StructPatternField {
            name: "name".to_string(),
            pattern: Some(Pattern::Identifier("n".to_string())),
        }],
        has_rest: true,
    });
    assert_eq!(result, "Config { name: n, .. }");
}

#[test]
fn test_format_pattern_struct_shorthand() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Struct {
        name: "Point".to_string(),
        fields: vec![
            StructPatternField {
                name: "x".to_string(),
                pattern: None, // shorthand syntax
            },
            StructPatternField {
                name: "y".to_string(),
                pattern: None,
            },
        ],
        has_rest: false,
    });
    assert_eq!(result, "Point { x, y }");
}

#[test]
fn test_format_pattern_tuple_variant() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::TupleVariant {
        path: vec!["Message".to_string(), "Text".to_string()],
        patterns: vec![Pattern::Identifier("content".to_string())],
    });
    assert_eq!(result, "Message::Text(content)");
}

#[test]
fn test_format_pattern_tuple_variant_multiple_args() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::TupleVariant {
        path: vec!["Result".to_string(), "Pair".to_string()],
        patterns: vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ],
    });
    assert_eq!(result, "Result::Pair(a, b)");
}

#[test]
fn test_format_pattern_range_inclusive() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Range {
        start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
        end: Box::new(Pattern::Literal(Literal::Integer(10, None))),
        inclusive: true,
    });
    assert_eq!(result, "1..=10");
}

#[test]
fn test_format_pattern_range_exclusive() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Range {
        start: Box::new(Pattern::Literal(Literal::Integer(0, None))),
        end: Box::new(Pattern::Literal(Literal::Integer(5, None))),
        inclusive: false,
    });
    assert_eq!(result, "0..5");
}

#[test]
fn test_format_pattern_or() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Or(vec![
        Pattern::Literal(Literal::Integer(1, None)),
        Pattern::Literal(Literal::Integer(2, None)),
        Pattern::Literal(Literal::Integer(3, None)),
    ]));
    assert_eq!(result, "1 | 2 | 3");
}

#[test]
fn test_format_pattern_rest() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Rest);
    assert_eq!(result, "..");
}

#[test]
fn test_format_pattern_rest_named() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::RestNamed("tail".to_string()));
    assert_eq!(result, "..tail");
}

#[test]
fn test_format_pattern_at_binding() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::AtBinding {
        name: "val".to_string(),
        pattern: Box::new(Pattern::Literal(Literal::Integer(42, None))),
    });
    assert_eq!(result, "val @ 42");
}

#[test]
fn test_format_pattern_with_default() {
    let f = make_formatter();
    let default_expr = Expr::new(
        ExprKind::Literal(Literal::Integer(10, None)),
        make_default_span(),
    );
    let result = f.format_pattern(&Pattern::WithDefault {
        pattern: Box::new(Pattern::Identifier("a".to_string())),
        default: Box::new(default_expr),
    });
    assert_eq!(result, "a = 10");
}

#[test]
fn test_format_pattern_mut() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Mut(Box::new(Pattern::Identifier(
        "x".to_string(),
    ))));
    assert_eq!(result, "mut x");
}

#[test]
fn test_format_pattern_ok() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Ok(Box::new(Pattern::Identifier(
        "val".to_string(),
    ))));
    assert_eq!(result, "Ok(val)");
}

#[test]
fn test_format_pattern_err() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Err(Box::new(Pattern::Identifier(
        "e".to_string(),
    ))));
    assert_eq!(result, "Err(e)");
}

#[test]
fn test_format_pattern_some() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Some(Box::new(Pattern::Identifier(
        "v".to_string(),
    ))));
    assert_eq!(result, "Some(v)");
}

#[test]
fn test_format_pattern_none() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::None);
    assert_eq!(result, "None");
}

#[test]
fn test_format_pattern_nested_tuple_in_list() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::List(vec![
        Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ]),
        Pattern::Wildcard,
    ]));
    assert_eq!(result, "[(a, b), _]");
}

#[test]
fn test_format_pattern_nested_ok_in_some() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Some(Box::new(Pattern::Ok(Box::new(
        Pattern::Identifier("val".to_string()),
    )))));
    assert_eq!(result, "Some(Ok(val))");
}

#[test]
fn test_format_pattern_literal_float() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Literal(Literal::Float(3.14)));
    assert_eq!(result, "3.14");
}

#[test]
fn test_format_pattern_literal_char() {
    let f = make_formatter();
    let result = f.format_pattern(&Pattern::Literal(Literal::Char('A')));
    assert_eq!(result, "'A'");
}

// ============================================================================
// Coverage tests for get_original_text (58 uncov lines, 0% coverage)
// ============================================================================

#[test]
fn test_get_original_text_no_source_returns_none() {
    let f = make_formatter();
    // Formatter without source set
    let expr = Expr::new(
        ExprKind::Literal(Literal::Integer(42, None)),
        Span::new(0, 2),
    );
    let result = f.get_original_text(&expr);
    assert!(result.is_none());
}

#[test]
fn test_get_original_text_simple_expression() {
    let mut f = make_formatter();
    let source = "let x = 42";
    f.set_source(source);

    let expr = Expr::new(
        ExprKind::Literal(Literal::Integer(42, None)),
        Span::new(0, 10),
    );
    let result = f.get_original_text(&expr);
    assert!(result.is_some());
    let text = result.expect("should have text");
    assert_eq!(text, "let x = 42");
}

#[test]
fn test_get_original_text_with_leading_comment() {
    let mut f = make_formatter();
    let source = "// comment\nlet x = 1";
    f.set_source(source);

    let comment = Comment::new(CommentKind::Line(" comment".to_string()), Span::new(0, 10));
    let mut expr = Expr::new(
        ExprKind::Literal(Literal::Integer(1, None)),
        Span::new(11, 20),
    );
    expr.leading_comments = vec![comment];

    let result = f.get_original_text(&expr);
    assert!(result.is_some());
    let text = result.expect("should have text");
    // Should include the comment since it starts from comment span
    assert!(text.contains("comment"));
}

#[test]
fn test_get_original_text_block_expression() {
    let mut f = make_formatter();
    let source = "{ 1 + 2 }";
    f.set_source(source);

    let inner = Expr::new(
        ExprKind::Literal(Literal::Integer(1, None)),
        Span::new(2, 3),
    );
    let expr = Expr::new(
        ExprKind::Block(vec![inner]),
        Span::new(0, 5), // Intentionally short span to test brace scanning
    );
    let result = f.get_original_text(&expr);
    assert!(result.is_some());
    let text = result.expect("should have text");
    // Should have scanned to find closing brace
    assert!(text.contains("{"));
    assert!(text.contains("}"));
}

#[test]
fn test_get_original_text_nested_braces() {
    let mut f = make_formatter();
    let source = "{ { inner } }";
    f.set_source(source);

    let inner = Expr::new(
        ExprKind::Literal(Literal::Integer(1, None)),
        Span::new(4, 9),
    );
    let expr = Expr::new(ExprKind::Block(vec![inner]), Span::new(0, 5));
    let result = f.get_original_text(&expr);
    assert!(result.is_some());
    let text = result.expect("should have text");
    // Should correctly match nested braces
    assert!(text.starts_with("{"));
    assert!(text.ends_with("}"));
}

#[test]
fn test_get_original_text_non_block_expression() {
    let mut f = make_formatter();
    let source = "x + y\nnext_line";
    f.set_source(source);

    let expr = Expr::new(ExprKind::Identifier("x".to_string()), Span::new(0, 1));
    let result = f.get_original_text(&expr);
    assert!(result.is_some());
    let text = result.expect("should have text");
    // Non-block should scan to end of line
    assert_eq!(text, "x + y");
}

#[test]
fn test_get_original_text_with_comment_lines_before_block() {
    let mut f = make_formatter();
    // Source with comment lines before a block
    let source = "// line comment\n\t{ body }";
    f.set_source(source);

    // Expression that starts at the comment line
    let comment = Comment::new(
        CommentKind::Line(" line comment".to_string()),
        Span::new(0, 15),
    );
    let mut expr = Expr::new(
        ExprKind::Block(vec![Expr::new(
            ExprKind::Identifier("body".to_string()),
            Span::new(18, 22),
        )]),
        Span::new(17, 22),
    );
    expr.leading_comments = vec![comment];

    let result = f.get_original_text(&expr);
    assert!(result.is_some());
}

#[test]
fn test_get_original_text_with_whitespace_before_block() {
    let mut f = make_formatter();
    let source = "   \t  { inner }";
    f.set_source(source);

    let expr = Expr::new(
        ExprKind::Block(vec![Expr::new(
            ExprKind::Identifier("inner".to_string()),
            Span::new(7, 12),
        )]),
        Span::new(0, 8),
    );
    let result = f.get_original_text(&expr);
    assert!(result.is_some());
    let text = result.expect("should have text");
    assert!(text.contains("{"));
    assert!(text.contains("}"));
}

#[test]
fn test_get_original_text_span_clamps_to_source_len() {
    let mut f = make_formatter();
    let source = "short";
    f.set_source(source);

    // Span beyond source length
    let expr = Expr::new(
        ExprKind::Literal(Literal::Integer(1, None)),
        Span::new(0, 100),
    );
    let result = f.get_original_text(&expr);
    assert!(result.is_some());
    let text = result.expect("should have text");
    assert_eq!(text, "short");
}

#[test]
fn test_get_original_text_empty_source() {
    let mut f = make_formatter();
    f.set_source("");

    let expr = Expr::new(
        ExprKind::Literal(Literal::Integer(1, None)),
        Span::new(0, 0),
    );
    let result = f.get_original_text(&expr);
    assert!(result.is_some());
    let text = result.expect("should have text");
    assert_eq!(text, "");
}
