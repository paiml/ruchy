//! Tests for AST (Abstract Syntax Tree) structures

use ruchy::frontend::ast::*;

#[test]
fn test_span_creation() {
    let span = Span::new(0, 10);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 10);
}

#[test]
fn test_span_merge() {
    let span1 = Span::new(0, 5);
    let span2 = Span::new(3, 10);
    let merged = span1.merge(span2);
    
    assert_eq!(merged.start, 0);
    assert_eq!(merged.end, 10);
}

#[test]
fn test_span_merge_non_overlapping() {
    let span1 = Span::new(10, 15);
    let span2 = Span::new(0, 5);
    let merged = span1.merge(span2);
    
    assert_eq!(merged.start, 0);
    assert_eq!(merged.end, 15);
}

#[test]
fn test_literal_integer() {
    let lit = Literal::Integer(42);
    assert!(matches!(lit, Literal::Integer(42)));
}

#[test]
fn test_literal_float() {
    let lit = Literal::Float(3.14);
    match lit {
        Literal::Float(f) => assert!((f - 3.14).abs() < 0.001),
        _ => panic!("Expected float literal"),
    }
}

#[test]
fn test_literal_string() {
    let lit = Literal::String("hello".to_string());
    assert!(matches!(lit, Literal::String(ref s) if s == "hello"));
}

#[test]
fn test_literal_bool() {
    let true_lit = Literal::Bool(true);
    let false_lit = Literal::Bool(false);
    
    assert!(matches!(true_lit, Literal::Bool(true)));
    assert!(matches!(false_lit, Literal::Bool(false)));
}

#[test]
fn test_literal_char() {
    let lit = Literal::Char('a');
    assert!(matches!(lit, Literal::Char('a')));
}

#[test]
fn test_expr_creation() {
    let expr = Expr::new(
        ExprKind::Literal(Literal::Integer(42)),
        Span::new(0, 2),
    );
    
    assert!(matches!(expr.kind, ExprKind::Literal(Literal::Integer(42))));
    assert_eq!(expr.span.start, 0);
    assert_eq!(expr.span.end, 2);
    assert!(expr.attributes.is_empty());
}

#[test]
fn test_expr_with_attributes() {
    let attrs = vec![
        Attribute {
            name: "test".to_string(),
            args: vec![],
        },
    ];
    
    let expr = Expr::with_attributes(
        ExprKind::Literal(Literal::Bool(true)),
        Span::new(0, 4),
        attrs.clone(),
    );
    
    assert!(matches!(expr.kind, ExprKind::Literal(Literal::Bool(true))));
    assert_eq!(expr.attributes.len(), 1);
    assert_eq!(expr.attributes[0].name, "test");
}

#[test]
fn test_binary_op_variants() {
    let ops = vec![
        BinaryOp::Add,
        BinaryOp::Subtract,
        BinaryOp::Multiply,
        BinaryOp::Divide,
        BinaryOp::Modulo,
        BinaryOp::Power,
        BinaryOp::Equal,
        BinaryOp::NotEqual,
        BinaryOp::Less,
        BinaryOp::LessEqual,
        BinaryOp::Greater,
        BinaryOp::GreaterEqual,
        BinaryOp::And,
        BinaryOp::Or,
        BinaryOp::BitwiseAnd,
        BinaryOp::BitwiseOr,
        BinaryOp::BitwiseXor,
        BinaryOp::LeftShift,
        BinaryOp::RightShift,
        BinaryOp::Pipeline,
    ];
    
    // Just ensure all variants exist
    assert_eq!(ops.len(), 20);
}

#[test]
fn test_unary_op_variants() {
    let ops = vec![
        UnaryOp::Not,
        UnaryOp::Negate,
        UnaryOp::BitwiseNot,
        UnaryOp::Reference,
    ];
    
    assert_eq!(ops.len(), 4);
}

#[test]
fn test_pattern_identifier() {
    let pattern = Pattern::Identifier("x".to_string());
    
    match pattern {
        Pattern::Identifier(name) => assert_eq!(name, "x"),
        _ => panic!("Expected identifier pattern"),
    }
}

#[test]
fn test_pattern_wildcard() {
    let pattern = Pattern::Wildcard;
    assert!(matches!(pattern, Pattern::Wildcard));
}

#[test]
fn test_pattern_literal() {
    let pattern = Pattern::Literal(Literal::Integer(42));
    
    match pattern {
        Pattern::Literal(Literal::Integer(n)) => assert_eq!(n, 42),
        _ => panic!("Expected literal pattern"),
    }
}

#[test]
fn test_pattern_tuple() {
    let patterns = vec![
        Pattern::Identifier("x".to_string()),
        Pattern::Identifier("y".to_string()),
    ];
    let pattern = Pattern::Tuple(patterns);
    
    match pattern {
        Pattern::Tuple(pats) => {
            assert_eq!(pats.len(), 2);
            match &pats[0] {
                Pattern::Identifier(name) => assert_eq!(name, "x"),
                _ => panic!("Expected identifier in tuple"),
            }
        }
        _ => panic!("Expected tuple pattern"),
    }
}

#[test]
fn test_pattern_struct() {
    let fields = vec![
        ("x".to_string(), Pattern::Identifier("a".to_string())),
        ("y".to_string(), Pattern::Wildcard),
    ];
    let pattern = Pattern::Struct("Point".to_string(), fields);
    
    match pattern {
        Pattern::Struct(name, f) => {
            assert_eq!(name, "Point");
            assert_eq!(f.len(), 2);
            assert_eq!(f[0].0, "x");
        }
        _ => panic!("Expected struct pattern"),
    }
}

#[test]
fn test_type_basic_variants() {
    let int_type = Type::Int;
    let float_type = Type::Float;
    let bool_type = Type::Bool;
    let string_type = Type::String;
    let char_type = Type::Char;
    let unit_type = Type::Unit;
    
    assert!(matches!(int_type, Type::Int));
    assert!(matches!(float_type, Type::Float));
    assert!(matches!(bool_type, Type::Bool));
    assert!(matches!(string_type, Type::String));
    assert!(matches!(char_type, Type::Char));
    assert!(matches!(unit_type, Type::Unit));
}

#[test]
fn test_type_array() {
    let array_type = Type::Array(Box::new(Type::Int));
    
    match array_type {
        Type::Array(elem) => {
            assert!(matches!(*elem, Type::Int));
        }
        _ => panic!("Expected array type"),
    }
}

#[test]
fn test_type_tuple() {
    let tuple_type = Type::Tuple(vec![Type::Int, Type::Bool]);
    
    match tuple_type {
        Type::Tuple(elems) => {
            assert_eq!(elems.len(), 2);
            assert!(matches!(elems[0], Type::Int));
            assert!(matches!(elems[1], Type::Bool));
        }
        _ => panic!("Expected tuple type"),
    }
}

#[test]
fn test_type_function() {
    let func_type = Type::Function(
        Box::new(Type::Tuple(vec![Type::Int, Type::Int])),
        Box::new(Type::Int),
    );
    
    match func_type {
        Type::Function(params, ret) => {
            match *params {
                Type::Tuple(p) => assert_eq!(p.len(), 2),
                _ => panic!("Expected tuple params"),
            }
            assert!(matches!(*ret, Type::Int));
        }
        _ => panic!("Expected function type"),
    }
}

#[test]
fn test_type_option() {
    let option_type = Type::Option(Box::new(Type::String));
    
    match option_type {
        Type::Option(inner) => {
            assert!(matches!(*inner, Type::String));
        }
        _ => panic!("Expected option type"),
    }
}

#[test]
fn test_type_result() {
    let result_type = Type::Result(Box::new(Type::Int), Box::new(Type::String));
    
    match result_type {
        Type::Result(ok, err) => {
            assert!(matches!(*ok, Type::Int));
            assert!(matches!(*err, Type::String));
        }
        _ => panic!("Expected result type"),
    }
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn prop_span_merge_commutative(
            start1 in 0usize..1000,
            end1 in 0usize..1000,
            start2 in 0usize..1000,
            end2 in 0usize..1000,
        ) {
            let span1 = Span { start: start1, end: end1 };
            let span2 = Span { start: start2, end: end2 };
            
            let merge1 = span1.merge(span2);
            let merge2 = span2.merge(span1);
            
            prop_assert_eq!(merge1.start, merge2.start);
            prop_assert_eq!(merge1.end, merge2.end);
        }
        
        #[test]
        fn prop_literal_integer_roundtrip(n in i64::MIN..i64::MAX) {
            let lit = Literal::Integer(n);
            match lit {
                Literal::Integer(m) => prop_assert_eq!(m, n),
                _ => prop_assert!(false),
            }
        }
        
        #[test]
        fn prop_pattern_identifier(name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}") {
            let pattern = Pattern::Identifier(name.clone());
            match pattern {
                Pattern::Identifier(n) => prop_assert_eq!(n, name),
                _ => prop_assert!(false),
            }
        }
    }
}