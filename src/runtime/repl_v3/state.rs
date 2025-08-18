//! Transactional state machine for REPL v3
//!
//! Uses persistent data structures (im crate) for O(1) checkpointing
//! and automatic rollback on failure.

use anyhow::Result;
use im::HashMap;
use std::fmt;

/// Value type in the REPL environment
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Function(String), // Function name/signature
    Unit,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(x) => write!(f, "{}", x),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Function(sig) => write!(f, "<function: {}>", sig),
            Value::Unit => write!(f, "()"),
        }
    }
}

/// Type environment for type checking
#[derive(Clone, Debug)]
pub struct TypeEnv {
    types: HashMap<String, String>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }
    
    pub fn insert(&mut self, name: String, ty: String) {
        self.types.insert(name, ty);
    }
    
    pub fn get(&self, name: &str) -> Option<&String> {
        self.types.get(name)
    }
}

/// Checkpoint for state recovery
#[derive(Clone, Debug)]
pub struct Checkpoint {
    pub bindings: HashMap<String, Value>,
    pub types: TypeEnv,
    pub pc: usize, // Program counter for recovery
}

impl Checkpoint {
    /// Restore environment from checkpoint
    pub fn restore(self) -> Environment {
        Environment {
            bindings: self.bindings,
            types: self.types,
        }
    }
}

/// REPL environment with bindings
#[derive(Clone, Debug)]
pub struct Environment {
    pub bindings: HashMap<String, Value>,
    pub types: TypeEnv,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            types: TypeEnv::new(),
        }
    }
    
    /// Create a checkpoint of current state
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            bindings: self.bindings.clone(),
            types: self.types.clone(),
            pc: 0,
        }
    }
    
    /// Extend environment with new binding
    pub fn extend(&self, name: String, value: Value) -> Self {
        let mut env = self.clone();
        env.bindings.insert(name, value);
        env
    }
    
    /// Look up a binding
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }
}

/// State machine for REPL execution
#[derive(Clone, Debug)]
pub enum State {
    /// Ready to evaluate
    Ready(Environment),
    /// Currently evaluating with checkpoint
    Evaluating(Environment, Checkpoint),
    /// Failed with checkpoint for recovery
    Failed(Checkpoint),
}

impl State {
    /// Evaluate input and transition state
    pub fn eval(self, input: &str) -> (State, Result<Value>) {
        match self {
            State::Ready(env) => {
                let checkpoint = env.checkpoint();
                
                // Simulate evaluation (will be replaced with actual eval)
                if input.contains("error") {
                    (State::Failed(checkpoint), Err(anyhow::anyhow!("Evaluation failed")))
                } else {
                    let value = Value::Int(42); // Placeholder
                    let new_env = env.extend("_".to_string(), value.clone());
                    (State::Ready(new_env), Ok(value))
                }
            }
            State::Failed(checkpoint) => {
                // Restore from checkpoint
                let env = checkpoint.restore();
                (State::Ready(env), Err(anyhow::anyhow!("Recovered from failure")))
            }
            State::Evaluating(_, checkpoint) => {
                // Should not happen in normal flow
                (State::Failed(checkpoint), Err(anyhow::anyhow!("Invalid state")))
            }
        }
    }
    
    /// Check if state is valid
    pub fn is_valid(&self) -> bool {
        match self {
            State::Ready(_) => true,
            State::Failed(_) => true,
            State::Evaluating(_, _) => true,
        }
    }
}

/// Main REPL state container
pub struct ReplState {
    pub state: State,
    pub history: Vec<String>,
    pub mode: ReplMode,
}

/// REPL execution modes
#[derive(Clone, Debug, PartialEq)]
pub enum ReplMode {
    Standard,
    Test,
    Debug,
}

impl ReplState {
    pub fn new() -> Self {
        Self {
            state: State::Ready(Environment::new()),
            history: Vec::new(),
            mode: ReplMode::Standard,
        }
    }
    
    /// Switch REPL mode
    pub fn set_mode(&mut self, mode: ReplMode) {
        self.mode = mode;
    }
    
    /// Add command to history
    pub fn add_history(&mut self, cmd: String) {
        self.history.push(cmd);
    }
}