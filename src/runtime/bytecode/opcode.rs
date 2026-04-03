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

    /// Valid opcode values bitmap (0x00-0x3B range, with gaps)
    const VALID_OPCODES: [bool; 64] = {
        let mut valid = [false; 64];
        let codes: &[u8] = &[
            0x00,0x01,0x02,0x03,0x04,0x05,0x06,0x07,0x08,0x09,0x0A,0x0B,0x0C,0x0D,0x0E,0x0F,
            0x10,0x11,0x12,0x13,0x14,0x15,0x16,0x17,0x18,0x19,0x1A,0x1B,0x1C,0x1D,0x1E,0x1F,
            0x20,0x21,0x22,0x23,0x24,0x25,0x26,0x27,0x28,0x29,0x2A,0x2B,0x2C,0x2D,
            0x30,0x31,0x32,0x33,0x34,0x35,0x36,0x37,0x38,0x39,0x3A,0x3B,
        ];
        let mut i = 0;
        while i < codes.len() { valid[codes[i] as usize] = true; i += 1; }
        valid
    };

    /// Try to convert u8 to opcode using validated transmute
    pub fn from_u8(value: u8) -> Option<Self> {
        if (value as usize) < Self::VALID_OPCODES.len() && Self::VALID_OPCODES[value as usize] {
            // SAFETY: value is validated against the exhaustive list of repr(u8) discriminants
            #[allow(unsafe_code)]
            Some(unsafe { std::mem::transmute::<u8, Self>(value) })
        } else {
            None
        }
    }

    /// Get human-readable name of opcode
    /// Static name table indexed by opcode discriminant
    const OPCODE_NAMES: [&'static str; 60] = [
        "Nop","Const","LoadLocal","StoreLocal","LoadGlobal","StoreGlobal",
        "LoadField","StoreField","LoadIndex","StoreIndex","LoadUpvalue","StoreUpvalue",
        "Move","Pop","Dup","Swap",
        "Add","Sub","Mul","Div","Mod","Neg","BitAnd","BitOr","BitXor","BitNot",
        "ShiftLeft","ShiftRight","NewObject","NewArray","NewClosure","GetType",
        "Equal","NotEqual","Greater","GreaterEqual","Less","LessEqual","Not","And","Or","InstanceOf",
        "InlineCache","Specialize","Deoptimize","NewTuple",
        "Reserved2E","Reserved2F",
        "Jump","JumpIfTrue","JumpIfFalse","Call","TailCall","Return",
        "Throw","EnterTry","ExitTry","For","MethodCall","Match",
    ];

    pub fn name(self) -> &'static str {
        let idx = self as u8 as usize;
        if idx < Self::OPCODE_NAMES.len() {
            Self::OPCODE_NAMES[idx]
        } else {
            "Unknown"
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

    #[test]
    fn test_opcode_clone() {
        let op = OpCode::Add;
        let cloned = op.clone();
        assert_eq!(op, cloned);
    }

    #[test]
    fn test_opcode_copy() {
        let op = OpCode::Sub;
        let copied = op;
        assert_eq!(op, copied);
    }

    #[test]
    fn test_opcode_eq() {
        assert_eq!(OpCode::Add, OpCode::Add);
        assert_ne!(OpCode::Add, OpCode::Sub);
    }

    #[test]
    fn test_opcode_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(OpCode::Add);
        set.insert(OpCode::Sub);
        assert!(set.contains(&OpCode::Add));
        assert!(set.contains(&OpCode::Sub));
        assert!(!set.contains(&OpCode::Mul));
    }

    #[test]
    fn test_opcode_debug() {
        let debug_str = format!("{:?}", OpCode::Add);
        assert!(debug_str.contains("Add"));
    }

    #[test]
    fn test_stack_operations_range() {
        for i in 0x00..=0x0F {
            assert!(
                OpCode::from_u8(i).is_some(),
                "Stack op 0x{:02X} should exist",
                i
            );
        }
    }

    #[test]
    fn test_arithmetic_operations_range() {
        for i in 0x10..=0x1F {
            assert!(
                OpCode::from_u8(i).is_some(),
                "Arithmetic op 0x{:02X} should exist",
                i
            );
        }
    }

    #[test]
    fn test_control_flow_operations_range() {
        for i in 0x30..=0x3B {
            assert!(
                OpCode::from_u8(i).is_some(),
                "Control flow op 0x{:02X} should exist",
                i
            );
        }
    }

    #[test]
    fn test_opcode_nop_is_zero() {
        assert_eq!(OpCode::Nop.to_u8(), 0);
    }

    #[test]
    fn test_opcode_category_stack() {
        let stack_ops = [
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
        ];
        for op in stack_ops {
            assert!(op.to_u8() <= 0x0F, "{:?} should be in stack range", op);
        }
    }

    #[test]
    fn test_opcode_category_arithmetic() {
        let arith_ops = [
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
        ];
        for op in arith_ops {
            let val = op.to_u8();
            assert!(
                val >= 0x10 && val <= 0x1F,
                "{:?} should be in arithmetic range",
                op
            );
        }
    }

    #[test]
    fn test_opcode_category_comparison() {
        let cmp_ops = [
            OpCode::Equal,
            OpCode::NotEqual,
            OpCode::Greater,
            OpCode::GreaterEqual,
            OpCode::Less,
            OpCode::LessEqual,
            OpCode::Not,
            OpCode::And,
            OpCode::Or,
        ];
        for op in cmp_ops {
            let val = op.to_u8();
            assert!(
                val >= 0x20 && val <= 0x2F,
                "{:?} should be in comparison range",
                op
            );
        }
    }

    #[test]
    fn test_opcode_category_control() {
        let ctrl_ops = [
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
        for op in ctrl_ops {
            let val = op.to_u8();
            assert!(
                val >= 0x30 && val <= 0x3F,
                "{:?} should be in control range",
                op
            );
        }
    }

    #[test]
    fn test_all_opcodes_count() {
        assert_eq!(ALL_OPCODES.len(), 58);
    }

    #[test]
    fn test_opcode_unique_values() {
        use std::collections::HashSet;
        let mut values = HashSet::new();
        for op in ALL_OPCODES {
            let val = op.to_u8();
            assert!(values.insert(val), "Duplicate opcode value 0x{:02X}", val);
        }
    }
}
