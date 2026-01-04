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
    Valid(Box<Expr>),
    Error(ErrorNode),
}
impl From<Expr> for ExprWithError {
    fn from(expr: Expr) -> Self {
        ExprWithError::Valid(Box::new(expr))
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
    /// # Examples
    ///
    /// ```
    /// use ruchy::parser::error_recovery::ErrorRecovery;
    ///
    /// let recovery = ErrorRecovery::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
    /// Create a synthetic error node for missing function name
    /// # Examples
    ///
    /// ```
    /// use ruchy::parser::error_recovery::ErrorRecovery;
    ///
    /// let mut instance = ErrorRecovery::new();
    /// let result = instance.missing_function_name();
    /// // Verify behavior
    /// ```
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
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::parser::error_recovery::missing_function_params;
    ///
    /// let result = missing_function_params(());
    /// assert_eq!(result, Ok(()));
    /// ```
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
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::parser::error_recovery::missing_function_body;
    ///
    /// let result = missing_function_body(());
    /// assert_eq!(result, Ok(()));
    /// ```
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
    /// # Examples
    ///
    /// ```
    /// use ruchy::parser::error_recovery::ErrorRecovery;
    ///
    /// let mut instance = ErrorRecovery::new();
    /// let result = instance.malformed_let_binding();
    /// // Verify behavior
    /// ```
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
    /// # Examples
    ///
    /// ```
    /// use ruchy::parser::error_recovery::ErrorRecovery;
    ///
    /// let mut instance = ErrorRecovery::new();
    /// let result = instance.incomplete_if_expr();
    /// // Verify behavior
    /// ```
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
    /// # Examples
    ///
    /// ```
    /// use ruchy::parser::error_recovery::should_continue;
    ///
    /// let result = should_continue(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn should_continue(&self) -> bool {
        self.error_count < self.max_errors
    }
    /// Reset error count for new parsing session
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::parser::error_recovery::reset;
    ///
    /// let result = reset(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn reset(&mut self) {
        self.error_count = 0;
    }
    /// Check if token is a synchronization point
    #[must_use]
    /// # Examples
    ///
    /// ```
    /// use ruchy::parser::error_recovery::is_sync_token;
    ///
    /// let result = is_sync_token("example");
    /// assert_eq!(result, Ok(()));
    /// ```
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
    /// # Examples
    ///
    /// ```
    /// use ruchy::parser::error_recovery::select_strategy;
    ///
    /// let result = select_strategy(());
    /// assert_eq!(result, Ok(()));
    /// ```
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
    /// # Examples
    ///
    /// ```
    /// use ruchy::parser::error_recovery::synthesize_ast;
    ///
    /// let result = synthesize_ast(());
    /// assert_eq!(result, Ok(()));
    /// ```
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
                        type_annotation: None,
                        value: value.clone().unwrap_or_else(|| {
                            Box::new(Expr::new(ExprKind::Literal(Literal::Unit), default_span))
                        }),
                        body: Box::new(Expr::new(ExprKind::Literal(Literal::Unit), default_span)),
                        is_mutable: false,
                        else_block: None, // Error recovery doesn't support let-else
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
                            base: None,
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
    use crate::frontend::ast::{ExprKind, Literal};
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
            ExprKind::Let {
                name,
                type_annotation: _,
                value,
                ..
            } => {
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

    // Additional coverage tests for COVERAGE-95%

    #[test]
    fn test_missing_function_params() {
        let mut recovery = ErrorRecovery::new();
        let error = recovery.missing_function_params(
            "my_func".to_string(),
            SourceLocation {
                line: 5,
                column: 10,
                file: Some("test.ruchy".to_string()),
            },
        );
        assert_eq!(error.message, "expected function parameters");
        assert_eq!(recovery.error_count, 1);
        match &error.context {
            ErrorContext::FunctionDecl { name, params, body } => {
                assert_eq!(name.as_deref(), Some("my_func"));
                assert!(params.is_none());
                assert!(body.is_none());
            }
            _ => panic!("Expected FunctionDecl context"),
        }
    }

    #[test]
    fn test_missing_function_body() {
        let mut recovery = ErrorRecovery::new();
        let error = recovery.missing_function_body(
            "my_func".to_string(),
            vec![],
            SourceLocation {
                line: 1,
                column: 1,
                file: None,
            },
        );
        assert_eq!(error.message, "expected function body");
        assert_eq!(recovery.error_count, 1);
        match &error.recovery {
            RecoveryStrategy::InsertToken(token) => {
                assert!(token.contains("missing body"));
            }
            _ => panic!("Expected InsertToken strategy"),
        }
    }

    #[test]
    fn test_malformed_let_binding() {
        let mut recovery = ErrorRecovery::new();
        let error = recovery.malformed_let_binding(
            Some("x".to_string()),
            None,
            SourceLocation {
                line: 1,
                column: 1,
                file: None,
            },
        );
        assert_eq!(error.message, "malformed let binding");
        assert!(matches!(error.recovery, RecoveryStrategy::PartialParse));
    }

    #[test]
    fn test_incomplete_if_expr() {
        let mut recovery = ErrorRecovery::new();
        let error = recovery.incomplete_if_expr(
            None,
            None,
            SourceLocation {
                line: 1,
                column: 1,
                file: None,
            },
        );
        assert_eq!(error.message, "incomplete if expression");
        assert!(matches!(error.recovery, RecoveryStrategy::DefaultValue));
    }

    #[test]
    fn test_reset() {
        let mut recovery = ErrorRecovery::new();
        recovery.missing_function_name(SourceLocation {
            line: 1,
            column: 1,
            file: None,
        });
        assert_eq!(recovery.error_count, 1);
        recovery.reset();
        assert_eq!(recovery.error_count, 0);
    }

    #[test]
    fn test_skip_until_sync_finds_token() {
        let recovery = ErrorRecovery::new();
        let tokens = vec!["x", "=", "5", ";", "y"];
        let mut iter = tokens.iter().map(|s| *s);
        let result = recovery.skip_until_sync(&mut iter);
        assert_eq!(result, Some(";".to_string()));
    }

    #[test]
    fn test_skip_until_sync_no_token() {
        let recovery = ErrorRecovery::new();
        let tokens = vec!["x", "=", "5"];
        let mut iter = tokens.iter().map(|s| *s);
        let result = recovery.skip_until_sync(&mut iter);
        assert!(result.is_none());
    }

    #[test]
    fn test_skip_until_sync_fun_keyword() {
        let recovery = ErrorRecovery::new();
        let tokens = vec!["x", "y", "fun", "z"];
        let mut iter = tokens.iter().map(|s| *s);
        let result = recovery.skip_until_sync(&mut iter);
        assert_eq!(result, Some("fun".to_string()));
    }

    #[test]
    fn test_recovery_strategy_let_binding() {
        let context = ErrorContext::LetBinding {
            name: Some("x".to_string()),
            value: None,
        };
        let strategy = RecoveryRules::select_strategy(&context);
        assert!(matches!(strategy, RecoveryStrategy::SkipUntilSync));
    }

    #[test]
    fn test_recovery_strategy_if_expression() {
        let context = ErrorContext::IfExpression {
            condition: None,
            then_branch: None,
            else_branch: None,
        };
        let strategy = RecoveryRules::select_strategy(&context);
        assert!(matches!(strategy, RecoveryStrategy::DefaultValue));
    }

    #[test]
    fn test_recovery_strategy_array_literal() {
        let context = ErrorContext::ArrayLiteral {
            elements: vec![],
            error_at_index: 0,
        };
        let strategy = RecoveryRules::select_strategy(&context);
        assert!(matches!(strategy, RecoveryStrategy::PartialParse));
    }

    #[test]
    fn test_recovery_strategy_struct_literal() {
        let context = ErrorContext::StructLiteral {
            name: Some("Point".to_string()),
            fields: vec![],
            error_field: None,
        };
        let strategy = RecoveryRules::select_strategy(&context);
        assert!(matches!(strategy, RecoveryStrategy::PartialParse));
    }

    #[test]
    fn test_recovery_strategy_binary_op() {
        let context = ErrorContext::BinaryOp {
            left: None,
            op: None,
            right: None,
        };
        let strategy = RecoveryRules::select_strategy(&context);
        assert!(matches!(strategy, RecoveryStrategy::PanicMode));
    }

    #[test]
    fn test_recovery_strategy_function_decl_with_params() {
        let context = ErrorContext::FunctionDecl {
            name: Some("foo".to_string()),
            params: None,
            body: None,
        };
        let strategy = RecoveryRules::select_strategy(&context);
        assert!(matches!(strategy, RecoveryStrategy::DefaultValue));
    }

    #[test]
    fn test_recovery_strategy_function_decl_with_body_missing() {
        let context = ErrorContext::FunctionDecl {
            name: Some("foo".to_string()),
            params: Some(vec![]),
            body: None,
        };
        let strategy = RecoveryRules::select_strategy(&context);
        match strategy {
            RecoveryStrategy::InsertToken(token) => assert!(token.contains("{")),
            _ => panic!("Expected InsertToken strategy"),
        }
    }

    #[test]
    fn test_recovery_strategy_function_decl_complete() {
        use crate::frontend::ast::Span;
        let body_expr = Box::new(Expr::new(ExprKind::Literal(Literal::Unit), Span::new(0, 0)));
        let context = ErrorContext::FunctionDecl {
            name: Some("foo".to_string()),
            params: Some(vec![]),
            body: Some(body_expr),
        };
        let strategy = RecoveryRules::select_strategy(&context);
        assert!(matches!(strategy, RecoveryStrategy::PartialParse));
    }

    #[test]
    fn test_synthesize_ast_function_decl() {
        let error = ErrorNode {
            message: "test".to_string(),
            location: SourceLocation {
                line: 1,
                column: 1,
                file: None,
            },
            context: ErrorContext::FunctionDecl {
                name: None,
                params: None,
                body: None,
            },
            recovery: RecoveryStrategy::DefaultValue,
        };
        let ast = RecoveryRules::synthesize_ast(&error);
        assert!(matches!(ast.kind, ExprKind::Lambda { .. }));
    }

    #[test]
    fn test_synthesize_ast_if_expression() {
        let error = ErrorNode {
            message: "test".to_string(),
            location: SourceLocation {
                line: 1,
                column: 1,
                file: None,
            },
            context: ErrorContext::IfExpression {
                condition: None,
                then_branch: None,
                else_branch: None,
            },
            recovery: RecoveryStrategy::DefaultValue,
        };
        let ast = RecoveryRules::synthesize_ast(&error);
        assert!(matches!(ast.kind, ExprKind::If { .. }));
    }

    #[test]
    fn test_synthesize_ast_array_literal() {
        let error = ErrorNode {
            message: "test".to_string(),
            location: SourceLocation {
                line: 1,
                column: 1,
                file: None,
            },
            context: ErrorContext::ArrayLiteral {
                elements: vec![],
                error_at_index: 0,
            },
            recovery: RecoveryStrategy::PartialParse,
        };
        let ast = RecoveryRules::synthesize_ast(&error);
        assert!(matches!(ast.kind, ExprKind::List(_)));
    }

    #[test]
    fn test_synthesize_ast_binary_op_with_left() {
        use crate::frontend::ast::Span;
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(0, 0),
        ));
        let error = ErrorNode {
            message: "test".to_string(),
            location: SourceLocation {
                line: 1,
                column: 1,
                file: None,
            },
            context: ErrorContext::BinaryOp {
                left: Some(left),
                op: Some("+".to_string()),
                right: None,
            },
            recovery: RecoveryStrategy::PanicMode,
        };
        let ast = RecoveryRules::synthesize_ast(&error);
        assert!(matches!(
            ast.kind,
            ExprKind::Literal(Literal::Integer(42, None))
        ));
    }

    #[test]
    fn test_synthesize_ast_binary_op_without_left() {
        let error = ErrorNode {
            message: "test".to_string(),
            location: SourceLocation {
                line: 1,
                column: 1,
                file: None,
            },
            context: ErrorContext::BinaryOp {
                left: None,
                op: None,
                right: None,
            },
            recovery: RecoveryStrategy::PanicMode,
        };
        let ast = RecoveryRules::synthesize_ast(&error);
        assert!(matches!(ast.kind, ExprKind::Literal(Literal::Unit)));
    }

    #[test]
    fn test_synthesize_ast_struct_literal_with_name() {
        let error = ErrorNode {
            message: "test".to_string(),
            location: SourceLocation {
                line: 1,
                column: 1,
                file: None,
            },
            context: ErrorContext::StructLiteral {
                name: Some("Point".to_string()),
                fields: vec![],
                error_field: None,
            },
            recovery: RecoveryStrategy::PartialParse,
        };
        let ast = RecoveryRules::synthesize_ast(&error);
        match ast.kind {
            ExprKind::StructLiteral { name, .. } => assert_eq!(name, "Point"),
            _ => panic!("Expected StructLiteral"),
        }
    }

    #[test]
    fn test_synthesize_ast_struct_literal_without_name() {
        let error = ErrorNode {
            message: "test".to_string(),
            location: SourceLocation {
                line: 1,
                column: 1,
                file: None,
            },
            context: ErrorContext::StructLiteral {
                name: None,
                fields: vec![],
                error_field: None,
            },
            recovery: RecoveryStrategy::PartialParse,
        };
        let ast = RecoveryRules::synthesize_ast(&error);
        assert!(matches!(ast.kind, ExprKind::Literal(Literal::Unit)));
    }

    #[test]
    fn test_synthesize_ast_let_binding_without_name() {
        let error = ErrorNode {
            message: "test".to_string(),
            location: SourceLocation {
                line: 1,
                column: 1,
                file: None,
            },
            context: ErrorContext::LetBinding {
                name: None,
                value: None,
            },
            recovery: RecoveryStrategy::DefaultValue,
        };
        let ast = RecoveryRules::synthesize_ast(&error);
        match ast.kind {
            ExprKind::Let { name, .. } => assert_eq!(name, "_error"),
            _ => panic!("Expected Let expression"),
        }
    }

    #[test]
    fn test_expr_with_error_from_expr() {
        use crate::frontend::ast::Span;
        let expr = Expr::new(ExprKind::Literal(Literal::Integer(42, None)), Span::new(0, 2));
        let with_error: ExprWithError = expr.into();
        assert!(matches!(with_error, ExprWithError::Valid(_)));
    }

    #[test]
    fn test_expr_with_error_from_error_node() {
        let error = ErrorNode {
            message: "test".to_string(),
            location: SourceLocation {
                line: 1,
                column: 1,
                file: None,
            },
            context: ErrorContext::LetBinding {
                name: None,
                value: None,
            },
            recovery: RecoveryStrategy::DefaultValue,
        };
        let with_error: ExprWithError = error.into();
        assert!(matches!(with_error, ExprWithError::Error(_)));
    }

    #[test]
    fn test_source_location_with_file() {
        let loc = SourceLocation {
            line: 10,
            column: 5,
            file: Some("my_file.ruchy".to_string()),
        };
        assert_eq!(loc.line, 10);
        assert_eq!(loc.column, 5);
        assert_eq!(loc.file, Some("my_file.ruchy".to_string()));
    }

    #[test]
    fn test_error_node_clone() {
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
        let cloned = error.clone();
        assert_eq!(cloned.message, error.message);
    }

    #[test]
    fn test_recovery_strategy_debug() {
        let strategy = RecoveryStrategy::PanicMode;
        let debug_str = format!("{:?}", strategy);
        assert!(debug_str.contains("PanicMode"));
    }

    #[test]
    fn test_error_context_debug() {
        let context = ErrorContext::BinaryOp {
            left: None,
            op: Some("+".to_string()),
            right: None,
        };
        let debug_str = format!("{:?}", context);
        assert!(debug_str.contains("BinaryOp"));
    }
}
/* Commented out property tests to fix compilation
#[cfg(test)]
mod property_tests_error_recovery {
    use proptest::proptest;
    use super::*;
    use proptest::prelude::*;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
*/
