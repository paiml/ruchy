//! REPL implementation for interactive Ruchy development
//!
//! Production-grade REPL with resource bounds, error recovery, and grammar coverage
//!
//! # Examples
//!
//! ```
//! use ruchy::runtime::Repl;
//!
//! let mut repl = Repl::new().unwrap();
//!
//! // Evaluate arithmetic
//! let result = repl.eval("2 + 2").unwrap();
//! assert_eq!(result, "4");
//!
//! // Define variables
//! repl.eval("let x = 10").unwrap();
//! let result = repl.eval("x * 2").unwrap();
//! assert_eq!(result, "20");
//! ```
//!
//! # One-liner evaluation
//!
//! ```
//! use ruchy::runtime::Repl;
//! use std::time::{Duration, Instant};
//!
//! let mut repl = Repl::new().unwrap();
//! let deadline = Some(Instant::now() + Duration::from_millis(100));
//!
//! let value = repl.evaluate_expr_str("5 + 3", deadline).unwrap();
//! assert_eq!(value.to_string(), "8");
//! ```

#![allow(clippy::print_stdout)] // REPL needs to print to stdout
#![allow(clippy::print_stderr)] // REPL needs to print errors
#![allow(clippy::expect_used)] // REPL can panic on initialization failure

use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, MatchArm, Pattern, PipelineStage, Span, UnaryOp};
use crate::{Parser, Transpiler};
use anyhow::{bail, Context, Result};
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

/// Runtime value for evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Char(char),
    List(Vec<Value>),
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Expr>,
    },
    Unit,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(x) => write!(f, "{x}"),
            Value::String(s) => write!(f, "\"{s}\""),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Char(c) => write!(f, "'{c}'"),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Value::Function { name, params, .. } => {
                write!(f, "fn {}({})", name, params.join(", "))
            }
            Value::Unit => write!(f, "()"),
        }
    }
}

/// REPL configuration
pub struct ReplConfig {
    /// Maximum memory for evaluation (default: 10MB)
    pub max_memory: usize,
    /// Timeout for evaluation (default: 100ms)
    pub timeout: Duration,
    /// Maximum stack depth (default: 1000)
    pub max_depth: usize,
    /// Enable debug mode
    pub debug: bool,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            max_memory: 10 * 1024 * 1024, // 10MB
            timeout: Duration::from_millis(100),
            max_depth: 1000,
            debug: false,
        }
    }
}

/// Memory tracker for bounded allocation
struct MemoryTracker {
    max_size: usize,
    current: usize,
}

impl MemoryTracker {
    fn new(max_size: usize) -> Self {
        Self {
            max_size,
            current: 0,
        }
    }

    fn try_alloc(&mut self, size: usize) -> Result<()> {
        if self.current + size > self.max_size {
            bail!(
                "Memory limit exceeded: {} + {} > {}",
                self.current,
                size,
                self.max_size
            );
        }
        self.current += size;
        Ok(())
    }

    fn reset(&mut self) {
        self.current = 0;
    }
}

/// REPL state management with resource bounds
pub struct Repl {
    /// History of successfully parsed expressions
    history: Vec<String>,
    /// Accumulated definitions for the session
    definitions: Vec<String>,
    /// Bindings and their types/values
    bindings: HashMap<String, Value>,
    /// Transpiler instance
    transpiler: Transpiler,
    /// Temporary directory for compilation
    temp_dir: PathBuf,
    /// Session counter for unique naming
    session_counter: usize,
    /// Configuration
    config: ReplConfig,
    /// Memory tracker
    memory: MemoryTracker,
}

impl Repl {
    /// Create a new REPL instance with default config
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary directory cannot be created
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::Repl;
    ///
    /// let repl = Repl::new();
    /// assert!(repl.is_ok());
    /// ```
    pub fn new() -> Result<Self> {
        Self::with_config(ReplConfig::default())
    }

    /// Create a new REPL instance with custom config
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary directory cannot be created
    pub fn with_config(config: ReplConfig) -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("ruchy_repl");
        fs::create_dir_all(&temp_dir)?;

        let memory = MemoryTracker::new(config.max_memory);

        Ok(Self {
            history: Vec::new(),
            definitions: Vec::new(),
            bindings: HashMap::new(),
            transpiler: Transpiler::new(),
            temp_dir,
            session_counter: 0,
            config,
            memory,
        })
    }

    /// Evaluate an expression string and return the Value
    ///
    /// This is used for one-liner evaluation from CLI
    ///
    /// # Errors
    ///
    /// Returns an error if parsing or evaluation fails
    pub fn evaluate_expr_str(&mut self, input: &str, deadline: Option<Instant>) -> Result<Value> {
        // Reset memory tracker for fresh evaluation
        self.memory.reset();

        // Track input memory
        self.memory.try_alloc(input.len())?;

        // Use provided deadline or default timeout
        let deadline = deadline.unwrap_or_else(|| Instant::now() + self.config.timeout);

        // Parse the input
        let mut parser = Parser::new(input);
        let ast = parser.parse().context("Failed to parse input")?;

        // Check memory for AST
        self.memory.try_alloc(std::mem::size_of_val(&ast))?;

        // Evaluate the expression
        let value = self.evaluate_expr(&ast, deadline, 0)?;

        // Handle let bindings specially
        if let ExprKind::Let { name, .. } = &ast.kind {
            self.bindings.insert(name.clone(), value.clone());
        }

        Ok(value)
    }

    /// Evaluate an expression with resource bounds
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Memory limit is exceeded
    /// - Timeout is reached
    /// - Stack depth limit is exceeded
    /// - Parse or evaluation fails
    pub fn eval(&mut self, input: &str) -> Result<String> {
        // Reset memory tracker for fresh evaluation
        self.memory.reset();

        // Track input memory
        self.memory.try_alloc(input.len())?;

        // Set evaluation deadline
        let deadline = Instant::now() + self.config.timeout;

        // Parse the input
        let mut parser = Parser::new(input);
        let ast = parser.parse().context("Failed to parse input")?;

        // Check memory for AST
        self.memory.try_alloc(std::mem::size_of_val(&ast))?;

        // Evaluate the expression
        let value = self.evaluate_expr(&ast, deadline, 0)?;

        // Store successful evaluation
        self.history.push(input.to_string());

        // Let bindings are handled in evaluate_expr, no need to duplicate here

        Ok(value.to_string())
    }

    /// Evaluate an expression to a value
    #[allow(clippy::too_many_lines)]
    fn evaluate_expr(&mut self, expr: &Expr, deadline: Instant, depth: usize) -> Result<Value> {
        // Check resource bounds
        if Instant::now() > deadline {
            bail!("Evaluation timeout exceeded");
        }
        if depth > self.config.max_depth {
            bail!("Maximum recursion depth {} exceeded", self.config.max_depth);
        }

        match &expr.kind {
            ExprKind::Literal(lit) => self.evaluate_literal(lit),
            ExprKind::Binary { left, op, right } => {
                let lhs = self.evaluate_expr(left, deadline, depth + 1)?;
                let rhs = self.evaluate_expr(right, deadline, depth + 1)?;
                Self::evaluate_binary(&lhs, *op, &rhs)
            }
            ExprKind::Unary { op, operand } => {
                let val = self.evaluate_expr(operand, deadline, depth + 1)?;
                Self::evaluate_unary(*op, &val)
            }
            ExprKind::Identifier(name) => self
                .bindings
                .get(name)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Undefined variable: {}", name)),
            ExprKind::Let { name, value, body, .. } => {
                let val = self.evaluate_expr(value, deadline, depth + 1)?;
                self.bindings.insert(name.clone(), val);
                // Evaluate the body with the new binding in scope
                self.evaluate_expr(body, deadline, depth + 1)
            }
            ExprKind::If { condition, then_branch, else_branch } => {
                self.evaluate_if(condition, then_branch, else_branch.as_deref(), deadline, depth)
            }
            ExprKind::Block(exprs) => {
                if exprs.is_empty() {
                    return Ok(Value::Unit);
                }

                let mut result = Value::Unit;
                for expr in exprs {
                    result = self.evaluate_expr(expr, deadline, depth + 1)?;
                }
                Ok(result)
            }
            ExprKind::List(elements) => {
                let mut results = Vec::new();
                for elem in elements {
                    let val = self.evaluate_expr(elem, deadline, depth + 1)?;
                    results.push(val);
                }
                Ok(Value::List(results))
            }
            ExprKind::Assign { target, value } => {
                let val = self.evaluate_expr(value, deadline, depth + 1)?;

                // For now, only support simple variable assignment
                if let ExprKind::Identifier(name) = &target.kind {
                    self.bindings.insert(name.clone(), val.clone());
                    Ok(val)
                } else {
                    bail!(
                        "Only simple variable assignment is supported, got: {:?}",
                        target.kind
                    );
                }
            }
            ExprKind::Range {
                start,
                end,
                inclusive: _,
            } => {
                let start_val = self.evaluate_expr(start, deadline, depth + 1)?;
                let end_val = self.evaluate_expr(end, deadline, depth + 1)?;

                // For REPL demo, just return a string representation
                match (start_val, end_val) {
                    (Value::Int(s), Value::Int(e)) => Ok(Value::String(format!("{s}..{e}"))),
                    _ => bail!("Range endpoints must be integers"),
                }
            }
            ExprKind::Function { name, params, body, .. } => {
                // Store function definition
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let func_value = Value::Function {
                    name: name.clone(),
                    params: param_names,
                    body: body.clone(),
                };

                // Store the function in bindings
                self.bindings.insert(name.clone(), func_value.clone());
                Ok(func_value)
            }
            ExprKind::Lambda { params, .. } => {
                // Lambda expressions (simplified for REPL demo)
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let lambda_signature = format!("|{}| <body>", param_names.join(", "));
                Ok(Value::String(lambda_signature))
            }
            ExprKind::Call { func, args } => self.evaluate_call(func, args, deadline, depth),
            ExprKind::DataFrame { columns } => {
                // For REPL demo, return a formatted representation
                let mut repr = String::from("DataFrame {\n");
                for col in columns {
                    write!(repr, "  {}: [", col.name).expect("String write cannot fail");
                    for (i, val_expr) in col.values.iter().enumerate() {
                        if i > 0 {
                            repr.push_str(", ");
                        }
                        let val = self.evaluate_expr(val_expr, deadline, depth + 1)?;
                        match val {
                            Value::String(s) => write!(repr, "\"{s}\"").expect("String write cannot fail"),
                            other => write!(repr, "{other}").expect("String write cannot fail"),
                        }
                    }
                    repr.push_str("]\n");
                }
                repr.push('}');
                Ok(Value::String(repr))
            }
            ExprKind::DataFrameOperation { .. } => {
                // DataFrame operations not yet implemented in REPL
                bail!("DataFrame operations not yet implemented in REPL")
            }
            ExprKind::Match { expr: match_expr, arms } => {
                self.evaluate_match(match_expr, arms, deadline, depth)
            }
            ExprKind::For { var, iter, body } => {
                // Evaluate the iterable
                let iterable = self.evaluate_expr(iter, deadline, depth + 1)?;
                
                // Save current bindings
                let saved_bindings = self.bindings.clone();
                
                // For now, only handle lists and ranges
                match iterable {
                    Value::List(items) => {
                        let mut result = Value::Unit;
                        for item in items {
                            // Bind the loop variable
                            self.bindings.insert(var.clone(), item);
                            // Execute the body
                            result = self.evaluate_expr(body, deadline, depth + 1)?;
                        }
                        // Restore bindings
                        self.bindings = saved_bindings;
                        Ok(result)
                    }
                    _ => bail!("For loops currently only support lists, got: {:?}", iterable),
                }
            }
            ExprKind::While { condition, body } => {
                let mut result = Value::Unit;
                let max_iterations = 1000; // Prevent infinite loops in REPL
                let mut iterations = 0;
                
                loop {
                    if iterations >= max_iterations {
                        bail!("While loop exceeded maximum iterations ({})", max_iterations);
                    }
                    
                    // Evaluate condition
                    let cond_val = self.evaluate_expr(condition, deadline, depth + 1)?;
                    match cond_val {
                        Value::Bool(true) => {
                            result = self.evaluate_expr(body, deadline, depth + 1)?;
                            iterations += 1;
                        }
                        Value::Bool(false) => break,
                        _ => bail!("While condition must be boolean, got: {:?}", cond_val),
                    }
                }
                
                Ok(result)
            }
            ExprKind::Pipeline { expr, stages } => {
                self.evaluate_pipeline(expr, stages, deadline, depth)
            }
            ExprKind::StringInterpolation { parts } => {
                use crate::frontend::ast::StringPart;
                
                let mut result = String::new();
                for part in parts {
                    match part {
                        StringPart::Text(text) => result.push_str(text),
                        StringPart::Expr(expr) => {
                            let value = self.evaluate_expr(expr, deadline, depth + 1)?;
                            // Format the value for interpolation (without quotes for strings)
                            match value {
                                Value::String(s) => result.push_str(&s),
                                Value::Char(c) => result.push(c),
                                other => result.push_str(&other.to_string()),
                            }
                        }
                    }
                }
                Ok(Value::String(result))
            }
            ExprKind::TryCatch { try_block, catch_clauses, finally_block } => {
                // Execute try block
                let try_result = self.evaluate_expr(try_block, deadline, depth + 1);
                
                match try_result {
                    Ok(value) => {
                        // Try succeeded, execute finally block if present
                        if let Some(finally) = finally_block {
                            let _ = self.evaluate_expr(finally, deadline, depth + 1);
                        }
                        Ok(value)
                    }
                    Err(error) => {
                        // Try failed, check catch clauses
                        if let Some(_catch_clause) = catch_clauses.iter().next() {
                            // For now, just catch any error and return unit
                            // Pattern matching on error types requires error type system
                            if let Some(finally) = finally_block {
                                let _ = self.evaluate_expr(finally, deadline, depth + 1);
                            }
                            return Ok(Value::Unit);
                        }
                        
                        // No catch clause matched, execute finally and re-throw
                        if let Some(finally) = finally_block {
                            let _ = self.evaluate_expr(finally, deadline, depth + 1);
                        }
                        Err(error)
                    }
                }
            }
            ExprKind::Try { expr } => {
                // The ? operator - for now just evaluate the expression
                // Error propagation requires Result type system integration
                self.evaluate_expr(expr, deadline, depth + 1)
            }
            ExprKind::Ok { value } => {
                // Evaluate the value and wrap in Result::Ok
                let val = self.evaluate_expr(value, deadline, depth + 1)?;
                // For now, represent as a tuple ("Ok", value)
                Ok(Value::List(vec![Value::String("Ok".to_string()), val]))
            }
            ExprKind::Err { error } => {
                // Evaluate the error and wrap in Result::Err
                let err = self.evaluate_expr(error, deadline, depth + 1)?;
                // For now, represent as a tuple ("Err", error)
                Ok(Value::List(vec![Value::String("Err".to_string()), err]))
            }
            _ => bail!("Expression type not yet implemented: {:?}", expr.kind),
        }
    }

    /// Check if a pattern matches a value and return bindings
    /// 
    /// Returns Some(bindings) if pattern matches, None if it doesn't
    fn pattern_matches(value: &Value, pattern: &Pattern) -> Result<Option<HashMap<String, Value>>> {
        let mut bindings = HashMap::new();
        
        if Self::pattern_matches_recursive(value, pattern, &mut bindings)? {
            Ok(Some(bindings))
        } else {
            Ok(None)
        }
    }
    
    /// Recursive pattern matching helper
    fn pattern_matches_recursive(
        value: &Value,
        pattern: &Pattern,
        bindings: &mut HashMap<String, Value>
    ) -> Result<bool> {
        match (value, pattern) {
            // Wildcard matches everything and literal Unit
            (_, Pattern::Wildcard) | (Value::Unit, Pattern::Literal(Literal::Unit)) => Ok(true),
            
            // Literal patterns
            (Value::Int(v), Pattern::Literal(Literal::Integer(p))) => Ok(v == p),
            (Value::Float(v), Pattern::Literal(Literal::Float(p))) => Ok((v - p).abs() < f64::EPSILON),
            (Value::String(v), Pattern::Literal(Literal::String(p))) => Ok(v == p),
            (Value::Bool(v), Pattern::Literal(Literal::Bool(p))) => Ok(v == p),
            
            // Identifier patterns (bind to variable)
            (value, Pattern::Identifier(name)) => {
                bindings.insert(name.clone(), value.clone());
                Ok(true)
            }
            
            // List patterns
            (Value::List(values), Pattern::List(patterns)) => {
                if values.len() != patterns.len() {
                    return Ok(false);
                }
                
                for (value, pattern) in values.iter().zip(patterns.iter()) {
                    if !Self::pattern_matches_recursive(value, pattern, bindings)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            
            // Tuple patterns (treat as list for now)
            (Value::List(values), Pattern::Tuple(patterns)) => {
                if values.len() != patterns.len() {
                    return Ok(false);
                }
                
                for (value, pattern) in values.iter().zip(patterns.iter()) {
                    if !Self::pattern_matches_recursive(value, pattern, bindings)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            
            // OR patterns - try each alternative
            (value, Pattern::Or(patterns)) => {
                for pattern in patterns {
                    let mut temp_bindings = HashMap::new();
                    if Self::pattern_matches_recursive(value, pattern, &mut temp_bindings)? {
                        // Merge bindings
                        for (name, val) in temp_bindings {
                            bindings.insert(name, val);
                        }
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            
            // Range patterns (simplified implementation)
            (Value::Int(v), Pattern::Range { start, end, inclusive }) => {
                // For simplicity, only handle integer literal patterns in ranges
                if let (Pattern::Literal(Literal::Integer(start_val)), Pattern::Literal(Literal::Integer(end_val))) = (start.as_ref(), end.as_ref()) {
                    if *inclusive {
                        Ok(*start_val <= *v && *v <= *end_val)
                    } else {
                        Ok(*start_val <= *v && *v < *end_val)
                    }
                } else {
                    bail!("Complex range patterns not yet supported");
                }
            }
            
            // Struct patterns not yet implemented
            (_, Pattern::Struct { .. }) => {
                bail!("Struct patterns not yet implemented in REPL");
            }
            
            // Type mismatches
            _ => Ok(false),
        }
    }

    /// Evaluate binary operations
    fn evaluate_binary(lhs: &Value, op: BinaryOp, rhs: &Value) -> Result<Value> {
        use Value::{Bool, Float, Int};

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
            }
            (Int(a), BinaryOp::Modulo, Int(b)) => {
                if *b == 0 {
                    bail!("Modulo by zero");
                }
                Ok(Int(a % b))
            }
            (Int(a), BinaryOp::Power, Int(b)) => {
                if *b < 0 {
                    bail!("Negative integer powers not supported in integer context");
                }
                let exp =
                    u32::try_from(*b).map_err(|_| anyhow::anyhow!("Power exponent too large"))?;
                let result = a.pow(exp);
                Ok(Int(result))
            }

            // Float arithmetic
            (Float(a), BinaryOp::Add, Float(b)) => Ok(Float(a + b)),
            (Float(a), BinaryOp::Subtract, Float(b)) => Ok(Float(a - b)),
            (Float(a), BinaryOp::Multiply, Float(b)) => Ok(Float(a * b)),
            (Float(a), BinaryOp::Divide, Float(b)) => {
                if *b == 0.0 {
                    bail!("Division by zero");
                }
                Ok(Float(a / b))
            }
            (Float(a), BinaryOp::Power, Float(b)) => Ok(Float(a.powf(*b))),

            // String operations
            (Value::String(a), BinaryOp::Add, Value::String(b)) => {
                Ok(Value::String(format!("{a}{b}")))
            }

            // Comparisons - Integers
            (Int(a), BinaryOp::Less, Int(b)) => Ok(Bool(a < b)),
            (Int(a), BinaryOp::LessEqual, Int(b)) => Ok(Bool(a <= b)),
            (Int(a), BinaryOp::Greater, Int(b)) => Ok(Bool(a > b)),
            (Int(a), BinaryOp::GreaterEqual, Int(b)) => Ok(Bool(a >= b)),
            (Int(a), BinaryOp::Equal, Int(b)) => Ok(Bool(a == b)),
            (Int(a), BinaryOp::NotEqual, Int(b)) => Ok(Bool(a != b)),

            // Comparisons - Strings
            (Value::String(a), BinaryOp::Equal, Value::String(b)) => Ok(Bool(a == b)),
            (Value::String(a), BinaryOp::NotEqual, Value::String(b)) => Ok(Bool(a != b)),

            // Comparisons - Booleans
            (Bool(a), BinaryOp::Equal, Bool(b)) => Ok(Bool(a == b)),
            (Bool(a), BinaryOp::NotEqual, Bool(b)) => Ok(Bool(a != b)),

            // Boolean logic
            (Bool(a), BinaryOp::And, Bool(b)) => Ok(Bool(*a && *b)),
            (Bool(a), BinaryOp::Or, Bool(b)) => Ok(Bool(*a || *b)),

            _ => bail!(
                "Type mismatch in binary operation: {:?} {:?} {:?}",
                lhs,
                op,
                rhs
            ),
        }
    }

    /// Evaluate unary operations
    fn evaluate_unary(op: UnaryOp, val: &Value) -> Result<Value> {
        use Value::{Bool, Float, Int};

        match (op, val) {
            (UnaryOp::Negate, Int(n)) => Ok(Int(-n)),
            (UnaryOp::Negate, Float(f)) => Ok(Float(-f)),
            (UnaryOp::Not, Bool(b)) => Ok(Bool(!b)),
            _ => bail!("Type mismatch in unary operation: {:?} {:?}", op, val),
        }
    }

    /// Run the interactive REPL
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Readline initialization fails
    /// - User input cannot be read
    /// - Commands fail to execute
    pub fn run(&mut self) -> Result<()> {
        println!("{}", "Welcome to Ruchy REPL v0.4.0".bright_cyan().bold());
        println!(
            "{}",
            "Type :help for commands, :quit to exit".bright_black()
        );
        println!();

        let mut rl = DefaultEditor::new()?;

        // Load history if it exists
        let history_path = self.temp_dir.join("history.txt");
        let _ = rl.load_history(&history_path);

        let mut multiline_buffer = String::new();
        let mut in_multiline = false;

        loop {
            let prompt = if in_multiline {
                format!("{} ", "   ...".bright_black())
            } else {
                format!("{} ", "ruchy>".bright_green())
            };
            let readline = rl.readline(&prompt);

            match readline {
                Ok(line) => {
                    // Skip empty lines unless we're in multiline mode
                    if line.trim().is_empty() && !in_multiline {
                        continue;
                    }

                    // Handle commands (only when not in multiline mode)
                    if !in_multiline && line.starts_with(':') {
                        if self.handle_command(&line)? {
                            break; // :quit command
                        }
                        continue;
                    }

                    // Check if this starts a multiline expression
                    if !in_multiline && Self::needs_continuation(&line) {
                        multiline_buffer.clone_from(&line);
                        in_multiline = true;
                        continue;
                    }

                    // If in multiline mode, accumulate lines
                    if in_multiline {
                        multiline_buffer.push('\n');
                        multiline_buffer.push_str(&line);

                        // Check if we have a complete expression
                        if !Self::needs_continuation(&multiline_buffer) {
                            // Add complete expression to history
                            let _ = rl.add_history_entry(multiline_buffer.as_str());

                            // Evaluate the complete expression
                            match self.eval(&multiline_buffer) {
                                Ok(result) => {
                                    println!("{}", result.bright_white());
                                }
                                Err(e) => {
                                    eprintln!("{}: {}", "Error".bright_red().bold(), e);
                                }
                            }

                            // Reset multiline mode
                            multiline_buffer.clear();
                            in_multiline = false;
                        }
                    } else {
                        // Single line expression
                        let _ = rl.add_history_entry(line.as_str());

                        // Evaluate the expression
                        match self.eval(&line) {
                            Ok(result) => {
                                println!("{}", result.bright_white());
                            }
                            Err(e) => {
                                eprintln!("{}: {}", "Error".bright_red().bold(), e);
                            }
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("\nUse :quit to exit");
                }
                Err(ReadlineError::Eof) => {
                    println!("\nGoodbye!");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {err:?}");
                    break;
                }
            }
        }

        // Save history
        let _ = rl.save_history(&history_path);
        Ok(())
    }

    /// Handle REPL commands (public for testing)
    ///
    /// # Errors
    ///
    /// Returns an error if command execution fails
    pub fn handle_command(&mut self, command: &str) -> Result<bool> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        match parts.first().copied() {
            Some(":quit" | ":q") => Ok(true),
            Some(":help" | ":h") => {
                Self::print_help();
                Ok(false)
            }
            Some(":history") => {
                for (i, item) in self.history.iter().enumerate() {
                    println!("{}: {}", i + 1, item);
                }
                Ok(false)
            }
            Some(":clear") => {
                self.history.clear();
                self.definitions.clear();
                self.bindings.clear();
                println!("Session cleared");
                Ok(false)
            }
            Some(":bindings" | ":env") => {
                if self.bindings.is_empty() {
                    println!("No bindings");
                } else {
                    for (name, value) in &self.bindings {
                        println!("{name}: {value}");
                    }
                }
                Ok(false)
            }
            Some(":compile") => {
                self.compile_session()?;
                Ok(false)
            }
            Some(":load") if parts.len() == 2 => {
                self.load_file(parts[1])?;
                Ok(false)
            }
            Some(":type") => {
                // Get the rest of the line after :type
                let expr = command.strip_prefix(":type").unwrap_or("").trim();
                if expr.is_empty() {
                    println!("Usage: :type <expression>");
                } else {
                    Self::show_type(expr);
                }
                Ok(false)
            }
            Some(":ast") => {
                // Get the rest of the line after :ast
                let expr = command.strip_prefix(":ast").unwrap_or("").trim();
                if expr.is_empty() {
                    println!("Usage: :ast <expression>");
                } else {
                    Self::show_ast(expr);
                }
                Ok(false)
            }
            Some(":reset") => {
                // Full reset - clear everything and restart
                self.history.clear();
                self.definitions.clear();
                self.bindings.clear();
                self.memory.reset();
                println!("REPL reset to initial state");
                Ok(false)
            }
            _ => {
                eprintln!("Unknown command: {command}");
                Self::print_help();
                Ok(false)
            }
        }
    }

    /// Print help message
    fn print_help() {
        println!("{}", "Available commands:".bright_cyan());
        println!("  :help, :h       - Show this help message");
        println!("  :quit, :q       - Exit the REPL");
        println!("  :history        - Show evaluation history");
        println!("  :clear          - Clear definitions and history");
        println!("  :reset          - Full reset to initial state");
        println!("  :bindings, :env - Show current variable bindings");
        println!("  :type <expr>    - Show type of expression");
        println!("  :ast <expr>     - Show AST of expression");
        println!("  :compile        - Compile and run the session");
        println!("  :load <file>    - Load and evaluate a file");
        println!();
        println!("{}", "Examples:".bright_cyan());
        println!("  2 + 2           - Evaluate expression");
        println!("  let x = 10      - Define variable");
        println!("  :type x * 2     - Show type of expression");
        println!("  :ast if true {{ 1 }} else {{ 2 }}");
    }

    /// Show the type of an expression
    fn show_type(expr: &str) {
        match Parser::new(expr).parse() {
            Ok(_ast) => {
                // For now, we don't have full type inference in REPL
                // Just show what we can determine from the expression
                println!("Type inference not yet implemented in REPL");
                println!("(This will show the inferred type once type checking is integrated)");
            }
            Err(e) => {
                eprintln!("Parse error: {e}");
            }
        }
    }

    /// Show the AST of an expression
    fn show_ast(expr: &str) {
        match Parser::new(expr).parse() {
            Ok(ast) => {
                println!("{ast:#?}");
            }
            Err(e) => {
                eprintln!("Parse error: {e}");
            }
        }
    }

    /// Check if input needs continuation (incomplete expression)
    pub fn needs_continuation(input: &str) -> bool {
        let trimmed = input.trim();

        // Empty input doesn't need continuation
        if trimmed.is_empty() {
            return false;
        }

        // Count braces, brackets, and parentheses
        let mut brace_depth = 0;
        let mut bracket_depth = 0;
        let mut paren_depth = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for ch in trimmed.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => brace_depth += 1,
                '}' if !in_string => brace_depth -= 1,
                '[' if !in_string => bracket_depth += 1,
                ']' if !in_string => bracket_depth -= 1,
                '(' if !in_string => paren_depth += 1,
                ')' if !in_string => paren_depth -= 1,
                _ => {}
            }
        }

        // Need continuation if any delimiters are unmatched
        brace_depth > 0 || bracket_depth > 0 || paren_depth > 0 || in_string ||
        // Or if line ends with certain tokens that expect continuation
        trimmed.ends_with('=') || 
        trimmed.ends_with("->") || 
        trimmed.ends_with("=>") ||
        trimmed.ends_with(',') ||
        trimmed.ends_with('+') ||
        trimmed.ends_with('-') ||
        trimmed.ends_with('*') ||
        trimmed.ends_with('/') ||
        trimmed.ends_with("&&") ||
        trimmed.ends_with("||") ||
        trimmed.ends_with("|>")
    }

    /// Compile and run the current session
    fn compile_session(&mut self) -> Result<()> {
        use std::fmt::Write;

        if self.history.is_empty() {
            println!("No expressions to compile");
            return Ok(());
        }

        println!("Compiling session...");

        // Generate Rust code for all expressions
        let mut rust_code = String::new();
        rust_code.push_str("#![allow(unused)]\n");
        rust_code.push_str("fn main() {\n");

        for expr in &self.history {
            match Parser::new(expr).parse() {
                Ok(ast) => {
                    let transpiled = self.transpiler.transpile(&ast)?;
                    let transpiled_str = transpiled.to_string();
                    // Check if this is already a print statement that should be executed directly
                    let trimmed = transpiled_str.trim();
                    if trimmed.starts_with("println !") || trimmed.starts_with("print !") || 
                       trimmed.starts_with("println!") || trimmed.starts_with("print!") {
                        let _ = writeln!(&mut rust_code, "    {transpiled};");
                    } else {
                        let _ = writeln!(
                            &mut rust_code,
                            "    println!(\"{{:?}}\", {{{transpiled}}});"
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse '{expr}': {e}");
                }
            }
        }

        rust_code.push_str("}\n");

        // Write to temporary file
        self.session_counter += 1;
        let file_name = format!("session_{}.rs", self.session_counter);
        let file_path = self.temp_dir.join(&file_name);
        fs::write(&file_path, rust_code)?;

        // Compile with rustc
        let output = Command::new("rustc")
            .arg(&file_path)
            .arg("-o")
            .arg(
                self.temp_dir
                    .join(format!("session_{}", self.session_counter)),
            )
            .current_dir(&self.temp_dir)
            .output()
            .context("Failed to run rustc")?;

        if !output.status.success() {
            eprintln!(
                "Compilation failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            return Ok(());
        }

        // Run the compiled program
        let exe_path = self
            .temp_dir
            .join(format!("session_{}", self.session_counter));
        let output = Command::new(&exe_path)
            .output()
            .context("Failed to run compiled program")?;

        println!("{}", "Output:".bright_green());
        print!("{}", String::from_utf8_lossy(&output.stdout));

        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    /// Load and evaluate a file
    fn load_file(&mut self, path: &str) -> Result<()> {
        let content =
            fs::read_to_string(path).with_context(|| format!("Failed to read file: {path}"))?;

        println!("Loading {path}...");

        for line in content.lines() {
            if line.trim().is_empty() || line.trim().starts_with("//") {
                continue;
            }

            match self.eval(line) {
                Ok(result) => {
                    println!("{}: {}", line.bright_black(), result);
                }
                Err(e) => {
                    eprintln!("{}: {} - {}", "Error".bright_red(), line, e);
                }
            }
        }

        Ok(())
    }

    /// Evaluate literal expressions
    fn evaluate_literal(&mut self, lit: &Literal) -> Result<Value> {
        match lit {
            Literal::Integer(n) => Ok(Value::Int(*n)),
            Literal::Float(f) => Ok(Value::Float(*f)),
            Literal::String(s) => {
                self.memory.try_alloc(s.len())?;
                Ok(Value::String(s.clone()))
            }
            Literal::Bool(b) => Ok(Value::Bool(*b)),
            Literal::Char(c) => Ok(Value::Char(*c)),
            Literal::Unit => Ok(Value::Unit),
        }
    }

    /// Evaluate if expressions
    fn evaluate_if(
        &mut self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let cond_val = self.evaluate_expr(condition, deadline, depth + 1)?;
        match cond_val {
            Value::Bool(true) => self.evaluate_expr(then_branch, deadline, depth + 1),
            Value::Bool(false) => {
                if let Some(else_expr) = else_branch {
                    self.evaluate_expr(else_expr, deadline, depth + 1)
                } else {
                    Ok(Value::Unit)
                }
            }
            _ => bail!("If condition must be boolean, got: {:?}", cond_val),
        }
    }

    /// Evaluate function calls
    fn evaluate_call(
        &mut self,
        func: &Expr,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if let ExprKind::Identifier(func_name) = &func.kind {
            match func_name.as_str() {
                "println" => self.evaluate_println(args, deadline, depth),
                "print" => self.evaluate_print(args, deadline, depth),
                _ => self.evaluate_user_function(func_name, args, deadline, depth),
            }
        } else {
            bail!("Complex function calls not yet supported");
        }
    }

    /// Evaluate println function
    fn evaluate_println(&mut self, args: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        let mut output = String::new();
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                output.push(' ');
            }
            let val = self.evaluate_expr(arg, deadline, depth + 1)?;
            match val {
                Value::String(s) => output.push_str(&s),
                other => output.push_str(&other.to_string()),
            }
        }
        println!("{output}");
        Ok(Value::Unit)
    }

    /// Evaluate print function
    fn evaluate_print(&mut self, args: &[Expr], deadline: Instant, depth: usize) -> Result<Value> {
        let mut output = String::new();
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                output.push(' ');
            }
            let val = self.evaluate_expr(arg, deadline, depth + 1)?;
            match val {
                Value::String(s) => output.push_str(&s),
                other => output.push_str(&other.to_string()),
            }
        }
        print!("{output}");
        Ok(Value::Unit)
    }

    /// Evaluate user-defined functions
    fn evaluate_user_function(
        &mut self,
        func_name: &str,
        args: &[Expr],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        if let Some(func_value) = self.bindings.get(func_name).cloned() {
            if let Value::Function { params, body, .. } = func_value {
                if args.len() != params.len() {
                    bail!(
                        "Function {} expects {} arguments, got {}",
                        func_name,
                        params.len(),
                        args.len()
                    );
                }

                let saved_bindings = self.bindings.clone();

                for (param, arg) in params.iter().zip(args.iter()) {
                    let arg_value = self.evaluate_expr(arg, deadline, depth + 1)?;
                    self.bindings.insert(param.clone(), arg_value);
                }

                let result = self.evaluate_expr(&body, deadline, depth + 1)?;
                self.bindings = saved_bindings;
                Ok(result)
            } else {
                bail!("'{}' is not a function", func_name);
            }
        } else {
            bail!("Unknown function: {}", func_name);
        }
    }

    /// Evaluate match expressions
    fn evaluate_match(
        &mut self,
        match_expr: &Expr,
        arms: &[MatchArm],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let match_value = self.evaluate_expr(match_expr, deadline, depth + 1)?;

        for arm in arms {
            if let Some(bindings) = Self::pattern_matches(&match_value, &arm.pattern)? {
                let saved_bindings = self.bindings.clone();

                for (name, value) in bindings {
                    self.bindings.insert(name, value);
                }

                let guard_passes = if let Some(guard) = &arm.guard {
                    let guard_val = self.evaluate_expr(guard, deadline, depth + 1)?;
                    match guard_val {
                        Value::Bool(true) => true,
                        Value::Bool(false) => false,
                        _ => bail!("Guard expression must be boolean"),
                    }
                } else {
                    true
                };

                if guard_passes {
                    let result = self.evaluate_expr(&arm.body, deadline, depth + 1)?;
                    self.bindings = saved_bindings;
                    return Ok(result);
                }

                self.bindings = saved_bindings;
            }
        }

        bail!("No matching pattern found in match expression");
    }

    /// Evaluate pipeline expressions
    fn evaluate_pipeline(
        &mut self,
        expr: &Expr,
        stages: &[PipelineStage],
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        let mut current_value = self.evaluate_expr(expr, deadline, depth + 1)?;

        for stage in stages {
            current_value = self.evaluate_pipeline_stage(&current_value, stage, deadline, depth)?;
        }

        Ok(current_value)
    }

    /// Evaluate a single pipeline stage
    fn evaluate_pipeline_stage(
        &mut self,
        current_value: &Value,
        stage: &PipelineStage,
        deadline: Instant,
        depth: usize,
    ) -> Result<Value> {
        match &stage.op.kind {
            ExprKind::Call { func, args } => {
                let mut new_args = vec![Self::value_to_literal_expr(current_value, stage.span)?];
                new_args.extend(args.iter().cloned());

                let new_call = Expr::new(
                    ExprKind::Call {
                        func: func.clone(),
                        args: new_args,
                    },
                    stage.span,
                );

                self.evaluate_expr(&new_call, deadline, depth + 1)
            }
            ExprKind::Identifier(_func_name) => {
                let call = Expr::new(
                    ExprKind::Call {
                        func: stage.op.clone(),
                        args: vec![Self::value_to_literal_expr(current_value, stage.span)?],
                    },
                    stage.span,
                );

                self.evaluate_expr(&call, deadline, depth + 1)
            }
            _ => bail!("Pipeline stages must be function calls or identifiers"),
        }
    }

    /// Convert value to literal expression for pipeline
    fn value_to_literal_expr(value: &Value, span: Span) -> Result<Expr> {
        let literal = match value {
            Value::Int(n) => Literal::Integer(*n),
            Value::Float(f) => Literal::Float(*f),
            Value::String(s) => Literal::String(s.clone()),
            Value::Bool(b) => Literal::Bool(*b),
            Value::Unit => Literal::Unit,
            _ => bail!("Cannot pipeline complex value types yet"),
        };
        Ok(Expr::new(ExprKind::Literal(literal), span))
    }
}
