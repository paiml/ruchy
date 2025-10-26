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
use std::collections::HashMap;
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
    /// Each entry: (params, body) - environment captured at runtime
    pub closures: Vec<(Vec<String>, Arc<Expr>)>,
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
            ExprKind::Unary { op, operand} => self.compile_unary(op, operand),
            ExprKind::Identifier(name) => self.compile_variable(name),
            ExprKind::Let { name, value, body, .. } => self.compile_let(name, value, body),
            ExprKind::Block(exprs) => self.compile_block(exprs),
            ExprKind::If { condition, then_branch, else_branch } => {
                self.compile_if(condition, then_branch, else_branch.as_deref())
            }
            ExprKind::Call { func, args} => self.compile_call(func, args),
            ExprKind::While { condition, body, .. } => self.compile_while(condition, body),
            ExprKind::Assign { target, value } => self.compile_assign(target, value),
            ExprKind::Function { name, params, body, .. } => self.compile_function(name, params, body),
            ExprKind::List(elements) => self.compile_list(elements),
            ExprKind::Tuple(elements) => self.compile_tuple(elements),
            ExprKind::ObjectLiteral { fields } => self.compile_object_literal(fields),
            ExprKind::For { var, iter, body, .. } => self.compile_for(var, iter, body),
            ExprKind::IndexAccess { object, index } => self.compile_index_access(object, index),
            ExprKind::MethodCall { receiver, method, args } => self.compile_method_call(receiver, method, args),
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
            _ => return Err(format!("Unsupported binary operator: {op:?}")),
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
                return Err(format!("Unsupported unary operator: {op:?}"));
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
    /// Generates: `loop_start` → check condition → if false jump to end → body → jump to start → `loop_end`
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
                    return Err(format!("Undefined variable: {name}"));
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
        self.chunk.emit(
            Instruction::abx(OpCode::Call, result_reg, call_info_idx),
            0,
        );

        // OPT-011: Don't free func_reg or arg_regs yet - they contain values needed at runtime
        // TODO: Implement proper lifetime analysis for register allocation
        // For now, accept some register pressure to ensure correctness

        Ok(result_reg)
    }

    /// Compile a function definition
    ///
    /// Creates a closure and stores it in locals for later invocation.
    fn compile_function(&mut self, name: &str, params: &[Param], body: &Expr) -> Result<u8, String> {
        // Extract parameter names
        let param_names: Vec<String> = params
            .iter()
            .map(crate::frontend::ast::Param::name)
            .collect();

        // Create closure value
        // Note: Using empty environment for now. Full lexical scoping will be added later.
        let closure = Value::Closure {
            params: param_names,
            body: Arc::new(body.clone()),
            env: Arc::new(HashMap::new()),
        };

        // Add closure to constant pool
        let const_index = self.chunk.add_constant(closure);

        // Allocate register for the closure
        let closure_reg = self.registers.allocate();

        // Emit CONST instruction to load closure into register
        self.chunk.emit(
            Instruction::abx(OpCode::Const, closure_reg, const_index),
            0,
        );

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
        let all_literals = elements.iter().all(|elem| {
            matches!(&elem.kind, ExprKind::Literal(_))
        });

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
                    };
                    element_values.push(value);
                }
            }

            let array_value = Value::from_array(element_values);
            let const_index = self.chunk.add_constant(array_value);

            let result_reg = self.registers.allocate();
            self.chunk.emit(
                Instruction::abx(OpCode::Const, result_reg, const_index),
                0,
            );
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
        let all_literals = elements.iter().all(|elem| {
            matches!(&elem.kind, ExprKind::Literal(_))
        });

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
                    };
                    element_values.push(value);
                }
            }

            let tuple_value = Value::Tuple(Arc::from(element_values.as_slice()));
            let const_index = self.chunk.add_constant(tuple_value);

            let result_reg = self.registers.allocate();
            self.chunk.emit(
                Instruction::abx(OpCode::Const, result_reg, const_index),
                0,
            );
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
    fn compile_object_literal(&mut self, fields: &[crate::frontend::ast::ObjectField]) -> Result<u8, String> {
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
                        };
                        object_map.insert(key.clone(), val);
                    }
                }
            }

            let object_value = Value::Object(Arc::new(object_map));
            let const_index = self.chunk.add_constant(object_value);

            let result_reg = self.registers.allocate();
            self.chunk.emit(
                Instruction::abx(OpCode::Const, result_reg, const_index),
                0,
            );
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
                        return Err("Spread operator in object literals not yet supported in bytecode mode".to_string());
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
            Value::Integer(i64::from(iter_reg)),  // Register holding the iterator
            Value::from_string(var.to_string()),  // Loop variable name
            Value::Integer(body_idx as i64),  // Index into chunk.loop_bodies
        ];
        let loop_info_value = Value::from_array(loop_info);
        let loop_info_idx = self.chunk.add_constant(loop_info_value);

        // Allocate result register
        let result_reg = self.registers.allocate();

        // Emit For instruction: ABx format
        // A = result register, Bx = loop_info constant index
        self.chunk.emit(
            Instruction::abx(OpCode::For, result_reg, loop_info_idx),
            0,
        );

        Ok(result_reg)
    }

    /// Compile a method call
    ///
    /// OPT-014: Hybrid approach - delegate to interpreter like for-loops
    /// Method calls require complex dispatch logic (stdlib, mutating methods, `DataFrame`, Actor),
    /// so we store the AST and let the VM execute via interpreter.
    fn compile_method_call(&mut self, receiver: &Expr, method: &str, args: &[Expr]) -> Result<u8, String> {
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
    fn compile_match(&mut self, expr: &Expr, arms: &[crate::frontend::ast::MatchArm]) -> Result<u8, String> {
        // Store match expression AST in chunk for interpreter access
        let match_idx = self.chunk.match_exprs.len();
        self.chunk.match_exprs.push((
            Arc::new(expr.clone()),
            arms.to_vec(),
        ));

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
    fn compile_closure(&mut self, params: &[crate::frontend::ast::Param], body: &Expr) -> Result<u8, String> {
        // Extract parameter names
        let param_names: Vec<String> = params
            .iter()
            .map(crate::frontend::ast::Param::name)
            .collect();

        // Store closure definition in chunk for runtime access
        let closure_idx = self.chunk.closures.len();
        self.chunk.closures.push((
            param_names,
            Arc::new(body.clone()),
        ));

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
        self.chunk.emit(Instruction::abc(OpCode::Return, self.last_result, 0, 0), 0);

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
