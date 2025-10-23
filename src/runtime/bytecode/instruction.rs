//! Bytecode instruction encoding and decoding
//!
//! OPT-001: Bytecode VM Foundation
//!
//! 32-bit fixed-width instruction format for efficient fetch and decode.
//! Register-based architecture with 32 general-purpose registers.
//!
//! Instruction Format (32-bit fixed width):
//! ```text
//! [ OpCode (6 bits) | Format (2 bits) | Operands (24 bits) ]
//! ```
//!
//! Register Formats:
//! - 0 (ABC):  [ A: 8 bits | B: 8 bits | C: 8 bits ] - Three register operands
//! - 1 (ABx):  [ A: 8 bits | Bx: 16 bits ]           - One register + 16-bit immediate
//! - 2 (AsBx): [ A: 8 bits | sBx: 16 bits ]          - One register + 16-bit signed immediate
//! - 3 (Ax):   [ Ax: 24 bits ]                       - 24-bit immediate or offset
//!
//! Reference: ../ruchyruchy/OPTIMIZATION_REPORT_FOR_RUCHY.md
//! Academic: Brunthaler (2010) - Inline Caching Meets Quickening

use super::opcode::OpCode;

/// Instruction format variant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InstructionFormat {
    /// ABC format: 3 register operands (A, B, C)
    ABC = 0,
    /// ABx format: 1 register + 16-bit unsigned immediate
    ABx = 1,
    /// AsBx format: 1 register + 16-bit signed immediate
    AsBx = 2,
    /// Ax format: 24-bit immediate (jumps, large constants)
    Ax = 3,
}

impl InstructionFormat {
    /// Convert format to u8
    #[inline]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Convert u8 to format
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::ABC),
            1 => Some(Self::ABx),
            2 => Some(Self::AsBx),
            3 => Some(Self::Ax),
            _ => None,
        }
    }
}

/// Bytecode instruction (32-bit fixed width)
///
/// # Memory Layout
///
/// The instruction is packed into a single u32:
/// ```text
/// Bit layout: [31..26: op] [25..24: fmt] [23..16: a] [15..8: b] [7..0: c]
/// ```
///
/// # Examples
///
/// ```
/// use ruchy::runtime::bytecode::{Instruction, OpCode};
///
/// // Create ADD instruction: R0 = R1 + R2
/// let add = Instruction::abc(OpCode::Add, 0, 1, 2);
///
/// // Create CONST instruction: R0 = constants[42]
/// let load_const = Instruction::abx(OpCode::Const, 0, 42);
///
/// // Create JUMP instruction: pc += 100
/// let jump = Instruction::ax(OpCode::Jump, 100);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction(u32);

impl Instruction {
    /// Create instruction with ABC format (3 register operands)
    ///
    /// Used for operations like: R[a] = R[b] op R[c]
    #[inline]
    pub const fn abc(op: OpCode, a: u8, b: u8, c: u8) -> Self {
        let op_bits = (op.to_u8() as u32) << 26;
        let fmt_bits = (InstructionFormat::ABC.to_u8() as u32) << 24;
        let a_bits = (a as u32) << 16;
        let b_bits = (b as u32) << 8;
        let c_bits = c as u32;

        Self(op_bits | fmt_bits | a_bits | b_bits | c_bits)
    }

    /// Create instruction with ABx format (register + 16-bit unsigned immediate)
    ///
    /// Used for operations like: R[a] = constants[bx]
    #[inline]
    pub const fn abx(op: OpCode, a: u8, bx: u16) -> Self {
        let op_bits = (op.to_u8() as u32) << 26;
        let fmt_bits = (InstructionFormat::ABx.to_u8() as u32) << 24;
        let a_bits = (a as u32) << 16;
        let bx_bits = bx as u32;

        Self(op_bits | fmt_bits | a_bits | bx_bits)
    }

    /// Create instruction with AsBx format (register + 16-bit signed immediate)
    ///
    /// Used for conditional jumps with signed offsets.
    #[inline]
    pub const fn asbx(op: OpCode, a: u8, sbx: i16) -> Self {
        let op_bits = (op.to_u8() as u32) << 26;
        let fmt_bits = (InstructionFormat::AsBx.to_u8() as u32) << 24;
        let a_bits = (a as u32) << 16;
        let sbx_bits = (sbx as u16) as u32;  // Preserve sign bits

        Self(op_bits | fmt_bits | a_bits | sbx_bits)
    }

    /// Create instruction with Ax format (24-bit immediate)
    ///
    /// Used for unconditional jumps and large immediate values.
    #[inline]
    pub const fn ax(op: OpCode, ax: u32) -> Self {
        let op_bits = (op.to_u8() as u32) << 26;
        let fmt_bits = (InstructionFormat::Ax.to_u8() as u32) << 24;
        let ax_bits = ax & 0x00FF_FFFF;  // Mask to 24 bits

        Self(op_bits | fmt_bits | ax_bits)
    }

    /// Get opcode from instruction
    #[inline]
    pub const fn opcode(self) -> u8 {
        (self.0 >> 26) as u8
    }

    /// Get instruction format
    #[inline]
    pub const fn format(self) -> u8 {
        ((self.0 >> 24) & 0b11) as u8
    }

    /// Get A operand (8 bits)
    #[inline]
    pub const fn get_a(self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    /// Get B operand (8 bits)
    #[inline]
    pub const fn get_b(self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    /// Get C operand (8 bits)
    #[inline]
    pub const fn get_c(self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    /// Get Bx operand (16-bit unsigned)
    #[inline]
    pub const fn get_bx(self) -> u16 {
        (self.0 & 0xFFFF) as u16
    }

    /// Get sBx operand (16-bit signed)
    #[inline]
    pub const fn get_sbx(self) -> i16 {
        (self.0 & 0xFFFF) as i16
    }

    /// Get Ax operand (24-bit)
    #[inline]
    pub const fn get_ax(self) -> u32 {
        self.0 & 0x00FF_FFFF
    }

    /// Get raw u32 value
    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Create instruction from raw u32
    #[inline]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_abc_format() {
        // Test ABC format: ADD R0, R1, R2
        let instr = Instruction::abc(OpCode::Add, 0, 1, 2);

        assert_eq!(instr.opcode(), OpCode::Add.to_u8());
        assert_eq!(instr.format(), InstructionFormat::ABC.to_u8());
        assert_eq!(instr.get_a(), 0);
        assert_eq!(instr.get_b(), 1);
        assert_eq!(instr.get_c(), 2);
    }

    #[test]
    fn test_instruction_abx_format() {
        // Test ABx format: CONST R0, 42
        let instr = Instruction::abx(OpCode::Const, 0, 42);

        assert_eq!(instr.opcode(), OpCode::Const.to_u8());
        assert_eq!(instr.format(), InstructionFormat::ABx.to_u8());
        assert_eq!(instr.get_a(), 0);
        assert_eq!(instr.get_bx(), 42);
    }

    #[test]
    fn test_instruction_asbx_format() {
        // Test AsBx format: JUMP_IF_FALSE R0, -10
        let instr = Instruction::asbx(OpCode::JumpIfFalse, 0, -10);

        assert_eq!(instr.opcode(), OpCode::JumpIfFalse.to_u8());
        assert_eq!(instr.format(), InstructionFormat::AsBx.to_u8());
        assert_eq!(instr.get_a(), 0);
        assert_eq!(instr.get_sbx(), -10);
    }

    #[test]
    fn test_instruction_ax_format() {
        // Test Ax format: JUMP 1000
        let instr = Instruction::ax(OpCode::Jump, 1000);

        assert_eq!(instr.opcode(), OpCode::Jump.to_u8());
        assert_eq!(instr.format(), InstructionFormat::Ax.to_u8());
        assert_eq!(instr.get_ax(), 1000);
    }

    #[test]
    fn test_instruction_roundtrip() {
        // Test that we can encode and decode without losing information
        let original = Instruction::abc(OpCode::Mul, 5, 10, 15);
        let raw = original.raw();
        let recovered = Instruction::from_raw(raw);

        assert_eq!(original, recovered);
    }

    #[test]
    fn test_large_bx_value() {
        // Test maximum 16-bit unsigned value
        let instr = Instruction::abx(OpCode::LoadGlobal, 255, 65535);
        assert_eq!(instr.get_a(), 255);
        assert_eq!(instr.get_bx(), 65535);
    }

    #[test]
    fn test_negative_sbx_value() {
        // Test negative signed offset
        let instr = Instruction::asbx(OpCode::JumpIfTrue, 10, -1000);
        assert_eq!(instr.get_a(), 10);
        assert_eq!(instr.get_sbx(), -1000);
    }

    #[test]
    fn test_ax_24bit_max() {
        // Test maximum 24-bit value (16,777,215)
        let max_24bit = 0x00FF_FFFF;
        let instr = Instruction::ax(OpCode::Jump, max_24bit);
        assert_eq!(instr.get_ax(), max_24bit);
    }

    #[test]
    fn test_instruction_format_conversion() {
        assert_eq!(InstructionFormat::from_u8(0), Some(InstructionFormat::ABC));
        assert_eq!(InstructionFormat::from_u8(1), Some(InstructionFormat::ABx));
        assert_eq!(InstructionFormat::from_u8(2), Some(InstructionFormat::AsBx));
        assert_eq!(InstructionFormat::from_u8(3), Some(InstructionFormat::Ax));
        assert_eq!(InstructionFormat::from_u8(4), None);
    }
}
