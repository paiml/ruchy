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
        arity: u8,
        code: Rc<Vec<u8>>, // Placeholder for bytecode/AST
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
    
    /// Global variable bindings (will be used in Phase 1)
    #[allow(dead_code)]
    globals: HashMap<std::string::String, Value>,
    
    /// Call frame for function calls (will be used in Phase 1)
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

impl Interpreter {
    /// Create new interpreter instance
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(1024),  // Pre-allocate stack
            globals: HashMap::new(),
            frames: Vec::new(),
            execution_counts: HashMap::new(),
        }
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
            Value::Closure { arity, .. } => {
                format!("function/{arity}")
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
}