use anyhow::{Result, bail};
use super::bytecode::{OpCode, Value, Instruction, BytecodeModule};

pub struct Compiler {
    module: BytecodeModule,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            module: BytecodeModule::new(),
        }
    }
    
    pub fn compile_expression(&mut self, expr: &str) -> Result<BytecodeModule> {
        // Simple expression compiler for testing
        // Real implementation will parse Ruchy AST
        
        if expr.contains('+') {
            let parts: Vec<&str> = expr.split('+').collect();
            if parts.len() == 2 {
                let a: i64 = parts[0].trim().parse()?;
                let b: i64 = parts[1].trim().parse()?;
                
                self.module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(a)));
                self.module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(b)));
                self.module.add_instruction(Instruction::new(OpCode::Add));
                self.module.add_instruction(Instruction::new(OpCode::Halt));
                
                return Ok(self.module.clone());
            }
        }
        
        bail!("Unsupported expression: {}", expr)
    }
    
    pub fn get_module(self) -> BytecodeModule {
        self.module
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::interpreter::VirtualMachine;
    
    #[test]
    fn test_compile_simple_addition() {
        let mut compiler = Compiler::new();
        let module = compiler.compile_expression("2 + 3").unwrap();
        
        assert_eq!(module.instructions.len(), 4);
        assert_eq!(module.instructions[0].opcode, OpCode::Push);
        assert_eq!(module.instructions[1].opcode, OpCode::Push);
        assert_eq!(module.instructions[2].opcode, OpCode::Add);
        assert_eq!(module.instructions[3].opcode, OpCode::Halt);
    }
    
    #[test]
    fn test_compile_and_execute() {
        let mut compiler = Compiler::new();
        let module = compiler.compile_expression("10 + 20").unwrap();
        
        let mut vm = VirtualMachine::new();
        let result = vm.execute(&module).unwrap();
        
        assert_eq!(result.value, Some(Value::Int(30)));
    }
}