//! Simple TDD tests for frontend modules
//! Target: Test AST and related with minimal complexity ≤5

#[cfg(test)]
mod tests {
    use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span};
    
    // Test Span operations (complexity: 2 each)
    #[test]
    fn test_span_creation() {
        let span = Span::new(0, 5);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 5);
    }
    
    #[test]
    fn test_span_length() {
        let span = Span::new(10, 20);
        assert_eq!(span.end - span.start, 10);
    }
    
    #[test]
    fn test_span_contains() {
        let span = Span::new(5, 15);
        assert!(span.start <= 10 && 10 < span.end); // Manual contains check
        assert!(!(span.start <= 3 && 3 < span.end));
        assert!(!(span.start <= 20 && 20 < span.end));
    }
    
    #[test]
    fn test_span_is_empty() {
        let empty_span = Span::new(5, 5);
        let non_empty_span = Span::new(5, 10);
        assert_eq!(empty_span.start, empty_span.end);
        assert_ne!(non_empty_span.start, non_empty_span.end);
    }
    
    // Test literal patterns (complexity: 2 each)
    #[test]
    fn test_literal_integer_match() {
        let lit = Literal::Integer(42);
        match lit {
            Literal::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected integer literal"),
        }
    }
    
    #[test]
    fn test_literal_float_match() {
        let lit = Literal::Float(3.14);
        match lit {
            Literal::Float(f) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected float literal"),
        }
    }
    
    #[test]
    fn test_literal_string_match() {
        let lit = Literal::String("hello".to_string());
        match lit {
            Literal::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string literal"),
        }
    }
    
    #[test]
    fn test_literal_bool_match() {
        let lit_true = Literal::Bool(true);
        let lit_false = Literal::Bool(false);
        
        match lit_true {
            Literal::Bool(b) => assert_eq!(b, true),
            _ => panic!("Expected bool literal"),
        }
        
        match lit_false {
            Literal::Bool(b) => assert_eq!(b, false),
            _ => panic!("Expected bool literal"),
        }
    }
    
    // Test expression kinds (complexity: 3 each)
    #[test]
    fn test_expr_kind_literal() {
        let expr = Expr {
            kind: ExprKind::Literal(Literal::Integer(42)),
            span: Span::new(0, 2),
            attributes: vec![],
        };
        
        match expr.kind {
            ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 42),
            _ => panic!("Expected literal expression"),
        }
    }
    
    #[test]
    fn test_expr_kind_identifier() {
        let expr = Expr {
            kind: ExprKind::Identifier("variable".to_string()),
            span: Span::new(0, 8),
            attributes: vec![],
        };
        
        match expr.kind {
            ExprKind::Identifier(name) => assert_eq!(name, "variable"),
            _ => panic!("Expected identifier expression"),
        }
    }
    
    #[test]
    fn test_expr_span_access() {
        let expr = Expr {
            kind: ExprKind::Literal(Literal::Integer(42)),
            span: Span::new(0, 2),
            attributes: vec![],
        };
        
        assert_eq!(expr.span.start, 0);
        assert_eq!(expr.span.end, 2);
        assert_eq!(expr.span.end - expr.span.start, 2);
    }
    
    #[test]
    fn test_expr_attributes() {
        let expr = Expr {
            kind: ExprKind::Literal(Literal::Integer(42)),
            span: Span::new(0, 2),
            attributes: vec![],
        };
        
        assert!(expr.attributes.is_empty());
    }
    
    // Test cloning and equality (complexity: 2 each)
    #[test]
    fn test_span_clone() {
        let span = Span::new(5, 10);
        let cloned = span.clone();
        assert_eq!(span.start, cloned.start);
        assert_eq!(span.end, cloned.end);
    }
    
    #[test]
    fn test_span_equality() {
        let span1 = Span::new(5, 10);
        let span2 = Span::new(5, 10);
        let span3 = Span::new(5, 11);
        
        assert_eq!(span1, span2);
        assert_ne!(span1, span3);
    }
    
    #[test]
    fn test_literal_clone() {
        let lit = Literal::Integer(42);
        let cloned = lit.clone();
        
        match (&lit, &cloned) {
            (Literal::Integer(a), Literal::Integer(b)) => assert_eq!(a, b),
            _ => panic!("Clone failed"),
        }
    }
    
    #[test]
    fn test_literal_equality() {
        let lit1 = Literal::Integer(42);
        let lit2 = Literal::Integer(42);
        let lit3 = Literal::Integer(43);
        
        assert_eq!(lit1, lit2);
        assert_ne!(lit1, lit3);
    }
    
    // Test different literal types (complexity: 2 each)
    #[test]
    fn test_literal_types() {
        let int_lit = Literal::Integer(42);
        let float_lit = Literal::Float(3.14);
        let string_lit = Literal::String("hello".to_string());
        let bool_lit = Literal::Bool(true);
        
        // Test that they're different types
        assert_ne!(
            std::mem::discriminant(&int_lit),
            std::mem::discriminant(&float_lit)
        );
        
        assert_ne!(
            std::mem::discriminant(&string_lit),
            std::mem::discriminant(&bool_lit)
        );
    }
    
    #[test]
    fn test_expr_kind_types() {
        let literal_expr = ExprKind::Literal(Literal::Integer(42));
        let identifier_expr = ExprKind::Identifier("x".to_string());
        
        // Test discriminants are different
        assert_ne!(
            std::mem::discriminant(&literal_expr),
            std::mem::discriminant(&identifier_expr)
        );
    }
    
    // Test more span operations (complexity: 3 each)
    #[test]
    fn test_span_ordering() {
        let span1 = Span::new(5, 10);
        let span2 = Span::new(15, 20);
        
        // Test that span1 comes before span2
        assert!(span1.start < span2.start);
        assert!(span1.end < span2.start);
    }
    
    #[test]
    fn test_span_overlapping() {
        let span1 = Span::new(5, 15);
        let span2 = Span::new(10, 20);
        
        // Manual overlap check
        assert!(span1.end > span2.start);
        assert!(span2.start < span1.end);
    }
    
    #[test]
    fn test_span_adjacent() {
        let span1 = Span::new(5, 10);
        let span2 = Span::new(10, 15);
        
        // Test adjacent spans
        assert_eq!(span1.end, span2.start);
    }
    
    // Debug formatting tests (complexity: 2 each)
    #[test]
    fn test_span_debug() {
        let span = Span::new(5, 10);
        let debug_str = format!("{:?}", span);
        assert!(debug_str.contains("5"));
        assert!(debug_str.contains("10"));
    }
    
    #[test]
    fn test_literal_debug() {
        let lit = Literal::Integer(42);
        let debug_str = format!("{:?}", lit);
        assert!(debug_str.contains("42"));
    }
}