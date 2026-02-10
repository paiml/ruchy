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
//!     ExprKind::Literal(Literal::Integer(42, None)),
//!     Span::new(0, 2)
//! );
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Comment information for AST nodes.
///
/// Comments are preserved during parsing to enable accurate code formatting
/// that maintains documentation and developer intent. Each comment tracks its
/// text content, position, and association with AST nodes.
///
/// # Examples
///
/// ```ignore
/// use ruchy::frontend::ast::{Comment, CommentKind, Span};
///
/// // Create a line comment
/// let comment = Comment::new(
///     CommentKind::Line("This is a comment".to_string()),
///     Span::new(0, 20)
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    /// The type and content of this comment.
    pub kind: CommentKind,
    /// Source location information for this comment.
    pub span: Span,
}

impl Comment {
    /// Creates a new comment with the given kind and span.
    #[must_use]
    pub fn new(kind: CommentKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// The type of comment.
///
/// Ruchy supports three types of comments:
/// - Line comments starting with `//`
/// - Doc comments starting with `///` for documentation
/// - Block comments enclosed in `/* ... */`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommentKind {
    /// A single-line comment: `// comment text`
    Line(String),
    /// A documentation comment: `/// doc comment text`
    Doc(String),
    /// A block comment: `/* comment text */`
    Block(String),
}

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

/// A single clause in a comprehension (for and optional if).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComprehensionClause {
    /// The variable or pattern to bind
    pub variable: String,
    /// The iterable to iterate over
    pub iterable: Box<Expr>,
    /// Optional filter condition (if clause)
    pub condition: Option<Box<Expr>>,
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
///     ExprKind::Literal(Literal::Integer(2, None)),
///     Span::new(0, 1)
/// ));
/// let right = Box::new(Expr::new(
///     ExprKind::Literal(Literal::Integer(3, None)),
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
    /// Comments that appear before this expression.
    ///
    /// Leading comments are associated with the expression they precede.
    /// These typically include documentation comments and explanatory notes.
    pub leading_comments: Vec<Comment>,
    /// Optional comment that appears at the end of the same line as this expression.
    ///
    /// Trailing comments are inline comments that provide context for the
    /// specific line of code they follow.
    pub trailing_comment: Option<Comment>,
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
            leading_comments: Vec::new(),
            trailing_comment: None,
        }
    }

    /// Creates a new expression with comments attached.
    ///
    /// This constructor is used during parsing to associate comments with
    /// their corresponding AST nodes, enabling accurate code formatting that
    /// preserves documentation.
    ///
    /// # Arguments
    ///
    /// * `kind` - The specific type of expression
    /// * `span` - The source location of this expression
    /// * `leading_comments` - Comments that appear before this expression
    /// * `trailing_comment` - Optional comment at the end of the line
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::frontend::ast::{Expr, ExprKind, Comment, CommentKind, Span};
    ///
    /// let comment = Comment::new(
    ///     CommentKind::Line("Important value".to_string()),
    ///     Span::new(0, 18)
    /// );
    ///
    /// let expr = Expr::with_comments(
    ///     ExprKind::Literal(Literal::Integer(42, None)),
    ///     Span::new(20, 22),
    ///     vec![],
    ///     Some(comment)
    /// );
    /// ```
    #[must_use]
    pub fn with_comments(
        kind: ExprKind,
        span: Span,
        leading_comments: Vec<Comment>,
        trailing_comment: Option<Comment>,
    ) -> Self {
        Self {
            kind,
            span,
            attributes: Vec::new(),
            leading_comments,
            trailing_comment,
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
    ///     ExprKind::Literal(Literal::Integer(42, None)),
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
            leading_comments: Vec::new(),
            trailing_comment: None,
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
    /// Ternary conditional expression (condition ? `true_expr` : `false_expr`)
    Ternary {
        condition: Box<Expr>,
        true_expr: Box<Expr>,
        false_expr: Box<Expr>,
    },
    Try {
        expr: Box<Expr>,
    },
    Await {
        expr: Box<Expr>,
    },
    Spawn {
        actor: Box<Expr>,
    },
    AsyncBlock {
        body: Box<Expr>,
    },
    /// Lazy evaluation - defers computation until value is accessed.
    Lazy {
        /// The expression to evaluate lazily.
        expr: Box<Expr>,
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
        else_block: Option<Box<Expr>>, // For let-else: `let x = val else { diverging }`
    },
    LetPattern {
        pattern: Pattern,
        type_annotation: Option<Type>,
        value: Box<Expr>,
        body: Box<Expr>,
        is_mutable: bool,
        else_block: Option<Box<Expr>>, // For let-else: `let Some(x) = val else { diverging }`
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
    AsyncLambda {
        params: Vec<String>,
        body: Box<Expr>,
    },
    Struct {
        name: String,
        type_params: Vec<String>,
        fields: Vec<StructField>,
        methods: Vec<ClassMethod>, // Methods defined inside struct body
        derives: Vec<String>,      // #[derive(Debug, Clone, ...)]
        is_pub: bool,
    },
    TupleStruct {
        name: String,
        type_params: Vec<String>,
        fields: Vec<Type>, // Just types, no names
        derives: Vec<String>,
        is_pub: bool,
    },
    Class {
        name: String,
        type_params: Vec<String>,
        superclass: Option<String>, // inheritance
        traits: Vec<String>,        // + Trait1 + Trait2
        fields: Vec<StructField>,
        constructors: Vec<Constructor>, // new() methods
        methods: Vec<ClassMethod>,
        constants: Vec<ClassConstant>,  // const NAME: TYPE = VALUE
        properties: Vec<ClassProperty>, // property NAME: TYPE { get => ..., set(v) => ... }
        derives: Vec<String>,           // #[derive(Debug, Clone, ...)]
        decorators: Vec<Decorator>,     // @Serializable, @Table("users"), etc.
        is_pub: bool,
        is_sealed: bool,   // sealed class (no external subclassing)
        is_abstract: bool, // abstract class (cannot be instantiated)
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
        base: Option<Box<Expr>>, // For ..expr update syntax
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
        associated_types: Vec<String>, // type Item, type Output, etc.
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
    /// SPEC-001-I: Effect declaration with operation signatures
    Effect {
        name: String,
        operations: Vec<EffectOperation>,
    },
    /// SPEC-001-J: Effect handler expression
    Handle {
        expr: Box<Expr>,
        handlers: Vec<EffectHandler>,
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
    Set(Vec<Expr>),
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
        clauses: Vec<ComprehensionClause>,
    },
    SetComprehension {
        element: Box<Expr>,
        clauses: Vec<ComprehensionClause>,
    },
    DictComprehension {
        key: Box<Expr>,
        value: Box<Expr>,
        clauses: Vec<ComprehensionClause>,
    },
    DataFrame {
        columns: Vec<DataFrameColumn>,
    },
    DataFrameOperation {
        source: Box<Expr>,
        operation: DataFrameOp,
    },
    For {
        label: Option<String>,
        var: String,              // Keep for backward compatibility
        pattern: Option<Pattern>, // New: Support destructuring patterns
        iter: Box<Expr>,
        body: Box<Expr>,
    },
    While {
        label: Option<String>,
        condition: Box<Expr>,
        body: Box<Expr>,
    },
    WhileLet {
        label: Option<String>,
        pattern: Pattern,
        expr: Box<Expr>,
        body: Box<Expr>,
    },
    Loop {
        label: Option<String>,
        body: Box<Expr>,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
    Module {
        name: String,
        body: Box<Expr>,
    },
    /// ISSUE-106: External module declaration (mod name;)
    /// Represents a module that should be loaded from an external file
    ModuleDeclaration {
        name: String,
    },
    Break {
        label: Option<String>,
        value: Option<Box<Expr>>,
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
    /// Import statement for modules
    Import {
        module: String,
        items: Option<Vec<String>>,
    },
    /// Import all items with an alias (import * as name)
    ImportAll {
        module: String,
        alias: String,
    },
    /// Import default export
    ImportDefault {
        module: String,
        name: String,
    },
    /// Export a declaration
    Export {
        expr: Box<Expr>,
        is_default: bool,
    },
    /// Export a list of identifiers
    ExportList {
        names: Vec<String>,
    },
    /// Re-export from another module
    ReExport {
        items: Vec<String>,
        module: String,
    },
    /// Export default declaration
    ExportDefault {
        expr: Box<Expr>,
    },
    /// Type alias declaration (type Name = Type)
    TypeAlias {
        name: String,
        target_type: Type,
    },
    /// Macro invocation (e.g., `println!("hello")`)
    MacroInvocation {
        name: String,
        args: Vec<Expr>,
    },
    /// Vector repeat pattern: `vec![value; count]`
    /// Issue #155: Separate from `MacroInvocation` to generate correct Rust syntax
    VecRepeat {
        value: Box<Expr>,
        count: Box<Expr>,
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
/// let int = Literal::Integer(42, None);
/// let float = Literal::Float(3.15);
/// let string = Literal::String("hello".to_string());
/// let boolean = Literal::Bool(true);
/// let character = Literal::Char('a');
/// let unit = Literal::Unit;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    /// A signed 64-bit integer literal with optional type suffix (e.g., i32, i64, u32).
    Integer(i64, Option<String>),
    /// A 64-bit floating-point literal.
    Float(f64),
    /// A string literal.
    String(String),
    /// A boolean literal (`true` or `false`).
    Bool(bool),
    /// A character literal.
    Char(char),
    /// A byte literal (0-255).
    Byte(u8),
    /// The unit value `()`.
    Unit,
    /// A null value.
    Null,
    /// An atom (interned identifier).
    Atom(String),
}
impl Literal {
    /// Convert a REPL Value to a Literal (for synthetic expressions)
    pub fn from_value(value: &crate::runtime::interpreter::Value) -> Self {
        use crate::runtime::interpreter::Value;
        match value {
            Value::Integer(i) => Literal::Integer(*i, None),
            Value::Float(f) => Literal::Float(*f),
            Value::String(s) => Literal::String(s.to_string()),
            Value::Bool(b) => Literal::Bool(*b),
            Value::Nil => Literal::Unit,
            Value::Atom(s) => Literal::Atom(s.clone()),
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
    // Containment
    In, // Membership test: element in collection (Python-style)
    // Logical
    And,
    Or,
    NullCoalesce,
    // Bitwise
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
    // Actor operations
    Send, // Actor message passing: actor ! message
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
    MutableReference, // PARSER-085: Added for &mut support (GitHub Issue #71)
    Deref,
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
pub enum Visibility {
    Private,
    Public,
    PubCrate,
    PubSuper,
    Protected, // For future inheritance support
}

impl Visibility {
    pub fn is_public(&self) -> bool {
        matches!(
            self,
            Visibility::Public | Visibility::PubCrate | Visibility::PubSuper
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub visibility: Visibility,
    pub is_mut: bool,                // mut field modifier
    pub default_value: Option<Expr>, // Default value for class fields
    pub decorators: Vec<Decorator>,  // @PrimaryKey, @Column("name"), etc.
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub kind: EnumVariantKind,
    pub discriminant: Option<i64>, // Explicit discriminant value for TypeScript compatibility
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnumVariantKind {
    Unit,                     // Quit
    Tuple(Vec<Type>),         // Write(String)
    Struct(Vec<StructField>), // Move { x: i32, y: i32 }
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

/// SPEC-001-I: Effect operation signature
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectOperation {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
}

/// SPEC-001-J: Effect handler clause
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectHandler {
    pub operation: String,
    pub params: Vec<Pattern>,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Box<Expr>,
    pub is_pub: bool,
    pub is_static: bool,     // static method (no self)
    pub is_override: bool,   // override keyword for explicit overriding
    pub is_final: bool,      // final method (cannot be overridden)
    pub is_abstract: bool,   // abstract method (no implementation)
    pub is_async: bool,      // async method (returns Future)
    pub self_type: SelfType, // &self, &mut self, or self (move)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassConstant {
    pub name: String,
    pub ty: Type,
    pub value: Expr,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassProperty {
    pub name: String,
    pub ty: Type,
    pub getter: Option<Box<Expr>>,
    pub setter: Option<PropertySetter>,
    pub is_pub: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertySetter {
    pub param_name: String,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SelfType {
    None,        // static method
    Owned,       // self (move)
    Borrowed,    // &self
    MutBorrowed, // &mut self
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constructor {
    pub name: Option<String>, // None for primary constructor, Some(name) for named constructors
    pub params: Vec<Param>,
    pub return_type: Option<Type>, // Optional return type for named constructors (e.g., Result<Self>)
    pub body: Box<Expr>,
    pub is_pub: bool,
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
    Generic {
        base: String,
        params: Vec<Type>,
    },
    Optional(Box<Type>),
    List(Box<Type>),
    Array {
        elem_type: Box<Type>,
        size: usize,
    },
    Tuple(Vec<Type>),
    Function {
        params: Vec<Type>,
        ret: Box<Type>,
    },
    DataFrame {
        columns: Vec<(String, Type)>,
    },
    Series {
        dtype: Box<Type>,
    },
    Reference {
        is_mut: bool,
        lifetime: Option<String>,
        inner: Box<Type>,
    },
    /// SPEC-001-H: Refined types - Type with constraint predicate
    /// Example: `x: i32 where x > 0`
    /// The constraint is a boolean expression that must hold
    Refined {
        base: Box<Type>,
        constraint: Box<Expr>,
    },
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
    TupleVariant {
        path: Vec<String>,      // e.g., ["Message", "Text"]
        patterns: Vec<Pattern>, // Arguments like (n) or (a, b)
    }, // For enum tuple variants like Message::Text(n)
    Range {
        start: Box<Pattern>,
        end: Box<Pattern>,
        inclusive: bool,
    },
    Or(Vec<Pattern>),
    Rest,              // For ... patterns
    RestNamed(String), // For ..name patterns
    AtBinding {
        name: String,
        pattern: Box<Pattern>,
    }, // For @ bindings like name @ pattern
    WithDefault {
        pattern: Box<Pattern>,
        default: Box<Expr>,
    }, // For patterns with default values like a = 10
    Mut(Box<Pattern>), // For mutable bindings in destructuring like (mut x, mut y)
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
                    fields
                        .first()
                        .map_or_else(|| "_struct".to_string(), |f| f.name.clone())
                } else {
                    name.clone()
                }
            }
            Pattern::TupleVariant { path, patterns } => {
                // For enum tuple variants, get first pattern's name or variant path
                patterns
                    .first()
                    .map_or_else(|| path.join("::"), Pattern::primary_name)
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
            Pattern::AtBinding { name, .. } => name.clone(),
            Pattern::WithDefault { pattern, .. } => pattern.primary_name(),
            Pattern::Mut(inner) => inner.primary_name(),
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

/// Represents a decorator (@name or @name(...))
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decorator {
    pub name: String,
    pub args: Vec<String>,
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
            Self::RightShift => write!(f, ">>"),
            Self::Gt => write!(f, ">"),
            Self::Send => write!(f, "!"),
            Self::In => write!(f, "in"),
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
            Self::MutableReference => write!(f, "&mut "), // PARSER-085: Issue #71
            Self::Deref => write!(f, "*"),
        }
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
#[path = "ast_tests.rs"]
mod tests;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
#[path = "ast_tests_part2.rs"]
mod tests_part2;
