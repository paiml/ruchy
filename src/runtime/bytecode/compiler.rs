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
//! Reference: ../`ruchyruchy/OPTIMIZATION_REPORT_FOR_RUCHY.md`
//! Expected: Efficient bytecode generation with minimal overhead

use super::instruction::Instruction;
use super::opcode::OpCode;
use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Param, UnaryOp};
use crate::runtime::Value;
use std::cell::RefCell; // ISSUE-119: For shared mutable environment
use std::collections::HashMap;
use std::rc::Rc; // ISSUE-119: For shared mutable environment
use std::sync::Arc;

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
    /// Loop bodies (for hybrid execution - OPT-012)
    /// Stores AST bodies for for-loops to enable interpreter delegation
    pub loop_bodies: Vec<Arc<Expr>>,
    /// Method calls (for hybrid execution - OPT-014)
    /// Stores AST for method calls to enable interpreter delegation
    /// Each entry: (`receiver_expr`, `method_name`, `args_exprs`)
    pub method_calls: Vec<(Arc<Expr>, String, Vec<Arc<Expr>>)>,
    /// Match expressions (for hybrid execution - OPT-018)
    /// Stores AST for match expressions to enable interpreter delegation
    /// Each entry: (`match_expr`, `match_arms`)
    pub match_exprs: Vec<(Arc<Expr>, Vec<crate::frontend::ast::MatchArm>)>,
    /// Closures (for hybrid execution - OPT-019)
    /// Stores AST for closures to enable interpreter delegation
    /// Each entry: (`params_with_defaults`, body) - environment captured at runtime
    /// RUNTIME-DEFAULT-PARAMS: Params now include default values
    pub closures: Vec<(Vec<(String, Option<Arc<Expr>>)>, Arc<Expr>)>,
    /// Array element registers (for runtime array construction - OPT-020)
    /// Stores register lists for `NewArray` opcodes (element registers may not be contiguous)
    pub array_element_regs: Vec<Vec<u8>>,
    /// Object field data (for runtime object construction - OPT-020)
    /// Stores (`key`, `value_register`) pairs for `NewObject` opcodes
    pub object_fields: Vec<Vec<(String, u8)>>,
    /// Local variable name to register mapping (for hybrid execution)
    /// Enables synchronization between bytecode registers and interpreter scope
    pub locals_map: HashMap<String, u8>,
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
            loop_bodies: Vec::new(),
            method_calls: Vec::new(),
            match_exprs: Vec::new(),
            closures: Vec::new(),
            array_element_regs: Vec::new(),
            object_fields: Vec::new(),
            locals_map: HashMap::new(),
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
            ExprKind::Let {
                name, value, body, ..
            } => self.compile_let(name, value, body),
            ExprKind::Block(exprs) => self.compile_block(exprs),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.compile_if(condition, then_branch, else_branch.as_deref()),
            ExprKind::Call { func, args } => self.compile_call(func, args),
            ExprKind::While {
                condition, body, ..
            } => self.compile_while(condition, body),
            ExprKind::Assign { target, value } => self.compile_assign(target, value),
            ExprKind::Function {
                name, params, body, ..
            } => self.compile_function(name, params, body),
            ExprKind::List(elements) => self.compile_list(elements),
            ExprKind::Tuple(elements) => self.compile_tuple(elements),
            ExprKind::ObjectLiteral { fields } => self.compile_object_literal(fields),
            ExprKind::For {
                var, iter, body, ..
            } => self.compile_for(var, iter, body),
            ExprKind::IndexAccess { object, index } => self.compile_index_access(object, index),
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } => self.compile_method_call(receiver, method, args),
            ExprKind::FieldAccess { object, field } => self.compile_field_access(object, field),
            ExprKind::Match { expr, arms } => self.compile_match(expr, arms),
            ExprKind::Lambda { params, body } => self.compile_closure(params, body),
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
            Literal::Byte(b) => Value::Integer(i64::from(*b)),
            Literal::Atom(s) => Value::Atom(s.clone()),
        };

        let const_index = self.chunk.add_constant(value);
        let result_reg = self.registers.allocate();

        // Emit CONST instruction: R[result] = constants[const_index]
        self.chunk.emit(
            Instruction::abx(OpCode::Const, result_reg, const_index),
            0, // Line number placeholder (debug info not yet tracked)
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
            _ => return Err(format!("Unsupported binary operator: {op:?}")),
        };

        // Emit binary operation: R[result] = R[left] op R[right]
        self.chunk
            .emit(Instruction::abc(opcode, result_reg, left_reg, right_reg), 0);

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
            UnaryOp::Reference | UnaryOp::MutableReference | UnaryOp::Deref => {
                // PARSER-085: Issue #71 - Added MutableReference
                return Err(format!("Unsupported unary operator: {op:?}"));
            }
        };

        // Emit unary operation: R[result] = op R[operand]
        // Using AB format: A = result, B = operand
        self.chunk
            .emit(Instruction::abc(opcode, result_reg, operand_reg, 0), 0);

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
            self.chunk
                .emit(Instruction::abc(OpCode::Move, temp_reg, var_reg, 0), 0);
            Ok(temp_reg)
        } else {
            // Global variable - need to load from global table
            let name_const = self
                .chunk
                .add_constant(Value::from_string(name.to_string()));
            let result_reg = self.registers.allocate();

            self.chunk.emit(
                Instruction::abx(OpCode::LoadGlobal, result_reg, name_const),
                0,
            );

            Ok(result_reg)
        }
    }

    /// Compile a let binding
    ///
    /// Stores the value in a register, records it in the locals table,
    /// then compiles and returns the body expression.
    fn compile_let(&mut self, name: &str, value: &Expr, body: &Expr) -> Result<u8, String> {
        let value_reg = self.compile_expr(value)?;

        // Store in locals table
        self.locals.insert(name.to_string(), value_reg);
        self.chunk.local_names.push(name.to_string());

        // Compile and return the body expression result
        self.compile_expr(body)
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
        let jump_to_else = self
            .chunk
            .emit(Instruction::asbx(OpCode::JumpIfFalse, cond_reg, 0), 0);
        self.registers.free(cond_reg);

        // Compile then branch
        let then_reg = self.compile_expr(then_branch)?;
        // Move result to result register
        self.chunk
            .emit(Instruction::abc(OpCode::Move, result_reg, then_reg, 0), 0);
        self.registers.free(then_reg);

        if let Some(else_expr) = else_branch {
            // Emit jump to skip else branch
            let jump_to_end = self.chunk.emit(Instruction::asbx(OpCode::Jump, 0, 0), 0);

            // Patch jump to else
            self.chunk.patch_jump(jump_to_else);

            // Compile else branch
            let else_reg = self.compile_expr(else_expr)?;
            self.chunk
                .emit(Instruction::abc(OpCode::Move, result_reg, else_reg, 0), 0);
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
    /// Generates: `loop_start` → check condition → if false jump to end → body → jump to start → `loop_end`
    fn compile_while(&mut self, condition: &Expr, body: &Expr) -> Result<u8, String> {
        let result_reg = self.registers.allocate();

        // Mark loop start position
        let loop_start = self.chunk.instructions.len();

        // Compile condition
        let cond_reg = self.compile_expr(condition)?;

        // Emit conditional jump: if !condition, jump to loop end
        let jump_to_end = self
            .chunk
            .emit(Instruction::asbx(OpCode::JumpIfFalse, cond_reg, 0), 0);
        self.registers.free(cond_reg);

        // Compile body
        let body_reg = self.compile_expr(body)?;
        self.registers.free(body_reg);

        // Emit backward jump to loop start
        let offset = -((self.chunk.instructions.len() - loop_start + 1) as i16);
        self.chunk
            .emit(Instruction::asbx(OpCode::Jump, 0, offset), 0);

        // Patch forward jump to end
        self.chunk.patch_jump(jump_to_end);

        // While loops return nil
        let nil_const = self.chunk.add_constant(Value::Nil);
        self.chunk
            .emit(Instruction::abx(OpCode::Const, result_reg, nil_const), 0);

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
                    return Err(format!("Undefined variable: {name}"));
                };

                // Compile the value expression
                let value_reg = self.compile_expr(value)?;

                // Move value to target register
                self.chunk
                    .emit(Instruction::abc(OpCode::Move, target_reg, value_reg, 0), 0);

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
        // OPT-011: Simplified calling convention for hybrid execution
        // Store function and arguments in separate registers, VM will extract them
        let result_reg = self.registers.allocate();

        // Compile function expression
        let func_reg = self.compile_expr(func)?;

        // Compile arguments to temporary registers
        let mut arg_regs = Vec::new();
        for arg in args {
            let arg_reg = self.compile_expr(arg)?;
            arg_regs.push(arg_reg);
        }

        // Store func_reg and argument registers in chunk constants for VM to access
        // Format: [func_reg, arg_reg1, arg_reg2, ...]
        // This is a workaround until we implement proper bytecode calling convention
        let mut call_info = vec![Value::Integer(i64::from(func_reg))];
        call_info.extend(arg_regs.iter().map(|&r| Value::Integer(i64::from(r))));
        let call_info_value = Value::from_array(call_info);
        let call_info_idx = self.chunk.add_constant(call_info_value);

        // Emit call instruction: R[result] = call with info from constants[call_info_idx]
        // ABx format: A = result, Bx = call_info constant index
        self.chunk
            .emit(Instruction::abx(OpCode::Call, result_reg, call_info_idx), 0);

        // OPT-011: Don't free func_reg or arg_regs yet - they contain values needed at runtime
        // DESIGN DECISION: Current register allocation accepts some register pressure to ensure correctness.
        // Future optimization: Implement precise lifetime analysis for register reuse (see OPT-011).

        Ok(result_reg)
    }

    /// Compile a function definition
    ///
    /// Creates a closure and stores it in locals for later invocation.
    fn compile_function(
        &mut self,
        name: &str,
        params: &[Param],
        body: &Expr,
    ) -> Result<u8, String> {
        // RUNTIME-DEFAULT-PARAMS: Extract both param names AND default values
        let params_with_defaults: Vec<(String, Option<Arc<Expr>>)> = params
            .iter()
            .map(|p| {
                (
                    p.name(),
                    p.default_value
                        .clone()
                        .map(|expr| Arc::new((*expr).clone())),
                )
            })
            .collect();

        // Create closure value
        // Note: Using empty environment for now. Full lexical scoping will be added later.
        let closure = Value::Closure {
            params: params_with_defaults,
            body: Arc::new(body.clone()),
            env: Rc::new(RefCell::new(HashMap::new())), // ISSUE-119: Wrap in Rc<RefCell>
        };

        // Add closure to constant pool
        let const_index = self.chunk.add_constant(closure);

        // Allocate register for the closure
        let closure_reg = self.registers.allocate();

        // Emit CONST instruction to load closure into register
        self.chunk
            .emit(Instruction::abx(OpCode::Const, closure_reg, const_index), 0);

        // Store function in locals table for later retrieval
        self.locals.insert(name.to_string(), closure_reg);
        self.chunk.local_names.push(name.to_string());

        Ok(closure_reg)
    }

    /// Compile a list/array literal
    ///
    /// OPT-020: Support both literal and non-literal elements
    /// - Literals: Compile to constant pool (optimization)
    /// - Non-literals: Compile elements to registers, emit `NewArray` opcode
    fn compile_list(&mut self, elements: &[Expr]) -> Result<u8, String> {
        // Check if all elements are literals (can optimize)
        let all_literals = elements
            .iter()
            .all(|elem| matches!(&elem.kind, ExprKind::Literal(_)));

        if all_literals && !elements.is_empty() {
            // Optimization: Create array at compile-time in constant pool
            let mut element_values = Vec::new();
            for elem in elements {
                if let ExprKind::Literal(lit) = &elem.kind {
                    let value = match lit {
                        Literal::Integer(i, _) => Value::Integer(*i),
                        Literal::Float(f) => Value::Float(*f),
                        Literal::String(s) => Value::from_string(s.clone()),
                        Literal::Bool(b) => Value::Bool(*b),
                        Literal::Unit | Literal::Null => Value::Nil,
                        Literal::Char(c) => Value::from_string(c.to_string()),
                        Literal::Byte(b) => Value::Integer(i64::from(*b)),
                        Literal::Atom(s) => Value::Atom(s.clone()),
                    };
                    element_values.push(value);
                }
            }

            let array_value = Value::from_array(element_values);
            let const_index = self.chunk.add_constant(array_value);

            let result_reg = self.registers.allocate();
            self.chunk
                .emit(Instruction::abx(OpCode::Const, result_reg, const_index), 0);
            Ok(result_reg)
        } else {
            // Runtime array construction: compile elements to registers
            let mut element_regs = Vec::new();
            for elem in elements {
                let elem_reg = self.compile_expr(elem)?;
                element_regs.push(elem_reg);
            }

            // Store element registers in chunk (element registers may not be contiguous)
            let element_regs_idx = self.chunk.array_element_regs.len();
            self.chunk.array_element_regs.push(element_regs);

            // Allocate destination register
            let result_reg = self.registers.allocate();

            // Emit NewArray instruction: result = new Array(elements...)
            // Format: NewArray result_reg, element_regs_idx (stored in chunk)
            // B field holds the index into chunk.array_element_regs
            self.chunk.emit(
                Instruction::abx(OpCode::NewArray, result_reg, element_regs_idx as u16),
                0,
            );

            Ok(result_reg)
        }
    }

    /// Compile a tuple literal
    ///
    /// OPT-020: Support both literal and non-literal elements
    /// - Literals: Compile to constant pool (optimization)
    /// - Non-literals: Compile elements to registers, emit `NewTuple` opcode
    fn compile_tuple(&mut self, elements: &[Expr]) -> Result<u8, String> {
        // Check if all elements are literals (can optimize)
        let all_literals = elements
            .iter()
            .all(|elem| matches!(&elem.kind, ExprKind::Literal(_)));

        if all_literals && !elements.is_empty() {
            // Optimization: Create tuple at compile-time in constant pool
            let mut element_values = Vec::new();
            for elem in elements {
                if let ExprKind::Literal(lit) = &elem.kind {
                    let value = match lit {
                        Literal::Integer(i, _) => Value::Integer(*i),
                        Literal::Float(f) => Value::Float(*f),
                        Literal::String(s) => Value::from_string(s.clone()),
                        Literal::Bool(b) => Value::Bool(*b),
                        Literal::Unit | Literal::Null => Value::Nil,
                        Literal::Char(c) => Value::from_string(c.to_string()),
                        Literal::Byte(b) => Value::Integer(i64::from(*b)),
                        Literal::Atom(s) => Value::Atom(s.clone()),
                    };
                    element_values.push(value);
                }
            }

            let tuple_value = Value::Tuple(Arc::from(element_values.as_slice()));
            let const_index = self.chunk.add_constant(tuple_value);

            let result_reg = self.registers.allocate();
            self.chunk
                .emit(Instruction::abx(OpCode::Const, result_reg, const_index), 0);
            Ok(result_reg)
        } else {
            // Runtime tuple construction: compile elements to registers
            let mut element_regs = Vec::new();
            for elem in elements {
                let elem_reg = self.compile_expr(elem)?;
                element_regs.push(elem_reg);
            }

            // Store element registers in chunk (reuse array_element_regs for tuples)
            let element_regs_idx = self.chunk.array_element_regs.len();
            self.chunk.array_element_regs.push(element_regs);

            // Allocate destination register
            let result_reg = self.registers.allocate();

            // Emit NewTuple instruction: result = new Tuple(elements...)
            // Format: NewTuple result_reg, element_regs_idx (stored in chunk)
            self.chunk.emit(
                Instruction::abx(OpCode::NewTuple, result_reg, element_regs_idx as u16),
                0,
            );

            Ok(result_reg)
        }
    }

    /// Compile an object literal
    ///
    /// OPT-020: Support both literal and non-literal field values
    /// - All literals: Compile to constant pool (optimization)
    /// - Non-literals: Compile values to registers, emit `NewObject` opcode
    fn compile_object_literal(
        &mut self,
        fields: &[crate::frontend::ast::ObjectField],
    ) -> Result<u8, String> {
        use crate::frontend::ast::ObjectField;
        use std::collections::HashMap;

        // Check if all field values are literals (can optimize)
        let all_literals = fields.iter().all(|field| {
            match field {
                ObjectField::KeyValue { value, .. } => matches!(&value.kind, ExprKind::Literal(_)),
                ObjectField::Spread { .. } => false, // Spread not supported yet
            }
        });

        if all_literals && !fields.is_empty() {
            // Optimization: Create object at compile-time in constant pool
            let mut object_map = HashMap::new();
            for field in fields {
                if let ObjectField::KeyValue { key, value } = field {
                    if let ExprKind::Literal(lit) = &value.kind {
                        let val = match lit {
                            Literal::Integer(i, _) => Value::Integer(*i),
                            Literal::Float(f) => Value::Float(*f),
                            Literal::String(s) => Value::from_string(s.clone()),
                            Literal::Bool(b) => Value::Bool(*b),
                            Literal::Unit | Literal::Null => Value::Nil,
                            Literal::Char(c) => Value::from_string(c.to_string()),
                            Literal::Byte(b) => Value::Integer(i64::from(*b)),
                            Literal::Atom(s) => Value::Atom(s.clone()),
                        };
                        object_map.insert(key.clone(), val);
                    }
                }
            }

            let object_value = Value::Object(Arc::new(object_map));
            let const_index = self.chunk.add_constant(object_value);

            let result_reg = self.registers.allocate();
            self.chunk
                .emit(Instruction::abx(OpCode::Const, result_reg, const_index), 0);
            Ok(result_reg)
        } else {
            // Runtime object construction: compile field values to registers
            let mut field_data = Vec::new();
            for field in fields {
                match field {
                    ObjectField::KeyValue { key, value } => {
                        let value_reg = self.compile_expr(value)?;
                        field_data.push((key.clone(), value_reg));
                    }
                    ObjectField::Spread { .. } => {
                        return Err(
                            "Spread operator in object literals not yet supported in bytecode mode"
                                .to_string(),
                        );
                    }
                }
            }

            // Store field data in chunk
            let field_data_idx = self.chunk.object_fields.len();
            self.chunk.object_fields.push(field_data);

            // Allocate destination register
            let result_reg = self.registers.allocate();

            // Emit NewObject instruction: result = new Object(fields...)
            // Format: NewObject result_reg, field_data_idx (stored in chunk)
            self.chunk.emit(
                Instruction::abx(OpCode::NewObject, result_reg, field_data_idx as u16),
                0,
            );

            Ok(result_reg)
        }
    }

    /// Compile array indexing: arr[index]
    ///
    /// OPT-013: Array indexing using `LoadIndex` opcode
    /// Compiles both the array and index expressions, then emits `LoadIndex`
    fn compile_index_access(&mut self, object: &Expr, index: &Expr) -> Result<u8, String> {
        // Compile the array/object expression
        let object_reg = self.compile_expr(object)?;

        // Compile the index expression
        let index_reg = self.compile_expr(index)?;

        // Allocate result register
        let result_reg = self.registers.allocate();

        // Emit LoadIndex instruction: result = object[index]
        // Format: LoadIndex result_reg, object_reg, index_reg
        self.chunk.emit(
            Instruction::abc(OpCode::LoadIndex, result_reg, object_reg, index_reg),
            0,
        );

        Ok(result_reg)
    }

    /// Compile a for-loop
    ///
    /// OPT-012: Hybrid approach - like function calls, delegate to interpreter
    /// For-loops require array iteration which is complex to compile,
    /// so we store loop info and let the VM execute via interpreter.
    fn compile_for(&mut self, var: &str, iter: &Expr, body: &Expr) -> Result<u8, String> {
        // Compile iterator expression (the array/collection)
        let iter_reg = self.compile_expr(iter)?;

        // Store body AST in chunk's loop_bodies for interpreter access
        let body_idx = self.chunk.loop_bodies.len();
        self.chunk.loop_bodies.push(Arc::new(body.clone()));

        // Store for-loop metadata in constant pool
        // Format: [iter_reg, var_name, body_index]
        let loop_info = vec![
            Value::Integer(i64::from(iter_reg)), // Register holding the iterator
            Value::from_string(var.to_string()), // Loop variable name
            Value::Integer(body_idx as i64),     // Index into chunk.loop_bodies
        ];
        let loop_info_value = Value::from_array(loop_info);
        let loop_info_idx = self.chunk.add_constant(loop_info_value);

        // Allocate result register
        let result_reg = self.registers.allocate();

        // Emit For instruction: ABx format
        // A = result register, Bx = loop_info constant index
        self.chunk
            .emit(Instruction::abx(OpCode::For, result_reg, loop_info_idx), 0);

        Ok(result_reg)
    }

    /// Compile a method call
    ///
    /// OPT-014: Hybrid approach - delegate to interpreter like for-loops
    /// Method calls require complex dispatch logic (stdlib, mutating methods, `DataFrame`, Actor),
    /// so we store the AST and let the VM execute via interpreter.
    fn compile_method_call(
        &mut self,
        receiver: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<u8, String> {
        // Store method call AST in chunk for interpreter access
        let method_call_idx = self.chunk.method_calls.len();
        self.chunk.method_calls.push((
            Arc::new(receiver.clone()),
            method.to_string(),
            args.iter().map(|arg| Arc::new(arg.clone())).collect(),
        ));

        // Store method call index in constant pool
        let call_info_value = Value::Integer(method_call_idx as i64);
        let call_info_idx = self.chunk.add_constant(call_info_value);

        // Allocate result register
        let result_reg = self.registers.allocate();

        // Emit MethodCall instruction: ABx format
        // A = result register, Bx = method_call_idx
        self.chunk.emit(
            Instruction::abx(OpCode::MethodCall, result_reg, call_info_idx),
            0,
        );

        Ok(result_reg)
    }

    /// Compile a match expression
    ///
    /// OPT-018: Hybrid approach - delegate to interpreter like for-loops
    /// Match expressions require complex pattern matching logic (destructuring, guards, scope management),
    /// so we store the AST and let the VM execute via interpreter.
    fn compile_match(
        &mut self,
        expr: &Expr,
        arms: &[crate::frontend::ast::MatchArm],
    ) -> Result<u8, String> {
        // Store match expression AST in chunk for interpreter access
        let match_idx = self.chunk.match_exprs.len();
        self.chunk
            .match_exprs
            .push((Arc::new(expr.clone()), arms.to_vec()));

        // Store match index in constant pool
        let match_info_value = Value::Integer(match_idx as i64);
        let match_info_idx = self.chunk.add_constant(match_info_value);

        // Allocate result register
        let result_reg = self.registers.allocate();

        // Emit Match instruction: ABx format
        // A = result register, Bx = match_idx
        self.chunk.emit(
            Instruction::abx(OpCode::Match, result_reg, match_info_idx),
            0,
        );

        Ok(result_reg)
    }

    /// Compile a closure expression
    ///
    /// OPT-019: Hybrid approach - delegate to interpreter like for-loops, method calls, match
    /// Closures require environment capture and complex scope management,
    /// so we store the AST and let the VM create the closure with captured environment.
    fn compile_closure(
        &mut self,
        params: &[crate::frontend::ast::Param],
        body: &Expr,
    ) -> Result<u8, String> {
        // RUNTIME-DEFAULT-PARAMS: Extract both param names AND default values
        let params_with_defaults: Vec<(String, Option<Arc<Expr>>)> = params
            .iter()
            .map(|p| {
                (
                    p.name(),
                    p.default_value
                        .clone()
                        .map(|expr| Arc::new((*expr).clone())),
                )
            })
            .collect();

        // Store closure definition in chunk for runtime access
        let closure_idx = self.chunk.closures.len();
        self.chunk
            .closures
            .push((params_with_defaults, Arc::new(body.clone())));

        // Store closure index in constant pool
        let closure_info_value = Value::Integer(closure_idx as i64);
        let closure_info_idx = self.chunk.add_constant(closure_info_value);

        // Allocate result register
        let result_reg = self.registers.allocate();

        // Emit NewClosure instruction: ABx format
        // A = result register, Bx = closure_idx
        self.chunk.emit(
            Instruction::abx(OpCode::NewClosure, result_reg, closure_info_idx),
            0,
        );

        Ok(result_reg)
    }

    /// Compile a field access
    ///
    /// OPT-015: Direct VM implementation (not hybrid)
    /// Field access is simpler than method calls - just extract field from Value.
    /// We can implement the match logic directly in the VM for better performance.
    fn compile_field_access(&mut self, object: &Expr, field: &str) -> Result<u8, String> {
        // Compile object expression
        let object_reg = self.compile_expr(object)?;

        // Store field name in constant pool
        let field_value = Value::from_string(field.to_string());
        let field_idx = self.chunk.add_constant(field_value);

        // Allocate result register
        let result_reg = self.registers.allocate();

        // Emit LoadField instruction: ABC format
        // A = result register, B = object register, C = field constant index
        self.chunk.emit(
            Instruction::abc(OpCode::LoadField, result_reg, object_reg, field_idx as u8),
            0,
        );

        Ok(result_reg)
    }

    /// Finalize compilation and return the bytecode chunk
    pub fn finalize(mut self) -> BytecodeChunk {
        // Emit return instruction with the last result register
        self.chunk
            .emit(Instruction::abc(OpCode::Return, self.last_result, 0, 0), 0);

        // Update register count
        self.chunk.register_count = self.registers.max_count();

        // Copy locals mapping for hybrid execution (for-loops need interpreter access)
        self.chunk.locals_map = self.locals.clone();

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
        assert_ne!(
            idx1, idx3,
            "Different constants should have different indices"
        );
        assert_eq!(
            chunk.constants.len(),
            2,
            "Should only store 2 unique constants"
        );
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
        assert_eq!(
            chunk.instructions.len(),
            2,
            "Should have CONST + RETURN instructions"
        );

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
            Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(3, None)),
                crate::frontend::ast::Span::default(),
            ),
        ];
        let block = Expr::new(
            ExprKind::Block(exprs),
            crate::frontend::ast::Span::default(),
        );

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
        assert!(
            chunk.instructions.len() >= 7,
            "Should have at least 7 instructions"
        );
        assert_eq!(chunk.constants.len(), 3, "Should have 3 constants");

        // Verify conditional jump exists
        let jump_if_false_found = chunk
            .instructions
            .iter()
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
        assert!(
            chunk.instructions.len() >= 4,
            "Should have at least 4 instructions"
        );

        // Verify conditional jump exists
        let jump_if_false_found = chunk
            .instructions
            .iter()
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
        assert!(
            chunk.instructions.len() >= 5,
            "Should have at least 5 instructions"
        );

        // Verify CALL instruction exists
        let call_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Call.to_u8());
        assert!(call_found, "Should have Call instruction");

        assert!(result_reg < 10, "Result register should be valid");
    }

    // Test 10: BytecodeChunk creation and basic operations
    #[test]
    fn test_bytecode_chunk_new() {
        let chunk = BytecodeChunk::new("test_func".to_string());
        assert_eq!(chunk.name, "test_func");
        assert!(chunk.instructions.is_empty());
        assert!(chunk.constants.is_empty());
        assert_eq!(chunk.register_count, 0);
        assert_eq!(chunk.parameter_count, 0);
    }

    // Test 11: BytecodeChunk emit instruction
    #[test]
    fn test_bytecode_chunk_emit() {
        let mut chunk = BytecodeChunk::new("test".to_string());
        let instr = Instruction::abc(OpCode::Add, 0, 1, 2);
        let index = chunk.emit(instr, 10);

        assert_eq!(index, 0);
        assert_eq!(chunk.instructions.len(), 1);
        assert_eq!(chunk.line_numbers.len(), 1);
        assert_eq!(chunk.line_numbers[0], 10);
    }

    // Test 12: BytecodeChunk add_constant deduplication
    #[test]
    fn test_bytecode_chunk_constant_dedup() {
        let mut chunk = BytecodeChunk::new("test".to_string());

        let idx1 = chunk.add_constant(Value::Integer(42));
        let idx2 = chunk.add_constant(Value::Integer(42));
        let idx3 = chunk.add_constant(Value::Integer(100));

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 0, "Duplicate constant should return same index");
        assert_eq!(idx3, 1, "New constant should get new index");
        assert_eq!(chunk.constants.len(), 2);
    }

    // Test 13: RegisterAllocator multiple allocations
    #[test]
    fn test_register_allocator_multiple() {
        let mut allocator = RegisterAllocator::new();

        let r0 = allocator.allocate();
        let r1 = allocator.allocate();
        let r2 = allocator.allocate();

        assert_eq!(r0, 0);
        assert_eq!(r1, 1);
        assert_eq!(r2, 2);
        assert_eq!(allocator.max_count(), 3);
    }

    // Test 14: RegisterAllocator free and reuse multiple
    #[test]
    fn test_register_allocator_free_multiple() {
        let mut allocator = RegisterAllocator::new();

        let r0 = allocator.allocate();
        let r1 = allocator.allocate();
        let r2 = allocator.allocate();

        allocator.free(r1);
        allocator.free(r0);

        // Should reuse in LIFO order
        let r3 = allocator.allocate();
        let r4 = allocator.allocate();

        assert_eq!(r3, r0, "Should reuse r0 first (LIFO)");
        assert_eq!(r4, r1, "Should reuse r1 second");
        assert_eq!(allocator.max_count(), 3, "Max should remain 3");
    }

    // Test 15: Compile unary negation
    #[test]
    fn test_compile_unary_negate() {
        let mut compiler = Compiler::new("test".to_string());
        let operand = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: Box::new(operand),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have: CONST(5), NEG, RETURN
        assert!(chunk.instructions.len() >= 2);

        // Verify NEG instruction exists
        let neg_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Neg.to_u8());
        assert!(neg_found, "Should have Neg instruction");

        assert!(result_reg < 10);
    }

    // Test 16: Compile unary not
    #[test]
    fn test_compile_unary_not() {
        let mut compiler = Compiler::new("test".to_string());
        let operand = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Not,
                operand: Box::new(operand),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have: CONST(true), NOT, RETURN
        assert!(chunk.instructions.len() >= 2);

        // Verify NOT instruction exists
        let not_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Not.to_u8());
        assert!(not_found, "Should have Not instruction");

        assert!(result_reg < 10);
    }

    // Test 17: Compile float literal
    #[test]
    fn test_compile_float_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Float(3.14)),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert_eq!(chunk.constants.len(), 1);
        match &chunk.constants[0] {
            Value::Float(f) => assert!((*f - 3.14).abs() < 0.001),
            _ => panic!("Expected float constant"),
        }
        assert!(result_reg < 10);
    }

    // Test 18: Compile string literal
    #[test]
    fn test_compile_string_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert_eq!(chunk.constants.len(), 1);
        match &chunk.constants[0] {
            Value::String(s) => assert_eq!(s.as_ref(), "hello"),
            _ => panic!("Expected string constant"),
        }
        assert!(result_reg < 10);
    }

    // Test 19: Compile boolean literal
    #[test]
    fn test_compile_bool_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert_eq!(chunk.constants.len(), 1);
        match &chunk.constants[0] {
            Value::Bool(b) => assert!(*b),
            _ => panic!("Expected bool constant"),
        }
        assert!(result_reg < 10);
    }

    // Test 20: Compile subtraction
    #[test]
    fn test_compile_binary_subtraction() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(50, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(8, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Subtract,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Verify SUB instruction exists
        let sub_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Sub.to_u8());
        assert!(sub_found, "Should have Sub instruction");
        assert!(result_reg < 10);
    }

    // Test 21: Compile multiplication
    #[test]
    fn test_compile_binary_multiplication() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(7, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(6, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Verify MUL instruction exists
        let mul_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Mul.to_u8());
        assert!(mul_found, "Should have Mul instruction");
        assert!(result_reg < 10);
    }

    // Test 22: Compile comparison equal
    #[test]
    fn test_compile_binary_equal() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Equal,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Verify EQUAL instruction exists
        let equal_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Equal.to_u8());
        assert!(equal_found, "Should have Equal instruction");
        assert!(result_reg < 10);
    }

    // Test 23: Compile list with literals (optimization path)
    #[test]
    fn test_compile_list_literals() {
        let mut compiler = Compiler::new("test".to_string());
        let elements = vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                crate::frontend::ast::Span::default(),
            ),
        ];
        let expr = Expr::new(
            ExprKind::List(elements),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Literal list optimization: array is created at compile-time in constant pool
        // Uses CONST instruction instead of NewArray
        assert!(chunk.constants.len() >= 1, "Should have array constant");
        assert!(result_reg < 10);
    }

    // Test 23b: Compile list with non-literals (NewArray path)
    #[test]
    fn test_compile_list_non_literals() {
        let mut compiler = Compiler::new("test".to_string());

        // Use identifier elements so it takes the NewArray path
        let elements = vec![
            Expr::new(
                ExprKind::Identifier("x".to_string()),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Identifier("y".to_string()),
                crate::frontend::ast::Span::default(),
            ),
        ];
        let expr = Expr::new(
            ExprKind::List(elements),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have NewArray instruction for non-literal elements
        let new_array_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::NewArray.to_u8());
        assert!(new_array_found, "Should have NewArray instruction");
        assert!(result_reg < 10);
    }

    // Test 24: Compile empty block
    #[test]
    fn test_compile_empty_block() {
        let mut compiler = Compiler::new("test".to_string());
        let block = Expr::new(
            ExprKind::Block(vec![]),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&block).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Empty block should produce nil
        assert_eq!(chunk.constants.len(), 1);
        assert!(result_reg < 10);
    }

    // Test 25: Compile while loop
    #[test]
    fn test_compile_while() {
        let mut compiler = Compiler::new("test".to_string());
        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::While {
                label: None,
                condition: Box::new(condition),
                body: Box::new(body),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have JumpIfFalse and Jump instructions
        let jump_if_false_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::JumpIfFalse.to_u8());
        let jump_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Jump.to_u8());

        assert!(jump_if_false_found, "Should have JumpIfFalse instruction");
        assert!(jump_found, "Should have Jump instruction");
        assert!(result_reg < 10);
    }

    // ===== EXTREME TDD Round 116 - Additional Tests =====

    #[test]
    fn test_compile_boolean_true() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(result_reg < 10);
        assert!(!chunk.constants.is_empty());
    }

    #[test]
    fn test_compile_boolean_false() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(result_reg < 10);
        assert!(!chunk.constants.is_empty());
    }

    #[test]
    fn test_compile_float_literal_pi() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Float(3.14)),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(result_reg < 10);
        assert!(!chunk.constants.is_empty());
        // Verify constant is a float
        match &chunk.constants[0] {
            Value::Float(f) => assert!((*f - 3.14).abs() < 0.001),
            _ => panic!("Expected float constant"),
        }
    }

    #[test]
    fn test_compile_nil_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Null),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(result_reg < 10);
        // Should have nil constant
        assert!(chunk.constants.iter().any(|c| matches!(c, Value::Nil)));
    }

    #[test]
    fn test_register_allocator_many_allocations() {
        let mut allocator = RegisterAllocator::new();

        // Allocate many registers
        let registers: Vec<u8> = (0..100).map(|_| allocator.allocate()).collect();

        assert_eq!(allocator.max_count(), 100);
        assert_eq!(registers[0], 0);
        assert_eq!(registers[99], 99);
    }

    #[test]
    fn test_register_allocator_free_all_reuse() {
        let mut allocator = RegisterAllocator::new();

        // Allocate 10 registers
        let registers: Vec<u8> = (0..10).map(|_| allocator.allocate()).collect();

        // Free all in reverse order
        for r in registers.into_iter().rev() {
            allocator.free(r);
        }

        // Reallocate should reuse
        let r0 = allocator.allocate();
        assert_eq!(r0, 0);
        assert_eq!(allocator.max_count(), 10, "Max count should remain 10");
    }

    #[test]
    fn test_values_equal_integers() {
        assert!(values_equal(&Value::Integer(42), &Value::Integer(42)));
        assert!(!values_equal(&Value::Integer(42), &Value::Integer(43)));
    }

    #[test]
    fn test_values_equal_floats() {
        assert!(values_equal(&Value::Float(3.14), &Value::Float(3.14)));
        assert!(!values_equal(&Value::Float(3.14), &Value::Float(2.71)));
    }

    #[test]
    fn test_values_equal_bools() {
        assert!(values_equal(&Value::Bool(true), &Value::Bool(true)));
        assert!(values_equal(&Value::Bool(false), &Value::Bool(false)));
        assert!(!values_equal(&Value::Bool(true), &Value::Bool(false)));
    }

    #[test]
    fn test_values_equal_nil() {
        assert!(values_equal(&Value::Nil, &Value::Nil));
    }

    #[test]
    fn test_values_equal_different_types() {
        assert!(!values_equal(&Value::Integer(42), &Value::Float(42.0)));
        assert!(!values_equal(&Value::Bool(true), &Value::Integer(1)));
        assert!(!values_equal(&Value::Nil, &Value::Integer(0)));
    }

    #[test]
    fn test_bytecode_chunk_name() {
        let chunk = BytecodeChunk::new("my_function".to_string());
        assert_eq!(chunk.name, "my_function");
        assert!(chunk.constants.is_empty());
        assert!(chunk.instructions.is_empty());
    }

    #[test]
    fn test_constant_pool_different_types() {
        let mut chunk = BytecodeChunk::new("test".to_string());

        let int_idx = chunk.add_constant(Value::Integer(42));
        let float_idx = chunk.add_constant(Value::Float(3.14));
        let bool_idx = chunk.add_constant(Value::Bool(true));
        let nil_idx = chunk.add_constant(Value::Nil);

        assert_ne!(int_idx, float_idx);
        assert_ne!(float_idx, bool_idx);
        assert_ne!(bool_idx, nil_idx);
        assert_eq!(chunk.constants.len(), 4);
    }

    // ===== EXTREME TDD Round 117 - Additional Coverage Tests =====

    // Helper to create a simple Param for tests
    fn make_test_param(name: &str) -> crate::frontend::ast::Param {
        crate::frontend::ast::Param {
            pattern: crate::frontend::ast::Pattern::Identifier(name.to_string()),
            ty: crate::frontend::ast::Type {
                kind: crate::frontend::ast::TypeKind::Named("Any".to_string()),
                span: crate::frontend::ast::Span::default(),
            },
            span: crate::frontend::ast::Span::default(),
            is_mutable: false,
            default_value: None,
        }
    }

    // Helper to create a Param with default value
    fn make_test_param_with_default(name: &str, default: Expr) -> crate::frontend::ast::Param {
        crate::frontend::ast::Param {
            pattern: crate::frontend::ast::Pattern::Identifier(name.to_string()),
            ty: crate::frontend::ast::Type {
                kind: crate::frontend::ast::TypeKind::Named("Any".to_string()),
                span: crate::frontend::ast::Span::default(),
            },
            span: crate::frontend::ast::Span::default(),
            is_mutable: false,
            default_value: Some(Box::new(default)),
        }
    }

    // Test literal types: Unit
    #[test]
    fn test_compile_unit_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Unit),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(result_reg < 10);
        assert!(!chunk.constants.is_empty());
        assert!(chunk.constants.iter().any(|c| matches!(c, Value::Nil)));
    }

    // Test literal types: Char
    #[test]
    fn test_compile_char_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Char('x')),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(result_reg < 10);
        assert!(!chunk.constants.is_empty());
        match &chunk.constants[0] {
            Value::String(s) => assert_eq!(s.as_ref(), "x"),
            _ => panic!("Expected string constant for char"),
        }
    }

    // Test literal types: Byte
    #[test]
    fn test_compile_byte_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Byte(255)),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(result_reg < 10);
        assert!(!chunk.constants.is_empty());
        match &chunk.constants[0] {
            Value::Integer(i) => assert_eq!(*i, 255),
            _ => panic!("Expected integer constant for byte"),
        }
    }

    // Test literal types: Atom
    #[test]
    fn test_compile_atom_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Atom("ok".to_string())),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(result_reg < 10);
        assert!(!chunk.constants.is_empty());
        match &chunk.constants[0] {
            Value::Atom(s) => assert_eq!(s.as_str(), "ok"),
            _ => panic!("Expected atom constant"),
        }
    }

    // Test binary operators: Division
    #[test]
    fn test_compile_binary_division() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Divide,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let div_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Div.to_u8());
        assert!(div_found, "Should have Div instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: Modulo
    #[test]
    fn test_compile_binary_modulo() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(17, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Modulo,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let mod_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Mod.to_u8());
        assert!(mod_found, "Should have Mod instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: NotEqual
    #[test]
    fn test_compile_binary_not_equal() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::NotEqual,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let neq_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::NotEqual.to_u8());
        assert!(neq_found, "Should have NotEqual instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: Greater
    #[test]
    fn test_compile_binary_greater() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Greater,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let gt_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Greater.to_u8());
        assert!(gt_found, "Should have Greater instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: GreaterEqual
    #[test]
    fn test_compile_binary_greater_equal() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::GreaterEqual,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let ge_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::GreaterEqual.to_u8());
        assert!(ge_found, "Should have GreaterEqual instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: Less
    #[test]
    fn test_compile_binary_less() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(3, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(7, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Less,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let lt_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Less.to_u8());
        assert!(lt_found, "Should have Less instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: LessEqual
    #[test]
    fn test_compile_binary_less_equal() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::LessEqual,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let le_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::LessEqual.to_u8());
        assert!(le_found, "Should have LessEqual instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: And
    #[test]
    fn test_compile_binary_and() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let and_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::And.to_u8());
        assert!(and_found, "Should have And instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: Or
    #[test]
    fn test_compile_binary_or() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let or_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Or.to_u8());
        assert!(or_found, "Should have Or instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: BitwiseAnd
    #[test]
    fn test_compile_binary_bitwise_and() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(0b1100, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(0b1010, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::BitwiseAnd,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let band_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::BitAnd.to_u8());
        assert!(band_found, "Should have BitAnd instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: BitwiseOr
    #[test]
    fn test_compile_binary_bitwise_or() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(0b1100, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(0b1010, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::BitwiseOr,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let bor_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::BitOr.to_u8());
        assert!(bor_found, "Should have BitOr instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: BitwiseXor
    #[test]
    fn test_compile_binary_bitwise_xor() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(0b1100, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(0b1010, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::BitwiseXor,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let bxor_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::BitXor.to_u8());
        assert!(bxor_found, "Should have BitXor instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: LeftShift
    #[test]
    fn test_compile_binary_left_shift() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(4, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::LeftShift,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let shl_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::ShiftLeft.to_u8());
        assert!(shl_found, "Should have ShiftLeft instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: RightShift
    #[test]
    fn test_compile_binary_right_shift() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(16, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::RightShift,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let shr_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::ShiftRight.to_u8());
        assert!(shr_found, "Should have ShiftRight instruction");
        assert!(result_reg < 10);
    }

    // Test unary operator: BitwiseNot
    #[test]
    fn test_compile_unary_bitwise_not() {
        let mut compiler = Compiler::new("test".to_string());
        let operand = Expr::new(
            ExprKind::Literal(Literal::Integer(0b1010, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::BitwiseNot,
                operand: Box::new(operand),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let bitnot_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::BitNot.to_u8());
        assert!(bitnot_found, "Should have BitNot instruction");
        assert!(result_reg < 10);
    }

    // Test unsupported unary operator: Reference
    #[test]
    fn test_compile_unary_reference_error() {
        let mut compiler = Compiler::new("test".to_string());
        let operand = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Reference,
                operand: Box::new(operand),
            },
            crate::frontend::ast::Span::default(),
        );

        let result = compiler.compile_expr(&expr);
        assert!(result.is_err(), "Reference operator should fail");
        assert!(result.unwrap_err().contains("Unsupported unary operator"));
    }

    // Test unsupported unary operator: MutableReference
    #[test]
    fn test_compile_unary_mutable_reference_error() {
        let mut compiler = Compiler::new("test".to_string());
        let operand = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::MutableReference,
                operand: Box::new(operand),
            },
            crate::frontend::ast::Span::default(),
        );

        let result = compiler.compile_expr(&expr);
        assert!(result.is_err(), "MutableReference operator should fail");
        assert!(result.unwrap_err().contains("Unsupported unary operator"));
    }

    // Test unsupported unary operator: Deref
    #[test]
    fn test_compile_unary_deref_error() {
        let mut compiler = Compiler::new("test".to_string());
        let operand = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Deref,
                operand: Box::new(operand),
            },
            crate::frontend::ast::Span::default(),
        );

        let result = compiler.compile_expr(&expr);
        assert!(result.is_err(), "Deref operator should fail");
        assert!(result.unwrap_err().contains("Unsupported unary operator"));
    }

    // Test let binding compilation
    #[test]
    fn test_compile_let_binding() {
        let mut compiler = Compiler::new("test".to_string());

        // let x = 42 in x
        let value = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(value),
                body: Box::new(body),
                is_mutable: false,
                else_block: None,
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&let_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(!chunk.local_names.is_empty(), "Should have local names");
        assert_eq!(chunk.local_names[0], "x");
        assert!(result_reg < 10);
    }

    // Test variable reference: local variable
    #[test]
    fn test_compile_variable_local() {
        let mut compiler = Compiler::new("test".to_string());

        // First, set up a local variable through let binding
        let value = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(value),
                body: Box::new(body),
                is_mutable: false,
                else_block: None,
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&let_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have MOVE instruction for copying local variable
        let move_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Move.to_u8());
        assert!(
            move_found,
            "Should have Move instruction for local variable access"
        );
        assert!(result_reg < 10);
    }

    // Test variable reference: global variable
    #[test]
    fn test_compile_variable_global() {
        let mut compiler = Compiler::new("test".to_string());

        // Reference a global variable (not defined locally)
        let expr = Expr::new(
            ExprKind::Identifier("global_var".to_string()),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have LOAD_GLOBAL instruction
        let load_global_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::LoadGlobal.to_u8());
        assert!(load_global_found, "Should have LoadGlobal instruction");
        assert!(result_reg < 10);
    }

    // Test assignment to local variable
    #[test]
    fn test_compile_assignment() {
        let mut compiler = Compiler::new("test".to_string());

        // let mut x = 10 in { x = 20; x }
        let init_value = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            crate::frontend::ast::Span::default(),
        );
        let target = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let new_value = Expr::new(
            ExprKind::Literal(Literal::Integer(20, None)),
            crate::frontend::ast::Span::default(),
        );
        let assign_expr = Expr::new(
            ExprKind::Assign {
                target: Box::new(target.clone()),
                value: Box::new(new_value),
            },
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Block(vec![assign_expr, target]),
            crate::frontend::ast::Span::default(),
        );
        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(init_value),
                body: Box::new(body),
                is_mutable: true,
                else_block: None,
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&let_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have MOVE instructions for assignment
        let move_count = chunk
            .instructions
            .iter()
            .filter(|i| i.opcode() == OpCode::Move.to_u8())
            .count();
        assert!(
            move_count >= 1,
            "Should have Move instruction(s) for assignment"
        );
        assert!(result_reg < 10);
    }

    // Test assignment to undefined variable (error)
    #[test]
    fn test_compile_assignment_undefined_error() {
        let mut compiler = Compiler::new("test".to_string());

        let target = Expr::new(
            ExprKind::Identifier("undefined_var".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let value = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );
        let assign_expr = Expr::new(
            ExprKind::Assign {
                target: Box::new(target),
                value: Box::new(value),
            },
            crate::frontend::ast::Span::default(),
        );

        let result = compiler.compile_expr(&assign_expr);
        assert!(
            result.is_err(),
            "Assignment to undefined variable should fail"
        );
        assert!(result.unwrap_err().contains("Undefined variable"));
    }

    // Test function definition
    #[test]
    fn test_compile_function_definition() {
        let mut compiler = Compiler::new("test".to_string());

        // fun add(a, b) { a + b }
        let param_a = make_test_param("a");
        let param_b = make_test_param("b");

        let a = Expr::new(
            ExprKind::Identifier("a".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let b = Expr::new(
            ExprKind::Identifier("b".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(a),
                right: Box::new(b),
            },
            crate::frontend::ast::Span::default(),
        );

        let func_expr = Expr::new(
            ExprKind::Function {
                name: "add".to_string(),
                type_params: vec![],
                params: vec![param_a, param_b],
                return_type: None,
                body: Box::new(body),
                is_async: false,
                is_pub: false,
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&func_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Function should be stored in locals
        assert!(
            chunk.local_names.contains(&"add".to_string()),
            "Function should be in local names"
        );
        // Should have a Closure constant
        let has_closure = chunk
            .constants
            .iter()
            .any(|c| matches!(c, Value::Closure { .. }));
        assert!(has_closure, "Should have Closure constant");
        assert!(result_reg < 10);
    }

    // Test for loop compilation
    #[test]
    fn test_compile_for_loop() {
        let mut compiler = Compiler::new("test".to_string());

        // for i in [1, 2, 3] { i }
        let iter_elements = vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(3, None)),
                crate::frontend::ast::Span::default(),
            ),
        ];
        let iter_expr = Expr::new(
            ExprKind::List(iter_elements),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Identifier("i".to_string()),
            crate::frontend::ast::Span::default(),
        );

        let for_expr = Expr::new(
            ExprKind::For {
                label: None,
                var: "i".to_string(),
                pattern: None,
                iter: Box::new(iter_expr),
                body: Box::new(body),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&for_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have For opcode
        let for_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::For.to_u8());
        assert!(for_found, "Should have For instruction");
        // Should have stored loop body
        assert!(
            !chunk.loop_bodies.is_empty(),
            "Should have stored loop body"
        );
        assert!(result_reg < 10);
    }

    // Test method call compilation
    #[test]
    fn test_compile_method_call() {
        let mut compiler = Compiler::new("test".to_string());

        // "hello".len()
        let receiver = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            crate::frontend::ast::Span::default(),
        );
        let method_call = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(receiver),
                method: "len".to_string(),
                args: vec![],
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&method_call)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have MethodCall opcode
        let method_call_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::MethodCall.to_u8());
        assert!(method_call_found, "Should have MethodCall instruction");
        // Should have stored method call info
        assert!(
            !chunk.method_calls.is_empty(),
            "Should have stored method call info"
        );
        assert!(result_reg < 10);
    }

    // Test field access compilation
    #[test]
    fn test_compile_field_access() {
        let mut compiler = Compiler::new("test".to_string());

        // obj.field
        let object = Expr::new(
            ExprKind::Identifier("obj".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let field_access = Expr::new(
            ExprKind::FieldAccess {
                object: Box::new(object),
                field: "field".to_string(),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&field_access)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have LoadField opcode
        let load_field_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::LoadField.to_u8());
        assert!(load_field_found, "Should have LoadField instruction");
        assert!(result_reg < 10);
    }

    // Test index access compilation
    #[test]
    fn test_compile_index_access() {
        let mut compiler = Compiler::new("test".to_string());

        // arr[0]
        let object = Expr::new(
            ExprKind::Identifier("arr".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let index = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            crate::frontend::ast::Span::default(),
        );
        let index_access = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(object),
                index: Box::new(index),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&index_access)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have LoadIndex opcode
        let load_index_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::LoadIndex.to_u8());
        assert!(load_index_found, "Should have LoadIndex instruction");
        assert!(result_reg < 10);
    }

    // Test tuple compilation with literals
    #[test]
    fn test_compile_tuple_literals() {
        let mut compiler = Compiler::new("test".to_string());

        // (1, 2, 3)
        let elements = vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(3, None)),
                crate::frontend::ast::Span::default(),
            ),
        ];
        let tuple_expr = Expr::new(
            ExprKind::Tuple(elements),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&tuple_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Literal tuple should be in constant pool
        let has_tuple = chunk.constants.iter().any(|c| matches!(c, Value::Tuple(_)));
        assert!(has_tuple, "Should have Tuple constant");
        assert!(result_reg < 10);
    }

    // Test tuple compilation with non-literals
    #[test]
    fn test_compile_tuple_non_literals() {
        let mut compiler = Compiler::new("test".to_string());

        // (x, y)
        let elements = vec![
            Expr::new(
                ExprKind::Identifier("x".to_string()),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Identifier("y".to_string()),
                crate::frontend::ast::Span::default(),
            ),
        ];
        let tuple_expr = Expr::new(
            ExprKind::Tuple(elements),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&tuple_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have NewTuple instruction for non-literal elements
        let new_tuple_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::NewTuple.to_u8());
        assert!(new_tuple_found, "Should have NewTuple instruction");
        assert!(result_reg < 10);
    }

    // Test object literal with literals
    #[test]
    fn test_compile_object_literal_literals() {
        use crate::frontend::ast::ObjectField;

        let mut compiler = Compiler::new("test".to_string());

        // { x: 1, y: 2 }
        let fields = vec![
            ObjectField::KeyValue {
                key: "x".to_string(),
                value: Expr::new(
                    ExprKind::Literal(Literal::Integer(1, None)),
                    crate::frontend::ast::Span::default(),
                ),
            },
            ObjectField::KeyValue {
                key: "y".to_string(),
                value: Expr::new(
                    ExprKind::Literal(Literal::Integer(2, None)),
                    crate::frontend::ast::Span::default(),
                ),
            },
        ];
        let obj_expr = Expr::new(
            ExprKind::ObjectLiteral { fields },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&obj_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Literal object should be in constant pool
        let has_object = chunk
            .constants
            .iter()
            .any(|c| matches!(c, Value::Object(_)));
        assert!(has_object, "Should have Object constant");
        assert!(result_reg < 10);
    }

    // Test object literal with non-literals
    #[test]
    fn test_compile_object_literal_non_literals() {
        use crate::frontend::ast::ObjectField;

        let mut compiler = Compiler::new("test".to_string());

        // { x: a, y: b }
        let fields = vec![
            ObjectField::KeyValue {
                key: "x".to_string(),
                value: Expr::new(
                    ExprKind::Identifier("a".to_string()),
                    crate::frontend::ast::Span::default(),
                ),
            },
            ObjectField::KeyValue {
                key: "y".to_string(),
                value: Expr::new(
                    ExprKind::Identifier("b".to_string()),
                    crate::frontend::ast::Span::default(),
                ),
            },
        ];
        let obj_expr = Expr::new(
            ExprKind::ObjectLiteral { fields },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&obj_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have NewObject instruction for non-literal elements
        let new_object_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::NewObject.to_u8());
        assert!(new_object_found, "Should have NewObject instruction");
        assert!(result_reg < 10);
    }

    // Test object literal with spread (error)
    #[test]
    fn test_compile_object_literal_spread_error() {
        use crate::frontend::ast::ObjectField;

        let mut compiler = Compiler::new("test".to_string());

        // { ...other }
        let fields = vec![ObjectField::Spread {
            expr: Expr::new(
                ExprKind::Identifier("other".to_string()),
                crate::frontend::ast::Span::default(),
            ),
        }];
        let obj_expr = Expr::new(
            ExprKind::ObjectLiteral { fields },
            crate::frontend::ast::Span::default(),
        );

        let result = compiler.compile_expr(&obj_expr);
        assert!(result.is_err(), "Spread in object literal should fail");
        assert!(result.unwrap_err().contains("Spread operator"));
    }

    // Test match expression compilation
    #[test]
    fn test_compile_match_expression() {
        use crate::frontend::ast::{MatchArm, Pattern};

        let mut compiler = Compiler::new("test".to_string());

        // match x { 1 => "one", _ => "other" }
        let match_expr = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Integer(1, None)),
                guard: None,
                body: Box::new(Expr::new(
                    ExprKind::Literal(Literal::String("one".to_string())),
                    crate::frontend::ast::Span::default(),
                )),
                span: crate::frontend::ast::Span::default(),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(Expr::new(
                    ExprKind::Literal(Literal::String("other".to_string())),
                    crate::frontend::ast::Span::default(),
                )),
                span: crate::frontend::ast::Span::default(),
            },
        ];
        let match_full = Expr::new(
            ExprKind::Match {
                expr: Box::new(match_expr),
                arms,
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&match_full)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have Match opcode
        let match_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Match.to_u8());
        assert!(match_found, "Should have Match instruction");
        // Should have stored match expression
        assert!(
            !chunk.match_exprs.is_empty(),
            "Should have stored match expression"
        );
        assert!(result_reg < 10);
    }

    // Test closure compilation
    #[test]
    fn test_compile_closure() {
        let mut compiler = Compiler::new("test".to_string());

        // |x| x + 1
        let param = make_test_param("x");
        let x = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let one = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(x),
                right: Box::new(one),
            },
            crate::frontend::ast::Span::default(),
        );

        let lambda = Expr::new(
            ExprKind::Lambda {
                params: vec![param],
                body: Box::new(body),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&lambda).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have NewClosure opcode
        let new_closure_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::NewClosure.to_u8());
        assert!(new_closure_found, "Should have NewClosure instruction");
        // Should have stored closure info
        assert!(
            !chunk.closures.is_empty(),
            "Should have stored closure info"
        );
        assert!(result_reg < 10);
    }

    // Test empty list compilation
    #[test]
    fn test_compile_empty_list() {
        let mut compiler = Compiler::new("test".to_string());

        // []
        let list_expr = Expr::new(
            ExprKind::List(vec![]),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&list_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Empty list uses NewArray path (not constant pool optimization)
        let new_array_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::NewArray.to_u8());
        assert!(
            new_array_found,
            "Should have NewArray instruction for empty list"
        );
        assert!(result_reg < 10);
    }

    // Test empty tuple compilation
    #[test]
    fn test_compile_empty_tuple() {
        let mut compiler = Compiler::new("test".to_string());

        // ()
        let tuple_expr = Expr::new(
            ExprKind::Tuple(vec![]),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&tuple_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Empty tuple uses NewTuple path
        let new_tuple_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::NewTuple.to_u8());
        assert!(
            new_tuple_found,
            "Should have NewTuple instruction for empty tuple"
        );
        assert!(result_reg < 10);
    }

    // Test empty object compilation
    #[test]
    fn test_compile_empty_object() {
        let mut compiler = Compiler::new("test".to_string());

        // {}
        let obj_expr = Expr::new(
            ExprKind::ObjectLiteral { fields: vec![] },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&obj_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Empty object uses NewObject path
        let new_object_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::NewObject.to_u8());
        assert!(
            new_object_found,
            "Should have NewObject instruction for empty object"
        );
        assert!(result_reg < 10);
    }

    // Test patch_jump functionality
    #[test]
    fn test_patch_jump() {
        let mut chunk = BytecodeChunk::new("test".to_string());

        // Emit a jump with placeholder offset
        let jump_idx = chunk.emit(Instruction::asbx(OpCode::Jump, 0, 0), 0);

        // Emit some instructions
        chunk.emit(Instruction::abc(OpCode::Add, 0, 1, 2), 0);
        chunk.emit(Instruction::abc(OpCode::Add, 0, 1, 2), 0);
        chunk.emit(Instruction::abc(OpCode::Add, 0, 1, 2), 0);

        // Patch the jump
        chunk.patch_jump(jump_idx);

        // Verify the jump offset was updated correctly
        let jump_instr = chunk.instructions[jump_idx];
        assert_eq!(jump_instr.opcode(), OpCode::Jump.to_u8());
        assert_eq!(
            jump_instr.get_sbx(),
            3,
            "Jump offset should be 3 (skip 3 instructions)"
        );
    }

    // Test finalize method
    #[test]
    fn test_finalize() {
        let mut compiler = Compiler::new("my_function".to_string());

        // Compile a simple expression
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );
        let _ = compiler.compile_expr(&expr).expect("Compilation failed");

        let chunk = compiler.finalize();

        // Check finalize results
        assert_eq!(chunk.name, "my_function");
        assert!(chunk.register_count > 0, "Should have register count set");
        // Last instruction should be Return
        let last_instr = chunk.instructions.last().expect("Should have instructions");
        assert_eq!(last_instr.opcode(), OpCode::Return.to_u8());
    }

    // Test is_local_register helper
    #[test]
    fn test_is_local_register() {
        let mut compiler = Compiler::new("test".to_string());

        // Initially no locals
        assert!(!compiler.is_local_register(0));
        assert!(!compiler.is_local_register(1));

        // Add a local variable through let binding
        let value = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            crate::frontend::ast::Span::default(),
        );
        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(value),
                body: Box::new(body),
                is_mutable: false,
                else_block: None,
            },
            crate::frontend::ast::Span::default(),
        );
        let _ = compiler
            .compile_expr(&let_expr)
            .expect("Compilation failed");

        // Register 0 should now be a local
        assert!(compiler.is_local_register(0));
        // Other registers should not be locals
        assert!(!compiler.is_local_register(100));
    }

    // Test unsupported expression kind
    #[test]
    fn test_compile_unsupported_expression() {
        let mut compiler = Compiler::new("test".to_string());

        // Use an unsupported expression kind (e.g., Import)
        let expr = Expr::new(
            ExprKind::Import {
                module: "std".to_string(),
                items: None,
            },
            crate::frontend::ast::Span::default(),
        );

        let result = compiler.compile_expr(&expr);
        assert!(result.is_err(), "Unsupported expression should fail");
        assert!(result.unwrap_err().contains("Unsupported expression kind"));
    }

    // Test function call with no arguments
    #[test]
    fn test_compile_function_call_no_args() {
        let mut compiler = Compiler::new("test".to_string());

        // foo()
        let func = Expr::new(
            ExprKind::Identifier("foo".to_string()),
            crate::frontend::ast::Span::default(),
        );

        let call = Expr::new(
            ExprKind::Call {
                func: Box::new(func),
                args: vec![],
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&call).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have Call instruction
        let call_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Call.to_u8());
        assert!(call_found, "Should have Call instruction");
        assert!(result_reg < 10);
    }

    // Test method call with arguments
    #[test]
    fn test_compile_method_call_with_args() {
        let mut compiler = Compiler::new("test".to_string());

        // "hello".substring(0, 3)
        let receiver = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            crate::frontend::ast::Span::default(),
        );
        let method_call = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(receiver),
                method: "substring".to_string(),
                args: vec![
                    Expr::new(
                        ExprKind::Literal(Literal::Integer(0, None)),
                        crate::frontend::ast::Span::default(),
                    ),
                    Expr::new(
                        ExprKind::Literal(Literal::Integer(3, None)),
                        crate::frontend::ast::Span::default(),
                    ),
                ],
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&method_call)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have stored method call with args
        assert!(!chunk.method_calls.is_empty());
        let (_, method_name, args) = &chunk.method_calls[0];
        assert_eq!(method_name, "substring");
        assert_eq!(args.len(), 2);
        assert!(result_reg < 10);
    }

    // Test nested if expression
    #[test]
    fn test_compile_nested_if() {
        let mut compiler = Compiler::new("test".to_string());

        // if true { if false { 1 } else { 2 } } else { 3 }
        let inner_condition = Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            crate::frontend::ast::Span::default(),
        );
        let inner_then = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            crate::frontend::ast::Span::default(),
        );
        let inner_else = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            crate::frontend::ast::Span::default(),
        );
        let inner_if = Expr::new(
            ExprKind::If {
                condition: Box::new(inner_condition),
                then_branch: Box::new(inner_then),
                else_branch: Some(Box::new(inner_else)),
            },
            crate::frontend::ast::Span::default(),
        );

        let outer_condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::default(),
        );
        let outer_else = Expr::new(
            ExprKind::Literal(Literal::Integer(3, None)),
            crate::frontend::ast::Span::default(),
        );
        let outer_if = Expr::new(
            ExprKind::If {
                condition: Box::new(outer_condition),
                then_branch: Box::new(inner_if),
                else_branch: Some(Box::new(outer_else)),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&outer_if)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have multiple JumpIfFalse instructions
        let jump_count = chunk
            .instructions
            .iter()
            .filter(|i| i.opcode() == OpCode::JumpIfFalse.to_u8())
            .count();
        assert!(
            jump_count >= 2,
            "Should have at least 2 JumpIfFalse instructions for nested if"
        );
        assert!(result_reg < 10);
    }

    // Test block with local variable not freed
    #[test]
    fn test_compile_block_preserves_local_registers() {
        let mut compiler = Compiler::new("test".to_string());

        // { let x = 1; let y = 2; x + y }
        let let_x = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(1, None)),
                    crate::frontend::ast::Span::default(),
                )),
                body: Box::new(Expr::new(
                    ExprKind::Let {
                        name: "y".to_string(),
                        type_annotation: None,
                        value: Box::new(Expr::new(
                            ExprKind::Literal(Literal::Integer(2, None)),
                            crate::frontend::ast::Span::default(),
                        )),
                        body: Box::new(Expr::new(
                            ExprKind::Binary {
                                op: BinaryOp::Add,
                                left: Box::new(Expr::new(
                                    ExprKind::Identifier("x".to_string()),
                                    crate::frontend::ast::Span::default(),
                                )),
                                right: Box::new(Expr::new(
                                    ExprKind::Identifier("y".to_string()),
                                    crate::frontend::ast::Span::default(),
                                )),
                            },
                            crate::frontend::ast::Span::default(),
                        )),
                        is_mutable: false,
                        else_block: None,
                    },
                    crate::frontend::ast::Span::default(),
                )),
                is_mutable: false,
                else_block: None,
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&let_x).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have both local names
        assert!(chunk.local_names.contains(&"x".to_string()));
        assert!(chunk.local_names.contains(&"y".to_string()));
        assert!(result_reg < 10);
    }

    // Test locals_map is populated in finalize
    #[test]
    fn test_finalize_populates_locals_map() {
        let mut compiler = Compiler::new("test".to_string());

        // let x = 42 in x
        let value = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(value),
                body: Box::new(body),
                is_mutable: false,
                else_block: None,
            },
            crate::frontend::ast::Span::default(),
        );

        let _ = compiler
            .compile_expr(&let_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // locals_map should contain x
        assert!(chunk.locals_map.contains_key("x"));
        assert_eq!(*chunk.locals_map.get("x").unwrap(), 0);
    }

    // Test Gt binary operator (alias for Greater)
    #[test]
    fn test_compile_binary_gt() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Gt,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let gt_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Greater.to_u8());
        assert!(gt_found, "Should have Greater instruction for Gt operator");
        assert!(result_reg < 10);
    }

    // Test float constant deduplication
    #[test]
    fn test_float_constant_deduplication() {
        let mut chunk = BytecodeChunk::new("test".to_string());

        let idx1 = chunk.add_constant(Value::Float(3.14));
        let idx2 = chunk.add_constant(Value::Float(3.14));
        let idx3 = chunk.add_constant(Value::Float(2.71));

        assert_eq!(idx1, idx2, "Duplicate floats should return same index");
        assert_ne!(idx1, idx3, "Different floats should have different indices");
        assert_eq!(chunk.constants.len(), 2);
    }

    // Test bool constant deduplication
    #[test]
    fn test_bool_constant_deduplication() {
        let mut chunk = BytecodeChunk::new("test".to_string());

        let idx1 = chunk.add_constant(Value::Bool(true));
        let idx2 = chunk.add_constant(Value::Bool(true));
        let idx3 = chunk.add_constant(Value::Bool(false));
        let idx4 = chunk.add_constant(Value::Bool(false));

        assert_eq!(idx1, idx2, "Duplicate true should return same index");
        assert_eq!(idx3, idx4, "Duplicate false should return same index");
        assert_ne!(idx1, idx3, "true and false should have different indices");
        assert_eq!(chunk.constants.len(), 2);
    }

    // Test nil constant deduplication
    #[test]
    fn test_nil_constant_deduplication() {
        let mut chunk = BytecodeChunk::new("test".to_string());

        let idx1 = chunk.add_constant(Value::Nil);
        let idx2 = chunk.add_constant(Value::Nil);
        let idx3 = chunk.add_constant(Value::Nil);

        assert_eq!(idx1, idx2, "Duplicate Nil should return same index");
        assert_eq!(idx2, idx3, "All Nil should return same index");
        assert_eq!(chunk.constants.len(), 1);
    }

    // Test function with default parameters
    #[test]
    fn test_compile_function_with_defaults() {
        let mut compiler = Compiler::new("test".to_string());

        // fun greet(name, greeting = "Hello") { greeting + name }
        let param_name = make_test_param("name");
        let default_value = Expr::new(
            ExprKind::Literal(Literal::String("Hello".to_string())),
            crate::frontend::ast::Span::default(),
        );
        let param_greeting = make_test_param_with_default("greeting", default_value);

        let greeting_ident = Expr::new(
            ExprKind::Identifier("greeting".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let name_ident = Expr::new(
            ExprKind::Identifier("name".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(greeting_ident),
                right: Box::new(name_ident),
            },
            crate::frontend::ast::Span::default(),
        );

        let func_expr = Expr::new(
            ExprKind::Function {
                name: "greet".to_string(),
                type_params: vec![],
                params: vec![param_name, param_greeting],
                return_type: None,
                body: Box::new(body),
                is_async: false,
                is_pub: false,
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&func_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Function should have closure with default parameters
        let has_closure = chunk.constants.iter().any(|c| {
            if let Value::Closure { params, .. } = c {
                // Second param should have a default value
                params.len() == 2 && params[1].1.is_some()
            } else {
                false
            }
        });
        assert!(has_closure, "Should have Closure with default parameter");
        assert!(result_reg < 10);
    }

    // Test closure with default parameters
    #[test]
    fn test_compile_closure_with_defaults() {
        let mut compiler = Compiler::new("test".to_string());

        // |x, y = 10| x + y
        let param_x = make_test_param("x");
        let default_y = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            crate::frontend::ast::Span::default(),
        );
        let param_y = make_test_param_with_default("y", default_y);

        let x = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let y = Expr::new(
            ExprKind::Identifier("y".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(x),
                right: Box::new(y),
            },
            crate::frontend::ast::Span::default(),
        );

        let lambda = Expr::new(
            ExprKind::Lambda {
                params: vec![param_x, param_y],
                body: Box::new(body),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&lambda).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Closure info should include default parameters
        assert!(!chunk.closures.is_empty());
        let (params, _) = &chunk.closures[0];
        assert_eq!(params.len(), 2);
        assert!(
            params[1].1.is_some(),
            "Second param should have default value"
        );
        assert!(result_reg < 10);
    }

    // Test string values not deduplicated (by reference)
    #[test]
    fn test_string_constant_not_deduplicated() {
        let mut chunk = BytecodeChunk::new("test".to_string());

        let idx1 = chunk.add_constant(Value::from_string("hello".to_string()));
        let idx2 = chunk.add_constant(Value::from_string("hello".to_string()));

        // String comparison returns false in values_equal (by reference, not value)
        // So strings are not deduplicated
        assert_ne!(
            idx1, idx2,
            "Strings should not be deduplicated (by reference)"
        );
        assert_eq!(chunk.constants.len(), 2);
    }
}
