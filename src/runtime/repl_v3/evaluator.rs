//! Resource-bounded evaluator for REPL v3
//!
//! Implements hard limits on memory, time, and stack depth to prevent
//! resource exhaustion and ensure predictable behavior.

use anyhow::{bail, Context, Result};
use std::time::{Duration, Instant};
use std::fmt;
use crate::frontend::parser::Parser;
use crate::frontend::ast::Expr;

/// Runtime value for evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum EvalValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Char(char),
    Unit,
}

impl fmt::Display for EvalValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalValue::Int(n) => write!(f, "{n}"),
            EvalValue::Float(x) => write!(f, "{x}"),
            EvalValue::String(s) => write!(f, "\"{s}\""),
            EvalValue::Bool(b) => write!(f, "{b}"),
            EvalValue::Char(c) => write!(f, "'{c}'"),
            EvalValue::Unit => write!(f, "()"),
        }
    }
}

/// Simple memory tracker for bounded allocation
pub struct MemoryTracker {
    max_size: usize,
    current: usize,
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            current: 0,
        }
    }
    
    /// Try to allocate memory
    ///
    /// # Errors
    ///
    /// Returns an error if allocation would exceed memory limit
    ///
    /// # Example
    ///
    /// ```
    /// use ruchy::runtime::repl_v3::evaluator::MemoryTracker;
    ///
    /// let mut tracker = MemoryTracker::new(100);
    /// assert!(tracker.try_alloc(50).is_ok());
    /// assert!(tracker.try_alloc(60).is_err()); // Would exceed limit
    /// ```
    pub fn try_alloc(&mut self, size: usize) -> Result<()> {
        if self.current + size > self.max_size {
            bail!("Memory limit exceeded: {} + {} > {}", 
                  self.current, size, self.max_size);
        }
        self.current += size;
        Ok(())
    }
    
    /// Free memory
    pub fn free(&mut self, size: usize) {
        self.current = self.current.saturating_sub(size);
    }
    
    /// Reset the tracker
    pub fn reset(&mut self) {
        self.current = 0;
    }
    
    /// Get current memory usage
    pub fn used(&self) -> usize {
        self.current
    }
}

/// Bounded evaluator with resource limits
pub struct BoundedEvaluator {
    memory: MemoryTracker,
    timeout: Duration,
    max_depth: usize,
}

impl BoundedEvaluator {
    /// Create a new bounded evaluator
    ///
    /// # Errors
    ///
    /// Returns an error if the memory tracker cannot be created
    ///
    /// # Example
    ///
    /// ```
    /// use ruchy::runtime::repl_v3::evaluator::BoundedEvaluator;
    /// use std::time::Duration;
    ///
    /// let evaluator = BoundedEvaluator::new(
    ///     1024 * 1024,  // 1MB memory
    ///     Duration::from_millis(100),
    ///     1000  // max stack depth
    /// );
    /// assert!(evaluator.is_ok());
    /// ```
    pub fn new(max_memory: usize, timeout: Duration, max_depth: usize) -> Result<Self> {
        let memory = MemoryTracker::new(max_memory);
        
        Ok(Self {
            memory,
            timeout,
            max_depth,
        })
    }
    
    /// Evaluate an expression with resource bounds
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Memory limit is exceeded
    /// - Timeout is reached
    /// - Stack depth limit is exceeded
    ///
    /// # Example
    ///
    /// ```
    /// use ruchy::runtime::repl_v3::evaluator::BoundedEvaluator;
    /// use std::time::Duration;
    ///
    /// let mut evaluator = BoundedEvaluator::new(
    ///     1024,  // Small memory limit
    ///     Duration::from_secs(1),
    ///     10
    /// ).unwrap();
    ///
    /// let result = evaluator.eval("42");
    /// assert!(result.is_ok());
    /// ```
    pub fn eval(&mut self, input: &str) -> Result<String> {
        // Reset memory tracker for fresh evaluation
        self.memory.reset();
        
        // Track input memory
        self.memory.try_alloc(input.len())?;
        
        // Set evaluation deadline
        let deadline = Instant::now() + self.timeout;
        
        // Execute with bounds checking
        self.eval_bounded(input, deadline, 0)
    }
    
    fn eval_bounded(&mut self, input: &str, deadline: Instant, depth: usize) -> Result<String> {
        // Check timeout
        if Instant::now() > deadline {
            bail!("Evaluation timeout exceeded");
        }
        
        // Check stack depth
        if depth > self.max_depth {
            bail!("Maximum recursion depth {} exceeded", self.max_depth);
        }
        
        // Parse the input with memory tracking
        let mut parser = Parser::new(input);
        let ast = parser.parse()
            .context("Failed to parse input")?;
        
        // Check memory for AST
        self.memory.try_alloc(std::mem::size_of_val(&ast))?;
        
        // Evaluate the expression
        let value = self.evaluate_expr(&ast, deadline, depth + 1)?;
        
        // Track result memory
        let result = value.to_string();
        self.memory.try_alloc(result.len())?;
        Ok(result)
    }
    
    /// Evaluate an expression to a value
    fn evaluate_expr(&mut self, expr: &Expr, deadline: Instant, depth: usize) -> Result<EvalValue> {
        use crate::frontend::ast::{ExprKind, Literal};
        
        // Check resource bounds
        if Instant::now() > deadline {
            bail!("Evaluation timeout exceeded");
        }
        if depth > self.max_depth {
            bail!("Maximum recursion depth {} exceeded", self.max_depth);
        }
        
        match &expr.kind {
            ExprKind::Literal(lit) => match lit {
                Literal::Integer(n) => Ok(EvalValue::Int(*n)),
                Literal::Float(f) => Ok(EvalValue::Float(*f)),
                Literal::String(s) => {
                    self.memory.try_alloc(s.len())?;
                    Ok(EvalValue::String(s.clone()))
                }
                Literal::Bool(b) => Ok(EvalValue::Bool(*b)),
                Literal::Unit => Ok(EvalValue::Unit),
            },
            ExprKind::Binary { left, op, right } => {
                let lhs = self.evaluate_expr(left, deadline, depth + 1)?;
                let rhs = self.evaluate_expr(right, deadline, depth + 1)?;
                Self::evaluate_binary(&lhs, *op, &rhs)
            },
            ExprKind::Unary { op, operand } => {
                let val = self.evaluate_expr(operand, deadline, depth + 1)?;
                Self::evaluate_unary(*op, &val)
            },
            _ => bail!("Expression type not yet implemented: {:?}", expr.kind),
        }
    }
    
    /// Evaluate binary operations
    fn evaluate_binary(lhs: &EvalValue, op: crate::frontend::ast::BinaryOp, rhs: &EvalValue) -> Result<EvalValue> {
        use crate::frontend::ast::BinaryOp;
        use EvalValue::{Int, Float, Bool};
        
        match (lhs, op, rhs) {
            // Integer arithmetic
            (Int(a), BinaryOp::Add, Int(b)) => Ok(Int(a + b)),
            (Int(a), BinaryOp::Subtract, Int(b)) => Ok(Int(a - b)),
            (Int(a), BinaryOp::Multiply, Int(b)) => Ok(Int(a * b)),
            (Int(a), BinaryOp::Divide, Int(b)) => {
                if *b == 0 {
                    bail!("Division by zero");
                }
                Ok(Int(a / b))
            },
            (Int(a), BinaryOp::Modulo, Int(b)) => {
                if *b == 0 {
                    bail!("Modulo by zero");
                }
                Ok(Int(a % b))
            },
            
            // Float arithmetic
            (Float(a), BinaryOp::Add, Float(b)) => Ok(Float(a + b)),
            (Float(a), BinaryOp::Subtract, Float(b)) => Ok(Float(a - b)),
            (Float(a), BinaryOp::Multiply, Float(b)) => Ok(Float(a * b)),
            (Float(a), BinaryOp::Divide, Float(b)) => {
                if *b == 0.0 {
                    bail!("Division by zero");
                }
                Ok(Float(a / b))
            },
            
            // Comparisons
            (Int(a), BinaryOp::Less, Int(b)) => Ok(Bool(a < b)),
            (Int(a), BinaryOp::LessEqual, Int(b)) => Ok(Bool(a <= b)),
            (Int(a), BinaryOp::Greater, Int(b)) => Ok(Bool(a > b)),
            (Int(a), BinaryOp::GreaterEqual, Int(b)) => Ok(Bool(a >= b)),
            (Int(a), BinaryOp::Equal, Int(b)) => Ok(Bool(a == b)),
            (Int(a), BinaryOp::NotEqual, Int(b)) => Ok(Bool(a != b)),
            
            // Boolean logic
            (Bool(a), BinaryOp::And, Bool(b)) => Ok(Bool(*a && *b)),
            (Bool(a), BinaryOp::Or, Bool(b)) => Ok(Bool(*a || *b)),
            
            _ => bail!("Type mismatch in binary operation: {:?} {:?} {:?}", lhs, op, rhs),
        }
    }
    
    /// Evaluate unary operations
    fn evaluate_unary(op: crate::frontend::ast::UnaryOp, val: &EvalValue) -> Result<EvalValue> {
        use crate::frontend::ast::UnaryOp;
        use EvalValue::{Int, Float, Bool};
        
        match (op, val) {
            (UnaryOp::Negate, Int(n)) => Ok(Int(-n)),
            (UnaryOp::Negate, Float(f)) => Ok(Float(-f)),
            (UnaryOp::Not, Bool(b)) => Ok(Bool(!b)),
            _ => bail!("Type mismatch in unary operation: {:?} {:?}", op, val),
        }
    }
    /// Get current memory usage
    pub fn memory_used(&self) -> usize {
        self.memory.used()
    }
}