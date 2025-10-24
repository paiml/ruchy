//! Bytecode compiler - AST to bytecode translation
//!
//! OPT-002: Bytecode Compiler
//!
//! Translates Ruchy AST to bytecode instructions with:
//! - Linear scan register allocation
//! - Constant pool management
//! - Jump target resolution
//! - Local variable tracking
//!
//! Reference: ../ruchyruchy/OPTIMIZATION_REPORT_FOR_RUCHY.md
//! Expected: Efficient bytecode generation with minimal overhead

use super::instruction::Instruction;
use super::opcode::OpCode;
use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, UnaryOp};
use crate::runtime::Value;
use std::collections::HashMap;

/// Bytecode function chunk
///
/// Contains compiled bytecode and associated metadata for a function.
#[derive(Debug, Clone)]
pub struct BytecodeChunk {
    /// Function name (for debugging)
    pub name: String,
    /// Bytecode instructions
    pub instructions: Vec<Instruction>,
    /// Constant pool (literals used in the function)
    pub constants: Vec<Value>,
    /// Number of registers required
    pub register_count: u8,
    /// Number of parameters
    pub parameter_count: u8,
    /// Local variable names (for debugging)
    pub local_names: Vec<String>,
    /// Source line numbers (parallel to instructions for debugging)
    pub line_numbers: Vec<usize>,
}

impl BytecodeChunk {
    /// Create a new empty bytecode chunk
    pub fn new(name: String) -> Self {
        Self {
            name,
            instructions: Vec::new(),
            constants: Vec::new(),
            register_count: 0,
            parameter_count: 0,
            local_names: Vec::new(),
            line_numbers: Vec::new(),
        }
    }

    /// Add an instruction to the chunk
    ///
    /// Returns the index where the instruction was added (for jump patching).
    pub fn emit(&mut self, instruction: Instruction, line: usize) -> usize {
        let index = self.instructions.len();
        self.instructions.push(instruction);
        self.line_numbers.push(line);
        index
    }

    /// Add a constant to the constant pool
    ///
    /// Returns the index of the constant (existing or newly added).
    pub fn add_constant(&mut self, value: Value) -> u16 {
        // Check if constant already exists
        for (i, existing) in self.constants.iter().enumerate() {
            if values_equal(existing, &value) {
                return i as u16;
            }
        }

        // Add new constant
        let index = self.constants.len();
        self.constants.push(value);
        index as u16
    }

    /// Patch a jump instruction at the given index
    ///
    /// Updates the jump offset to point to the current instruction position.
    pub fn patch_jump(&mut self, jump_index: usize) {
        let offset = (self.instructions.len() - jump_index - 1) as i16;
        let instruction = &self.instructions[jump_index];

        // Recreate instruction with patched offset
        let patched = Instruction::asbx(
            OpCode::from_u8(instruction.opcode()).expect("Invalid opcode"),
            instruction.get_a(),
            offset,
        );

        self.instructions[jump_index] = patched;
    }
}

/// Simple register allocator using linear scan
///
/// Tracks which registers are in use and allocates new ones as needed.
#[derive(Debug)]
struct RegisterAllocator {
    /// Number of registers currently allocated
    next_register: u8,
    /// Maximum register count seen (for function metadata)
    max_registers: u8,
    /// Stack of free registers (for reuse)
    free_registers: Vec<u8>,
}

impl RegisterAllocator {
    /// Create a new register allocator
    fn new() -> Self {
        Self {
            next_register: 0,
            max_registers: 0,
            free_registers: Vec::new(),
        }
    }

    /// Allocate a new register
    ///
    /// Returns the register index. Reuses freed registers when possible.
    fn allocate(&mut self) -> u8 {
        if let Some(reg) = self.free_registers.pop() {
            reg
        } else {
            let reg = self.next_register;
            self.next_register += 1;
            self.max_registers = self.max_registers.max(self.next_register);
            reg
        }
    }

    /// Free a register for reuse
    fn free(&mut self, register: u8) {
        self.free_registers.push(register);
    }

    /// Get the maximum number of registers used
    fn max_count(&self) -> u8 {
        self.max_registers
    }
}

/// Bytecode compiler state
///
/// Maintains compilation context including local variables and register allocation.
pub struct Compiler {
    /// Current bytecode chunk being compiled
    chunk: BytecodeChunk,
    /// Register allocator
    registers: RegisterAllocator,
    /// Local variable mapping (name -> register)
    locals: HashMap<String, u8>,
    /// Current scope depth
    scope_depth: usize,
    /// Last result register (for Return instruction)
    last_result: u8,
}

impl Compiler {
    /// Create a new compiler
    pub fn new(function_name: String) -> Self {
        Self {
            chunk: BytecodeChunk::new(function_name),
            registers: RegisterAllocator::new(),
            locals: HashMap::new(),
            scope_depth: 0,
            last_result: 0,
        }
    }

    /// Compile an expression to bytecode
    ///
    /// Returns the register containing the result.
    pub fn compile_expr(&mut self, expr: &Expr) -> Result<u8, String> {
        let result = match &expr.kind {
            ExprKind::Literal(lit) => self.compile_literal(lit),
            ExprKind::Binary { op, left, right } => self.compile_binary(op, left, right),
            ExprKind::Unary { op, operand } => self.compile_unary(op, operand),
            ExprKind::Identifier(name) => self.compile_variable(name),
            ExprKind::Let { name, value, .. } => self.compile_let(name, value),
            ExprKind::Block(exprs) => self.compile_block(exprs),
            ExprKind::If { condition, then_branch, else_branch } => {
                self.compile_if(condition, then_branch, else_branch.as_deref())
            }
            ExprKind::Call { func, args} => self.compile_call(func, args),
            ExprKind::While { condition, body, .. } => self.compile_while(condition, body),
            ExprKind::Assign { target, value } => self.compile_assign(target, value),
            _ => Err(format!("Unsupported expression kind: {:?}", expr.kind)),
        }?;
        self.last_result = result;
        Ok(result)
    }

    /// Compile a literal value
    fn compile_literal(&mut self, literal: &Literal) -> Result<u8, String> {
        let value = match literal {
            Literal::Integer(i, _) => Value::Integer(*i),
            Literal::Float(f) => Value::Float(*f),
            Literal::String(s) => Value::from_string(s.clone()),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Unit | Literal::Null => Value::Nil,
            Literal::Char(c) => Value::from_string(c.to_string()),
            Literal::Byte(b) => Value::Integer(*b as i64),
        };

        let const_index = self.chunk.add_constant(value);
        let result_reg = self.registers.allocate();

        // Emit CONST instruction: R[result] = constants[const_index]
        self.chunk.emit(
            Instruction::abx(OpCode::Const, result_reg, const_index),
            0, // TODO: Track line numbers from AST
        );

        Ok(result_reg)
    }

    /// Compile a binary operation
    fn compile_binary(&mut self, op: &BinaryOp, left: &Expr, right: &Expr) -> Result<u8, String> {
        let left_reg = self.compile_expr(left)?;
        let right_reg = self.compile_expr(right)?;
        let result_reg = self.registers.allocate();

        let opcode = match op {
            BinaryOp::Add => OpCode::Add,
            BinaryOp::Subtract => OpCode::Sub,
            BinaryOp::Multiply => OpCode::Mul,
            BinaryOp::Divide => OpCode::Div,
            BinaryOp::Modulo => OpCode::Mod,
            BinaryOp::Equal => OpCode::Equal,
            BinaryOp::NotEqual => OpCode::NotEqual,
            BinaryOp::Greater | BinaryOp::Gt => OpCode::Greater,
            BinaryOp::GreaterEqual => OpCode::GreaterEqual,
            BinaryOp::Less => OpCode::Less,
            BinaryOp::LessEqual => OpCode::LessEqual,
            BinaryOp::And => OpCode::And,
            BinaryOp::Or => OpCode::Or,
            BinaryOp::BitwiseAnd => OpCode::BitAnd,
            BinaryOp::BitwiseOr => OpCode::BitOr,
            BinaryOp::BitwiseXor => OpCode::BitXor,
            BinaryOp::LeftShift => OpCode::ShiftLeft,
            BinaryOp::RightShift => OpCode::ShiftRight,
            _ => return Err(format!("Unsupported binary operator: {:?}", op)),
        };

        // Emit binary operation: R[result] = R[left] op R[right]
        self.chunk.emit(
            Instruction::abc(opcode, result_reg, left_reg, right_reg),
            0,
        );

        // Free input registers
        self.registers.free(left_reg);
        self.registers.free(right_reg);

        Ok(result_reg)
    }

    /// Compile a unary operation
    fn compile_unary(&mut self, op: &UnaryOp, operand: &Expr) -> Result<u8, String> {
        let operand_reg = self.compile_expr(operand)?;
        let result_reg = self.registers.allocate();

        let opcode = match op {
            UnaryOp::Negate => OpCode::Neg,
            UnaryOp::Not => OpCode::Not,
            UnaryOp::BitwiseNot => OpCode::BitNot,
            UnaryOp::Reference | UnaryOp::Deref => {
                return Err(format!("Unsupported unary operator: {:?}", op));
            }
        };

        // Emit unary operation: R[result] = op R[operand]
        // Using AB format: A = result, B = operand
        self.chunk.emit(
            Instruction::abc(opcode, result_reg, operand_reg, 0),
            0,
        );

        // Free input register
        self.registers.free(operand_reg);

        Ok(result_reg)
    }

    /// Compile a variable reference
    fn compile_variable(&mut self, name: &str) -> Result<u8, String> {
        if let Some(&var_reg) = self.locals.get(name) {
            // Local variable - copy to temporary register
            // This prevents compile_binary() from freeing the variable's register
            let temp_reg = self.registers.allocate();
            self.chunk.emit(
                Instruction::abc(OpCode::Move, temp_reg, var_reg, 0),
                0,
            );
            Ok(temp_reg)
        } else {
            // Global variable - need to load from global table
            let name_const = self.chunk.add_constant(Value::from_string(name.to_string()));
            let result_reg = self.registers.allocate();

            self.chunk.emit(
                Instruction::abx(OpCode::LoadGlobal, result_reg, name_const),
                0,
            );

            Ok(result_reg)
        }
    }

    /// Compile a let binding
    fn compile_let(&mut self, name: &str, value: &Expr) -> Result<u8, String> {
        let value_reg = self.compile_expr(value)?;

        // Store in locals table
        self.locals.insert(name.to_string(), value_reg);
        self.chunk.local_names.push(name.to_string());

        Ok(value_reg)
    }

    /// Compile a block expression
    ///
    /// Returns the register containing the result of the last expression.
    fn compile_block(&mut self, exprs: &[Expr]) -> Result<u8, String> {
        if exprs.is_empty() {
            // Empty block returns nil
            return self.compile_literal(&Literal::Unit);
        }

        // Compile all expressions, free intermediate registers
        let mut last_reg = 0;
        for (i, expr) in exprs.iter().enumerate() {
            if i > 0 {
                // Free previous result (except the last one)
                // But DON'T free if it's a local variable's register
                if !self.is_local_register(last_reg) {
                    self.registers.free(last_reg);
                }
            }
            last_reg = self.compile_expr(expr)?;
        }

        Ok(last_reg)
    }

    /// Check if a register is used by a local variable
    fn is_local_register(&self, reg: u8) -> bool {
        self.locals.values().any(|&r| r == reg)
    }

    /// Compile an if expression
    ///
    /// Generates conditional branching bytecode with optional else branch.
    fn compile_if(
        &mut self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) -> Result<u8, String> {
        let result_reg = self.registers.allocate();

        // Compile condition
        let cond_reg = self.compile_expr(condition)?;

        // Emit conditional jump: if !R[cond] jump to else/end
        let jump_to_else = self.chunk.emit(
            Instruction::asbx(OpCode::JumpIfFalse, cond_reg, 0),
            0,
        );
        self.registers.free(cond_reg);

        // Compile then branch
        let then_reg = self.compile_expr(then_branch)?;
        // Move result to result register
        self.chunk.emit(
            Instruction::abc(OpCode::Move, result_reg, then_reg, 0),
            0,
        );
        self.registers.free(then_reg);

        if let Some(else_expr) = else_branch {
            // Emit jump to skip else branch
            let jump_to_end = self.chunk.emit(
                Instruction::asbx(OpCode::Jump, 0, 0),
                0,
            );

            // Patch jump to else
            self.chunk.patch_jump(jump_to_else);

            // Compile else branch
            let else_reg = self.compile_expr(else_expr)?;
            self.chunk.emit(
                Instruction::abc(OpCode::Move, result_reg, else_reg, 0),
                0,
            );
            self.registers.free(else_reg);

            // Patch jump to end
            self.chunk.patch_jump(jump_to_end);
        } else {
            // No else branch - result is nil if condition is false
            // Patch jump to end (just after then branch)
            self.chunk.patch_jump(jump_to_else);
        }

        Ok(result_reg)
    }

    /// Compile a while loop
    ///
    /// Generates: loop_start → check condition → if false jump to end → body → jump to start → loop_end
    fn compile_while(&mut self, condition: &Expr, body: &Expr) -> Result<u8, String> {
        let result_reg = self.registers.allocate();

        // Mark loop start position
        let loop_start = self.chunk.instructions.len();

        // Compile condition
        let cond_reg = self.compile_expr(condition)?;

        // Emit conditional jump: if !condition, jump to loop end
        let jump_to_end = self.chunk.emit(
            Instruction::asbx(OpCode::JumpIfFalse, cond_reg, 0),
            0,
        );
        self.registers.free(cond_reg);

        // Compile body
        let body_reg = self.compile_expr(body)?;
        self.registers.free(body_reg);

        // Emit backward jump to loop start
        let offset = -((self.chunk.instructions.len() - loop_start + 1) as i16);
        self.chunk.emit(
            Instruction::asbx(OpCode::Jump, 0, offset),
            0,
        );

        // Patch forward jump to end
        self.chunk.patch_jump(jump_to_end);

        // While loops return nil
        let nil_const = self.chunk.add_constant(Value::Nil);
        self.chunk.emit(
            Instruction::abx(OpCode::Const, result_reg, nil_const),
            0,
        );

        Ok(result_reg)
    }

    /// Compile an assignment expression
    ///
    /// Generates: value → register, Move to target register
    /// Returns the value register (assignment is an expression that returns the assigned value)
    fn compile_assign(&mut self, target: &Expr, value: &Expr) -> Result<u8, String> {
        // For now, only support simple identifier assignments
        match &target.kind {
            ExprKind::Identifier(name) => {
                // Look up the variable's register
                let target_reg = if let Some(&reg) = self.locals.get(name) {
                    reg
                } else {
                    return Err(format!("Undefined variable: {}", name));
                };

                // Compile the value expression
                let value_reg = self.compile_expr(value)?;

                // Move value to target register
                self.chunk.emit(
                    Instruction::abc(OpCode::Move, target_reg, value_reg, 0),
                    0,
                );

                // Free temporary value register if different from target
                if value_reg != target_reg {
                    self.registers.free(value_reg);
                }

                // Assignment returns the assigned value
                Ok(target_reg)
            }
            _ => Err(format!("Unsupported assignment target: {:?}", target.kind)),
        }
    }

    /// Compile a function call
    fn compile_call(&mut self, func: &Expr, args: &[Expr]) -> Result<u8, String> {
        let result_reg = self.registers.allocate();

        // Compile function expression
        let func_reg = self.compile_expr(func)?;

        // Compile arguments
        let mut arg_regs = Vec::new();
        for arg in args {
            let arg_reg = self.compile_expr(arg)?;
            arg_regs.push(arg_reg);
        }

        // Emit call instruction: R[result] = call R[func](R[arg0], R[arg1], ...)
        // Using ABC format: A = result, B = func, C = arg_count
        self.chunk.emit(
            Instruction::abc(OpCode::Call, result_reg, func_reg, args.len() as u8),
            0,
        );

        // Free function and argument registers
        self.registers.free(func_reg);
        for arg_reg in arg_regs {
            self.registers.free(arg_reg);
        }

        Ok(result_reg)
    }

    /// Finalize compilation and return the bytecode chunk
    pub fn finalize(mut self) -> BytecodeChunk {
        // Emit return instruction with the last result register
        self.chunk.emit(Instruction::abc(OpCode::Return, self.last_result, 0, 0), 0);

        // Update register count
        self.chunk.register_count = self.registers.max_count();

        self.chunk
    }
}

/// Compare two values for equality (for constant pool deduplication)
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Nil, Value::Nil) => true,
        // String comparison by reference (interned strings would be ideal)
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_pool_deduplication() {
        let mut chunk = BytecodeChunk::new("test".to_string());

        let idx1 = chunk.add_constant(Value::Integer(42));
        let idx2 = chunk.add_constant(Value::Integer(42));
        let idx3 = chunk.add_constant(Value::Integer(100));

        assert_eq!(idx1, idx2, "Duplicate constants should return same index");
        assert_ne!(idx1, idx3, "Different constants should have different indices");
        assert_eq!(chunk.constants.len(), 2, "Should only store 2 unique constants");
    }

    #[test]
    fn test_register_allocator_basic() {
        let mut allocator = RegisterAllocator::new();

        let r0 = allocator.allocate();
        let r1 = allocator.allocate();
        let r2 = allocator.allocate();

        assert_eq!(r0, 0);
        assert_eq!(r1, 1);
        assert_eq!(r2, 2);
        assert_eq!(allocator.max_count(), 3);
    }

    #[test]
    fn test_register_allocator_reuse() {
        let mut allocator = RegisterAllocator::new();

        let r0 = allocator.allocate();
        let _r1 = allocator.allocate();

        allocator.free(r0);
        let r2 = allocator.allocate();

        assert_eq!(r2, r0, "Should reuse freed register");
        assert_eq!(allocator.max_count(), 2, "Max count shouldn't change");
    }

    #[test]
    fn test_compile_integer_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert_eq!(result_reg, 0, "First expression should use register 0");
        assert_eq!(chunk.constants.len(), 1, "Should have 1 constant");
        assert_eq!(chunk.instructions.len(), 2, "Should have CONST + RETURN instructions");

        // Verify CONST instruction
        let const_instr = chunk.instructions[0];
        assert_eq!(const_instr.opcode(), OpCode::Const.to_u8());
        assert_eq!(const_instr.get_a(), 0, "Should load into register 0");
        assert_eq!(const_instr.get_bx(), 0, "Should load constant at index 0");
    }

    #[test]
    fn test_compile_binary_addition() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(32, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have: CONST (10), CONST (32), ADD, RETURN
        assert_eq!(chunk.instructions.len(), 4);
        assert_eq!(chunk.constants.len(), 2);

        // Verify ADD instruction
        let add_instr = chunk.instructions[2];
        assert_eq!(add_instr.opcode(), OpCode::Add.to_u8());
        assert_eq!(add_instr.get_a(), result_reg, "Result register");
    }

    #[test]
    fn test_compile_block() {
        let mut compiler = Compiler::new("test".to_string());

        // Block with 3 expressions: 1, 2, 3
        let exprs = vec![
            Expr::new(ExprKind::Literal(Literal::Integer(1, None)), crate::frontend::ast::Span::default()),
            Expr::new(ExprKind::Literal(Literal::Integer(2, None)), crate::frontend::ast::Span::default()),
            Expr::new(ExprKind::Literal(Literal::Integer(3, None)), crate::frontend::ast::Span::default()),
        ];
        let block = Expr::new(ExprKind::Block(exprs), crate::frontend::ast::Span::default());

        let result_reg = compiler.compile_expr(&block).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have: CONST(1), CONST(2), CONST(3), RETURN
        assert_eq!(chunk.instructions.len(), 4);
        assert_eq!(chunk.constants.len(), 3, "Should have 3 constants");

        // Result should be the last expression (3)
        assert!(result_reg < 10, "Result register should be valid");
    }

    #[test]
    fn test_compile_if_with_else() {
        let mut compiler = Compiler::new("test".to_string());

        // if true { 42 } else { 0 }
        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::default(),
        );
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );
        let else_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            crate::frontend::ast::Span::default(),
        );

        let if_expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Some(Box::new(else_branch)),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&if_expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have:
        // CONST (true), JUMP_IF_FALSE, CONST (42), MOVE, JUMP, CONST (0), MOVE, RETURN
        assert!(chunk.instructions.len() >= 7, "Should have at least 7 instructions");
        assert_eq!(chunk.constants.len(), 3, "Should have 3 constants");

        // Verify conditional jump exists
        let jump_if_false_found = chunk.instructions.iter()
            .any(|i| i.opcode() == OpCode::JumpIfFalse.to_u8());
        assert!(jump_if_false_found, "Should have JumpIfFalse instruction");

        assert!(result_reg < 10, "Result register should be valid");
    }

    #[test]
    fn test_compile_if_without_else() {
        let mut compiler = Compiler::new("test".to_string());

        // if true { 42 }
        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::default(),
        );
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );

        let if_expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: None,
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&if_expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have: CONST (true), JUMP_IF_FALSE, CONST (42), MOVE, RETURN
        assert!(chunk.instructions.len() >= 4, "Should have at least 4 instructions");

        // Verify conditional jump exists
        let jump_if_false_found = chunk.instructions.iter()
            .any(|i| i.opcode() == OpCode::JumpIfFalse.to_u8());
        assert!(jump_if_false_found, "Should have JumpIfFalse instruction");

        assert!(result_reg < 10, "Result register should be valid");
    }

    #[test]
    fn test_compile_function_call() {
        let mut compiler = Compiler::new("test".to_string());

        // foo(1, 2)
        let func = Expr::new(
            ExprKind::Identifier("foo".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let arg1 = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            crate::frontend::ast::Span::default(),
        );
        let arg2 = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            crate::frontend::ast::Span::default(),
        );

        let call = Expr::new(
            ExprKind::Call {
                func: Box::new(func),
                args: vec![arg1, arg2],
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&call).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have: LOAD_GLOBAL(foo), CONST(1), CONST(2), CALL, RETURN
        assert!(chunk.instructions.len() >= 5, "Should have at least 5 instructions");

        // Verify CALL instruction exists
        let call_found = chunk.instructions.iter()
            .any(|i| i.opcode() == OpCode::Call.to_u8());
        assert!(call_found, "Should have Call instruction");

        assert!(result_reg < 10, "Result register should be valid");
    }
}
