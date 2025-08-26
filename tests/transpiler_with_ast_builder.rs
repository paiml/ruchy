//! Transpiler tests using AST builder to bypass parser limitations
//! This demonstrates how to test transpiler features the parser doesn't support

#![allow(clippy::unwrap_used)]

// Use AstBuilder from testing module directly
use ruchy::Transpiler;
use ruchy::AstBuilder;
use ruchy::frontend::ast::{Pattern, BinaryOp, Literal};

/// Test pattern guards (parser doesn't support these)
#[test]
fn test_pattern_guards_with_builder() {
    let builder = AstBuilder::new();
    let transpiler = Transpiler::new();
    
    // Create: match x { n if n > 0 => "positive", n if n < 0 => "negative", _ => "zero" }
    let ast = builder.match_expr(
        builder.ident("x"),
        vec![
            // First arm: n if n > 0 => "positive"
            builder.match_arm(
                builder.pattern_ident("n"),
                Some(builder.binary(
                    builder.ident("n"),
                    BinaryOp::Greater,
                    builder.int(0),
                )),
                builder.string("positive"),
            ),
            // Second arm: n if n < 0 => "negative"
            builder.match_arm(
                builder.pattern_ident("n"),
                Some(builder.binary(
                    builder.ident("n"),
                    BinaryOp::Less,
                    builder.int(0),
                )),
                builder.string("negative"),
            ),
            // Default arm: _ => "zero"
            builder.match_arm(
                builder.pattern_wildcard(),
                None,
                builder.string("zero"),
            ),
        ],
    );
    
    // This should transpile successfully even though parser can't handle it
    let result = transpiler.transpile(&ast).unwrap();
    let code = result.to_string();
    
    // Verify the transpiled code contains expected elements
    assert!(code.contains("match"));
    assert!(code.contains("positive"));
    assert!(code.contains("negative"));
    assert!(code.contains("zero"));
}

/// Test or-patterns (parser doesn't support these)
#[test]
fn test_or_patterns_with_builder() {
    let builder = AstBuilder::new();
    let transpiler = Transpiler::new();
    
    // Create: match x { 1 | 2 | 3 => "small", 4 | 5 | 6 => "medium", _ => "large" }
    let ast = builder.match_expr(
        builder.ident("x"),
        vec![
            // First arm: 1 | 2 | 3 => "small"
            builder.match_arm(
                builder.pattern_or(vec![
                    builder.pattern_literal(Literal::Integer(1)),
                    builder.pattern_literal(Literal::Integer(2)),
                    builder.pattern_literal(Literal::Integer(3)),
                ]),
                None,
                builder.string("small"),
            ),
            // Second arm: 4 | 5 | 6 => "medium"
            builder.match_arm(
                builder.pattern_or(vec![
                    builder.pattern_literal(Literal::Integer(4)),
                    builder.pattern_literal(Literal::Integer(5)),
                    builder.pattern_literal(Literal::Integer(6)),
                ]),
                None,
                builder.string("medium"),
            ),
            // Default arm: _ => "large"
            builder.match_arm(
                builder.pattern_wildcard(),
                None,
                builder.string("large"),
            ),
        ],
    );
    
    let result = transpiler.transpile(&ast).unwrap();
    let code = result.to_string();
    
    assert!(code.contains("match"));
    assert!(code.contains("small"));
    assert!(code.contains("medium"));
    assert!(code.contains("large"));
}

/// Test rest patterns (parser doesn't support these)
#[test]
fn test_rest_patterns_with_builder() {
    let builder = AstBuilder::new();
    let transpiler = Transpiler::new();
    
    // Create: match list { [first, ..rest] => first, [] => 0 }
    let ast = builder.match_expr(
        builder.ident("list"),
        vec![
            // First arm: [first, ..rest] => first
            builder.match_arm(
                Pattern::List(vec![
                    builder.pattern_ident("first"),
                    builder.pattern_rest(),
                ]),
                None,
                builder.ident("first"),
            ),
            // Second arm: [] => 0
            builder.match_arm(
                Pattern::List(vec![]),
                None,
                builder.int(0),
            ),
        ],
    );
    
    let result = transpiler.transpile(&ast).unwrap();
    let code = result.to_string();
    
    assert!(code.contains("match"));
    assert!(code.contains("first"));
}

/// Test complex struct patterns
#[test]
fn test_struct_patterns_with_builder() {
    let builder = AstBuilder::new();
    let transpiler = Transpiler::new();
    
    // Create: match point { Point { x: 0, y: 0 } => "origin", Point { x, y } => format!("{},{}", x, y) }
    let ast = builder.match_expr(
        builder.ident("point"),
        vec![
            // First arm: Point { x: 0, y: 0 } => "origin"
            builder.match_arm(
                builder.pattern_struct(
                    "Point".to_string(),
                    vec![
                        ("x".to_string(), builder.pattern_literal(Literal::Integer(0))),
                        ("y".to_string(), builder.pattern_literal(Literal::Integer(0))),
                    ],
                ),
                None,
                builder.string("origin"),
            ),
            // Second arm: Point { x, y } => format!("{},{}", x, y)
            builder.match_arm(
                builder.pattern_struct(
                    "Point".to_string(),
                    vec![
                        ("x".to_string(), builder.pattern_ident("x")),
                        ("y".to_string(), builder.pattern_ident("y")),
                    ],
                ),
                None,
                builder.call(
                    builder.ident("format!"),
                    vec![
                        builder.string("{},{}"),
                        builder.ident("x"),
                        builder.ident("y"),
                    ],
                ),
            ),
        ],
    );
    
    let result = transpiler.transpile(&ast).unwrap();
    let code = result.to_string();
    
    assert!(code.contains("Point"));
    assert!(code.contains("origin"));
}

/// Test nested patterns with guards
#[test]
fn test_nested_patterns_with_guards() {
    let builder = AstBuilder::new();
    let transpiler = Transpiler::new();
    
    // Create: match pair { (Some(x), Some(y)) if x > y => x, (Some(x), _) => x, _ => 0 }
    let ast = builder.match_expr(
        builder.ident("pair"),
        vec![
            // First arm: (Some(x), Some(y)) if x > y => x
            builder.match_arm(
                builder.pattern_tuple(vec![
                    Pattern::Some(Box::new(builder.pattern_ident("x"))),
                    Pattern::Some(Box::new(builder.pattern_ident("y"))),
                ]),
                Some(builder.binary(
                    builder.ident("x"),
                    BinaryOp::Greater,
                    builder.ident("y"),
                )),
                builder.ident("x"),
            ),
            // Second arm: (Some(x), _) => x
            builder.match_arm(
                builder.pattern_tuple(vec![
                    Pattern::Some(Box::new(builder.pattern_ident("x"))),
                    builder.pattern_wildcard(),
                ]),
                None,
                builder.ident("x"),
            ),
            // Default arm: _ => 0
            builder.match_arm(
                builder.pattern_wildcard(),
                None,
                builder.int(0),
            ),
        ],
    );
    
    let result = transpiler.transpile(&ast).unwrap();
    let code = result.to_string();
    
    assert!(code.contains("Some"));
    assert!(code.contains("match"));
}

/// Test Result type construction and matching
#[test]
fn test_result_patterns_with_builder() {
    let builder = AstBuilder::new();
    let transpiler = Transpiler::new();
    
    // Create: match result { Ok(val) if val > 0 => val, Err(msg) => panic!(msg), _ => 0 }
    let ast = builder.match_expr(
        builder.ident("result"),
        vec![
            // First arm: Ok(val) if val > 0 => val
            builder.match_arm(
                Pattern::Ok(Box::new(builder.pattern_ident("val"))),
                Some(builder.binary(
                    builder.ident("val"),
                    BinaryOp::Greater,
                    builder.int(0),
                )),
                builder.ident("val"),
            ),
            // Second arm: Err(msg) => panic!(msg)
            builder.match_arm(
                Pattern::Err(Box::new(builder.pattern_ident("msg"))),
                None,
                builder.call(
                    builder.ident("panic!"),
                    vec![builder.ident("msg")],
                ),
            ),
            // Default arm: _ => 0
            builder.match_arm(
                builder.pattern_wildcard(),
                None,
                builder.int(0),
            ),
        ],
    );
    
    let result = transpiler.transpile(&ast).unwrap();
    let code = result.to_string();
    
    assert!(code.contains("Ok"));
    assert!(code.contains("Err"));
    assert!(code.contains("panic"));
}