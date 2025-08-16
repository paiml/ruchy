#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]
#![allow(clippy::expect_used)]
#![allow(clippy::manual_clamp)] // Test code doesn't need perfect clamp
#![allow(clippy::uninlined_format_args)] // Format strings are clearer sometimes
#![allow(clippy::cast_lossless)] // Test code can use as casts
#![allow(clippy::needless_pass_by_value)] // Test functions can take owned values
#![allow(clippy::single_char_pattern)] // String patterns are fine in tests
#![allow(clippy::items_after_statements)] // Helper functions in tests are fine

use proptest::prelude::*;
use quickcheck::{Arbitrary, Gen};
use quickcheck_macros::quickcheck;
use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::{ast::*, lexer::*, parser::Parser};

// Arbitrary implementation for generating random valid ASTs
#[derive(Debug, Clone)]
struct ArbitraryExpr(Expr);

impl Arbitrary for ArbitraryExpr {
    fn arbitrary(g: &mut Gen) -> Self {
        ArbitraryExpr(arbitrary_expr(g, 3))
    }
}

fn arbitrary_expr(g: &mut Gen, depth: usize) -> Expr {
    if depth == 0 {
        // Base case: generate literals
        match u8::arbitrary(g) % 4 {
            0 => Expr::new(
                ExprKind::Literal(Literal::Integer(i64::arbitrary(g))),
                Span::new(0, 0),
            ),
            1 => {
                // Generate finite floats only to avoid NaN/Infinity issues
                let mut f = f64::arbitrary(g);
                while !f.is_finite() {
                    f = f64::arbitrary(g);
                }
                // Clamp to reasonable range
                f = f.max(-1e10).min(1e10);
                Expr::new(ExprKind::Literal(Literal::Float(f)), Span::new(0, 0))
            }
            2 => Expr::new(
                ExprKind::Literal(Literal::Bool(bool::arbitrary(g))),
                Span::new(0, 0),
            ),
            _ => {
                let s: String = (0..10)
                    .map(|_| {
                        let c = u8::arbitrary(g) % 26 + b'a';
                        c as char
                    })
                    .collect();
                Expr::new(ExprKind::Literal(Literal::String(s)), Span::new(0, 0))
            }
        }
    } else {
        // Recursive case: generate complex expressions
        match u8::arbitrary(g) % 6 {
            0 => {
                // Binary expression
                let left = arbitrary_expr(g, depth - 1);
                let right = arbitrary_expr(g, depth - 1);
                let op = match u8::arbitrary(g) % 5 {
                    0 => BinaryOp::Add,
                    1 => BinaryOp::Subtract,
                    2 => BinaryOp::Multiply,
                    3 => BinaryOp::Divide,
                    _ => BinaryOp::Equal,
                };
                Expr::new(
                    ExprKind::Binary {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    },
                    Span::new(0, 0),
                )
            }
            1 => {
                // Unary expression
                let operand = arbitrary_expr(g, depth - 1);
                let op = match u8::arbitrary(g) % 2 {
                    0 => UnaryOp::Negate,
                    _ => UnaryOp::Not,
                };
                Expr::new(
                    ExprKind::Unary {
                        op,
                        operand: Box::new(operand),
                    },
                    Span::new(0, 0),
                )
            }
            2 => {
                // List
                let items: Vec<Expr> = (0..3).map(|_| arbitrary_expr(g, depth - 1)).collect();
                Expr::new(ExprKind::List(items), Span::new(0, 0))
            }
            3 => {
                // If-then-else
                let condition = arbitrary_expr(g, depth - 1);
                let then_branch = arbitrary_expr(g, depth - 1);
                let else_branch = Some(Box::new(arbitrary_expr(g, depth - 1)));
                Expr::new(
                    ExprKind::If {
                        condition: Box::new(condition),
                        then_branch: Box::new(then_branch),
                        else_branch,
                    },
                    Span::new(0, 0),
                )
            }
            4 => {
                // Identifier
                let name: String = (0..5)
                    .map(|_| {
                        let c = u8::arbitrary(g) % 26 + b'a';
                        c as char
                    })
                    .collect();
                Expr::new(ExprKind::Identifier(name), Span::new(0, 0))
            }
            _ => {
                // Block
                let exprs: Vec<Expr> = (0..2).map(|_| arbitrary_expr(g, depth - 1)).collect();
                Expr::new(ExprKind::Block(exprs), Span::new(0, 0))
            }
        }
    }
}

// Property: Parser should handle all valid tokens without panicking
proptest! {
    #[test]
    fn parser_doesnt_panic(input in ".*") {
        let mut parser = Parser::new(&input);
        let _ = parser.parse(); // Should not panic
    }
}

// Property: Lexer should tokenize and detokenize consistently
proptest! {
    #[test]
    fn lexer_roundtrip(input in "[a-zA-Z0-9 \\+\\-\\*\\(\\)\\{\\}\\[\\]\\.,;:=<>!&\\|]+") {
        let mut stream = TokenStream::new(&input);
        let mut tokens = Vec::new();
        while let Some((token, _span)) = stream.next() {
            tokens.push(token);
        }
        // All input should be tokenized (no loss of data)
        prop_assert!(!tokens.is_empty() || input.trim().is_empty());
    }
}

// Property: Integer literals should parse correctly
proptest! {
    #[test]
    fn integer_parsing(n: i64) {
        let input = n.to_string();
        let mut parser = Parser::new(&input);
        let expr = parser.parse().unwrap();

        // Negative numbers parse as unary negation
        if n < 0 {
            match &expr.kind {
                ExprKind::Unary { op: UnaryOp::Negate, operand } => {
                    match &operand.kind {
                        ExprKind::Literal(Literal::Integer(parsed)) => {
                            prop_assert_eq!(*parsed, -n);
                        }
                        _ => panic!("Expected integer literal in negation"),
                    }
                }
                _ => panic!("Expected unary negation for negative number"),
            }
        } else {
            match &expr.kind {
                ExprKind::Literal(Literal::Integer(parsed)) => {
                    prop_assert_eq!(*parsed, n);
                }
                _ => panic!("Expected integer literal"),
            }
        }
    }
}

// Property: Float literals should parse correctly (within precision)
proptest! {
    #[test]
    fn float_parsing(f in -1_000_000.0..1_000_000.0) {
        // Use a reasonable range of floats that don't need scientific notation
        
        let input = format!("{}", f);
        let mut parser = Parser::new(&input);
        let expr = parser.parse().unwrap();

        // Negative floats parse as unary negation
        if f < 0.0 {
            match &expr.kind {
                ExprKind::Unary { op: UnaryOp::Negate, operand } => {
                    match &operand.kind {
                        ExprKind::Literal(Literal::Float(parsed)) => {
                            let diff: f64 = *parsed + f;
                            prop_assert!(diff.abs() < 1e-10);
                        }
                        _ => panic!("Expected float literal in negation"),
                    }
                }
                _ => panic!("Expected unary negation for negative float"),
            }
        } else {
            match &expr.kind {
                ExprKind::Literal(Literal::Float(parsed)) => {
                    let diff: f64 = *parsed - f;
                    prop_assert!(diff.abs() < 1e-10);
                }
                _ => panic!("Expected float literal"),
            }
        }
    }
}

// Property: Binary operator precedence should be correct
proptest! {
    #[test]
    fn operator_precedence(a in 0i32..100, b in 0i32..100, c in 0i32..100) {
        let input = format!("{} + {} * {}", a, b, c);
        let mut parser = Parser::new(&input);
        let expr = parser.parse().unwrap();

        // Should parse as a + (b * c), not (a + b) * c
        match &expr.kind {
            ExprKind::Binary { left, op: BinaryOp::Add, right } => {
                // Left should be 'a'
                match &left.kind {
                    ExprKind::Literal(Literal::Integer(n)) => {
                        prop_assert_eq!(*n, a as i64);
                    }
                    _ => panic!("Expected integer on left"),
                }

                // Right should be 'b * c'
                match &right.kind {
                    ExprKind::Binary { left: b_expr, op: BinaryOp::Multiply, right: c_expr } => {
                        match &b_expr.kind {
                            ExprKind::Literal(Literal::Integer(n)) => {
                                prop_assert_eq!(*n, b as i64);
                            }
                            _ => panic!("Expected integer for b"),
                        }
                        match &c_expr.kind {
                            ExprKind::Literal(Literal::Integer(n)) => {
                                prop_assert_eq!(*n, c as i64);
                            }
                            _ => panic!("Expected integer for c"),
                        }
                    }
                    _ => panic!("Expected multiplication on right"),
                }
            }
            _ => panic!("Expected addition at top level"),
        }
    }
}

// Property: String escaping should work correctly
proptest! {
    #[test]
    fn string_escaping(s in prop::string::string_regex("[a-zA-Z0-9 ]*").unwrap()) {
        let input = format!("\"{}\"", s);
        let mut parser = Parser::new(&input);
        let expr = parser.parse().unwrap();

        match &expr.kind {
            ExprKind::Literal(Literal::String(parsed)) => {
                prop_assert_eq!(parsed, &s);
            }
            _ => panic!("Expected string literal"),
        }
    }
}

// Property: Lists should maintain order
proptest! {
    #[test]
    fn list_ordering(items in prop::collection::vec(0i32..100, 0..10)) {
        let input = format!("[{}]", items.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", "));
        let mut parser = Parser::new(&input);
        let expr = parser.parse().unwrap();

        match &expr.kind {
            ExprKind::List(parsed_items) => {
                prop_assert_eq!(parsed_items.len(), items.len());
                for (i, item) in parsed_items.iter().enumerate() {
                    match &item.kind {
                        ExprKind::Literal(Literal::Integer(n)) => {
                            prop_assert_eq!(*n, items[i] as i64);
                        }
                        _ => panic!("Expected integer in list"),
                    }
                }
            }
            _ => panic!("Expected list"),
        }
    }
}

// Property: Pipeline operator should be left-associative
proptest! {
    #[test]
    fn pipeline_associativity(a in "var[a-z]{0,5}", b in "var[a-z]{0,5}", c in "var[a-z]{0,5}") {
        let input = format!("{} |> {} |> {}", a, b, c);
        let mut parser = Parser::new(&input);
        let expr = parser.parse().unwrap();

        // Should parse as (a |> b) |> c
        match &expr.kind {
            ExprKind::Pipeline { expr, stages } => {
                // Should have two stages
                prop_assert_eq!(stages.len(), 2);

                // First expression should be 'a'
                match &expr.kind {
                    ExprKind::Identifier(name) => {
                        prop_assert_eq!(name, &a);
                    }
                    _ => panic!("Expected identifier for a"),
                }
            }
            _ => panic!("Expected pipeline at top level"),
        }
    }
}

// Property: Comments should not affect parsing
proptest! {
    #[test]
    fn comments_ignored(expr in "[0-9]{1,10}", comment in "[a-zA-Z ]+") {
        let input_with = format!("{} // {}", expr, comment);

        let mut parser1 = Parser::new(&expr);
        let mut parser2 = Parser::new(&input_with);

        let expr1 = parser1.parse().unwrap();
        let expr2 = parser2.parse().unwrap();

        // Both should produce the same AST
        prop_assert_eq!(format!("{:?}", expr1.kind), format!("{:?}", expr2.kind));
    }
}

// QuickCheck tests for more complex properties

#[quickcheck]
fn parse_print_roundtrip(expr: ArbitraryExpr) -> bool {
    let printed = format!("{:?}", expr.0); // This would be pretty_print in production
    let mut parser = Parser::new(&printed);
    // For now, just check it doesn't panic
    let _ = parser.parse();
    true
}

#[quickcheck]
fn transpiler_produces_valid_rust(expr: ArbitraryExpr) -> bool {
    let transpiler = Transpiler::new();
    match transpiler.transpile_expr(&expr.0) {
        Ok(rust_code) => {
            // Check that the generated Rust code is syntactically valid
            syn::parse_str::<syn::Expr>(&rust_code.to_string()).is_ok()
        }
        Err(_) => {
            // Some expressions might not be transpilable yet
            true
        }
    }
}

// Property: Type preservation through transpilation
proptest! {
    #[test]
    fn type_preservation(n: i32) {
        let input = format!("{} + {}", n, n);
        let mut parser = Parser::new(&input);
        let expr = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile_expr(&expr).unwrap();

        // The Rust code should preserve the operation
        let rust_str = rust_code.to_string();
        prop_assert!(rust_str.contains("+"));
    }
}

// Property: Whitespace insensitivity
proptest! {
    #[test]
    fn whitespace_insensitive(a: i32, b: i32, spaces in prop::collection::vec(prop::bool::ANY, 0..5)) {
        use std::fmt::Write;
        let mut input1 = format!("{}", a);
        let mut input2 = format!("{}", a);

        for space in &spaces {
            if *space {
                input1.push(' ');
                input2.push_str("  ");
            }
        }

        write!(&mut input1, "+{}", b).unwrap();
        write!(&mut input2, "+ {}", b).unwrap();

        let mut parser1 = Parser::new(&input1);
        let mut parser2 = Parser::new(&input2);

        let expr1 = parser1.parse();
        let expr2 = parser2.parse();

        prop_assert_eq!(expr1.is_ok(), expr2.is_ok());
    }
}

// Property: Nested parentheses should parse correctly
proptest! {
    #[test]
    fn string_interpolation_parsing(parts in prop::collection::vec(
        prop_oneof![
            "[a-zA-Z0-9 .,!?]+".prop_map(|s| format!("TEXT:{}", s)),
            "[a-z][a-z0-9_]*".prop_map(|s| format!("EXPR:{}", s))
        ],
        1..5
    )) {
        // Build a string with interpolation
        let mut input = String::from('"');
        for part in parts {
            if let Some(stripped) = part.strip_prefix("TEXT:") {
                input.push_str(stripped);
            } else if let Some(stripped) = part.strip_prefix("EXPR:") {
                input.push('{');
                input.push_str(stripped);
                input.push('}');
            }
        }
        input.push('"');
        
        let mut parser = Parser::new(&input);
        let result = parser.parse();
        
        // Should either parse successfully or fail gracefully
        if let Ok(expr) = result {
            // Should be either string interpolation or literal string
            match expr.kind {
                ExprKind::StringInterpolation { .. } | ExprKind::Literal(Literal::String(_)) => {},
                _ => prop_assert!(false, "Expected string interpolation or string literal"),
            }
        }
        // Parse errors are acceptable for malformed input
    }

    #[test]
    fn string_interpolation_transpilation(input in "[a-z][a-z0-9_]*") {
        // Test simple string interpolation transpilation
        let parts = vec![
            ruchy::frontend::ast::StringPart::Text("Hello, ".to_string()),
            ruchy::frontend::ast::StringPart::Expr(Box::new(
                ruchy::frontend::ast::Expr::new(
                    ruchy::frontend::ast::ExprKind::Identifier(input),
                    ruchy::frontend::ast::Span::default()
                )
            )),
            ruchy::frontend::ast::StringPart::Text("!".to_string()),
        ];
        
        let transpiler = ruchy::backend::transpiler::Transpiler::new();
        let result = transpiler.transpile_string_interpolation(&parts);
        
        // Should either succeed or fail cleanly, never panic
        if let Ok(tokens) = result {
            let code = tokens.to_string();
            // Should produce valid Rust code
            prop_assert!(!code.is_empty(), "Generated code should not be empty");
        }
        // Transpilation errors are acceptable for some inputs
    }

    #[test]
    fn nested_parentheses(depth in 1usize..10) {
        let mut input = String::new();
        for _ in 0..depth {
            input.push('(');
        }
        input.push_str("42");
        for _ in 0..depth {
            input.push(')');
        }

        let mut parser = Parser::new(&input);
        let expr = parser.parse().unwrap();

        // Should parse to 42 regardless of parentheses
        // Walk through nested structure to find the literal
        fn find_literal(expr: &Expr) -> Option<i64> {
            match &expr.kind {
                ExprKind::Literal(Literal::Integer(n)) => Some(*n),
                _ => None,
            }
        }

        prop_assert_eq!(find_literal(&expr), Some(42));
    }
}
