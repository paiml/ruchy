//! Deterministic Error Recovery for Ruchy Parser
//!
//! Based on docs/ruchy-transpiler-docs.md Section 4: Deterministic Error Recovery
//! Ensures predictable parser behavior on malformed input

use crate::frontend::ast::{Expr, ExprKind, Literal, Param, Span};

/// Synthetic error node that can be embedded in the AST
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorNode {
    /// The error message
    pub message: String,
    /// Source location of the error
    pub location: SourceLocation,
    /// The partial context that was successfully parsed
    pub context: ErrorContext,
    /// Recovery strategy used
    pub recovery: RecoveryStrategy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub file: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorContext {
    FunctionDecl {
        name: Option<String>,
        params: Option<Vec<Param>>,
        body: Option<Box<Expr>>,
    },
    LetBinding {
        name: Option<String>,
        value: Option<Box<Expr>>,
    },
    IfExpression {
        condition: Option<Box<Expr>>,
        then_branch: Option<Box<Expr>>,
        else_branch: Option<Box<Expr>>,
    },
    ArrayLiteral {
        elements: Vec<Expr>,
        error_at_index: usize,
    },
    BinaryOp {
        left: Option<Box<Expr>>,
        op: Option<String>,
        right: Option<Box<Expr>>,
    },
    StructLiteral {
        name: Option<String>,
        fields: Vec<(String, Expr)>,
        error_field: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    /// Skip tokens until a synchronization point
    SkipUntilSync,
    /// Insert a synthetic token
    InsertToken(String),
    /// Replace with a default value
    DefaultValue,
    /// Wrap partial parse in error node
    PartialParse,
    /// Panic mode - skip until statement boundary
    PanicMode,
}

/// Extension to Expr to support error nodes
#[derive(Debug, Clone, PartialEq)]
pub enum ExprWithError {
    Valid(Expr),
    Error(ErrorNode),
}

impl From<Expr> for ExprWithError {
    fn from(expr: Expr) -> Self {
        ExprWithError::Valid(expr)
    }
}

impl From<ErrorNode> for ExprWithError {
    fn from(error: ErrorNode) -> Self {
        ExprWithError::Error(error)
    }
}

/// Parser error recovery implementation
pub struct ErrorRecovery {
    /// Synchronization tokens for panic mode recovery
    sync_tokens: Vec<String>,
    /// Maximum errors before giving up
    max_errors: usize,
    /// Current error count
    error_count: usize,
}

impl Default for ErrorRecovery {
    fn default() -> Self {
        Self {
            sync_tokens: vec![
                ";".to_string(),
                "}".to_string(),
                "fun".to_string(),
                "let".to_string(),
                "if".to_string(),
                "for".to_string(),
                "while".to_string(),
                "return".to_string(),
                "struct".to_string(),
                "enum".to_string(),
            ],
            max_errors: 100,
            error_count: 0,
        }
    }
}

impl ErrorRecovery {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a synthetic error node for missing function name
    pub fn missing_function_name(&mut self, location: SourceLocation) -> ErrorNode {
        self.error_count += 1;
        ErrorNode {
            message: "expected function name".to_string(),
            location,
            context: ErrorContext::FunctionDecl {
                name: None,
                params: None,
                body: None,
            },
            recovery: RecoveryStrategy::InsertToken("error_fn".to_string()),
        }
    }

    /// Create a synthetic error node for missing function parameters
    pub fn missing_function_params(&mut self, name: String, location: SourceLocation) -> ErrorNode {
        self.error_count += 1;
        ErrorNode {
            message: "expected function parameters".to_string(),
            location,
            context: ErrorContext::FunctionDecl {
                name: Some(name),
                params: None,
                body: None,
            },
            recovery: RecoveryStrategy::DefaultValue,
        }
    }

    /// Create a synthetic error node for missing function body
    pub fn missing_function_body(
        &mut self,
        name: String,
        params: Vec<Param>,
        location: SourceLocation,
    ) -> ErrorNode {
        self.error_count += 1;
        ErrorNode {
            message: "expected function body".to_string(),
            location,
            context: ErrorContext::FunctionDecl {
                name: Some(name),
                params: Some(params),
                body: None,
            },
            recovery: RecoveryStrategy::InsertToken("{ /* missing body */ }".to_string()),
        }
    }

    /// Create error node for malformed let binding
    pub fn malformed_let_binding(
        &mut self,
        partial_name: Option<String>,
        partial_value: Option<Box<Expr>>,
        location: SourceLocation,
    ) -> ErrorNode {
        self.error_count += 1;
        ErrorNode {
            message: "malformed let binding".to_string(),
            location,
            context: ErrorContext::LetBinding {
                name: partial_name,
                value: partial_value,
            },
            recovery: RecoveryStrategy::PartialParse,
        }
    }

    /// Create error node for incomplete if expression
    pub fn incomplete_if_expr(
        &mut self,
        condition: Option<Box<Expr>>,
        then_branch: Option<Box<Expr>>,
        location: SourceLocation,
    ) -> ErrorNode {
        self.error_count += 1;
        ErrorNode {
            message: "incomplete if expression".to_string(),
            location,
            context: ErrorContext::IfExpression {
                condition,
                then_branch,
                else_branch: None,
            },
            recovery: RecoveryStrategy::DefaultValue,
        }
    }

    /// Check if we should continue parsing or give up
    #[must_use]
    pub fn should_continue(&self) -> bool {
        self.error_count < self.max_errors
    }

    /// Reset error count for new parsing session
    pub fn reset(&mut self) {
        self.error_count = 0;
    }

    /// Check if token is a synchronization point
    #[must_use]
    pub fn is_sync_token(&self, token: &str) -> bool {
        self.sync_tokens.contains(&token.to_string())
    }

    /// Skip tokens until we find a synchronization point
    pub fn skip_until_sync<'a, I>(&self, tokens: &mut I) -> Option<String>
    where
        I: Iterator<Item = &'a str>,
    {
        for token in tokens {
            if self.is_sync_token(token) {
                return Some(token.to_string());
            }
        }
        None
    }
}

/// Error recovery rules for different contexts
pub struct RecoveryRules;

impl RecoveryRules {
    /// Determine recovery strategy based on context
    #[must_use]
    pub fn select_strategy(context: &ErrorContext) -> RecoveryStrategy {
        match context {
            ErrorContext::FunctionDecl { name, params, body } => {
                if name.is_none() {
                    RecoveryStrategy::InsertToken("error_fn".to_string())
                } else if params.is_none() {
                    RecoveryStrategy::DefaultValue
                } else if body.is_none() {
                    RecoveryStrategy::InsertToken("{ }".to_string())
                } else {
                    RecoveryStrategy::PartialParse
                }
            }
            ErrorContext::LetBinding { .. } => RecoveryStrategy::SkipUntilSync,
            ErrorContext::IfExpression { .. } => RecoveryStrategy::DefaultValue,
            ErrorContext::ArrayLiteral { .. } | ErrorContext::StructLiteral { .. } => {
                RecoveryStrategy::PartialParse
            }
            ErrorContext::BinaryOp { .. } => RecoveryStrategy::PanicMode,
        }
    }

    /// Generate synthetic AST for error recovery
    #[must_use]
    pub fn synthesize_ast(error: &ErrorNode) -> Expr {
        let default_span = Span::new(0, 0);
        match &error.context {
            ErrorContext::FunctionDecl { .. } => {
                // Return a synthetic function that does nothing
                Expr::new(
                    ExprKind::Lambda {
                        params: vec![],
                        body: Box::new(Expr::new(ExprKind::Literal(Literal::Unit), default_span)),
                    },
                    default_span,
                )
            }
            ErrorContext::LetBinding { name, value } => {
                // Create a let with whatever we could parse
                Expr::new(
                    ExprKind::Let {
                        name: name.clone().unwrap_or_else(|| "_error".to_string()),
                        value: value.clone().unwrap_or_else(|| {
                            Box::new(Expr::new(ExprKind::Literal(Literal::Unit), default_span))
                        }),
                        body: Box::new(Expr::new(ExprKind::Literal(Literal::Unit), default_span)),
                        is_mutable: false,
                    },
                    default_span,
                )
            }
            ErrorContext::IfExpression {
                condition,
                then_branch,
                ..
            } => {
                // Create an if with defaults for missing parts
                Expr::new(
                    ExprKind::If {
                        condition: condition.clone().unwrap_or_else(|| {
                            Box::new(Expr::new(
                                ExprKind::Literal(Literal::Bool(false)),
                                default_span,
                            ))
                        }),
                        then_branch: then_branch.clone().unwrap_or_else(|| {
                            Box::new(Expr::new(ExprKind::Literal(Literal::Unit), default_span))
                        }),
                        else_branch: Some(Box::new(Expr::new(
                            ExprKind::Literal(Literal::Unit),
                            default_span,
                        ))),
                    },
                    default_span,
                )
            }
            ErrorContext::ArrayLiteral { elements, .. } => {
                // Return partial array with valid elements
                Expr::new(ExprKind::List(elements.clone()), default_span)
            }
            ErrorContext::BinaryOp { left, .. } => {
                // Return left side if available, otherwise unit
                if let Some(left) = left {
                    *left.clone()
                } else {
                    Expr::new(ExprKind::Literal(Literal::Unit), default_span)
                }
            }
            ErrorContext::StructLiteral { name, fields, .. } => {
                // Return struct with partial fields
                if let Some(name) = name {
                    Expr::new(
                        ExprKind::StructLiteral {
                            name: name.clone(),
                            fields: fields.clone(),
                        },
                        default_span,
                    )
                } else {
                    Expr::new(ExprKind::Literal(Literal::Unit), default_span)
                }
            }
        }
    }
}

/// Integration with parser
pub trait ErrorRecoverable {
    /// Try to recover from parse error
    fn recover_from_error(&mut self, error: ErrorNode) -> Option<Expr>;

    /// Check if we're in a recoverable state
    fn can_recover(&self) -> bool;

    /// Get current error nodes
    fn get_errors(&self) -> Vec<ErrorNode>;
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recovery_creation() {
        let mut recovery = ErrorRecovery::new();

        let error = recovery.missing_function_name(SourceLocation {
            line: 1,
            column: 5,
            file: None,
        });

        assert_eq!(error.message, "expected function name");
        assert_eq!(recovery.error_count, 1);
        assert!(recovery.should_continue());
    }

    #[test]
    fn test_recovery_strategy_selection() {
        let context = ErrorContext::FunctionDecl {
            name: None,
            params: None,
            body: None,
        };

        let strategy = RecoveryRules::select_strategy(&context);

        match strategy {
            RecoveryStrategy::InsertToken(token) => {
                assert_eq!(token, "error_fn");
            }
            _ => panic!("Expected InsertToken strategy"),
        }
    }

    #[test]
    fn test_synthetic_ast_generation() {
        let error = ErrorNode {
            message: "test error".to_string(),
            location: SourceLocation {
                line: 1,
                column: 1,
                file: None,
            },
            context: ErrorContext::LetBinding {
                name: Some("x".to_string()),
                value: None,
            },
            recovery: RecoveryStrategy::DefaultValue,
        };

        let ast = RecoveryRules::synthesize_ast(&error);

        match ast.kind {
            ExprKind::Let { name, value, .. } => {
                assert_eq!(name, "x");
                match value.kind {
                    ExprKind::Literal(Literal::Unit) => {}
                    _ => panic!("Expected Unit value"),
                }
            }
            _ => panic!("Expected Let expression"),
        }
    }

    #[test]
    fn test_sync_token_detection() {
        let recovery = ErrorRecovery::new();

        assert!(recovery.is_sync_token(";"));
        assert!(recovery.is_sync_token("fun"));
        assert!(recovery.is_sync_token("let"));
        assert!(!recovery.is_sync_token("="));
        assert!(!recovery.is_sync_token("+"));
    }

    #[test]
    fn test_max_errors_limit() {
        let mut recovery = ErrorRecovery::new();
        recovery.max_errors = 3;

        for i in 0..5 {
            if recovery.should_continue() {
                recovery.missing_function_name(SourceLocation {
                    line: i,
                    column: 0,
                    file: None,
                });
            }
        }

        assert_eq!(recovery.error_count, 3);
        assert!(!recovery.should_continue());
    }
}
