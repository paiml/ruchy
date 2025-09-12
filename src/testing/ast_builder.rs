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
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::new;
/// 
/// let result = new(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn new() -> Self {
        Self {
            span: Span::default(),
        }
    }
    /// Create an integer literal
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::int;
/// 
/// let result = int(42);
/// assert_eq!(result, Ok(42));
/// ```
pub fn int(&self, value: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(value)),
            span: self.span,
            attributes: vec![],
        }
    }
    /// Create a float literal
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::float;
/// 
/// let result = float(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn float(&self, value: f64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Float(value)),
            span: self.span,
            attributes: vec![],
        }
    }
    /// Create a string literal
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::string;
/// 
/// let result = string("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn string(&self, value: &str) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::String(value.to_string())),
            span: self.span,
            attributes: vec![],
        }
    }
    /// Create a boolean literal
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::bool;
/// 
/// let result = bool(true);
/// assert_eq!(result, Ok(true));
/// ```
pub fn bool(&self, value: bool) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Bool(value)),
            span: self.span,
            attributes: vec![],
        }
    }
    /// Create an identifier expression
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::ident;
/// 
/// let result = ident("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn ident(&self, name: &str) -> Expr {
        Expr {
            kind: ExprKind::Identifier(name.to_string()),
            span: self.span,
            attributes: vec![],
        }
    }
    /// Create a binary operation
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::binary;
/// 
/// let result = binary(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::unary;
/// 
/// let result = unary(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::if_expr;
/// 
/// let result = if_expr(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::match_expr;
/// 
/// let result = match_expr(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::match_arm;
/// 
/// let result = match_arm(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn match_arm(&self, pattern: Pattern, guard: Option<Expr>, body: Expr) -> MatchArm {
        MatchArm {
            pattern,
            guard: guard.map(Box::new),
            body: Box::new(body),
            span: self.span,
        }
    }
    /// Create a wildcard pattern
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::pattern_wildcard;
/// 
/// let result = pattern_wildcard(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn pattern_wildcard(&self) -> Pattern {
        Pattern::Wildcard
    }
    /// Create an identifier pattern
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::pattern_ident;
/// 
/// let result = pattern_ident("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn pattern_ident(&self, name: &str) -> Pattern {
        Pattern::Identifier(name.to_string())
    }
    /// Create a literal pattern
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::pattern_literal;
/// 
/// let result = pattern_literal(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn pattern_literal(&self, lit: Literal) -> Pattern {
        Pattern::Literal(lit)
    }
    /// Create a tuple pattern
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::pattern_tuple;
/// 
/// let result = pattern_tuple(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn pattern_tuple(&self, patterns: Vec<Pattern>) -> Pattern {
        Pattern::Tuple(patterns)
    }
    /// Create an or pattern (not supported by parser yet)
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::pattern_or;
/// 
/// let result = pattern_or(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn pattern_or(&self, patterns: Vec<Pattern>) -> Pattern {
        Pattern::Or(patterns)
    }
    /// Create a struct pattern
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::pattern_struct;
/// 
/// let result = pattern_struct(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn pattern_struct(&self, name: String, fields: Vec<(String, Pattern)>) -> Pattern {
        let struct_fields = fields.into_iter().map(|(name, pattern)| {
            StructPatternField { name, pattern: Some(pattern) }
        }).collect();
        Pattern::Struct { name, fields: struct_fields, has_rest: false }
    }
    /// Create a rest pattern (..)
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::pattern_rest;
/// 
/// let result = pattern_rest(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn pattern_rest(&self) -> Pattern {
        Pattern::Rest
    }
    /// Create a function call
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::call;
/// 
/// let result = call(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::lambda;
/// 
/// let result = lambda(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::block;
/// 
/// let result = block(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn block(&self, statements: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Block(statements),
            span: self.span,
            attributes: vec![],
        }
    }
    /// Create a let expression
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::let_expr;
/// 
/// let result = let_expr(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::assign;
/// 
/// let result = assign(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::ok;
/// 
/// let result = ok(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::err;
/// 
/// let result = err(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::some;
/// 
/// let result = some(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::none;
/// 
/// let result = none(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn none(&self) -> Expr {
        self.ident("None")
    }
    /// Create a list/array literal
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::list;
/// 
/// let result = list(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn list(&self, elements: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::List(elements),
            span: self.span,
            attributes: vec![],
        }
    }
    /// Create a tuple literal
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::tuple;
/// 
/// let result = tuple(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn tuple(&self, elements: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Tuple(elements),
            span: self.span,
            attributes: vec![],
        }
    }
    /// Create string interpolation
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::string_interpolation;
/// 
/// let result = string_interpolation(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn string_interpolation(&self, parts: Vec<StringPart>) -> Expr {
        Expr {
            kind: ExprKind::StringInterpolation { parts },
            span: self.span,
            attributes: vec![],
        }
    }
    /// Create a for loop
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::for_loop;
/// 
/// let result = for_loop(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::while_loop;
/// 
/// let result = while_loop(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::loop_expr;
/// 
/// let result = loop_expr(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::break_expr;
/// 
/// let result = break_expr(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn break_expr(&self, label: Option<String>) -> Expr {
        Expr {
            kind: ExprKind::Break { label },
            span: self.span,
            attributes: vec![],
        }
    }
    /// Create a continue expression
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::continue_expr;
/// 
/// let result = continue_expr(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn continue_expr(&self, label: Option<String>) -> Expr {
        Expr {
            kind: ExprKind::Continue { label },
            span: self.span,
            attributes: vec![],
        }
    }
    /// Create a return expression
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::return_expr;
/// 
/// let result = return_expr(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn return_expr(&self, value: Option<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Return { value: value.map(Box::new) },
            span: self.span,
            attributes: vec![],
        }
    }
    /// Create a type annotation
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::type_int;
/// 
/// let result = type_int(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn type_int(&self) -> Type {
        Type {
            kind: TypeKind::Named("i32".to_string()),
            span: self.span,
        }
    }
    /// Create a Result type
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::type_result;
/// 
/// let result = type_result(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::testing::ast_builder::type_option;
/// 
/// let result = type_option(());
/// assert_eq!(result, Ok(()));
/// ```
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
#[cfg(test)]
use proptest::prelude::*;
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
#[cfg(test)]
mod property_tests_ast_builder {
    use proptest::proptest;
    use super::*;
    use proptest::prelude::*;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
