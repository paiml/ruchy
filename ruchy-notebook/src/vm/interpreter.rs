use anyhow::{Result, bail};
use super::bytecode::{OpCode, Value, Instruction, BytecodeModule};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub value: Option<Value>,
    pub output: Vec<String>,
}

pub struct VirtualMachine {
    stack: Vec<Value>,
    call_stack: Vec<usize>,
    locals: HashMap<usize, Value>,
    globals: HashMap<String, Value>,
    output: Vec<String>,
    pc: usize,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            call_stack: Vec::with_capacity(32),
            locals: HashMap::new(),
            globals: HashMap::new(),
            output: Vec::new(),
            pc: 0,
        }
    }
    
    pub fn execute(&mut self, module: &BytecodeModule) -> Result<ExecutionResult> {
        self.pc = module.entry_point;
        
        while self.pc < module.instructions.len() {
            let instruction = &module.instructions[self.pc];
            
            match instruction.opcode {
                OpCode::Push => self.execute_push(instruction)?,
                OpCode::Pop => self.execute_pop()?,
                OpCode::Dup => self.execute_dup()?,
                OpCode::Swap => self.execute_swap()?,
                
                OpCode::Add => self.execute_binary_op(|a, b| a + b)?,
                OpCode::Sub => self.execute_binary_op(|a, b| a - b)?,
                OpCode::Mul => self.execute_binary_op(|a, b| a * b)?,
                OpCode::Div => self.execute_div()?,
                OpCode::Mod => self.execute_mod()?,
                OpCode::Neg => self.execute_neg()?,
                
                OpCode::Eq => self.execute_comparison(|a, b| a == b)?,
                OpCode::Ne => self.execute_comparison(|a, b| a != b)?,
                OpCode::Lt => self.execute_comparison(|a, b| a < b)?,
                OpCode::Le => self.execute_comparison(|a, b| a <= b)?,
                OpCode::Gt => self.execute_comparison(|a, b| a > b)?,
                OpCode::Ge => self.execute_comparison(|a, b| a >= b)?,
                
                OpCode::And => self.execute_and()?,
                OpCode::Or => self.execute_or()?,
                OpCode::Not => self.execute_not()?,
                
                OpCode::Jump => self.execute_jump(instruction)?,
                OpCode::JumpIf => self.execute_jump_if(instruction)?,
                OpCode::JumpIfNot => self.execute_jump_if_not(instruction)?,
                
                OpCode::LoadLocal => self.execute_load_local(instruction)?,
                OpCode::StoreLocal => self.execute_store_local(instruction)?,
                OpCode::LoadGlobal => self.execute_load_global(instruction)?,
                OpCode::StoreGlobal => self.execute_store_global(instruction)?,
                
                OpCode::Print => self.execute_print()?,
                OpCode::Halt => break,
                
                _ => bail!("Unimplemented opcode: {:?}", instruction.opcode),
            }
            
            self.pc += 1;
        }
        
        Ok(ExecutionResult {
            value: self.stack.last().cloned(),
            output: self.output.clone(),
        })
    }
    
    fn execute_push(&mut self, inst: &Instruction) -> Result<()> {
        let value = inst.operand.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Push requires an operand"))?;
        self.stack.push(value.clone());
        Ok(())
    }
    
    fn execute_pop(&mut self) -> Result<()> {
        self.stack.pop()
            .ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        Ok(())
    }
    
    fn execute_dup(&mut self) -> Result<()> {
        let value = self.stack.last()
            .ok_or_else(|| anyhow::anyhow!("Stack underflow"))?
            .clone();
        self.stack.push(value);
        Ok(())
    }
    
    fn execute_swap(&mut self) -> Result<()> {
        let len = self.stack.len();
        if len < 2 {
            bail!("Stack underflow: need 2 values to swap");
        }
        self.stack.swap(len - 1, len - 2);
        Ok(())
    }
    
    fn execute_binary_op(&mut self, op: fn(i64, i64) -> i64) -> Result<()> {
        let b = self.pop_int()?;
        let a = self.pop_int()?;
        self.stack.push(Value::Int(op(a, b)));
        Ok(())
    }
    
    fn execute_div(&mut self) -> Result<()> {
        let b = self.pop_int()?;
        let a = self.pop_int()?;
        if b == 0 {
            bail!("Division by zero");
        }
        self.stack.push(Value::Int(a / b));
        Ok(())
    }
    
    fn execute_mod(&mut self) -> Result<()> {
        let b = self.pop_int()?;
        let a = self.pop_int()?;
        if b == 0 {
            bail!("Modulo by zero");
        }
        self.stack.push(Value::Int(a % b));
        Ok(())
    }
    
    fn execute_neg(&mut self) -> Result<()> {
        let value = self.pop_int()?;
        self.stack.push(Value::Int(-value));
        Ok(())
    }
    
    fn execute_comparison(&mut self, op: fn(i64, i64) -> bool) -> Result<()> {
        let b = self.pop_int()?;
        let a = self.pop_int()?;
        self.stack.push(Value::Bool(op(a, b)));
        Ok(())
    }
    
    fn execute_and(&mut self) -> Result<()> {
        let b = self.pop_bool()?;
        let a = self.pop_bool()?;
        self.stack.push(Value::Bool(a && b));
        Ok(())
    }
    
    fn execute_or(&mut self) -> Result<()> {
        let b = self.pop_bool()?;
        let a = self.pop_bool()?;
        self.stack.push(Value::Bool(a || b));
        Ok(())
    }
    
    fn execute_not(&mut self) -> Result<()> {
        let value = self.pop_bool()?;
        self.stack.push(Value::Bool(!value));
        Ok(())
    }
    
    fn execute_jump(&mut self, inst: &Instruction) -> Result<()> {
        let target = self.get_jump_target(inst)?;
        self.pc = target.saturating_sub(1); // -1 because pc++ happens after
        Ok(())
    }
    
    fn execute_jump_if(&mut self, inst: &Instruction) -> Result<()> {
        let condition = self.pop_bool()?;
        if condition {
            let target = self.get_jump_target(inst)?;
            self.pc = target.saturating_sub(1);
        }
        Ok(())
    }
    
    fn execute_jump_if_not(&mut self, inst: &Instruction) -> Result<()> {
        let condition = self.pop_bool()?;
        if !condition {
            let target = self.get_jump_target(inst)?;
            self.pc = target.saturating_sub(1);
        }
        Ok(())
    }
    
    fn execute_load_local(&mut self, inst: &Instruction) -> Result<()> {
        let index = self.get_index(inst)?;
        let value = self.locals.get(&index)
            .ok_or_else(|| anyhow::anyhow!("Undefined local variable at index {}", index))?
            .clone();
        self.stack.push(value);
        Ok(())
    }
    
    fn execute_store_local(&mut self, inst: &Instruction) -> Result<()> {
        let index = self.get_index(inst)?;
        let value = self.stack.pop()
            .ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        self.locals.insert(index, value);
        Ok(())
    }
    
    fn execute_load_global(&mut self, inst: &Instruction) -> Result<()> {
        let name = self.get_string(inst)?;
        let value = self.globals.get(&name)
            .ok_or_else(|| anyhow::anyhow!("Undefined global variable: {}", name))?
            .clone();
        self.stack.push(value);
        Ok(())
    }
    
    fn execute_store_global(&mut self, inst: &Instruction) -> Result<()> {
        let name = self.get_string(inst)?;
        let value = self.stack.pop()
            .ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        self.globals.insert(name, value);
        Ok(())
    }
    
    fn execute_print(&mut self) -> Result<()> {
        let value = self.stack.pop()
            .ok_or_else(|| anyhow::anyhow!("Stack underflow"))?;
        self.output.push(format!("{:?}", value));
        Ok(())
    }
    
    // Helper methods
    fn pop_int(&mut self) -> Result<i64> {
        match self.stack.pop() {
            Some(Value::Int(n)) => Ok(n),
            Some(_) => bail!("Expected integer on stack"),
            None => bail!("Stack underflow"),
        }
    }
    
    fn pop_bool(&mut self) -> Result<bool> {
        match self.stack.pop() {
            Some(Value::Bool(b)) => Ok(b),
            Some(_) => bail!("Expected boolean on stack"),
            None => bail!("Stack underflow"),
        }
    }
    
    fn get_jump_target(&self, inst: &Instruction) -> Result<usize> {
        match &inst.operand {
            Some(Value::Int(target)) => Ok(*target as usize),
            _ => bail!("Jump instruction requires integer target"),
        }
    }
    
    fn get_index(&self, inst: &Instruction) -> Result<usize> {
        match &inst.operand {
            Some(Value::Int(idx)) => Ok(*idx as usize),
            _ => bail!("Index operand must be integer"),
        }
    }
    
    fn get_string(&self, inst: &Instruction) -> Result<String> {
        match &inst.operand {
            Some(Value::String(s)) => Ok(s.clone()),
            _ => bail!("String operand required"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vm_arithmetic() {
        let mut vm = VirtualMachine::new();
        let mut module = BytecodeModule::new();
        
        // 2 + 3 = 5
        module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(2)));
        module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(3)));
        module.add_instruction(Instruction::new(OpCode::Add));
        module.add_instruction(Instruction::new(OpCode::Halt));
        
        let result = vm.execute(&module).unwrap();
        assert_eq!(result.value, Some(Value::Int(5)));
    }
    
    #[test]
    fn test_vm_comparison() {
        let mut vm = VirtualMachine::new();
        let mut module = BytecodeModule::new();
        
        // 5 > 3 = true
        module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(5)));
        module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(3)));
        module.add_instruction(Instruction::new(OpCode::Gt));
        module.add_instruction(Instruction::new(OpCode::Halt));
        
        let result = vm.execute(&module).unwrap();
        assert_eq!(result.value, Some(Value::Bool(true)));
    }
    
    #[test]
    fn test_vm_print() {
        let mut vm = VirtualMachine::new();
        let mut module = BytecodeModule::new();
        
        module.add_instruction(Instruction::with_operand(OpCode::Push, Value::String("Hello".to_string())));
        module.add_instruction(Instruction::new(OpCode::Print));
        module.add_instruction(Instruction::new(OpCode::Halt));
        
        let result = vm.execute(&module).unwrap();
        assert_eq!(result.output, vec!["String(\"Hello\")"]);
    }
    
    #[test]
    fn test_vm_variables() {
        let mut vm = VirtualMachine::new();
        let mut module = BytecodeModule::new();
        
        // Store 42 in local[0], then load it
        module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(42)));
        module.add_instruction(Instruction::with_operand(OpCode::StoreLocal, Value::Int(0)));
        module.add_instruction(Instruction::with_operand(OpCode::LoadLocal, Value::Int(0)));
        module.add_instruction(Instruction::new(OpCode::Halt));
        
        let result = vm.execute(&module).unwrap();
        assert_eq!(result.value, Some(Value::Int(42)));
    }
    
    #[test]
    fn test_vm_jump() {
        let mut vm = VirtualMachine::new();
        let mut module = BytecodeModule::new();
        
        // Jump over the push 99
        module.add_instruction(Instruction::with_operand(OpCode::Jump, Value::Int(2)));
        module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(99)));
        module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(42)));
        module.add_instruction(Instruction::new(OpCode::Halt));
        
        let result = vm.execute(&module).unwrap();
        assert_eq!(result.value, Some(Value::Int(42)));
    }
}