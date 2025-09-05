//! Direct-threaded interpreter for efficient bytecode execution
//! Extracted from interpreter.rs for modularity (complexity: ≤10 per function)

use super::value::Value;
use super::error::{InterpreterError, InterpreterResult};
use std::collections::HashMap;

/// Bytecode instruction set
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Load constant
    LoadConst(Value),
    /// Load variable
    LoadVar(String),
    /// Store variable
    StoreVar(String),
    /// Binary operation
    BinaryOp(BinaryOp),
    /// Unary operation
    UnaryOp(UnaryOp),
    /// Jump to address
    Jump(usize),
    /// Jump if false
    JumpIfFalse(usize),
    /// Jump if true
    JumpIfTrue(usize),
    /// Call function
    Call(usize), // argument count
    /// Return from function
    Return,
    /// Pop value from stack
    Pop,
    /// Duplicate top of stack
    Dup,
    /// Build array
    BuildArray(usize),
    /// Build tuple
    BuildTuple(usize),
    /// Index operation
    Index,
}

/// Binary operations for bytecode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

/// Unary operations for bytecode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// Operand for instructions
#[derive(Debug, Clone)]
pub enum Operand {
    Value(Value),
    Variable(String),
    Address(usize),
    Count(usize),
}

/// Direct-threaded interpreter state
pub struct DirectThreadedInterpreter {
    /// Instruction stream
    instructions: Vec<Instruction>,
    /// Program counter
    pc: usize,
    /// Value stack
    stack: Vec<Value>,
    /// Local variables
    locals: HashMap<String, Value>,
    /// Call stack for function returns
    call_stack: Vec<usize>,
    /// Maximum stack size
    max_stack_size: usize,
}

impl DirectThreadedInterpreter {
    /// Create new threaded interpreter
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            pc: 0,
            stack: Vec::new(),
            locals: HashMap::new(),
            call_stack: Vec::new(),
            max_stack_size: 10000,
        }
    }

    /// Compile expression to bytecode
    pub fn compile(&mut self, expr: &crate::frontend::ast::Expr) -> InterpreterResult<()> {
        self.compile_expr(expr)?;
        self.instructions.push(Instruction::Return);
        Ok(())
    }

    /// Execute compiled bytecode
    pub fn execute(&mut self) -> InterpreterResult<Value> {
        self.pc = 0;
        self.stack.clear();
        
        while self.pc < self.instructions.len() {
            self.execute_instruction()?;
        }
        
        self.pop_stack()
    }

    /// Execute single instruction
    fn execute_instruction(&mut self) -> InterpreterResult<()> {
        let instruction = self.instructions[self.pc].clone();
        self.pc += 1;
        
        match instruction {
            Instruction::LoadConst(val) => self.push_stack(val),
            Instruction::LoadVar(name) => self.execute_load_var(name),
            Instruction::StoreVar(name) => self.execute_store_var(name),
            Instruction::BinaryOp(op) => self.execute_binary_op(op),
            Instruction::UnaryOp(op) => self.execute_unary_op(op),
            Instruction::Jump(addr) => self.pc = addr,
            Instruction::JumpIfFalse(addr) => self.execute_jump_if_false(addr),
            Instruction::JumpIfTrue(addr) => self.execute_jump_if_true(addr),
            Instruction::Call(argc) => self.execute_call(argc),
            Instruction::Return => self.execute_return(),
            Instruction::Pop => { self.pop_stack()?; Ok(()) }
            Instruction::Dup => self.execute_dup(),
            Instruction::BuildArray(count) => self.execute_build_array(count),
            Instruction::BuildTuple(count) => self.execute_build_tuple(count),
            Instruction::Index => self.execute_index(),
        }
    }

    /// Push value onto stack
    fn push_stack(&mut self, value: Value) -> InterpreterResult<()> {
        if self.stack.len() >= self.max_stack_size {
            return Err(InterpreterError::StackOverflow);
        }
        self.stack.push(value);
        Ok(())
    }

    /// Pop value from stack
    fn pop_stack(&mut self) -> InterpreterResult<Value> {
        self.stack.pop().ok_or_else(|| {
            InterpreterError::runtime("Stack underflow")
        })
    }

    /// Execute load variable
    fn execute_load_var(&mut self, name: String) -> InterpreterResult<()> {
        let value = self.locals
            .get(&name)
            .cloned()
            .ok_or_else(|| InterpreterError::undefined_variable(&name))?;
        self.push_stack(value)
    }

    /// Execute store variable
    fn execute_store_var(&mut self, name: String) -> InterpreterResult<()> {
        let value = self.pop_stack()?;
        self.locals.insert(name, value);
        Ok(())
    }

    /// Execute binary operation
    fn execute_binary_op(&mut self, op: BinaryOp) -> InterpreterResult<()> {
        let right = self.pop_stack()?;
        let left = self.pop_stack()?;
        
        let result = match op {
            BinaryOp::Add => left.add(&right),
            BinaryOp::Sub => left.subtract(&right),
            BinaryOp::Mul => left.multiply(&right),
            BinaryOp::Div => left.divide(&right),
            BinaryOp::Mod => left.modulo(&right),
            BinaryOp::Pow => left.power(&right),
            BinaryOp::Eq => Ok(Value::Bool(left.equals(&right))),
            BinaryOp::Ne => Ok(Value::Bool(!left.equals(&right))),
            BinaryOp::Lt => compare_values(&left, &right, |ord| ord == std::cmp::Ordering::Less),
            BinaryOp::Le => compare_values(&left, &right, |ord| ord != std::cmp::Ordering::Greater),
            BinaryOp::Gt => compare_values(&left, &right, |ord| ord == std::cmp::Ordering::Greater),
            BinaryOp::Ge => compare_values(&left, &right, |ord| ord != std::cmp::Ordering::Less),
            BinaryOp::And => Ok(Value::Bool(left.is_truthy() && right.is_truthy())),
            BinaryOp::Or => Ok(Value::Bool(left.is_truthy() || right.is_truthy())),
        }.map_err(InterpreterError::runtime)?;
        
        self.push_stack(result)
    }

    /// Execute unary operation
    fn execute_unary_op(&mut self, op: UnaryOp) -> InterpreterResult<()> {
        let operand = self.pop_stack()?;
        
        let result = match op {
            UnaryOp::Neg => negate_value(&operand),
            UnaryOp::Not => Ok(Value::Bool(!operand.is_truthy())),
        }.map_err(InterpreterError::runtime)?;
        
        self.push_stack(result)
    }

    /// Execute conditional jump
    fn execute_jump_if_false(&mut self, addr: usize) -> InterpreterResult<()> {
        let condition = self.pop_stack()?;
        if !condition.is_truthy() {
            self.pc = addr;
        }
        Ok(())
    }

    /// Execute conditional jump
    fn execute_jump_if_true(&mut self, addr: usize) -> InterpreterResult<()> {
        let condition = self.pop_stack()?;
        if condition.is_truthy() {
            self.pc = addr;
        }
        Ok(())
    }

    /// Execute function call
    fn execute_call(&mut self, _argc: usize) -> InterpreterResult<()> {
        // Simplified for now - would handle actual function calls
        Err(InterpreterError::runtime("Function calls not yet implemented"))
    }

    /// Execute return
    fn execute_return(&mut self) -> InterpreterResult<()> {
        if let Some(return_addr) = self.call_stack.pop() {
            self.pc = return_addr;
        } else {
            self.pc = self.instructions.len(); // End execution
        }
        Ok(())
    }

    /// Execute duplicate
    fn execute_dup(&mut self) -> InterpreterResult<()> {
        let value = self.stack.last().cloned().ok_or_else(|| {
            InterpreterError::runtime("Cannot duplicate: stack empty")
        })?;
        self.push_stack(value)
    }

    /// Execute build array
    fn execute_build_array(&mut self, count: usize) -> InterpreterResult<()> {
        let mut elements = Vec::with_capacity(count);
        for _ in 0..count {
            elements.push(self.pop_stack()?);
        }
        elements.reverse();
        self.push_stack(Value::from_array(elements))
    }

    /// Execute build tuple
    fn execute_build_tuple(&mut self, count: usize) -> InterpreterResult<()> {
        let mut elements = Vec::with_capacity(count);
        for _ in 0..count {
            elements.push(self.pop_stack()?);
        }
        elements.reverse();
        self.push_stack(Value::from_tuple(elements))
    }

    /// Execute index operation
    fn execute_index(&mut self) -> InterpreterResult<()> {
        let index = self.pop_stack()?;
        let collection = self.pop_stack()?;
        
        let result = perform_index(&collection, &index)
            .map_err(InterpreterError::runtime)?;
        
        self.push_stack(result)
    }

    /// Compile expression to bytecode (simplified)
    fn compile_expr(&mut self, expr: &crate::frontend::ast::Expr) -> InterpreterResult<()> {
        use crate::frontend::ast::{ExprKind, Literal};
        
        match &expr.kind {
            ExprKind::Literal(lit) => {
                let value = match lit {
                    Literal::Integer(i) => Value::Integer(*i),
                    Literal::Float(f) => Value::Float(*f),
                    Literal::Bool(b) => Value::Bool(*b),
                    Literal::String(s) => Value::from_string(s.clone()),
                    Literal::None => Value::Nil,
                    _ => return Err(InterpreterError::runtime("Unsupported literal")),
                };
                self.instructions.push(Instruction::LoadConst(value));
                Ok(())
            }
            ExprKind::Identifier(name) => {
                self.instructions.push(Instruction::LoadVar(name.clone()));
                Ok(())
            }
            _ => Err(InterpreterError::runtime("Expression compilation not implemented")),
        }
    }
}

impl Default for DirectThreadedInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions (complexity: ≤10)

fn compare_values<F>(left: &Value, right: &Value, op: F) -> Result<Value, String>
where
    F: FnOnce(std::cmp::Ordering) -> bool,
{
    match left.compare(right) {
        Some(ord) => Ok(Value::Bool(op(ord))),
        None => Err(format!("Cannot compare {} and {}", left.type_name(), right.type_name())),
    }
}

fn negate_value(value: &Value) -> Result<Value, String> {
    match value {
        Value::Integer(i) => Ok(Value::Integer(-i)),
        Value::Float(f) => Ok(Value::Float(-f)),
        _ => Err(format!("Cannot negate {}", value.type_name())),
    }
}

fn perform_index(collection: &Value, index: &Value) -> Result<Value, String> {
    match (collection, index) {
        (Value::Array(arr), Value::Integer(i)) => {
            let idx = if *i < 0 {
                (arr.len() as i64 + i) as usize
            } else {
                *i as usize
            };
            
            arr.get(idx)
                .cloned()
                .ok_or_else(|| format!("Index {} out of bounds", i))
        }
        _ => Err(format!("Cannot index {} with {}", collection.type_name(), index.type_name())),
    }
}