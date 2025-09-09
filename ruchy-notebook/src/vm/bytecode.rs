use serde::{Deserialize, Serialize};
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum OpCode {
    // Stack operations
    Push = 0x01,
    Pop = 0x02,
    Dup = 0x03,
    Swap = 0x04,
    
    // Arithmetic
    Add = 0x10,
    Sub = 0x11,
    Mul = 0x12,
    Div = 0x13,
    Mod = 0x14,
    Neg = 0x15,
    
    // Comparison
    Eq = 0x20,
    Ne = 0x21,
    Lt = 0x22,
    Le = 0x23,
    Gt = 0x24,
    Ge = 0x25,
    
    // Boolean
    And = 0x30,
    Or = 0x31,
    Not = 0x32,
    
    // Control flow
    Jump = 0x40,
    JumpIf = 0x41,
    JumpIfNot = 0x42,
    Call = 0x43,
    Return = 0x44,
    
    // Variables
    LoadLocal = 0x50,
    StoreLocal = 0x51,
    LoadGlobal = 0x52,
    StoreGlobal = 0x53,
    
    // Data structures
    BuildList = 0x60,
    BuildMap = 0x61,
    Index = 0x62,
    SetIndex = 0x63,
    
    // Special
    Print = 0x70,
    Halt = 0xFF,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Map(Vec<(String, Value)>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub opcode: OpCode,
    pub operand: Option<Value>,
}

impl Instruction {
    pub fn new(opcode: OpCode) -> Self {
        Self { opcode, operand: None }
    }
    
    pub fn with_operand(opcode: OpCode, operand: Value) -> Self {
        Self { opcode, operand: Some(operand) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytecodeModule {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub entry_point: usize,
}

impl BytecodeModule {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            entry_point: 0,
        }
    }
    
    pub fn add_instruction(&mut self, instruction: Instruction) -> usize {
        let index = self.instructions.len();
        self.instructions.push(instruction);
        index
    }
    
    pub fn add_constant(&mut self, value: Value) -> usize {
        let index = self.constants.len();
        self.constants.push(value);
        index
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_opcode_encoding() {
        assert_eq!(OpCode::Push as u8, 0x01);
        assert_eq!(OpCode::Add as u8, 0x10);
        assert_eq!(OpCode::Halt as u8, 0xFF);
    }
    
    #[test]
    fn test_instruction_creation() {
        let inst = Instruction::new(OpCode::Add);
        assert_eq!(inst.opcode, OpCode::Add);
        assert_eq!(inst.operand, None);
        
        let inst_with_val = Instruction::with_operand(OpCode::Push, Value::Int(42));
        assert_eq!(inst_with_val.opcode, OpCode::Push);
        assert_eq!(inst_with_val.operand, Some(Value::Int(42)));
    }
    
    #[test]
    fn test_bytecode_module() {
        let mut module = BytecodeModule::new();
        
        let const_idx = module.add_constant(Value::Int(100));
        assert_eq!(const_idx, 0);
        
        let inst_idx = module.add_instruction(Instruction::new(OpCode::Push));
        assert_eq!(inst_idx, 0);
        
        assert_eq!(module.instructions.len(), 1);
        assert_eq!(module.constants.len(), 1);
    }
}