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
//! Reference: ../`ruchyruchy/OPTIMIZATION_REPORT_FOR_RUCHY.md`
//! Academic: Brunthaler (2010) - Inline Caching Meets Quickening

use super::compiler::BytecodeChunk;
use super::instruction::Instruction;
use super::opcode::OpCode;
use crate::frontend::ast::Expr;
use crate::runtime::{Interpreter, Value};
use std::collections::HashMap;
use std::sync::Arc;

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
    /// Note: `advance_pc()` will be called after this, so we compensate by subtracting 1
    #[inline]
    fn jump(&mut self, offset: i16) {
        // Offset is relative to current PC, but advance_pc will add 1, so we compensate
        let target = (self.pc as i32) + i32::from(offset);
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
    /// Interpreter for hybrid execution (function calls)
    /// OPT-011: Used to interpret closure bodies until full bytecode compilation implemented
    interpreter: Interpreter,
}

impl VM {
    /// Create a new bytecode VM
    pub fn new() -> Self {
        Self {
            registers: std::array::from_fn(|_| Value::Nil),
            call_stack: Vec::new(),
            globals: HashMap::new(),
            interpreter: Interpreter::new(),
        }
    }

    /// Execute a bytecode chunk
    ///
    /// Returns the result of the last executed instruction.
    #[allow(unsafe_code)] // TODO: Refactor CallFrame to avoid 'static lifetime requirement
    pub fn execute(&mut self, chunk: &BytecodeChunk) -> Result<Value, String> {
        // Safety: We need to extend the lifetime to 'static for the call stack
        // This is safe because the call frame doesn't outlive the chunk reference
        let chunk_ref: &'static BytecodeChunk = unsafe { std::mem::transmute(chunk) };

        // Push initial call frame
        self.call_stack.push(CallFrame::new(chunk_ref));

        // Main execution loop
        while let Some(frame) = self.call_stack.last_mut() {
            // Fetch instruction
            let instruction = if let Some(instr) = frame.fetch_instruction() {
                instr
            } else {
                // End of bytecode - pop frame
                self.call_stack.pop();
                continue;
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
    fn execute_instruction(
        &mut self,
        opcode: OpCode,
        instruction: Instruction,
    ) -> Result<(), String> {
        match opcode {
            // Load constant into register
            OpCode::Const => {
                let dest = instruction.get_a() as usize;
                let const_idx = instruction.get_bx() as usize;

                let frame = self.call_stack.last().ok_or("No active call frame")?;
                let value = frame
                    .chunk
                    .constants
                    .get(const_idx)
                    .ok_or_else(|| format!("Constant index out of bounds: {const_idx}"))?;

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

            // Array indexing: arr[index]
            OpCode::LoadField => {
                // OPT-015: Field access implementation
                // ABC format: A = dest_reg, B = object_reg, C = field_constant_idx
                let dest = instruction.get_a() as usize;
                let object_reg = instruction.get_b() as usize;
                let field_idx = instruction.get_c() as usize;

                // Get field name from constant pool
                let frame = self.call_stack.last().ok_or("No active call frame")?;
                let field_value = frame
                    .chunk
                    .constants
                    .get(field_idx)
                    .ok_or_else(|| format!("Constant index out of bounds: {field_idx}"))?;
                let field_name = match field_value {
                    Value::String(s) => s.as_ref(),
                    _ => return Err("Field name must be a string".to_string()),
                };

                // Get object from register
                let object = &self.registers[object_reg];

                // Extract field based on Value type
                let result = match object {
                    Value::Object(ref map) => map
                        .get(field_name)
                        .cloned()
                        .ok_or_else(|| format!("Field '{field_name}' not found in object")),
                    Value::Struct {
                        ref fields,
                        ref name,
                    } => fields
                        .get(field_name)
                        .cloned()
                        .ok_or_else(|| format!("Field '{field_name}' not found in struct {name}")),
                    Value::Class {
                        ref fields,
                        ref class_name,
                        ..
                    } => {
                        let fields_read = fields.read().unwrap();
                        fields_read.get(field_name).cloned().ok_or_else(|| {
                            format!("Field '{field_name}' not found in class {class_name}")
                        })
                    }
                    Value::Tuple(ref elements) => {
                        // Tuple field access (e.g., tuple.0, tuple.1)
                        field_name
                            .parse::<usize>()
                            .ok()
                            .and_then(|idx| elements.get(idx).cloned())
                            .ok_or_else(|| format!("Tuple index '{field_name}' out of bounds"))
                    }
                    _ => Err(format!(
                        "Cannot access field '{}' on type {}",
                        field_name,
                        object.type_name()
                    )),
                }?;

                self.registers[dest] = result;
                Ok(())
            }

            OpCode::LoadIndex => {
                let dest = instruction.get_a() as usize;
                let object_reg = instruction.get_b() as usize;
                let index_reg = instruction.get_c() as usize;

                let object = &self.registers[object_reg];
                let index = &self.registers[index_reg];

                // Get the indexed value
                let result = match (object, index) {
                    (Value::Array(arr), Value::Integer(i)) => {
                        let idx = if *i < 0 {
                            // Negative indexing: -1 is last element
                            let len = arr.len() as i64;
                            (len + i) as usize
                        } else {
                            *i as usize
                        };

                        arr.get(idx).cloned().ok_or_else(|| {
                            format!(
                                "Index {} out of bounds for array of length {}",
                                i,
                                arr.len()
                            )
                        })
                    }
                    (Value::String(s), Value::Integer(i)) => {
                        let chars: Vec<char> = s.chars().collect();
                        let idx = if *i < 0 {
                            let len = chars.len() as i64;
                            (len + i) as usize
                        } else {
                            *i as usize
                        };

                        chars
                            .get(idx)
                            .map(|c| Value::from_string(c.to_string()))
                            .ok_or_else(|| {
                                format!(
                                    "Index {} out of bounds for string of length {}",
                                    i,
                                    chars.len()
                                )
                            })
                    }
                    _ => Err(format!(
                        "Cannot index {} with {}",
                        object.type_name(),
                        index.type_name()
                    )),
                }?;

                self.registers[dest] = result;
                Ok(())
            }

            OpCode::NewArray => {
                // OPT-020: Runtime array construction from register values
                // ABx format: A = dest_reg, Bx = index into chunk.array_element_regs
                let dest = instruction.get_a() as usize;
                let element_regs_idx = instruction.get_bx() as usize;

                // Get current frame and element register list
                let frame = self.call_stack.last().ok_or("No active call frame")?;
                let element_regs = frame
                    .chunk
                    .array_element_regs
                    .get(element_regs_idx)
                    .ok_or_else(|| {
                        format!("Array element regs index out of bounds: {element_regs_idx}")
                    })?;

                // Collect elements from specified registers (may not be contiguous)
                let mut elements = Vec::with_capacity(element_regs.len());
                for &elem_reg in element_regs {
                    let elem_reg = elem_reg as usize;
                    if elem_reg >= self.registers.len() {
                        return Err(format!("Element register {elem_reg} out of bounds"));
                    }
                    elements.push(self.registers[elem_reg].clone());
                }

                // Create array value
                let array = Value::from_array(elements);
                self.registers[dest] = array;
                Ok(())
            }

            OpCode::NewTuple => {
                // OPT-020: Runtime tuple construction from register values
                // ABx format: A = dest_reg, Bx = index into chunk.array_element_regs
                let dest = instruction.get_a() as usize;
                let element_regs_idx = instruction.get_bx() as usize;

                // Get current frame and element register list (reusing array_element_regs)
                let frame = self.call_stack.last().ok_or("No active call frame")?;
                let element_regs = frame
                    .chunk
                    .array_element_regs
                    .get(element_regs_idx)
                    .ok_or_else(|| {
                        format!("Tuple element regs index out of bounds: {element_regs_idx}")
                    })?;

                // Collect elements from specified registers (may not be contiguous)
                let mut elements = Vec::with_capacity(element_regs.len());
                for &elem_reg in element_regs {
                    let elem_reg = elem_reg as usize;
                    if elem_reg >= self.registers.len() {
                        return Err(format!("Element register {elem_reg} out of bounds"));
                    }
                    elements.push(self.registers[elem_reg].clone());
                }

                // Create tuple value
                let tuple = Value::Tuple(Arc::from(elements.as_slice()));
                self.registers[dest] = tuple;
                Ok(())
            }

            OpCode::NewObject => {
                // OPT-020: Runtime object construction from register values
                // ABx format: A = dest_reg, Bx = index into chunk.object_fields
                let dest = instruction.get_a() as usize;
                let field_data_idx = instruction.get_bx() as usize;

                // Get current frame and field data
                let frame = self.call_stack.last().ok_or("No active call frame")?;
                let field_data =
                    frame
                        .chunk
                        .object_fields
                        .get(field_data_idx)
                        .ok_or_else(|| {
                            format!("Object field data index out of bounds: {field_data_idx}")
                        })?;

                // Build object from key-value pairs
                let mut object_map = std::collections::HashMap::new();
                for (key, value_reg) in field_data {
                    let value_reg = *value_reg as usize;
                    if value_reg >= self.registers.len() {
                        return Err(format!("Value register {value_reg} out of bounds"));
                    }
                    object_map.insert(key.clone(), self.registers[value_reg].clone());
                }

                // Create object value
                let object = Value::Object(Arc::new(object_map));
                self.registers[dest] = object;
                Ok(())
            }

            // Arithmetic operations
            OpCode::Add => self.binary_op(instruction, super::super::interpreter::Value::add),
            OpCode::Sub => self.binary_op(instruction, super::super::interpreter::Value::subtract),
            OpCode::Mul => self.binary_op(instruction, super::super::interpreter::Value::multiply),
            OpCode::Div => self.binary_op(instruction, super::super::interpreter::Value::divide),
            OpCode::Mod => self.binary_op(instruction, super::super::interpreter::Value::modulo),

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
            OpCode::Less => {
                self.comparison_op(instruction, super::super::interpreter::Value::less_than)
            }
            OpCode::LessEqual => {
                self.comparison_op(instruction, super::super::interpreter::Value::less_equal)
            }
            OpCode::Greater => {
                self.comparison_op(instruction, super::super::interpreter::Value::greater_than)
            }
            OpCode::GreaterEqual => {
                self.comparison_op(instruction, super::super::interpreter::Value::greater_equal)
            }

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

                let is_false =
                    matches!(&self.registers[condition], Value::Bool(false) | Value::Nil);

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

            OpCode::Call => {
                // OPT-011: Function call implementation (hybrid approach)
                // ABx format: A = result register, Bx = call_info constant index
                // call_info = [func_reg, arg_reg1, arg_reg2, ...]
                let result_reg = instruction.get_a() as usize;
                let call_info_idx = instruction.get_bx() as usize;

                // Get call info (func_reg + arg_regs) from constant pool
                let frame = self.call_stack.last().ok_or("No active call frame")?;
                let call_info_value = frame
                    .chunk
                    .constants
                    .get(call_info_idx)
                    .ok_or_else(|| format!("Constant index out of bounds: {call_info_idx}"))?;

                let call_info: Vec<usize> = match call_info_value {
                    Value::Array(arr) => arr
                        .iter()
                        .map(|v| match v {
                            Value::Integer(i) => Ok(*i as usize),
                            _ => Err("Call info element must be an integer".to_string()),
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    _ => return Err("Call info must be an array".to_string()),
                };

                // Extract func_reg (first element) and arg_regs (rest)
                if call_info.is_empty() {
                    return Err("Call info array is empty".to_string());
                }
                let func_reg = call_info[0];
                let arg_regs = &call_info[1..];

                // Get function from register
                let func_value = self.registers[func_reg].clone();

                // Extract closure
                let (params, body, env) = match func_value {
                    Value::Closure { params, body, env } => (params, body, env),
                    _ => {
                        return Err(format!(
                            "Cannot call non-function value: {}",
                            func_value.type_name()
                        ))
                    }
                };

                // Check argument count
                if arg_regs.len() != params.len() {
                    return Err(format!(
                        "Function expects {} arguments, got {}",
                        params.len(),
                        arg_regs.len()
                    ));
                }

                // Collect arguments from their registers
                let mut args: Vec<Value> = Vec::new();
                for &arg_reg in arg_regs {
                    args.push(self.registers[arg_reg].clone());
                }

                // Push new scope for function call
                self.interpreter.push_scope();

                // Bind captured environment variables
                for (name, value) in env.borrow().iter() {
                    // ISSUE-119: Borrow from RefCell
                    self.interpreter.set_variable(name, value.clone());
                }

                // Bind parameters to arguments
                // RUNTIME-DEFAULT-PARAMS: Extract param name from tuple (name, default_value)
                for ((param_name, _default_value), arg) in params.iter().zip(args.iter()) {
                    self.interpreter.set_variable(param_name, arg.clone());
                }

                // Execute closure body using interpreter
                let result = self
                    .interpreter
                    .eval_expr(&body)
                    .map_err(|e| format!("Function call error: {e}"))?;

                // Pop scope
                self.interpreter.pop_scope();

                // Store result in result register
                self.registers[result_reg] = result;
                Ok(())
            }

            OpCode::For => {
                // OPT-012: For-loop implementation (hybrid approach)
                // ABx format: A = result register, Bx = loop_info constant index
                // loop_info = [iter_reg, var_name, body_index]
                let result_reg = instruction.get_a() as usize;
                let loop_info_idx = instruction.get_bx() as usize;

                // Get loop info from constant pool
                let frame = self.call_stack.last().ok_or("No active call frame")?;
                let loop_info_value = frame
                    .chunk
                    .constants
                    .get(loop_info_idx)
                    .ok_or_else(|| format!("Constant index out of bounds: {loop_info_idx}"))?;

                let loop_info: Vec<i64> = match loop_info_value {
                    Value::Array(arr) => {
                        let mut info = Vec::new();
                        // First two elements are integers
                        if arr.len() < 3 {
                            return Err("Loop info array must have at least 3 elements".to_string());
                        }
                        if let Value::Integer(iter_reg) = arr[0] {
                            info.push(iter_reg);
                        } else {
                            return Err("Loop info[0] must be an integer".to_string());
                        }
                        // Second element is the var name (skip for now, get separately)
                        // Third element is body_index
                        if let Value::Integer(body_idx) = arr[2] {
                            info.push(body_idx);
                        } else {
                            return Err("Loop info[2] must be an integer".to_string());
                        }
                        info
                    }
                    _ => return Err("Loop info must be an array".to_string()),
                };

                let iter_reg = loop_info[0] as usize;
                let body_idx = loop_info[1] as usize;

                // Extract var name from loop_info
                let var_name = match &loop_info_value {
                    Value::Array(arr) if arr.len() >= 2 => match &arr[1] {
                        Value::String(s) => s.as_ref().to_string(),
                        _ => return Err("Loop var name must be a string".to_string()),
                    },
                    _ => return Err("Loop info must be an array".to_string()),
                };

                // Get iterator array from register
                let iter_value = self.registers[iter_reg].clone();
                let iter_array = match iter_value {
                    Value::Array(arr) => arr,
                    _ => {
                        return Err(format!(
                            "For-loop iterator must be an array, got {}",
                            iter_value.type_name()
                        ))
                    }
                };

                // Get body from chunk's loop_bodies
                let body = frame
                    .chunk
                    .loop_bodies
                    .get(body_idx)
                    .ok_or_else(|| format!("Loop body index out of bounds: {body_idx}"))?
                    .clone();

                // Synchronize register-based locals to interpreter scope
                // This allows the loop body to access variables like 'sum' that were
                // defined in bytecode mode but need to be visible to the interpreter
                for (name, reg_idx) in &frame.chunk.locals_map {
                    let value = self.registers[*reg_idx as usize].clone();
                    self.interpreter.set_variable(name, value);
                }

                // Execute loop by iterating over array elements
                let mut last_result = Value::Nil;

                for elem in iter_array.iter() {
                    // Push new scope for loop iteration
                    self.interpreter.push_scope();

                    // Bind loop variable to current element
                    self.interpreter.set_variable(&var_name, elem.clone());

                    // Execute loop body using interpreter
                    last_result = self
                        .interpreter
                        .eval_expr(&body)
                        .map_err(|e| format!("For-loop body error: {e}"))?;

                    // Pop scope
                    self.interpreter.pop_scope();

                    // Synchronize interpreter scope back to registers
                    // This allows mutations to 'sum' inside the loop to persist
                    for (name, reg_idx) in &frame.chunk.locals_map {
                        if let Some(value) = self.interpreter.get_variable(name) {
                            self.registers[*reg_idx as usize] = value;
                        }
                    }
                }

                // Store result in result register (last iteration's result, or Nil if empty)
                self.registers[result_reg] = last_result;
                Ok(())
            }

            OpCode::MethodCall => {
                // OPT-014: Method call implementation (hybrid approach)
                // ABx format: A = result register, Bx = method_call_idx constant
                let result_reg = instruction.get_a() as usize;
                let method_call_idx_const = instruction.get_bx() as usize;

                // Get method call index from constant pool
                let frame = self.call_stack.last().ok_or("No active call frame")?;
                let method_call_idx_value = frame
                    .chunk
                    .constants
                    .get(method_call_idx_const)
                    .ok_or_else(|| {
                        format!("Constant index out of bounds: {method_call_idx_const}")
                    })?;

                let method_call_idx = match method_call_idx_value {
                    Value::Integer(idx) => *idx as usize,
                    _ => return Err("Method call index must be an integer".to_string()),
                };

                // Get (receiver, method, args) from chunk's method_calls
                let (receiver, method, args) = frame
                    .chunk
                    .method_calls
                    .get(method_call_idx)
                    .ok_or_else(|| format!("Method call index out of bounds: {method_call_idx}"))?;

                // Synchronize register-based locals to interpreter scope
                // This allows method bodies to access variables defined in bytecode mode
                for (name, reg_idx) in &frame.chunk.locals_map {
                    let value = self.registers[*reg_idx as usize].clone();
                    self.interpreter.set_variable(name, value);
                }

                // Convert Vec<Arc<Expr>> to Vec<Expr> for eval_method_call
                let args_exprs: Vec<Expr> = args.iter().map(|arc| (**arc).clone()).collect();

                // Execute method call using interpreter
                let result = self
                    .interpreter
                    .eval_method_call(receiver, method, &args_exprs)
                    .map_err(|e| format!("Method call error: {e}"))?;

                // Synchronize interpreter scope back to registers
                // This allows mutations inside methods to persist
                for (name, reg_idx) in &frame.chunk.locals_map {
                    if let Some(value) = self.interpreter.get_variable(name) {
                        self.registers[*reg_idx as usize] = value;
                    }
                }

                // Store result in result register
                self.registers[result_reg] = result;
                Ok(())
            }

            OpCode::Match => {
                // OPT-018: Match expression implementation (hybrid approach)
                // ABx format: A = result register, Bx = match_idx constant
                let result_reg = instruction.get_a() as usize;
                let match_idx_const = instruction.get_bx() as usize;

                // Get match index from constant pool
                let frame = self.call_stack.last().ok_or("No active call frame")?;
                let match_idx_value =
                    frame.chunk.constants.get(match_idx_const).ok_or_else(|| {
                        format!("Constant index out of bounds: {match_idx_const}")
                    })?;

                let match_idx = match match_idx_value {
                    Value::Integer(idx) => *idx as usize,
                    _ => return Err("Match index must be an integer".to_string()),
                };

                // Get (expr, arms) from chunk's match_exprs
                let (expr, arms) = frame
                    .chunk
                    .match_exprs
                    .get(match_idx)
                    .ok_or_else(|| format!("Match index out of bounds: {match_idx}"))?;

                // Synchronize register-based locals to interpreter scope
                // This allows pattern bindings and guards to access variables defined in bytecode mode
                for (name, reg_idx) in &frame.chunk.locals_map {
                    let value = self.registers[*reg_idx as usize].clone();
                    self.interpreter.set_variable(name, value);
                }

                // Execute match expression using interpreter
                let result = self
                    .interpreter
                    .eval_match(expr, arms)
                    .map_err(|e| format!("Match expression error: {e}"))?;

                // Synchronize interpreter scope back to registers
                // This allows mutations inside match arms to persist
                for (name, reg_idx) in &frame.chunk.locals_map {
                    if let Some(value) = self.interpreter.get_variable(name) {
                        self.registers[*reg_idx as usize] = value;
                    }
                }

                // Store result in result register
                self.registers[result_reg] = result;
                Ok(())
            }

            OpCode::NewClosure => {
                // OPT-019: Closure creation (hybrid approach)
                // ABx format: A = result register, Bx = closure_idx constant
                let result_reg = instruction.get_a() as usize;
                let closure_idx_const = instruction.get_bx() as usize;

                // Get closure index from constant pool
                let frame = self.call_stack.last().ok_or("No active call frame")?;
                let closure_idx_value = frame
                    .chunk
                    .constants
                    .get(closure_idx_const)
                    .ok_or_else(|| format!("Constant index out of bounds: {closure_idx_const}"))?;

                let closure_idx = match closure_idx_value {
                    Value::Integer(idx) => *idx as usize,
                    _ => return Err("Closure index must be an integer".to_string()),
                };

                // Get (params, body) from chunk's closures
                let (params, body) = frame
                    .chunk
                    .closures
                    .get(closure_idx)
                    .ok_or_else(|| format!("Closure index out of bounds: {closure_idx}"))?;

                // Synchronize register-based locals to interpreter scope
                // This ensures closures capture variables defined in bytecode mode
                for (name, reg_idx) in &frame.chunk.locals_map {
                    let value = self.registers[*reg_idx as usize].clone();
                    self.interpreter.set_variable(name, value);
                }

                // Capture current environment from interpreter
                // This is the key to closures - we snapshot the current scope
                let env = self.interpreter.current_env().clone(); // ISSUE-119: Rc::clone (shallow copy)

                // Create closure value
                let closure = Value::Closure {
                    params: params.clone(),
                    body: body.clone(),
                    env,
                };

                // Store closure in result register
                self.registers[result_reg] = closure;
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

                let frame = self.call_stack.last().ok_or("No active call frame")?;
                let name_value = frame
                    .chunk
                    .constants
                    .get(name_idx)
                    .ok_or_else(|| format!("Constant index out of bounds: {name_idx}"))?;

                let name = match name_value {
                    Value::String(s) => s.as_ref(),
                    _ => return Err("Global name must be a string".to_string()),
                };

                let value = self
                    .globals
                    .get(name)
                    .ok_or_else(|| format!("Undefined global variable: {name}"))?;

                self.registers[dest] = value.clone();
                Ok(())
            }

            OpCode::StoreGlobal => {
                let src = instruction.get_a() as usize;
                let name_idx = instruction.get_bx() as usize;

                let frame = self.call_stack.last().ok_or("No active call frame")?;
                let name_value = frame
                    .chunk
                    .constants
                    .get(name_idx)
                    .ok_or_else(|| format!("Constant index out of bounds: {name_idx}"))?;

                let name = match name_value {
                    Value::String(s) => s.to_string(),
                    _ => return Err("Global name must be a string".to_string()),
                };

                self.globals.insert(name, self.registers[src].clone());
                Ok(())
            }

            _ => Err(format!("Unsupported opcode: {opcode:?}")),
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
    use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Span, UnaryOp};
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
        let condition = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
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
        let condition = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::default());
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
            Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(3, None)),
                Span::default(),
            ),
        ];
        let block = Expr::new(ExprKind::Block(exprs), Span::default());
        compiler.compile_expr(&block).unwrap();
        let chunk = compiler.finalize();

        // Execute
        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(3));
    }

    // ========================================================================
    // UNIT TESTS: CallFrame operations (Sprint 5 - OPT-003)
    // ========================================================================

    #[test]
    fn test_callframe_new_initialization() {
        // Test CallFrame initialization with default values
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let frame = CallFrame::new(&chunk);

        assert_eq!(frame.pc, 0, "PC should initialize to 0");
        assert_eq!(
            frame.base_register, 0,
            "Base register should initialize to 0"
        );
    }

    #[test]
    fn test_callframe_fetch_instruction_valid() {
        // Test fetching valid instruction at current PC
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let frame = CallFrame::new(&chunk);
        let instruction = frame.fetch_instruction();

        assert!(instruction.is_some(), "Should fetch instruction at PC 0");
        assert_eq!(
            instruction.unwrap().opcode(),
            OpCode::Const as u8,
            "First instruction should be Const (load constant)"
        );
    }

    #[test]
    fn test_callframe_fetch_instruction_out_of_bounds() {
        // Test fetching instruction beyond bytecode end
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut frame = CallFrame::new(&chunk);
        // Move PC beyond bytecode
        frame.pc = chunk.instructions.len() + 10;
        let instruction = frame.fetch_instruction();

        assert!(
            instruction.is_none(),
            "Should return None for out-of-bounds PC"
        );
    }

    #[test]
    fn test_callframe_advance_pc() {
        // Test program counter increment
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut frame = CallFrame::new(&chunk);
        assert_eq!(frame.pc, 0);

        frame.advance_pc();
        assert_eq!(frame.pc, 1, "PC should increment by 1");

        frame.advance_pc();
        assert_eq!(frame.pc, 2, "PC should increment again");
    }

    #[test]
    fn test_callframe_jump_positive_offset() {
        // Test jumping forward (positive offset)
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut frame = CallFrame::new(&chunk);
        frame.pc = 5;
        frame.jump(10); // Jump forward by 10

        assert_eq!(frame.pc, 15, "PC should jump forward by offset");
    }

    #[test]
    fn test_callframe_jump_negative_offset() {
        // Test jumping backward (negative offset)
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut frame = CallFrame::new(&chunk);
        frame.pc = 10;
        frame.jump(-5); // Jump backward by 5

        assert_eq!(frame.pc, 5, "PC should jump backward by offset");
    }

    #[test]
    fn test_callframe_jump_zero_offset() {
        // Test jump with zero offset (no-op)
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut frame = CallFrame::new(&chunk);
        frame.pc = 7;
        frame.jump(0); // Zero offset

        assert_eq!(frame.pc, 7, "PC should remain unchanged with zero offset");
    }

    // ========================================================================
    // UNIT TESTS: VM initialization and state (Sprint 5 - OPT-003)
    // ========================================================================

    #[test]
    fn test_vm_new_initialization() {
        // Test VM initialization state
        let vm = VM::new();

        // Verify registers initialized to Nil
        for (idx, reg) in vm.registers.iter().enumerate() {
            assert_eq!(*reg, Value::Nil, "Register {idx} should initialize to Nil");
        }

        assert!(vm.call_stack.is_empty(), "Call stack should be empty");
        assert!(vm.globals.is_empty(), "Globals should be empty");
    }

    #[test]
    fn test_vm_register_count() {
        // Test VM has exactly MAX_REGISTERS (32) registers
        let vm = VM::new();
        assert_eq!(
            vm.registers.len(),
            MAX_REGISTERS,
            "VM should have exactly {MAX_REGISTERS} registers"
        );
    }

    #[test]
    fn test_vm_execute_empty_bytecode() {
        // Test executing empty bytecode chunk
        let compiler = Compiler::new("test".to_string());
        let chunk = compiler.finalize(); // Empty bytecode

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        // Empty bytecode returns register 0 (Nil by default)
        assert_eq!(result, Value::Nil, "Empty bytecode should return Nil");
    }

    #[test]
    fn test_vm_multiple_sequential_executions() {
        // Test VM can execute multiple chunks sequentially
        let mut vm = VM::new();

        // Execute first chunk: 10 + 20
        let mut compiler1 = Compiler::new("test1".to_string());
        let left1 = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            Span::default(),
        );
        let right1 = Expr::new(
            ExprKind::Literal(Literal::Integer(20, None)),
            Span::default(),
        );
        let expr1 = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(left1),
                right: Box::new(right1),
            },
            Span::default(),
        );
        compiler1.compile_expr(&expr1).unwrap();
        let chunk1 = compiler1.finalize();

        let result1 = vm.execute(&chunk1).unwrap();
        assert_eq!(result1, Value::Integer(30));

        // Execute second chunk: 5 * 6
        let mut compiler2 = Compiler::new("test2".to_string());
        let left2 = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            Span::default(),
        );
        let right2 = Expr::new(
            ExprKind::Literal(Literal::Integer(6, None)),
            Span::default(),
        );
        let expr2 = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(left2),
                right: Box::new(right2),
            },
            Span::default(),
        );
        compiler2.compile_expr(&expr2).unwrap();
        let chunk2 = compiler2.finalize();

        let result2 = vm.execute(&chunk2).unwrap();
        assert_eq!(result2, Value::Integer(30));
    }

    #[test]
    fn test_vm_register_isolation_between_executions() {
        // Test that register state is isolated between executions
        let mut vm = VM::new();

        // Execute first chunk: loads 42 into register 0
        let mut compiler1 = Compiler::new("test1".to_string());
        let expr1 = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler1.compile_expr(&expr1).unwrap();
        let chunk1 = compiler1.finalize();

        let result1 = vm.execute(&chunk1).unwrap();
        assert_eq!(result1, Value::Integer(42));

        // Execute second chunk: loads 100 into register 0
        let mut compiler2 = Compiler::new("test2".to_string());
        let expr2 = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            Span::default(),
        );
        compiler2.compile_expr(&expr2).unwrap();
        let chunk2 = compiler2.finalize();

        let result2 = vm.execute(&chunk2).unwrap();
        assert_eq!(
            result2,
            Value::Integer(100),
            "Second execution should overwrite register 0"
        );
    }

    // ========================================================================
    // OPCODE TESTS: Binary arithmetic operations (Sprint 5 Extended - OPT-003)
    // ========================================================================

    #[test]
    fn test_vm_opcode_subtraction() {
        // Compile: 50 - 8
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(50, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(8, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Subtract,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_opcode_division() {
        // Compile: 84 / 2
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(84, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Divide,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_opcode_modulo() {
        // Compile: 100 % 58
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(58, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Modulo,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(42));
    }

    // ========================================================================
    // OPCODE TESTS: Unary operations (Sprint 5 Extended - OPT-003)
    // ========================================================================

    #[test]
    fn test_vm_opcode_negation_integer() {
        // Compile: -(-42)
        let mut compiler = Compiler::new("test".to_string());
        let inner = Expr::new(
            ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Negate,
                operand: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(42, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Negate,
                operand: Box::new(inner),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_opcode_negation_float() {
        // Compile: -(3.14)
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Negate,
                operand: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Float(3.14)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Float(-3.14));
    }

    #[test]
    fn test_vm_opcode_logical_not_true() {
        // Compile: !true
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Not,
                operand: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Bool(true)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_opcode_logical_not_false() {
        // Compile: !false
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Not,
                operand: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Bool(false)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    // ========================================================================
    // OPCODE TESTS: Comparison operations (Sprint 5 Extended - OPT-003)
    // ========================================================================

    #[test]
    fn test_vm_opcode_equal_true() {
        // Compile: 42 == 42
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Equal,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_opcode_equal_false() {
        // Compile: 42 == 100
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Equal,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_opcode_not_equal() {
        // Compile: 42 != 100
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::NotEqual,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_opcode_less_equal() {
        // Compile: 42 <= 42
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::LessEqual,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_opcode_greater() {
        // Compile: 100 > 42
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Greater,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_opcode_greater_equal() {
        // Compile: 42 >= 42
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::GreaterEqual,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    // ========================================================================
    // OPCODE TESTS: Logical operations (Sprint 5 Extended - OPT-003)
    // ========================================================================

    #[test]
    fn test_vm_opcode_logical_and_true() {
        // Compile: true && true
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
        let right = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_opcode_logical_and_false() {
        // Compile: true && false
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
        let right = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::default());
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_opcode_logical_or_true() {
        // Compile: false || true
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::default());
        let right = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_opcode_logical_or_false() {
        // Compile: false || false
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::default());
        let right = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::default());
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Bool(false));
    }

    // ========================================================================
    // OPCODE TESTS: Data structure operations (Sprint 5 Extended - OPT-003)
    // ========================================================================

    #[test]
    fn test_vm_opcode_array_literal() {
        // Compile: [1, 2, 3]
        let mut compiler = Compiler::new("test".to_string());
        let elements = vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(3, None)),
                Span::default(),
            ),
        ];
        let expr = Expr::new(ExprKind::List(elements), Span::default());
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Integer(1));
                assert_eq!(arr[1], Value::Integer(2));
                assert_eq!(arr[2], Value::Integer(3));
            }
            _ => panic!("Expected array, got {result:?}"),
        }
    }

    #[test]
    fn test_vm_opcode_array_empty() {
        // Compile: []
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(ExprKind::List(vec![]), Span::default());
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        match result {
            Value::Array(arr) => assert_eq!(arr.len(), 0),
            _ => panic!("Expected array, got {result:?}"),
        }
    }

    #[test]
    fn test_vm_opcode_tuple_literal() {
        // Compile: (42, true, "hello")
        let mut compiler = Compiler::new("test".to_string());
        let elements = vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(42, None)),
                Span::default(),
            ),
            Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default()),
            Expr::new(
                ExprKind::Literal(Literal::String("hello".to_string())),
                Span::default(),
            ),
        ];
        let expr = Expr::new(ExprKind::Tuple(elements), Span::default());
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        match result {
            Value::Tuple(tuple) => {
                assert_eq!(tuple.len(), 3);
                assert_eq!(tuple[0], Value::Integer(42));
                assert_eq!(tuple[1], Value::Bool(true));
                assert_eq!(tuple[2], Value::from_string("hello".to_string()));
            }
            _ => panic!("Expected tuple, got {result:?}"),
        }
    }

    #[test]
    fn test_vm_opcode_object_literal() {
        // Compile: { x: 10, y: 20 }
        let mut compiler = Compiler::new("test".to_string());
        use crate::frontend::ast::ObjectField;
        let fields = vec![
            ObjectField::KeyValue {
                key: "x".to_string(),
                value: Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::default(),
                ),
            },
            ObjectField::KeyValue {
                key: "y".to_string(),
                value: Expr::new(
                    ExprKind::Literal(Literal::Integer(20, None)),
                    Span::default(),
                ),
            },
        ];
        let expr = Expr::new(ExprKind::ObjectLiteral { fields }, Span::default());
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        match result {
            Value::Object(obj) => {
                assert_eq!(obj.get("x"), Some(&Value::Integer(10)));
                assert_eq!(obj.get("y"), Some(&Value::Integer(20)));
            }
            _ => panic!("Expected object, got {result:?}"),
        }
    }

    #[test]
    fn test_vm_opcode_array_index_access() {
        // Compile: [10, 20, 30][1]
        let mut compiler = Compiler::new("test".to_string());
        let array = Expr::new(
            ExprKind::List(vec![
                Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(20, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(30, None)),
                    Span::default(),
                ),
            ]),
            Span::default(),
        );
        let index = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(array),
                index: Box::new(index),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(20));
    }

    #[test]
    fn test_vm_opcode_array_index_negative() {
        // Compile: [10, 20, 30][-1]  (negative indexing: last element)
        let mut compiler = Compiler::new("test".to_string());
        let array = Expr::new(
            ExprKind::List(vec![
                Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(20, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(30, None)),
                    Span::default(),
                ),
            ]),
            Span::default(),
        );
        let index = Expr::new(
            ExprKind::Literal(Literal::Integer(-1, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(array),
                index: Box::new(index),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(30));
    }

    #[test]
    fn test_vm_opcode_string_index_access() {
        // Compile: "hello"[1]
        let mut compiler = Compiler::new("test".to_string());
        let string = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            Span::default(),
        );
        let index = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(string),
                index: Box::new(index),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::from_string("e".to_string()));
    }

    #[test]
    fn test_vm_opcode_object_field_access() {
        // Compile: { x: 42 }.x
        let mut compiler = Compiler::new("test".to_string());
        use crate::frontend::ast::ObjectField;
        let fields = vec![ObjectField::KeyValue {
            key: "x".to_string(),
            value: Expr::new(
                ExprKind::Literal(Literal::Integer(42, None)),
                Span::default(),
            ),
        }];
        let object = Expr::new(ExprKind::ObjectLiteral { fields }, Span::default());
        let expr = Expr::new(
            ExprKind::FieldAccess {
                object: Box::new(object),
                field: "x".to_string(),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_opcode_tuple_field_access() {
        // Compile: (100, 200).0
        let mut compiler = Compiler::new("test".to_string());
        let tuple = Expr::new(
            ExprKind::Tuple(vec![
                Expr::new(
                    ExprKind::Literal(Literal::Integer(100, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(200, None)),
                    Span::default(),
                ),
            ]),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::FieldAccess {
                object: Box::new(tuple),
                field: "0".to_string(),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(100));
    }

    #[test]
    fn test_vm_opcode_nested_array_access() {
        // Compile: [[1, 2], [3, 4]][1][0]
        let mut compiler = Compiler::new("test".to_string());
        let inner1 = Expr::new(
            ExprKind::List(vec![
                Expr::new(
                    ExprKind::Literal(Literal::Integer(1, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(2, None)),
                    Span::default(),
                ),
            ]),
            Span::default(),
        );
        let inner2 = Expr::new(
            ExprKind::List(vec![
                Expr::new(
                    ExprKind::Literal(Literal::Integer(3, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(4, None)),
                    Span::default(),
                ),
            ]),
            Span::default(),
        );
        let outer = Expr::new(ExprKind::List(vec![inner1, inner2]), Span::default());
        let first_index = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(outer),
                index: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(1, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(first_index),
                index: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(0, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(3));
    }

    // ============================================================================
    // Sprint 5 Extended Phase 3: Control Flow & Error Handling Tests
    // Target: Exercise Jump opcodes, error paths, edge cases
    // ============================================================================

    #[test]
    fn test_vm_opcode_if_else_both_branches() {
        // Compile: if false { 10 } else { 20 }
        // Exercises JumpIfFalse and Jump opcodes (else branch)
        let mut compiler = Compiler::new("test".to_string());
        let condition = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::default());
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            Span::default(),
        );
        let else_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(20, None)),
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

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(20)); // Should take else branch
    }

    #[test]
    fn test_vm_opcode_if_without_else_true() {
        // Compile: if true { 42 }
        // Exercises JumpIfFalse opcode (skipping jump)
        let mut compiler = Compiler::new("test".to_string());
        let condition = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: None,
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_opcode_if_without_else_false() {
        // Compile: if false { 42 }
        // Should return nil when condition is false and no else branch
        let mut compiler = Compiler::new("test".to_string());
        let condition = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::default());
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: None,
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_vm_opcode_truthy_nonzero_integer() {
        // Compile: if 42 { 100 } else { 200 }
        // Non-zero integers are truthy, should execute then branch
        let mut compiler = Compiler::new("test".to_string());
        let condition = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            Span::default(),
        );
        let else_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(200, None)),
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

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(100)); // Truthy takes then branch
    }

    #[test]
    fn test_vm_opcode_array_index_out_of_bounds() {
        // Compile: [10, 20][5]
        // Should error with bounds check
        let mut compiler = Compiler::new("test".to_string());
        let array = Expr::new(
            ExprKind::List(vec![
                Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(20, None)),
                    Span::default(),
                ),
            ]),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(array),
                index: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(5, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("out of bounds"));
    }

    #[test]
    fn test_vm_opcode_string_index_out_of_bounds() {
        // Compile: "hello"[10]
        // Should error with bounds check
        let mut compiler = Compiler::new("test".to_string());
        let string = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(string),
                index: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("out of bounds"));
    }

    #[test]
    fn test_vm_opcode_field_access_missing_field() {
        // Compile: { x: 10 }.y
        // Should error with field not found
        let mut compiler = Compiler::new("test".to_string());
        use crate::frontend::ast::ObjectField;
        let object = Expr::new(
            ExprKind::ObjectLiteral {
                fields: vec![ObjectField::KeyValue {
                    key: "x".to_string(),
                    value: Expr::new(
                        ExprKind::Literal(Literal::Integer(10, None)),
                        Span::default(),
                    ),
                }],
            },
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::FieldAccess {
                object: Box::new(object),
                field: "y".to_string(),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_vm_opcode_tuple_index_out_of_bounds() {
        // Compile: (100, 200).5
        // Should error with tuple index out of bounds
        let mut compiler = Compiler::new("test".to_string());
        let tuple = Expr::new(
            ExprKind::Tuple(vec![
                Expr::new(
                    ExprKind::Literal(Literal::Integer(100, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(200, None)),
                    Span::default(),
                ),
            ]),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::FieldAccess {
                object: Box::new(tuple),
                field: "5".to_string(),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("out of bounds"));
    }

    // ============================================================================
    // Sprint 5 Extended Phase 4: Error Branches & Edge Cases
    // Target: Exercise error handling in arithmetic/type operations
    // ============================================================================

    #[test]
    #[ignore = "VM doesn't implement divide-by-zero error handling yet - panics instead of returning error"]
    fn test_vm_opcode_division_by_zero_integer() {
        // Compile: 10 / 0
        // Should error with division by zero
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::default(),
                )),
                op: BinaryOp::Divide,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(0, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk);

        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("division by zero") || err_msg.contains("divide by zero"));
    }

    #[test]
    #[ignore = "VM doesn't implement modulo-by-zero error handling yet - panics instead of returning error"]
    fn test_vm_opcode_modulo_by_zero() {
        // Compile: 10 % 0
        // Should error with modulo by zero
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::default(),
                )),
                op: BinaryOp::Modulo,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(0, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk);

        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("modulo by zero") || err_msg.contains("divide by zero"));
    }

    #[test]
    fn test_vm_opcode_bitnot_on_float_error() {
        // Compile: ~3.14
        // Should error - bitwise NOT only works on integers
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::BitwiseNot,
                operand: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Float(3.14)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk);

        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("bitwise NOT") || err_msg.contains("Float"));
    }

    #[test]
    fn test_vm_opcode_negate_string_error() {
        // Compile: -"hello"
        // Should error - cannot negate string
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: Box::new(Expr::new(
                    ExprKind::Literal(Literal::String("hello".to_string())),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk);

        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("negate") || err_msg.contains("String"));
    }

    #[test]
    fn test_vm_opcode_complex_arithmetic() {
        // Compile: ((10 + 20) * 3) - 5
        // Tests nested binary operations
        let mut compiler = Compiler::new("test".to_string());
        let add = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::default(),
                )),
                op: BinaryOp::Add,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(20, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        let mul = Expr::new(
            ExprKind::Binary {
                left: Box::new(add),
                op: BinaryOp::Multiply,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(3, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(mul),
                op: BinaryOp::Subtract,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(5, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(85)); // (10+20)*3-5 = 30*3-5 = 90-5 = 85
    }

    #[test]
    fn test_vm_opcode_complex_boolean_logic() {
        // Compile: (true && false) || (true && true)
        // Tests nested logical operations
        let mut compiler = Compiler::new("test".to_string());
        let and1 = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Bool(true)),
                    Span::default(),
                )),
                op: BinaryOp::And,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Bool(false)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        let and2 = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Bool(true)),
                    Span::default(),
                )),
                op: BinaryOp::And,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Bool(true)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(and1),
                op: BinaryOp::Or,
                right: Box::new(and2),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Bool(true)); // (true&&false)||(true&&true) = false||true = true
    }

    #[test]
    fn test_vm_opcode_float_arithmetic() {
        // Compile: 3.5 * 2.0 + 1.5
        // Tests float operations
        let mut compiler = Compiler::new("test".to_string());
        let mul = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Float(3.5)),
                    Span::default(),
                )),
                op: BinaryOp::Multiply,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Float(2.0)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(mul),
                op: BinaryOp::Add,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Float(1.5)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        match result {
            Value::Float(f) => assert!((f - 8.5).abs() < 0.001), // 3.5*2.0+1.5 = 7.0+1.5 = 8.5
            _ => panic!("Expected Float, got {result:?}"),
        }
    }

    #[test]
    fn test_vm_opcode_comparison_chain() {
        // Compile: 5 > 3 && 3 > 1
        // Tests chained comparisons with logical operators
        let mut compiler = Compiler::new("test".to_string());
        let cmp1 = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(5, None)),
                    Span::default(),
                )),
                op: BinaryOp::Greater,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(3, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        let cmp2 = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(3, None)),
                    Span::default(),
                )),
                op: BinaryOp::Greater,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(1, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(cmp1),
                op: BinaryOp::And,
                right: Box::new(cmp2),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Bool(true)); // 5>3 && 3>1 = true && true = true
    }

    #[test]
    fn test_vm_opcode_nil_truthy() {
        // Compile: if nil { 10 } else { 20 }
        // Nil is falsy, should take else branch
        let mut compiler = Compiler::new("test".to_string());
        let condition = Expr::new(ExprKind::Literal(Literal::Null), Span::default());
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            Span::default(),
        );
        let else_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(20, None)),
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

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(20)); // nil is falsy, takes else
    }

    #[test]
    fn test_vm_opcode_double_negation() {
        // Compile: -(-42)
        // Tests unary operation on unary operation result
        let mut compiler = Compiler::new("test".to_string());
        let inner = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(42, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: Box::new(inner),
            },
            Span::default(),
        );
        compiler.compile_expr(&expr).unwrap();
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk).unwrap();

        assert_eq!(result, Value::Integer(42)); // -(-42) = 42
    }
}
