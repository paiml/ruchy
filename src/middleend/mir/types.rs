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
#[derive(Debug, Clone, PartialEq)]
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
    /// Character literal
    Char(char),
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
    NullCoalesce,
    // Actor operations
    Send,
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
    /// Reference (borrow)
    Ref,
    /// Dereference
    Deref,
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_program_creation() {
        let program = Program {
            functions: HashMap::new(),
            entry: "main".to_string(),
        };
        assert_eq!(program.entry, "main");
        assert_eq!(program.functions.len(), 0);
    }
    #[test]
    fn test_block_id() {
        let id1 = BlockId(0);
        let id2 = BlockId(1);
        let id3 = BlockId(0);
        assert_eq!(id1, id3);
        assert_ne!(id1, id2);
        assert_eq!(format!("{id1:?}"), "BlockId(0)");
    }
    #[test]
    fn test_local_variable() {
        let local1 = Local(0);
        let local2 = Local(1);
        let local3 = Local(0);
        assert_eq!(local1, local3);
        assert_ne!(local1, local2);
        assert_eq!(format!("{local1:?}"), "Local(0)");
    }
    #[test]
    fn test_type_variants() {
        let types = vec![
            Type::Unit,
            Type::Bool,
            Type::I32,
            Type::I64,
            Type::F32,
            Type::F64,
            Type::String,
            Type::Array(Box::new(Type::I32), 10),
            Type::Vec(Box::new(Type::F64)),
            Type::Tuple(vec![Type::I32, Type::Bool]),
        ];
        for ty in types {
            assert!(!format!("{ty:?}").is_empty());
        }
    }
    #[test]
    fn test_type_equality() {
        assert_eq!(Type::I32, Type::I32);
        assert_ne!(Type::I32, Type::I64);
        assert_eq!(
            Type::Array(Box::new(Type::U8), 5),
            Type::Array(Box::new(Type::U8), 5)
        );
        assert_ne!(
            Type::Array(Box::new(Type::U8), 5),
            Type::Array(Box::new(Type::U8), 10)
        );
    }

    #[test]
    fn test_function_creation() {
        let func = Function {
            name: "test_func".to_string(),
            params: vec![Local(0), Local(1)],
            return_ty: Type::I32,
            locals: vec![],
            blocks: vec![],
            entry_block: BlockId(0),
        };
        assert_eq!(func.name, "test_func");
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.return_ty, Type::I32);
    }

    #[test]
    fn test_basic_block() {
        let block = BasicBlock {
            id: BlockId(0),
            statements: vec![],
            terminator: Terminator::Return(None),
        };
        assert_eq!(block.id, BlockId(0));
        assert!(block.statements.is_empty());
        assert!(matches!(block.terminator, Terminator::Return(None)));
    }

    #[test]
    fn test_local_decl() {
        let decl = LocalDecl {
            id: Local(5),
            ty: Type::String,
            mutable: true,
            name: Some("my_var".to_string()),
        };
        assert_eq!(decl.id, Local(5));
        assert_eq!(decl.ty, Type::String);
        assert!(decl.mutable);
        assert_eq!(decl.name, Some("my_var".to_string()));
    }

    #[test]
    fn test_operand_variants() {
        let operands = vec![
            Operand::Copy(Place::Local(Local(0))),
            Operand::Move(Place::Local(Local(1))),
            Operand::Constant(Constant::Int(42, Type::I64)),
        ];
        for op in operands {
            assert!(!format!("{op:?}").is_empty());
        }
    }

    #[test]
    fn test_place_projection() {
        let place = Place::Local(Local(0));
        assert!(matches!(place, Place::Local(_)));

        let field_place = Place::Field(Box::new(Place::Local(Local(0))), FieldIdx(1));
        assert!(matches!(field_place, Place::Field(_, _)));
    }

    #[test]
    fn test_constant_variants() {
        let constants = vec![
            Constant::Unit,
            Constant::Bool(true),
            Constant::Int(123, Type::I64),
            Constant::Float(3.15, Type::F64),
            Constant::String("hello".to_string()),
        ];
        for c in constants {
            assert!(!format!("{c:?}").is_empty());
        }
    }

    #[test]
    fn test_rvalue_variants() {
        let rvalues = vec![
            Rvalue::Use(Operand::Constant(Constant::Int(42, Type::I64))),
            Rvalue::BinaryOp(
                BinOp::Add,
                Operand::Constant(Constant::Int(1, Type::I64)),
                Operand::Constant(Constant::Int(2, Type::I64)),
            ),
            Rvalue::UnaryOp(UnOp::Neg, Operand::Constant(Constant::Int(5, Type::I64))),
            Rvalue::Ref(Mutability::Immutable, Place::Local(Local(0))),
        ];
        for rv in rvalues {
            assert!(!format!("{rv:?}").is_empty());
        }
    }

    #[test]
    fn test_binary_ops() {
        let ops = vec![
            BinOp::Add,
            BinOp::Sub,
            BinOp::Mul,
            BinOp::Div,
            BinOp::Eq,
            BinOp::Ne,
            BinOp::Lt,
            BinOp::Gt,
            BinOp::And,
            BinOp::Or,
            BinOp::BitAnd,
            BinOp::BitOr,
        ];
        for op in ops {
            assert!(!format!("{op:?}").is_empty());
        }
    }

    #[test]
    fn test_unary_ops() {
        let ops = vec![UnOp::Neg, UnOp::Not, UnOp::BitNot, UnOp::Ref];
        for op in ops {
            assert!(!format!("{op:?}").is_empty());
        }
    }

    #[test]
    fn test_aggregate_kind() {
        let kinds = vec![
            AggregateKind::Tuple,
            AggregateKind::Array(Type::I32),
            AggregateKind::Struct("MyStruct".to_string()),
        ];
        for kind in kinds {
            assert!(!format!("{kind:?}").is_empty());
        }
    }

    #[test]
    fn test_cast_kind() {
        assert_eq!(CastKind::Numeric, CastKind::Numeric);
        assert_ne!(CastKind::Numeric, CastKind::Pointer);
        assert_ne!(CastKind::Pointer, CastKind::Unsize);
    }

    #[test]
    fn test_terminator_variants() {
        let terminators = vec![
            Terminator::Goto(BlockId(1)),
            Terminator::If {
                condition: Operand::Constant(Constant::Bool(true)),
                then_block: BlockId(2),
                else_block: BlockId(3),
            },
            Terminator::Return(Some(Operand::Constant(Constant::Int(0, Type::I64)))),
            Terminator::Unreachable,
        ];
        for term in terminators {
            assert!(!format!("{term:?}").is_empty());
        }
    }

    #[test]
    fn test_mutability() {
        assert_eq!(Mutability::Immutable, Mutability::Immutable);
        assert_eq!(Mutability::Mutable, Mutability::Mutable);
        assert_ne!(Mutability::Immutable, Mutability::Mutable);
    }

    #[test]
    fn test_statement_assign() {
        let stmt = Statement::Assign(
            Place::Local(Local(0)),
            Rvalue::Use(Operand::Constant(Constant::Int(42, Type::I64))),
        );
        assert!(matches!(stmt, Statement::Assign(_, _)));
    }

    #[test]
    fn test_display_block_id() {
        let id = BlockId(42);
        assert_eq!(format!("{id}"), "bb42");
    }

    #[test]
    fn test_display_local() {
        let local = Local(7);
        assert_eq!(format!("{local}"), "__7");
    }

    #[test]
    fn test_display_types() {
        assert_eq!(format!("{}", Type::Unit), "()");
        assert_eq!(format!("{}", Type::Bool), "bool");
        assert_eq!(format!("{}", Type::I32), "i32");
        assert_eq!(format!("{}", Type::F64), "f64");
        assert_eq!(format!("{}", Type::String), "String");
        assert_eq!(format!("{}", Type::Vec(Box::new(Type::I32))), "Vec<i32>");
        assert_eq!(
            format!("{}", Type::Tuple(vec![Type::I32, Type::Bool])),
            "(i32, bool)"
        );
    }

    #[test]
    fn test_ref_type_display() {
        let immut_ref = Type::Ref(Box::new(Type::I32), Mutability::Immutable);
        assert_eq!(format!("{immut_ref}"), "&i32");

        let mut_ref = Type::Ref(Box::new(Type::String), Mutability::Mutable);
        assert_eq!(format!("{mut_ref}"), "&mut String");
    }

    #[test]
    fn test_array_type_display() {
        let arr = Type::Array(Box::new(Type::U8), 256);
        assert_eq!(format!("{arr}"), "[u8; 256]");
    }

    #[test]
    fn test_function_pointer_display() {
        let fn_ptr = Type::FnPtr(vec![Type::I32, Type::Bool], Box::new(Type::String));
        assert_eq!(format!("{fn_ptr}"), "fn(i32, bool) -> String");
    }

    #[test]
    fn test_user_type_display() {
        let user_type = Type::UserType("MyCustomType".to_string());
        assert_eq!(format!("{user_type}"), "MyCustomType");
    }

    #[test]
    fn test_switch_terminator() {
        let switch = Terminator::Switch {
            discriminant: Operand::Copy(Place::Local(Local(0))),
            targets: vec![
                (Constant::Int(0, Type::I64), BlockId(1)),
                (Constant::Int(1, Type::I64), BlockId(2)),
            ],
            default: Some(BlockId(3)),
        };
        if let Terminator::Switch {
            targets, default, ..
        } = switch
        {
            assert_eq!(targets.len(), 2);
            assert_eq!(default, Some(BlockId(3)));
        } else {
            panic!("Expected Switch terminator");
        }
    }

    #[test]
    fn test_call_terminator() {
        let call = Terminator::Call {
            func: Operand::Constant(Constant::String("my_func".to_string())),
            args: vec![
                Operand::Constant(Constant::Int(1, Type::I64)),
                Operand::Constant(Constant::Bool(true)),
            ],
            destination: Some((Place::Local(Local(0)), BlockId(1))),
        };
        if let Terminator::Call {
            args, destination, ..
        } = call
        {
            assert_eq!(args.len(), 2);
            assert!(destination.is_some());
        } else {
            panic!("Expected Call terminator");
        }
    }

    #[test]
    fn test_place_field() {
        let base = Place::Local(Local(0));
        let field = Place::Field(Box::new(base.clone()), FieldIdx(2));
        if let Place::Field(p, idx) = field {
            assert_eq!(*p, base);
            assert_eq!(idx, FieldIdx(2));
        } else {
            panic!("Expected Field place");
        }
    }

    #[test]
    fn test_place_index() {
        let base = Place::Local(Local(0));
        let index = Place::Index(Box::new(base.clone()), Box::new(Place::Local(Local(1))));
        if let Place::Index(p, _) = index {
            assert_eq!(*p, base);
        } else {
            panic!("Expected Index place");
        }
    }

    #[test]
    fn test_rvalue_aggregate() {
        let agg = Rvalue::Aggregate(
            AggregateKind::Tuple,
            vec![
                Operand::Constant(Constant::Int(1, Type::I64)),
                Operand::Constant(Constant::Bool(false)),
            ],
        );
        if let Rvalue::Aggregate(_, operands) = agg {
            assert_eq!(operands.len(), 2);
        } else {
            panic!("Expected Aggregate rvalue");
        }
    }

    #[test]
    fn test_rvalue_cast() {
        let cast = Rvalue::Cast(
            CastKind::Numeric,
            Operand::Constant(Constant::Int(42, Type::I64)),
            Type::F64,
        );
        if let Rvalue::Cast(kind, _, ty) = cast {
            assert_eq!(kind, CastKind::Numeric);
            assert_eq!(ty, Type::F64);
        } else {
            panic!("Expected Cast rvalue");
        }
    }
}
