//! Direct-threaded Interpreter Compilation Module
//!
//! EXTREME TDD: Full test coverage, zero entropy, <10 complexity per function
//! Extracted from interpreter.rs to eliminate bloat and follow Toyota Way principles.
//!
//! This module implements a direct-threaded interpreter for Ruchy that compiles
//! AST expressions to instruction streams for high-performance execution.
//!
//! # Features
//!
//! - Direct-threaded dispatch for optimal performance
//! - Instruction-based compilation of AST expressions
//! - Support for basic operations: arithmetic, conditionals, literals
//! - Stack-based execution model
//! - Comprehensive error handling with proper bounds checking

use crate::frontend::ast::{Expr, ExprKind, Literal};
use crate::runtime::{InterpreterError, Value};

/// Result of instruction execution
#[derive(Debug, Clone)]
pub enum InstructionResult {
    /// Continue to next instruction
    Continue,
    /// Jump to specific instruction address
    Jump(usize),
    /// Return from execution with value
    Return(Value),
    /// Error occurred during execution
    Error(InterpreterError),
}

/// Compiled instruction for direct-threaded execution
#[derive(Clone)]
pub struct ThreadedInstruction {
    /// Function pointer to instruction handler
    pub handler: fn(&mut InterpreterState, u32) -> InstructionResult,
    /// Operand for the instruction
    pub operand: u32,
}

impl std::fmt::Debug for ThreadedInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadedInstruction")
            .field("handler", &"<function>")
            .field("operand", &self.operand)
            .finish()
    }
}

/// Execution state for the direct-threaded interpreter
#[derive(Debug, Clone)]
pub struct InterpreterState {
    /// Execution stack
    pub stack: Vec<Value>,
    /// Constants pool
    pub constants: Vec<Value>,
}

impl InterpreterState {
    /// Create new interpreter state
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1 (within Toyota Way limits)
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            constants: Vec::new(),
        }
    }
}

impl Default for InterpreterState {
    fn default() -> Self {
        Self::new()
    }
}

/// Direct-threaded interpreter for high-performance execution
///
/// Uses function pointers and direct dispatch to avoid switch overhead.
/// Compiles AST expressions to instruction streams for optimal execution.
#[derive(Debug)]
pub struct DirectThreadedInterpreter {
    /// Compiled instruction stream
    code: Vec<ThreadedInstruction>,
    /// Constants pool for literals
    constants: Vec<Value>,
    /// Program counter
    pc: usize,
    /// Execution state
    state: InterpreterState,
}

impl DirectThreadedInterpreter {
    /// Create new direct-threaded interpreter
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1 (within Toyota Way limits)
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            pc: 0,
            state: InterpreterState::new(),
        }
    }

    /// Compile expression to instruction stream
    ///
    /// # Complexity
    /// Cyclomatic complexity: 7 (within Toyota Way limits)
    pub fn compile(&mut self, expr: &Expr) -> Result<(), InterpreterError> {
        self.code.clear();
        self.constants.clear();
        self.pc = 0;

        // Compile expression to instruction stream
        self.compile_expr(expr)?;

        // Add return instruction if needed
        if self.code.is_empty()
            || !matches!(self.code.last(), Some(instr) if
            std::ptr::eq(instr.handler as *const (), op_return as *const ()))
        {
            self.emit_instruction(op_return, 0);
        }

        // Copy constants to state
        self.state.constants = self.constants.clone();

        Ok(())
    }

    /// Execute compiled instruction stream using direct-threaded dispatch
    ///
    /// # Complexity
    /// Cyclomatic complexity: 8 (within Toyota Way limits)
    pub fn execute(&mut self) -> Result<Value, InterpreterError> {
        self.pc = 0;

        loop {
            // Bounds check
            if self.pc >= self.code.len() {
                return Err(InterpreterError::RuntimeError(
                    "PC out of bounds".to_string(),
                ));
            }

            // Direct function pointer call - no switch overhead
            let instruction = &self.code[self.pc];
            let result = (instruction.handler)(&mut self.state, instruction.operand);

            match result {
                InstructionResult::Continue => {
                    self.pc += 1;
                }
                InstructionResult::Jump(target) => {
                    if target >= self.code.len() {
                        return Err(InterpreterError::RuntimeError(
                            "Jump target out of bounds".to_string(),
                        ));
                    }
                    self.pc = target;
                }
                InstructionResult::Return(value) => {
                    return Ok(value);
                }
                InstructionResult::Error(error) => {
                    return Err(error);
                }
            }

            // Periodic interrupt check for long-running loops
            if self.pc.trailing_zeros() >= 10 {
                // Could add interrupt checking here in the future
            }
        }
    }

    /// Compile single expression to instruction stream
    ///
    /// # Complexity
    /// Cyclomatic complexity: 5 (within Toyota Way limits)
    fn compile_expr(&mut self, expr: &Expr) -> Result<(), InterpreterError> {
        match &expr.kind {
            ExprKind::Literal(lit) => self.compile_literal(lit),
            ExprKind::Binary { left, op, right } => self.compile_binary_expr(left, op, right),
            ExprKind::Identifier(name) => self.compile_identifier(name),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.compile_if_expr(condition, then_branch, else_branch.as_deref()),
            _ => self.compile_fallback_expr(),
        }
    }

    /// Compile literal expression
    ///
    /// # Complexity
    /// Cyclomatic complexity: 3 (within Toyota Way limits)
    fn compile_literal(&mut self, lit: &Literal) -> Result<(), InterpreterError> {
        if matches!(lit, Literal::Unit | Literal::Null) {
            self.emit_instruction(op_load_nil, 0);
        } else {
            let const_idx = self.add_constant(self.literal_to_value(lit));
            self.emit_instruction(op_load_const, const_idx);
        }
        Ok(())
    }

    /// Compile binary expression
    ///
    /// # Complexity
    /// Cyclomatic complexity: 2 (within Toyota Way limits)
    fn compile_binary_expr(
        &mut self,
        left: &Expr,
        op: &crate::frontend::ast::BinaryOp,
        right: &Expr,
    ) -> Result<(), InterpreterError> {
        self.compile_expr(left)?;
        self.compile_expr(right)?;

        let op_code = self.binary_op_to_opcode(op)?;
        self.emit_instruction(op_code, 0);
        Ok(())
    }

    /// Convert binary operator to opcode
    ///
    /// # Complexity
    /// Cyclomatic complexity: 5 (within Toyota Way limits)
    fn binary_op_to_opcode(
        &self,
        op: &crate::frontend::ast::BinaryOp,
    ) -> Result<fn(&mut InterpreterState, u32) -> InstructionResult, InterpreterError> {
        match op {
            crate::frontend::ast::BinaryOp::Add => Ok(op_add),
            crate::frontend::ast::BinaryOp::Subtract => Ok(op_sub),
            crate::frontend::ast::BinaryOp::Multiply => Ok(op_mul),
            crate::frontend::ast::BinaryOp::Divide => Ok(op_div),
            _ => Err(InterpreterError::RuntimeError(format!(
                "Unsupported binary operation: {op:?}"
            ))),
        }
    }

    /// Compile identifier expression
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1 (within Toyota Way limits)
    fn compile_identifier(&mut self, name: &str) -> Result<(), InterpreterError> {
        let name_idx = self.add_constant(Value::from_string(name.to_string()));
        self.emit_instruction(op_load_var, name_idx);
        Ok(())
    }

    /// Compile if expression
    ///
    /// # Complexity
    /// Cyclomatic complexity: 3 (within Toyota Way limits)
    fn compile_if_expr(
        &mut self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) -> Result<(), InterpreterError> {
        self.compile_expr(condition)?;

        let else_jump_addr = self.code.len();
        self.emit_instruction(op_jump_if_false, 0);

        self.compile_expr(then_branch)?;

        if let Some(else_expr) = else_branch {
            self.compile_if_with_else_branch(else_jump_addr, else_expr)
        } else {
            self.compile_if_without_else_branch(else_jump_addr)
        }
    }

    /// Compile if expression with else branch
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1 (within Toyota Way limits)
    fn compile_if_with_else_branch(
        &mut self,
        else_jump_addr: usize,
        else_expr: &Expr,
    ) -> Result<(), InterpreterError> {
        let end_jump_addr = self.code.len();
        self.emit_instruction(op_jump, 0);

        self.patch_jump_target(else_jump_addr, self.code.len());
        self.compile_expr(else_expr)?;
        self.patch_jump_target(end_jump_addr, self.code.len());

        Ok(())
    }

    /// Compile if expression without else branch
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1 (within Toyota Way limits)
    fn compile_if_without_else_branch(
        &mut self,
        else_jump_addr: usize,
    ) -> Result<(), InterpreterError> {
        self.patch_jump_target(else_jump_addr, self.code.len());
        self.emit_instruction(op_load_nil, 0);
        Ok(())
    }

    /// Patch jump target for branching instructions
    ///
    /// # Complexity
    /// Cyclomatic complexity: 2 (within Toyota Way limits)
    fn patch_jump_target(&mut self, jump_addr: usize, target: usize) {
        if let Some(instr) = self.code.get_mut(jump_addr) {
            instr.operand = target as u32;
        }
    }

    /// Compile fallback for unsupported expressions
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1 (within Toyota Way limits)
    fn compile_fallback_expr(&mut self) -> Result<(), InterpreterError> {
        let value_idx = self.add_constant(Value::from_string("AST_FALLBACK".to_string()));
        self.emit_instruction(op_ast_fallback, value_idx);
        Ok(())
    }

    /// Add constant to pool and return index
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1 (within Toyota Way limits)
    #[allow(clippy::cast_possible_truncation)] // Index bounds are controlled
    fn add_constant(&mut self, value: Value) -> u32 {
        let idx = self.constants.len();
        self.constants.push(value);
        idx as u32
    }

    /// Emit instruction to code stream
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1 (within Toyota Way limits)
    fn emit_instruction(
        &mut self,
        handler: fn(&mut InterpreterState, u32) -> InstructionResult,
        operand: u32,
    ) {
        self.code.push(ThreadedInstruction { handler, operand });
    }

    /// Convert literal to value
    ///
    /// # Complexity
    /// Cyclomatic complexity: 8 (within Toyota Way limits)
    fn literal_to_value(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Integer(n, _) => Value::Integer(*n),
            Literal::Float(f) => Value::Float(*f),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::String(s) => Value::from_string(s.clone()),
            Literal::Char(c) => Value::from_string(c.to_string()), // Convert char to single-character string
            Literal::Byte(b) => Value::Byte(*b),
            Literal::Unit => Value::Nil, // Unit maps to Nil
            Literal::Null => Value::Nil, // Null maps to Nil
            Literal::Atom(s) => Value::Atom(s.clone()),
        }
    }

    /// Get instruction count
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1 (within Toyota Way limits)
    pub fn instruction_count(&self) -> usize {
        self.code.len()
    }

    /// Get constants count
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1 (within Toyota Way limits)
    pub fn constants_count(&self) -> usize {
        self.constants.len()
    }

    /// Add instruction to code stream (public interface for tests)
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1 (within Toyota Way limits)
    pub fn add_instruction(
        &mut self,
        handler: fn(&mut InterpreterState, u32) -> InstructionResult,
        operand: u32,
    ) {
        self.emit_instruction(handler, operand);
    }

    /// Clear all instructions and constants
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1 (within Toyota Way limits)
    pub fn clear(&mut self) {
        self.code.clear();
        self.constants.clear();
        self.pc = 0;
        self.state = InterpreterState::new();
    }

    /// Execute with custom interpreter state (for tests)
    ///
    /// # Complexity
    /// Cyclomatic complexity: 8 (within Toyota Way limits)
    pub fn execute_with_state(
        &mut self,
        state: &mut InterpreterState,
    ) -> Result<Value, InterpreterError> {
        self.pc = 0;

        loop {
            // Bounds check
            if self.pc >= self.code.len() {
                return Err(InterpreterError::RuntimeError(
                    "PC out of bounds".to_string(),
                ));
            }

            // Direct function pointer call - no switch overhead
            let instruction = &self.code[self.pc];
            let result = (instruction.handler)(state, instruction.operand);

            match result {
                InstructionResult::Continue => {
                    self.pc += 1;
                }
                InstructionResult::Jump(target) => {
                    if target >= self.code.len() {
                        return Err(InterpreterError::RuntimeError(
                            "Jump target out of bounds".to_string(),
                        ));
                    }
                    self.pc = target;
                }
                InstructionResult::Return(value) => {
                    return Ok(value);
                }
                InstructionResult::Error(error) => {
                    return Err(error);
                }
            }

            // Periodic interrupt check for long-running loops
            if self.pc.trailing_zeros() >= 10 {
                // Could add interrupt checking here in the future
            }
        }
    }
}

impl Default for DirectThreadedInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

// Instruction handler functions - these are called via function pointers

/// Load constant onto stack
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn op_load_const(state: &mut InterpreterState, const_idx: u32) -> InstructionResult {
    if let Some(value) = state.constants.get(const_idx as usize) {
        state.stack.push(value.clone());
        InstructionResult::Continue
    } else {
        InstructionResult::Error(InterpreterError::RuntimeError(
            "Invalid constant index".to_string(),
        ))
    }
}

/// Load nil value onto stack
///
/// # Complexity
/// Cyclomatic complexity: 1 (within Toyota Way limits)
pub fn op_load_nil(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    state.stack.push(Value::Nil);
    InstructionResult::Continue
}

/// Load variable value onto stack
///
/// # Complexity
/// Cyclomatic complexity: 1 (within Toyota Way limits)
#[allow(dead_code)] // Will be used when variable support is implemented
pub fn op_load_var(_state: &mut InterpreterState, name_idx: u32) -> InstructionResult {
    // For now, just return error - variable lookup not implemented in isolated module
    InstructionResult::Error(InterpreterError::RuntimeError(format!(
        "Variable lookup not implemented in isolated compilation module: index {name_idx}"
    )))
}

/// Return value from execution
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn op_return(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    if let Some(value) = state.stack.pop() {
        InstructionResult::Return(value)
    } else {
        InstructionResult::Return(Value::Nil)
    }
}

/// Jump unconditionally
///
/// # Complexity
/// Cyclomatic complexity: 1 (within Toyota Way limits)
pub fn op_jump(_state: &mut InterpreterState, target: u32) -> InstructionResult {
    InstructionResult::Jump(target as usize)
}

/// Jump if top of stack is false
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
pub fn op_jump_if_false(state: &mut InterpreterState, target: u32) -> InstructionResult {
    if let Some(value) = state.stack.pop() {
        match value {
            Value::Bool(false) | Value::Nil => InstructionResult::Jump(target as usize),
            _ => InstructionResult::Continue,
        }
    } else {
        InstructionResult::Error(InterpreterError::RuntimeError(
            "Stack underflow in jump_if_false".to_string(),
        ))
    }
}

/// AST fallback handler
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
#[allow(dead_code)] // Will be used for unsupported AST nodes
pub fn op_ast_fallback(state: &mut InterpreterState, fallback_idx: u32) -> InstructionResult {
    if let Some(Value::String(fallback_msg)) = state.constants.get(fallback_idx as usize) {
        InstructionResult::Error(InterpreterError::RuntimeError(format!(
            "AST fallback not implemented: {fallback_msg}"
        )))
    } else {
        InstructionResult::Error(InterpreterError::RuntimeError(
            "Invalid AST fallback constant".to_string(),
        ))
    }
}

/// Binary addition operation
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn op_add(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    binary_arithmetic_op(state, |a, b| match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => Some(Value::Integer(x + y)),
        (Value::Float(x), Value::Float(y)) => Some(Value::Float(x + y)),
        (Value::Integer(x), Value::Float(y)) => Some(Value::Float(*x as f64 + y)),
        (Value::Float(x), Value::Integer(y)) => Some(Value::Float(x + *y as f64)),
        _ => None,
    })
}

/// Binary subtraction operation
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn op_sub(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    binary_arithmetic_op(state, |a, b| match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => Some(Value::Integer(x - y)),
        (Value::Float(x), Value::Float(y)) => Some(Value::Float(x - y)),
        (Value::Integer(x), Value::Float(y)) => Some(Value::Float(*x as f64 - y)),
        (Value::Float(x), Value::Integer(y)) => Some(Value::Float(x - *y as f64)),
        _ => None,
    })
}

/// Binary multiplication operation
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn op_mul(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    binary_arithmetic_op(state, |a, b| match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => Some(Value::Integer(x * y)),
        (Value::Float(x), Value::Float(y)) => Some(Value::Float(x * y)),
        (Value::Integer(x), Value::Float(y)) => Some(Value::Float(*x as f64 * y)),
        (Value::Float(x), Value::Integer(y)) => Some(Value::Float(x * *y as f64)),
        _ => None,
    })
}

/// Binary division operation
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
pub fn op_div(state: &mut InterpreterState, _operand: u32) -> InstructionResult {
    binary_arithmetic_op(state, |a, b| match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => {
            if *y == 0 {
                None // Division by zero
            } else {
                Some(Value::Integer(x / y))
            }
        }
        (Value::Float(x), Value::Float(y)) => {
            if *y == 0.0 {
                None // Division by zero
            } else {
                Some(Value::Float(x / y))
            }
        }
        (Value::Integer(x), Value::Float(y)) => {
            if *y == 0.0 {
                None // Division by zero
            } else {
                Some(Value::Float(*x as f64 / y))
            }
        }
        (Value::Float(x), Value::Integer(y)) => {
            if *y == 0 {
                None // Division by zero
            } else {
                Some(Value::Float(x / *y as f64))
            }
        }
        _ => None,
    })
}

/// Helper for binary arithmetic operations
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn binary_arithmetic_op<F>(state: &mut InterpreterState, op: F) -> InstructionResult
where
    F: FnOnce(&Value, &Value) -> Option<Value>,
{
    if state.stack.len() < 2 {
        return InstructionResult::Error(InterpreterError::RuntimeError(
            "Stack underflow in arithmetic operation".to_string(),
        ));
    }

    let right = state.stack.pop().expect("Stack underflow checked above");
    let left = state.stack.pop().expect("Stack underflow checked above");

    match op(&left, &right) {
        Some(result) => {
            state.stack.push(result);
            InstructionResult::Continue
        }
        None => InstructionResult::Error(InterpreterError::RuntimeError(format!(
            "Invalid arithmetic operation between {left:?} and {right:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpreter_state_creation() {
        let state = InterpreterState::new();
        assert_eq!(state.stack.len(), 0);
        assert_eq!(state.constants.len(), 0);
    }

    #[test]
    fn test_direct_threaded_interpreter_creation() {
        let interpreter = DirectThreadedInterpreter::new();
        assert_eq!(interpreter.instruction_count(), 0);
        assert_eq!(interpreter.constants_count(), 0);
    }

    #[test]
    fn test_literal_to_value() {
        let interpreter = DirectThreadedInterpreter::new();

        assert_eq!(
            interpreter.literal_to_value(&Literal::Integer(42, None)),
            Value::Integer(42)
        );
        assert_eq!(
            interpreter.literal_to_value(&Literal::Float(3.15)),
            Value::Float(3.15)
        );
        assert_eq!(
            interpreter.literal_to_value(&Literal::Bool(true)),
            Value::Bool(true)
        );
        assert_eq!(interpreter.literal_to_value(&Literal::Unit), Value::Nil);
        assert_eq!(interpreter.literal_to_value(&Literal::Null), Value::Nil);
    }

    #[test]
    fn test_add_constant() {
        let mut interpreter = DirectThreadedInterpreter::new();
        let idx = interpreter.add_constant(Value::Integer(42));
        assert_eq!(idx, 0);
        assert_eq!(interpreter.constants_count(), 1);
    }

    #[test]
    fn test_op_load_const() {
        let mut state = InterpreterState::new();
        state.constants.push(Value::Integer(42));

        let result = op_load_const(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack.len(), 1);
        assert_eq!(state.stack[0], Value::Integer(42));
    }

    #[test]
    fn test_op_load_nil() {
        let mut state = InterpreterState::new();

        let result = op_load_nil(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack.len(), 1);
        assert_eq!(state.stack[0], Value::Nil);
    }

    #[test]
    fn test_op_return() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Integer(42));

        let result = op_return(&mut state, 0);
        assert!(matches!(
            result,
            InstructionResult::Return(Value::Integer(42))
        ));
    }

    #[test]
    fn test_op_add() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Integer(10));
        state.stack.push(Value::Integer(5));

        let result = op_add(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack.len(), 1);
        assert_eq!(state.stack[0], Value::Integer(15));
    }

    #[test]
    fn test_op_div_by_zero() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Integer(10));
        state.stack.push(Value::Integer(0));

        let result = op_div(&mut state, 0);
        assert!(matches!(result, InstructionResult::Error(_)));
    }

    // Additional coverage tests for COVERAGE-95%

    #[test]
    fn test_op_sub_integers() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Integer(10));
        state.stack.push(Value::Integer(3));

        let result = op_sub(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack[0], Value::Integer(7));
    }

    #[test]
    fn test_op_sub_floats() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Float(10.5));
        state.stack.push(Value::Float(3.5));

        let result = op_sub(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack[0], Value::Float(7.0));
    }

    #[test]
    fn test_op_sub_mixed_int_float() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Integer(10));
        state.stack.push(Value::Float(3.0));

        let result = op_sub(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack[0], Value::Float(7.0));
    }

    #[test]
    fn test_op_sub_mixed_float_int() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Float(10.0));
        state.stack.push(Value::Integer(3));

        let result = op_sub(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack[0], Value::Float(7.0));
    }

    #[test]
    fn test_op_mul_integers() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Integer(6));
        state.stack.push(Value::Integer(7));

        let result = op_mul(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack[0], Value::Integer(42));
    }

    #[test]
    fn test_op_mul_floats() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Float(3.0));
        state.stack.push(Value::Float(4.0));

        let result = op_mul(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack[0], Value::Float(12.0));
    }

    #[test]
    fn test_op_mul_mixed() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Integer(5));
        state.stack.push(Value::Float(2.5));

        let result = op_mul(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack[0], Value::Float(12.5));
    }

    #[test]
    fn test_op_div_integers() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Integer(42));
        state.stack.push(Value::Integer(6));

        let result = op_div(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack[0], Value::Integer(7));
    }

    #[test]
    fn test_op_div_floats() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Float(10.0));
        state.stack.push(Value::Float(4.0));

        let result = op_div(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack[0], Value::Float(2.5));
    }

    #[test]
    fn test_op_div_by_zero_float() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Float(10.0));
        state.stack.push(Value::Float(0.0));

        let result = op_div(&mut state, 0);
        assert!(matches!(result, InstructionResult::Error(_)));
    }

    #[test]
    fn test_op_div_mixed_int_float() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Integer(10));
        state.stack.push(Value::Float(4.0));

        let result = op_div(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack[0], Value::Float(2.5));
    }

    #[test]
    fn test_op_div_mixed_int_float_by_zero() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Integer(10));
        state.stack.push(Value::Float(0.0));

        let result = op_div(&mut state, 0);
        assert!(matches!(result, InstructionResult::Error(_)));
    }

    #[test]
    fn test_op_div_mixed_float_int() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Float(10.0));
        state.stack.push(Value::Integer(4));

        let result = op_div(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack[0], Value::Float(2.5));
    }

    #[test]
    fn test_op_div_mixed_float_int_by_zero() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Float(10.0));
        state.stack.push(Value::Integer(0));

        let result = op_div(&mut state, 0);
        assert!(matches!(result, InstructionResult::Error(_)));
    }

    #[test]
    fn test_op_add_floats() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Float(1.5));
        state.stack.push(Value::Float(2.5));

        let result = op_add(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack[0], Value::Float(4.0));
    }

    #[test]
    fn test_op_add_mixed_int_float() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Integer(1));
        state.stack.push(Value::Float(2.5));

        let result = op_add(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack[0], Value::Float(3.5));
    }

    #[test]
    fn test_op_add_mixed_float_int() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Float(2.5));
        state.stack.push(Value::Integer(1));

        let result = op_add(&mut state, 0);
        assert!(matches!(result, InstructionResult::Continue));
        assert_eq!(state.stack[0], Value::Float(3.5));
    }

    #[test]
    fn test_op_add_invalid_types() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::from_string("hello".to_string()));
        state.stack.push(Value::Integer(5));

        let result = op_add(&mut state, 0);
        assert!(matches!(result, InstructionResult::Error(_)));
    }

    #[test]
    fn test_op_jump() {
        let mut state = InterpreterState::new();
        let result = op_jump(&mut state, 42);
        assert!(matches!(result, InstructionResult::Jump(42)));
    }

    #[test]
    fn test_op_jump_if_false_with_false() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Bool(false));

        let result = op_jump_if_false(&mut state, 10);
        assert!(matches!(result, InstructionResult::Jump(10)));
    }

    #[test]
    fn test_op_jump_if_false_with_nil() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Nil);

        let result = op_jump_if_false(&mut state, 10);
        assert!(matches!(result, InstructionResult::Jump(10)));
    }

    #[test]
    fn test_op_jump_if_false_with_true() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Bool(true));

        let result = op_jump_if_false(&mut state, 10);
        assert!(matches!(result, InstructionResult::Continue));
    }

    #[test]
    fn test_op_jump_if_false_with_integer() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Integer(42));

        let result = op_jump_if_false(&mut state, 10);
        assert!(matches!(result, InstructionResult::Continue));
    }

    #[test]
    fn test_op_jump_if_false_empty_stack() {
        let mut state = InterpreterState::new();
        let result = op_jump_if_false(&mut state, 10);
        assert!(matches!(result, InstructionResult::Error(_)));
    }

    #[test]
    fn test_op_load_const_invalid_index() {
        let mut state = InterpreterState::new();
        let result = op_load_const(&mut state, 999);
        assert!(matches!(result, InstructionResult::Error(_)));
    }

    #[test]
    fn test_op_load_var() {
        let mut state = InterpreterState::new();
        let result = op_load_var(&mut state, 0);
        assert!(matches!(result, InstructionResult::Error(_)));
    }

    #[test]
    fn test_op_ast_fallback_with_string() {
        let mut state = InterpreterState::new();
        state.constants.push(Value::from_string("test_fallback".to_string()));

        let result = op_ast_fallback(&mut state, 0);
        assert!(matches!(result, InstructionResult::Error(_)));
    }

    #[test]
    fn test_op_ast_fallback_invalid_index() {
        let mut state = InterpreterState::new();
        let result = op_ast_fallback(&mut state, 999);
        assert!(matches!(result, InstructionResult::Error(_)));
    }

    #[test]
    fn test_op_return_empty_stack() {
        let mut state = InterpreterState::new();
        let result = op_return(&mut state, 0);
        assert!(matches!(result, InstructionResult::Return(Value::Nil)));
    }

    #[test]
    fn test_binary_arithmetic_op_stack_underflow() {
        let mut state = InterpreterState::new();
        state.stack.push(Value::Integer(5)); // Only one value

        let result = op_add(&mut state, 0);
        assert!(matches!(result, InstructionResult::Error(_)));
    }

    #[test]
    fn test_interpreter_default() {
        let interpreter: DirectThreadedInterpreter = Default::default();
        assert_eq!(interpreter.instruction_count(), 0);
    }

    #[test]
    fn test_interpreter_state_default() {
        let state: InterpreterState = Default::default();
        assert!(state.stack.is_empty());
        assert!(state.constants.is_empty());
    }

    #[test]
    fn test_interpreter_clear() {
        let mut interpreter = DirectThreadedInterpreter::new();
        interpreter.add_instruction(op_load_nil, 0);
        let _ = interpreter.add_constant(Value::Integer(42));
        interpreter.clear();
        assert_eq!(interpreter.instruction_count(), 0);
        assert_eq!(interpreter.constants_count(), 0);
    }

    #[test]
    fn test_threaded_instruction_debug() {
        let instr = ThreadedInstruction {
            handler: op_load_nil,
            operand: 42,
        };
        let debug_str = format!("{:?}", instr);
        assert!(debug_str.contains("operand: 42"));
    }

    #[test]
    fn test_instruction_result_clone() {
        let result = InstructionResult::Continue;
        let cloned = result.clone();
        assert!(matches!(cloned, InstructionResult::Continue));

        let result = InstructionResult::Jump(10);
        let cloned = result.clone();
        assert!(matches!(cloned, InstructionResult::Jump(10)));

        let result = InstructionResult::Return(Value::Integer(42));
        let cloned = result.clone();
        assert!(matches!(cloned, InstructionResult::Return(Value::Integer(42))));
    }

    #[test]
    fn test_literal_to_value_string() {
        let interpreter = DirectThreadedInterpreter::new();
        let result = interpreter.literal_to_value(&Literal::String("hello".to_string()));
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_literal_to_value_char() {
        let interpreter = DirectThreadedInterpreter::new();
        let result = interpreter.literal_to_value(&Literal::Char('x'));
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_literal_to_value_byte() {
        let interpreter = DirectThreadedInterpreter::new();
        let result = interpreter.literal_to_value(&Literal::Byte(255));
        assert_eq!(result, Value::Byte(255));
    }

    #[test]
    fn test_literal_to_value_atom() {
        let interpreter = DirectThreadedInterpreter::new();
        let result = interpreter.literal_to_value(&Literal::Atom("ok".to_string()));
        assert_eq!(result, Value::Atom("ok".to_string()));
    }

    #[test]
    fn test_compile_and_execute_integer_literal() {
        use crate::frontend::ast::Span;

        let mut interpreter = DirectThreadedInterpreter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Integer(42, None)), Span::new(0, 2));

        interpreter.compile(&expr).expect("Compilation should succeed");
        let result = interpreter.execute().expect("Execution should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_compile_and_execute_nil_literal() {
        use crate::frontend::ast::Span;

        let mut interpreter = DirectThreadedInterpreter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Unit), Span::new(0, 2));

        interpreter.compile(&expr).expect("Compilation should succeed");
        let result = interpreter.execute().expect("Execution should succeed");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_compile_identifier() {
        use crate::frontend::ast::Span;

        let mut interpreter = DirectThreadedInterpreter::new();
        let expr = Expr::new(ExprKind::Identifier("x".to_string()), Span::new(0, 1));

        interpreter.compile(&expr).expect("Compilation should succeed");
        // Execution will error because variable lookup isn't implemented
        let result = interpreter.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_empty_code() {
        let mut interpreter = DirectThreadedInterpreter::new();
        let result = interpreter.execute();
        // Empty code should error with PC out of bounds
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_with_state() {
        let mut interpreter = DirectThreadedInterpreter::new();
        interpreter.add_instruction(op_load_nil, 0);
        interpreter.add_instruction(op_return, 0);

        let mut state = InterpreterState::new();
        let result = interpreter.execute_with_state(&mut state);
        assert_eq!(result.unwrap(), Value::Nil);
    }

    #[test]
    fn test_execute_with_state_jump_out_of_bounds() {
        let mut interpreter = DirectThreadedInterpreter::new();
        interpreter.add_instruction(op_jump, 999);

        let mut state = InterpreterState::new();
        let result = interpreter.execute_with_state(&mut state);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_constants() {
        let mut interpreter = DirectThreadedInterpreter::new();
        let idx1 = interpreter.add_constant(Value::Integer(1));
        let idx2 = interpreter.add_constant(Value::Integer(2));
        let idx3 = interpreter.add_constant(Value::Integer(3));

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 2);
        assert_eq!(interpreter.constants_count(), 3);
    }
}
