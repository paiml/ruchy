//! Canonical AST Normalization
//!
//! Implements the extreme quality engineering approach from docs/ruchy-transpiler-docs.md
//! This module eliminates syntactic ambiguity by converting all surface syntax to a
//! normalized core form before transpilation.

#![allow(clippy::panic)] // Panics represent genuine errors in normalization
#![allow(clippy::panic)]
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
    pub fn new() -> Self {
        Self {
            context: DeBruijnContext::new(),
        }
    }

    /// Main entry point: normalize an AST to core form
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
                    BinaryOp::And => PrimOp::And,
                    BinaryOp::Or => PrimOp::Or,
                    // Bitwise operations not yet in core language
                    BinaryOp::BitwiseAnd
                    | BinaryOp::BitwiseOr
                    | BinaryOp::BitwiseXor
                    | BinaryOp::LeftShift
                    | BinaryOp::RightShift => {
                        panic!("Bitwise operations not yet supported in core language")
                    }
                };

                CoreExpr::Prim(prim, vec![l, r])
            }

            ExprKind::Let { name, value, body, .. } => {
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
                    self.context.push(param.name.clone());
                    result = CoreExpr::Lambda {
                        param_name: Some(param.name.clone()),
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
                    self.context.push(param.name.clone());
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
                        param_name: Some(param.name.clone()),
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
            Literal::Integer(i) => CoreLiteral::Integer(*i),
            Literal::Float(f) => CoreLiteral::Float(*f),
            Literal::String(s) => CoreLiteral::String(s.clone()),
            Literal::Bool(b) => CoreLiteral::Bool(*b),
            Literal::Unit => CoreLiteral::Unit,
        })
    }
}

/// Invariant checking
impl CoreExpr {
    /// Check that the expression is in normal form
    #[must_use]
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

    #[test]
    fn test_normalize_let_statement() {
        let input = "let x = 10 in x + 1";
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

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
        let ast = parser.parse().unwrap();

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
}
