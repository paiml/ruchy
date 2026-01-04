//! Bytecode opcode definitions
//!
//! OPT-001: Bytecode VM Foundation
//!
//! Compact bytecode opcodes (6 bits, supports up to 64 operations)
//! Based on ruchyruchy optimization research - register-based VM architecture
//!
//! Reference: ../`ruchyruchy/OPTIMIZATION_REPORT_FOR_RUCHY.md`
//! Academic: Würthinger et al. (2017) - One VM to Rule Them All

/// Bytecode operation codes
///
/// 6-bit encoding allows 64 unique operations.
/// Organized by category for readability and dispatch optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OpCode {
    // Stack Operations (0x00-0x0F)
    /// No operation - used for padding and alignment
    Nop = 0x00,
    /// Push constant onto stack
    Const = 0x01,
    /// Load local variable onto stack
    LoadLocal = 0x02,
    /// Store stack top to local variable
    StoreLocal = 0x03,
    /// Load global variable onto stack
    LoadGlobal = 0x04,
    /// Store stack top to global variable
    StoreGlobal = 0x05,
    /// Load object field onto stack
    LoadField = 0x06,
    /// Store stack top to object field
    StoreField = 0x07,
    /// Load array element onto stack
    LoadIndex = 0x08,
    /// Store stack top to array element
    StoreIndex = 0x09,
    /// Load upvalue (closed-over variable)
    LoadUpvalue = 0x0A,
    /// Store to upvalue
    StoreUpvalue = 0x0B,
    /// Move value from one register to another
    Move = 0x0C,
    /// Pop top of stack
    Pop = 0x0D,
    /// Duplicate top of stack
    Dup = 0x0E,
    /// Swap top two stack items
    Swap = 0x0F,

    // Arithmetic Operations (0x10-0x1F)
    /// Add top two values
    Add = 0x10,
    /// Subtract top two values
    Sub = 0x11,
    /// Multiply top two values
    Mul = 0x12,
    /// Divide top two values
    Div = 0x13,
    /// Modulo operation
    Mod = 0x14,
    /// Negate top value
    Neg = 0x15,
    /// Bitwise AND
    BitAnd = 0x16,
    /// Bitwise OR
    BitOr = 0x17,
    /// Bitwise XOR
    BitXor = 0x18,
    /// Bitwise NOT
    BitNot = 0x19,
    /// Bit shift left
    ShiftLeft = 0x1A,
    /// Bit shift right
    ShiftRight = 0x1B,
    /// Create new object
    NewObject = 0x1C,
    /// Create new array
    NewArray = 0x1D,
    /// Create new closure
    NewClosure = 0x1E,
    /// Get type of object
    GetType = 0x1F,

    // Logical Operations (0x20-0x2F)
    /// Equality comparison
    Equal = 0x20,
    /// Inequality comparison
    NotEqual = 0x21,
    /// Greater than
    Greater = 0x22,
    /// Greater than or equal
    GreaterEqual = 0x23,
    /// Less than
    Less = 0x24,
    /// Less than or equal
    LessEqual = 0x25,
    /// Logical NOT
    Not = 0x26,
    /// Logical AND (short-circuit)
    And = 0x27,
    /// Logical OR (short-circuit)
    Or = 0x28,
    /// Check if object is instance of type
    InstanceOf = 0x29,
    /// Inline cache for property access
    InlineCache = 0x2A,
    /// Type specialization
    Specialize = 0x2B,
    /// Deoptimize to baseline code
    Deoptimize = 0x2C,
    /// Create new tuple
    NewTuple = 0x2D,

    // Control Flow (0x30-0x3F)
    /// Unconditional jump
    Jump = 0x30,
    /// Jump if top of stack is true
    JumpIfTrue = 0x31,
    /// Jump if top of stack is false
    JumpIfFalse = 0x32,
    /// Call function
    Call = 0x33,
    /// Call function and return its result (tail call optimization)
    TailCall = 0x34,
    /// Return from function
    Return = 0x35,
    /// Throw exception
    Throw = 0x36,
    /// Enter try block
    EnterTry = 0x37,
    /// Exit try block
    ExitTry = 0x38,
    /// For-loop iteration (hybrid execution - delegates to interpreter)
    For = 0x39,
    /// Method call (hybrid execution - delegates to interpreter)
    MethodCall = 0x3A,
    /// Match expression (hybrid execution - delegates to interpreter)
    Match = 0x3B,
}

impl OpCode {
    /// Convert opcode to u8 value
    #[inline]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Try to convert u8 to opcode
    ///
    /// Returns None if the u8 value doesn't correspond to a valid opcode.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            // Stack Operations
            0x00 => Some(Self::Nop),
            0x01 => Some(Self::Const),
            0x02 => Some(Self::LoadLocal),
            0x03 => Some(Self::StoreLocal),
            0x04 => Some(Self::LoadGlobal),
            0x05 => Some(Self::StoreGlobal),
            0x06 => Some(Self::LoadField),
            0x07 => Some(Self::StoreField),
            0x08 => Some(Self::LoadIndex),
            0x09 => Some(Self::StoreIndex),
            0x0A => Some(Self::LoadUpvalue),
            0x0B => Some(Self::StoreUpvalue),
            0x0C => Some(Self::Move),
            0x0D => Some(Self::Pop),
            0x0E => Some(Self::Dup),
            0x0F => Some(Self::Swap),

            // Arithmetic Operations
            0x10 => Some(Self::Add),
            0x11 => Some(Self::Sub),
            0x12 => Some(Self::Mul),
            0x13 => Some(Self::Div),
            0x14 => Some(Self::Mod),
            0x15 => Some(Self::Neg),
            0x16 => Some(Self::BitAnd),
            0x17 => Some(Self::BitOr),
            0x18 => Some(Self::BitXor),
            0x19 => Some(Self::BitNot),
            0x1A => Some(Self::ShiftLeft),
            0x1B => Some(Self::ShiftRight),
            0x1C => Some(Self::NewObject),
            0x1D => Some(Self::NewArray),
            0x1E => Some(Self::NewClosure),
            0x1F => Some(Self::GetType),

            // Logical Operations
            0x20 => Some(Self::Equal),
            0x21 => Some(Self::NotEqual),
            0x22 => Some(Self::Greater),
            0x23 => Some(Self::GreaterEqual),
            0x24 => Some(Self::Less),
            0x25 => Some(Self::LessEqual),
            0x26 => Some(Self::Not),
            0x27 => Some(Self::And),
            0x28 => Some(Self::Or),
            0x29 => Some(Self::InstanceOf),
            0x2A => Some(Self::InlineCache),
            0x2B => Some(Self::Specialize),
            0x2C => Some(Self::Deoptimize),
            0x2D => Some(Self::NewTuple),

            // Control Flow
            0x30 => Some(Self::Jump),
            0x31 => Some(Self::JumpIfTrue),
            0x32 => Some(Self::JumpIfFalse),
            0x33 => Some(Self::Call),
            0x34 => Some(Self::TailCall),
            0x35 => Some(Self::Return),
            0x36 => Some(Self::Throw),
            0x37 => Some(Self::EnterTry),
            0x38 => Some(Self::ExitTry),
            0x39 => Some(Self::For),
            0x3A => Some(Self::MethodCall),
            0x3B => Some(Self::Match),

            _ => None,
        }
    }

    /// Get human-readable name of opcode
    pub const fn name(self) -> &'static str {
        match self {
            Self::Nop => "Nop",
            Self::Const => "Const",
            Self::LoadLocal => "LoadLocal",
            Self::StoreLocal => "StoreLocal",
            Self::LoadGlobal => "LoadGlobal",
            Self::StoreGlobal => "StoreGlobal",
            Self::LoadField => "LoadField",
            Self::StoreField => "StoreField",
            Self::LoadIndex => "LoadIndex",
            Self::StoreIndex => "StoreIndex",
            Self::LoadUpvalue => "LoadUpvalue",
            Self::StoreUpvalue => "StoreUpvalue",
            Self::Move => "Move",
            Self::Pop => "Pop",
            Self::Dup => "Dup",
            Self::Swap => "Swap",
            Self::Add => "Add",
            Self::Sub => "Sub",
            Self::Mul => "Mul",
            Self::Div => "Div",
            Self::Mod => "Mod",
            Self::Neg => "Neg",
            Self::BitAnd => "BitAnd",
            Self::BitOr => "BitOr",
            Self::BitXor => "BitXor",
            Self::BitNot => "BitNot",
            Self::ShiftLeft => "ShiftLeft",
            Self::ShiftRight => "ShiftRight",
            Self::Equal => "Equal",
            Self::NotEqual => "NotEqual",
            Self::Greater => "Greater",
            Self::GreaterEqual => "GreaterEqual",
            Self::Less => "Less",
            Self::LessEqual => "LessEqual",
            Self::Not => "Not",
            Self::And => "And",
            Self::Or => "Or",
            Self::Jump => "Jump",
            Self::JumpIfTrue => "JumpIfTrue",
            Self::JumpIfFalse => "JumpIfFalse",
            Self::Call => "Call",
            Self::TailCall => "TailCall",
            Self::Return => "Return",
            Self::Throw => "Throw",
            Self::EnterTry => "EnterTry",
            Self::ExitTry => "ExitTry",
            Self::For => "For",
            Self::MethodCall => "MethodCall",
            Self::Match => "Match",
            Self::NewObject => "NewObject",
            Self::NewArray => "NewArray",
            Self::NewClosure => "NewClosure",
            Self::GetType => "GetType",
            Self::InstanceOf => "InstanceOf",
            Self::InlineCache => "InlineCache",
            Self::Specialize => "Specialize",
            Self::Deoptimize => "Deoptimize",
            Self::NewTuple => "NewTuple",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // All opcodes for comprehensive testing
    const ALL_OPCODES: &[OpCode] = &[
        // Stack Operations
        OpCode::Nop,
        OpCode::Const,
        OpCode::LoadLocal,
        OpCode::StoreLocal,
        OpCode::LoadGlobal,
        OpCode::StoreGlobal,
        OpCode::LoadField,
        OpCode::StoreField,
        OpCode::LoadIndex,
        OpCode::StoreIndex,
        OpCode::LoadUpvalue,
        OpCode::StoreUpvalue,
        OpCode::Move,
        OpCode::Pop,
        OpCode::Dup,
        OpCode::Swap,
        // Arithmetic Operations
        OpCode::Add,
        OpCode::Sub,
        OpCode::Mul,
        OpCode::Div,
        OpCode::Mod,
        OpCode::Neg,
        OpCode::BitAnd,
        OpCode::BitOr,
        OpCode::BitXor,
        OpCode::BitNot,
        OpCode::ShiftLeft,
        OpCode::ShiftRight,
        OpCode::NewObject,
        OpCode::NewArray,
        OpCode::NewClosure,
        OpCode::GetType,
        // Logical Operations
        OpCode::Equal,
        OpCode::NotEqual,
        OpCode::Greater,
        OpCode::GreaterEqual,
        OpCode::Less,
        OpCode::LessEqual,
        OpCode::Not,
        OpCode::And,
        OpCode::Or,
        OpCode::InstanceOf,
        OpCode::InlineCache,
        OpCode::Specialize,
        OpCode::Deoptimize,
        OpCode::NewTuple,
        // Control Flow
        OpCode::Jump,
        OpCode::JumpIfTrue,
        OpCode::JumpIfFalse,
        OpCode::Call,
        OpCode::TailCall,
        OpCode::Return,
        OpCode::Throw,
        OpCode::EnterTry,
        OpCode::ExitTry,
        OpCode::For,
        OpCode::MethodCall,
        OpCode::Match,
    ];

    #[test]
    fn test_opcode_to_u8_roundtrip_all() {
        for opcode in ALL_OPCODES {
            let u8_val = opcode.to_u8();
            let recovered = OpCode::from_u8(u8_val).expect("Failed to recover opcode");
            assert_eq!(*opcode, recovered, "Opcode roundtrip failed for {opcode:?}");
        }
    }

    #[test]
    fn test_invalid_opcode() {
        assert!(OpCode::from_u8(0xFF).is_none());
        assert!(OpCode::from_u8(0x60).is_none());
        assert!(OpCode::from_u8(0xAA).is_none());
        assert!(OpCode::from_u8(0x3C).is_none()); // Just past Match
        assert!(OpCode::from_u8(0x2E).is_none()); // Gap in logical section
    }

    #[test]
    fn test_opcode_names_all() {
        for opcode in ALL_OPCODES {
            let name = opcode.name();
            assert!(!name.is_empty(), "Opcode {opcode:?} has empty name");
        }
    }

    #[test]
    fn test_opcode_names_specific() {
        assert_eq!(OpCode::Nop.name(), "Nop");
        assert_eq!(OpCode::Const.name(), "Const");
        assert_eq!(OpCode::LoadLocal.name(), "LoadLocal");
        assert_eq!(OpCode::StoreLocal.name(), "StoreLocal");
        assert_eq!(OpCode::LoadGlobal.name(), "LoadGlobal");
        assert_eq!(OpCode::StoreGlobal.name(), "StoreGlobal");
        assert_eq!(OpCode::LoadField.name(), "LoadField");
        assert_eq!(OpCode::StoreField.name(), "StoreField");
        assert_eq!(OpCode::LoadIndex.name(), "LoadIndex");
        assert_eq!(OpCode::StoreIndex.name(), "StoreIndex");
        assert_eq!(OpCode::LoadUpvalue.name(), "LoadUpvalue");
        assert_eq!(OpCode::StoreUpvalue.name(), "StoreUpvalue");
        assert_eq!(OpCode::Move.name(), "Move");
        assert_eq!(OpCode::Pop.name(), "Pop");
        assert_eq!(OpCode::Dup.name(), "Dup");
        assert_eq!(OpCode::Swap.name(), "Swap");
        assert_eq!(OpCode::Add.name(), "Add");
        assert_eq!(OpCode::Sub.name(), "Sub");
        assert_eq!(OpCode::Mul.name(), "Mul");
        assert_eq!(OpCode::Div.name(), "Div");
        assert_eq!(OpCode::Mod.name(), "Mod");
        assert_eq!(OpCode::Neg.name(), "Neg");
        assert_eq!(OpCode::BitAnd.name(), "BitAnd");
        assert_eq!(OpCode::BitOr.name(), "BitOr");
        assert_eq!(OpCode::BitXor.name(), "BitXor");
        assert_eq!(OpCode::BitNot.name(), "BitNot");
        assert_eq!(OpCode::ShiftLeft.name(), "ShiftLeft");
        assert_eq!(OpCode::ShiftRight.name(), "ShiftRight");
        assert_eq!(OpCode::NewObject.name(), "NewObject");
        assert_eq!(OpCode::NewArray.name(), "NewArray");
        assert_eq!(OpCode::NewClosure.name(), "NewClosure");
        assert_eq!(OpCode::GetType.name(), "GetType");
        assert_eq!(OpCode::Equal.name(), "Equal");
        assert_eq!(OpCode::NotEqual.name(), "NotEqual");
        assert_eq!(OpCode::Greater.name(), "Greater");
        assert_eq!(OpCode::GreaterEqual.name(), "GreaterEqual");
        assert_eq!(OpCode::Less.name(), "Less");
        assert_eq!(OpCode::LessEqual.name(), "LessEqual");
        assert_eq!(OpCode::Not.name(), "Not");
        assert_eq!(OpCode::And.name(), "And");
        assert_eq!(OpCode::Or.name(), "Or");
        assert_eq!(OpCode::InstanceOf.name(), "InstanceOf");
        assert_eq!(OpCode::InlineCache.name(), "InlineCache");
        assert_eq!(OpCode::Specialize.name(), "Specialize");
        assert_eq!(OpCode::Deoptimize.name(), "Deoptimize");
        assert_eq!(OpCode::NewTuple.name(), "NewTuple");
        assert_eq!(OpCode::Jump.name(), "Jump");
        assert_eq!(OpCode::JumpIfTrue.name(), "JumpIfTrue");
        assert_eq!(OpCode::JumpIfFalse.name(), "JumpIfFalse");
        assert_eq!(OpCode::Call.name(), "Call");
        assert_eq!(OpCode::TailCall.name(), "TailCall");
        assert_eq!(OpCode::Return.name(), "Return");
        assert_eq!(OpCode::Throw.name(), "Throw");
        assert_eq!(OpCode::EnterTry.name(), "EnterTry");
        assert_eq!(OpCode::ExitTry.name(), "ExitTry");
        assert_eq!(OpCode::For.name(), "For");
        assert_eq!(OpCode::MethodCall.name(), "MethodCall");
        assert_eq!(OpCode::Match.name(), "Match");
    }

    #[test]
    fn test_opcode_u8_values() {
        // Stack operations 0x00-0x0F
        assert_eq!(OpCode::Nop.to_u8(), 0x00);
        assert_eq!(OpCode::Swap.to_u8(), 0x0F);
        // Arithmetic 0x10-0x1F
        assert_eq!(OpCode::Add.to_u8(), 0x10);
        assert_eq!(OpCode::GetType.to_u8(), 0x1F);
        // Logical 0x20-0x2D
        assert_eq!(OpCode::Equal.to_u8(), 0x20);
        assert_eq!(OpCode::NewTuple.to_u8(), 0x2D);
        // Control 0x30-0x3B
        assert_eq!(OpCode::Jump.to_u8(), 0x30);
        assert_eq!(OpCode::Match.to_u8(), 0x3B);
    }

    #[test]
    fn test_from_u8_boundary_values() {
        // Test boundary values for each section
        assert!(OpCode::from_u8(0x00).is_some()); // First stack op
        assert!(OpCode::from_u8(0x0F).is_some()); // Last stack op
        assert!(OpCode::from_u8(0x10).is_some()); // First arithmetic
        assert!(OpCode::from_u8(0x1F).is_some()); // Last arithmetic
        assert!(OpCode::from_u8(0x20).is_some()); // First logical
        assert!(OpCode::from_u8(0x2D).is_some()); // Last logical (NewTuple)
        assert!(OpCode::from_u8(0x2E).is_none()); // Gap
        assert!(OpCode::from_u8(0x2F).is_none()); // Gap
        assert!(OpCode::from_u8(0x30).is_some()); // First control
        assert!(OpCode::from_u8(0x3B).is_some()); // Last control (Match)
        assert!(OpCode::from_u8(0x3C).is_none()); // Past end
    }
}
