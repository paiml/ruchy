//! High-Performance Interpreter with Safe Value Representation
//!
//! This module implements the two-tier execution strategy from ruchy-interpreter-spec.md:
//! - Tier 0: AST interpreter with enum-based values (safe alternative)
//! - Tier 1: JIT compilation (future)
//!
//! Uses safe Rust enum approach instead of tagged pointers to respect `unsafe_code = "forbid"`.

#![allow(clippy::unused_self)]      // Methods will use self in future phases
#![allow(clippy::only_used_in_recursion)]  // Recursive print_value is intentional
#![allow(clippy::uninlined_format_args)]   // Some format strings are clearer unexpanded

use std::collections::HashMap;
use std::rc::Rc;
#[cfg(test)]
use crate::frontend::Param;
use crate::frontend::ast::{Expr, ExprKind, Literal, BinaryOp as AstBinaryOp};

/// Runtime value representation using safe enum approach
/// Alternative to tagged pointers that respects project's `unsafe_code = "forbid"`
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// 64-bit signed integer
    Integer(i64),
    /// 64-bit float
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// Nil/null value
    Nil,
    /// String value (reference-counted for efficiency)
    String(Rc<String>),
    /// Array of values
    Array(Rc<Vec<Value>>),
    /// Function closure
    Closure {
        params: Vec<String>,
        body: Rc<Expr>,
        env: Rc<HashMap<String, Value>>, // Captured environment
    },
}

impl Value {
    /// Create integer value
    pub fn from_i64(i: i64) -> Self {
        Value::Integer(i)
    }

    /// Create float value
    pub fn from_f64(f: f64) -> Self {
        Value::Float(f)
    }

    /// Create boolean value
    pub fn from_bool(b: bool) -> Self {
        Value::Bool(b)
    }

    /// Create nil value
    pub fn nil() -> Self {
        Value::Nil
    }

    /// Create string value
    pub fn from_string(s: String) -> Self {
        Value::String(Rc::new(s))
    }

    /// Create array value
    pub fn from_array(arr: Vec<Value>) -> Self {
        Value::Array(Rc::new(arr))
    }

    /// Check if value is nil
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    /// Check if value is truthy (everything except false and nil)
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nil => false,
            _ => true,
        }
    }

    /// Extract integer value
    /// # Errors
    /// Returns error if the value is not an integer
    pub fn as_i64(&self) -> Result<i64, InterpreterError> {
        match self {
            Value::Integer(i) => Ok(*i),
            _ => Err(InterpreterError::TypeError(format!("Expected integer, got {}", self.type_name()))),
        }
    }

    /// Extract float value  
    /// # Errors
    /// Returns error if the value is not a float
    pub fn as_f64(&self) -> Result<f64, InterpreterError> {
        match self {
            Value::Float(f) => Ok(*f),
            _ => Err(InterpreterError::TypeError(format!("Expected float, got {}", self.type_name()))),
        }
    }

    /// Extract boolean value
    /// # Errors
    /// Returns error if the value is not a boolean
    pub fn as_bool(&self) -> Result<bool, InterpreterError> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(InterpreterError::TypeError(format!("Expected boolean, got {}", self.type_name()))),
        }
    }

    /// Get type name for debugging
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::Bool(_) => "boolean",
            Value::Nil => "nil",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Closure { .. } => "function",
        }
    }
}

// Note: Complex object structures (ObjectHeader, Class, etc.) will be implemented
// in Phase 1 of the interpreter spec when we add proper GC and method dispatch.

/// Runtime interpreter state
pub struct Interpreter {
    /// Tagged pointer values for fast operation
    stack: Vec<Value>,
    
    /// Environment stack for lexical scoping
    env_stack: Vec<HashMap<std::string::String, Value>>,
    
    /// Call frame for function calls
    #[allow(dead_code)]
    frames: Vec<CallFrame>,
    
    /// Execution statistics for tier transition (will be used in Phase 1)
    #[allow(dead_code)]
    execution_counts: HashMap<usize, u32>,  // Function/method ID -> execution count
}

/// Call frame for function invocation (will be used in Phase 1)
#[derive(Debug)]
#[allow(dead_code)]
pub struct CallFrame {
    /// Function being executed
    closure: Value,
    
    /// Instruction pointer
    ip: *const u8,
    
    /// Base of stack frame
    base: usize,
    
    /// Number of locals in this frame
    locals: usize,
}

/// Interpreter execution result
pub enum InterpreterResult {
    Continue,
    Jump(usize),
    Return(Value),
    Error(InterpreterError),
}

/// Interpreter errors
#[derive(Debug, Clone)]
pub enum InterpreterError {
    TypeError(std::string::String),
    RuntimeError(std::string::String),
    StackOverflow,
    StackUnderflow,
    InvalidInstruction,
    DivisionByZero,
    IndexOutOfBounds,
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::TypeError(msg) => write!(f, "Type error: {msg}"),
            InterpreterError::RuntimeError(msg) => write!(f, "Runtime error: {msg}"),
            InterpreterError::StackOverflow => write!(f, "Stack overflow"),
            InterpreterError::StackUnderflow => write!(f, "Stack underflow"),
            InterpreterError::InvalidInstruction => write!(f, "Invalid instruction"),
            InterpreterError::DivisionByZero => write!(f, "Division by zero"),
            InterpreterError::IndexOutOfBounds => write!(f, "Index out of bounds"),
        }
    }
}

impl std::error::Error for InterpreterError {}

impl Interpreter {
    /// Create new interpreter instance
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(1024),  // Pre-allocate stack
            env_stack: vec![HashMap::new()],  // Start with global environment
            frames: Vec::new(),
            execution_counts: HashMap::new(),
        }
    }

    /// Evaluate an AST expression directly
    /// # Errors
    /// Returns error if evaluation fails (type errors, runtime errors, etc.)
    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, InterpreterError> {
        self.eval_expr_kind(&expr.kind)
    }

    /// Evaluate an expression kind directly (main AST walker)
    /// # Errors
    /// Returns error if evaluation fails
    fn eval_expr_kind(&mut self, expr_kind: &ExprKind) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::Literal(lit) => Ok(self.eval_literal(lit)),
            
            ExprKind::Identifier(name) => self.lookup_variable(name),
            
            ExprKind::Binary { left, op, right } => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                self.eval_binary_op(*op, &left_val, &right_val)
            },
            
            ExprKind::Unary { op, operand } => {
                let operand_val = self.eval_expr(operand)?;
                self.eval_unary_op(*op, &operand_val)
            },
            
            ExprKind::If { condition, then_branch, else_branch } => {
                let cond_val = self.eval_expr(condition)?;
                if cond_val.is_truthy() {
                    self.eval_expr(then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.eval_expr(else_branch)
                } else {
                    Ok(Value::nil())
                }
            },
            
            ExprKind::Let { name, value, body, .. } => {
                let val = self.eval_expr(value)?;
                // Store in current environment
                self.env_set(name.clone(), val);
                let result = self.eval_expr(body)?;
                // Remove binding 
                self.env_remove(name);
                Ok(result)
            },
            
            ExprKind::Function { name, params, body, .. } => {
                let param_names: Vec<String> = params.iter()
                    .map(crate::frontend::ast::Param::name)
                    .collect();
                    
                let closure = Value::Closure {
                    params: param_names,
                    body: Rc::new(body.as_ref().clone()),
                    env: Rc::new(self.current_env().clone()),
                };
                
                // Bind function name in environment for recursion
                self.env_set(name.clone(), closure.clone());
                Ok(closure)
            },
            
            ExprKind::Lambda { params, body } => {
                let param_names: Vec<String> = params.iter()
                    .map(crate::frontend::ast::Param::name)
                    .collect();
                    
                let closure = Value::Closure {
                    params: param_names,
                    body: Rc::new(body.as_ref().clone()),
                    env: Rc::new(self.current_env().clone()),
                };
                
                Ok(closure)
            },
            
            ExprKind::Call { func, args } => {
                let func_val = self.eval_expr(func)?;
                let arg_vals: Result<Vec<Value>, InterpreterError> = args.iter()
                    .map(|arg| self.eval_expr(arg))
                    .collect();
                let arg_vals = arg_vals?;
                
                self.call_function(func_val, &arg_vals)
            },
            
            // Placeholder implementations for other expression types
            _ => Err(InterpreterError::RuntimeError(format!(
                "Expression type not yet implemented: {expr_kind:?}"
            ))),
        }
    }

    /// Evaluate a literal value
    fn eval_literal(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Integer(i) => Value::from_i64(*i),
            Literal::Float(f) => Value::from_f64(*f),
            Literal::String(s) => Value::from_string(s.clone()),
            Literal::Bool(b) => Value::from_bool(*b),
            Literal::Char(c) => Value::from_string(c.to_string()),
            Literal::Unit => Value::nil(),
        }
    }

    /// Look up a variable in the environment (searches from innermost to outermost)
    fn lookup_variable(&self, name: &str) -> Result<Value, InterpreterError> {
        for env in self.env_stack.iter().rev() {
            if let Some(value) = env.get(name) {
                return Ok(value.clone());
            }
        }
        Err(InterpreterError::RuntimeError(format!("Undefined variable: {name}")))
    }
    
    /// Get the current (innermost) environment
    #[allow(clippy::expect_used)]  // Environment stack invariant ensures this never panics
    fn current_env(&self) -> &HashMap<String, Value> {
        self.env_stack.last().expect("Environment stack should never be empty")
    }
    
    /// Set a variable in the current environment
    #[allow(clippy::expect_used)]  // Environment stack invariant ensures this never panics
    fn env_set(&mut self, name: String, value: Value) {
        let env = self.env_stack.last_mut().expect("Environment stack should never be empty");
        env.insert(name, value);
    }
    
    /// Remove a variable from the current environment
    #[allow(clippy::expect_used)]  // Environment stack invariant ensures this never panics
    fn env_remove(&mut self, name: &str) {
        let env = self.env_stack.last_mut().expect("Environment stack should never be empty");
        env.remove(name);
    }
    
    /// Push a new environment onto the stack
    fn env_push(&mut self, env: HashMap<String, Value>) {
        self.env_stack.push(env);
    }
    
    /// Pop the current environment from the stack
    fn env_pop(&mut self) -> Option<HashMap<String, Value>> {
        if self.env_stack.len() > 1 {  // Keep at least the global environment
            self.env_stack.pop()
        } else {
            None
        }
    }
    
    /// Call a function with given arguments
    fn call_function(&mut self, func: Value, args: &[Value]) -> Result<Value, InterpreterError> {
        match func {
            Value::Closure { params, body, env } => {
                // Check argument count
                if args.len() != params.len() {
                    return Err(InterpreterError::RuntimeError(format!(
                        "Function expects {} arguments, got {}",
                        params.len(),
                        args.len()
                    )));
                }
                
                // Create new environment with captured environment as base
                let mut new_env = env.as_ref().clone();
                
                // Bind parameters to arguments
                for (param, arg) in params.iter().zip(args) {
                    new_env.insert(param.clone(), arg.clone());
                }
                
                // Push new environment
                self.env_push(new_env);
                
                // Evaluate function body
                let result = self.eval_expr(&body);
                
                // Pop environment
                self.env_pop();
                
                result
            },
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot call non-function value: {}",
                func.type_name()
            ))),
        }
    }

    /// Evaluate a binary operation from AST
    fn eval_binary_op(&self, op: AstBinaryOp, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
        match op {
            AstBinaryOp::Add => self.add_values(left, right),
            AstBinaryOp::Subtract => self.sub_values(left, right),
            AstBinaryOp::Multiply => self.mul_values(left, right),
            AstBinaryOp::Divide => self.div_values(left, right),
            AstBinaryOp::Equal => Ok(Value::from_bool(self.equal_values(left, right))),
            AstBinaryOp::Less => Ok(Value::from_bool(self.less_than_values(left, right)?)),
            AstBinaryOp::Greater => Ok(Value::from_bool(self.greater_than_values(left, right)?)),
            AstBinaryOp::LessEqual => {
                let less = self.less_than_values(left, right)?;
                let equal = self.equal_values(left, right);
                Ok(Value::from_bool(less || equal))
            },
            AstBinaryOp::GreaterEqual => {
                let greater = self.greater_than_values(left, right)?;
                let equal = self.equal_values(left, right);
                Ok(Value::from_bool(greater || equal))
            },
            AstBinaryOp::NotEqual => Ok(Value::from_bool(!self.equal_values(left, right))),
            AstBinaryOp::And => {
                // Short-circuit evaluation for logical AND
                if left.is_truthy() {
                    Ok(right.clone())
                } else {
                    Ok(left.clone())
                }
            },
            AstBinaryOp::Or => {
                // Short-circuit evaluation for logical OR
                if left.is_truthy() {
                    Ok(left.clone())
                } else {
                    Ok(right.clone())
                }
            },
            _ => Err(InterpreterError::RuntimeError(format!(
                "Binary operator not yet implemented: {op:?}"
            ))),
        }
    }

    /// Evaluate a unary operation
    fn eval_unary_op(&self, op: crate::frontend::ast::UnaryOp, operand: &Value) -> Result<Value, InterpreterError> {
        use crate::frontend::ast::UnaryOp;
        match op {
            UnaryOp::Negate => match operand {
                Value::Integer(i) => Ok(Value::from_i64(-i)),
                Value::Float(f) => Ok(Value::from_f64(-f)),
                _ => Err(InterpreterError::TypeError(format!(
                    "Cannot negate {}", operand.type_name()
                ))),
            },
            UnaryOp::Not => Ok(Value::from_bool(!operand.is_truthy())),
            _ => Err(InterpreterError::RuntimeError(format!(
                "Unary operator not yet implemented: {op:?}"
            ))),
        }
    }

    /// Helper function for testing - evaluate a string expression via parser
    /// # Errors
    /// Returns error if parsing or evaluation fails
    #[cfg(test)]
    pub fn eval_string(&mut self, input: &str) -> Result<Value, Box<dyn std::error::Error>> {
        use crate::frontend::parser::Parser;
        
        let mut parser = Parser::new(input);
        let expr = parser.parse_expr()?;
        
        Ok(self.eval_expr(&expr)?)
    }

    /// Push value onto stack
    /// # Errors
    /// Returns error if stack overflow occurs
    pub fn push(&mut self, value: Value) -> Result<(), InterpreterError> {
        if self.stack.len() >= 10_000 {  // Stack limit from spec
            return Err(InterpreterError::StackOverflow);
        }
        self.stack.push(value);
        Ok(())
    }

    /// Pop value from stack
    /// # Errors
    /// Returns error if stack underflow occurs
    pub fn pop(&mut self) -> Result<Value, InterpreterError> {
        self.stack.pop().ok_or(InterpreterError::StackUnderflow)
    }

    /// Peek at top of stack without popping
    /// # Errors
    /// Returns error if stack underflow occurs
    pub fn peek(&self, depth: usize) -> Result<Value, InterpreterError> {
        let index = self.stack.len().checked_sub(depth + 1)
            .ok_or(InterpreterError::StackUnderflow)?;
        Ok(self.stack[index].clone())
    }

    /// Binary arithmetic operation with type checking
    /// # Errors
    /// Returns error if stack underflow, type mismatch, or arithmetic error occurs
    pub fn binary_op(&mut self, op: BinaryOp) -> Result<(), InterpreterError> {
        let right = self.pop()?;
        let left = self.pop()?;
        
        let result = match op {
            BinaryOp::Add => self.add_values(&left, &right)?,
            BinaryOp::Sub => self.sub_values(&left, &right)?,
            BinaryOp::Mul => self.mul_values(&left, &right)?,
            BinaryOp::Div => self.div_values(&left, &right)?,
            BinaryOp::Eq => Value::from_bool(self.equal_values(&left, &right)),
            BinaryOp::Lt => Value::from_bool(self.less_than_values(&left, &right)?),
            BinaryOp::Gt => Value::from_bool(self.greater_than_values(&left, &right)?),
        };
        
        self.push(result)?;
        Ok(())
    }

    /// Add two values with type coercion
    fn add_values(&self, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::from_i64(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::from_f64(a + b)),
            (Value::Integer(a), Value::Float(b)) => {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64(*a as f64 + b))
            },
            (Value::Float(a), Value::Integer(b)) => {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64(a + *b as f64))
            },
            (Value::String(a), Value::String(b)) => {
                Ok(Value::from_string(format!("{}{}", a.as_ref(), b.as_ref())))
            },
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot add {} and {}", 
                left.type_name(), 
                right.type_name()
            ))),
        }
    }

    /// Subtract two values
    fn sub_values(&self, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::from_i64(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::from_f64(a - b)),
            (Value::Integer(a), Value::Float(b)) => {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64(*a as f64 - b))
            },
            (Value::Float(a), Value::Integer(b)) => {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64(a - *b as f64))
            },
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot subtract {} from {}", 
                right.type_name(),
                left.type_name()
            ))),
        }
    }

    /// Multiply two values
    fn mul_values(&self, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::from_i64(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::from_f64(a * b)),
            (Value::Integer(a), Value::Float(b)) => {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64(*a as f64 * b))
            },
            (Value::Float(a), Value::Integer(b)) => {
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64(a * *b as f64))
            },
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot multiply {} and {}", 
                left.type_name(), 
                right.type_name()
            ))),
        }
    }

    /// Divide two values
    fn div_values(&self, left: &Value, right: &Value) -> Result<Value, InterpreterError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if *b == 0 {
                    return Err(InterpreterError::DivisionByZero);
                }
                Ok(Value::from_i64(a / b))
            },
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err(InterpreterError::DivisionByZero);
                }
                Ok(Value::from_f64(a / b))
            },
            (Value::Integer(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err(InterpreterError::DivisionByZero);
                }
                #[allow(clippy::cast_precision_loss)]
                Ok(Value::from_f64(*a as f64 / b))
            },
            (Value::Float(a), Value::Integer(b)) => {
                #[allow(clippy::cast_precision_loss)]
                let divisor = *b as f64;
                if divisor == 0.0 {
                    return Err(InterpreterError::DivisionByZero);
                }
                Ok(Value::from_f64(a / divisor))
            },
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot divide {} by {}", 
                left.type_name(), 
                right.type_name()
            ))),
        }
    }

    /// Check equality of two values
    fn equal_values(&self, left: &Value, right: &Value) -> bool {
        left == right  // PartialEq is derived for Value
    }

    /// Check if left < right
    fn less_than_values(&self, left: &Value, right: &Value) -> Result<bool, InterpreterError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a < b),
            (Value::Float(a), Value::Float(b)) => Ok(a < b),
            (Value::Integer(a), Value::Float(b)) => {
                #[allow(clippy::cast_precision_loss)]
                Ok((*a as f64) < *b)
            },
            (Value::Float(a), Value::Integer(b)) => {
                #[allow(clippy::cast_precision_loss)]
                Ok(*a < (*b as f64))
            },
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot compare {} and {}", 
                left.type_name(), 
                right.type_name()
            ))),
        }
    }

    /// Check if left > right
    fn greater_than_values(&self, left: &Value, right: &Value) -> Result<bool, InterpreterError> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a > b),
            (Value::Float(a), Value::Float(b)) => Ok(a > b),
            (Value::Integer(a), Value::Float(b)) => {
                #[allow(clippy::cast_precision_loss)]
                Ok((*a as f64) > *b)
            },
            (Value::Float(a), Value::Integer(b)) => {
                #[allow(clippy::cast_precision_loss)]
                Ok(*a > (*b as f64))
            },
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot compare {} and {}", 
                left.type_name(), 
                right.type_name()
            ))),
        }
    }

    /// Print value for debugging
    pub fn print_value(&self, value: &Value) -> std::string::String {
        match value {
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Nil => "nil".to_string(),
            Value::String(s) => s.as_ref().clone(),
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter()
                    .map(|v| self.print_value(v))
                    .collect();
                format!("[{}]", elements.join(", "))
            },
            Value::Closure { params, .. } => {
                format!("function/{}", params.len())
            },
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

/// Binary operations
#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Gt,
}

#[cfg(test)]
#[allow(clippy::expect_used)]        // Tests can use expect for clarity
#[allow(clippy::bool_assert_comparison)]  // Clear test assertions
#[allow(clippy::approx_constant)]    // Test constants are acceptable
#[allow(clippy::panic)]              // Tests can panic on assertion failures
mod tests {
    use super::*;
    use crate::frontend::ast::Span;

    #[test]
    fn test_value_creation() {
        let int_val = Value::from_i64(42);
        assert_eq!(int_val.as_i64().expect("Should be integer"), 42);
        assert_eq!(int_val.type_name(), "integer");

        let bool_val = Value::from_bool(true);
        assert_eq!(bool_val.as_bool().expect("Should be boolean"), true);
        assert_eq!(bool_val.type_name(), "boolean");

        let nil_val = Value::nil();
        assert!(nil_val.is_nil());
        assert_eq!(nil_val.type_name(), "nil");

        let float_val = Value::from_f64(3.14);
        let f_value = float_val.as_f64().expect("Should be float");
        assert!((f_value - 3.14).abs() < f64::EPSILON);
        assert_eq!(float_val.type_name(), "float");

        let string_val = Value::from_string("hello".to_string());
        assert_eq!(string_val.type_name(), "string");
    }

    #[test]
    fn test_arithmetic() {
        let mut interp = Interpreter::new();
        
        // Test 2 + 3 = 5
        assert!(interp.push(Value::from_i64(2)).is_ok());
        assert!(interp.push(Value::from_i64(3)).is_ok());
        assert!(interp.binary_op(BinaryOp::Add).is_ok());
        
        let result = interp.pop().expect("Stack should not be empty");
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_mixed_arithmetic() {
        let mut interp = Interpreter::new();
        
        // Test 2 + 3.5 = 5.5 (int + float -> float)
        assert!(interp.push(Value::from_i64(2)).is_ok());
        assert!(interp.push(Value::from_f64(3.5)).is_ok());
        assert!(interp.binary_op(BinaryOp::Add).is_ok());
        
        let result = interp.pop().expect("Stack should not be empty");
        match result {
            Value::Float(f) => assert!((f - 5.5).abs() < f64::EPSILON),
            _ => unreachable!("Expected float, got {result:?}"),
        }
    }

    #[test]
    fn test_division_by_zero() {
        let mut interp = Interpreter::new();
        
        assert!(interp.push(Value::from_i64(10)).is_ok());
        assert!(interp.push(Value::from_i64(0)).is_ok());
        
        let result = interp.binary_op(BinaryOp::Div);
        assert!(matches!(result, Err(InterpreterError::DivisionByZero)));
    }

    #[test]
    fn test_comparison() {
        let mut interp = Interpreter::new();
        
        // Test 5 < 10
        assert!(interp.push(Value::from_i64(5)).is_ok());
        assert!(interp.push(Value::from_i64(10)).is_ok());
        assert!(interp.binary_op(BinaryOp::Lt).is_ok());
        
        let result = interp.pop().expect("Stack should not be empty");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_stack_operations() {
        let mut interp = Interpreter::new();
        
        let val1 = Value::from_i64(42);
        let val2 = Value::from_bool(true);
        
        assert!(interp.push(val1.clone()).is_ok());
        assert!(interp.push(val2.clone()).is_ok());
        
        assert_eq!(interp.peek(0).expect("Should peek at top"), val2);
        assert_eq!(interp.peek(1).expect("Should peek at second"), val1);
        
        assert_eq!(interp.pop().expect("Should pop top"), val2);
        assert_eq!(interp.pop().expect("Should pop second"), val1);
    }

    #[test]
    fn test_truthiness() {
        assert!(Value::from_i64(42).is_truthy());
        assert!(Value::from_bool(true).is_truthy());
        assert!(!Value::from_bool(false).is_truthy());
        assert!(!Value::nil().is_truthy());
        assert!(Value::from_f64(std::f64::consts::PI).is_truthy());
        assert!(Value::from_f64(0.0).is_truthy()); // 0.0 is truthy in Ruchy
        assert!(Value::from_string("hello".to_string()).is_truthy());
    }

    // AST Walker tests
    
    #[test]
    fn test_eval_literal() {
        let mut interp = Interpreter::new();
        
        // Test integer literal
        let int_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            Span::new(0, 2)
        );
        let result = interp.eval_expr(&int_expr).expect("Should evaluate integer");
        assert_eq!(result, Value::Integer(42));
        
        // Test string literal
        let str_expr = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            Span::new(0, 7)
        );
        let result = interp.eval_expr(&str_expr).expect("Should evaluate string");
        assert_eq!(result.type_name(), "string");
        
        // Test boolean literal
        let bool_expr = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 4)
        );
        let result = interp.eval_expr(&bool_expr).expect("Should evaluate boolean");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_arithmetic() {
        let mut interp = Interpreter::new();
        
        // Test 5 + 3 = 8
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(5)),
            Span::new(0, 1)
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(3)),
            Span::new(4, 5)
        ));
        let add_expr = Expr::new(
            ExprKind::Binary { left, op: AstBinaryOp::Add, right },
            Span::new(0, 5)
        );
        
        let result = interp.eval_expr(&add_expr).expect("Should evaluate addition");
        assert_eq!(result, Value::Integer(8));
    }

    #[test]
    fn test_eval_binary_comparison() {
        let mut interp = Interpreter::new();
        
        // Test 5 < 10 = true
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(5)),
            Span::new(0, 1)
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(10)),
            Span::new(4, 6)
        ));
        let cmp_expr = Expr::new(
            ExprKind::Binary { left, op: AstBinaryOp::Less, right },
            Span::new(0, 6)
        );
        
        let result = interp.eval_expr(&cmp_expr).expect("Should evaluate comparison");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_unary_operations() {
        let mut interp = Interpreter::new();
        
        // Test -42 = -42
        let operand = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            Span::new(1, 3)
        ));
        let neg_expr = Expr::new(
            ExprKind::Unary { 
                op: crate::frontend::ast::UnaryOp::Negate, 
                operand 
            },
            Span::new(0, 3)
        );
        
        let result = interp.eval_expr(&neg_expr).expect("Should evaluate negation");
        assert_eq!(result, Value::Integer(-42));
        
        // Test !true = false
        let operand = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(1, 5)
        ));
        let not_expr = Expr::new(
            ExprKind::Unary { 
                op: crate::frontend::ast::UnaryOp::Not, 
                operand 
            },
            Span::new(0, 5)
        );
        
        let result = interp.eval_expr(&not_expr).expect("Should evaluate logical not");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_if_expression() {
        let mut interp = Interpreter::new();
        
        // Test if true then 1 else 2 = 1
        let condition = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(3, 7)
        ));
        let then_branch = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1)),
            Span::new(13, 14)
        ));
        let else_branch = Some(Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(2)),
            Span::new(20, 21)
        )));
        
        let if_expr = Expr::new(
            ExprKind::If { condition, then_branch, else_branch },
            Span::new(0, 21)
        );
        
        let result = interp.eval_expr(&if_expr).expect("Should evaluate if expression");
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_eval_let_expression() {
        let mut interp = Interpreter::new();
        
        // Test let x = 5 in x + 2 = 7
        let value = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(5)),
            Span::new(8, 9)
        ));
        
        let left = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(13, 14)
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(2)),
            Span::new(17, 18)
        ));
        let body = Box::new(Expr::new(
            ExprKind::Binary { left, op: AstBinaryOp::Add, right },
            Span::new(13, 18)
        ));
        
        let let_expr = Expr::new(
            ExprKind::Let { 
                name: "x".to_string(), 
                value, 
                body, 
                is_mutable: false 
            },
            Span::new(0, 18)
        );
        
        let result = interp.eval_expr(&let_expr).expect("Should evaluate let expression");
        assert_eq!(result, Value::Integer(7));
    }

    #[test]
    fn test_eval_logical_operators() {
        let mut interp = Interpreter::new();
        
        // Test true && false = false (short-circuit)
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 4)
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            Span::new(8, 13)
        ));
        let and_expr = Expr::new(
            ExprKind::Binary { left, op: AstBinaryOp::And, right },
            Span::new(0, 13)
        );
        
        let result = interp.eval_expr(&and_expr).expect("Should evaluate logical AND");
        assert_eq!(result, Value::Bool(false));
        
        // Test false || true = true (short-circuit)
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            Span::new(0, 5)
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(9, 13)
        ));
        let or_expr = Expr::new(
            ExprKind::Binary { left, op: AstBinaryOp::Or, right },
            Span::new(0, 13)
        );
        
        let result = interp.eval_expr(&or_expr).expect("Should evaluate logical OR");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_parser_integration() {
        let mut interp = Interpreter::new();
        
        // Test simple arithmetic: 2 + 3 * 4 = 14
        let result = interp.eval_string("2 + 3 * 4").expect("Should parse and evaluate");
        assert_eq!(result, Value::Integer(14));
        
        // Test comparison: 5 > 3 = true  
        let result = interp.eval_string("5 > 3").expect("Should parse and evaluate");
        assert_eq!(result, Value::Bool(true));
        
        // Test boolean literals: true && false = false
        let result = interp.eval_string("true && false").expect("Should parse and evaluate");
        assert_eq!(result, Value::Bool(false));
        
        // Test unary operations: -42 = -42
        let result = interp.eval_string("-42").expect("Should parse and evaluate");
        assert_eq!(result, Value::Integer(-42));
        
        // Test string literals
        let result = interp.eval_string(r#""hello""#).expect("Should parse and evaluate");
        assert_eq!(result.type_name(), "string");
    }

    #[test]
    fn test_eval_lambda() {
        use crate::frontend::ast::{Pattern, Type, TypeKind};
        let mut interp = Interpreter::new();
        
        // Test lambda: |x| x + 1
        let param = Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type { kind: TypeKind::Named("i32".to_string()), span: Span::new(0, 3) },
            span: Span::new(0, 1),
            is_mutable: false,
        };
        
        let left = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(0, 1)
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1)),
            Span::new(4, 5)
        ));
        let body = Box::new(Expr::new(
            ExprKind::Binary { left, op: AstBinaryOp::Add, right },
            Span::new(0, 5)
        ));
        
        let lambda_expr = Expr::new(
            ExprKind::Lambda { params: vec![param], body },
            Span::new(0, 10)
        );
        
        let result = interp.eval_expr(&lambda_expr).expect("Should evaluate lambda");
        assert_eq!(result.type_name(), "function");
    }

    #[test]
    fn test_eval_function_call() {
        use crate::frontend::ast::{Pattern, Type, TypeKind};
        let mut interp = Interpreter::new();
        
        // Create lambda: |x| x + 1
        let param = Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type { kind: TypeKind::Named("i32".to_string()), span: Span::new(0, 3) },
            span: Span::new(0, 1),
            is_mutable: false,
        };
        
        let left = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(0, 1)
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1)),
            Span::new(4, 5)
        ));
        let body = Box::new(Expr::new(
            ExprKind::Binary { left, op: AstBinaryOp::Add, right },
            Span::new(0, 5)
        ));
        
        let lambda_expr = Expr::new(
            ExprKind::Lambda { params: vec![param], body },
            Span::new(0, 10)
        );
        
        // Call lambda with argument 5: (|x| x + 1)(5) = 6
        let call_expr = Expr::new(
            ExprKind::Call {
                func: Box::new(lambda_expr),
                args: vec![Expr::new(
                    ExprKind::Literal(Literal::Integer(5)),
                    Span::new(0, 1)
                )],
            },
            Span::new(0, 15)
        );
        
        let result = interp.eval_expr(&call_expr).expect("Should evaluate function call");
        assert_eq!(result, Value::Integer(6));
    }

    #[test]
    fn test_eval_function_definition() {
        use crate::frontend::ast::{Pattern, Type, TypeKind};
        let mut interp = Interpreter::new();
        
        // Create function: fn add_one(x) = x + 1
        let param = Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type { kind: TypeKind::Named("i32".to_string()), span: Span::new(0, 3) },
            span: Span::new(0, 1),
            is_mutable: false,
        };
        
        let left = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(0, 1)
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1)),
            Span::new(4, 5)
        ));
        let body = Box::new(Expr::new(
            ExprKind::Binary { left, op: AstBinaryOp::Add, right },
            Span::new(0, 5)
        ));
        
        let func_expr = Expr::new(
            ExprKind::Function {
                name: "add_one".to_string(),
                type_params: vec![],
                params: vec![param],
                return_type: None,
                body,
                is_async: false,
            },
            Span::new(0, 20)
        );
        
        let result = interp.eval_expr(&func_expr).expect("Should evaluate function");
        assert_eq!(result.type_name(), "function");
        
        // Verify function is bound in environment
        let bound_func = interp.lookup_variable("add_one").expect("Function should be bound");
        assert_eq!(bound_func.type_name(), "function");
    }

    #[test]
    fn test_eval_recursive_function() {
        use crate::frontend::ast::{Pattern, Type, TypeKind};
        let mut interp = Interpreter::new();
        
        // Create recursive factorial function
        let param = Param {
            pattern: Pattern::Identifier("n".to_string()),
            ty: Type { kind: TypeKind::Named("i32".to_string()), span: Span::new(0, 3) },
            span: Span::new(0, 1),
            is_mutable: false,
        };
        
        // if n <= 1 then 1 else n * factorial(n - 1)
        let n_id = Expr::new(ExprKind::Identifier("n".to_string()), Span::new(0, 1));
        let one = Expr::new(ExprKind::Literal(Literal::Integer(1)), Span::new(0, 1));
        
        let condition = Box::new(Expr::new(
            ExprKind::Binary {
                left: Box::new(n_id.clone()),
                op: AstBinaryOp::LessEqual,
                right: Box::new(one.clone()),
            },
            Span::new(0, 6)
        ));
        
        let then_branch = Box::new(one.clone());
        
        // n * factorial(n - 1)
        let n_minus_1 = Expr::new(
            ExprKind::Binary {
                left: Box::new(n_id.clone()),
                op: AstBinaryOp::Subtract,
                right: Box::new(one),
            },
            Span::new(0, 5)
        );
        
        let recursive_call = Expr::new(
            ExprKind::Call {
                func: Box::new(Expr::new(
                    ExprKind::Identifier("factorial".to_string()),
                    Span::new(0, 9)
                )),
                args: vec![n_minus_1],
            },
            Span::new(0, 15)
        );
        
        let else_branch = Some(Box::new(Expr::new(
            ExprKind::Binary {
                left: Box::new(n_id),
                op: AstBinaryOp::Multiply,
                right: Box::new(recursive_call),
            },
            Span::new(0, 20)
        )));
        
        let body = Box::new(Expr::new(
            ExprKind::If { condition, then_branch, else_branch },
            Span::new(0, 25)
        ));
        
        let factorial_expr = Expr::new(
            ExprKind::Function {
                name: "factorial".to_string(),
                type_params: vec![],
                params: vec![param],
                return_type: None,
                body,
                is_async: false,
            },
            Span::new(0, 30)
        );
        
        // Define factorial function
        let result = interp.eval_expr(&factorial_expr).expect("Should evaluate factorial function");
        assert_eq!(result.type_name(), "function");
        
        // Test factorial(5) = 120
        let call_expr = Expr::new(
            ExprKind::Call {
                func: Box::new(Expr::new(
                    ExprKind::Identifier("factorial".to_string()),
                    Span::new(0, 9)
                )),
                args: vec![Expr::new(
                    ExprKind::Literal(Literal::Integer(5)),
                    Span::new(0, 1)
                )],
            },
            Span::new(0, 15)
        );
        
        let result = interp.eval_expr(&call_expr).expect("Should evaluate factorial(5)");
        assert_eq!(result, Value::Integer(120));
    }

    #[test]
    fn test_function_closure() {
        use crate::frontend::ast::{Pattern, Type, TypeKind};
        let mut interp = Interpreter::new();
        
        // Test closure: let x = 10 in |y| x + y
        let x_val = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(10)),
            Span::new(8, 10)
        ));
        
        let param = Param {
            pattern: Pattern::Identifier("y".to_string()),
            ty: Type { kind: TypeKind::Named("i32".to_string()), span: Span::new(0, 3) },
            span: Span::new(0, 1),
            is_mutable: false,
        };
        
        let left = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(0, 1)
        ));
        let right = Box::new(Expr::new(
            ExprKind::Identifier("y".to_string()),
            Span::new(4, 5)
        ));
        let lambda_body = Box::new(Expr::new(
            ExprKind::Binary { left, op: AstBinaryOp::Add, right },
            Span::new(0, 5)
        ));
        
        let lambda = Expr::new(
            ExprKind::Lambda { params: vec![param], body: lambda_body },
            Span::new(14, 24)
        );
        
        let let_body = Box::new(lambda);
        
        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                value: x_val,
                body: let_body,
                is_mutable: false,
            },
            Span::new(0, 24)
        );
        
        let closure = interp.eval_expr(&let_expr).expect("Should evaluate closure");
        assert_eq!(closure.type_name(), "function");
        
        // Call closure with argument 5: (|y| x + y)(5) = 15 (x = 10)
        let call_expr = Expr::new(
            ExprKind::Call {
                func: Box::new(let_expr), // Re-create the closure
                args: vec![Expr::new(
                    ExprKind::Literal(Literal::Integer(5)),
                    Span::new(0, 1)
                )],
            },
            Span::new(0, 30)
        );
        
        // Note: This test demonstrates lexical scoping where the closure captures 'x'
        let result = interp.eval_expr(&call_expr).expect("Should evaluate closure call");
        assert_eq!(result, Value::Integer(15));
    }
}