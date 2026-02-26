//! Control flow implementation module
//!
//! This module handles loops (for, while, loop), match expressions,
//! pattern matching, and assignment operations.
//! Extracted from interpreter.rs for maintainability.

#![allow(clippy::unused_self)]
#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::expect_used)]
#![allow(clippy::cast_possible_truncation)]

use crate::frontend::ast::{BinaryOp as AstBinaryOp, Expr, ExprKind, Literal, MatchArm, Pattern};
use crate::runtime::interpreter::{Interpreter, LoopControlOrError};
use crate::runtime::{InterpreterError, Value};
use std::sync::Arc;

impl Interpreter {
    /// Evaluate a for loop
    pub(crate) fn eval_for_loop(
        &mut self,
        label: Option<&String>,
        var: &str,
        _pattern: Option<&Pattern>,
        iter: &Expr,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        let iter_value = self.eval_expr(iter)?;

        match iter_value {
            Value::Array(ref arr) => self.eval_for_array_iteration(label, var, arr, body),
            Value::Range {
                ref start,
                ref end,
                inclusive,
            } => self.eval_for_range_iteration(label, var, start, end, inclusive, body),
            _ => Err(InterpreterError::TypeError(
                "For loop requires an iterable".to_string(),
            )),
        }
    }

    /// Evaluate for loop iteration over an array
    /// Complexity: ≤8
    pub(crate) fn eval_for_array_iteration(
        &mut self,
        label: Option<&String>,
        loop_var: &str,
        arr: &[Value],
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        let mut last_value = Value::nil();

        for item in arr {
            self.set_variable(loop_var, item.clone());
            match self.eval_loop_body_with_control_flow(body) {
                Ok(value) => last_value = value,
                Err(LoopControlOrError::Break(break_label, break_val)) => {
                    // If break has no label or matches this loop's label, break here
                    if break_label.is_none() || break_label.as_deref() == label.map(String::as_str)
                    {
                        return Ok(break_val);
                    }
                    // Otherwise, propagate to outer loop
                    return Err(InterpreterError::Break(break_label, break_val));
                }
                Err(LoopControlOrError::Continue(continue_label)) => {
                    // If continue has no label or matches this loop's label, continue here
                    if continue_label.is_none()
                        || continue_label.as_deref() == label.map(String::as_str)
                    {
                        continue;
                    }
                    // Otherwise, propagate to outer loop
                    return Err(InterpreterError::Continue(continue_label));
                }
                Err(LoopControlOrError::Return(return_val)) => {
                    return Err(InterpreterError::Return(return_val))
                }
                Err(LoopControlOrError::Error(e)) => return Err(e),
            }
        }

        Ok(last_value)
    }

    /// Evaluate for loop iteration over a range
    /// Complexity: ≤9
    pub(crate) fn eval_for_range_iteration(
        &mut self,
        label: Option<&String>,
        loop_var: &str,
        start: &Value,
        end: &Value,
        inclusive: bool,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        let (start_val, end_val) = self.extract_range_bounds(start, end)?;
        let mut last_value = Value::nil();

        for i in self.create_range_iterator(start_val, end_val, inclusive) {
            self.set_variable(loop_var, Value::Integer(i));
            match self.eval_loop_body_with_control_flow(body) {
                Ok(value) => last_value = value,
                Err(LoopControlOrError::Break(break_label, break_val)) => {
                    if break_label.is_none() || break_label.as_deref() == label.map(String::as_str)
                    {
                        return Ok(break_val);
                    }
                    return Err(InterpreterError::Break(break_label, break_val));
                }
                Err(LoopControlOrError::Continue(continue_label)) => {
                    if continue_label.is_none()
                        || continue_label.as_deref() == label.map(String::as_str)
                    {
                        continue;
                    }
                    return Err(InterpreterError::Continue(continue_label));
                }
                Err(LoopControlOrError::Return(return_val)) => {
                    return Err(InterpreterError::Return(return_val))
                }
                Err(LoopControlOrError::Error(e)) => return Err(e),
            }
        }

        Ok(last_value)
    }

    /// Extract integer bounds from range values
    /// Complexity: ≤3
    pub(crate) fn extract_range_bounds(
        &self,
        start: &Value,
        end: &Value,
    ) -> Result<(i64, i64), InterpreterError> {
        match (start, end) {
            (Value::Integer(s), Value::Integer(e)) => Ok((*s, *e)),
            _ => Err(InterpreterError::TypeError(
                "Range bounds must be integers".to_string(),
            )),
        }
    }

    /// Create range iterator based on inclusive flag
    /// Complexity: ≤2
    pub(crate) fn create_range_iterator(
        &self,
        start: i64,
        end: i64,
        inclusive: bool,
    ) -> Box<dyn Iterator<Item = i64>> {
        if inclusive {
            Box::new(start..=end)
        } else {
            Box::new(start..end)
        }
    }

    /// Evaluate loop body with control flow handling
    /// Complexity: ≤5
    pub(crate) fn eval_loop_body_with_control_flow(
        &mut self,
        body: &Expr,
    ) -> Result<Value, LoopControlOrError> {
        match self.eval_expr(body) {
            Ok(value) => Ok(value),
            Err(InterpreterError::Break(label, val)) => Err(LoopControlOrError::Break(label, val)),
            Err(InterpreterError::Continue(label)) => Err(LoopControlOrError::Continue(label)),
            Err(InterpreterError::Return(val)) => Err(LoopControlOrError::Return(val)),
            Err(e) => Err(LoopControlOrError::Error(e)),
        }
    }

    /// Evaluate a while loop
    pub(crate) fn eval_while_loop(
        &mut self,
        label: Option<&String>,
        condition: &Expr,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        let mut last_value = Value::Nil;
        loop {
            let cond_value = self.eval_expr(condition)?;
            if !matches!(cond_value, Value::Bool(true)) && cond_value != Value::Integer(1) {
                break;
            }

            match self.eval_loop_body_with_control_flow(body) {
                Ok(value) => last_value = value,
                Err(LoopControlOrError::Break(break_label, break_val)) => {
                    if break_label.is_none() || break_label.as_deref() == label.map(String::as_str)
                    {
                        return Ok(break_val);
                    }
                    return Err(InterpreterError::Break(break_label, break_val));
                }
                Err(LoopControlOrError::Continue(continue_label)) => {
                    if continue_label.is_none()
                        || continue_label.as_deref() == label.map(String::as_str)
                    {
                        continue;
                    }
                    return Err(InterpreterError::Continue(continue_label));
                }
                Err(LoopControlOrError::Return(return_val)) => {
                    return Err(InterpreterError::Return(return_val))
                }
                Err(LoopControlOrError::Error(e)) => return Err(e),
            }
        }
        Ok(last_value)
    }

    /// Evaluate an infinite loop (loop { ... })
    pub(crate) fn eval_loop(
        &mut self,
        label: Option<&String>,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        loop {
            match self.eval_loop_body_with_control_flow(body) {
                Ok(_) => {}
                Err(LoopControlOrError::Break(break_label, break_val)) => {
                    if break_label.is_none() || break_label.as_deref() == label.map(String::as_str)
                    {
                        return Ok(break_val);
                    }
                    return Err(InterpreterError::Break(break_label, break_val));
                }
                Err(LoopControlOrError::Continue(continue_label)) => {
                    if continue_label.is_none()
                        || continue_label.as_deref() == label.map(String::as_str)
                    {
                        continue;
                    }
                    return Err(InterpreterError::Continue(continue_label));
                }
                Err(LoopControlOrError::Return(return_val)) => {
                    return Err(InterpreterError::Return(return_val))
                }
                Err(LoopControlOrError::Error(e)) => return Err(e),
            }
        }
    }

    /// Evaluate a match expression
    pub fn eval_match(
        &mut self,
        expr: &Expr,
        arms: &[MatchArm],
    ) -> Result<Value, InterpreterError> {
        let value = self.eval_expr(expr)?;

        for arm in arms {
            // First check if pattern matches
            if let Some(bindings) = self.try_pattern_match(&arm.pattern, &value)? {
                // Create new scope for pattern bindings
                self.push_scope();

                // Bind pattern variables
                for (name, val) in bindings {
                    self.env_set(name, val);
                }

                // Check guard condition if present
                let guard_passed = if let Some(guard) = &arm.guard {
                    match self.eval_expr(guard)? {
                        Value::Bool(true) => true,
                        Value::Bool(false) => false,
                        _ => {
                            self.pop_scope();
                            return Err(InterpreterError::RuntimeError(
                                "Guard condition must evaluate to a boolean".to_string(),
                            ));
                        }
                    }
                } else {
                    true // No guard means always pass
                };

                if guard_passed {
                    // Evaluate body with bindings in scope
                    let result = self.eval_expr(&arm.body);
                    self.pop_scope();
                    return result;
                }
                // Guard failed, restore scope and try next arm
                self.pop_scope();
            }
        }

        Err(InterpreterError::RuntimeError(
            "No match arm matched the value".to_string(),
        ))
    }

    /// Evaluate a let pattern expression (array/tuple destructuring)
    /// Extract names of identifiers marked as mutable in a pattern
    /// Complexity: 4 (within Toyota Way limits)
    pub(crate) fn extract_mut_names(pattern: &Pattern) -> std::collections::HashSet<String> {
        let mut mut_names = std::collections::HashSet::new();

        fn walk_pattern(
            p: &Pattern,
            mut_names: &mut std::collections::HashSet<String>,
            is_mut: bool,
        ) {
            match p {
                Pattern::Mut(inner) => walk_pattern(inner, mut_names, true),
                Pattern::Identifier(name) if is_mut => {
                    mut_names.insert(name.clone());
                }
                Pattern::Tuple(patterns) | Pattern::List(patterns) => {
                    for pat in patterns {
                        walk_pattern(pat, mut_names, is_mut);
                    }
                }
                Pattern::Struct { fields, .. } => {
                    for field in fields {
                        if let Some(ref pat) = field.pattern {
                            walk_pattern(pat, mut_names, is_mut);
                        }
                    }
                }
                Pattern::AtBinding { pattern, .. } => walk_pattern(pattern, mut_names, is_mut),
                _ => {}
            }
        }

        walk_pattern(pattern, &mut mut_names, false);
        mut_names
    }

    /// Evaluate let pattern with support for mut destructuring
    /// Complexity: 6 (within Toyota Way limits)
    pub(crate) fn eval_let_pattern(
        &mut self,
        pattern: &Pattern,
        value: &Expr,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        // Evaluate the right-hand side value
        let rhs_value = self.eval_expr(value)?;

        // Extract names marked as mutable in the pattern
        let mut_names = Self::extract_mut_names(pattern);

        // Try to match the pattern against the value
        if let Some(bindings) = self.try_pattern_match(pattern, &rhs_value)? {
            // Bind pattern variables, using mutable binding for names wrapped in Pattern::Mut
            for (name, val) in bindings {
                if mut_names.contains(&name) {
                    self.env_set_mut(name.clone(), val);
                } else {
                    self.env_set(name, val);
                }
            }

            // If body is unit (empty), return the value like REPL does
            // This makes `let [a, b] = [1, 2]` return [1, 2] instead of nil
            match &body.kind {
                ExprKind::Literal(Literal::Unit) => Ok(rhs_value),
                _ => self.eval_expr(body),
            }
        } else {
            Err(InterpreterError::RuntimeError(
                "Pattern did not match the value".to_string(),
            ))
        }
    }

    /// Evaluate an assignment
    /// Evaluates assignment expressions including field assignments.
    ///
    /// This method handles variable assignments (`x = value`) and field assignments (`obj.field = value`).
    /// For field assignments, it creates a new object with the updated field value.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::frontend::parser::Parser;
    /// use ruchy::runtime::interpreter::{Interpreter, Value};
    ///
    /// let mut interpreter = Interpreter::new();
    /// let code = r#"
    ///     class Point {
    ///         x: i32,
    ///         y: i32
    ///
    ///         new(x: i32, y: i32) {
    ///             self.x = x
    ///             self.y = y
    ///         }
    ///     }
    ///
    ///     fn main() {
    ///         let p = Point::new(10, 20)
    ///         p.x
    ///     }
    /// "#;
    ///
    /// let mut parser = Parser::new(code);
    /// let expr = parser.parse().expect("parse should succeed in doctest");
    /// interpreter.eval_expr(&expr).expect("eval_expr should succeed in doctest");
    /// let main_call = Parser::new("main()").parse().expect("parse should succeed in doctest");
    /// let result = interpreter.eval_expr(&main_call).expect("eval_expr should succeed in doctest");
    /// assert!(matches!(result, Value::Integer(10)));
    /// ```
    pub(crate) fn eval_assign(
        &mut self,
        target: &Expr,
        value: &Expr,
    ) -> Result<Value, InterpreterError> {
        let val = self.eval_expr(value)?;

        // Handle different assignment targets
        match &target.kind {
            ExprKind::Identifier(name) => {
                self.set_variable(name, val.clone());
                Ok(val)
            }
            ExprKind::FieldAccess { object, field } => {
                self.eval_field_assign(object, field, val)
            }
            // BUG-003: Support array index assignment (arr[i] = value)
            ExprKind::IndexAccess { object, index } => self.eval_index_assign(object, index, val),
            _ => Err(InterpreterError::RuntimeError(
                "Invalid assignment target".to_string(),
            )),
        }
    }

    /// BUG-003: Evaluate array/vector index assignment (arr[i] = value, matrix[i][j] = value)
    ///
    /// Handles both simple (arr[0] = 99) and nested (matrix[0][1] = 99) index assignment.
    /// Complexity: 8 (≤10 target)
    /// Assign a value into a 2D array: `arr[outer_idx][inner_idx] = val`.
    fn assign_nested_array(
        &mut self,
        arr_name: &str,
        outer_idx: usize,
        inner_idx_val: Value,
        val: Value,
    ) -> Result<Value, InterpreterError> {
        let inner_idx = match inner_idx_val {
            Value::Integer(i) => i as usize,
            _ => {
                return Err(InterpreterError::RuntimeError(
                    "Array index must be an integer".to_string(),
                ))
            }
        };

        let arr = self.lookup_variable(arr_name)?;
        let Value::Array(ref outer_vec) = arr else {
            return Err(InterpreterError::RuntimeError(
                "Cannot index non-array value".to_string(),
            ));
        };

        let mut new_outer = outer_vec.to_vec();
        if outer_idx >= new_outer.len() {
            return Err(InterpreterError::RuntimeError(format!(
                "Outer index {outer_idx} out of bounds"
            )));
        }

        let Value::Array(ref inner_vec) = new_outer[outer_idx] else {
            return Err(InterpreterError::RuntimeError(
                "Cannot index non-array value".to_string(),
            ));
        };

        let mut new_inner = inner_vec.to_vec();
        if inner_idx >= new_inner.len() {
            return Err(InterpreterError::RuntimeError(format!(
                "Inner index {inner_idx} out of bounds"
            )));
        }

        new_inner[inner_idx] = val.clone();
        new_outer[outer_idx] = Value::Array(Arc::from(new_inner));
        self.set_variable(arr_name, Value::Array(Arc::from(new_outer)));
        Ok(val)
    }

    /// Evaluate field assignment: `obj.field = value`.
    ///
    /// Handles Object, ObjectMut, Class, and Struct field updates.
    fn eval_field_assign(
        &mut self,
        object: &Expr,
        field: &str,
        val: Value,
    ) -> Result<Value, InterpreterError> {
        let ExprKind::Identifier(obj_name) = &object.kind else {
            return Err(InterpreterError::RuntimeError(
                "Complex field access not supported".to_string(),
            ));
        };
        let obj = self.lookup_variable(obj_name)?;

        match obj {
            Value::Object(ref map) => {
                let mut new_map = (**map).clone();
                new_map.insert(field.to_string(), val.clone());
                self.set_variable(obj_name, Value::Object(Arc::new(new_map)));
                Ok(val)
            }
            Value::ObjectMut(ref cell) => {
                cell.lock()
                    .expect("Mutex poisoned: object lock is corrupted")
                    .insert(field.to_string(), val.clone());
                Ok(val)
            }
            Value::Class { ref fields, .. } => {
                fields
                    .write()
                    .expect("RwLock poisoned: class fields lock is corrupted")
                    .insert(field.to_string(), val.clone());
                Ok(val)
            }
            Value::Struct {
                ref name,
                ref fields,
            } => {
                let mut new_fields = (**fields).clone();
                new_fields.insert(field.to_string(), val.clone());
                let new_struct = Value::Struct {
                    name: name.clone(),
                    fields: Arc::new(new_fields),
                };
                self.set_variable(obj_name, new_struct);
                Ok(val)
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Cannot access field '{field}' on non-object"
            ))),
        }
    }

    pub(crate) fn eval_index_assign(
        &mut self,
        object: &Expr,
        index: &Expr,
        val: Value,
    ) -> Result<Value, InterpreterError> {
        match &object.kind {
            ExprKind::Identifier(arr_name) => {
                // Simple case: arr[i] = value
                let idx_val = self.eval_expr(index)?;
                let idx = match idx_val {
                    Value::Integer(i) => i as usize,
                    _ => {
                        return Err(InterpreterError::RuntimeError(
                            "Array index must be an integer".to_string(),
                        ))
                    }
                };

                let arr = self.lookup_variable(arr_name)?;
                match arr {
                    Value::Array(ref vec) => {
                        let mut new_vec = vec.to_vec();
                        if idx < new_vec.len() {
                            new_vec[idx] = val.clone();
                            self.set_variable(arr_name, Value::Array(Arc::from(new_vec)));
                            Ok(val)
                        } else {
                            Err(InterpreterError::RuntimeError(format!(
                                "Index {} out of bounds for array of length {}",
                                idx,
                                new_vec.len()
                            )))
                        }
                    }
                    _ => Err(InterpreterError::RuntimeError(
                        "Cannot index non-array value".to_string(),
                    )),
                }
            }
            ExprKind::IndexAccess {
                object: nested_obj,
                index: nested_idx,
            } => {
                // Nested case: matrix[i][j] = value
                // Get the outer index first
                let outer_idx_val = self.eval_expr(nested_idx)?;
                let outer_idx = match outer_idx_val {
                    Value::Integer(i) => i as usize,
                    _ => {
                        return Err(InterpreterError::RuntimeError(
                            "Array index must be an integer".to_string(),
                        ))
                    }
                };

                // Get the root array name (only handle Identifier for now)
                if let ExprKind::Identifier(arr_name) = &nested_obj.kind {
                    let inner_idx_val = self.eval_expr(index)?;
                    self.assign_nested_array(arr_name, outer_idx, inner_idx_val, val)
                } else {
                    Err(InterpreterError::RuntimeError(
                        "Complex nested index assignment not yet supported".to_string(),
                    ))
                }
            }
            _ => Err(InterpreterError::RuntimeError(
                "Complex array assignment targets not yet supported".to_string(),
            )),
        }
    }

    /// Evaluate a compound assignment
    /// Complexity: 6
    pub(crate) fn eval_compound_assign(
        &mut self,
        target: &Expr,
        op: AstBinaryOp,
        value: &Expr,
    ) -> Result<Value, InterpreterError> {
        // Get current value
        let current = match &target.kind {
            ExprKind::Identifier(name) => self.lookup_variable(name)?,
            ExprKind::FieldAccess { object, field } => self.eval_field_access(object, field)?,
            _ => {
                return Err(InterpreterError::RuntimeError(
                    "Invalid compound assignment target".to_string(),
                ))
            }
        };

        // Compute new value
        let rhs = self.eval_expr(value)?;
        let new_val = self.apply_binary_op(&current, op, &rhs)?;

        // Assign back
        match &target.kind {
            ExprKind::Identifier(name) => {
                self.set_variable(name, new_val.clone());
            }
            ExprKind::FieldAccess { object, field } => {
                // Reuse eval_field_assign which handles all object types
                self.eval_field_assign(object, field, new_val.clone())?;
            }
            _ => {
                return Err(InterpreterError::RuntimeError(
                    "Complex assignment targets not supported in compound assignment"
                        .to_string(),
                ))
            }
        }

        Ok(new_val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Span, Type, TypeKind};
    use std::collections::HashMap;

    fn make_interpreter() -> Interpreter {
        Interpreter::new()
    }

    fn make_expr(kind: ExprKind) -> Expr {
        Expr::new(kind, Span::default())
    }

    fn make_type(name: &str) -> Type {
        Type {
            kind: TypeKind::Named(name.to_string()),
            span: Span::default(),
        }
    }

    fn make_match_arm(pattern: Pattern, guard: Option<Expr>, body: Expr) -> MatchArm {
        MatchArm {
            pattern,
            guard: guard.map(Box::new),
            body: Box::new(body),
            span: Span::default(),
        }
    }

    // =========================================================================
    // Range extraction and iteration tests
    // =========================================================================

    #[test]
    fn test_extract_range_bounds_success() {
        let interp = make_interpreter();
        let start = Value::Integer(0);
        let end = Value::Integer(10);
        let result = interp.extract_range_bounds(&start, &end).unwrap();
        assert_eq!(result, (0, 10));
    }

    #[test]
    fn test_extract_range_bounds_negative() {
        let interp = make_interpreter();
        let start = Value::Integer(-5);
        let end = Value::Integer(5);
        let result = interp.extract_range_bounds(&start, &end).unwrap();
        assert_eq!(result, (-5, 5));
    }

    #[test]
    fn test_extract_range_bounds_non_integer() {
        let interp = make_interpreter();
        let start = Value::Float(0.0);
        let end = Value::Integer(10);
        let result = interp.extract_range_bounds(&start, &end);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("integers"));
    }

    #[test]
    fn test_create_range_iterator_exclusive() {
        let interp = make_interpreter();
        let iter = interp.create_range_iterator(0, 5, false);
        let collected: Vec<i64> = iter.collect();
        assert_eq!(collected, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_create_range_iterator_inclusive() {
        let interp = make_interpreter();
        let iter = interp.create_range_iterator(0, 5, true);
        let collected: Vec<i64> = iter.collect();
        assert_eq!(collected, vec![0, 1, 2, 3, 4, 5]);
    }

    // =========================================================================
    // For loop array iteration tests
    // =========================================================================

    #[test]
    fn test_eval_for_array_iteration_empty() {
        let mut interp = make_interpreter();
        let body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));

        let result = interp
            .eval_for_array_iteration(None, "x", &[], &body)
            .unwrap();
        assert_eq!(result, Value::nil());
    }

    #[test]
    fn test_eval_for_array_iteration_simple() {
        let mut interp = make_interpreter();
        interp.set_variable("count", Value::Integer(0));

        let arr = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        // Just return the loop variable - this tests iteration works
        let body = make_expr(ExprKind::Identifier("x".to_string()));

        let result = interp
            .eval_for_array_iteration(None, "x", &arr, &body)
            .unwrap();
        // Last value is 3
        assert_eq!(result, Value::Integer(3));
    }

    // =========================================================================
    // For loop range iteration tests
    // =========================================================================

    #[test]
    fn test_eval_for_range_iteration_simple() {
        let mut interp = make_interpreter();

        let start = Value::Integer(0);
        let end = Value::Integer(3);
        let body = make_expr(ExprKind::Identifier("i".to_string()));

        let result = interp
            .eval_for_range_iteration(None, "i", &start, &end, false, &body)
            .unwrap();
        // Last value is 2 (exclusive)
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_eval_for_range_iteration_inclusive() {
        let mut interp = make_interpreter();

        let start = Value::Integer(0);
        let end = Value::Integer(3);
        let body = make_expr(ExprKind::Identifier("i".to_string()));

        let result = interp
            .eval_for_range_iteration(None, "i", &start, &end, true, &body)
            .unwrap();
        // Last value is 3 (inclusive)
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_for_loop_non_iterable() {
        let mut interp = make_interpreter();

        let non_iterable = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let body = make_expr(ExprKind::Literal(Literal::Unit));

        let result = interp.eval_for_loop(None, "x", None, &non_iterable, &body);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("iterable"));
    }

    // =========================================================================
    // While loop tests
    // =========================================================================

    #[test]
    fn test_eval_while_loop_false_condition() {
        let mut interp = make_interpreter();

        // while false { ... } - never executes
        let condition = make_expr(ExprKind::Literal(Literal::Bool(false)));
        let body = make_expr(ExprKind::Literal(Literal::Integer(999, None)));

        let result = interp.eval_while_loop(None, &condition, &body).unwrap();
        assert_eq!(result, Value::Nil); // Never executed, returns Nil
    }

    // =========================================================================
    // Loop body control flow tests
    // =========================================================================

    #[test]
    fn test_eval_loop_body_with_control_flow_ok() {
        let mut interp = make_interpreter();
        let body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));

        let result = interp.eval_loop_body_with_control_flow(&body).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_loop_body_with_control_flow_break() {
        let mut interp = make_interpreter();
        let body = make_expr(ExprKind::Break {
            label: None,
            value: Some(Box::new(make_expr(ExprKind::Literal(Literal::Integer(
                99, None,
            ))))),
        });

        let result = interp.eval_loop_body_with_control_flow(&body);
        match result {
            Err(LoopControlOrError::Break(None, Value::Integer(99))) => {}
            _ => panic!("Expected Break with value 99"),
        }
    }

    #[test]
    fn test_eval_loop_body_with_control_flow_continue() {
        let mut interp = make_interpreter();
        let body = make_expr(ExprKind::Continue { label: None });

        let result = interp.eval_loop_body_with_control_flow(&body);
        match result {
            Err(LoopControlOrError::Continue(None)) => {}
            _ => panic!("Expected Continue"),
        }
    }

    #[test]
    fn test_eval_loop_body_with_control_flow_return() {
        let mut interp = make_interpreter();
        let body = make_expr(ExprKind::Return {
            value: Some(Box::new(make_expr(ExprKind::Literal(Literal::Integer(
                42, None,
            ))))),
        });

        let result = interp.eval_loop_body_with_control_flow(&body);
        match result {
            Err(LoopControlOrError::Return(Value::Integer(42))) => {}
            _ => panic!("Expected Return with value 42"),
        }
    }

    // =========================================================================
    // Match expression tests
    // =========================================================================

    #[test]
    fn test_eval_match_literal() {
        let mut interp = make_interpreter();

        // match 1 { 1 => "one", _ => "other" }
        let expr = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let arms = vec![
            make_match_arm(
                Pattern::Literal(Literal::Integer(1, None)),
                None,
                make_expr(ExprKind::Literal(Literal::String("one".to_string()))),
            ),
            make_match_arm(
                Pattern::Wildcard,
                None,
                make_expr(ExprKind::Literal(Literal::String("other".to_string()))),
            ),
        ];

        let result = interp.eval_match(&expr, &arms).unwrap();
        assert_eq!(result, Value::from_string("one".to_string()));
    }

    #[test]
    fn test_eval_match_wildcard() {
        let mut interp = make_interpreter();

        let expr = make_expr(ExprKind::Literal(Literal::Integer(999, None)));
        let arms = vec![
            make_match_arm(
                Pattern::Literal(Literal::Integer(1, None)),
                None,
                make_expr(ExprKind::Literal(Literal::String("one".to_string()))),
            ),
            make_match_arm(
                Pattern::Wildcard,
                None,
                make_expr(ExprKind::Literal(Literal::String("other".to_string()))),
            ),
        ];

        let result = interp.eval_match(&expr, &arms).unwrap();
        assert_eq!(result, Value::from_string("other".to_string()));
    }

    #[test]
    fn test_eval_match_with_binding() {
        let mut interp = make_interpreter();

        // match 42 { x => x }
        let expr = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let arms = vec![make_match_arm(
            Pattern::Identifier("x".to_string()),
            None,
            make_expr(ExprKind::Identifier("x".to_string())),
        )];

        let result = interp.eval_match(&expr, &arms).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_match_with_guard() {
        let mut interp = make_interpreter();

        // match 5 { x if x > 3 => "big", _ => "small" }
        let expr = make_expr(ExprKind::Literal(Literal::Integer(5, None)));
        let arms = vec![
            make_match_arm(
                Pattern::Identifier("x".to_string()),
                Some(make_expr(ExprKind::Binary {
                    left: Box::new(make_expr(ExprKind::Identifier("x".to_string()))),
                    op: AstBinaryOp::Greater,
                    right: Box::new(make_expr(ExprKind::Literal(Literal::Integer(3, None)))),
                })),
                make_expr(ExprKind::Literal(Literal::String("big".to_string()))),
            ),
            make_match_arm(
                Pattern::Wildcard,
                None,
                make_expr(ExprKind::Literal(Literal::String("small".to_string()))),
            ),
        ];

        let result = interp.eval_match(&expr, &arms).unwrap();
        assert_eq!(result, Value::from_string("big".to_string()));
    }

    #[test]
    fn test_eval_match_guard_fails() {
        let mut interp = make_interpreter();

        // match 2 { x if x > 3 => "big", _ => "small" }
        let expr = make_expr(ExprKind::Literal(Literal::Integer(2, None)));
        let arms = vec![
            make_match_arm(
                Pattern::Identifier("x".to_string()),
                Some(make_expr(ExprKind::Binary {
                    left: Box::new(make_expr(ExprKind::Identifier("x".to_string()))),
                    op: AstBinaryOp::Greater,
                    right: Box::new(make_expr(ExprKind::Literal(Literal::Integer(3, None)))),
                })),
                make_expr(ExprKind::Literal(Literal::String("big".to_string()))),
            ),
            make_match_arm(
                Pattern::Wildcard,
                None,
                make_expr(ExprKind::Literal(Literal::String("small".to_string()))),
            ),
        ];

        let result = interp.eval_match(&expr, &arms).unwrap();
        assert_eq!(result, Value::from_string("small".to_string()));
    }

    #[test]
    fn test_eval_match_no_match() {
        let mut interp = make_interpreter();

        let expr = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let arms = vec![make_match_arm(
            Pattern::Literal(Literal::Integer(1, None)),
            None,
            make_expr(ExprKind::Literal(Literal::Integer(0, None))),
        )];

        let result = interp.eval_match(&expr, &arms);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No match arm matched"));
    }

    // =========================================================================
    // Extract mut names tests
    // =========================================================================

    #[test]
    fn test_extract_mut_names_simple() {
        let pattern = Pattern::Mut(Box::new(Pattern::Identifier("x".to_string())));
        let mut_names = Interpreter::extract_mut_names(&pattern);
        assert!(mut_names.contains("x"));
    }

    #[test]
    fn test_extract_mut_names_tuple() {
        // (mut a, b, mut c)
        let pattern = Pattern::Tuple(vec![
            Pattern::Mut(Box::new(Pattern::Identifier("a".to_string()))),
            Pattern::Identifier("b".to_string()),
            Pattern::Mut(Box::new(Pattern::Identifier("c".to_string()))),
        ]);
        let mut_names = Interpreter::extract_mut_names(&pattern);
        assert!(mut_names.contains("a"));
        assert!(!mut_names.contains("b"));
        assert!(mut_names.contains("c"));
    }

    #[test]
    fn test_extract_mut_names_list() {
        // [mut x, y]
        let pattern = Pattern::List(vec![
            Pattern::Mut(Box::new(Pattern::Identifier("x".to_string()))),
            Pattern::Identifier("y".to_string()),
        ]);
        let mut_names = Interpreter::extract_mut_names(&pattern);
        assert!(mut_names.contains("x"));
        assert!(!mut_names.contains("y"));
    }

    #[test]
    fn test_extract_mut_names_no_mut() {
        let pattern = Pattern::Identifier("x".to_string());
        let mut_names = Interpreter::extract_mut_names(&pattern);
        assert!(mut_names.is_empty());
    }

    // =========================================================================
    // Let pattern tests
    // =========================================================================

    #[test]
    fn test_eval_let_pattern_simple() {
        let mut interp = make_interpreter();

        // let x = 42; body returns x
        let pattern = Pattern::Identifier("x".to_string());
        let value = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let body = make_expr(ExprKind::Identifier("x".to_string()));

        let result = interp.eval_let_pattern(&pattern, &value, &body).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_let_pattern_tuple() {
        let mut interp = make_interpreter();

        // let (a, b) = (1, 2); body returns a + b
        let pattern = Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ]);
        let value = make_expr(ExprKind::Tuple(vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Literal(Literal::Integer(2, None))),
        ]));
        let body = make_expr(ExprKind::Binary {
            left: Box::new(make_expr(ExprKind::Identifier("a".to_string()))),
            op: AstBinaryOp::Add,
            right: Box::new(make_expr(ExprKind::Identifier("b".to_string()))),
        });

        let result = interp.eval_let_pattern(&pattern, &value, &body).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_let_pattern_unit_body() {
        let mut interp = make_interpreter();

        // let x = 42; () - should return the value like REPL
        let pattern = Pattern::Identifier("x".to_string());
        let value = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let body = make_expr(ExprKind::Literal(Literal::Unit));

        let result = interp.eval_let_pattern(&pattern, &value, &body).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    // =========================================================================
    // Assignment tests
    // =========================================================================

    #[test]
    fn test_eval_assign_identifier() {
        let mut interp = make_interpreter();
        interp.set_variable("x", Value::Integer(0));

        let target = make_expr(ExprKind::Identifier("x".to_string()));
        let value = make_expr(ExprKind::Literal(Literal::Integer(42, None)));

        let result = interp.eval_assign(&target, &value).unwrap();
        assert_eq!(result, Value::Integer(42));
        assert_eq!(interp.lookup_variable("x").unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_eval_assign_object_field() {
        let mut interp = make_interpreter();

        // Create an object with a field
        let mut obj = HashMap::new();
        obj.insert("x".to_string(), Value::Integer(0));
        interp.set_variable("point", Value::Object(Arc::new(obj)));

        // point.x = 42
        let target = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("point".to_string()))),
            field: "x".to_string(),
        });
        let value = make_expr(ExprKind::Literal(Literal::Integer(42, None)));

        let result = interp.eval_assign(&target, &value).unwrap();
        assert_eq!(result, Value::Integer(42));

        let updated_point = interp.lookup_variable("point").unwrap();
        if let Value::Object(obj) = updated_point {
            assert_eq!(obj.get("x"), Some(&Value::Integer(42)));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_assign_struct_field() {
        let mut interp = make_interpreter();

        // Create a struct with a field
        let mut fields = HashMap::new();
        fields.insert("x".to_string(), Value::Integer(0));
        interp.set_variable(
            "point",
            Value::Struct {
                name: "Point".to_string(),
                fields: Arc::new(fields),
            },
        );

        // point.x = 42
        let target = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("point".to_string()))),
            field: "x".to_string(),
        });
        let value = make_expr(ExprKind::Literal(Literal::Integer(42, None)));

        let result = interp.eval_assign(&target, &value).unwrap();
        assert_eq!(result, Value::Integer(42));

        let updated_point = interp.lookup_variable("point").unwrap();
        if let Value::Struct { fields, .. } = updated_point {
            assert_eq!(fields.get("x"), Some(&Value::Integer(42)));
        } else {
            panic!("Expected Struct");
        }
    }

    #[test]
    fn test_eval_assign_invalid_target() {
        let mut interp = make_interpreter();

        // 42 = 0 - invalid target
        let target = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let value = make_expr(ExprKind::Literal(Literal::Integer(0, None)));

        let result = interp.eval_assign(&target, &value);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid assignment target"));
    }

    // =========================================================================
    // Index assignment tests
    // =========================================================================

    #[test]
    fn test_eval_index_assign_simple() {
        let mut interp = make_interpreter();

        // arr = [1, 2, 3]
        interp.set_variable(
            "arr",
            Value::Array(Arc::from(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ])),
        );

        // arr[1] = 99
        let object = make_expr(ExprKind::Identifier("arr".to_string()));
        let index = make_expr(ExprKind::Literal(Literal::Integer(1, None)));

        let result = interp
            .eval_index_assign(&object, &index, Value::Integer(99))
            .unwrap();
        assert_eq!(result, Value::Integer(99));

        let updated_arr = interp.lookup_variable("arr").unwrap();
        if let Value::Array(arr) = updated_arr {
            assert_eq!(arr[1], Value::Integer(99));
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_index_assign_out_of_bounds() {
        let mut interp = make_interpreter();

        interp.set_variable(
            "arr",
            Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)])),
        );

        let object = make_expr(ExprKind::Identifier("arr".to_string()));
        let index = make_expr(ExprKind::Literal(Literal::Integer(10, None)));

        let result = interp.eval_index_assign(&object, &index, Value::Integer(99));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }

    #[test]
    fn test_eval_index_assign_non_integer_index() {
        let mut interp = make_interpreter();

        interp.set_variable("arr", Value::Array(Arc::from(vec![Value::Integer(1)])));

        let object = make_expr(ExprKind::Identifier("arr".to_string()));
        let index = make_expr(ExprKind::Literal(Literal::String("key".to_string())));

        let result = interp.eval_index_assign(&object, &index, Value::Integer(99));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be an integer"));
    }

    #[test]
    fn test_eval_index_assign_non_array() {
        let mut interp = make_interpreter();

        interp.set_variable("x", Value::Integer(42));

        let object = make_expr(ExprKind::Identifier("x".to_string()));
        let index = make_expr(ExprKind::Literal(Literal::Integer(0, None)));

        let result = interp.eval_index_assign(&object, &index, Value::Integer(99));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot index non-array"));
    }

    #[test]
    fn test_eval_index_assign_nested() {
        let mut interp = make_interpreter();

        // matrix = [[1, 2], [3, 4]]
        interp.set_variable(
            "matrix",
            Value::Array(Arc::from(vec![
                Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)])),
                Value::Array(Arc::from(vec![Value::Integer(3), Value::Integer(4)])),
            ])),
        );

        // matrix[0][1] = 99
        let nested_obj = make_expr(ExprKind::Identifier("matrix".to_string()));
        let outer_idx = make_expr(ExprKind::Literal(Literal::Integer(0, None)));
        let object = make_expr(ExprKind::IndexAccess {
            object: Box::new(nested_obj),
            index: Box::new(outer_idx),
        });
        let inner_idx = make_expr(ExprKind::Literal(Literal::Integer(1, None)));

        let result = interp
            .eval_index_assign(&object, &inner_idx, Value::Integer(99))
            .unwrap();
        assert_eq!(result, Value::Integer(99));

        let updated = interp.lookup_variable("matrix").unwrap();
        if let Value::Array(outer) = updated {
            if let Value::Array(inner) = &outer[0] {
                assert_eq!(inner[1], Value::Integer(99));
            } else {
                panic!("Expected inner array");
            }
        } else {
            panic!("Expected Array");
        }
    }

    // =========================================================================
    // Compound assignment tests
    // =========================================================================

    #[test]
    fn test_eval_compound_assign_add() {
        let mut interp = make_interpreter();
        interp.set_variable("x", Value::Integer(10));

        // x += 5
        let target = make_expr(ExprKind::Identifier("x".to_string()));
        let value = make_expr(ExprKind::Literal(Literal::Integer(5, None)));

        let result = interp
            .eval_compound_assign(&target, AstBinaryOp::Add, &value)
            .unwrap();
        assert_eq!(result, Value::Integer(15));
        assert_eq!(interp.lookup_variable("x").unwrap(), Value::Integer(15));
    }

    #[test]
    fn test_eval_compound_assign_sub() {
        let mut interp = make_interpreter();
        interp.set_variable("x", Value::Integer(10));

        // x -= 3
        let target = make_expr(ExprKind::Identifier("x".to_string()));
        let value = make_expr(ExprKind::Literal(Literal::Integer(3, None)));

        let result = interp
            .eval_compound_assign(&target, AstBinaryOp::Subtract, &value)
            .unwrap();
        assert_eq!(result, Value::Integer(7));
    }

    #[test]
    fn test_eval_compound_assign_mul() {
        let mut interp = make_interpreter();
        interp.set_variable("x", Value::Integer(10));

        // x *= 2
        let target = make_expr(ExprKind::Identifier("x".to_string()));
        let value = make_expr(ExprKind::Literal(Literal::Integer(2, None)));

        let result = interp
            .eval_compound_assign(&target, AstBinaryOp::Multiply, &value)
            .unwrap();
        assert_eq!(result, Value::Integer(20));
    }

    #[test]
    fn test_eval_compound_assign_field() {
        let mut interp = make_interpreter();

        let mut obj = HashMap::new();
        obj.insert("count".to_string(), Value::Integer(5));
        interp.set_variable("counter", Value::Object(Arc::new(obj)));

        // counter.count += 1
        let target = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("counter".to_string()))),
            field: "count".to_string(),
        });
        let value = make_expr(ExprKind::Literal(Literal::Integer(1, None)));

        let result = interp
            .eval_compound_assign(&target, AstBinaryOp::Add, &value)
            .unwrap();
        assert_eq!(result, Value::Integer(6));
    }

    #[test]
    fn test_eval_compound_assign_invalid_target() {
        let mut interp = make_interpreter();

        // 42 += 1 - invalid target
        let target = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let value = make_expr(ExprKind::Literal(Literal::Integer(1, None)));

        let result = interp.eval_compound_assign(&target, AstBinaryOp::Add, &value);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid compound assignment target"));
    }

    // ============================================================================
    // Coverage tests for eval_index_assign (26 uncov lines, 66.2% coverage)
    // ============================================================================

    #[test]
    fn test_eval_index_assign_simple_array() {
        let mut interp = make_interpreter();
        interp.set_variable(
            "arr",
            Value::Array(Arc::from(vec![
                Value::Integer(10),
                Value::Integer(20),
                Value::Integer(30),
            ])),
        );

        let object = make_expr(ExprKind::Identifier("arr".to_string()));
        let index = make_expr(ExprKind::Literal(Literal::Integer(1, None)));

        let result = interp
            .eval_index_assign(&object, &index, Value::Integer(99))
            .expect("simple index assign should succeed");
        assert_eq!(result, Value::Integer(99));

        // Verify the array was updated
        let arr = interp.lookup_variable("arr").unwrap();
        if let Value::Array(v) = arr {
            assert_eq!(v[1], Value::Integer(99));
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_index_assign_out_of_bounds_v2() {
        let mut interp = make_interpreter();
        interp.set_variable("arr", Value::Array(Arc::from(vec![Value::Integer(1)])));

        let object = make_expr(ExprKind::Identifier("arr".to_string()));
        let index = make_expr(ExprKind::Literal(Literal::Integer(5, None)));

        let result = interp.eval_index_assign(&object, &index, Value::Integer(99));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }

    #[test]
    fn test_eval_index_assign_non_integer_index_v2() {
        let mut interp = make_interpreter();
        interp.set_variable("arr", Value::Array(Arc::from(vec![Value::Integer(1)])));

        let object = make_expr(ExprKind::Identifier("arr".to_string()));
        let index = make_expr(ExprKind::Literal(Literal::String("bad".to_string())));

        let result = interp.eval_index_assign(&object, &index, Value::Integer(99));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("index must be an integer"));
    }

    #[test]
    fn test_eval_index_assign_non_array_target() {
        let mut interp = make_interpreter();
        interp.set_variable("x", Value::Integer(42));

        let object = make_expr(ExprKind::Identifier("x".to_string()));
        let index = make_expr(ExprKind::Literal(Literal::Integer(0, None)));

        let result = interp.eval_index_assign(&object, &index, Value::Integer(99));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot index non-array"));
    }

    #[test]
    fn test_eval_index_assign_nested_matrix() {
        let mut interp = make_interpreter();
        let matrix = Value::Array(Arc::from(vec![
            Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)])),
            Value::Array(Arc::from(vec![Value::Integer(3), Value::Integer(4)])),
        ]));
        interp.set_variable("matrix", matrix);

        // matrix[0][1] = 99
        let inner_obj = Box::new(make_expr(ExprKind::Identifier("matrix".to_string())));
        let inner_idx = Box::new(make_expr(ExprKind::Literal(Literal::Integer(0, None))));
        let outer_access = make_expr(ExprKind::IndexAccess {
            object: inner_obj,
            index: inner_idx,
        });
        let outer_idx = make_expr(ExprKind::Literal(Literal::Integer(1, None)));

        let result = interp
            .eval_index_assign(&outer_access, &outer_idx, Value::Integer(99))
            .expect("nested index assign should succeed");
        assert_eq!(result, Value::Integer(99));

        // Verify
        let mat = interp.lookup_variable("matrix").unwrap();
        if let Value::Array(outer) = mat {
            if let Value::Array(inner) = &outer[0] {
                assert_eq!(inner[1], Value::Integer(99));
            } else {
                panic!("Expected inner Array");
            }
        } else {
            panic!("Expected outer Array");
        }
    }

    #[test]
    fn test_eval_index_assign_nested_outer_oob() {
        let mut interp = make_interpreter();
        let matrix = Value::Array(Arc::from(vec![Value::Array(Arc::from(vec![
            Value::Integer(1),
        ]))]));
        interp.set_variable("matrix", matrix);

        // matrix[5][0] = 99
        let inner_obj = Box::new(make_expr(ExprKind::Identifier("matrix".to_string())));
        let inner_idx = Box::new(make_expr(ExprKind::Literal(Literal::Integer(5, None))));
        let outer_access = make_expr(ExprKind::IndexAccess {
            object: inner_obj,
            index: inner_idx,
        });
        let outer_idx = make_expr(ExprKind::Literal(Literal::Integer(0, None)));

        let result = interp.eval_index_assign(&outer_access, &outer_idx, Value::Integer(99));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Outer index"));
    }

    #[test]
    fn test_eval_index_assign_nested_inner_oob() {
        let mut interp = make_interpreter();
        let matrix = Value::Array(Arc::from(vec![Value::Array(Arc::from(vec![
            Value::Integer(1),
        ]))]));
        interp.set_variable("matrix", matrix);

        // matrix[0][5] = 99
        let inner_obj = Box::new(make_expr(ExprKind::Identifier("matrix".to_string())));
        let inner_idx = Box::new(make_expr(ExprKind::Literal(Literal::Integer(0, None))));
        let outer_access = make_expr(ExprKind::IndexAccess {
            object: inner_obj,
            index: inner_idx,
        });
        let outer_idx = make_expr(ExprKind::Literal(Literal::Integer(5, None)));

        let result = interp.eval_index_assign(&outer_access, &outer_idx, Value::Integer(99));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Inner index"));
    }

    #[test]
    fn test_eval_index_assign_nested_non_array_inner() {
        let mut interp = make_interpreter();
        // matrix[0] is not an array
        let matrix = Value::Array(Arc::from(vec![Value::Integer(42)]));
        interp.set_variable("matrix", matrix);

        let inner_obj = Box::new(make_expr(ExprKind::Identifier("matrix".to_string())));
        let inner_idx = Box::new(make_expr(ExprKind::Literal(Literal::Integer(0, None))));
        let outer_access = make_expr(ExprKind::IndexAccess {
            object: inner_obj,
            index: inner_idx,
        });
        let outer_idx = make_expr(ExprKind::Literal(Literal::Integer(0, None)));

        let result = interp.eval_index_assign(&outer_access, &outer_idx, Value::Integer(99));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot index non-array"));
    }

    #[test]
    fn test_eval_index_assign_unsupported_target() {
        let mut interp = make_interpreter();
        // Use a literal as the assignment target (not Identifier or IndexAccess)
        let object = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let index = make_expr(ExprKind::Literal(Literal::Integer(0, None)));

        let result = interp.eval_index_assign(&object, &index, Value::Integer(99));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not yet supported"));
    }

    // ============================================================================
    // Coverage tests for eval_assign uncovered branches (18 uncov, 66.7%)
    // ============================================================================

    #[test]
    fn test_eval_assign_object_mut_field() {
        let mut interp = make_interpreter();

        // Create an ObjectMut with a field
        let mut obj = HashMap::new();
        obj.insert("x".to_string(), Value::Integer(0));
        let obj_mut = Value::ObjectMut(Arc::new(std::sync::Mutex::new(obj)));
        interp.set_variable("point", obj_mut);

        // point.x = 42
        let target = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("point".to_string()))),
            field: "x".to_string(),
        });
        let value = make_expr(ExprKind::Literal(Literal::Integer(42, None)));

        let result = interp.eval_assign(&target, &value).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_assign_class_field() {
        let mut interp = make_interpreter();

        // Create a Class value
        let mut fields = HashMap::new();
        fields.insert("x".to_string(), Value::Integer(0));
        let class_val = Value::Class {
            class_name: "Point".to_string(),
            fields: Arc::new(std::sync::RwLock::new(fields)),
            methods: Arc::new(HashMap::new()),
        };
        interp.set_variable("point", class_val);

        // point.x = 42
        let target = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("point".to_string()))),
            field: "x".to_string(),
        });
        let value = make_expr(ExprKind::Literal(Literal::Integer(42, None)));

        let result = interp.eval_assign(&target, &value).unwrap();
        assert_eq!(result, Value::Integer(42));

        // Verify the field was updated
        let updated = interp.lookup_variable("point").unwrap();
        if let Value::Class { fields, .. } = updated {
            let f = fields.read().unwrap();
            assert_eq!(f.get("x"), Some(&Value::Integer(42)));
        } else {
            panic!("Expected Class");
        }
    }

    #[test]
    fn test_eval_assign_field_on_non_object() {
        let mut interp = make_interpreter();

        // Set a plain integer variable
        interp.set_variable("x", Value::Integer(42));

        // Try x.field = 10 -- should fail (non-object)
        let target = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("x".to_string()))),
            field: "field".to_string(),
        });
        let value = make_expr(ExprKind::Literal(Literal::Integer(10, None)));

        let result = interp.eval_assign(&target, &value);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot access field"));
    }

    #[test]
    fn test_eval_assign_complex_field_target() {
        let mut interp = make_interpreter();

        // Set up nested object
        let mut inner = HashMap::new();
        inner.insert("z".to_string(), Value::Integer(0));
        let mut outer = HashMap::new();
        outer.insert("inner".to_string(), Value::Object(Arc::new(inner)));
        interp.set_variable("obj", Value::Object(Arc::new(outer)));

        // obj.inner.z = 5  -- complex target (object is a FieldAccess, not Identifier)
        let target = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::FieldAccess {
                object: Box::new(make_expr(ExprKind::Identifier("obj".to_string()))),
                field: "inner".to_string(),
            })),
            field: "z".to_string(),
        });
        let value = make_expr(ExprKind::Literal(Literal::Integer(5, None)));

        let result = interp.eval_assign(&target, &value);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Complex field access not supported"));
    }

    #[test]
    fn test_eval_assign_index_access() {
        let mut interp = make_interpreter();

        // arr = [1, 2, 3]
        interp.set_variable(
            "arr",
            Value::Array(Arc::from(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ])),
        );

        // arr[0] = 99
        let target = make_expr(ExprKind::IndexAccess {
            object: Box::new(make_expr(ExprKind::Identifier("arr".to_string()))),
            index: Box::new(make_expr(ExprKind::Literal(Literal::Integer(0, None)))),
        });
        let value = make_expr(ExprKind::Literal(Literal::Integer(99, None)));

        let result = interp.eval_assign(&target, &value).unwrap();
        assert_eq!(result, Value::Integer(99));
    }

    // ============================================================================
    // Coverage tests for eval_compound_assign uncovered branches (17 uncov, 64.6%)
    // ============================================================================

    #[test]
    fn test_eval_compound_assign_struct_field() {
        let mut interp = make_interpreter();

        let mut fields = HashMap::new();
        fields.insert("count".to_string(), Value::Integer(5));
        interp.set_variable(
            "counter",
            Value::Struct {
                name: "Counter".to_string(),
                fields: Arc::new(fields),
            },
        );

        // counter.count += 1
        let target = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("counter".to_string()))),
            field: "count".to_string(),
        });
        let value = make_expr(ExprKind::Literal(Literal::Integer(1, None)));

        let result = interp
            .eval_compound_assign(&target, AstBinaryOp::Add, &value)
            .unwrap();
        assert_eq!(result, Value::Integer(6));

        // Verify updated
        let updated = interp.lookup_variable("counter").unwrap();
        if let Value::Struct { fields, .. } = updated {
            assert_eq!(fields.get("count"), Some(&Value::Integer(6)));
        } else {
            panic!("Expected Struct");
        }
    }

    #[test]
    fn test_eval_compound_assign_field_non_object_error() {
        let mut interp = make_interpreter();

        // Set a plain integer
        interp.set_variable("x", Value::Integer(10));

        // x.field += 1  -- should fail
        let target = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("x".to_string()))),
            field: "field".to_string(),
        });
        let value = make_expr(ExprKind::Literal(Literal::Integer(1, None)));

        let result = interp.eval_compound_assign(&target, AstBinaryOp::Add, &value);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_compound_assign_complex_field_access_error() {
        let mut interp = make_interpreter();

        // Set up nested object
        let mut inner = HashMap::new();
        inner.insert("z".to_string(), Value::Integer(5));
        let mut outer = HashMap::new();
        outer.insert("inner".to_string(), Value::Object(Arc::new(inner)));
        interp.set_variable("obj", Value::Object(Arc::new(outer)));

        // obj.inner.z += 1  -- complex field access not supported
        let target = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::FieldAccess {
                object: Box::new(make_expr(ExprKind::Identifier("obj".to_string()))),
                field: "inner".to_string(),
            })),
            field: "z".to_string(),
        });
        let value = make_expr(ExprKind::Literal(Literal::Integer(1, None)));

        let result = interp.eval_compound_assign(&target, AstBinaryOp::Add, &value);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Complex field access not supported"));
    }

    #[test]
    fn test_eval_compound_assign_multiply_float() {
        let mut interp = make_interpreter();
        interp.set_variable("x", Value::Float(2.5));

        // x *= 2.0
        let target = make_expr(ExprKind::Identifier("x".to_string()));
        let value = make_expr(ExprKind::Literal(Literal::Float(2.0)));

        let result = interp
            .eval_compound_assign(&target, AstBinaryOp::Multiply, &value)
            .unwrap();
        assert_eq!(result, Value::Float(5.0));
    }

    #[test]
    fn test_eval_compound_assign_divide() {
        let mut interp = make_interpreter();
        interp.set_variable("x", Value::Integer(10));

        // x /= 2
        let target = make_expr(ExprKind::Identifier("x".to_string()));
        let value = make_expr(ExprKind::Literal(Literal::Integer(2, None)));

        let result = interp
            .eval_compound_assign(&target, AstBinaryOp::Divide, &value)
            .unwrap();
        assert_eq!(result, Value::Integer(5));
    }
}
