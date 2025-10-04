//! Sprint 81: Test macros module to boost coverage

use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span};
use ruchy::macros::{MacroExpander, MacroRegistry};

#[test]
fn test_macro_registry() {
    let registry = MacroRegistry::new();

    // Check has_macro
    assert!(!registry.has_macro("undefined_macro"));
    assert!(!registry.has_macro("test_macro"));

    // Check registration
    let ast = Expr {
        kind: ExprKind::Literal(Literal::Integer(42)),
        span: Span::default(),
        attributes: vec![],
    };
    let result = registry.register_from_ast(&ast);
    assert!(result.is_ok());

    // After registration, test macro check
    assert!(registry.has_macro("say_hello"));
}

#[test]
fn test_macro_registry_default() {
    let _registry = MacroRegistry::default();
}

#[test]
fn test_macro_expander() {
    let expander = MacroExpander::new();

    // Check expanding a regular expression
    let ast = Expr {
        kind: ExprKind::Literal(Literal::Integer(42)),
        span: Span::default(),
        attributes: vec![],
    };
    let result = expander.expand(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_macro_expander_default() {
    let _expander = MacroExpander::default();
}

#[test]
fn test_builtin_macro_stringify() {
    let expander = MacroExpander::new();

    // Check stringify! macro
    let ast = Expr {
        kind: ExprKind::MacroInvocation {
            name: "stringify".to_string(),
            args: vec![],
        },
        span: Span::default(),
        attributes: vec![],
    };

    let result = expander.expand(&ast);
    assert!(result.is_ok());

    if let Ok(expanded) = result {
        match &expanded.kind {
            ExprKind::Literal(Literal::String(s)) => {
                assert_eq!(s, "hello + world");
            }
            _ => panic!("Expected string literal"),
        }
    }
}

#[test]
fn test_builtin_macro_line() {
    let expander = MacroExpander::new();

    // Check line! macro
    let ast = Expr {
        kind: ExprKind::MacroInvocation {
            name: "line".to_string(),
            args: vec![],
        },
        span: Span::default(),
        attributes: vec![],
    };

    let result = expander.expand(&ast);
    assert!(result.is_ok());

    if let Ok(expanded) = result {
        match &expanded.kind {
            ExprKind::Literal(Literal::Integer(n)) => {
                assert_eq!(*n, 42);
            }
            _ => panic!("Expected integer literal"),
        }
    }
}

#[test]
fn test_builtin_macro_file() {
    let expander = MacroExpander::new();

    // Check file! macro
    let ast = Expr {
        kind: ExprKind::MacroInvocation {
            name: "file".to_string(),
            args: vec![],
        },
        span: Span::default(),
        attributes: vec![],
    };

    let result = expander.expand(&ast);
    assert!(result.is_ok());

    if let Ok(expanded) = result {
        match &expanded.kind {
            ExprKind::Literal(Literal::String(s)) => {
                assert_eq!(s, "test.ruchy");
            }
            _ => panic!("Expected string literal"),
        }
    }
}

#[test]
fn test_unknown_macro() {
    let expander = MacroExpander::new();

    // Check unknown macro (should return unchanged)
    let ast = Expr {
        kind: ExprKind::MacroInvocation {
            name: "unknown_macro".to_string(),
            args: vec![],
        },
        span: Span::default(),
        attributes: vec![],
    };

    let result = expander.expand(&ast);
    assert!(result.is_ok());

    if let Ok(expanded) = result {
        match &expanded.kind {
            ExprKind::MacroInvocation { name, .. } => {
                assert_eq!(name, "unknown_macro");
            }
            _ => panic!("Expected macro invocation to remain unchanged"),
        }
    }
}

#[test]
fn test_macro_expansion_with_different_ast_kinds() {
    let expander = MacroExpander::new();

    // Check with identifier
    let ast = Expr {
        kind: ExprKind::Identifier("test".to_string()),
        span: Span::default(),
        attributes: vec![],
    };
    let result = expander.expand(&ast);
    assert!(result.is_ok());

    // Check with return
    let ast = Expr {
        kind: ExprKind::Return { value: None },
        span: Span::default(),
        attributes: vec![],
    };
    let result = expander.expand(&ast);
    assert!(result.is_ok());
}
