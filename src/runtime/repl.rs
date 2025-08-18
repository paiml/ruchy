//! REPL implementation for interactive Ruchy development
//!
//! Production-grade REPL with resource bounds, error recovery, and grammar coverage

#![allow(clippy::print_stdout)] // REPL needs to print to stdout
#![allow(clippy::print_stderr)] // REPL needs to print errors
#![allow(clippy::expect_used)] // REPL can panic on initialization failure

use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, UnaryOp};
use crate::{Parser, Transpiler};
use anyhow::{bail, Context, Result};
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::collections::HashMap;
use std::fmt;
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

        // Handle let bindings specially
        if let ExprKind::Let { name, .. } = &ast.kind {
            self.bindings.insert(name.clone(), value.clone());
        }

        Ok(value.to_string())
    }

    /// Evaluate an expression to a value
    fn evaluate_expr(&mut self, expr: &Expr, deadline: Instant, depth: usize) -> Result<Value> {
        // Check resource bounds
        if Instant::now() > deadline {
            bail!("Evaluation timeout exceeded");
        }
        if depth > self.config.max_depth {
            bail!("Maximum recursion depth {} exceeded", self.config.max_depth);
        }

        match &expr.kind {
            ExprKind::Literal(lit) => match lit {
                Literal::Integer(n) => Ok(Value::Int(*n)),
                Literal::Float(f) => Ok(Value::Float(*f)),
                Literal::String(s) => {
                    self.memory.try_alloc(s.len())?;
                    Ok(Value::String(s.clone()))
                }
                Literal::Bool(b) => Ok(Value::Bool(*b)),
                Literal::Unit => Ok(Value::Unit),
            },
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
            ExprKind::Let { name, value, .. } => {
                let val = self.evaluate_expr(value, deadline, depth + 1)?;
                self.bindings.insert(name.clone(), val.clone());
                Ok(val)
            }
            ExprKind::If { condition, then_branch, else_branch } => {
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
                // For now, just evaluate each element and return the last one
                // In a full implementation, this would return a proper list value
                if elements.is_empty() {
                    Ok(Value::Unit)
                } else {
                    let mut results = Vec::new();
                    for elem in elements {
                        let val = self.evaluate_expr(elem, deadline, depth + 1)?;
                        results.push(val);
                    }
                    // For REPL demonstration, just show the first element
                    // In a real implementation, we'd have a List(Vec<Value>) variant
                    Ok(results.into_iter().next().unwrap_or(Value::Unit))
                }
            }
            ExprKind::Assign { target, value } => {
                let val = self.evaluate_expr(value, deadline, depth + 1)?;
                
                // For now, only support simple variable assignment
                if let ExprKind::Identifier(name) = &target.kind {
                    self.bindings.insert(name.clone(), val.clone());
                    Ok(val)
                } else {
                    bail!("Only simple variable assignment is supported, got: {:?}", target.kind);
                }
            }
            ExprKind::Range { start, end, inclusive: _ } => {
                let start_val = self.evaluate_expr(start, deadline, depth + 1)?;
                let end_val = self.evaluate_expr(end, deadline, depth + 1)?;
                
                // For REPL demo, just return a string representation
                match (start_val, end_val) {
                    (Value::Int(s), Value::Int(e)) => {
                        Ok(Value::String(format!("{s}..{e}")))
                    }
                    _ => bail!("Range endpoints must be integers")
                }
            }
            ExprKind::Function { name, params, .. } => {
                // Store function definition (simplified for REPL demo)
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let func_signature = format!("fn {}({})", name, param_names.join(", "));
                
                // Store as a special function value
                self.bindings.insert(name.clone(), Value::String(func_signature.clone()));
                Ok(Value::String(func_signature))
            }
            ExprKind::Lambda { params, .. } => {
                // Lambda expressions (simplified for REPL demo)  
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let lambda_signature = format!("|{}| <body>", param_names.join(", "));
                Ok(Value::String(lambda_signature))
            }
            ExprKind::Call { func, args } => {
                // Handle built-in functions
                //
                // # Examples
                //
                // ```rust
                // use ruchy::Repl;
                // let mut repl = Repl::new().unwrap();
                // 
                // // Basic println
                // let result = repl.eval(r#"println("Hello, World!")"#).unwrap();
                // assert_eq!(result, "()");
                //
                // // Multiple arguments
                // let result = repl.eval(r#"println("Hello", "World", "!")"#).unwrap();
                // assert_eq!(result, "()");
                //
                // // With variables
                // repl.eval("let x = 42").unwrap();
                // let result = repl.eval("println(x)").unwrap();
                // assert_eq!(result, "()");
                // ```
                if let ExprKind::Identifier(func_name) = &func.kind {
                    match func_name.as_str() {
                        "println" => {
                            // Evaluate arguments
                            let mut output = String::new();
                            for (i, arg) in args.iter().enumerate() {
                                if i > 0 {
                                    output.push(' ');
                                }
                                let val = self.evaluate_expr(arg, deadline, depth + 1)?;
                                // Format the value for printing (without quotes for strings)
                                match val {
                                    Value::String(s) => output.push_str(&s),
                                    other => output.push_str(&other.to_string()),
                                }
                            }
                            // Print to stdout (REPL allows this)
                            println!("{output}");
                            Ok(Value::Unit)
                        }
                        "print" => {
                            // Same as println but without newline
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
                        _ => bail!("Unknown function: {}", func_name),
                    }
                } else {
                    bail!("Complex function calls not yet supported");
                }
            }
            _ => bail!("Expression type not yet implemented: {:?}", expr.kind),
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
                let exp = u32::try_from(*b).map_err(|_| anyhow::anyhow!("Power exponent too large"))?;
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

    /// Handle REPL commands
    fn handle_command(&mut self, command: &str) -> Result<bool> {
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
    fn needs_continuation(input: &str) -> bool {
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
                    let _ = writeln!(
                        &mut rust_code,
                        "    println!(\"{{:?}}\", {{{transpiled}}});"
                    );
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
}
