//! Abstract Syntax Tree definitions for Ruchy

use serde::{Deserialize, Serialize};
use std::fmt;

/// Source location tracking for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[must_use]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// Catch clause in try-catch blocks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatchClause {
    pub pattern: Pattern,  // The error pattern to match
    pub body: Box<Expr>,    // The catch block body
}

/// The main AST node type for expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
    pub attributes: Vec<Attribute>,
}

impl Expr {
    #[must_use]
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self {
            kind,
            span,
            attributes: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_attributes(kind: ExprKind, span: Span, attributes: Vec<Attribute>) -> Self {
        Self {
            kind,
            span,
            attributes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExprKind {
    Literal(Literal),
    Identifier(String),
    QualifiedName {
        module: String,
        name: String,
    },
    StringInterpolation {
        parts: Vec<StringPart>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Throw {
        expr: Box<Expr>,
    },
    TryCatch {
        try_block: Box<Expr>,
        catch_clauses: Vec<CatchClause>,
        finally_block: Option<Box<Expr>>,
    },
    Ok {
        value: Box<Expr>,
    },
    Err {
        error: Box<Expr>,
    },
    Some {
        value: Box<Expr>,
    },
    None,
    TypeCast {
        expr: Box<Expr>,
        target_type: String,
    },
    Try {
        expr: Box<Expr>,
    },
    Await {
        expr: Box<Expr>,
    },
    AsyncBlock {
        body: Box<Expr>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    IfLet {
        pattern: Pattern,
        expr: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    Let {
        name: String,
        type_annotation: Option<Type>,
        value: Box<Expr>,
        body: Box<Expr>,
        is_mutable: bool,
    },
    LetPattern {
        pattern: Pattern,
        type_annotation: Option<Type>,
        value: Box<Expr>,
        body: Box<Expr>,
        is_mutable: bool,
    },
    Function {
        name: String,
        type_params: Vec<String>,
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Box<Expr>,
        is_async: bool,
        is_pub: bool,
    },
    Lambda {
        params: Vec<Param>,
        body: Box<Expr>,
    },
    Struct {
        name: String,
        type_params: Vec<String>,
        fields: Vec<StructField>,
        is_pub: bool,
    },
    Enum {
        name: String,
        type_params: Vec<String>,
        variants: Vec<EnumVariant>,
        is_pub: bool,
    },
    StructLiteral {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    ObjectLiteral {
        fields: Vec<ObjectField>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    OptionalFieldAccess {
        object: Box<Expr>,
        field: String,
    },
    IndexAccess {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Slice {
        object: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    Trait {
        name: String,
        type_params: Vec<String>,
        methods: Vec<TraitMethod>,
        is_pub: bool,
    },
    Impl {
        type_params: Vec<String>,
        trait_name: Option<String>,
        for_type: String,
        methods: Vec<ImplMethod>,
        is_pub: bool,
    },
    Actor {
        name: String,
        state: Vec<StructField>,
        handlers: Vec<ActorHandler>,
    },
    Send {
        actor: Box<Expr>,
        message: Box<Expr>,
    },
    Command {
        program: String,
        args: Vec<String>,
        env: Vec<(String, String)>,
        working_dir: Option<String>,
    },
    Ask {
        actor: Box<Expr>,
        message: Box<Expr>,
        timeout: Option<Box<Expr>>,
    },
    /// Fire-and-forget actor send (left <- right)
    ActorSend {
        actor: Box<Expr>,
        message: Box<Expr>,
    },
    /// Actor query with reply (left <? right)
    ActorQuery {
        actor: Box<Expr>,
        message: Box<Expr>,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    Macro {
        name: String,
        args: Vec<Expr>,
    },
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    OptionalMethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    Block(Vec<Expr>),
    Pipeline {
        expr: Box<Expr>,
        stages: Vec<PipelineStage>,
    },
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    List(Vec<Expr>),
    Tuple(Vec<Expr>),
    Spread {
        expr: Box<Expr>,
    },
    ListComprehension {
        element: Box<Expr>,
        variable: String,
        iterable: Box<Expr>,
        condition: Option<Box<Expr>>,
    },
    DataFrame {
        columns: Vec<DataFrameColumn>,
    },
    DataFrameOperation {
        source: Box<Expr>,
        operation: DataFrameOp,
    },
    For {
        var: String,  // Keep for backward compatibility
        pattern: Option<Pattern>,  // New: Support destructuring patterns
        iter: Box<Expr>,
        body: Box<Expr>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Expr>,
    },
    WhileLet {
        pattern: Pattern,
        expr: Box<Expr>,
        body: Box<Expr>,
    },
    Loop {
        body: Box<Expr>,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
    Import {
        path: String,
        items: Vec<ImportItem>,
    },
    Module {
        name: String,
        body: Box<Expr>,
    },
    Export {
        items: Vec<String>,
    },
    Break {
        label: Option<String>,
    },
    Continue {
        label: Option<String>,
    },
    Return {
        value: Option<Box<Expr>>,
    },
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    CompoundAssign {
        target: Box<Expr>,
        op: BinaryOp,
        value: Box<Expr>,
    },
    PreIncrement {
        target: Box<Expr>,
    },
    PostIncrement {
        target: Box<Expr>,
    },
    PreDecrement {
        target: Box<Expr>,
    },
    PostDecrement {
        target: Box<Expr>,
    },
    Extension {
        target_type: String,
        methods: Vec<ImplMethod>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Char(char),
    Unit,
}

impl Literal {
    /// Convert a REPL Value to a Literal (for synthetic expressions)
    pub fn from_value(value: &crate::runtime::repl::Value) -> Self {
        use crate::runtime::repl::Value;
        match value {
            Value::Int(i) => Literal::Integer(*i),
            Value::Float(f) => Literal::Float(*f),
            Value::String(s) => Literal::String(s.clone()),
            Value::Bool(b) => Literal::Bool(*b),
            Value::Char(c) => Literal::Char(*c),
            Value::Unit => Literal::Unit,
            _ => Literal::Unit, // Fallback for complex types
        }
    }
}

/// String interpolation parts - either literal text or an expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StringPart {
    /// Literal text portion of the string
    Text(String),
    /// Expression to be interpolated without format specifier
    Expr(Box<Expr>),
    /// Expression with format specifier (e.g., {value:.2})
    ExprWithFormat {
        expr: Box<Expr>,
        format_spec: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,

    // Comparison
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Logical
    And,
    Or,
    NullCoalesce,

    // Bitwise
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
    Negate,
    BitwiseNot,
    Reference,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub pattern: Pattern,
    pub ty: Type,
    pub span: Span,
    pub is_mutable: bool,
    pub default_value: Option<Box<Expr>>,
}

impl Param {
    /// Get the primary name from this parameter pattern.
    /// For complex patterns, this returns the first/primary identifier.
    /// For simple patterns, this returns the identifier itself.
    #[must_use]
    pub fn name(&self) -> String {
        self.pattern.primary_name()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Option<Vec<Type>>, // None for unit variant, Some for tuple variant
    pub discriminant: Option<i64>, // Explicit discriminant value for TypeScript compatibility
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ObjectField {
    KeyValue { key: String, value: Expr },
    Spread { expr: Expr },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Option<Box<Expr>>, // None for method signatures, Some for default implementations
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImplMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Box<Expr>,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActorHandler {
    pub message_type: String,
    pub params: Vec<Param>,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeKind {
    Named(String),
    Generic { base: String, params: Vec<Type> },
    Optional(Box<Type>),
    List(Box<Type>),
    Array { elem_type: Box<Type>, size: usize },
    Tuple(Vec<Type>),
    Function { params: Vec<Type>, ret: Box<Type> },
    DataFrame { columns: Vec<(String, Type)> },
    Series { dtype: Box<Type> },
    Reference { is_mut: bool, inner: Box<Type> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipelineStage {
    pub op: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,  // Pattern guard: if condition
    pub body: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Wildcard,
    Literal(Literal),
    Identifier(String),
    QualifiedName(Vec<String>), // For patterns like Ordering::Less
    Tuple(Vec<Pattern>),
    List(Vec<Pattern>),
    Struct {
        name: String,
        fields: Vec<StructPatternField>,
        has_rest: bool,
    },
    Range {
        start: Box<Pattern>,
        end: Box<Pattern>,
        inclusive: bool,
    },
    Or(Vec<Pattern>),
    Rest, // For ... patterns
    RestNamed(String), // For ..name patterns
    WithDefault {
        pattern: Box<Pattern>,
        default: Box<Expr>,
    }, // For patterns with default values like a = 10
    Ok(Box<Pattern>),
    Err(Box<Pattern>),
    Some(Box<Pattern>),
    None,
}

impl Pattern {
    /// Get the primary identifier name from this pattern.
    /// For complex patterns, returns the first/most significant identifier.
    #[must_use]
    pub fn primary_name(&self) -> String {
        match self {
            Pattern::Identifier(name) => name.clone(),
            Pattern::QualifiedName(path) => path.join("::"),
            Pattern::Tuple(patterns) => {
                // Return the name of the first pattern
                patterns
                    .first()
                    .map_or_else(|| "_tuple".to_string(), Pattern::primary_name)
            }
            Pattern::List(patterns) => {
                // Return the name of the first pattern
                patterns
                    .first()
                    .map_or_else(|| "_list".to_string(), Pattern::primary_name)
            }
            Pattern::Struct { name, fields, .. } => {
                // Return the struct type name, or first field name if anonymous
                if name.is_empty() {
                    fields.first().map_or_else(|| "_struct".to_string(), |f| f.name.clone())
                } else {
                    name.clone()
                }
            }
            Pattern::Ok(inner) | Pattern::Err(inner) | Pattern::Some(inner) => inner.primary_name(),
            Pattern::None => "_none".to_string(),
            Pattern::Or(patterns) => {
                // Return the name of the first pattern
                patterns
                    .first()
                    .map_or_else(|| "_or".to_string(), Pattern::primary_name)
            }
            Pattern::Wildcard => "_".to_string(),
            Pattern::Rest => "_rest".to_string(),
            Pattern::RestNamed(name) => name.clone(),
            Pattern::WithDefault { pattern, .. } => pattern.primary_name(),
            Pattern::Literal(lit) => format!("_literal_{lit:?}"),
            Pattern::Range { .. } => "_range".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructPatternField {
    pub name: String,
    pub pattern: Option<Pattern>, // None for shorthand like { x } instead of { x: x }
}


/// Custom error type definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorTypeDef {
    pub name: String,
    pub fields: Vec<StructField>,
    pub extends: Option<String>, // Parent error type
}

/// Attribute for annotating expressions (e.g., `#[property]`)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataFrameColumn {
    pub name: String,
    pub values: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataFrameOp {
    Filter(Box<Expr>),
    Select(Vec<String>),
    GroupBy(Vec<String>),
    Sort(Vec<String>),
    Join {
        other: Box<Expr>,
        on: Vec<String>,
        how: JoinType,
    },
    Aggregate(Vec<AggregateOp>),
    Limit(usize),
    Head(usize),
    Tail(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Outer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportItem {
    /// Import a specific name: `use std::collections::HashMap`
    Named(String),
    /// Import with alias: `use std::collections::HashMap as Map`
    Aliased { name: String, alias: String },
    /// Import all: `use std::collections::*`
    Wildcard,
}

impl ImportItem {
    /// Check if this import is for a URL module
    pub fn is_url_import(path: &str) -> bool {
        path.starts_with("https://") || path.starts_with("http://")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AggregateOp {
    Sum(String),
    Mean(String),
    Min(String),
    Max(String),
    Count(String),
    Std(String),
    Var(String),
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Subtract => write!(f, "-"),
            Self::Multiply => write!(f, "*"),
            Self::Divide => write!(f, "/"),
            Self::Modulo => write!(f, "%"),
            Self::Power => write!(f, "**"),
            Self::Equal => write!(f, "=="),
            Self::NotEqual => write!(f, "!="),
            Self::Less => write!(f, "<"),
            Self::LessEqual => write!(f, "<="),
            Self::Greater => write!(f, ">"),
            Self::GreaterEqual => write!(f, ">="),
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
            Self::NullCoalesce => write!(f, "??"),
            Self::BitwiseAnd => write!(f, "&"),
            Self::BitwiseOr => write!(f, "|"),
            Self::BitwiseXor => write!(f, "^"),
            Self::LeftShift => write!(f, "<<"),
        }
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Not => write!(f, "!"),
            Self::Negate => write!(f, "-"),
            Self::BitwiseNot => write!(f, "~"),
            Self::Reference => write!(f, "&"),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_span_merge(start1 in 0usize..1000, end1 in 0usize..1000,
                          start2 in 0usize..1000, end2 in 0usize..1000) {
            let span1 = Span::new(start1, end1);
            let span2 = Span::new(start2, end2);
            let merged = span1.merge(span2);

            prop_assert!(merged.start <= span1.start);
            prop_assert!(merged.start <= span2.start);
            prop_assert!(merged.end >= span1.end);
            prop_assert!(merged.end >= span2.end);
        }
    }

    #[test]
    fn test_ast_size() {
        // Track AST node sizes for optimization
        let expr_size = std::mem::size_of::<Expr>();
        let kind_size = std::mem::size_of::<ExprKind>();
        // Current sizes are larger than ideal but acceptable for MVP
        // Future optimization: Use arena allocation and indices
        assert!(expr_size <= 192, "Expr too large: {expr_size} bytes");
        assert!(kind_size <= 152, "ExprKind too large: {kind_size} bytes");
    }

    #[test]
    fn test_span_creation() {
        let span = Span::new(10, 20);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
    }

    #[test]
    fn test_span_merge_simple() {
        let span1 = Span::new(5, 10);
        let span2 = Span::new(8, 15);
        let merged = span1.merge(span2);
        assert_eq!(merged.start, 5);
        assert_eq!(merged.end, 15);
    }

    #[test]
    fn test_span_merge_disjoint() {
        let span1 = Span::new(0, 5);
        let span2 = Span::new(10, 15);
        let merged = span1.merge(span2);
        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 15);
    }

    #[test]
    fn test_expr_creation() {
        let span = Span::new(0, 10);
        let expr = Expr::new(ExprKind::Literal(Literal::Integer(42)), span);
        assert_eq!(expr.span.start, 0);
        assert_eq!(expr.span.end, 10);
        match expr.kind {
            ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 42),
            _ => panic!("Wrong expression kind"),
        }
    }

    #[test]
    fn test_literal_variants() {
        let literals = vec![
            Literal::Integer(42),
            #[allow(clippy::approx_constant)]
            Literal::Float(3.14), // Not PI, just a test value
            Literal::String("hello".to_string()),
            Literal::Bool(true),
            Literal::Unit,
        ];

        for lit in literals {
            let expr = Expr::new(ExprKind::Literal(lit.clone()), Span::new(0, 0));
            match expr.kind {
                ExprKind::Literal(l) => assert_eq!(l, lit),
                _ => panic!("Expected literal"),
            }
        }
    }

    #[test]
    fn test_binary_op_display() {
        assert_eq!(BinaryOp::Add.to_string(), "+");
        assert_eq!(BinaryOp::Subtract.to_string(), "-");
        assert_eq!(BinaryOp::Multiply.to_string(), "*");
        assert_eq!(BinaryOp::Divide.to_string(), "/");
        assert_eq!(BinaryOp::Modulo.to_string(), "%");
        assert_eq!(BinaryOp::Power.to_string(), "**");
        assert_eq!(BinaryOp::Equal.to_string(), "==");
        assert_eq!(BinaryOp::NotEqual.to_string(), "!=");
        assert_eq!(BinaryOp::Less.to_string(), "<");
        assert_eq!(BinaryOp::LessEqual.to_string(), "<=");
        assert_eq!(BinaryOp::Greater.to_string(), ">");
        assert_eq!(BinaryOp::GreaterEqual.to_string(), ">=");
        assert_eq!(BinaryOp::And.to_string(), "&&");
        assert_eq!(BinaryOp::Or.to_string(), "||");
        assert_eq!(BinaryOp::BitwiseAnd.to_string(), "&");
        assert_eq!(BinaryOp::BitwiseOr.to_string(), "|");
        assert_eq!(BinaryOp::BitwiseXor.to_string(), "^");
        assert_eq!(BinaryOp::LeftShift.to_string(), "<<");
    }

    #[test]
    fn test_unary_op_display() {
        assert_eq!(UnaryOp::Not.to_string(), "!");
        assert_eq!(UnaryOp::Negate.to_string(), "-");
        assert_eq!(UnaryOp::BitwiseNot.to_string(), "~");
        assert_eq!(UnaryOp::Reference.to_string(), "&");
    }

    #[test]
    fn test_binary_expression() {
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1)),
            Span::new(0, 1),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(2)),
            Span::new(4, 5),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Add,
                right,
            },
            Span::new(0, 5),
        );

        match expr.kind {
            ExprKind::Binary {
                left: l,
                op,
                right: r,
            } => {
                assert_eq!(op, BinaryOp::Add);
                match l.kind {
                    ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 1),
                    _ => panic!("Wrong left operand"),
                }
                match r.kind {
                    ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 2),
                    _ => panic!("Wrong right operand"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_unary_expression() {
        let operand = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(1, 5),
        ));
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Not,
                operand,
            },
            Span::new(0, 5),
        );

        match expr.kind {
            ExprKind::Unary { op, operand } => {
                assert_eq!(op, UnaryOp::Not);
                match operand.kind {
                    ExprKind::Literal(Literal::Bool(b)) => assert!(b),
                    _ => panic!("Wrong operand"),
                }
            }
            _ => panic!("Expected unary expression"),
        }
    }

    #[test]
    fn test_if_expression() {
        let condition = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(3, 7),
        ));
        let then_branch = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1)),
            Span::new(10, 11),
        ));
        let else_branch = Some(Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(2)),
            Span::new(17, 18),
        )));

        let expr = Expr::new(
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            },
            Span::new(0, 18),
        );

        match expr.kind {
            ExprKind::If {
                condition: c,
                then_branch: t,
                else_branch: e,
            } => {
                match c.kind {
                    ExprKind::Literal(Literal::Bool(b)) => assert!(b),
                    _ => panic!("Wrong condition"),
                }
                match t.kind {
                    ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 1),
                    _ => panic!("Wrong then branch"),
                }
                assert!(e.is_some());
                if let Some(else_expr) = e {
                    match else_expr.kind {
                        ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 2),
                        _ => panic!("Wrong else branch"),
                    }
                }
            }
            _ => panic!("Expected if expression"),
        }
    }

    #[test]
    fn test_let_expression() {
        let value = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            Span::new(8, 10),
        ));
        let body = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(14, 15),
        ));

        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value,
                body,
                is_mutable: false,
            },
            Span::new(0, 15),
        );

        match expr.kind {
            ExprKind::Let {
                name,
                value: v,
                body: b,
                ..
            } => {
                assert_eq!(name, "x");
                match v.kind {
                    ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 42),
                    _ => panic!("Wrong value"),
                }
                match b.kind {
                    ExprKind::Identifier(id) => assert_eq!(id, "x"),
                    _ => panic!("Wrong body"),
                }
            }
            _ => panic!("Expected let expression"),
        }
    }

    #[test]
    fn test_function_expression() {
        let params = vec![Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::new(10, 13),
            },
            span: Span::new(8, 13),
            is_mutable: false,
            default_value: None,
        }];
        let body = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(20, 21),
        ));

        let expr = Expr::new(
            ExprKind::Function {
                name: "identity".to_string(),
                type_params: vec![],
                params,
                return_type: Some(Type {
                    kind: TypeKind::Named("i32".to_string()),
                    span: Span::new(16, 19),
                }),
                body,
                is_async: false,
                is_pub: false,
            },
            Span::new(0, 22),
        );

        match expr.kind {
            ExprKind::Function {
                name,
                params: p,
                return_type,
                body: b,
                ..
            } => {
                assert_eq!(name, "identity");
                assert_eq!(p.len(), 1);
                assert_eq!(p[0].name(), "x");
                assert!(return_type.is_some());
                match b.kind {
                    ExprKind::Identifier(id) => assert_eq!(id, "x"),
                    _ => panic!("Wrong body"),
                }
            }
            _ => panic!("Expected function expression"),
        }
    }

    #[test]
    fn test_call_expression() {
        let func = Box::new(Expr::new(
            ExprKind::Identifier("add".to_string()),
            Span::new(0, 3),
        ));
        let args = vec![
            Expr::new(ExprKind::Literal(Literal::Integer(1)), Span::new(4, 5)),
            Expr::new(ExprKind::Literal(Literal::Integer(2)), Span::new(7, 8)),
        ];

        let expr = Expr::new(ExprKind::Call { func, args }, Span::new(0, 9));

        match expr.kind {
            ExprKind::Call { func: f, args: a } => {
                match f.kind {
                    ExprKind::Identifier(name) => assert_eq!(name, "add"),
                    _ => panic!("Wrong function"),
                }
                assert_eq!(a.len(), 2);
            }
            _ => panic!("Expected call expression"),
        }
    }

    #[test]
    fn test_block_expression() {
        let exprs = vec![
            Expr::new(ExprKind::Literal(Literal::Integer(1)), Span::new(2, 3)),
            Expr::new(ExprKind::Literal(Literal::Integer(2)), Span::new(5, 6)),
        ];

        let expr = Expr::new(ExprKind::Block(exprs), Span::new(0, 8));

        match expr.kind {
            ExprKind::Block(block) => {
                assert_eq!(block.len(), 2);
            }
            _ => panic!("Expected block expression"),
        }
    }

    #[test]
    fn test_list_expression() {
        let items = vec![
            Expr::new(ExprKind::Literal(Literal::Integer(1)), Span::new(1, 2)),
            Expr::new(ExprKind::Literal(Literal::Integer(2)), Span::new(4, 5)),
            Expr::new(ExprKind::Literal(Literal::Integer(3)), Span::new(7, 8)),
        ];

        let expr = Expr::new(ExprKind::List(items), Span::new(0, 9));

        match expr.kind {
            ExprKind::List(list) => {
                assert_eq!(list.len(), 3);
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_for_expression() {
        let iter = Box::new(Expr::new(
            ExprKind::Range {
                start: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(0)),
                    Span::new(10, 11),
                )),
                end: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(10)),
                    Span::new(13, 15),
                )),
                inclusive: false,
            },
            Span::new(10, 15),
        ));
        let body = Box::new(Expr::new(
            ExprKind::Identifier("i".to_string()),
            Span::new(20, 21),
        ));

        let expr = Expr::new(
            ExprKind::For {
                var: "i".to_string(),
                pattern: None,
                iter,
                body,
            },
            Span::new(0, 22),
        );

        match expr.kind {
            ExprKind::For {
                var,
                iter: it,
                body: b,
                ..
            } => {
                assert_eq!(var, "i");
                match it.kind {
                    ExprKind::Range { .. } => {}
                    _ => panic!("Wrong iterator"),
                }
                match b.kind {
                    ExprKind::Identifier(id) => assert_eq!(id, "i"),
                    _ => panic!("Wrong body"),
                }
            }
            _ => panic!("Expected for expression"),
        }
    }

    #[test]
    fn test_range_expression() {
        let start = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1)),
            Span::new(0, 1),
        ));
        let end = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(10)),
            Span::new(3, 5),
        ));

        let expr = Expr::new(
            ExprKind::Range {
                start,
                end,
                inclusive: false,
            },
            Span::new(0, 5),
        );

        match expr.kind {
            ExprKind::Range {
                start: s,
                end: e,
                inclusive,
            } => {
                assert!(!inclusive);
                match s.kind {
                    ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 1),
                    _ => panic!("Wrong start"),
                }
                match e.kind {
                    ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 10),
                    _ => panic!("Wrong end"),
                }
            }
            _ => panic!("Expected range expression"),
        }
    }

    #[test]
    fn test_import_expression() {
        let expr = Expr::new(
            ExprKind::Import {
                path: "std::collections".to_string(),
                items: vec![
                    ImportItem::Named("HashMap".to_string()),
                    ImportItem::Named("HashSet".to_string()),
                ],
            },
            Span::new(0, 30),
        );

        match expr.kind {
            ExprKind::Import { path, items } => {
                assert_eq!(path, "std::collections");
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], ImportItem::Named("HashMap".to_string()));
                assert_eq!(items[1], ImportItem::Named("HashSet".to_string()));
            }
            _ => panic!("Expected import expression"),
        }
    }

    #[test]
    fn test_pipeline_expression() {
        let expr_start = Box::new(Expr::new(
            ExprKind::List(vec![
                Expr::new(ExprKind::Literal(Literal::Integer(1)), Span::new(1, 2)),
                Expr::new(ExprKind::Literal(Literal::Integer(2)), Span::new(4, 5)),
            ]),
            Span::new(0, 6),
        ));
        let stages = vec![PipelineStage {
            op: Box::new(Expr::new(
                ExprKind::Identifier("filter".to_string()),
                Span::new(10, 16),
            )),
            span: Span::new(10, 16),
        }];

        let expr = Expr::new(
            ExprKind::Pipeline {
                expr: expr_start,
                stages,
            },
            Span::new(0, 16),
        );

        match expr.kind {
            ExprKind::Pipeline { expr: e, stages: s } => {
                assert_eq!(s.len(), 1);
                match e.kind {
                    ExprKind::List(list) => assert_eq!(list.len(), 2),
                    _ => panic!("Wrong pipeline start"),
                }
            }
            _ => panic!("Expected pipeline expression"),
        }
    }

    #[test]
    fn test_match_expression() {
        let expr_to_match = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(6, 7),
        ));
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Integer(1)),
                guard: None,
                body: Box::new(Expr::new(
                    ExprKind::Literal(Literal::String("one".to_string())),
                    Span::new(15, 20),
                )),
                span: Span::new(10, 20),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(Expr::new(
                    ExprKind::Literal(Literal::String("other".to_string())),
                    Span::new(28, 35),
                )),
                span: Span::new(25, 35),
            },
        ];

        let expr = Expr::new(
            ExprKind::Match {
                expr: expr_to_match,
                arms,
            },
            Span::new(0, 36),
        );

        match expr.kind {
            ExprKind::Match { expr: e, arms: a } => {
                assert_eq!(a.len(), 2);
                match e.kind {
                    ExprKind::Identifier(id) => assert_eq!(id, "x"),
                    _ => panic!("Wrong match expression"),
                }
            }
            _ => panic!("Expected match expression"),
        }
    }

    #[test]
    fn test_pattern_variants() {
        let patterns = vec![
            Pattern::Wildcard,
            Pattern::Literal(Literal::Integer(42)),
            Pattern::Identifier("x".to_string()),
            Pattern::Tuple(vec![
                Pattern::Literal(Literal::Integer(1)),
                Pattern::Identifier("x".to_string()),
            ]),
            Pattern::List(vec![
                Pattern::Literal(Literal::Integer(1)),
                Pattern::Literal(Literal::Integer(2)),
            ]),
            Pattern::Struct {
                name: "Point".to_string(),
                fields: vec![StructPatternField {
                    name: "x".to_string(),
                    pattern: Some(Pattern::Identifier("x".to_string())),
                }],
                has_rest: false,
            },
            Pattern::Range {
                start: Box::new(Pattern::Literal(Literal::Integer(1))),
                end: Box::new(Pattern::Literal(Literal::Integer(10))),
                inclusive: true,
            },
            Pattern::Or(vec![
                Pattern::Literal(Literal::Integer(1)),
                Pattern::Literal(Literal::Integer(2)),
            ]),
            Pattern::Rest,
        ];

        for pattern in patterns {
            match pattern {
                Pattern::Tuple(list) | Pattern::List(list) => assert!(!list.is_empty()),
                Pattern::Struct { fields, .. } => assert!(!fields.is_empty()),
                Pattern::Or(patterns) => assert!(!patterns.is_empty()),
                Pattern::Range { .. }
                | Pattern::Wildcard
                | Pattern::Literal(_)
                | Pattern::Identifier(_)
                | Pattern::Rest
                | Pattern::RestNamed(_)
                | Pattern::Ok(_)
                | Pattern::Err(_)
                | Pattern::Some(_)
                | Pattern::None
                | Pattern::QualifiedName(_) 
                | Pattern::WithDefault { .. } => {} // Simple patterns
            }
        }
    }

    #[test]
    fn test_type_kinds() {
        let types = vec![
            Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::new(0, 3),
            },
            Type {
                kind: TypeKind::Optional(Box::new(Type {
                    kind: TypeKind::Named("String".to_string()),
                    span: Span::new(0, 6),
                })),
                span: Span::new(0, 7),
            },
            Type {
                kind: TypeKind::List(Box::new(Type {
                    kind: TypeKind::Named("f64".to_string()),
                    span: Span::new(1, 4),
                })),
                span: Span::new(0, 5),
            },
            Type {
                kind: TypeKind::Function {
                    params: vec![Type {
                        kind: TypeKind::Named("i32".to_string()),
                        span: Span::new(0, 3),
                    }],
                    ret: Box::new(Type {
                        kind: TypeKind::Named("String".to_string()),
                        span: Span::new(7, 13),
                    }),
                },
                span: Span::new(0, 13),
            },
        ];

        for ty in types {
            match ty.kind {
                TypeKind::Named(name) => assert!(!name.is_empty()),
                TypeKind::Generic { base, params } => {
                    assert!(!base.is_empty());
                    assert!(!params.is_empty());
                }
                TypeKind::Optional(_) | TypeKind::List(_) | TypeKind::Series { .. } => {}
                TypeKind::Function { params, .. } => assert!(!params.is_empty()),
                TypeKind::DataFrame { columns } => assert!(!columns.is_empty()),
                TypeKind::Tuple(ref types) => assert!(!types.is_empty()),
                TypeKind::Reference { is_mut: _, ref inner } => {
                    // Reference types should have a valid inner type
                    if let TypeKind::Named(ref name) = inner.kind { 
                        assert!(!name.is_empty());
                    }
                }
            }
        }
    }

    #[test]
    fn test_param_creation() {
        let param = Param {
            pattern: Pattern::Identifier("count".to_string()),
            ty: Type {
                kind: TypeKind::Named("usize".to_string()),
                span: Span::new(6, 11),
            },
            span: Span::new(0, 11),
            is_mutable: false,
            default_value: None,
        };

        assert_eq!(param.name(), "count");
        match param.ty.kind {
            TypeKind::Named(name) => assert_eq!(name, "usize"),
            _ => panic!("Wrong type kind"),
        }
    }
}
