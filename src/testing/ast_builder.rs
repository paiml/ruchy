//! AST Builder for Testing
//! 
//! Provides convenient methods to construct AST nodes directly without parsing.
//! This allows testing transpiler functionality that the parser doesn't support yet.

use crate::frontend::ast::{
    Expr, ExprKind, Literal, Pattern, MatchArm, Type, TypeKind,
    Param, Span, BinaryOp, UnaryOp, StringPart, StructPatternField,
};

/// Builder for creating AST expressions programmatically
pub struct AstBuilder {
    span: Span,
}

impl AstBuilder {
    /// Create a new AST builder
    pub fn new() -> Self {
        Self {
            span: Span::default(),
        }
    }
    
    /// Create an integer literal
    pub fn int(&self, value: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(value)),
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a float literal
    pub fn float(&self, value: f64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Float(value)),
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a string literal
    pub fn string(&self, value: &str) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::String(value.to_string())),
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a boolean literal
    pub fn bool(&self, value: bool) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Bool(value)),
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create an identifier expression
    pub fn ident(&self, name: &str) -> Expr {
        Expr {
            kind: ExprKind::Identifier(name.to_string()),
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a binary operation
    pub fn binary(&self, left: Expr, op: BinaryOp, right: Expr) -> Expr {
        Expr {
            kind: ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a unary operation
    pub fn unary(&self, op: UnaryOp, operand: Expr) -> Expr {
        Expr {
            kind: ExprKind::Unary {
                op,
                operand: Box::new(operand),
            },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create an if expression
    pub fn if_expr(&self, condition: Expr, then_branch: Expr, else_branch: Option<Expr>) -> Expr {
        Expr {
            kind: ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: else_branch.map(Box::new),
            },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a match expression
    pub fn match_expr(&self, expr: Expr, arms: Vec<MatchArm>) -> Expr {
        Expr {
            kind: ExprKind::Match {
                expr: Box::new(expr),
                arms,
            },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a match arm
    pub fn match_arm(&self, pattern: Pattern, guard: Option<Expr>, body: Expr) -> MatchArm {
        MatchArm {
            pattern,
            guard: guard.map(Box::new),
            body: Box::new(body),
            span: self.span,
        }
    }
    
    /// Create a wildcard pattern
    pub fn pattern_wildcard(&self) -> Pattern {
        Pattern::Wildcard
    }
    
    /// Create an identifier pattern
    pub fn pattern_ident(&self, name: &str) -> Pattern {
        Pattern::Identifier(name.to_string())
    }
    
    /// Create a literal pattern
    pub fn pattern_literal(&self, lit: Literal) -> Pattern {
        Pattern::Literal(lit)
    }
    
    /// Create a tuple pattern
    pub fn pattern_tuple(&self, patterns: Vec<Pattern>) -> Pattern {
        Pattern::Tuple(patterns)
    }
    
    /// Create an or pattern (not supported by parser yet)
    pub fn pattern_or(&self, patterns: Vec<Pattern>) -> Pattern {
        Pattern::Or(patterns)
    }
    
    /// Create a struct pattern
    pub fn pattern_struct(&self, name: String, fields: Vec<(String, Pattern)>) -> Pattern {
        let struct_fields = fields.into_iter().map(|(name, pattern)| {
            StructPatternField { name, pattern: Some(pattern) }
        }).collect();
        Pattern::Struct { name, fields: struct_fields, has_rest: false }
    }
    
    /// Create a rest pattern (..)
    pub fn pattern_rest(&self) -> Pattern {
        Pattern::Rest
    }
    
    /// Create a function call
    pub fn call(&self, func: Expr, args: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Call {
                func: Box::new(func),
                args,
            },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a lambda expression
    pub fn lambda(&self, params: Vec<Param>, body: Expr) -> Expr {
        Expr {
            kind: ExprKind::Lambda {
                params,
                body: Box::new(body),
            },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a block expression
    pub fn block(&self, statements: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Block(statements),
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a let expression
    pub fn let_expr(&self, name: String, value: Expr) -> Expr {
        Expr {
            kind: ExprKind::Let {
                name,
                value: Box::new(value),
                type_annotation: None,
                is_mutable: false,
                body: Box::new(Expr {
                    kind: ExprKind::Literal(Literal::Unit),
                    span: self.span,
                    attributes: vec![],
                }),
            },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create an assignment
    pub fn assign(&self, target: Expr, value: Expr) -> Expr {
        Expr {
            kind: ExprKind::Assign {
                target: Box::new(target),
                value: Box::new(value),
            },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a `Result::Ok` variant
    pub fn ok(&self, value: Expr) -> Expr {
        Expr {
            kind: ExprKind::Call {
                func: Box::new(self.ident("Ok")),
                args: vec![value],
            },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a `Result::Err` variant
    pub fn err(&self, value: Expr) -> Expr {
        Expr {
            kind: ExprKind::Call {
                func: Box::new(self.ident("Err")),
                args: vec![value],
            },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create an `Option::Some` variant
    pub fn some(&self, value: Expr) -> Expr {
        Expr {
            kind: ExprKind::Call {
                func: Box::new(self.ident("Some")),
                args: vec![value],
            },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create an `Option::None` variant
    pub fn none(&self) -> Expr {
        self.ident("None")
    }
    
    /// Create a list/array literal
    pub fn list(&self, elements: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::List(elements),
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a tuple literal
    pub fn tuple(&self, elements: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Tuple(elements),
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create string interpolation
    pub fn string_interpolation(&self, parts: Vec<StringPart>) -> Expr {
        Expr {
            kind: ExprKind::StringInterpolation { parts },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a for loop
    pub fn for_loop(&self, var: String, iter: Expr, body: Expr) -> Expr {
        Expr {
            kind: ExprKind::For {
                var,
                iter: Box::new(iter),
                body: Box::new(body),
                pattern: None,
            },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a while loop
    pub fn while_loop(&self, condition: Expr, body: Expr) -> Expr {
        Expr {
            kind: ExprKind::While {
                condition: Box::new(condition),
                body: Box::new(body),
            },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a loop expression
    pub fn loop_expr(&self, body: Expr) -> Expr {
        Expr {
            kind: ExprKind::Loop {
                body: Box::new(body),
            },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a break expression
    pub fn break_expr(&self, label: Option<String>) -> Expr {
        Expr {
            kind: ExprKind::Break { label },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a continue expression
    pub fn continue_expr(&self, label: Option<String>) -> Expr {
        Expr {
            kind: ExprKind::Continue { label },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a return expression
    pub fn return_expr(&self, value: Option<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Return { value: value.map(Box::new) },
            span: self.span,
            attributes: vec![],
        }
    }
    
    /// Create a type annotation
    pub fn type_int(&self) -> Type {
        Type {
            kind: TypeKind::Named("i32".to_string()),
            span: self.span,
        }
    }
    
    /// Create a Result type
    pub fn type_result(&self, ok: Type, err: Type) -> Type {
        Type {
            kind: TypeKind::Generic {
                base: "Result".to_string(),
                params: vec![ok, err],
            },
            span: self.span,
        }
    }
    
    /// Create an Option type
    pub fn type_option(&self, inner: Type) -> Type {
        Type {
            kind: TypeKind::Generic {
                base: "Option".to_string(),
                params: vec![inner],
            },
            span: self.span,
        }
    }
}

impl Default for AstBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Transpiler;
    
    #[test]
    fn test_ast_builder_basic() {
        let builder = AstBuilder::new();
        
        // Create: if x > 0 { "positive" } else { "negative" }
        let ast = builder.if_expr(
            builder.binary(
                builder.ident("x"),
                BinaryOp::Greater,
                builder.int(0),
            ),
            builder.string("positive"),
            Some(builder.string("negative")),
        );
        
        // Should be able to transpile
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_ast_builder_match_with_guard() {
        let builder = AstBuilder::new();
        
        // Create match with pattern guard (parser doesn't support this)
        let ast = builder.match_expr(
            builder.ident("x"),
            vec![
                builder.match_arm(
                    builder.pattern_ident("n"),
                    Some(builder.binary(
                        builder.ident("n"),
                        BinaryOp::Greater,
                        builder.int(0),
                    )),
                    builder.string("positive"),
                ),
                builder.match_arm(
                    builder.pattern_wildcard(),
                    None,
                    builder.string("other"),
                ),
            ],
        );
        
        // Should be able to transpile even though parser can't parse this
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_ast_builder_or_pattern() {
        let builder = AstBuilder::new();
        
        // Create or-pattern (parser doesn't support this)
        let ast = builder.match_expr(
            builder.ident("x"),
            vec![
                builder.match_arm(
                    builder.pattern_or(vec![
                        builder.pattern_literal(Literal::Integer(1)),
                        builder.pattern_literal(Literal::Integer(2)),
                        builder.pattern_literal(Literal::Integer(3)),
                    ]),
                    None,
                    builder.string("small"),
                ),
                builder.match_arm(
                    builder.pattern_wildcard(),
                    None,
                    builder.string("other"),
                ),
            ],
        );
        
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }
}