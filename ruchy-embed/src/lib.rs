//! # ruchy-embed: Embeddable Ruchy Scripting Engine
//!
//! Embed Ruchy as a scripting language in Rust applications.

use anyhow::Result;
use ruchy::runtime::interpreter::Interpreter;
use ruchy::runtime::value::Value as RuchyValue;
use std::collections::HashMap;

/// The main embeddable scripting engine.
pub struct Engine {
    interp: Interpreter,
    globals: HashMap<String, Value>,
}

/// Runtime values in the embedded engine.
#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    None,
}

/// A pre-compiled script for repeated evaluation.
pub struct CompiledScript {
    source: String,
}

impl Engine {
    /// Create a new engine with safe defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            interp: Interpreter::new(),
            globals: HashMap::new(),
        }
    }

    /// Set a global variable accessible to scripts.
    pub fn set(&mut self, name: &str, value: impl Into<Value>) {
        let v = value.into();
        let rv = embed_to_ruchy(&v);
        self.interp.set_variable(name, rv);
        self.globals.insert(name.to_string(), v);
    }

    /// Get a global variable from the script context.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.globals.get(name)
    }

    /// Compile a script for repeated evaluation (validates syntax).
    pub fn compile(&self, source: &str) -> Result<CompiledScript> {
        let mut parser = ruchy::Parser::new(source);
        let _ast = parser
            .parse()
            .map_err(|e| anyhow::anyhow!("Parse error: {e}"))?;
        Ok(CompiledScript {
            source: source.to_string(),
        })
    }

    /// Evaluate a source string and return the result.
    pub fn eval(&mut self, source: &str) -> Result<Value> {
        let mut parser = ruchy::Parser::new(source);
        let ast = parser
            .parse()
            .map_err(|e| anyhow::anyhow!("Parse error: {e}"))?;
        let result = self
            .interp
            .eval_expr(&ast)
            .map_err(|e| anyhow::anyhow!("Eval error: {e}"))?;
        Ok(ruchy_to_embed(result))
    }

    /// Evaluate a pre-compiled script.
    pub fn eval_compiled(&mut self, script: &CompiledScript) -> Result<Value> {
        self.eval(&script.source)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Integer(v)
    }
}
impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Float(v)
    }
}
impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}
impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}
impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_string())
    }
}

fn embed_to_ruchy(v: &Value) -> RuchyValue {
    match v {
        Value::Integer(n) => RuchyValue::Integer(*n),
        Value::Float(f) => RuchyValue::Float(*f),
        Value::Bool(b) => RuchyValue::Bool(*b),
        Value::String(s) => RuchyValue::String(s.as_str().into()),
        Value::None => RuchyValue::Nil,
    }
}

fn ruchy_to_embed(v: RuchyValue) -> Value {
    match v {
        RuchyValue::Integer(n) => Value::Integer(n),
        RuchyValue::Float(f) => Value::Float(f),
        RuchyValue::Bool(b) => Value::Bool(b),
        RuchyValue::String(s) => Value::String(s.to_string()),
        RuchyValue::Nil => Value::None,
        other => Value::String(format!("{other:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_new() {
        let engine = Engine::new();
        assert!(engine.globals.is_empty());
    }

    #[test]
    fn test_engine_set_get() {
        let mut engine = Engine::new();
        engine.set("x", 42i64);
        assert!(engine.get("x").is_some());
    }

    #[test]
    fn test_engine_compile_valid() {
        let engine = Engine::new();
        let result = engine.compile("1 + 2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_engine_eval_literal() {
        let mut engine = Engine::new();
        let result = engine.eval("42");
        assert!(result.is_ok(), "Eval failed: {:?}", result.err());
        match result.unwrap() {
            Value::Integer(n) => assert_eq!(n, 42),
            other => panic!("Expected Integer(42), got {:?}", other),
        }
    }

    #[test]
    fn test_engine_eval_arithmetic() {
        let mut engine = Engine::new();
        let result = engine.eval("10 + 32");
        assert!(result.is_ok(), "Eval failed: {:?}", result.err());
        match result.unwrap() {
            Value::Integer(n) => assert_eq!(n, 42),
            other => panic!("Expected Integer(42), got {:?}", other),
        }
    }

    #[test]
    fn test_engine_eval_with_global() {
        let mut engine = Engine::new();
        engine.set("bonus", 50i64);
        let result = engine.eval("bonus");
        assert!(result.is_ok(), "Eval failed: {:?}", result.err());
    }
}
