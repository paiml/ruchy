//! Comprehensive TDD test suite for expressions.rs parser module
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every expression parsing path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::{Parser, frontend::{ExprKind, Literal, UnaryOp}};

// ==================== LITERAL PARSING TESTS ====================

#[test]
fn test_parse_integer_literal() {
    let mut parser = Parser::new("42");
    let ast = parser.parse().unwrap();
    
    // Should parse as a program with single integer
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Literal(Literal::Integer(val)) => assert_eq!(*val, 42),
                _ => panic!("Expected integer literal"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_float_literal() {
    let mut parser = Parser::new("3.14");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Literal(Literal::Float(val)) => assert_eq!(*val, 3.14),
                _ => panic!("Expected float literal"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_string_literal() {
    let mut parser = Parser::new(r#""hello world""#);
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Literal(Literal::String(val)) => assert_eq!(val, "hello world"),
                _ => panic!("Expected string literal"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_char_literal() {
    let mut parser = Parser::new("'a'");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Literal(Literal::Char(val)) => assert_eq!(*val, 'a'),
                _ => panic!("Expected char literal"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_bool_true() {
    let mut parser = Parser::new("true");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Literal(Literal::Bool(val)) => assert_eq!(*val, true),
                _ => panic!("Expected bool literal"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_bool_false() {
    let mut parser = Parser::new("false");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Literal(Literal::Bool(val)) => assert_eq!(*val, false),
                _ => panic!("Expected bool literal"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_unit_literal() {
    let mut parser = Parser::new("()");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Literal(Literal::Unit) => {},
                _ => panic!("Expected unit literal"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_f_string() {
    let mut parser = Parser::new(r#"f"Hello {name}""#);
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::StringInterpolation { .. } => {},
                _ => panic!("Expected string interpolation"),
            }
        },
        _ => panic!("Expected program"),
    }
}

// ==================== IDENTIFIER PARSING TESTS ====================

#[test]
fn test_parse_identifier() {
    let mut parser = Parser::new("variable_name");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Identifier(name) => assert_eq!(name, "variable_name"),
                _ => panic!("Expected identifier"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_underscore() {
    let mut parser = Parser::new("_");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Identifier(name) => assert_eq!(name, "_"),
                _ => panic!("Expected underscore identifier"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_fat_arrow_lambda() {
    let mut parser = Parser::new("x => x + 1");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Lambda { params, .. } => {
                    assert_eq!(params.len(), 1);
                },
                _ => panic!("Expected lambda expression"),
            }
        },
        _ => panic!("Expected program"),
    }
}

// ==================== UNARY OPERATOR TESTS ====================

#[test]
fn test_parse_unary_minus() {
    let mut parser = Parser::new("-42");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Unary { op: UnaryOp::Negate, operand } => {
                    match &operand.kind {
                        ExprKind::Literal(Literal::Integer(val)) => assert_eq!(*val, 42),
                        _ => panic!("Expected integer in unary minus"),
                    }
                },
                _ => panic!("Expected unary negation"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_unary_not() {
    let mut parser = Parser::new("!true");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Unary { op: UnaryOp::Not, operand } => {
                    match &operand.kind {
                        ExprKind::Literal(Literal::Bool(val)) => assert_eq!(*val, true),
                        _ => panic!("Expected bool in unary not"),
                    }
                },
                _ => panic!("Expected unary not"),
            }
        },
        _ => panic!("Expected program"),
    }
}

// ==================== PARENTHESES TESTS ====================

#[test]
fn test_parse_grouped_expression() {
    let mut parser = Parser::new("(1 + 2)");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Binary { .. } => {},
                _ => panic!("Expected binary expression"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_tuple_two_elements() {
    let mut parser = Parser::new("(1, 2)");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Tuple(elements) => assert_eq!(elements.len(), 2),
                _ => panic!("Expected tuple"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_tuple_three_elements() {
    let mut parser = Parser::new("(1, 2, 3)");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Tuple(elements) => assert_eq!(elements.len(), 3),
                _ => panic!("Expected tuple"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_tuple_trailing_comma() {
    let mut parser = Parser::new("(1, 2,)");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Tuple(elements) => assert_eq!(elements.len(), 2),
                _ => panic!("Expected tuple"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_single_element_tuple() {
    let mut parser = Parser::new("(1,)");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Tuple(elements) => assert_eq!(elements.len(), 1),
                _ => panic!("Expected single element tuple"),
            }
        },
        _ => panic!("Expected program"),
    }
}

// ==================== CONTROL FLOW TESTS ====================

#[test]
fn test_parse_if_expression() {
    let mut parser = Parser::new("if true { 1 } else { 2 }");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::If { .. } => {},
                _ => panic!("Expected if expression"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_match_expression() {
    let mut parser = Parser::new("match x { 1 => true, _ => false }");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Match { .. } => {},
                _ => panic!("Expected match expression"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_while_loop() {
    let mut parser = Parser::new("while x > 0 { x = x - 1 }");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::While { .. } => {},
                _ => panic!("Expected while loop"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_for_loop() {
    let mut parser = Parser::new("for x in [1, 2, 3] { print(x) }");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::For { .. } => {},
                _ => panic!("Expected for loop"),
            }
        },
        _ => panic!("Expected program"),
    }
}

// ==================== CONTROL STATEMENT TESTS ====================

#[test]
fn test_parse_break() {
    let mut parser = Parser::new("break");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Break { label } => assert!(label.is_none()),
                _ => panic!("Expected break statement"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_break_with_label() {
    let mut parser = Parser::new("break outer");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Break { label } => assert_eq!(label.as_deref(), Some("outer")),
                _ => panic!("Expected labeled break"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_continue() {
    let mut parser = Parser::new("continue");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Continue { label } => assert!(label.is_none()),
                _ => panic!("Expected continue statement"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_continue_with_label() {
    let mut parser = Parser::new("continue outer");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Continue { label } => assert_eq!(label.as_deref(), Some("outer")),
                _ => panic!("Expected labeled continue"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_return_empty() {
    let mut parser = Parser::new("return");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Return { value } => assert!(value.is_none()),
                _ => panic!("Expected return statement"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_return_with_value() {
    let mut parser = Parser::new("return 42");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Return { value } => assert!(value.is_some()),
                _ => panic!("Expected return with value"),
            }
        },
        _ => panic!("Expected program"),
    }
}

// ==================== CONSTRUCTOR TESTS ====================

#[test]
fn test_parse_some_constructor() {
    let mut parser = Parser::new("Some");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Identifier(name) => assert_eq!(name, "Some"),
                _ => panic!("Expected Some constructor"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_none_constructor() {
    let mut parser = Parser::new("None");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::None => {},
                _ => panic!("Expected None constructor"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_ok_constructor() {
    let mut parser = Parser::new("Ok");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Identifier(name) => assert_eq!(name, "Ok"),
                _ => panic!("Expected Ok constructor"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_err_constructor() {
    let mut parser = Parser::new("Err");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Identifier(name) => assert_eq!(name, "Err"),
                _ => panic!("Expected Err constructor"),
            }
        },
        _ => panic!("Expected program"),
    }
}

// ==================== COLLECTION TESTS ====================

#[test]
fn test_parse_empty_list() {
    let mut parser = Parser::new("[]");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::List(elements) => assert_eq!(elements.len(), 0),
                _ => panic!("Expected empty list"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_list_with_elements() {
    let mut parser = Parser::new("[1, 2, 3]");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::List(elements) => assert_eq!(elements.len(), 3),
                _ => panic!("Expected list with elements"),
            }
        },
        _ => panic!("Expected program"),
    }
}

// ==================== FUNCTION TESTS ====================

#[test]
fn test_parse_function_declaration() {
    let mut parser = Parser::new("fun add(x, y) { x + y }");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Function { name, params, .. } => {
                    assert_eq!(name.as_deref(), Some("add"));
                    assert_eq!(params.len(), 2);
                },
                _ => panic!("Expected function declaration"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_anonymous_function() {
    let mut parser = Parser::new("fn (x) { x * 2 }");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Function { name, params, .. } => {
                    assert!(name.is_none());
                    assert_eq!(params.len(), 1);
                },
                _ => panic!("Expected anonymous function"),
            }
        },
        _ => panic!("Expected program"),
    }
}

// ==================== BLOCK TESTS ====================

#[test]
fn test_parse_block_expression() {
    let mut parser = Parser::new("{ let x = 1; x + 2 }");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Block { .. } => {},
                _ => panic!("Expected block expression"),
            }
        },
        _ => panic!("Expected program"),
    }
}

// ==================== VARIABLE DECLARATION TESTS ====================

#[test]
fn test_parse_let_declaration() {
    let mut parser = Parser::new("let x = 42");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Let { .. } => {},
                _ => panic!("Expected let declaration"),
            }
        },
        _ => panic!("Expected program"),
    }
}

#[test]
fn test_parse_var_declaration() {
    let mut parser = Parser::new("var x = 42");
    let ast = parser.parse().unwrap();
    
    match &ast.kind {
        ExprKind::Program(exprs) => {
            assert_eq!(exprs.len(), 1);
            match &exprs[0].kind {
                ExprKind::Var { .. } => {},
                _ => panic!("Expected var declaration"),
            }
        },
        _ => panic!("Expected program"),
    }
}

// Run all tests with: cargo test expressions_parser_tdd --test expressions_parser_tdd