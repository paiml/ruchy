//! Abstract Syntax Tree (AST) definitions for the Ruchy programming language.
//!
//! This module contains the complete AST representation used by the Ruchy compiler.
//! The AST is the primary intermediate representation produced by the parser and consumed
//! by subsequent compilation phases including type checking, optimization, and code generation.
//!
//! # Architecture
//!
//! The AST follows a traditional expression-based design where most constructs are
//! represented as expressions that can be composed. Key design principles:
//!
//! - **Location tracking**: Every AST node carries a `Span` for precise error reporting
//! - **Attributes**: Nodes can be decorated with attributes for metadata and directives
//! - **Pattern matching**: First-class support for destructuring and pattern matching
//! - **Type annotations**: Optional type annotations for gradual typing
//!
//! # Example
//!
//! ```ignore
//! use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span};
//!
//! // Create a simple literal expression
//! let expr = Expr::new(
//!     ExprKind::Literal(Literal::Integer(42)),
//!     Span::new(0, 2)
//! );
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Source location information for AST nodes.
///
/// A `Span` represents a contiguous range of characters in the source code,
/// enabling precise error reporting and source mapping. All AST nodes carry
/// span information to maintain the connection between the abstract representation
/// and the original source text.
///
/// # Examples
///
/// ```ignore
/// use ruchy::frontend::ast::Span;
///
/// // Create a span for characters 10-15 in the source
/// let span = Span::new(10, 15);
///
/// // Merge two spans to get their combined range
/// let span1 = Span::new(10, 20);
/// let span2 = Span::new(15, 25);
/// let merged = span1.merge(span2); // Span { start: 10, end: 25 }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Span {
    /// The byte offset of the first character in this span.
    pub start: usize,
    /// The byte offset one past the last character in this span.
    pub end: usize,
}
impl Span {
    /// Creates a new span with the given start and end positions.
    ///
    /// # Arguments
    ///
    /// * `start` - The byte offset of the first character
    /// * `end` - The byte offset one past the last character
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::frontend::ast::Span;
    ///
    /// let span = Span::new(0, 10);
    /// assert_eq!(span.start, 0);
    /// assert_eq!(span.end, 10);
    /// ```
    #[must_use]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
    /// Merges two spans to create a new span covering both ranges.
    ///
    /// This is useful when combining multiple tokens or expressions into
    /// a larger syntactic construct. The resulting span starts at the
    /// minimum of both start positions and ends at the maximum of both
    /// end positions.
    ///
    /// # Arguments
    ///
    /// * `other` - The span to merge with this one
    ///
    /// # Returns
    ///
    /// A new span covering the entire range of both input spans
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::frontend::ast::Span;
    ///
    /// let span1 = Span::new(10, 20);
    /// let span2 = Span::new(15, 25);
    /// let merged = span1.merge(span2);
    /// assert_eq!(merged.start, 10);
    /// assert_eq!(merged.end, 25);
    /// ```
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}
/// A catch clause in a try-catch expression.
///
/// Catch clauses provide pattern-based error handling, allowing different
/// error types or patterns to be handled with specific recovery logic.
///
/// # Examples
///
/// ```ignore
/// // Catch a specific error pattern
/// try {
///     risky_operation()
/// } catch IOError(msg) {
///     println("IO error: {msg}")
/// } catch e {
///     println("Unknown error: {e}")
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatchClause {
    /// The pattern to match against the caught error.
    pub pattern: Pattern,
    /// The expression to execute when this pattern matches.
    pub body: Box<Expr>,
}
/// The primary AST node representing an expression in Ruchy.
///
/// `Expr` is the fundamental building block of Ruchy's AST. Nearly all language
/// constructs are represented as expressions, including statements, declarations,
/// and control flow. This expression-oriented design enables powerful composition
/// and simplifies the language semantics.
///
/// Each expression consists of:
/// - `kind`: The specific type of expression (literal, binary op, function, etc.)
/// - `span`: Source location information for error reporting
/// - `attributes`: Optional metadata and compiler directives
///
/// # Examples
///
/// ```ignore
/// use ruchy::frontend::ast::{Expr, ExprKind, BinaryOp, Literal, Span};
///
/// // Create a binary expression: 2 + 3
/// let left = Box::new(Expr::new(
///     ExprKind::Literal(Literal::Integer(2)),
///     Span::new(0, 1)
/// ));
/// let right = Box::new(Expr::new(
///     ExprKind::Literal(Literal::Integer(3)),
///     Span::new(4, 5)
/// ));
/// let expr = Expr::new(
///     ExprKind::Binary { left, op: BinaryOp::Add, right },
///     Span::new(0, 5)
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expr {
    /// The specific type of expression.
    pub kind: ExprKind,
    /// Source location information for this expression.
    pub span: Span,
    /// Compiler attributes and metadata attached to this expression.
    pub attributes: Vec<Attribute>,
}
impl Expr {
    /// Creates a new expression with the given kind and span.
    ///
    /// This is the primary constructor for building AST nodes. The expression
    /// starts with no attributes; use `with_attributes` if attributes are needed.
    ///
    /// # Arguments
    ///
    /// * `kind` - The specific type of expression
    /// * `span` - The source location of this expression
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span};
    ///
    /// let expr = Expr::new(
    ///     ExprKind::Literal(Literal::Boolean(true)),
    ///     Span::new(0, 4)
    /// );
    /// ```
    #[must_use]
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self {
            kind,
            span,
            attributes: Vec::new(),
        }
    }
    /// Creates a new expression with attributes attached.
    ///
    /// Attributes provide metadata and compiler directives that modify
    /// how an expression is processed. Common uses include optimization
    /// hints, debugging information, and feature flags.
    ///
    /// # Arguments
    ///
    /// * `kind` - The specific type of expression
    /// * `span` - The source location of this expression
    /// * `attributes` - Compiler attributes to attach
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::frontend::ast::{Expr, ExprKind, Attribute, Literal, Span};
    ///
    /// let expr = Expr::with_attributes(
    ///     ExprKind::Literal(Literal::Integer(42)),
    ///     Span::new(0, 2),
    ///     vec![Attribute::inline()]
    /// );
    /// ```
    #[must_use]
    pub fn with_attributes(kind: ExprKind, span: Span, attributes: Vec<Attribute>) -> Self {
        Self {
            kind,
            span,
            attributes,
        }
    }
}
/// The specific type of expression represented by an AST node.
///
/// `ExprKind` is a comprehensive enumeration of all expression types supported
/// by the Ruchy language. This includes literals, operators, control flow,
/// declarations, and advanced features like actors and dataframes.
///
/// # Expression Categories
///
/// ## Literals and Identifiers
/// - `Literal`: Constant values (integers, strings, booleans, etc.)
/// - `Identifier`: Variable references
/// - `QualifiedName`: Module-qualified identifiers
///
/// ## Operations
/// - `Binary`: Binary operations (arithmetic, logical, comparison)
/// - `Unary`: Unary operations (negation, not)
/// - `Pipeline`: Functional pipeline operations
///
/// ## Control Flow
/// - `If`, `IfLet`: Conditional expressions
/// - `Match`: Pattern matching
/// - `For`, `While`, `Loop`: Iteration constructs
/// - `TryCatch`, `Throw`: Exception handling
///
/// ## Declarations
/// - `Let`, `LetPattern`: Variable bindings
/// - `Function`, `Lambda`: Function definitions
/// - `Struct`, `Enum`, `Trait`: Type definitions
///
/// ## Advanced Features
/// - `Actor`: Actor model for concurrency
/// - `DataFrame`: Tabular data operations
/// - `Async`, `Await`: Asynchronous programming
/// - `Command`: System command execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExprKind {
    /// A literal value (integer, string, boolean, etc.).
    Literal(Literal),
    /// A simple identifier reference.
    Identifier(String),
    /// A module-qualified identifier (e.g., `std::println`).
    QualifiedName {
        /// The module path.
        module: String,
        /// The identifier within the module.
        name: String,
    },
    /// An interpolated string with embedded expressions.
    StringInterpolation {
        /// The parts of the interpolated string.
        parts: Vec<StringPart>,
    },
    /// A binary operation (e.g., `a + b`, `x && y`).
    Binary {
        /// The left operand.
        left: Box<Expr>,
        /// The binary operator.
        op: BinaryOp,
        /// The right operand.
        right: Box<Expr>,
    },
    /// A unary operation (e.g., `-x`, `!flag`).
    Unary {
        /// The unary operator.
        op: UnaryOp,
        /// The operand expression.
        operand: Box<Expr>,
    },
    /// Throws an exception.
    Throw {
        /// The exception value to throw.
        expr: Box<Expr>,
    },
    /// Exception handling with pattern-based catch clauses.
    TryCatch {
        /// The expression to try.
        try_block: Box<Expr>,
        /// Pattern-based catch handlers.
        catch_clauses: Vec<CatchClause>,
        /// Optional finally block executed regardless of success/failure.
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
    ArrayInit {
        value: Box<Expr>,
        size: Box<Expr>,
    },
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
/// Literal values that can appear in the source code.
///
/// Literals represent compile-time constant values that are directly
/// embedded in the program. These form the base values from which
/// more complex expressions are constructed.
///
/// # Examples
///
/// ```ignore
/// use ruchy::frontend::ast::Literal;
///
/// let int = Literal::Integer(42);
/// let float = Literal::Float(3.14);
/// let string = Literal::String("hello".to_string());
/// let boolean = Literal::Bool(true);
/// let character = Literal::Char('a');
/// let unit = Literal::Unit;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    /// A signed 64-bit integer literal.
    Integer(i64),
    /// A 64-bit floating-point literal.
    Float(f64),
    /// A string literal.
    String(String),
    /// A boolean literal (`true` or `false`).
    Bool(bool),
    /// A character literal.
    Char(char),
    /// The unit value `()`.
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
/// Binary operators for two-operand expressions.
///
/// These operators cover arithmetic, comparison, logical, and bitwise operations.
/// Each operator has specific precedence and associativity rules defined in the parser.
///
/// # Examples
///
/// ```ignore
/// use ruchy::frontend::ast::BinaryOp;
///
/// // Arithmetic: a + b * c
/// let add = BinaryOp::Add;
/// let mul = BinaryOp::Multiply;
///
/// // Comparison: x >= y
/// let ge = BinaryOp::GreaterEqual;
///
/// // Logical: flag1 && flag2
/// let and = BinaryOp::And;
/// ```
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
    Gt, // Alias for Greater (for compatibility)
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
/// Unary operators for single-operand expressions.
///
/// These operators include logical negation, arithmetic negation,
/// bitwise complement, and reference operations.
///
/// # Examples
///
/// ```ignore
/// use ruchy::frontend::ast::UnaryOp;
///
/// // Logical negation: !flag
/// let not = UnaryOp::Not;
///
/// // Arithmetic negation: -value
/// let neg = UnaryOp::Negate;
///
/// // Bitwise complement: ~bits
/// let complement = UnaryOp::BitwiseNot;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
    Negate,
    BitwiseNot,
    Reference,
}
/// A function or method parameter.
///
/// Parameters support pattern matching for destructuring, type annotations
/// for type checking, default values for optional parameters, and mutability
/// modifiers.
///
/// # Examples
///
/// ```ignore
/// // Simple parameter: x: int
/// // Pattern parameter: (a, b): (int, int)
/// // Optional parameter: name: string = "default"
/// // Mutable parameter: mut buffer: Vec<u8>
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    /// The pattern for destructuring the parameter.
    pub pattern: Pattern,
    /// The type annotation for this parameter.
    pub ty: Type,
    /// Source location of this parameter.
    pub span: Span,
    /// Whether this parameter is mutable.
    pub is_mutable: bool,
    /// Optional default value for this parameter.
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
/// Type annotations in the AST.
///
/// Types provide static type information for type checking and code generation.
/// Ruchy supports a rich type system including generics, optionals, references,
/// and specialized types for dataframes and async operations.
///
/// # Examples
///
/// ```ignore
/// use ruchy::frontend::ast::{Type, TypeKind, Span};
///
/// // Simple named type: int
/// let int_type = Type {
///     kind: TypeKind::Named("int".to_string()),
///     span: Span::new(0, 3),
/// };
///
/// // Generic type: Vec<string>
/// let vec_type = Type {
///     kind: TypeKind::Generic {
///         base: "Vec".to_string(),
///         params: vec![string_type],
///     },
///     span: Span::new(0, 11),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Type {
    /// The specific type variant.
    pub kind: TypeKind,
    /// Source location of this type annotation.
    pub span: Span,
}
/// Specific type variants supported by Ruchy.
///
/// This enumeration covers all type forms from simple named types
/// to complex generic, functional, and structural types.
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
/// An arm in a match expression.
///
/// Match arms consist of a pattern to match against, an optional guard
/// condition for additional filtering, and a body expression to execute
/// when the pattern matches.
///
/// # Examples
///
/// ```ignore
/// match value {
///     Some(x) if x > 0 => x * 2,  // Pattern with guard
///     None => 0,                   // Simple pattern
///     _ => -1,                     // Wildcard pattern
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    /// The pattern to match against.
    pub pattern: Pattern,
    /// Optional guard condition that must be true for this arm to match.
    pub guard: Option<Box<Expr>>,
    /// The expression to execute when this arm matches.
    pub body: Box<Expr>,
    /// Source location of this match arm.
    pub span: Span,
}
/// Patterns for destructuring and matching values.
///
/// Patterns are used in match expressions, let bindings, function parameters,
/// and other contexts where values need to be destructured or tested against
/// a structure. Ruchy supports a rich pattern language including literals,
/// destructuring, ranges, and alternative patterns.
///
/// # Pattern Types
///
/// - **Wildcard**: `_` matches anything
/// - **Literal**: Matches exact values
/// - **Identifier**: Binds matched value to a name
/// - **Tuple/List**: Destructures sequences
/// - **Struct**: Destructures struct fields
/// - **Or**: Matches any of several patterns
/// - **Rest**: Captures remaining elements
///
/// # Examples
///
/// ```ignore
/// match value {
///     0 => "zero",                    // Literal pattern
///     1..=10 => "small",              // Range pattern
///     Some(x) => format!("value: {x}"), // Enum pattern
///     [first, ..rest] => "list",     // List pattern with rest
///     _ => "other",                   // Wildcard
/// }
/// ```
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
/// A field in a struct destructuring pattern.
///
/// Supports both explicit field patterns (`field: pattern`) and
/// shorthand syntax (`field` as shorthand for `field: field`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructPatternField {
    /// The field name to match.
    pub name: String,
    /// The pattern for this field's value (None for shorthand syntax).
    pub pattern: Option<Pattern>,
}
/// Definition of a custom error type.
///
/// Custom error types allow defining domain-specific error structures
/// with typed fields and optional inheritance from base error types.
///
/// # Examples
///
/// ```ignore
/// error NetworkError {
///     code: int,
///     message: string,
/// } extends IOError
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorTypeDef {
    /// The name of this error type.
    pub name: String,
    /// Fields contained in this error type.
    pub fields: Vec<StructField>,
    /// Optional parent error type to inherit from.
    pub extends: Option<String>,
}
/// Compiler attributes for annotating expressions.
///
/// Attributes provide metadata that influences compilation, optimization,
/// and runtime behavior. They use the `#[name(args)]` syntax similar to Rust.
///
/// # Common Attributes
///
/// - `#[inline]`: Hint for function inlining
/// - `#[test]`: Mark function as a test
/// - `#[property]`: Mark as a property accessor
/// - `#[deprecated("message")]`: Mark as deprecated
///
/// # Examples
///
/// ```ignore
/// #[inline]
/// #[property]
/// fn get_value() -> int {
///     self.value
/// }
/// ```
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
/// Operations on `DataFrame` values.
///
/// `DataFrames` are Ruchy's built-in tabular data structure, similar to
/// pandas `DataFrames` or SQL tables. These operations provide a fluent
/// API for data manipulation and analysis.
///
/// # Examples
///
/// ```ignore
/// df.filter(x => x.age > 18)
///   .select(["name", "email"])
///   .sort(["name"])
///   .limit(10)
/// ```
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
            Self::Gt => write!(f, ">"),
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
                TypeKind::Array { elem_type: _, size } => {
                    // Array types should have a valid size
                    assert!(size > 0);
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

    #[test]
    fn test_string_interpolation_parts() {
        // Test string interpolation with mixed parts
        let parts = vec![
            StringPart::Text("Hello, ".to_string()),
            StringPart::Expr(Box::new(Expr::new(
                ExprKind::Identifier("name".to_string()),
                Span::new(8, 12),
            ))),
            StringPart::Text("!".to_string()),
        ];

        let expr = Expr::new(
            ExprKind::StringInterpolation { parts },
            Span::new(0, 13),
        );

        if let ExprKind::StringInterpolation { parts } = expr.kind {
            assert_eq!(parts.len(), 3);
            match &parts[0] {
                StringPart::Text(s) => assert_eq!(s, "Hello, "),
                _ => panic!("Expected static part"),
            }
            match &parts[1] {
                StringPart::Expr(e) => {
                    if let ExprKind::Identifier(id) = &e.kind {
                        assert_eq!(id, "name");
                    }
                }
                _ => panic!("Expected dynamic part"),
            }
        }
    }

    #[test]
    fn test_async_function_creation() {
        // Test async function with await
        let func = Expr::new(
            ExprKind::Function {
                name: "fetch_data".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(Expr::new(
                    ExprKind::Await {
                        expr: Box::new(Expr::new(
                            ExprKind::Identifier("api_call".to_string()),
                            Span::new(0, 8),
                        )),
                    },
                    Span::new(0, 14),
                )),
                is_async: true,
                is_pub: false,
            },
            Span::new(0, 30),
        );

        if let ExprKind::Function { is_async, body, .. } = func.kind {
            assert!(is_async);
            if let ExprKind::Await { .. } = body.kind {
                // Correctly contains await expression
            } else {
                panic!("Expected await in async function");
            }
        }
    }

    #[test]
    fn test_try_catch_finally() {
        // Test try-catch-finally structure
        let try_catch = Expr::new(
            ExprKind::TryCatch {
                try_block: Box::new(Expr::new(
                    ExprKind::Identifier("risky_operation".to_string()),
                    Span::new(4, 19),
                )),
                catch_clauses: vec![CatchClause {
                    pattern: Pattern::Identifier("e".to_string()),
                    body: Box::new(Expr::new(
                        ExprKind::Identifier("handle_error".to_string()),
                        Span::new(25, 37),
                    )),
                }],
                finally_block: Some(Box::new(Expr::new(
                    ExprKind::Identifier("cleanup".to_string()),
                    Span::new(45, 52),
                ))),
            },
            Span::new(0, 52),
        );

        if let ExprKind::TryCatch {
            catch_clauses,
            finally_block,
            ..
        } = try_catch.kind
        {
            assert_eq!(catch_clauses.len(), 1);
            assert!(finally_block.is_some());
        }
    }

    #[test]
    fn test_result_option_types() {
        // Test Result and Option type constructors
        let ok_val = Expr::new(
            ExprKind::Ok {
                value: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(42)),
                    Span::new(3, 5),
                )),
            },
            Span::new(0, 6),
        );

        let err_val = Expr::new(
            ExprKind::Err {
                error: Box::new(Expr::new(
                    ExprKind::Literal(Literal::String("error".to_string())),
                    Span::new(4, 11),
                )),
            },
            Span::new(0, 12),
        );

        let some_val = Expr::new(
            ExprKind::Some {
                value: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(1)),
                    Span::new(5, 6),
                )),
            },
            Span::new(0, 7),
        );

        let none_val = Expr::new(ExprKind::None, Span::new(0, 4));

        assert!(matches!(ok_val.kind, ExprKind::Ok { .. }));
        assert!(matches!(err_val.kind, ExprKind::Err { .. }));
        assert!(matches!(some_val.kind, ExprKind::Some { .. }));
        assert!(matches!(none_val.kind, ExprKind::None));
    }

    // TODO: Pipeline operator not yet in BinaryOp enum
    // #[test]
    // fn test_pipeline_operator() {
    //     // Test pipeline operator expression
    //     let pipeline = Expr::new(
    //         ExprKind::Binary {
    //             left: Box::new(Expr::new(
    //                 ExprKind::Literal(Literal::Integer(5)),
    //                 Span::new(0, 1),
    //             )),
    //             op: BinaryOp::Pipeline,
    //             right: Box::new(Expr::new(
    //                 ExprKind::Identifier("double".to_string()),
    //                 Span::new(5, 11),
    //             )),
    //         },
    //         Span::new(0, 11),
    //     );
    //
    //     if let ExprKind::Binary { op, .. } = pipeline.kind {
    //         assert_eq!(op, BinaryOp::Pipeline);
    //     }
    // }

    #[test]
    fn test_destructuring_patterns() {
        // Test tuple and struct destructuring
        let tuple_pattern = Pattern::Tuple(vec![
            Pattern::Identifier("x".to_string()),
            Pattern::Identifier("y".to_string()),
            Pattern::Rest,
        ]);

        let struct_pattern = Pattern::Struct {
            name: "User".to_string(),
            fields: vec![
                StructPatternField {
                    name: "name".to_string(),
                    pattern: Some(Pattern::Identifier("n".to_string())),
                },
                StructPatternField {
                    name: "age".to_string(),
                    pattern: None,
                },
            ],
            has_rest: true,
        };

        if let Pattern::Tuple(elements) = tuple_pattern {
            assert_eq!(elements.len(), 3);
            assert!(matches!(elements[2], Pattern::Rest));
        }

        if let Pattern::Struct {
            fields, has_rest, ..
        } = struct_pattern
        {
            assert_eq!(fields.len(), 2);
            assert!(has_rest);
        }
    }

    #[test]
    fn test_qualified_names() {
        // Test module-qualified names
        let qualified = Expr::new(
            ExprKind::QualifiedName {
                module: "std".to_string(),
                name: "println".to_string(),
            },
            Span::new(0, 11),
        );

        if let ExprKind::QualifiedName { module, name } = qualified.kind {
            assert_eq!(module, "std");
            assert_eq!(name, "println");
        }
    }

    #[test]
    fn test_import_export_statements() {
        // Test import and export statements
        let import = Expr::new(
            ExprKind::Import {
                path: "std::collections".to_string(),
                items: vec![ImportItem::Aliased {
                    name: "HashMap".to_string(),
                    alias: "Map".to_string(),
                }],
            },
            Span::new(0, 30),
        );

        let export = Expr::new(
            ExprKind::Export {
                items: vec!["MyClass".to_string()],
            },
            Span::new(0, 25),
        );

        if let ExprKind::Import { path, items } = import.kind {
            assert_eq!(path, "std::collections");
            assert_eq!(items.len(), 1);
        }

        if let ExprKind::Export { items } = export.kind {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0], "MyClass");
        }
    }

    #[test]
    fn test_decorator_attributes() {
        // Test decorator/attribute attachment
        let decorated = Expr::with_attributes(
            ExprKind::Function {
                name: "test_func".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Unit),
                    Span::new(0, 0),
                )),
                is_async: false,
                is_pub: false,
            },
            Span::new(0, 20),
            vec![
                Attribute {
                    name: "test".to_string(),
                    args: vec![],
                    span: Span::new(0, 5),
                },
                Attribute {
                    name: "bench".to_string(),
                    args: vec![],
                    span: Span::new(0, 6),
                },
            ],
        );

        assert_eq!(decorated.attributes.len(), 2);
        assert_eq!(decorated.attributes[0].name, "test");
        assert_eq!(decorated.attributes[1].name, "bench");
    }

    // Test removed - CompClause type not defined

    #[test]
    fn test_dataframe_operations() {
        // Test DataFrame literal and operations
        let df = Expr::new(
            ExprKind::DataFrame {
                columns: vec![
                    DataFrameColumn {
                        name: "name".to_string(),
                        values: vec![
                            Expr::new(
                                ExprKind::Literal(Literal::String("Alice".to_string())),
                                Span::new(0, 7),
                            ),
                            Expr::new(
                                ExprKind::Literal(Literal::String("Bob".to_string())),
                                Span::new(8, 13),
                            ),
                        ],
                    },
                    DataFrameColumn {
                        name: "age".to_string(),
                        values: vec![
                            Expr::new(
                                ExprKind::Literal(Literal::Integer(25)),
                                Span::new(14, 16),
                            ),
                            Expr::new(
                                ExprKind::Literal(Literal::Integer(30)),
                                Span::new(17, 19),
                            ),
                        ],
                    },
                ],
            },
            Span::new(0, 50),
        );

        if let ExprKind::DataFrame { columns } = df.kind {
            assert_eq!(columns.len(), 2);
            assert_eq!(columns[0].name, "name");
            assert_eq!(columns[0].values.len(), 2);
            assert_eq!(columns[1].name, "age");
            assert_eq!(columns[1].values.len(), 2);
        }
    }

    #[test]
    fn test_type_cast_operations() {
        // Test type casting
        let cast = Expr::new(
            ExprKind::TypeCast {
                expr: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(42)),
                    Span::new(0, 2),
                )),
                target_type: "f64".to_string(),
            },
            Span::new(0, 10),
        );

        if let ExprKind::TypeCast { target_type, .. } = cast.kind {
            assert_eq!(target_type, "f64");
        }
    }

    #[test]
    fn test_binary_operators_complete() {
        // Test all binary operators
        let ops = vec![
            BinaryOp::Add,
            BinaryOp::Subtract,
            BinaryOp::Multiply,
            BinaryOp::Divide,
            BinaryOp::Modulo,
            BinaryOp::Power,
            BinaryOp::Equal,
            BinaryOp::NotEqual,
            BinaryOp::Less,
            BinaryOp::Greater,
            BinaryOp::LessEqual,
            BinaryOp::GreaterEqual,
            BinaryOp::And,
            BinaryOp::Or,
            // BinaryOp::Pipeline, // TODO: Not yet in enum
            BinaryOp::BitwiseAnd,
            BinaryOp::BitwiseOr,
            BinaryOp::BitwiseXor,
            BinaryOp::LeftShift,
            // BinaryOp::RightShift, // TODO: Not yet in enum
        ];

        for op in ops {
            let expr = Expr::new(
                ExprKind::Binary {
                    left: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(1)),
                        Span::new(0, 1),
                    )),
                    op,
                    right: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(2)),
                        Span::new(2, 3),
                    )),
                },
                Span::new(0, 3),
            );

            if let ExprKind::Binary { op: test_op, .. } = expr.kind {
                assert_eq!(test_op, op);
            }
        }
    }

    #[test]
    fn test_span_operations() {
        // Test span merging and creation
        let span1 = Span::new(0, 10);
        let span2 = Span::new(5, 15);
        let merged = span1.merge(span2);

        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 15);

        // Test with reverse order
        let merged2 = span2.merge(span1);
        assert_eq!(merged2.start, 0);
        assert_eq!(merged2.end, 15);
    }

    #[test]
    fn test_pattern_with_default() {
        // Test pattern with default value
        let pattern = Pattern::WithDefault {
            pattern: Box::new(Pattern::Identifier("count".to_string())),
            default: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(0)),
                Span::new(0, 1),
            )),
        };

        if let Pattern::WithDefault { pattern, default } = pattern {
            match *pattern {
                Pattern::Identifier(name) => assert_eq!(name, "count"),
                _ => panic!("Expected identifier pattern"),
            }
            match default.kind {
                ExprKind::Literal(Literal::Integer(val)) => assert_eq!(val, 0),
                _ => panic!("Expected integer literal"),
            }
        }
    }

    // Test removed - Generator and CompClause types not defined

    #[test]
    fn test_mutable_parameter() {
        // Test mutable parameter
        let param = Param {
            pattern: Pattern::Identifier("data".to_string()),
            ty: Type {
                kind: TypeKind::List(Box::new(Type {
                    kind: TypeKind::Named("i32".to_string()),
                    span: Span::new(0, 3),
                })),
                span: Span::new(0, 6),
            },
            span: Span::new(0, 10),
            is_mutable: true,
            default_value: None,
        };

        assert!(param.is_mutable);
        assert_eq!(param.name(), "data");
    }

    #[test]
    fn test_reference_types() {
        // Test reference and mutable reference types
        let ref_type = Type {
            kind: TypeKind::Reference {
                is_mut: false,
                inner: Box::new(Type {
                    kind: TypeKind::Named("String".to_string()),
                    span: Span::new(1, 7),
                }),
            },
            span: Span::new(0, 7),
        };

        let mut_ref_type = Type {
            kind: TypeKind::Reference {
                is_mut: true,
                inner: Box::new(Type {
                    kind: TypeKind::Named("Vec".to_string()),
                    span: Span::new(4, 7),
                }),
            },
            span: Span::new(0, 7),
        };

        if let TypeKind::Reference { is_mut, inner } = ref_type.kind {
            assert!(!is_mut);
            if let TypeKind::Named(name) = inner.kind {
                assert_eq!(name, "String");
            }
        }

        if let TypeKind::Reference { is_mut, .. } = mut_ref_type.kind {
            assert!(is_mut);
        }
    }
}
