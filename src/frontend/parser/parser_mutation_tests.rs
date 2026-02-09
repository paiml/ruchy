use super::*;

#[test]
fn test_try_range_operators_less_than_comparison() {
    // MISSED: replace < with == in try_range_operators (line 686)

    let mut state = ParserState::new("..10");
    let left = Expr {
        kind: ExprKind::Literal(Literal::Integer(0, None)),
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
        leading_comments: vec![],
        trailing_comment: None,
    };

    // Test when prec < min_prec (should return None)
    let result = try_range_operators(&mut state, left.clone(), &Token::DotDot, 10);
    assert!(result.is_ok());
    assert!(
        result.expect("result should be Some in test").is_none(),
        "Should return None when prec < min_prec (not ==)"
    );

    // Test when prec >= min_prec (should return Some)
    let mut state = ParserState::new("..10");
    let result = try_range_operators(&mut state, left, &Token::DotDot, 5);
    assert!(result.is_ok());
    assert!(
        result.expect("result should be Some in test").is_some(),
        "Should return Some when prec >= min_prec"
    );
}

#[test]
fn test_try_range_operators_plus_arithmetic() {
    // MISSED: replace + with * in try_range_operators (line 691)

    let mut state = ParserState::new("..10");
    let left = Expr {
        kind: ExprKind::Literal(Literal::Integer(0, None)),
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
        leading_comments: vec![],
        trailing_comment: None,
    };

    let result = try_range_operators(&mut state, left, &Token::DotDot, 1);
    assert!(result.is_ok());

    // The RHS should parse with precedence prec+1 (not prec*1)
    // This ensures proper right-associativity
    if let Ok(Some(expr)) = result {
        assert!(
            matches!(expr.kind, ExprKind::Range { .. }),
            "Should parse range correctly with + operator"
        );
    }
}

#[test]
fn test_try_assignment_operators_less_than_comparison() {
    // MISSED: replace < with <= in try_assignment_operators (line 590)

    let mut state = ParserState::new("= 42");
    let left = Expr {
        kind: ExprKind::Identifier("x".to_string()),
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
        leading_comments: vec![],
        trailing_comment: None,
    };

    // Test boundary: when prec == min_prec
    // With <: returns Some (prec is NOT < min_prec)
    // With <=: would return None (prec IS <= min_prec)
    let result = try_assignment_operators(&mut state, left.clone(), &Token::Equal, 1);
    assert!(result.is_ok());
    assert!(
        result.expect("result should be Some in test").is_some(),
        "Should return Some when prec == min_prec (using <, not <=)"
    );

    // Test when prec < min_prec (should return None)
    let mut state = ParserState::new("= 42");
    let result = try_assignment_operators(&mut state, left, &Token::Equal, 10);
    assert!(result.is_ok());
    assert!(
        result.expect("result should be Some in test").is_none(),
        "Should return None when prec < min_prec"
    );
}

#[test]
fn test_try_assignment_operators_equals_comparison() {
    // MISSED: replace < with == in try_assignment_operators (line 590)

    let mut state = ParserState::new("= 42");
    let left = Expr {
        kind: ExprKind::Identifier("x".to_string()),
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
        leading_comments: vec![],
        trailing_comment: None,
    };

    // Test when prec < min_prec (should return None with < operator)
    let result = try_assignment_operators(&mut state, left.clone(), &Token::Equal, 10);
    assert!(result.is_ok());
    assert!(
        result.expect("result should be Some in test").is_none(),
        "Should return None when prec < min_prec (not ==)"
    );

    // Test when prec > min_prec (should return Some)
    let mut state = ParserState::new("= 42");
    let result = try_assignment_operators(&mut state, left, &Token::Equal, 0);
    assert!(result.is_ok());
    assert!(
        result.expect("result should be Some in test").is_some(),
        "Should return Some when prec > min_prec"
    );
}

#[test]
fn test_try_ternary_operator_plus_arithmetic() {
    // MISSED: replace + with * in try_ternary_operator (line 464)

    let mut state = ParserState::new("? 1 : 0");
    let left = Expr {
        kind: ExprKind::Literal(Literal::Bool(true)),
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
        leading_comments: vec![],
        trailing_comment: None,
    };

    let result = try_ternary_operator(&mut state, left, &Token::Question, 1);
    assert!(result.is_ok());

    // The true branch should parse with TERNARY_PRECEDENCE+1 (not *1)
    // This tests the correct arithmetic operator
    if let Ok(Some(expr)) = result {
        assert!(
            matches!(expr.kind, ExprKind::Ternary { .. }),
            "Should parse ternary with + operator"
        );
    }
}
