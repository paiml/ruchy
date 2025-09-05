//! Main expression evaluator
//! Extracted from interpreter.rs for modularity (complexity: ≤10 per function)

use super::value::Value;
use super::error::{InterpreterError, InterpreterResult};
use super::cache::{InlineCache, TypeId};
use super::type_feedback::TypeFeedback;
use super::gc::ConservativeGC;
use super::builtin;
use crate::frontend::ast::{Expr, ExprKind, Stmt, StmtKind, BinaryOp, UnaryOp, Literal};
use std::collections::HashMap;
use std::rc::Rc;

/// Main interpreter evaluator
pub struct Evaluator {
    /// Variable bindings
    locals: HashMap<String, Value>,
    /// Global bindings
    globals: HashMap<String, Value>,
    /// Built-in functions
    builtins: HashMap<String, builtin::BuiltinFunction>,
    /// Call stack depth
    call_depth: usize,
    /// Maximum call depth
    max_call_depth: usize,
    /// Inline cache for optimization
    cache: InlineCache,
    /// Type feedback for JIT
    type_feedback: TypeFeedback,
    /// Garbage collector
    gc: ConservativeGC,
    /// Evaluation counter for cache locations
    eval_counter: usize,
}

impl Evaluator {
    /// Create new evaluator
    pub fn new() -> Self {
        let mut evaluator = Self {
            locals: HashMap::new(),
            globals: HashMap::new(),
            builtins: HashMap::new(),
            call_depth: 0,
            max_call_depth: 1000,
            cache: InlineCache::new(),
            type_feedback: TypeFeedback::new(),
            gc: ConservativeGC::new(),
            eval_counter: 0,
        };

        // Register built-in functions
        for (name, func) in builtin::get_builtins() {
            evaluator.builtins.insert(name.to_string(), func);
        }

        evaluator
    }

    /// Evaluate an expression
    pub fn eval(&mut self, expr: &Expr) -> InterpreterResult<Value> {
        self.check_call_depth()?;
        self.call_depth += 1;
        self.eval_counter += 1;
        
        let result = self.eval_expr(expr);
        
        self.call_depth -= 1;
        result
    }

    /// Evaluate a statement
    pub fn eval_stmt(&mut self, stmt: &Stmt) -> InterpreterResult<Option<Value>> {
        match &stmt.kind {
            StmtKind::Expr(expr) => {
                let val = self.eval(expr)?;
                Ok(Some(val))
            }
            StmtKind::Let { pattern, value, .. } => {
                let val = self.eval(value)?;
                self.bind_pattern(pattern, val)?;
                Ok(None)
            }
            StmtKind::Return(expr) => {
                let val = if let Some(e) = expr {
                    self.eval(e)?
                } else {
                    Value::Nil
                };
                Ok(Some(val))
            }
            _ => Err(InterpreterError::runtime("Statement not implemented")),
        }
    }

    /// Main expression evaluation
    fn eval_expr(&mut self, expr: &Expr) -> InterpreterResult<Value> {
        let location = self.eval_counter;
        
        let result = match &expr.kind {
            ExprKind::Literal(lit) => self.eval_literal(lit),
            ExprKind::Identifier(name) => self.eval_identifier(name),
            ExprKind::Binary { op, left, right } => self.eval_binary(op, left, right, location),
            ExprKind::Unary { op, operand } => self.eval_unary(op, operand),
            ExprKind::If { condition, then_expr, else_expr } => {
                self.eval_if(condition, then_expr, else_expr.as_deref())
            }
            ExprKind::List(exprs) => self.eval_list(exprs),
            ExprKind::Tuple(exprs) => self.eval_tuple(exprs),
            ExprKind::Index { object, index } => self.eval_index(object, index),
            ExprKind::Call { func, args } => self.eval_call(func, args),
            ExprKind::Lambda { params, body } => self.eval_lambda(params, body),
            ExprKind::Block(stmts) => self.eval_block(stmts),
            _ => Err(InterpreterError::runtime("Expression not implemented")),
        };

        // Record type feedback if successful
        if let Ok(ref val) = result {
            let type_id = TypeId::from_value(val);
            self.cache.insert(location, type_id);
        }

        result
    }

    /// Check call depth
    fn check_call_depth(&self) -> InterpreterResult<()> {
        if self.call_depth >= self.max_call_depth {
            Err(InterpreterError::StackOverflow)
        } else {
            Ok(())
        }
    }

    /// Evaluate literal
    fn eval_literal(&self, lit: &Literal) -> InterpreterResult<Value> {
        match lit {
            Literal::Integer(i) => Ok(Value::Integer(*i)),
            Literal::Float(f) => Ok(Value::Float(*f)),
            Literal::String(s) => Ok(Value::from_string(s.clone())),
            Literal::Bool(b) => Ok(Value::Bool(*b)),
            Literal::None => Ok(Value::Nil),
            _ => Err(InterpreterError::runtime("Literal not implemented")),
        }
    }

    /// Evaluate identifier
    fn eval_identifier(&self, name: &str) -> InterpreterResult<Value> {
        // Check locals first
        if let Some(val) = self.locals.get(name) {
            return Ok(val.clone());
        }

        // Then globals
        if let Some(val) = self.globals.get(name) {
            return Ok(val.clone());
        }

        Err(InterpreterError::undefined_variable(name))
    }

    /// Evaluate binary operation
    fn eval_binary(
        &mut self,
        op: &BinaryOp,
        left: &Expr,
        right: &Expr,
        location: usize,
    ) -> InterpreterResult<Value> {
        let left_val = self.eval(left)?;
        let right_val = self.eval(right)?;

        // Record type feedback
        let left_type = TypeId::from_value(&left_val);
        let right_type = TypeId::from_value(&right_val);
        self.type_feedback.record_binary_op(location, left_type, right_type);

        match op {
            BinaryOp::Add => left_val.add(&right_val),
            BinaryOp::Sub => left_val.subtract(&right_val),
            BinaryOp::Mul => left_val.multiply(&right_val),
            BinaryOp::Div => left_val.divide(&right_val),
            BinaryOp::Mod => left_val.modulo(&right_val),
            BinaryOp::Pow => left_val.power(&right_val),
            BinaryOp::Eq => Ok(Value::Bool(left_val.equals(&right_val))),
            BinaryOp::Ne => Ok(Value::Bool(!left_val.equals(&right_val))),
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                self.eval_comparison(op, &left_val, &right_val)
            }
            BinaryOp::And => Ok(Value::Bool(left_val.is_truthy() && right_val.is_truthy())),
            BinaryOp::Or => Ok(Value::Bool(left_val.is_truthy() || right_val.is_truthy())),
            _ => Err(InterpreterError::runtime("Binary operator not implemented")),
        }.map_err(InterpreterError::runtime)
    }

    /// Evaluate comparison
    fn eval_comparison(&self, op: &BinaryOp, left: &Value, right: &Value) -> InterpreterResult<Value> {
        match left.compare(right) {
            Some(ord) => {
                let result = match op {
                    BinaryOp::Lt => ord == std::cmp::Ordering::Less,
                    BinaryOp::Le => ord != std::cmp::Ordering::Greater,
                    BinaryOp::Gt => ord == std::cmp::Ordering::Greater,
                    BinaryOp::Ge => ord != std::cmp::Ordering::Less,
                    _ => return Err(InterpreterError::runtime("Invalid comparison operator")),
                };
                Ok(Value::Bool(result))
            }
            None => Err(InterpreterError::type_mismatch(
                "comparable types",
                format!("{} and {}", left.type_name(), right.type_name()),
            )),
        }
    }

    /// Evaluate unary operation
    fn eval_unary(&mut self, op: &UnaryOp, operand: &Expr) -> InterpreterResult<Value> {
        let val = self.eval(operand)?;

        match op {
            UnaryOp::Not => Ok(Value::Bool(!val.is_truthy())),
            UnaryOp::Neg => match val {
                Value::Integer(i) => Ok(Value::Integer(-i)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Err(InterpreterError::type_mismatch("number", val.type_name())),
            },
            _ => Err(InterpreterError::runtime("Unary operator not implemented")),
        }
    }

    /// Evaluate if expression
    fn eval_if(
        &mut self,
        condition: &Expr,
        then_expr: &Expr,
        else_expr: Option<&Expr>,
    ) -> InterpreterResult<Value> {
        let cond_val = self.eval(condition)?;

        if cond_val.is_truthy() {
            self.eval(then_expr)
        } else if let Some(else_e) = else_expr {
            self.eval(else_e)
        } else {
            Ok(Value::Nil)
        }
    }

    /// Evaluate list literal
    fn eval_list(&mut self, exprs: &[Expr]) -> InterpreterResult<Value> {
        let mut values = Vec::new();
        for expr in exprs {
            values.push(self.eval(expr)?);
        }
        Ok(Value::from_array(values))
    }

    /// Evaluate tuple literal
    fn eval_tuple(&mut self, exprs: &[Expr]) -> InterpreterResult<Value> {
        let mut values = Vec::new();
        for expr in exprs {
            values.push(self.eval(expr)?);
        }
        Ok(Value::from_tuple(values))
    }

    /// Evaluate index operation
    fn eval_index(&mut self, object: &Expr, index: &Expr) -> InterpreterResult<Value> {
        let obj_val = self.eval(object)?;
        let idx_val = self.eval(index)?;

        match (&obj_val, &idx_val) {
            (Value::Array(arr), Value::Integer(i)) => {
                let idx = if *i < 0 {
                    (arr.len() as i64 + i) as usize
                } else {
                    *i as usize
                };

                arr.get(idx)
                    .cloned()
                    .ok_or_else(|| InterpreterError::index_out_of_bounds(*i, arr.len()))
            }
            _ => Err(InterpreterError::invalid_operation(
                format!("Cannot index {} with {}", obj_val.type_name(), idx_val.type_name())
            )),
        }
    }

    /// Evaluate function call
    fn eval_call(&mut self, func: &Expr, args: &[Expr]) -> InterpreterResult<Value> {
        // Check for built-in functions
        if let ExprKind::Identifier(name) = &func.kind {
            if let Some(builtin_fn) = self.builtins.get(name) {
                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(self.eval(arg)?);
                }
                return builtin_fn(&arg_vals);
            }
        }

        // Regular function call
        let func_val = self.eval(func)?;
        let mut arg_vals = Vec::new();
        for arg in args {
            arg_vals.push(self.eval(arg)?);
        }

        self.apply_function(func_val, arg_vals)
    }

    /// Apply function with arguments
    fn apply_function(&mut self, func: Value, args: Vec<Value>) -> InterpreterResult<Value> {
        match func {
            Value::Closure { params, body, env } => {
                if params.len() != args.len() {
                    return Err(InterpreterError::argument_count_mismatch(
                        params.len(),
                        args.len(),
                    ));
                }

                // Save current locals
                let saved_locals = self.locals.clone();

                // Set up new environment
                self.locals = (*env).clone();
                for (param, arg) in params.iter().zip(args.iter()) {
                    self.locals.insert(param.clone(), arg.clone());
                }

                // Evaluate body
                let result = self.eval(&body);

                // Restore locals
                self.locals = saved_locals;

                result
            }
            _ => Err(InterpreterError::type_mismatch("function", func.type_name())),
        }
    }

    /// Evaluate lambda expression
    fn eval_lambda(&self, params: &[String], body: &Expr) -> InterpreterResult<Value> {
        Ok(Value::Closure {
            params: params.to_vec(),
            body: Rc::new(body.clone()),
            env: Rc::new(self.locals.clone()),
        })
    }

    /// Evaluate block
    fn eval_block(&mut self, stmts: &[Stmt]) -> InterpreterResult<Value> {
        let mut last_val = Value::Nil;

        for stmt in stmts {
            if let Some(val) = self.eval_stmt(stmt)? {
                last_val = val;
            }
        }

        Ok(last_val)
    }

    /// Bind pattern to value
    fn bind_pattern(&mut self, pattern: &crate::frontend::ast::Pattern, value: Value) -> InterpreterResult<()> {
        use crate::frontend::ast::Pattern;

        match pattern {
            Pattern::Identifier(name) => {
                self.locals.insert(name.clone(), value);
                Ok(())
            }
            Pattern::Wildcard => Ok(()),
            _ => Err(InterpreterError::runtime("Pattern not implemented")),
        }
    }

    /// Set global variable
    pub fn set_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }

    /// Get global variable
    pub fn get_global(&self, name: &str) -> Option<&Value> {
        self.globals.get(name)
    }

    /// Clear all variables
    pub fn clear(&mut self) {
        self.locals.clear();
        self.globals.clear();
        self.cache.clear();
        self.type_feedback.clear();
        self.gc.clear();
        self.eval_counter = 0;
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}