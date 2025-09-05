//! REPL expression evaluator module
//! Extracted from repl.rs for modularity (complexity: ≤10 per function)

use super::value::Value;
use crate::frontend::ast::{Expr, ExprKind, Stmt, StmtKind, BinaryOp, UnaryOp};
use std::collections::HashMap;

/// Expression evaluator for the REPL
pub struct Evaluator {
    bindings: HashMap<String, Value>,
    call_depth: usize,
    max_call_depth: usize,
}

impl Evaluator {
    /// Create a new evaluator
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            call_depth: 0,
            max_call_depth: 1000,
        }
    }

    /// Set a binding
    pub fn set_binding(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    /// Get a binding
    pub fn get_binding(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }

    /// Clear all bindings
    pub fn clear_bindings(&mut self) {
        self.bindings.clear();
    }

    /// Get all bindings
    pub fn bindings(&self) -> &HashMap<String, Value> {
        &self.bindings
    }

    /// Evaluate an expression
    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, String> {
        self.check_call_depth()?;
        self.call_depth += 1;
        let result = self.evaluate_expr(expr);
        self.call_depth -= 1;
        result
    }

    /// Check call depth limit
    fn check_call_depth(&self) -> Result<(), String> {
        if self.call_depth >= self.max_call_depth {
            Err(format!("Maximum call depth {} exceeded", self.max_call_depth))
        } else {
            Ok(())
        }
    }

    /// Main expression evaluation dispatch
    fn evaluate_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match &expr.kind {
            ExprKind::Literal(lit) => self.evaluate_literal(lit),
            ExprKind::Identifier(name) => self.evaluate_identifier(name),
            ExprKind::Binary { op, left, right } => {
                self.evaluate_binary(op, left, right)
            }
            ExprKind::Unary { op, operand } => self.evaluate_unary(op, operand),
            ExprKind::If { condition, then_expr, else_expr } => {
                self.evaluate_if(condition, then_expr, else_expr.as_deref())
            }
            ExprKind::List(exprs) => self.evaluate_list(exprs),
            ExprKind::Tuple(exprs) => self.evaluate_tuple(exprs),
            ExprKind::Index { object, index } => self.evaluate_index(object, index),
            ExprKind::Call { func, args } => self.evaluate_call(func, args),
            _ => Err(format!("Unsupported expression: {:?}", expr.kind)),
        }
    }

    /// Evaluate a literal
    fn evaluate_literal(&self, lit: &crate::frontend::ast::Literal) -> Result<Value, String> {
        use crate::frontend::ast::Literal;
        match lit {
            Literal::Integer(i) => Ok(Value::Int(*i)),
            Literal::Float(f) => Ok(Value::Float(*f)),
            Literal::String(s) => Ok(Value::String(s.clone())),
            Literal::Bool(b) => Ok(Value::Bool(*b)),
            Literal::Char(c) => Ok(Value::Char(*c)),
            Literal::None => Ok(Value::Nil),
        }
    }

    /// Evaluate an identifier
    fn evaluate_identifier(&self, name: &str) -> Result<Value, String> {
        self.bindings
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Undefined variable: {}", name))
    }

    /// Evaluate a binary operation
    fn evaluate_binary(
        &mut self,
        op: &BinaryOp,
        left: &Expr,
        right: &Expr,
    ) -> Result<Value, String> {
        let left_val = self.evaluate(left)?;
        let right_val = self.evaluate(right)?;

        match op {
            BinaryOp::Add => self.add_values(&left_val, &right_val),
            BinaryOp::Sub => self.subtract_values(&left_val, &right_val),
            BinaryOp::Mul => self.multiply_values(&left_val, &right_val),
            BinaryOp::Div => self.divide_values(&left_val, &right_val),
            BinaryOp::Mod => self.modulo_values(&left_val, &right_val),
            BinaryOp::Eq => Ok(Value::Bool(left_val == right_val)),
            BinaryOp::Ne => Ok(Value::Bool(left_val != right_val)),
            BinaryOp::Lt => self.less_than(&left_val, &right_val),
            BinaryOp::Le => self.less_equal(&left_val, &right_val),
            BinaryOp::Gt => self.greater_than(&left_val, &right_val),
            BinaryOp::Ge => self.greater_equal(&left_val, &right_val),
            BinaryOp::And => self.logical_and(&left_val, &right_val),
            BinaryOp::Or => self.logical_or(&left_val, &right_val),
            _ => Err(format!("Unsupported binary operator: {:?}", op)),
        }
    }

    /// Evaluate a unary operation
    fn evaluate_unary(&mut self, op: &UnaryOp, operand: &Expr) -> Result<Value, String> {
        let val = self.evaluate(operand)?;

        match op {
            UnaryOp::Not => self.logical_not(&val),
            UnaryOp::Neg => self.negate_value(&val),
            _ => Err(format!("Unsupported unary operator: {:?}", op)),
        }
    }

    /// Evaluate an if expression
    fn evaluate_if(
        &mut self,
        condition: &Expr,
        then_expr: &Expr,
        else_expr: Option<&Expr>,
    ) -> Result<Value, String> {
        let cond_val = self.evaluate(condition)?;
        
        if cond_val.is_truthy() {
            self.evaluate(then_expr)
        } else if let Some(else_e) = else_expr {
            self.evaluate(else_e)
        } else {
            Ok(Value::Unit)
        }
    }

    /// Evaluate a list literal
    fn evaluate_list(&mut self, exprs: &[Expr]) -> Result<Value, String> {
        let mut values = Vec::new();
        for expr in exprs {
            values.push(self.evaluate(expr)?);
        }
        Ok(Value::List(values))
    }

    /// Evaluate a tuple literal
    fn evaluate_tuple(&mut self, exprs: &[Expr]) -> Result<Value, String> {
        let mut values = Vec::new();
        for expr in exprs {
            values.push(self.evaluate(expr)?);
        }
        Ok(Value::Tuple(values))
    }

    /// Evaluate an index operation
    fn evaluate_index(&mut self, object: &Expr, index: &Expr) -> Result<Value, String> {
        let obj_val = self.evaluate(object)?;
        let idx_val = self.evaluate(index)?;

        match (&obj_val, &idx_val) {
            (Value::List(list), Value::Int(i)) => {
                let idx = if *i < 0 {
                    (list.len() as i64 + i) as usize
                } else {
                    *i as usize
                };

                list.get(idx)
                    .cloned()
                    .ok_or_else(|| format!("Index {} out of bounds", i))
            }
            (Value::String(s), Value::Int(i)) => {
                let idx = if *i < 0 {
                    (s.len() as i64 + i) as usize
                } else {
                    *i as usize
                };

                s.chars()
                    .nth(idx)
                    .map(|c| Value::Char(c))
                    .ok_or_else(|| format!("Index {} out of bounds", i))
            }
            _ => Err(format!("Cannot index {:?} with {:?}", obj_val, idx_val)),
        }
    }

    /// Evaluate a function call
    fn evaluate_call(&mut self, func: &Expr, args: &[Expr]) -> Result<Value, String> {
        // Simple built-in function handling
        if let ExprKind::Identifier(name) = &func.kind {
            return self.evaluate_builtin(name, args);
        }

        Err("Function calls not yet fully implemented".to_string())
    }

    /// Evaluate built-in functions
    fn evaluate_builtin(&mut self, name: &str, args: &[Expr]) -> Result<Value, String> {
        match name {
            "print" => self.builtin_print(args),
            "len" => self.builtin_len(args),
            "type" => self.builtin_type(args),
            "str" => self.builtin_str(args),
            "int" => self.builtin_int(args),
            "float" => self.builtin_float(args),
            "bool" => self.builtin_bool(args),
            _ => Err(format!("Unknown function: {}", name)),
        }
    }

    // Arithmetic operations (complexity: ≤10)

    fn add_values(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(format!("Cannot add {:?} and {:?}", left, right)),
        }
    }

    fn subtract_values(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
            _ => Err(format!("Cannot subtract {:?} from {:?}", right, left)),
        }
    }

    fn multiply_values(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
            _ => Err(format!("Cannot multiply {:?} and {:?}", left, right)),
        }
    }

    fn divide_values(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (_, Value::Int(0)) | (_, Value::Float(f)) if *f == 0.0 => {
                Err("Division by zero".to_string())
            }
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a / *b as f64)),
            _ => Err(format!("Cannot divide {:?} by {:?}", left, right)),
        }
    }

    fn modulo_values(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::Int(a % b))
                }
            }
            _ => Err(format!("Cannot compute {:?} modulo {:?}", left, right)),
        }
    }

    // Comparison operations (complexity: ≤10)

    fn less_than(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            _ => Err(format!("Cannot compare {:?} < {:?}", left, right)),
        }
    }

    fn less_equal(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            _ => Err(format!("Cannot compare {:?} <= {:?}", left, right)),
        }
    }

    fn greater_than(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            _ => Err(format!("Cannot compare {:?} > {:?}", left, right)),
        }
    }

    fn greater_equal(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
            _ => Err(format!("Cannot compare {:?} >= {:?}", left, right)),
        }
    }

    // Logical operations (complexity: ≤10)

    fn logical_and(&self, left: &Value, right: &Value) -> Result<Value, String> {
        Ok(Value::Bool(left.is_truthy() && right.is_truthy()))
    }

    fn logical_or(&self, left: &Value, right: &Value) -> Result<Value, String> {
        Ok(Value::Bool(left.is_truthy() || right.is_truthy()))
    }

    fn logical_not(&self, val: &Value) -> Result<Value, String> {
        Ok(Value::Bool(!val.is_truthy()))
    }

    fn negate_value(&self, val: &Value) -> Result<Value, String> {
        match val {
            Value::Int(i) => Ok(Value::Int(-i)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(format!("Cannot negate {:?}", val)),
        }
    }

    // Built-in functions (complexity: ≤10)

    fn builtin_print(&mut self, args: &[Expr]) -> Result<Value, String> {
        for arg in args {
            let val = self.evaluate(arg)?;
            println!("{}", val);
        }
        Ok(Value::Unit)
    }

    fn builtin_len(&mut self, args: &[Expr]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("len() expects 1 argument, got {}", args.len()));
        }

        let val = self.evaluate(&args[0])?;
        match val {
            Value::String(s) => Ok(Value::Int(s.len() as i64)),
            Value::List(l) => Ok(Value::Int(l.len() as i64)),
            Value::Tuple(t) => Ok(Value::Int(t.len() as i64)),
            _ => Err(format!("len() not supported for {:?}", val)),
        }
    }

    fn builtin_type(&mut self, args: &[Expr]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("type() expects 1 argument, got {}", args.len()));
        }

        let val = self.evaluate(&args[0])?;
        Ok(Value::String(val.type_name().to_string()))
    }

    fn builtin_str(&mut self, args: &[Expr]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("str() expects 1 argument, got {}", args.len()));
        }

        let val = self.evaluate(&args[0])?;
        Ok(Value::String(format!("{}", val)))
    }

    fn builtin_int(&mut self, args: &[Expr]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("int() expects 1 argument, got {}", args.len()));
        }

        let val = self.evaluate(&args[0])?;
        match val {
            Value::Int(i) => Ok(Value::Int(i)),
            Value::Float(f) => Ok(Value::Int(f as i64)),
            Value::String(s) => {
                s.parse::<i64>()
                    .map(Value::Int)
                    .map_err(|_| format!("Cannot convert '{}' to int", s))
            }
            Value::Bool(b) => Ok(Value::Int(if b { 1 } else { 0 })),
            _ => Err(format!("Cannot convert {:?} to int", val)),
        }
    }

    fn builtin_float(&mut self, args: &[Expr]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("float() expects 1 argument, got {}", args.len()));
        }

        let val = self.evaluate(&args[0])?;
        match val {
            Value::Float(f) => Ok(Value::Float(f)),
            Value::Int(i) => Ok(Value::Float(i as f64)),
            Value::String(s) => {
                s.parse::<f64>()
                    .map(Value::Float)
                    .map_err(|_| format!("Cannot convert '{}' to float", s))
            }
            _ => Err(format!("Cannot convert {:?} to float", val)),
        }
    }

    fn builtin_bool(&mut self, args: &[Expr]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("bool() expects 1 argument, got {}", args.len()));
        }

        let val = self.evaluate(&args[0])?;
        Ok(Value::Bool(val.is_truthy()))
    }
}