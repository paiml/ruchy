//! REPL Module - Orchestrator for modular REPL implementation
//! 
//! This module coordinates all REPL functionality through clean interfaces.
//! Each submodule maintains complexity <15 per function for maintainability.
//!
//! # Architecture
//! 
//! - `evaluation`: Expression evaluation engine
//! - `completion`: Tab completion system  
//! - `history`: Command and result history
//! - `state`: Session state management
//! - `debug`: Debug and introspection
//! - `errors`: Error handling and recovery

pub mod completion;
pub mod debug;
pub mod errors;
pub mod evaluation;
pub mod history;
pub mod state;

use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

pub use self::errors::{ReplError, ErrorContext, RecoveryStrategy};
pub use self::state::{ReplMode, ReplConfig, ReplState};
pub use self::history::HistoryManager;
pub use self::debug::{DebugManager, DebugConfig};
pub use self::completion::CompletionEngine;
pub use self::evaluation::EvaluationConfig;

use crate::frontend::ast::{Expr, ExprKind, Literal};

/// Main REPL value type
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    Char(char),
    String(String),
    List(Vec<Value>),
    Tuple(Vec<Value>),
    HashMap(HashMap<Value, Value>),
    HashSet(HashSet<Value>),
    Range { start: i64, end: i64, inclusive: bool },
    Function { name: String, params: Vec<String>, body: Box<Expr> },
    Lambda { params: Vec<String>, body: Box<Expr> },
    Object(HashMap<String, Value>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Unit => write!(f, "()"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Char(c) => write!(f, "'{}'", c),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Tuple(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
            Value::HashMap(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::HashSet(set) => {
                write!(f, "#{{")?;
                for (i, item) in set.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "}}")
            }
            Value::Range { start, end, inclusive } => {
                if *inclusive {
                    write!(f, "{}..={}", start, end)
                } else {
                    write!(f, "{}..{}", start, end)
                }
            }
            Value::Function { name, params, .. } => {
                write!(f, "fn {}({})", name, params.join(", "))
            }
            Value::Lambda { params, .. } => {
                write!(f, "λ({})", params.join(", "))
            }
            Value::Object(fields) => {
                write!(f, "object {{")?;
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

// Hash implementation for Value (needed for HashMap keys)
impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Nil => 0.hash(state),
            Value::Unit => 1.hash(state),
            Value::Bool(b) => (2, b).hash(state),
            Value::Int(n) => (3, n).hash(state),
            Value::Float(f) => (4, f.to_bits()).hash(state),
            Value::Char(c) => (5, c).hash(state),
            Value::String(s) => (6, s).hash(state),
            Value::List(items) => (7, items).hash(state),
            Value::Tuple(items) => (8, items).hash(state),
            _ => (9, format!("{:?}", self)).hash(state),
        }
    }
}

impl Eq for Value {}

/// Main REPL structure - orchestrates all modules
pub struct Repl {
    /// Session state
    state: ReplState,
    /// Variable bindings
    bindings: BindingManager,
    /// Function definitions
    functions: HashMap<String, (Vec<String>, Box<Expr>)>,
    /// History manager
    history: HistoryManager,
    /// Completion engine
    completion: CompletionEngine,
    /// Debug manager
    debug: DebugManager,
    /// Error handler
    error_handler: errors::ErrorHandler,
}

impl Repl {
    /// Create new REPL instance (complexity: 3)
    pub fn new() -> Result<Self> {
        let config = ReplConfig::default();
        let history_config = history::HistoryConfig::default();
        let debug_config = DebugConfig::default();
        
        Ok(Self {
            state: ReplState::with_config(config),
            bindings: BindingManager::new(),
            functions: HashMap::new(),
            history: HistoryManager::new(history_config),
            completion: CompletionEngine::new(),
            debug: DebugManager::new(debug_config),
            error_handler: errors::ErrorHandler::new(),
        })
    }

    /// Main evaluation entry point (complexity: 10)
    pub fn eval(&mut self, input: &str) -> Result<String> {
        // Add to history
        self.history.add_command(input.to_string());
        
        // Handle special cases
        if input.starts_with(':') {
            return self.process_command(input);
        }
        
        if input.starts_with('!') {
            return self.execute_shell_command(&input[1..]);
        }
        
        // Parse input
        let expr = match self.parse_input(input) {
            Ok(e) => e,
            Err(e) => {
                let error = ReplError::ParseError {
                    message: e.to_string(),
                    position: 0,
                };
                let context = ErrorContext::new().with_source(input.to_string());
                let strategy = self.error_handler.handle_error(error, context)?;
                return self.apply_recovery_strategy(strategy);
            }
        };
        
        // Evaluate with timeout
        let deadline = Instant::now() + self.state.config().timeout;
        match self.evaluate_expression(&expr, deadline) {
            Ok(value) => {
                self.history.add_result(value.clone());
                self.state.record_evaluation(true, deadline.elapsed());
                Ok(value.to_string())
            }
            Err(e) => {
                self.state.record_evaluation(false, deadline.elapsed());
                Err(e)
            }
        }
    }

    /// Process REPL command (complexity: 8)
    fn process_command(&mut self, command: &str) -> Result<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        match parts.first().map(|s| *s) {
            Some(":help") => Ok(self.show_help()),
            Some(":quit" | ":exit") => bail!("Exit requested"),
            Some(":history") => Ok(self.history.format_display()),
            Some(":clear") => Ok(self.clear_screen()),
            Some(":reset") => {
                self.reset_state();
                Ok("State reset".to_string())
            }
            Some(":mode") => {
                if parts.len() > 1 {
                    if let Some(mode) = ReplMode::from_str(parts[1]) {
                        self.state.set_mode(mode);
                        Ok(format!("Mode set to {}", parts[1]))
                    } else {
                        Ok(format!("Unknown mode: {}", parts[1]))
                    }
                } else {
                    Ok(format!("Current mode: {}", self.state.mode().as_str()))
                }
            }
            Some(":debug") => {
                if parts.len() > 1 && parts[1] == "on" {
                    self.debug.config.trace_enabled = true;
                    Ok("Debug mode enabled".to_string())
                } else {
                    self.debug.config.trace_enabled = false;
                    Ok("Debug mode disabled".to_string())
                }
            }
            Some(":stats") => Ok(self.state.stats().format_display()),
            Some(cmd) => Ok(format!("Unknown command: {}", cmd)),
            None => Ok("Empty command".to_string()),
        }
    }

    /// Parse input string (complexity: 5)
    fn parse_input(&self, input: &str) -> Result<Expr> {
        // Simplified parsing - would use actual lexer/parser
        // This is a stub for the example
        Ok(Expr {
            kind: ExprKind::Literal(Literal::String(input.to_string())),
            span: Default::default(),
            attributes: Vec::new(),
        })
    }

    /// Evaluate expression (complexity: 8)
    fn evaluate_expression(&mut self, expr: &Expr, deadline: Instant) -> Result<Value> {
        // Set up evaluation context
        let mut context = evaluation::EvaluationContext {
            bindings: &mut self.bindings,
            functions: &self.functions,
            config: &EvaluationConfig::default(),
        };
        
        // Trace if debugging
        let trace_ctx = if self.debug.config.trace_enabled {
            self.debug.trace_eval_start(expr, 0)
        } else {
            None
        };
        
        // Evaluate
        let result = evaluation::evaluate_expression(expr, &mut context, deadline, 0);
        
        // Complete trace
        if let Some(ctx) = trace_ctx {
            if let Ok(ref val) = result {
                self.debug.trace_eval_end(ctx, val);
            }
        }
        
        result
    }

    /// Apply recovery strategy (complexity: 5)
    fn apply_recovery_strategy(&self, strategy: RecoveryStrategy) -> Result<String> {
        match strategy {
            RecoveryStrategy::Skip => Ok("Skipped".to_string()),
            RecoveryStrategy::UseDefault(val) => Ok(val),
            RecoveryStrategy::Retry { modification } => {
                Ok(format!("Retry with: {}", modification))
            }
            RecoveryStrategy::AskUser { prompt } => {
                Ok(format!("User input needed: {}", prompt))
            }
            RecoveryStrategy::Abort => bail!("Evaluation aborted"),
        }
    }

    /// Execute shell command (complexity: 4)
    fn execute_shell_command(&self, command: &str) -> Result<String> {
        use std::process::Command;
        
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            bail!("Command failed: {}", String::from_utf8_lossy(&output.stderr))
        }
    }

    /// Show help message (complexity: 2)
    fn show_help(&self) -> String {
        "REPL Commands:\n\
         :help     - Show this help\n\
         :quit     - Exit REPL\n\
         :history  - Show command history\n\
         :clear    - Clear screen\n\
         :reset    - Reset state\n\
         :mode     - Get/set mode\n\
         :debug    - Toggle debug mode\n\
         :stats    - Show statistics".to_string()
    }

    /// Clear screen (complexity: 2)
    fn clear_screen(&self) -> String {
        print!("\x1B[2J\x1B[1;1H");
        "Screen cleared".to_string()
    }

    /// Reset state (complexity: 3)
    fn reset_state(&mut self) {
        self.bindings.clear();
        self.functions.clear();
        self.history.clear();
        self.debug.clear();
        self.state.reset_stats();
    }

    /// Get completion suggestions (complexity: 3)
    pub fn get_completions(&mut self, input: &str, position: usize) -> Vec<String> {
        self.completion.get_completions(input, position)
            .into_iter()
            .map(|c| c.text)
            .collect()
    }
}

/// Binding manager implementation
struct BindingManager {
    bindings: HashMap<String, Value>,
    mutability: HashMap<String, bool>,
}

impl BindingManager {
    fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            mutability: HashMap::new(),
        }
    }

    fn set_binding(&mut self, name: String, value: Value, is_mutable: bool) -> Result<()> {
        if let Some(mutable) = self.mutability.get(&name) {
            if !mutable {
                bail!("Cannot reassign immutable binding '{}'", name);
            }
        }
        
        self.bindings.insert(name.clone(), value);
        self.mutability.insert(name, is_mutable);
        Ok(())
    }

    fn get_binding(&self, name: &str) -> Option<Value> {
        self.bindings.get(name).cloned()
    }

    fn clear(&mut self) {
        self.bindings.clear();
        self.mutability.clear();
    }
}

impl evaluation::BindingProvider for BindingManager {
    fn get_binding(&self, name: &str) -> Option<Value> {
        self.bindings.get(name).cloned()
    }

    fn set_binding(&mut self, name: String, value: Value, is_mutable: bool) -> Result<()> {
        BindingManager::set_binding(self, name, value, is_mutable)
    }

    fn push_scope(&mut self) {
        // Simplified - would need scope stack
    }

    fn pop_scope(&mut self) {
        // Simplified - would need scope stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_creation() {
        let repl = Repl::new().unwrap();
        assert_eq!(repl.state.mode(), &ReplMode::Normal);
    }

    #[test]
    fn test_value_display() {
        assert_eq!(Value::Int(42).to_string(), "42");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::String("hello".to_string()).to_string(), "\"hello\"");
    }

    #[test]
    fn test_value_hash() {
        use std::collections::HashSet;
        
        let mut set = HashSet::new();
        set.insert(Value::Int(42));
        set.insert(Value::Int(42));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_binding_manager() {
        let mut mgr = BindingManager::new();
        mgr.set_binding("x".to_string(), Value::Int(10), false).unwrap();
        assert_eq!(mgr.get_binding("x"), Some(Value::Int(10)));
        
        // Immutable binding
        assert!(mgr.set_binding("x".to_string(), Value::Int(20), false).is_err());
    }
}