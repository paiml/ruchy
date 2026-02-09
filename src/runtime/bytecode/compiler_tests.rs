use super::*;

// Shared test helpers

fn make_test_param(name: &str) -> crate::frontend::ast::Param {
    crate::frontend::ast::Param {
        pattern: crate::frontend::ast::Pattern::Identifier(name.to_string()),
        ty: crate::frontend::ast::Type {
            kind: crate::frontend::ast::TypeKind::Named("Any".to_string()),
            span: crate::frontend::ast::Span::default(),
        },
        span: crate::frontend::ast::Span::default(),
        is_mutable: false,
        default_value: None,
    }
}

fn make_test_param_with_default(name: &str, default: Expr) -> crate::frontend::ast::Param {
    crate::frontend::ast::Param {
        pattern: crate::frontend::ast::Pattern::Identifier(name.to_string()),
        ty: crate::frontend::ast::Type {
            kind: crate::frontend::ast::TypeKind::Named("Any".to_string()),
            span: crate::frontend::ast::Span::default(),
        },
        span: crate::frontend::ast::Span::default(),
        is_mutable: false,
        default_value: Some(Box::new(default)),
    }
}

#[path = "compiler_tests_part1.rs"]
mod part1;

#[path = "compiler_tests_part2.rs"]
mod part2;
