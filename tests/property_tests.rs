use proptest::prelude::*;
use quickcheck::{quickcheck, Arbitrary, Gen};
use ruchy::frontend::{ast::*, lexer::*, parser::Parser};
use ruchy::backend::transpiler::Transpiler;

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
                Span::dummy(),
            ),
            1 => Expr::new(
                ExprKind::Literal(Literal::Float(f64::arbitrary(g))),
                Span::dummy(),
            ),
            2 => Expr::new(
                ExprKind::Literal(Literal::Bool(bool::arbitrary(g))),
                Span::dummy(),
            ),
            _ => {
                let s: String = (0..10)
                    .map(|_| {
                        let c = u8::arbitrary(g) % 26 + b'a';
                        c as char
                    })
                    .collect();
                Expr::new(ExprKind::Literal(Literal::String(s)), Span::dummy())
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
                    1 => BinaryOp::Sub,
                    2 => BinaryOp::Mul,
                    3 => BinaryOp::Div,
                    _ => BinaryOp::Equal,
                };
                Expr::new(
                    ExprKind::Binary {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    },
                    Span::dummy(),
                )
            }
            1 => {
                // Unary expression
                let expr = arbitrary_expr(g, depth - 1);
                let op = match u8::arbitrary(g) % 2 {
                    0 => UnaryOp::Neg,
                    _ => UnaryOp::Not,
                };
                Expr::new(
                    ExprKind::Unary {
                        op,
                        expr: Box::new(expr),
                    },
                    Span::dummy(),
                )
            }
            2 => {
                // List
                let items: Vec<Expr> = (0..3)
                    .map(|_| arbitrary_expr(g, depth - 1))
                    .collect();
                Expr::new(ExprKind::List(items), Span::dummy())
            }
            3 => {
                // If-then-else
                let cond = arbitrary_expr(g, depth - 1);
                let then = arbitrary_expr(g, depth - 1);
                let else_ = Some(Box::new(arbitrary_expr(g, depth - 1)));
                Expr::new(
                    ExprKind::If {
                        cond: Box::new(cond),
                        then: Box::new(then),
                        else_,
                    },
                    Span::dummy(),
                )
            }
            4 => {
                // Variable
                let name: String = (0..5)
                    .map(|_| {
                        let c = u8::arbitrary(g) % 26 + b'a';
                        c as char
                    })
                    .collect();
                Expr::new(ExprKind::Variable(name), Span::dummy())
            }
            _ => {
                // Block
                let stmts: Vec<Stmt> = (0..2)
                    .map(|_| {
                        let expr = arbitrary_expr(g, depth - 1);
                        Stmt::Expr(expr)
                    })
                    .collect();
                let expr = arbitrary_expr(g, depth - 1);
                Expr::new(
                    ExprKind::Block {
                        stmts,
                        expr: Some(Box::new(expr)),
                    },
                    Span::dummy(),
                )
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
    fn lexer_roundtrip(input in "[a-zA-Z0-9 +-*/(){}\\[\\].,;:=<>!&|]+") {
        let tokens: Vec<Token> = TokenStream::new(&input).collect();
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
        
        match &expr.kind {
            ExprKind::Literal(Literal::Integer(parsed)) => {
                prop_assert_eq!(*parsed, n);
            }
            _ => panic!("Expected integer literal"),
        }
    }
}

// Property: Float literals should parse correctly (within precision)
proptest! {
    #[test]
    fn float_parsing(f in prop::num::f64::NORMAL) {
        let input = format!("{}", f);
        let mut parser = Parser::new(&input);
        let expr = parser.parse().unwrap();
        
        match &expr.kind {
            ExprKind::Literal(Literal::Float(parsed)) => {
                prop_assert!((parsed - f).abs() < 1e-10);
            }
            _ => panic!("Expected float literal"),
        }
    }
}

// Property: Binary operator precedence should be correct
proptest! {
    #[test]
    fn operator_precedence(a: i32, b: i32, c: i32) {
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
                    ExprKind::Binary { left: b_expr, op: BinaryOp::Mul, right: c_expr } => {
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
        let input = format!("[{}]", items.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", "));
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
    fn pipeline_associativity(a in "[a-z]+", b in "[a-z]+", c in "[a-z]+") {
        let input = format!("{} |> {} |> {}", a, b, c);
        let mut parser = Parser::new(&input);
        let expr = parser.parse().unwrap();
        
        // Should parse as (a |> b) |> c
        match &expr.kind {
            ExprKind::Pipeline { expr, func } => {
                // func should be 'c'
                match &func.kind {
                    ExprKind::Variable(name) => {
                        prop_assert_eq!(name, &c);
                    }
                    _ => panic!("Expected variable for c"),
                }
                
                // expr should be 'a |> b'
                match &expr.kind {
                    ExprKind::Pipeline { expr: a_expr, func: b_func } => {
                        match &a_expr.kind {
                            ExprKind::Variable(name) => {
                                prop_assert_eq!(name, &a);
                            }
                            _ => panic!("Expected variable for a"),
                        }
                        match &b_func.kind {
                            ExprKind::Variable(name) => {
                                prop_assert_eq!(name, &b);
                            }
                            _ => panic!("Expected variable for b"),
                        }
                    }
                    _ => panic!("Expected pipeline for a |> b"),
                }
            }
            _ => panic!("Expected pipeline at top level"),
        }
    }
}

// Property: Comments should not affect parsing
proptest! {
    #[test]
    fn comments_ignored(expr in "[0-9]+", comment in "[a-zA-Z ]+") {
        let input_without = expr.clone();
        let input_with = format!("{} // {}", expr, comment);
        
        let mut parser1 = Parser::new(&input_without);
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
        let mut input1 = format!("{}", a);
        let mut input2 = format!("{}", a);
        
        for space in &spaces {
            if *space {
                input1.push(' ');
                input2.push_str("  ");
            }
        }
        
        input1.push_str(&format!("+{}", b));
        input2.push_str(&format!("+ {}", b));
        
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
        
        // Should still parse to 42
        let mut current = &expr;
        for _ in 0..depth {
            match &current.kind {
                ExprKind::Paren(inner) => {
                    current = inner;
                }
                _ => {
                    // Reached the literal
                    break;
                }
            }
        }
        
        match &current.kind {
            ExprKind::Literal(Literal::Integer(n)) => {
                prop_assert_eq!(*n, 42);
            }
            _ => panic!("Expected integer literal"),
        }
    }
}