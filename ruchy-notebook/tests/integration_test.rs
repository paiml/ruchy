use ruchy_notebook::{VirtualMachine, BytecodeModule, OpCode};
use ruchy_notebook::vm::{Instruction, bytecode::Value};

#[test]
fn test_notebook_vm_integration() {
    let mut vm = VirtualMachine::new();
    let mut module = BytecodeModule::new();
    
    // Test program: (10 + 20) * 3 = 90
    module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(10)));
    module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(20)));
    module.add_instruction(Instruction::new(OpCode::Add));
    module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(3)));
    module.add_instruction(Instruction::new(OpCode::Mul));
    module.add_instruction(Instruction::new(OpCode::Halt));
    
    let result = vm.execute(&module).unwrap();
    assert_eq!(result.value, Some(Value::Int(90)));
}

#[test]
fn test_vm_conditionals() {
    let mut vm = VirtualMachine::new();
    let mut module = BytecodeModule::new();
    
    // Test: if 5 > 3 then 100 else 200
    module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(5)));
    module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(3)));
    module.add_instruction(Instruction::new(OpCode::Gt));
    module.add_instruction(Instruction::with_operand(OpCode::JumpIfNot, Value::Int(6)));
    module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(100)));
    module.add_instruction(Instruction::with_operand(OpCode::Jump, Value::Int(7)));
    module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(200)));
    module.add_instruction(Instruction::new(OpCode::Halt));
    
    let result = vm.execute(&module).unwrap();
    assert_eq!(result.value, Some(Value::Int(100)));
}

#[test]
fn test_vm_variables() {
    let mut vm = VirtualMachine::new();
    let mut module = BytecodeModule::new();
    
    // Test: x = 42; y = 58; x + y = 100
    module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(42)));
    module.add_instruction(Instruction::with_operand(OpCode::StoreLocal, Value::Int(0)));
    module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(58)));
    module.add_instruction(Instruction::with_operand(OpCode::StoreLocal, Value::Int(1)));
    module.add_instruction(Instruction::with_operand(OpCode::LoadLocal, Value::Int(0)));
    module.add_instruction(Instruction::with_operand(OpCode::LoadLocal, Value::Int(1)));
    module.add_instruction(Instruction::new(OpCode::Add));
    module.add_instruction(Instruction::new(OpCode::Halt));
    
    let result = vm.execute(&module).unwrap();
    assert_eq!(result.value, Some(Value::Int(100)));
}

#[test]
fn test_vm_stack_operations() {
    let mut vm = VirtualMachine::new();
    let mut module = BytecodeModule::new();
    
    // Test stack operations: push 1, dup, add = 2
    module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(1)));
    module.add_instruction(Instruction::new(OpCode::Dup));
    module.add_instruction(Instruction::new(OpCode::Add));
    module.add_instruction(Instruction::new(OpCode::Halt));
    
    let result = vm.execute(&module).unwrap();
    assert_eq!(result.value, Some(Value::Int(2)));
}

#[test]
fn test_vm_division_by_zero() {
    let mut vm = VirtualMachine::new();
    let mut module = BytecodeModule::new();
    
    // Test division by zero error handling
    module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(10)));
    module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(0)));
    module.add_instruction(Instruction::new(OpCode::Div));
    module.add_instruction(Instruction::new(OpCode::Halt));
    
    let result = vm.execute(&module);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Division by zero"));
}