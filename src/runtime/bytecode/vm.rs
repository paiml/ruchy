//! Bytecode Virtual Machine Executor
//!
//! OPT-003: Bytecode VM Executor
//!
//! Register-based bytecode interpreter with optimized dispatch loop.
//! Expected performance: 40-60% faster than AST walking, 30-40% memory reduction.
//!
//! # Architecture
//!
//! - **Register File**: 32 general-purpose registers per call frame
//! - **Call Stack**: Stack of call frames for function invocations
//! - **Dispatch Loop**: Match-based dispatch (later: computed goto optimization)
//! - **Memory**: Shared global variable storage + per-frame locals
//!
//! Reference: ../ruchyruchy/OPTIMIZATION_REPORT_FOR_RUCHY.md
//! Academic: Brunthaler (2010) - Inline Caching Meets Quickening

use super::instruction::Instruction;
use super::opcode::OpCode;
use super::compiler::BytecodeChunk;
use crate::runtime::Value;
use std::collections::HashMap;

/// Maximum number of registers per call frame
const MAX_REGISTERS: usize = 32;

/// Call frame for function invocation
///
/// Represents a single function call on the VM's call stack.
/// Contains the bytecode chunk being executed, program counter, and register base.
#[derive(Debug)]
struct CallFrame<'a> {
    /// Bytecode chunk being executed
    chunk: &'a BytecodeChunk,
    /// Program counter (instruction index)
    pc: usize,
    /// Base register index for this frame
    base_register: u8,
}

impl<'a> CallFrame<'a> {
    /// Create a new call frame
    fn new(chunk: &'a BytecodeChunk) -> Self {
        Self {
            chunk,
            pc: 0,
            base_register: 0,
        }
    }

    /// Fetch the current instruction
    #[inline]
    fn fetch_instruction(&self) -> Option<Instruction> {
        self.chunk.instructions.get(self.pc).copied()
    }

    /// Advance the program counter
    #[inline]
    fn advance_pc(&mut self) {
        self.pc += 1;
    }

    /// Jump to a specific instruction offset (relative)
    ///
    /// Note: advance_pc() will be called after this, so we compensate by subtracting 1
    #[inline]
    fn jump(&mut self, offset: i16) {
        // Offset is relative to current PC, but advance_pc will add 1, so we compensate
        let target = (self.pc as i32) + (offset as i32);
        self.pc = target as usize;
    }
}

/// Bytecode Virtual Machine
///
/// Executes bytecode instructions with register-based architecture.
///
/// # Examples
///
/// ```
/// use ruchy::runtime::bytecode::{VM, Compiler, Instruction, OpCode};
/// use ruchy::runtime::Value;
/// use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span};
///
/// // Compile: 42
/// let mut compiler = Compiler::new("test".to_string());
/// let expr = Expr::new(ExprKind::Literal(Literal::Integer(42, None)), Span::default());
/// compiler.compile_expr(&expr).unwrap();
/// let chunk = compiler.finalize();
///
/// // Execute
/// let mut vm = VM::new();
/// let result = vm.execute(&chunk).unwrap();
/// assert_eq!(result, Value::Integer(42));
/// ```
#[derive(Debug)]
pub struct VM {
    /// Register file (32 general-purpose registers)
    registers: [Value; MAX_REGISTERS],
    /// Call stack (function invocations)
    call_stack: Vec<CallFrame<'static>>,
    /// Global variables
    globals: HashMap<String, Value>,
}

impl VM {
    /// Create a new bytecode VM
    pub fn new() -> Self {
        Self {
            registers: std::array::from_fn(|_| Value::Nil),
            call_stack: Vec::new(),
            globals: HashMap::new(),
        }
    }

    /// Execute a bytecode chunk
    ///
    /// Returns the result of the last executed instruction.
    pub fn execute(&mut self, chunk: &BytecodeChunk) -> Result<Value, String> {
        // Safety: We need to extend the lifetime to 'static for the call stack
        // This is safe because the call frame doesn't outlive the chunk reference
        let chunk_ref: &'static BytecodeChunk = unsafe {
            std::mem::transmute(chunk)
        };

        // Push initial call frame
        self.call_stack.push(CallFrame::new(chunk_ref));

        // Main execution loop
        while let Some(frame) = self.call_stack.last_mut() {
            // Fetch instruction
            let instruction = match frame.fetch_instruction() {
                Some(instr) => instr,
                None => {
                    // End of bytecode - pop frame
                    self.call_stack.pop();
                    continue;
                }
            };

            // Decode opcode
            let opcode = OpCode::from_u8(instruction.opcode())
                .ok_or_else(|| format!("Invalid opcode: {}", instruction.opcode()))?;

            // Execute instruction
            self.execute_instruction(opcode, instruction)?;

            // Advance PC (unless instruction modified it)
            if let Some(frame) = self.call_stack.last_mut() {
                frame.advance_pc();
            }
        }

        // Return value is in register 0
        Ok(self.registers[0].clone())
    }

    /// Execute a single instruction
    #[inline]
    fn execute_instruction(&mut self, opcode: OpCode, instruction: Instruction) -> Result<(), String> {
        match opcode {
            // Load constant into register
            OpCode::Const => {
                let dest = instruction.get_a() as usize;
                let const_idx = instruction.get_bx() as usize;

                let frame = self.call_stack.last()
                    .ok_or("No active call frame")?;
                let value = frame.chunk.constants.get(const_idx)
                    .ok_or_else(|| format!("Constant index out of bounds: {}", const_idx))?;

                self.registers[dest] = value.clone();
                Ok(())
            }

            // Move value between registers
            OpCode::Move => {
                let dest = instruction.get_a() as usize;
                let src = instruction.get_b() as usize;

                self.registers[dest] = self.registers[src].clone();
                Ok(())
            }

            // Arithmetic operations
            OpCode::Add => self.binary_op(instruction, |a, b| a.add(b)),
            OpCode::Sub => self.binary_op(instruction, |a, b| a.subtract(b)),
            OpCode::Mul => self.binary_op(instruction, |a, b| a.multiply(b)),
            OpCode::Div => self.binary_op(instruction, |a, b| a.divide(b)),
            OpCode::Mod => self.binary_op(instruction, |a, b| a.modulo(b)),

            // Unary operations
            OpCode::Neg => self.unary_op(instruction, |v| match v {
                Value::Integer(i) => Ok(Value::Integer(-i)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Err(format!("Cannot negate {}", v.type_name())),
            }),
            OpCode::Not => self.unary_op(instruction, |v| Ok(Value::Bool(!v.is_truthy()))),
            OpCode::BitNot => self.unary_op(instruction, |v| match v {
                Value::Integer(i) => Ok(Value::Integer(!i)),
                _ => Err(format!("Cannot apply bitwise NOT to {}", v.type_name())),
            }),

            // Comparison operations
            OpCode::Equal => self.binary_op(instruction, |a, b| Ok(Value::Bool(a == b))),
            OpCode::NotEqual => self.binary_op(instruction, |a, b| Ok(Value::Bool(a != b))),
            OpCode::Less => self.comparison_op(instruction, |a, b| a.less_than(b)),
            OpCode::LessEqual => self.comparison_op(instruction, |a, b| a.less_equal(b)),
            OpCode::Greater => self.comparison_op(instruction, |a, b| a.greater_than(b)),
            OpCode::GreaterEqual => self.comparison_op(instruction, |a, b| a.greater_equal(b)),

            // Logical operations
            OpCode::And => self.logical_op(instruction, |a, b| a && b),
            OpCode::Or => self.logical_op(instruction, |a, b| a || b),

            // Control flow
            OpCode::Jump => {
                let offset = instruction.get_sbx();
                if let Some(frame) = self.call_stack.last_mut() {
                    frame.jump(offset);
                }
                Ok(())
            }

            OpCode::JumpIfFalse => {
                let condition = instruction.get_a() as usize;
                let offset = instruction.get_sbx();

                let is_false = match &self.registers[condition] {
                    Value::Bool(false) | Value::Nil => true,
                    _ => false,
                };

                if is_false {
                    if let Some(frame) = self.call_stack.last_mut() {
                        frame.jump(offset);
                    }
                }
                Ok(())
            }

            OpCode::JumpIfTrue => {
                let condition = instruction.get_a() as usize;
                let offset = instruction.get_sbx();

                let is_true = match &self.registers[condition] {
                    Value::Bool(true) => true,
                    Value::Bool(false) | Value::Nil => false,
                    _ => true, // Truthy by default
                };

                if is_true {
                    if let Some(frame) = self.call_stack.last_mut() {
                        frame.jump(offset);
                    }
                }
                Ok(())
            }

            OpCode::Return => {
                // Get return value from register specified in instruction
                let return_reg = instruction.get_a() as usize;
                let return_value = self.registers[return_reg].clone();

                // Pop call frame
                self.call_stack.pop();

                // Store return value in register 0 for caller
                self.registers[0] = return_value;
                Ok(())
            }

            // Load/store global variables
            OpCode::LoadGlobal => {
                let dest = instruction.get_a() as usize;
                let name_idx = instruction.get_bx() as usize;

                let frame = self.call_stack.last()
                    .ok_or("No active call frame")?;
                let name_value = frame.chunk.constants.get(name_idx)
                    .ok_or_else(|| format!("Constant index out of bounds: {}", name_idx))?;

                let name = match name_value {
                    Value::String(s) => s.as_ref(),
                    _ => return Err("Global name must be a string".to_string()),
                };

                let value = self.globals.get(name)
                    .ok_or_else(|| format!("Undefined global variable: {}", name))?;

                self.registers[dest] = value.clone();
                Ok(())
            }

            OpCode::StoreGlobal => {
                let src = instruction.get_a() as usize;
                let name_idx = instruction.get_bx() as usize;

                let frame = self.call_stack.last()
                    .ok_or("No active call frame")?;
                let name_value = frame.chunk.constants.get(name_idx)
                    .ok_or_else(|| format!("Constant index out of bounds: {}", name_idx))?;

                let name = match name_value {
                    Value::String(s) => s.to_string(),
                    _ => return Err("Global name must be a string".to_string()),
                };

                self.globals.insert(name, self.registers[src].clone());
                Ok(())
            }

            _ => Err(format!("Unsupported opcode: {:?}", opcode)),
        }
    }

    /// Execute a binary arithmetic operation
    #[inline]
    fn binary_op<F>(&mut self, instruction: Instruction, op: F) -> Result<(), String>
    where
        F: FnOnce(&Value, &Value) -> Result<Value, String>,
    {
        let dest = instruction.get_a() as usize;
        let left = instruction.get_b() as usize;
        let right = instruction.get_c() as usize;

        let result = op(&self.registers[left], &self.registers[right])?;
        self.registers[dest] = result;
        Ok(())
    }

    /// Execute a unary operation
    #[inline]
    fn unary_op<F>(&mut self, instruction: Instruction, op: F) -> Result<(), String>
    where
        F: FnOnce(&Value) -> Result<Value, String>,
    {
        let dest = instruction.get_a() as usize;
        let operand = instruction.get_b() as usize;

        let result = op(&self.registers[operand])?;
        self.registers[dest] = result;
        Ok(())
    }

    /// Execute a comparison operation
    #[inline]
    fn comparison_op<F>(&mut self, instruction: Instruction, op: F) -> Result<(), String>
    where
        F: FnOnce(&Value, &Value) -> bool,
    {
        let dest = instruction.get_a() as usize;
        let left = instruction.get_b() as usize;
        let right = instruction.get_c() as usize;

        let result = op(&self.registers[left], &self.registers[right]);
        self.registers[dest] = Value::Bool(result);
        Ok(())
    }

    /// Execute a logical operation
    #[inline]
    fn logical_op<F>(&mut self, instruction: Instruction, op: F) -> Result<(), String>
    where
        F: FnOnce(bool, bool) -> bool,
    {
        let dest = instruction.get_a() as usize;
        let left = instruction.get_b() as usize;
        let right = instruction.get_c() as usize;

        let left_bool = self.registers[left].is_truthy();
        let right_bool = self.registers[right].is_truthy();

        let result = op(left_bool, right_bool);
        self.registers[dest] = Value::Bool(result);
        Ok(())
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Span};
    use crate::runtime::bytecode::Compiler;

    #[test]
    fn test_vm_execute_integer_literal() {
        // Compile: 42
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        // Execute
        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_execute_addition() {
        // Compile: 10 + 32
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(32, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        // Execute
        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_execute_multiplication() {
        // Compile: 6 * 7
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(6, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(7, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        // Execute
        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_execute_comparison() {
        // Compile: 10 < 20
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(20, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Less,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        // Execute
        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_execute_if_true_branch() {
        // Compile: if true { 42 } else { 0 }
        let mut compiler = Compiler::new("test".to_string());
        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::default(),
        );
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let else_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Some(Box::new(else_branch)),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        // Execute
        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_execute_if_false_branch() {
        // Compile: if false { 42 } else { 100 }
        let mut compiler = Compiler::new("test".to_string());
        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            Span::default(),
        );
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let else_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Some(Box::new(else_branch)),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        // Execute
        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(100));
    }

    #[test]
    fn test_vm_execute_block() {
        // Compile: { 1; 2; 3 }
        let mut compiler = Compiler::new("test".to_string());
        let exprs = vec![
            Expr::new(ExprKind::Literal(Literal::Integer(1, None)), Span::default()),
            Expr::new(ExprKind::Literal(Literal::Integer(2, None)), Span::default()),
            Expr::new(ExprKind::Literal(Literal::Integer(3, None)), Span::default()),
        ];
        let block = Expr::new(ExprKind::Block(exprs), Span::default());
        compiler.compile_expr(&block).unwrap();
        let chunk = compiler.finalize();

        // Execute
        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(3));
    }
}
