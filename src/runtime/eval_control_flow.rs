//! Control flow expression evaluation module
//!
//! This module handles evaluation of control flow expressions (if, while, for, match, etc).
//! Extracted from the monolithic interpreter.rs to improve maintainability.
//! Complexity: <10 per function (Toyota Way compliant)

use crate::frontend::ast::{Expr, MatchArm, Pattern};
use crate::runtime::{InterpreterError, Value};

/// Control flow evaluation context
pub struct ControlFlowEvaluator<'a> {
    eval_expr: &'a mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>,
    env: &'a mut dyn EnvironmentOps,
}

/// Environment operations required for control flow
pub trait EnvironmentOps {
    fn push_scope(&mut self);
    fn pop_scope(&mut self);
    fn bind_pattern(&mut self, pattern: &Pattern, value: &Value) -> Result<bool, InterpreterError>;
    fn set_variable(&mut self, name: &str, value: Value) -> Result<(), InterpreterError>;
}

impl<'a> ControlFlowEvaluator<'a> {
    pub fn new(
        eval_expr: &'a mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>,
        env: &'a mut dyn EnvironmentOps,
    ) -> Self {
        Self { eval_expr, env }
    }

    /// Evaluate if expression
    ///
    /// # Complexity
    /// Cyclomatic complexity: 4 (within limit of 10)
    pub fn eval_if(
        &mut self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: &Option<Box<Expr>>,
    ) -> Result<Value, InterpreterError> {
        let cond_value = (self.eval_expr)(condition)?;

        if cond_value.is_truthy() {
            (self.eval_expr)(then_branch)
        } else if let Some(else_expr) = else_branch {
            (self.eval_expr)(else_expr)
        } else {
            Ok(Value::nil())
        }
    }

    /// Evaluate while loop
    ///
    /// # Complexity
    /// Cyclomatic complexity: 6 (within limit of 10)
    pub fn eval_while(&mut self, condition: &Expr, body: &Expr) -> Result<Value, InterpreterError> {
        let mut last_value = Value::nil();

        loop {
            let cond_value = (self.eval_expr)(condition)?;
            if !cond_value.is_truthy() {
                break;
            }

            match (self.eval_expr)(body) {
                Ok(val) => last_value = val,
                Err(InterpreterError::Break(None, val)) => {
                    last_value = val;
                    break;
                }
                Err(InterpreterError::Continue(_)) => {}
                Err(e) => return Err(e),
            }
        }

        Ok(last_value)
    }

    /// Evaluate for loop
    ///
    /// # Complexity
    /// Cyclomatic complexity: 8 (within limit of 10)
    pub fn eval_for(
        &mut self,
        variable: &str,
        iterable: &Expr,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        let iter_value = (self.eval_expr)(iterable)?;
        let items = self.get_iterable_items(iter_value)?;
        let mut last_value = Value::nil();

        self.env.push_scope();

        for item in items {
            self.env.set_variable(variable, item)?;

            match (self.eval_expr)(body) {
                Ok(val) => last_value = val,
                Err(InterpreterError::Break(None, val)) => {
                    last_value = val;
                    break;
                }
                Err(InterpreterError::Continue(_)) => {}
                Err(e) => {
                    self.env.pop_scope();
                    return Err(e);
                }
            }
        }

        self.env.pop_scope();
        Ok(last_value)
    }

    /// Get items from an iterable value
    ///
    /// # Complexity
    /// Cyclomatic complexity: 5 (within limit of 10)
    fn get_iterable_items(&self, value: Value) -> Result<Vec<Value>, InterpreterError> {
        match value {
            Value::Array(arr) => Ok(arr.to_vec()),
            Value::String(s) => Ok(s
                .chars()
                .map(|c| Value::from_string(c.to_string()))
                .collect()),
            Value::Range {
                start,
                end,
                inclusive,
            } => match (start.as_ref(), end.as_ref()) {
                (Value::Integer(s), Value::Integer(e)) => {
                    let range_values: Vec<Value> = if inclusive {
                        (*s..=*e).map(Value::from_i64).collect()
                    } else {
                        (*s..*e).map(Value::from_i64).collect()
                    };
                    Ok(range_values)
                }
                _ => Err(InterpreterError::RuntimeError(
                    "Range bounds must be integers".to_string(),
                )),
            },
            _ => Err(InterpreterError::RuntimeError(format!(
                "Cannot iterate over {}",
                value.type_name()
            ))),
        }
    }

    /// Evaluate match expression
    ///
    /// # Complexity
    /// Cyclomatic complexity: 7 (within limit of 10)
    pub fn eval_match(
        &mut self,
        expr: &Expr,
        arms: &[MatchArm],
    ) -> Result<Value, InterpreterError> {
        let value = (self.eval_expr)(expr)?;

        for arm in arms {
            self.env.push_scope();

            let matched = self.env.bind_pattern(&arm.pattern, &value)?;

            if matched {
                // Check guard if present
                let guard_passed = if let Some(guard) = &arm.guard {
                    let guard_val = (self.eval_expr)(guard)?;
                    guard_val.is_truthy()
                } else {
                    true
                };

                if guard_passed {
                    let result = (self.eval_expr)(&arm.body);
                    self.env.pop_scope();
                    return result;
                }
            }

            self.env.pop_scope();
        }

        Err(InterpreterError::RuntimeError(
            "No match arm matched the value".to_string(),
        ))
    }

    /// Evaluate break expression
    ///
    /// # Complexity
    /// Cyclomatic complexity: 2 (within limit of 10)
    pub fn eval_break(&mut self, value: &Option<Box<Expr>>) -> Result<Value, InterpreterError> {
        let val = if let Some(expr) = value {
            (self.eval_expr)(expr)?
        } else {
            Value::nil()
        };
        Err(InterpreterError::Break(None, val))
    }

    /// Evaluate continue expression
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1 (within limit of 10)
    pub fn eval_continue(&mut self) -> Result<Value, InterpreterError> {
        Err(InterpreterError::Continue(None))
    }

    /// Evaluate return expression
    ///
    /// # Complexity
    /// Cyclomatic complexity: 2 (within limit of 10)
    pub fn eval_return(&mut self, value: &Option<Box<Expr>>) -> Result<Value, InterpreterError> {
        let val = if let Some(expr) = value {
            (self.eval_expr)(expr)?
        } else {
            Value::nil()
        };
        Err(InterpreterError::Return(val))
    }

    /// Evaluate let binding
    ///
    /// # Complexity
    /// Cyclomatic complexity: 3 (within limit of 10)
    pub fn eval_let(
        &mut self,
        pattern: &Pattern,
        value: &Expr,
        body: &Option<Box<Expr>>,
    ) -> Result<Value, InterpreterError> {
        let val = (self.eval_expr)(value)?;

        self.env.push_scope();
        self.env.bind_pattern(pattern, &val)?;

        let result = if let Some(body_expr) = body {
            (self.eval_expr)(body_expr)
        } else {
            Ok(Value::nil())
        };

        self.env.pop_scope();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span};

    pub struct MockEnv {
        scopes: Vec<std::collections::HashMap<String, Value>>,
    }

    impl MockEnv {
        pub fn new() -> Self {
            Self {
                scopes: vec![std::collections::HashMap::new()],
            }
        }
    }

    impl EnvironmentOps for MockEnv {
        fn push_scope(&mut self) {
            self.scopes.push(std::collections::HashMap::new());
        }

        fn pop_scope(&mut self) {
            if self.scopes.len() > 1 {
                self.scopes.pop();
            }
        }

        fn bind_pattern(
            &mut self,
            pattern: &Pattern,
            value: &Value,
        ) -> Result<bool, InterpreterError> {
            match pattern {
                Pattern::Identifier(name) => {
                    self.set_variable(name, value.clone())?;
                    Ok(true)
                }
                Pattern::Wildcard => Ok(true),
                Pattern::Literal(lit) => {
                    // Simple literal matching
                    Ok(match (lit, value) {
                        (Literal::Integer(a), Value::Integer(b)) => a == b,
                        (Literal::Bool(a), Value::Bool(b)) => a == b,
                        _ => false,
                    })
                }
                _ => Ok(false),
            }
        }

        fn set_variable(&mut self, name: &str, value: Value) -> Result<(), InterpreterError> {
            if let Some(scope) = self.scopes.last_mut() {
                scope.insert(name.to_string(), value);
                Ok(())
            } else {
                Err(InterpreterError::RuntimeError(
                    "No scope available".to_string(),
                ))
            }
        }
    }

    #[test]
    fn test_if_true_branch() {
        let mut env = MockEnv::new();
        let mut eval_expr = |expr: &Expr| -> Result<Value, InterpreterError> {
            match &expr.kind {
                ExprKind::Literal(Literal::Bool(b)) => Ok(Value::Bool(*b)),
                ExprKind::Literal(Literal::Integer(i)) => Ok(Value::Integer(*i)),
                _ => Ok(Value::nil()),
            }
        };
        let mut evaluator = ControlFlowEvaluator::new(&mut eval_expr, &mut env);

        let condition = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::new(0, 0));
        let then_branch = Expr::new(ExprKind::Literal(Literal::Integer(42)), Span::new(0, 0));
        let else_branch = Some(Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(0)),
            Span::new(0, 0),
        )));

        let result = evaluator
            .eval_if(&condition, &then_branch, &else_branch)
            .unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_if_false_branch() {
        let mut env = MockEnv::new();
        let mut eval_expr = |expr: &Expr| -> Result<Value, InterpreterError> {
            match &expr.kind {
                ExprKind::Literal(Literal::Bool(b)) => Ok(Value::Bool(*b)),
                ExprKind::Literal(Literal::Integer(i)) => Ok(Value::Integer(*i)),
                _ => Ok(Value::nil()),
            }
        };
        let mut evaluator = ControlFlowEvaluator::new(&mut eval_expr, &mut env);

        let condition = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::new(0, 0));
        let then_branch = Expr::new(ExprKind::Literal(Literal::Integer(42)), Span::new(0, 0));
        let else_branch = Some(Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(99)),
            Span::new(0, 0),
        )));

        let result = evaluator
            .eval_if(&condition, &then_branch, &else_branch)
            .unwrap();
        assert_eq!(result, Value::Integer(99));
    }

    #[test]
    fn test_while_loop_iterations() {
        let mut env = MockEnv::new();
        let mut counter = 0;
        let mut eval_expr = |expr: &Expr| -> Result<Value, InterpreterError> {
            match &expr.kind {
                ExprKind::Literal(Literal::Bool(_)) => {
                    // Condition: loop 3 times
                    counter += 1;
                    Ok(Value::Bool(counter <= 3))
                }
                ExprKind::Literal(Literal::Integer(i)) => Ok(Value::Integer(*i)),
                _ => Ok(Value::Integer(i64::from(counter))),
            }
        };
        let mut evaluator = ControlFlowEvaluator::new(&mut eval_expr, &mut env);

        let condition = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::new(0, 0));
        let body = Expr::new(ExprKind::Literal(Literal::Integer(1)), Span::new(0, 0));

        let _result = evaluator.eval_while(&condition, &body).unwrap();
        assert_eq!(counter, 4); // Checks condition 4 times (3 true, 1 false)
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span};
    use crate::runtime::{InterpreterError, Value};
    use proptest::prelude::*;

    // Mock environment for testing
    pub struct MockEnv {
        scopes: Vec<std::collections::HashMap<String, Value>>,
    }

    impl MockEnv {
        pub fn new() -> Self {
            Self {
                scopes: vec![std::collections::HashMap::new()],
            }
        }
    }

    impl super::EnvironmentOps for MockEnv {
        fn push_scope(&mut self) {
            self.scopes.push(std::collections::HashMap::new());
        }

        fn pop_scope(&mut self) {
            self.scopes.pop();
        }

        fn bind_pattern(
            &mut self,
            _pattern: &super::Pattern,
            _value: &Value,
        ) -> Result<bool, InterpreterError> {
            Ok(true)
        }

        fn set_variable(&mut self, name: &str, value: Value) -> Result<(), InterpreterError> {
            if let Some(scope) = self.scopes.last_mut() {
                scope.insert(name.to_string(), value);
            }
            Ok(())
        }
    }

    proptest! {
        #[test]
        fn test_if_expression_deterministic(condition: bool, then_val: i64, else_val: i64) {
            let expected = if condition { then_val } else { else_val };

            let mut env = MockEnv::new();
            let mut eval_expr = move |expr: &Expr| -> Result<Value, InterpreterError> {
                match &expr.kind {
                    ExprKind::Literal(Literal::Bool(_)) => Ok(Value::Bool(condition)),
                    ExprKind::Literal(Literal::Integer(i)) => Ok(Value::Integer(*i)),
                    _ => Ok(Value::nil()),
                }
            };
            let mut evaluator = ControlFlowEvaluator::new(&mut eval_expr, &mut env);

            let cond_expr = Expr::new(ExprKind::Literal(Literal::Bool(condition)), Span::new(0, 0));
            let then_expr = Expr::new(ExprKind::Literal(Literal::Integer(then_val)), Span::new(0, 0));
            let else_expr = Some(Box::new(Expr::new(ExprKind::Literal(Literal::Integer(else_val)), Span::new(0, 0))));

            let result = evaluator.eval_if(&cond_expr, &then_expr, &else_expr).unwrap();
            prop_assert_eq!(result, Value::Integer(expected));
        }
    }
}
