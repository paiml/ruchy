//! Frontend parsing and lexical analysis
//!
//! This module handles tokenization, parsing, and AST construction for the
//! Ruchy programming language, implementing a complete frontend compiler pipeline.
//!
//! # Architecture
//!
//! The frontend follows the traditional compiler design:
//!
//! ```text
//! Source Code → Lexer → Token Stream → Parser → AST
//!      ↓          ↓         ↓           ↓        ↓
//!   String    Tokens   TokenStream   Parse   Expressions
//! ```
//!
//! # Components
//!
//! ## Lexer
//! Converts source code into a stream of tokens:
//! - Keyword recognition and context-sensitive parsing
//! - String literals with escape sequences
//! - Numeric literals (integers, floats, scientific notation)
//! - Operators and punctuation
//! - Comment handling and whitespace management
//!
//! ## Parser
//! Builds an Abstract Syntax Tree (AST) from tokens:
//! - **Pratt Parser**: Operator precedence parsing
//! - **Recursive Descent**: For statements and declarations
//! - **Error Recovery**: Continue parsing after syntax errors
//! - **Look-ahead**: Minimal token buffering for efficiency
//!
//! ## AST
//! Type-safe representation of Ruchy programs:
//! - Expression nodes for all value-producing constructs
//! - Pattern matching for destructuring
//! - Type annotations and generics
//! - Actor system constructs
//! - `DataFrame` and data science operations
//!
//! ## Error Recovery
//! Robust error handling for interactive development:
//! - Syntax error reporting with location information
//! - Error recovery strategies to continue parsing
//! - Diagnostic suggestions for common mistakes
//!
//! ## Memory Management
//! Efficient AST construction with arena allocation:
//! - Bump allocation for AST nodes
//! - String interning for identifiers
//! - Memory reuse across parse sessions
//!
//! # Examples
//!
//! ## Basic Parsing
//! ```
//! use ruchy::frontend::{Parser, Expr, ExprKind};
//!
//! let mut parser = Parser::new("42 + 3.14");
//! let ast = parser.parse().unwrap();
//!
//! if let ExprKind::Binary { left, op, right } = &ast.kind {
//!     println!("Found binary operation: {:?}", op);
//! }
//! ```
//!
//! ## Tokenization
//! ```
//! use ruchy::frontend::{TokenStream, Token};
//!
//! let mut tokens = TokenStream::new("let x = 42");
//! while let Some((token, _span)) = tokens.next() {
//!     match token {
//!         Token::Let => println!("Let keyword"),
//!         Token::Identifier(name) => println!("Identifier: {}", name),
//!         Token::Integer(n) => println!("Number: {}", n),
//!         _ => {}
//!     }
//! }
//! ```
//!
//! ## Error Handling
//! ```
//! use ruchy::frontend::Parser;
//!
//! let mut parser = Parser::new("let x = ");  // Incomplete
//! match parser.parse() {
//!     Ok(ast) => println!("Parsed: {:?}", ast),
//!     Err(error) => println!("Parse error: {}", error),
//! }
//! ```
//!
//! # Language Features Supported
//!
//! - **Expressions**: Literals, variables, operators, function calls
//! - **Control Flow**: if/else, match, loops (for, while)
//! - **Functions**: Definitions, lambdas, generics, async/await
//! - **Data Types**: Structs, enums, traits, implementations
//! - **Pattern Matching**: Destructuring, guards, exhaustiveness
//! - **Actors**: Message passing, supervision, ask/tell operations
//! - **`DataFrames`**: Column operations, data manipulation
//! - **Imports**: Module system with dependency resolution
pub mod arena;
pub mod ast;
pub mod diagnostics;
pub mod error_recovery;
pub mod lexer;
pub mod parser;
pub use ast::*;
pub use error_recovery::{ParseError, ParseResult, RecoveryParser};
pub use lexer::{Token, TokenStream};
pub use parser::Parser;

#[cfg(test)]
mod tests {
    use super::*;

    // Sprint 5: Comprehensive frontend integration tests

    #[test]
    fn test_parser_creation() {
        let mut parser = Parser::new("42");
        // Parser should be created successfully
        assert!(parser.parse().is_ok());
    }

    #[test]
    fn test_parse_simple_expression() {
        let mut parser = Parser::new("42");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Integer(42))));
    }

    #[test]
    fn test_parse_binary_operation() {
        let mut parser = Parser::new("1 + 2");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_string_literal() {
        let mut parser = Parser::new(r#""hello world""#);
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let ExprKind::Literal(Literal::String(s)) = expr.kind {
            assert_eq!(s, "hello world");
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_parse_boolean_literals() {
        let mut parser = Parser::new("true");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Bool(true))));

        let mut parser = Parser::new("false");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Bool(false))));
    }

    #[test]
    fn test_parse_identifier() {
        let mut parser = Parser::new("variable_name");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let ExprKind::Identifier(name) = expr.kind {
            assert_eq!(name, "variable_name");
        } else {
            panic!("Expected identifier");
        }
    }

    #[test]
    fn test_parse_list_literal() {
        let mut parser = Parser::new("[1, 2, 3]");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let ExprKind::List(items) = expr.kind {
            assert_eq!(items.len(), 3);
        } else {
            panic!("Expected list literal");
        }
    }

    #[test]
    fn test_parse_empty_list() {
        let mut parser = Parser::new("[]");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let ExprKind::List(items) = expr.kind {
            assert!(items.is_empty());
        } else {
            panic!("Expected empty list");
        }
    }

    #[test]
    fn test_parse_tuple() {
        let mut parser = Parser::new("(1, 2, 3)");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let ExprKind::Tuple(items) = expr.kind {
            assert_eq!(items.len(), 3);
        } else {
            panic!("Expected tuple");
        }
    }

    #[test]
    fn test_parse_unary_negation() {
        let mut parser = Parser::new("-42");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(
            expr.kind,
            ExprKind::Unary {
                op: UnaryOp::Negate,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_unary_not() {
        let mut parser = Parser::new("!true");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(
            expr.kind,
            ExprKind::Unary {
                op: UnaryOp::Not,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_comparison_operators() {
        let test_cases = vec![
            ("5 > 3", BinaryOp::Greater),
            ("3 < 5", BinaryOp::Less),
            ("5 >= 5", BinaryOp::GreaterEqual),
            ("3 <= 5", BinaryOp::LessEqual),
            ("5 == 5", BinaryOp::Equal),
            ("3 != 5", BinaryOp::NotEqual),
        ];

        for (input, expected_op) in test_cases {
            let mut parser = Parser::new(input);
            let result = parser.parse();
            assert!(result.is_ok(), "Failed to parse: {input}");
            let expr = result.unwrap();
            if let ExprKind::Binary { op, .. } = expr.kind {
                assert_eq!(op, expected_op, "Wrong operator for: {input}");
            } else {
                panic!("Expected binary operation for: {input}");
            }
        }
    }

    #[test]
    fn test_parse_logical_operators() {
        let mut parser = Parser::new("true && false");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::And,
                ..
            }
        ));

        let mut parser = Parser::new("true || false");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        assert!(matches!(
            expr.kind,
            ExprKind::Binary {
                op: BinaryOp::Or,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_arithmetic_operators() {
        let test_cases = vec![
            ("1 + 2", BinaryOp::Add),
            ("5 - 3", BinaryOp::Subtract),
            ("2 * 3", BinaryOp::Multiply),
            ("6 / 2", BinaryOp::Divide),
            ("7 % 3", BinaryOp::Modulo),
        ];

        for (input, expected_op) in test_cases {
            let mut parser = Parser::new(input);
            let result = parser.parse();
            assert!(result.is_ok(), "Failed to parse: {input}");
            let expr = result.unwrap();
            if let ExprKind::Binary { op, .. } = expr.kind {
                assert_eq!(op, expected_op, "Wrong operator for: {input}");
            } else {
                panic!("Expected binary operation for: {input}");
            }
        }
    }

    #[test]
    fn test_parse_bitwise_operators() {
        let test_cases = vec![
            ("1 & 2", BinaryOp::BitwiseAnd),
            ("1 | 2", BinaryOp::BitwiseOr),
            ("1 ^ 2", BinaryOp::BitwiseXor),
        ];

        for (input, expected_op) in test_cases {
            let mut parser = Parser::new(input);
            let result = parser.parse();
            assert!(result.is_ok(), "Failed to parse: {input}");
            let expr = result.unwrap();
            if let ExprKind::Binary { op, .. } = expr.kind {
                assert_eq!(op, expected_op, "Wrong operator for: {input}");
            } else {
                panic!("Expected binary operation for: {input}");
            }
        }
    }

    #[test]
    fn test_parse_parenthesized_expression() {
        let mut parser = Parser::new("(42)");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        // Parentheses don't create a special node, just affect precedence
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Integer(42))));
    }

    #[test]
    fn test_parse_precedence() {
        // Test that multiplication has higher precedence than addition
        let mut parser = Parser::new("1 + 2 * 3");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        // Should parse as 1 + (2 * 3), not (1 + 2) * 3
        if let ExprKind::Binary {
            op: BinaryOp::Add,
            right,
            ..
        } = expr.kind
        {
            assert!(matches!(
                right.kind,
                ExprKind::Binary {
                    op: BinaryOp::Multiply,
                    ..
                }
            ));
        } else {
            panic!("Expected addition at top level");
        }
    }

    #[test]
    fn test_parse_float_literal() {
        let mut parser = Parser::new("3.15");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let ExprKind::Literal(Literal::Float(f)) = expr.kind {
            assert!((f - 3.15).abs() < 0.001);
        } else {
            panic!("Expected float literal");
        }
    }

    #[test]
    fn test_parse_character_literal() {
        let mut parser = Parser::new("'a'");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let ExprKind::Literal(Literal::Char(c)) = expr.kind {
            assert_eq!(c, 'a');
        } else {
            panic!("Expected character literal");
        }
    }

    #[test]

    fn test_parse_nil_literal() {
        let mut parser = Parser::new("nil");
        let result = parser.parse();
        assert!(result.is_ok());
        // Nil might be parsed differently
    }

    #[test]
    fn test_parse_error_recovery() {
        // Test that parser can handle incomplete expressions
        let mut parser = Parser::new("let x =");
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_complex_expression() {
        let mut parser = Parser::new("(a + b) * (c - d) / 2");
        let result = parser.parse();
        assert!(result.is_ok());
        // Just verify it parses without checking the exact structure
    }

    #[test]
    fn test_token_stream_iteration() {
        let mut tokens = TokenStream::new("let x = 42");
        let mut token_count = 0;
        while tokens.next().is_some() {
            token_count += 1;
        }
        assert!(token_count > 0);
    }

    #[test]
    fn test_token_stream_peek() {
        let mut tokens = TokenStream::new("42 + 3");
        // Peek shouldn't consume the token
        let first_peek = tokens.peek().cloned();
        let second_peek = tokens.peek().cloned();
        assert_eq!(first_peek, second_peek);

        // Next should consume it
        let next = tokens.next();
        assert!(next.is_some());

        // Now peek should show a different token
        let peek_after_next = tokens.peek().cloned();
        assert_ne!(first_peek, peek_after_next);
    }

    #[test]
    fn test_parse_range_expression() {
        let mut parser = Parser::new("1..10");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let ExprKind::Range { inclusive, .. } = expr.kind {
            assert!(!inclusive);
        } else {
            panic!("Expected range expression");
        }
    }

    #[test]
    fn test_parse_inclusive_range() {
        let mut parser = Parser::new("1..=10");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let ExprKind::Range { inclusive, .. } = expr.kind {
            assert!(inclusive);
        } else {
            panic!("Expected inclusive range");
        }
    }

    #[test]
    fn test_parse_nested_list() {
        let mut parser = Parser::new("[[1, 2], [3, 4]]");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let ExprKind::List(outer) = expr.kind {
            assert_eq!(outer.len(), 2);
            for item in outer {
                assert!(matches!(item.kind, ExprKind::List(_)));
            }
        } else {
            panic!("Expected nested list");
        }
    }

    #[test]
    fn test_parse_empty_tuple() {
        let mut parser = Parser::new("()");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        // Empty tuple might be parsed as unit or as empty tuple
        assert!(
            matches!(expr.kind, ExprKind::Tuple(_))
                || matches!(expr.kind, ExprKind::Literal(Literal::Unit))
        );
    }

    #[test]
    fn test_parse_single_element_tuple() {
        let mut parser = Parser::new("(42,)");
        let result = parser.parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        if let ExprKind::Tuple(items) = expr.kind {
            assert_eq!(items.len(), 1);
        } else {
            panic!("Expected single-element tuple");
        }
    }
}
