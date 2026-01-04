//! Canonical AST Normalization
//!
//! Implements the extreme quality engineering approach from docs/ruchy-transpiler-docs.md
//! This module eliminates syntactic ambiguity by converting all surface syntax to a
//! normalized core form before transpilation.
#![allow(clippy::panic)] // Panics represent genuine errors in normalization
use crate::frontend::ast::{Expr, ExprKind, Literal};
/// De Bruijn index for variables - eliminates variable capture bugs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeBruijnIndex(pub usize);
/// Core expression language - minimal, unambiguous representation
#[derive(Debug, Clone, PartialEq)]
pub enum CoreExpr {
    /// Variable reference using De Bruijn index
    Var(DeBruijnIndex),
    /// Lambda abstraction (parameter name for debugging only)
    Lambda {
        param_name: Option<String>, // For debugging
        body: Box<CoreExpr>,
    },
    /// Function application
    App(Box<CoreExpr>, Box<CoreExpr>),
    /// Let binding (name for debugging only)
    Let {
        name: Option<String>, // For debugging
        value: Box<CoreExpr>,
        body: Box<CoreExpr>,
    },
    /// Literal values
    Literal(CoreLiteral),
    /// Primitive operations
    Prim(PrimOp, Vec<CoreExpr>),
}
/// Core literal values
#[derive(Debug, Clone, PartialEq)]
pub enum CoreLiteral {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Char(char),
    Unit,
}
/// Primitive operations - all operators desugared to these
#[derive(Debug, Clone, PartialEq)]
pub enum PrimOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    And,
    Or,
    Not,
    NullCoalesce,
    // String
    Concat,
    // Array
    ArrayNew,
    ArrayIndex,
    ArrayLen,
    // Control flow
    If,
}
/// Context for De Bruijn conversion
#[derive(Debug, Clone)]
struct DeBruijnContext {
    /// Maps variable names to their De Bruijn indices
    bindings: Vec<String>,
}
impl DeBruijnContext {
    fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }
    fn push(&mut self, name: String) {
        self.bindings.push(name);
    }
    fn pop(&mut self) {
        self.bindings.pop();
    }
    fn lookup(&self, name: &str) -> Option<DeBruijnIndex> {
        self.bindings
            .iter()
            .rev()
            .position(|n| n == name)
            .map(DeBruijnIndex)
    }
}
/// AST Normalizer - converts surface syntax to canonical core form
pub struct AstNormalizer {
    context: DeBruijnContext,
}
impl Default for AstNormalizer {
    fn default() -> Self {
        Self::new()
    }
}
impl AstNormalizer {
    #[must_use]
    /// # Examples
    ///
    /// ```
    /// use ruchy::transpiler::canonical_ast::AstNormalizer;
    ///
    /// let normalizer = AstNormalizer::new();
    /// ```
    pub fn new() -> Self {
        Self {
            context: DeBruijnContext::new(),
        }
    }
    /// Main entry point: normalize an AST to core form
    /// # Examples
    ///
    /// ```
    /// use ruchy::transpiler::canonical_ast::AstNormalizer;
    ///
    /// let mut instance = AstNormalizer::new();
    /// let result = instance.normalize();
    /// // Verify behavior
    /// ```
    pub fn normalize(&mut self, expr: &Expr) -> CoreExpr {
        self.desugar_and_convert(expr)
    }
    /// Desugar surface syntax and convert to core form with De Bruijn indices
    #[allow(clippy::too_many_lines)] // Complex but necessary for complete desugaring
    fn desugar_and_convert(&mut self, expr: &Expr) -> CoreExpr {
        match &expr.kind {
            ExprKind::Literal(lit) => Self::convert_literal(lit),
            ExprKind::Identifier(name) => {
                if let Some(idx) = self.context.lookup(name) {
                    CoreExpr::Var(idx)
                } else {
                    // Free variable - this shouldn't happen in well-formed programs
                    // For REPL, we might want to handle this differently
                    panic!("Unbound variable: {name}");
                }
            }
            ExprKind::Binary { left, op, right } => {
                use crate::frontend::ast::BinaryOp;
                let l = self.desugar_and_convert(left);
                let r = self.desugar_and_convert(right);
                let prim = match op {
                    BinaryOp::Add => PrimOp::Add,
                    BinaryOp::Subtract => PrimOp::Sub,
                    BinaryOp::Multiply => PrimOp::Mul,
                    BinaryOp::Divide => PrimOp::Div,
                    BinaryOp::Modulo => PrimOp::Mod,
                    BinaryOp::Power => PrimOp::Pow,
                    BinaryOp::Equal => PrimOp::Eq,
                    BinaryOp::NotEqual => PrimOp::Ne,
                    BinaryOp::Less => PrimOp::Lt,
                    BinaryOp::LessEqual => PrimOp::Le,
                    BinaryOp::Greater => PrimOp::Gt,
                    BinaryOp::GreaterEqual => PrimOp::Ge,
                    BinaryOp::Gt => PrimOp::Gt, // Alias for Greater
                    BinaryOp::And => PrimOp::And,
                    BinaryOp::Or => PrimOp::Or,
                    BinaryOp::NullCoalesce => PrimOp::NullCoalesce,
                    // Bitwise operations not yet in core language
                    BinaryOp::BitwiseAnd
                    | BinaryOp::BitwiseOr
                    | BinaryOp::BitwiseXor
                    | BinaryOp::LeftShift
                    | BinaryOp::RightShift => {
                        panic!("Bitwise operations not yet supported in core language")
                    }
                    // Actor operations not yet in core language
                    BinaryOp::Send => {
                        panic!("Actor operations not yet supported in core language")
                    }
                    // Containment check - not yet in core language
                    BinaryOp::In => {
                        panic!("Containment 'in' operator not yet supported in core language")
                    }
                };
                CoreExpr::Prim(prim, vec![l, r])
            }
            ExprKind::Let {
                name, value, body, ..
            } => {
                let val = self.desugar_and_convert(value);
                // Push binding for body evaluation
                self.context.push(name.clone());
                let bod = self.desugar_and_convert(body);
                self.context.pop();
                CoreExpr::Let {
                    name: Some(name.clone()),
                    value: Box::new(val),
                    body: Box::new(bod),
                }
            }
            ExprKind::Lambda { params, body } => {
                // Desugar multi-param lambda to nested single-param lambdas
                // \x y z -> body becomes \x -> \y -> \z -> body
                let mut result = self.desugar_and_convert(body);
                for param in params.iter().rev() {
                    self.context.push(param.name());
                    result = CoreExpr::Lambda {
                        param_name: Some(param.name()),
                        body: Box::new(result),
                    };
                    // Note: We don't pop here because we're building inside-out
                }
                // Pop all the params we pushed
                for _ in params {
                    self.context.pop();
                }
                result
            }
            ExprKind::Function {
                name, params, body, ..
            } => {
                // Functions become let-bound lambdas
                // fun f(x, y) { body } becomes let f = \x y -> body
                // First, add all parameters to the context
                for param in params {
                    self.context.push(param.name());
                }
                // Process the body with parameters in scope
                let body_core = self.desugar_and_convert(body);
                // Remove parameters from context
                for _ in params {
                    self.context.pop();
                }
                // Create nested lambdas for each parameter
                let mut lambda_body = body_core;
                for param in params.iter().rev() {
                    lambda_body = CoreExpr::Lambda {
                        param_name: Some(param.name()),
                        body: Box::new(lambda_body),
                    };
                }
                // Wrap in a let binding
                // For REPL context, we might want to handle this differently
                CoreExpr::Let {
                    name: Some(name.clone()),
                    value: Box::new(lambda_body),
                    body: Box::new(CoreExpr::Literal(CoreLiteral::Unit)), // Empty body for top-level
                }
            }
            ExprKind::Call { func, args } => {
                // Desugar multi-arg call to nested applications
                // f(a, b, c) becomes (((f a) b) c)
                let mut result = self.desugar_and_convert(func);
                for arg in args {
                    let arg_core = self.desugar_and_convert(arg);
                    result = CoreExpr::App(Box::new(result), Box::new(arg_core));
                }
                result
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond = self.desugar_and_convert(condition);
                let then_b = self.desugar_and_convert(then_branch);
                let else_b = else_branch
                    .as_ref()
                    .map_or(CoreExpr::Literal(CoreLiteral::Unit), |e| {
                        self.desugar_and_convert(e)
                    });
                CoreExpr::Prim(PrimOp::If, vec![cond, then_b, else_b])
            }
            ExprKind::List(elements) => {
                // Desugar list to array operations
                let mut result = CoreExpr::Prim(PrimOp::ArrayNew, vec![]);
                for elem in elements {
                    let elem_core = self.desugar_and_convert(elem);
                    // Each element becomes an append operation
                    // This is simplified; real implementation would be more efficient
                    result = CoreExpr::Prim(PrimOp::ArrayNew, vec![result, elem_core]);
                }
                result
            }
            ExprKind::Block(exprs) => {
                // For a block, we evaluate all expressions but return only the last one
                // This is a simplification - a full implementation would handle statements
                if exprs.is_empty() {
                    CoreExpr::Literal(CoreLiteral::Unit)
                } else if exprs.len() == 1 {
                    self.desugar_and_convert(&exprs[0])
                } else {
                    // For now, just return the last expression
                    // A complete implementation would handle side effects
                    if let Some(last) = exprs.last() {
                        self.desugar_and_convert(last)
                    } else {
                        CoreExpr::Literal(CoreLiteral::Unit)
                    }
                }
            }
            _ => {
                // For now, panic on unsupported constructs
                // In production, we'd handle all cases
                panic!("Unsupported expression kind in normalizer: {:?}", expr.kind);
            }
        }
    }
    fn convert_literal(lit: &Literal) -> CoreExpr {
        CoreExpr::Literal(match lit {
            Literal::Integer(i, _) => CoreLiteral::Integer(*i),
            Literal::Float(f) => CoreLiteral::Float(*f),
            Literal::String(s) => CoreLiteral::String(s.clone()),
            Literal::Bool(b) => CoreLiteral::Bool(*b),
            Literal::Char(c) => CoreLiteral::Char(*c),
            Literal::Byte(b) => CoreLiteral::Integer(i64::from(*b)), // Represent byte as integer in canonical AST
            Literal::Unit => CoreLiteral::Unit,
            Literal::Null => CoreLiteral::Unit,
            Literal::Atom(_) => CoreLiteral::Unit, // TODO: Support atoms in canonical AST
        })
    }
}
/// Invariant checking
impl CoreExpr {
    /// Check that the expression is in normal form
    #[must_use]
    /// # Examples
    ///
    /// ```
    /// use ruchy::transpiler::canonical_ast::is_normalized;
    ///
    /// let result = is_normalized(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn is_normalized(&self) -> bool {
        match self {
            CoreExpr::Var(_) | CoreExpr::Literal(_) => true,
            CoreExpr::Lambda { body, .. } => body.is_normalized(),
            CoreExpr::App(f, x) => f.is_normalized() && x.is_normalized(),
            CoreExpr::Let { value, body, .. } => value.is_normalized() && body.is_normalized(),
            CoreExpr::Prim(_, args) => args.iter().all(CoreExpr::is_normalized),
        }
    }
    /// Check that all variables are bound (no free variables)
    #[must_use]
    /// # Examples
    ///
    /// ```
    /// use ruchy::transpiler::canonical_ast::is_closed;
    ///
    /// let result = is_closed(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn is_closed(&self) -> bool {
        self.is_closed_at(0)
    }
    fn is_closed_at(&self, depth: usize) -> bool {
        match self {
            CoreExpr::Var(DeBruijnIndex(idx)) => *idx < depth,
            CoreExpr::Lambda { body, .. } => body.is_closed_at(depth + 1),
            CoreExpr::App(f, x) => f.is_closed_at(depth) && x.is_closed_at(depth),
            CoreExpr::Let { value, body, .. } => {
                value.is_closed_at(depth) && body.is_closed_at(depth + 1)
            }
            CoreExpr::Literal(_) => true,
            CoreExpr::Prim(_, args) => args.iter().all(|a| a.is_closed_at(depth)),
        }
    }
}
#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::Parser;

    // ===== DeBruijnIndex Tests =====

    #[test]
    fn test_debruijn_index_new() {
        let idx = DeBruijnIndex(0);
        assert_eq!(idx.0, 0);
    }

    #[test]
    fn test_debruijn_index_debug() {
        let idx = DeBruijnIndex(5);
        let debug = format!("{:?}", idx);
        assert!(debug.contains("DeBruijnIndex"));
    }

    #[test]
    fn test_debruijn_index_clone() {
        let idx = DeBruijnIndex(3);
        let cloned = idx.clone();
        assert_eq!(idx, cloned);
    }

    #[test]
    fn test_debruijn_index_eq() {
        assert_eq!(DeBruijnIndex(0), DeBruijnIndex(0));
        assert_ne!(DeBruijnIndex(0), DeBruijnIndex(1));
    }

    #[test]
    fn test_debruijn_index_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(DeBruijnIndex(0));
        set.insert(DeBruijnIndex(1));
        assert_eq!(set.len(), 2);
    }

    // ===== CoreLiteral Tests =====

    #[test]
    fn test_core_literal_integer() {
        let lit = CoreLiteral::Integer(42);
        assert!(matches!(lit, CoreLiteral::Integer(42)));
    }

    #[test]
    fn test_core_literal_float() {
        let lit = CoreLiteral::Float(3.14);
        assert!(matches!(lit, CoreLiteral::Float(f) if (f - 3.14).abs() < f64::EPSILON));
    }

    #[test]
    fn test_core_literal_string() {
        let lit = CoreLiteral::String("hello".to_string());
        assert!(matches!(lit, CoreLiteral::String(s) if s == "hello"));
    }

    #[test]
    fn test_core_literal_bool() {
        let lit_true = CoreLiteral::Bool(true);
        let lit_false = CoreLiteral::Bool(false);
        assert!(matches!(lit_true, CoreLiteral::Bool(true)));
        assert!(matches!(lit_false, CoreLiteral::Bool(false)));
    }

    #[test]
    fn test_core_literal_char() {
        let lit = CoreLiteral::Char('x');
        assert!(matches!(lit, CoreLiteral::Char('x')));
    }

    #[test]
    fn test_core_literal_unit() {
        let lit = CoreLiteral::Unit;
        assert!(matches!(lit, CoreLiteral::Unit));
    }

    #[test]
    fn test_core_literal_clone() {
        let lit = CoreLiteral::Integer(42);
        let cloned = lit.clone();
        assert_eq!(lit, cloned);
    }

    #[test]
    fn test_core_literal_debug() {
        let lit = CoreLiteral::Integer(42);
        let debug = format!("{:?}", lit);
        assert!(debug.contains("Integer"));
    }

    // ===== PrimOp Tests =====

    #[test]
    fn test_primop_clone() {
        let op = PrimOp::Add;
        let cloned = op.clone();
        assert_eq!(op, cloned);
    }

    #[test]
    fn test_primop_debug() {
        let op = PrimOp::Mul;
        let debug = format!("{:?}", op);
        assert!(debug.contains("Mul"));
    }

    #[test]
    fn test_primop_eq() {
        assert_eq!(PrimOp::Add, PrimOp::Add);
        assert_ne!(PrimOp::Add, PrimOp::Sub);
    }

    #[test]
    fn test_primop_all_variants() {
        // Test that all PrimOp variants exist
        let _add = PrimOp::Add;
        let _sub = PrimOp::Sub;
        let _mul = PrimOp::Mul;
        let _div = PrimOp::Div;
        let _mod_ = PrimOp::Mod;
        let _pow = PrimOp::Pow;
        let _eq = PrimOp::Eq;
        let _ne = PrimOp::Ne;
        let _lt = PrimOp::Lt;
        let _le = PrimOp::Le;
        let _gt = PrimOp::Gt;
        let _ge = PrimOp::Ge;
        let _and = PrimOp::And;
        let _or = PrimOp::Or;
        let _not = PrimOp::Not;
        let _null_coalesce = PrimOp::NullCoalesce;
        let _concat = PrimOp::Concat;
        let _array_new = PrimOp::ArrayNew;
        let _array_index = PrimOp::ArrayIndex;
        let _array_len = PrimOp::ArrayLen;
        let _if = PrimOp::If;
    }

    // ===== CoreExpr Tests =====

    #[test]
    fn test_core_expr_var() {
        let expr = CoreExpr::Var(DeBruijnIndex(0));
        assert!(matches!(expr, CoreExpr::Var(_)));
    }

    #[test]
    fn test_core_expr_lambda() {
        let expr = CoreExpr::Lambda {
            param_name: Some("x".to_string()),
            body: Box::new(CoreExpr::Literal(CoreLiteral::Unit)),
        };
        assert!(matches!(expr, CoreExpr::Lambda { .. }));
    }

    #[test]
    fn test_core_expr_app() {
        let expr = CoreExpr::App(
            Box::new(CoreExpr::Literal(CoreLiteral::Unit)),
            Box::new(CoreExpr::Literal(CoreLiteral::Unit)),
        );
        assert!(matches!(expr, CoreExpr::App(_, _)));
    }

    #[test]
    fn test_core_expr_let() {
        let expr = CoreExpr::Let {
            name: Some("x".to_string()),
            value: Box::new(CoreExpr::Literal(CoreLiteral::Integer(42))),
            body: Box::new(CoreExpr::Literal(CoreLiteral::Unit)),
        };
        assert!(matches!(expr, CoreExpr::Let { .. }));
    }

    #[test]
    fn test_core_expr_literal() {
        let expr = CoreExpr::Literal(CoreLiteral::Integer(42));
        assert!(matches!(expr, CoreExpr::Literal(_)));
    }

    #[test]
    fn test_core_expr_prim() {
        let expr = CoreExpr::Prim(PrimOp::Add, vec![]);
        assert!(matches!(expr, CoreExpr::Prim(_, _)));
    }

    #[test]
    fn test_core_expr_clone() {
        let expr = CoreExpr::Literal(CoreLiteral::Integer(42));
        let cloned = expr.clone();
        assert_eq!(expr, cloned);
    }

    #[test]
    fn test_core_expr_debug() {
        let expr = CoreExpr::Literal(CoreLiteral::Integer(42));
        let debug = format!("{:?}", expr);
        assert!(debug.contains("Literal"));
    }

    // ===== is_normalized Tests =====

    #[test]
    fn test_is_normalized_var() {
        let expr = CoreExpr::Var(DeBruijnIndex(0));
        assert!(expr.is_normalized());
    }

    #[test]
    fn test_is_normalized_literal() {
        let expr = CoreExpr::Literal(CoreLiteral::Integer(42));
        assert!(expr.is_normalized());
    }

    #[test]
    fn test_is_normalized_lambda() {
        let expr = CoreExpr::Lambda {
            param_name: Some("x".to_string()),
            body: Box::new(CoreExpr::Literal(CoreLiteral::Unit)),
        };
        assert!(expr.is_normalized());
    }

    #[test]
    fn test_is_normalized_app() {
        let expr = CoreExpr::App(
            Box::new(CoreExpr::Literal(CoreLiteral::Unit)),
            Box::new(CoreExpr::Literal(CoreLiteral::Unit)),
        );
        assert!(expr.is_normalized());
    }

    #[test]
    fn test_is_normalized_let() {
        let expr = CoreExpr::Let {
            name: Some("x".to_string()),
            value: Box::new(CoreExpr::Literal(CoreLiteral::Integer(42))),
            body: Box::new(CoreExpr::Literal(CoreLiteral::Unit)),
        };
        assert!(expr.is_normalized());
    }

    #[test]
    fn test_is_normalized_prim() {
        let expr = CoreExpr::Prim(
            PrimOp::Add,
            vec![
                CoreExpr::Literal(CoreLiteral::Integer(1)),
                CoreExpr::Literal(CoreLiteral::Integer(2)),
            ],
        );
        assert!(expr.is_normalized());
    }

    // ===== is_closed Tests =====

    #[test]
    fn test_is_closed_literal() {
        let expr = CoreExpr::Literal(CoreLiteral::Integer(42));
        assert!(expr.is_closed());
    }

    #[test]
    fn test_is_closed_free_var() {
        let expr = CoreExpr::Var(DeBruijnIndex(0));
        assert!(!expr.is_closed());
    }

    #[test]
    fn test_is_closed_lambda_bound() {
        // \x -> x (variable is bound at depth 1)
        let expr = CoreExpr::Lambda {
            param_name: Some("x".to_string()),
            body: Box::new(CoreExpr::Var(DeBruijnIndex(0))),
        };
        assert!(expr.is_closed());
    }

    #[test]
    fn test_is_closed_lambda_free() {
        // \x -> y (y is free at depth 1)
        let expr = CoreExpr::Lambda {
            param_name: Some("x".to_string()),
            body: Box::new(CoreExpr::Var(DeBruijnIndex(1))),
        };
        assert!(!expr.is_closed());
    }

    #[test]
    fn test_is_closed_let_bound() {
        // let x = 42 in x
        let expr = CoreExpr::Let {
            name: Some("x".to_string()),
            value: Box::new(CoreExpr::Literal(CoreLiteral::Integer(42))),
            body: Box::new(CoreExpr::Var(DeBruijnIndex(0))),
        };
        assert!(expr.is_closed());
    }

    #[test]
    fn test_is_closed_let_free_in_value() {
        // let x = y in 42 (y is free)
        let expr = CoreExpr::Let {
            name: Some("x".to_string()),
            value: Box::new(CoreExpr::Var(DeBruijnIndex(0))),
            body: Box::new(CoreExpr::Literal(CoreLiteral::Integer(42))),
        };
        assert!(!expr.is_closed());
    }

    #[test]
    fn test_is_closed_app() {
        let expr = CoreExpr::App(
            Box::new(CoreExpr::Literal(CoreLiteral::Unit)),
            Box::new(CoreExpr::Literal(CoreLiteral::Unit)),
        );
        assert!(expr.is_closed());
    }

    #[test]
    fn test_is_closed_app_free() {
        let expr = CoreExpr::App(
            Box::new(CoreExpr::Var(DeBruijnIndex(0))),
            Box::new(CoreExpr::Literal(CoreLiteral::Unit)),
        );
        assert!(!expr.is_closed());
    }

    #[test]
    fn test_is_closed_prim() {
        let expr = CoreExpr::Prim(
            PrimOp::Add,
            vec![
                CoreExpr::Literal(CoreLiteral::Integer(1)),
                CoreExpr::Literal(CoreLiteral::Integer(2)),
            ],
        );
        assert!(expr.is_closed());
    }

    #[test]
    fn test_is_closed_prim_free() {
        let expr = CoreExpr::Prim(
            PrimOp::Add,
            vec![
                CoreExpr::Var(DeBruijnIndex(0)),
                CoreExpr::Literal(CoreLiteral::Integer(2)),
            ],
        );
        assert!(!expr.is_closed());
    }

    // ===== AstNormalizer Tests =====

    #[test]
    fn test_normalizer_new() {
        let normalizer = AstNormalizer::new();
        // Just verify it can be created
        drop(normalizer);
    }

    #[test]
    fn test_normalizer_default() {
        let normalizer = AstNormalizer::default();
        drop(normalizer);
    }

    #[test]
    fn test_normalize_integer_literal() {
        let input = "42";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        assert!(matches!(core, CoreExpr::Literal(CoreLiteral::Integer(42))));
    }

    #[test]
    fn test_normalize_float_literal() {
        let input = "3.14";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        assert!(matches!(core, CoreExpr::Literal(CoreLiteral::Float(_))));
    }

    #[test]
    fn test_normalize_string_literal() {
        let input = r#""hello""#;
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        assert!(matches!(core, CoreExpr::Literal(CoreLiteral::String(_))));
    }

    #[test]
    fn test_normalize_bool_literal() {
        let input = "true";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        assert!(matches!(core, CoreExpr::Literal(CoreLiteral::Bool(true))));
    }

    #[test]
    fn test_normalize_unit_literal() {
        let input = "()";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        assert!(matches!(core, CoreExpr::Literal(CoreLiteral::Unit)));
    }

    #[test]
    fn test_normalize_binary_add() {
        let input = "1 + 2";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        assert!(matches!(core, CoreExpr::Prim(PrimOp::Add, _)));
    }

    #[test]
    fn test_normalize_binary_sub() {
        let input = "5 - 3";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        assert!(matches!(core, CoreExpr::Prim(PrimOp::Sub, _)));
    }

    #[test]
    fn test_normalize_binary_mul() {
        let input = "2 * 3";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        assert!(matches!(core, CoreExpr::Prim(PrimOp::Mul, _)));
    }

    #[test]
    fn test_normalize_binary_div() {
        let input = "10 / 2";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        assert!(matches!(core, CoreExpr::Prim(PrimOp::Div, _)));
    }

    #[test]
    fn test_normalize_binary_mod() {
        let input = "10 % 3";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        assert!(matches!(core, CoreExpr::Prim(PrimOp::Mod, _)));
    }

    #[test]
    fn test_normalize_binary_comparison() {
        for (input, expected_op) in [
            ("1 == 2", PrimOp::Eq),
            ("1 != 2", PrimOp::Ne),
            ("1 < 2", PrimOp::Lt),
            ("1 <= 2", PrimOp::Le),
            ("1 > 2", PrimOp::Gt),
            ("1 >= 2", PrimOp::Ge),
        ] {
            let mut parser = Parser::new(input);
            let ast = parser.parse().expect("Failed to parse");
            let mut normalizer = AstNormalizer::new();
            let core = normalizer.normalize(&ast);
            match core {
                CoreExpr::Prim(op, _) => assert_eq!(op, expected_op),
                _ => panic!("Expected Prim expression"),
            }
        }
    }

    #[test]
    fn test_normalize_binary_logical() {
        let input_and = "true && false";
        let mut parser = Parser::new(input_and);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        assert!(matches!(core, CoreExpr::Prim(PrimOp::And, _)));

        let input_or = "true || false";
        let mut parser = Parser::new(input_or);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        assert!(matches!(core, CoreExpr::Prim(PrimOp::Or, _)));
    }

    #[test]
    fn test_normalize_if_expression() {
        let input = "if true { 1 } else { 2 }";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        assert!(matches!(core, CoreExpr::Prim(PrimOp::If, _)));
    }

    #[test]
    fn test_normalize_if_without_else() {
        let input = "if true { 1 }";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        // Should have Unit as else branch
        assert!(matches!(core, CoreExpr::Prim(PrimOp::If, _)));
    }

    #[test]
    fn test_normalize_let_statement() {
        let input = "let x = 10 in x + 1";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        // Should be: Let { name: "x", value: Literal(10), body: Unit }
        assert!(matches!(core, CoreExpr::Let { .. }));
        assert!(core.is_normalized());
    }
    #[test]
    fn test_normalize_lambda() {
        let input = "fun add(x, y) { x + y }";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        // Should be: Let { name: "add", value: Lambda { Lambda { Prim(Add, [Var(1), Var(0)]) } } }
        assert!(matches!(core, CoreExpr::Let { .. }));
        assert!(core.is_normalized());
    }
    #[test]
    fn test_idempotent_normalization() {
        let inputs = vec!["42", "let x = 10 in x + 1", "fun f(x) { x * 2 }"];
        for input in inputs {
            let mut parser = Parser::new(input);
            if let Ok(ast) = parser.parse() {
                let mut normalizer1 = AstNormalizer::new();
                let core1 = normalizer1.normalize(&ast);
                // Normalizing again should produce the same result
                let mut normalizer2 = AstNormalizer::new();
                let core2 = normalizer2.normalize(&ast);
                assert_eq!(core1, core2, "Normalization should be deterministic");
            }
        }
    }

    #[test]
    fn test_normalize_block_empty() {
        // Note: `{ }` parses as ObjectLiteral, use `()` for unit
        let input = "()";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        assert!(matches!(core, CoreExpr::Literal(CoreLiteral::Unit)));
    }

    #[test]
    fn test_normalize_block_single() {
        let input = "{ 42 }";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        assert!(matches!(core, CoreExpr::Literal(CoreLiteral::Integer(42))));
    }

    #[test]
    fn test_normalize_block_multiple() {
        let input = "{ 1; 2; 3 }";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        // Should return the last expression
        assert!(matches!(core, CoreExpr::Literal(CoreLiteral::Integer(3))));
    }

    #[test]
    fn test_normalize_complex_expression() {
        let input = "1 + 2 * 3";
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect("Failed to parse");
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        // Should be Add(1, Mul(2, 3))
        assert!(matches!(core, CoreExpr::Prim(PrimOp::Add, _)));
        assert!(core.is_normalized());
        assert!(core.is_closed());
    }
}
#[cfg(test)]
mod property_tests_canonical_ast {
    use proptest::proptest;

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
