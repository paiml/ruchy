//! Reference Interpreter - Ground Truth for Semantic Verification
//! 
//! A minimal, unoptimized, obviously correct interpreter for the core language.
//! This serves as the oracle for differential testing against the transpiler.
//! 
//! Design principles:
//! - Clarity over performance
//! - No optimizations whatsoever
//! - Under 1000 LOC
//! - Direct operational semantics

use crate::transpiler::canonical_ast::{CoreExpr, CoreLiteral, DeBruijnIndex, PrimOp};
use std::rc::Rc;

/// Runtime values
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
    /// Closure captures the body and environment at creation time
    Closure {
        body: Rc<CoreExpr>,
        env: Environment,
    },
    /// Arrays are just vectors
    Array(Vec<Value>),
}

/// Environment for variable bindings
#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    bindings: Vec<Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self { bindings: Vec::new() }
    }
    
    pub fn push(&mut self, value: Value) {
        self.bindings.push(value);
    }
    
    pub fn pop(&mut self) {
        self.bindings.pop();
    }
    
    pub fn lookup(&self, index: &DeBruijnIndex) -> Option<&Value> {
        // De Bruijn indices count from the end
        let pos = self.bindings.len().checked_sub(index.0 + 1)?;
        self.bindings.get(pos)
    }
}

/// Reference interpreter - deliberately simple and unoptimized
pub struct ReferenceInterpreter {
    env: Environment,
    trace: Vec<String>, // For debugging
}

impl ReferenceInterpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
            trace: Vec::new(),
        }
    }
    
    /// Evaluate an expression to a value
    /// This is the core of the interpreter - direct operational semantics
    pub fn eval(&mut self, expr: &CoreExpr) -> Result<Value, String> {
        self.trace.push(format!("Evaluating: {:?}", expr));
        
        match expr {
            CoreExpr::Var(idx) => {
                self.env.lookup(idx)
                    .cloned()
                    .ok_or_else(|| format!("Unbound variable: {:?}", idx))
            }
            
            CoreExpr::Lambda { body, .. } => {
                // Create closure capturing current environment
                Ok(Value::Closure {
                    body: Rc::new(body.as_ref().clone()),
                    env: self.env.clone(),
                })
            }
            
            CoreExpr::App(func, arg) => {
                // Evaluate function
                let func_val = self.eval(func)?;
                
                // Evaluate argument (call-by-value)
                let arg_val = self.eval(arg)?;
                
                // Apply function to argument
                match func_val {
                    Value::Closure { body, mut env } => {
                        // Save current environment
                        let saved_env = self.env.clone();
                        
                        // Set up closure environment with argument
                        env.push(arg_val);
                        self.env = env;
                        
                        // Evaluate body
                        let result = self.eval(&body)?;
                        
                        // Restore environment
                        self.env = saved_env;
                        
                        Ok(result)
                    }
                    _ => Err(format!("Cannot apply non-function: {:?}", func_val))
                }
            }
            
            CoreExpr::Let { value, body, name } => {
                self.trace.push(format!("Let binding: {:?}", name));
                
                // Evaluate the value
                let val = self.eval(value)?;
                
                // Bind it in the environment
                self.env.push(val.clone());
                
                // Evaluate the body
                let result = self.eval(body)?;
                
                // Pop the binding
                self.env.pop();
                
                Ok(result)
            }
            
            CoreExpr::Literal(lit) => {
                Ok(match lit {
                    CoreLiteral::Integer(i) => Value::Integer(*i),
                    CoreLiteral::Float(f) => Value::Float(*f),
                    CoreLiteral::String(s) => Value::String(s.clone()),
                    CoreLiteral::Bool(b) => Value::Bool(*b),
                    CoreLiteral::Unit => Value::Unit,
                })
            }
            
            CoreExpr::Prim(op, args) => {
                self.eval_prim(op, args)
            }
        }
    }
    
    /// Evaluate primitive operations
    fn eval_prim(&mut self, op: &PrimOp, args: &[CoreExpr]) -> Result<Value, String> {
        // Evaluate all arguments first (strict evaluation)
        let mut values = Vec::new();
        for arg in args {
            values.push(self.eval(arg)?);
        }
        
        match op {
            // Arithmetic operations
            PrimOp::Add => {
                if values.len() != 2 {
                    return Err(format!("Add expects 2 arguments, got {}", values.len()));
                }
                match (&values[0], &values[1]) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        Ok(Value::Integer(a + b))
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        Ok(Value::Float(a + b))
                    }
                    (Value::String(a), Value::String(b)) => {
                        Ok(Value::String(format!("{}{}", a, b)))
                    }
                    _ => Err(format!("Type error in addition: {:?} + {:?}", values[0], values[1]))
                }
            }
            
            PrimOp::Sub => {
                match (&values[0], &values[1]) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                    _ => Err("Type error in subtraction".to_string())
                }
            }
            
            PrimOp::Mul => {
                match (&values[0], &values[1]) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                    _ => Err("Type error in multiplication".to_string())
                }
            }
            
            PrimOp::Div => {
                match (&values[0], &values[1]) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if *b == 0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(Value::Integer(a / b))
                        }
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        if *b == 0.0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(Value::Float(a / b))
                        }
                    }
                    _ => Err("Type error in division".to_string())
                }
            }
            
            PrimOp::Mod => {
                match (&values[0], &values[1]) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if *b == 0 {
                            Err("Modulo by zero".to_string())
                        } else {
                            Ok(Value::Integer(a % b))
                        }
                    }
                    _ => Err("Type error in modulo".to_string())
                }
            }
            
            PrimOp::Pow => {
                match (&values[0], &values[1]) {
                    (Value::Integer(a), Value::Integer(b)) => {
                        if *b < 0 {
                            Err("Negative exponent for integer".to_string())
                        } else {
                            Ok(Value::Integer(a.pow(*b as u32)))
                        }
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        Ok(Value::Float(a.powf(*b)))
                    }
                    _ => Err("Type error in power".to_string())
                }
            }
            
            // Comparison operations
            PrimOp::Eq => {
                Ok(Value::Bool(values[0] == values[1]))
            }
            
            PrimOp::Ne => {
                Ok(Value::Bool(values[0] != values[1]))
            }
            
            PrimOp::Lt => {
                match (&values[0], &values[1]) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a < b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
                    _ => Err("Type error in less-than".to_string())
                }
            }
            
            PrimOp::Le => {
                match (&values[0], &values[1]) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a <= b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
                    _ => Err("Type error in less-equal".to_string())
                }
            }
            
            PrimOp::Gt => {
                match (&values[0], &values[1]) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a > b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
                    _ => Err("Type error in greater-than".to_string())
                }
            }
            
            PrimOp::Ge => {
                match (&values[0], &values[1]) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a >= b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
                    _ => Err("Type error in greater-equal".to_string())
                }
            }
            
            // Logical operations
            PrimOp::And => {
                match (&values[0], &values[1]) {
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
                    _ => Err("Type error in AND".to_string())
                }
            }
            
            PrimOp::Or => {
                match (&values[0], &values[1]) {
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
                    _ => Err("Type error in OR".to_string())
                }
            }
            
            PrimOp::Not => {
                if values.len() != 1 {
                    return Err(format!("NOT expects 1 argument, got {}", values.len()));
                }
                match &values[0] {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    _ => Err("Type error in NOT".to_string())
                }
            }
            
            // Control flow
            PrimOp::If => {
                if values.len() != 3 {
                    return Err(format!("IF expects 3 arguments, got {}", values.len()));
                }
                
                // Note: We already evaluated all branches (strict evaluation)
                // A lazy interpreter would evaluate condition first, then the appropriate branch
                match &values[0] {
                    Value::Bool(true) => Ok(values[1].clone()),
                    Value::Bool(false) => Ok(values[2].clone()),
                    _ => Err("Type error: IF condition must be boolean".to_string())
                }
            }
            
            // Array operations
            PrimOp::ArrayNew => {
                // Create array from all arguments
                Ok(Value::Array(values))
            }
            
            PrimOp::ArrayIndex => {
                if values.len() != 2 {
                    return Err("Array index expects 2 arguments".to_string());
                }
                match (&values[0], &values[1]) {
                    (Value::Array(arr), Value::Integer(idx)) => {
                        if *idx < 0 || *idx as usize >= arr.len() {
                            Err(format!("Array index out of bounds: {}", idx))
                        } else {
                            Ok(arr[*idx as usize].clone())
                        }
                    }
                    _ => Err("Type error in array indexing".to_string())
                }
            }
            
            PrimOp::ArrayLen => {
                if values.len() != 1 {
                    return Err("Array length expects 1 argument".to_string());
                }
                match &values[0] {
                    Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
                    _ => Err("Type error: expected array".to_string())
                }
            }
            
            _ => Err(format!("Unsupported primitive: {:?}", op))
        }
    }
    
    /// Get execution trace for debugging
    pub fn get_trace(&self) -> &[String] {
        &self.trace
    }
    
    /// Clear the trace
    pub fn clear_trace(&mut self) {
        self.trace.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transpiler::canonical_ast::AstNormalizer;
    use crate::Parser;
    
    #[test]
    fn test_eval_arithmetic() {
        let input = "1 + 2 * 3";
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();
        
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        
        let mut interp = ReferenceInterpreter::new();
        let result = interp.eval(&core).unwrap();
        
        assert_eq!(result, Value::Integer(7)); // 1 + (2 * 3)
    }
    
    #[test]
    fn test_eval_let_binding() {
        let input = "let x = 10";
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();
        
        let mut normalizer = AstNormalizer::new();
        let core = normalizer.normalize(&ast);
        
        let mut interp = ReferenceInterpreter::new();
        let result = interp.eval(&core).unwrap();
        
        // Let with unit body evaluates to unit
        assert_eq!(result, Value::Unit);
    }
    
    #[test]
    fn test_eval_function() {
        // This would need more setup to test properly
        // as we need to handle function definitions and calls
    }
}