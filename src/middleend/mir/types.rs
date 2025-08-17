//! MIR type definitions

use std::collections::HashMap;
use std::fmt;

/// A MIR program consists of functions
#[derive(Debug, Clone)]
pub struct Program {
    /// Global functions in the program
    pub functions: HashMap<String, Function>,
    /// Entry point function name
    pub entry: String,
}

/// A function in MIR representation
#[derive(Debug, Clone)]
pub struct Function {
    /// Function name
    pub name: String,
    /// Parameters (as local variables)
    pub params: Vec<Local>,
    /// Return type
    pub return_ty: Type,
    /// Local variables (including parameters)
    pub locals: Vec<LocalDecl>,
    /// Basic blocks making up the function body
    pub blocks: Vec<BasicBlock>,
    /// Entry block index
    pub entry_block: BlockId,
}

/// A basic block is a sequence of statements with a single entry and exit
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Block identifier
    pub id: BlockId,
    /// Statements in this block (no control flow)
    pub statements: Vec<Statement>,
    /// Terminator - how control leaves this block
    pub terminator: Terminator,
}

/// Block identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

/// Local variable identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Local(pub usize);

/// Local variable declaration
#[derive(Debug, Clone)]
pub struct LocalDecl {
    /// Variable identifier
    pub id: Local,
    /// Variable type
    pub ty: Type,
    /// Is this mutable?
    pub mutable: bool,
    /// Optional name for debugging
    pub name: Option<String>,
}

/// MIR types (simplified from AST types)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Unit type
    Unit,
    /// Boolean
    Bool,
    /// Signed integers
    I8,
    I16,
    I32,
    I64,
    I128,
    /// Unsigned integers
    U8,
    U16,
    U32,
    U64,
    U128,
    /// Floating point
    F32,
    F64,
    /// String
    String,
    /// Reference (borrowed)
    Ref(Box<Type>, Mutability),
    /// Array with known size
    Array(Box<Type>, usize),
    /// Dynamic vector
    Vec(Box<Type>),
    /// Tuple
    Tuple(Vec<Type>),
    /// Function pointer
    FnPtr(Vec<Type>, Box<Type>),
    /// User-defined type
    UserType(String),
}

/// Mutability of references
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

/// A statement that doesn't affect control flow
#[derive(Debug, Clone)]
pub enum Statement {
    /// Assign an rvalue to a place
    Assign(Place, Rvalue),
    /// Mark a local as live (for storage)
    StorageLive(Local),
    /// Mark a local as dead (storage can be reclaimed)
    StorageDead(Local),
    /// No operation
    Nop,
}

/// A place where a value can be stored
#[derive(Debug, Clone)]
pub enum Place {
    /// Local variable
    Local(Local),
    /// Field projection (e.g., struct.field)
    Field(Box<Place>, FieldIdx),
    /// Array/slice index
    Index(Box<Place>, Box<Place>),
    /// Dereference
    Deref(Box<Place>),
}

/// Field index in a struct/tuple
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldIdx(pub usize);

/// Right-hand side of an assignment
#[derive(Debug, Clone)]
pub enum Rvalue {
    /// Use a value from a place
    Use(Operand),
    /// Binary operation
    BinaryOp(BinOp, Operand, Operand),
    /// Unary operation
    UnaryOp(UnOp, Operand),
    /// Create a reference
    Ref(Mutability, Place),
    /// Create an aggregate (struct, tuple, array)
    Aggregate(AggregateKind, Vec<Operand>),
    /// Function call
    Call(Operand, Vec<Operand>),
    /// Cast between types
    Cast(CastKind, Operand, Type),
}

/// An operand (value that can be used)
#[derive(Debug, Clone)]
pub enum Operand {
    /// Copy value from a place
    Copy(Place),
    /// Move value from a place
    Move(Place),
    /// Constant value
    Constant(Constant),
}

/// Constant values
#[derive(Debug, Clone)]
pub enum Constant {
    /// Unit value
    Unit,
    /// Boolean
    Bool(bool),
    /// Integer
    Int(i128, Type),
    /// Unsigned integer
    Uint(u128, Type),
    /// Float
    Float(f64, Type),
    /// String literal
    String(String),
}

/// Binary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical (short-circuiting is handled by control flow)
    And,
    Or,
}

/// Unary operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    /// Negation (arithmetic)
    Neg,
    /// Logical not
    Not,
    /// Bitwise not
    BitNot,
}

/// Aggregate kinds
#[derive(Debug, Clone)]
pub enum AggregateKind {
    /// Tuple
    Tuple,
    /// Array
    Array(Type),
    /// Struct
    Struct(String),
}

/// Cast kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastKind {
    /// Numeric cast (int to int, float to float, etc.)
    Numeric,
    /// Pointer to pointer
    Pointer,
    /// Unsizing (e.g., array to slice)
    Unsize,
}

/// How control flow leaves a basic block
#[derive(Debug, Clone)]
pub enum Terminator {
    /// Unconditional jump
    Goto(BlockId),
    /// Conditional branch
    If {
        condition: Operand,
        then_block: BlockId,
        else_block: BlockId,
    },
    /// Switch/match on a value
    Switch {
        discriminant: Operand,
        targets: Vec<(Constant, BlockId)>,
        default: Option<BlockId>,
    },
    /// Return from function
    Return(Option<Operand>),
    /// Call a function and continue
    Call {
        func: Operand,
        args: Vec<Operand>,
        destination: Option<(Place, BlockId)>,
    },
    /// Unreachable code (for exhaustiveness)
    Unreachable,
}

// Display implementations for debugging
impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

impl fmt::Display for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "__{}", self.0)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Unit => write!(f, "()"),
            Type::Bool => write!(f, "bool"),
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::I128 => write!(f, "i128"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::U128 => write!(f, "u128"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::String => write!(f, "String"),
            Type::Ref(ty, Mutability::Immutable) => write!(f, "&{ty}"),
            Type::Ref(ty, Mutability::Mutable) => write!(f, "&mut {ty}"),
            Type::Array(ty, size) => write!(f, "[{ty}; {size}]"),
            Type::Vec(ty) => write!(f, "Vec<{ty}>"),
            Type::Tuple(tys) => {
                write!(f, "(")?;
                for (i, ty) in tys.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{ty}")?;
                }
                write!(f, ")")
            }
            Type::FnPtr(params, ret) => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{param}")?;
                }
                write!(f, ") -> {ret}")
            }
            Type::UserType(name) => write!(f, "{name}"),
        }
    }
}
