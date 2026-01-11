//! Instance method dispatch for mutable objects (actors, classes, structs, files)
//!
//! Extracted from interpreter_methods.rs for coverage attribution.

#![allow(clippy::unused_self)]
#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::expect_used)]

use crate::runtime::interpreter::Interpreter;
use crate::runtime::{InterpreterError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

impl Interpreter {
    // ObjectMut adapter methods - delegate to immutable versions via borrow
    // Complexity: 2 each (simple delegation)

    pub(crate) fn eval_actor_instance_method_mut(
        &mut self,
        cell_rc: &Arc<std::sync::Mutex<std::collections::HashMap<String, Value>>>,
        actor_name: &str,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Special handling for send method - needs mutable state access
        if method == "send" {
            if arg_values.is_empty() {
                return Err(InterpreterError::RuntimeError(
                    "send() requires a message argument".to_string(),
                ));
            }
            return self.process_actor_message_sync_mut(cell_rc, &arg_values[0]);
        }

        // For other methods, delegate to non-mut version
        let instance = cell_rc
            .lock()
            .expect("Mutex poisoned: instance lock is corrupted");
        self.eval_actor_instance_method(&instance, actor_name, method, arg_values)
    }

    pub(crate) fn eval_class_instance_method_mut(
        &mut self,
        cell_rc: &Arc<std::sync::Mutex<std::collections::HashMap<String, Value>>>,
        class_name: &str,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        // For mutable instances, we need to pass ObjectMut as self (not a copy)
        // This allows &mut self methods to mutate the instance in place

        // Look up the class definition
        let class_def = self.lookup_variable(class_name)?;

        if let Value::Object(ref class_info) = class_def {
            // Look for the method in the class definition
            if let Some(Value::Object(ref methods)) = class_info.get("__methods") {
                if let Some(Value::Object(ref method_meta)) = methods.get(method) {
                    // Get the method closure
                    if let Some(Value::Closure { params, body, .. }) = method_meta.get("closure") {
                        // Check if it's a static method
                        let is_static = method_meta
                            .get("is_static")
                            .and_then(|v| {
                                if let Value::Bool(b) = v {
                                    Some(*b)
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(false);

                        if is_static {
                            return Err(InterpreterError::RuntimeError(format!(
                                "Cannot call static method {} on instance",
                                method
                            )));
                        }

                        // Create environment for method execution
                        let mut method_env = HashMap::new();

                        // CRITICAL: Pass ObjectMut as self, using the SAME Arc<RefCell<>>
                        // This enables &mut self methods to mutate the shared instance
                        method_env
                            .insert("self".to_string(), Value::ObjectMut(Arc::clone(cell_rc)));

                        // Bind method parameters to arguments
                        if arg_values.len() != params.len() {
                            return Err(InterpreterError::RuntimeError(format!(
                                "Method {} expects {} arguments, got {}",
                                method,
                                params.len(),
                                arg_values.len()
                            )));
                        }

                        // RUNTIME-DEFAULT-PARAMS: Extract param name from tuple (name, default_value)
                        for ((param_name, _default_value), arg) in params.iter().zip(arg_values) {
                            method_env.insert(param_name.clone(), arg.clone());
                        }

                        // Push method environment
                        self.env_push(method_env);

                        // Execute method body
                        let result = self.eval_expr(body)?;

                        // Pop environment
                        self.env_pop();

                        return Ok(result);
                    }
                }
            }

            // Method not found
            Err(InterpreterError::RuntimeError(format!(
                "Class {} has no method named {}",
                class_name, method
            )))
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "{} is not a class",
                class_name
            )))
        }
    }

    /// Evaluate instance method on `Value::Class` variant
    /// This is for the new Class implementation with Arc<`RwLock`<HashMap>>
    pub(crate) fn eval_class_instance_method_on_class(
        &mut self,
        class_name: &str,
        fields: &Arc<std::sync::RwLock<HashMap<String, Value>>>,
        methods: &Arc<HashMap<String, Value>>,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Look up the method in the methods map
        if let Some(method_closure) = methods.get(method) {
            if let Value::Closure { params, body, .. } = method_closure {
                // Check argument count
                if arg_values.len() != params.len() {
                    return Err(InterpreterError::RuntimeError(format!(
                        "Method {} expects {} arguments, got {}",
                        method,
                        params.len(),
                        arg_values.len()
                    )));
                }

                // Create environment for method execution
                let mut method_env = HashMap::new();

                // Bind 'self' to the class instance
                method_env.insert(
                    "self".to_string(),
                    Value::Class {
                        class_name: class_name.to_string(),
                        fields: Arc::clone(fields),
                        methods: Arc::clone(methods),
                    },
                );

                // RUNTIME-DEFAULT-PARAMS: Bind method parameters to arguments
                for ((param_name, _default_value), arg) in params.iter().zip(arg_values) {
                    method_env.insert(param_name.clone(), arg.clone());
                }

                // Push method environment
                self.env_push(method_env);

                // Execute method body
                let result = self.eval_expr(body)?;

                // Pop environment
                self.env_pop();

                Ok(result)
            } else {
                Err(InterpreterError::RuntimeError(format!(
                    "Method {} is not a closure",
                    method
                )))
            }
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "Method '{}' not found for type class",
                method
            )))
        }
    }

    pub(crate) fn eval_struct_instance_method_mut(
        &mut self,
        cell_rc: &Arc<std::sync::Mutex<std::collections::HashMap<String, Value>>>,
        struct_name: &str,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        // RUNTIME-ISSUE-148 FIX: Handle &mut self mutations correctly
        // Strategy: Execute method with modified version that returns (result, modified_self)

        // Clone instance data before executing method
        let instance_data = {
            let locked = cell_rc
                .lock()
                .expect("Mutex poisoned: cell lock is corrupted");
            locked.clone()
        };

        // Execute the method and capture modified self
        let (result, modified_self_opt) = self.eval_struct_instance_method_with_self_capture(
            &instance_data,
            struct_name,
            method,
            arg_values,
        )?;

        // If self was modified, write fields back to ObjectMut
        if let Some(modified_fields) = modified_self_opt {
            let mut locked = cell_rc
                .lock()
                .expect("Mutex poisoned: cell lock is corrupted");
            for (field_name, field_value) in modified_fields.iter() {
                locked.insert(field_name.clone(), field_value.clone());
            }
        }

        Ok(result)
    }

    /// Helper: Execute struct method and capture modified self
    /// Returns (`method_result`, Option<`modified_self_fields`>)
    pub(crate) fn eval_struct_instance_method_with_self_capture(
        &mut self,
        instance: &std::collections::HashMap<String, Value>,
        struct_name: &str,
        method: &str,
        arg_values: &[Value],
    ) -> Result<
        (
            Value,
            Option<std::sync::Arc<std::collections::HashMap<String, Value>>>,
        ),
        InterpreterError,
    > {
        // Look up impl method with qualified name
        let qualified_method_name = format!("{}::{}", struct_name, method);

        if let Ok(method_closure) = self.lookup_variable(&qualified_method_name) {
            if let Value::Closure { params, body, env } = method_closure {
                // Check argument count (including self)
                let expected_args = params.len();
                let provided_args = arg_values.len() + 1; // +1 for self

                if provided_args != expected_args {
                    return Err(InterpreterError::RuntimeError(format!(
                        "Method {} expects {} arguments, got {}",
                        method,
                        expected_args - 1, // -1 because self is implicit
                        arg_values.len()
                    )));
                }

                // Create new environment with method's captured environment as base
                let mut new_env = env.borrow().clone();

                // Bind self parameter (first parameter)
                let self_param_name = if let Some((name, _)) = params.first() {
                    new_env.insert(
                        name.clone(),
                        Value::Struct {
                            name: struct_name.to_string(),
                            fields: std::sync::Arc::new(instance.clone()),
                        },
                    );
                    name.clone()
                } else {
                    return Err(InterpreterError::RuntimeError(
                        "Method has no self parameter".to_string(),
                    ));
                };

                // Bind other parameters
                for (i, arg_value) in arg_values.iter().enumerate() {
                    if let Some((param_name, _)) = params.get(i + 1) {
                        new_env.insert(param_name.clone(), arg_value.clone());
                    }
                }

                // Execute method body with new environment
                self.env_stack.push(Rc::new(RefCell::new(new_env)));
                let result = self.eval_expr(&body);

                // CRITICAL: Extract modified self BEFORE popping environment
                let modified_self = if let Some(env_rc) = self.env_stack.last() {
                    let env_ref = env_rc.borrow();
                    if let Some(Value::Struct { fields, .. }) = env_ref.get(&self_param_name) {
                        Some(fields.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };

                self.env_stack.pop();

                result.map(|r| (r, modified_self))
            } else {
                Err(InterpreterError::RuntimeError(format!(
                    "Found {} but it's not a method closure",
                    qualified_method_name
                )))
            }
        } else {
            // Fall back to generic method handling - no self modifications
            let result = self.eval_generic_method(
                &Value::Object(std::sync::Arc::new(instance.clone())),
                method,
                arg_values.is_empty(),
            )?;
            Ok((result, None))
        }
    }

    /// ISSUE-116: Evaluate File object methods (.`read_line()`, .`close()`)
    /// Complexity: 6
    pub(crate) fn eval_file_method_mut(
        &mut self,
        file_obj: &Arc<std::sync::Mutex<HashMap<String, Value>>>,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        match method {
            "read_line" => {
                // Check args
                if !arg_values.is_empty() {
                    return Err(InterpreterError::RuntimeError(
                        "read_line() takes no arguments".to_string(),
                    ));
                }

                let mut obj = file_obj
                    .lock()
                    .expect("Mutex poisoned: file object lock is corrupted");

                // Check if closed
                if let Some(Value::Bool(true)) = obj.get("closed") {
                    return Err(InterpreterError::RuntimeError(
                        "Cannot read from closed file".to_string(),
                    ));
                }

                // Get current position
                let position = if let Some(Value::Integer(pos)) = obj.get("position") {
                    *pos
                } else {
                    0
                };

                // Get lines array
                let lines = if let Some(Value::Array(lines)) = obj.get("lines") {
                    lines.clone()
                } else {
                    return Err(InterpreterError::RuntimeError(
                        "File object corrupted: missing lines".to_string(),
                    ));
                };

                // Check if EOF
                if position >= lines.len() as i64 {
                    // Return empty string at EOF
                    return Ok(Value::from_string(String::new()));
                }

                // Get the line
                let line = lines[position as usize].clone();

                // Advance position
                obj.insert("position".to_string(), Value::Integer(position + 1));

                Ok(line)
            }
            "read" => {
                // Check args
                if !arg_values.is_empty() {
                    return Err(InterpreterError::RuntimeError(
                        "read() takes no arguments".to_string(),
                    ));
                }

                let obj = file_obj
                    .lock()
                    .expect("Mutex poisoned: file object lock is corrupted");

                // Check if closed
                if let Some(Value::Bool(true)) = obj.get("closed") {
                    return Err(InterpreterError::RuntimeError(
                        "Cannot read from closed file".to_string(),
                    ));
                }

                // Get lines array
                let lines = if let Some(Value::Array(lines)) = obj.get("lines") {
                    lines.clone()
                } else {
                    return Err(InterpreterError::RuntimeError(
                        "File object corrupted: missing lines".to_string(),
                    ));
                };

                // Join all lines with newline
                let content: String = lines
                    .iter()
                    .filter_map(|v| {
                        if let Value::String(s) = v {
                            Some(s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                Ok(Value::from_string(content))
            }
            "close" => {
                // Check args
                if !arg_values.is_empty() {
                    return Err(InterpreterError::RuntimeError(
                        "close() takes no arguments".to_string(),
                    ));
                }

                let mut obj = file_obj
                    .lock()
                    .expect("Mutex poisoned: file object lock is corrupted");
                obj.insert("closed".to_string(), Value::Bool(true));
                Ok(Value::Nil)
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Unknown method '{}' on File object",
                method
            ))),
        }
    }

    pub(crate) fn eval_object_method_mut(
        &mut self,
        cell_rc: &Arc<std::sync::Mutex<std::collections::HashMap<String, Value>>>,
        method: &str,
        arg_values: &[Value],
        args_empty: bool,
    ) -> Result<Value, InterpreterError> {
        let instance = cell_rc
            .lock()
            .expect("Mutex poisoned: instance lock is corrupted");
        self.eval_object_method(&instance, method, arg_values, args_empty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn make_interpreter() -> Interpreter {
        Interpreter::new()
    }

    // File method tests
    #[test]
    fn test_file_read_line() {
        let mut interp = make_interpreter();
        let mut file_obj = HashMap::new();
        file_obj.insert("__type".to_string(), Value::from_string("File".to_string()));
        file_obj.insert("position".to_string(), Value::Integer(0));
        file_obj.insert("closed".to_string(), Value::Bool(false));
        file_obj.insert(
            "lines".to_string(),
            Value::Array(Arc::from(vec![
                Value::from_string("line1".to_string()),
                Value::from_string("line2".to_string()),
                Value::from_string("line3".to_string()),
            ])),
        );
        let file_rc = Arc::new(Mutex::new(file_obj));

        // Read first line
        let result = interp.eval_file_method_mut(&file_rc, "read_line", &[]).unwrap();
        assert_eq!(result, Value::from_string("line1".to_string()));

        // Read second line
        let result = interp.eval_file_method_mut(&file_rc, "read_line", &[]).unwrap();
        assert_eq!(result, Value::from_string("line2".to_string()));

        // Read third line
        let result = interp.eval_file_method_mut(&file_rc, "read_line", &[]).unwrap();
        assert_eq!(result, Value::from_string("line3".to_string()));

        // EOF - empty string
        let result = interp.eval_file_method_mut(&file_rc, "read_line", &[]).unwrap();
        assert_eq!(result, Value::from_string("".to_string()));
    }

    #[test]
    fn test_file_read_line_wrong_args() {
        let mut interp = make_interpreter();
        let mut file_obj = HashMap::new();
        file_obj.insert("__type".to_string(), Value::from_string("File".to_string()));
        file_obj.insert("lines".to_string(), Value::Array(Arc::from(vec![])));
        let file_rc = Arc::new(Mutex::new(file_obj));

        let result = interp.eval_file_method_mut(&file_rc, "read_line", &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no arguments"));
    }

    #[test]
    fn test_file_read_line_closed() {
        let mut interp = make_interpreter();
        let mut file_obj = HashMap::new();
        file_obj.insert("__type".to_string(), Value::from_string("File".to_string()));
        file_obj.insert("closed".to_string(), Value::Bool(true));
        file_obj.insert("lines".to_string(), Value::Array(Arc::from(vec![])));
        let file_rc = Arc::new(Mutex::new(file_obj));

        let result = interp.eval_file_method_mut(&file_rc, "read_line", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("closed file"));
    }

    #[test]
    fn test_file_read_line_missing_lines() {
        let mut interp = make_interpreter();
        let mut file_obj = HashMap::new();
        file_obj.insert("__type".to_string(), Value::from_string("File".to_string()));
        file_obj.insert("closed".to_string(), Value::Bool(false));
        // No "lines" field
        let file_rc = Arc::new(Mutex::new(file_obj));

        let result = interp.eval_file_method_mut(&file_rc, "read_line", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("corrupted"));
    }

    #[test]
    fn test_file_read() {
        let mut interp = make_interpreter();
        let mut file_obj = HashMap::new();
        file_obj.insert("__type".to_string(), Value::from_string("File".to_string()));
        file_obj.insert("closed".to_string(), Value::Bool(false));
        file_obj.insert(
            "lines".to_string(),
            Value::Array(Arc::from(vec![
                Value::from_string("line1".to_string()),
                Value::from_string("line2".to_string()),
            ])),
        );
        let file_rc = Arc::new(Mutex::new(file_obj));

        let result = interp.eval_file_method_mut(&file_rc, "read", &[]).unwrap();
        assert_eq!(result, Value::from_string("line1\nline2".to_string()));
    }

    #[test]
    fn test_file_read_wrong_args() {
        let mut interp = make_interpreter();
        let mut file_obj = HashMap::new();
        file_obj.insert("__type".to_string(), Value::from_string("File".to_string()));
        file_obj.insert("lines".to_string(), Value::Array(Arc::from(vec![])));
        let file_rc = Arc::new(Mutex::new(file_obj));

        let result = interp.eval_file_method_mut(&file_rc, "read", &[Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_read_closed() {
        let mut interp = make_interpreter();
        let mut file_obj = HashMap::new();
        file_obj.insert("__type".to_string(), Value::from_string("File".to_string()));
        file_obj.insert("closed".to_string(), Value::Bool(true));
        file_obj.insert("lines".to_string(), Value::Array(Arc::from(vec![])));
        let file_rc = Arc::new(Mutex::new(file_obj));

        let result = interp.eval_file_method_mut(&file_rc, "read", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("closed file"));
    }

    #[test]
    fn test_file_read_missing_lines() {
        let mut interp = make_interpreter();
        let mut file_obj = HashMap::new();
        file_obj.insert("__type".to_string(), Value::from_string("File".to_string()));
        file_obj.insert("closed".to_string(), Value::Bool(false));
        let file_rc = Arc::new(Mutex::new(file_obj));

        let result = interp.eval_file_method_mut(&file_rc, "read", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("corrupted"));
    }

    #[test]
    fn test_file_close() {
        let mut interp = make_interpreter();
        let mut file_obj = HashMap::new();
        file_obj.insert("__type".to_string(), Value::from_string("File".to_string()));
        file_obj.insert("closed".to_string(), Value::Bool(false));
        let file_rc = Arc::new(Mutex::new(file_obj));

        let result = interp.eval_file_method_mut(&file_rc, "close", &[]).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify closed flag is set
        let obj = file_rc.lock().unwrap();
        assert_eq!(obj.get("closed"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_file_close_wrong_args() {
        let mut interp = make_interpreter();
        let file_obj = HashMap::new();
        let file_rc = Arc::new(Mutex::new(file_obj));

        let result = interp.eval_file_method_mut(&file_rc, "close", &[Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_unknown_method() {
        let mut interp = make_interpreter();
        let file_obj = HashMap::new();
        let file_rc = Arc::new(Mutex::new(file_obj));

        let result = interp.eval_file_method_mut(&file_rc, "unknown", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown method"));
    }

    // Object method mut tests - missing type marker error
    #[test]
    fn test_object_method_mut_missing_type() {
        let mut interp = make_interpreter();
        let obj = HashMap::new();
        let obj_rc = Arc::new(Mutex::new(obj));

        let result = interp.eval_object_method_mut(&obj_rc, "test", &[], true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing __type marker"));
    }

    // Actor instance method mut tests
    #[test]
    fn test_actor_instance_method_mut_send_empty() {
        let mut interp = make_interpreter();
        let mut obj = HashMap::new();
        obj.insert("__actor".to_string(), Value::from_string("TestActor".to_string()));
        let obj_rc = Arc::new(Mutex::new(obj));

        let result = interp.eval_actor_instance_method_mut(&obj_rc, "TestActor", "send", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires a message"));
    }

    // Struct instance method with self capture tests - error path
    #[test]
    fn test_struct_instance_method_with_self_capture_not_closure() {
        let mut interp = make_interpreter();
        // Set up a variable that's not a closure
        interp.set_variable("TestStruct::method", Value::Integer(42));

        let instance = HashMap::new();
        let result = interp.eval_struct_instance_method_with_self_capture(
            &instance,
            "TestStruct",
            "method",
            &[],
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a method closure"));
    }

    #[test]
    fn test_struct_instance_method_mut_not_closure() {
        let mut interp = make_interpreter();
        // Set up a variable that's not a closure
        interp.set_variable("TestStruct::method", Value::Integer(42));

        let obj = HashMap::new();
        let obj_rc = Arc::new(Mutex::new(obj));

        let result = interp.eval_struct_instance_method_mut(&obj_rc, "TestStruct", "method", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a method closure"));
    }

    // Class instance method on class tests
    #[test]
    fn test_class_instance_method_on_class_not_found() {
        let mut interp = make_interpreter();
        let fields = Arc::new(std::sync::RwLock::new(HashMap::new()));
        let methods = Arc::new(HashMap::new());

        let result = interp.eval_class_instance_method_on_class(
            "TestClass",
            &fields,
            &methods,
            "unknown_method",
            &[],
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_class_instance_method_on_class_not_closure() {
        let mut interp = make_interpreter();
        let fields = Arc::new(std::sync::RwLock::new(HashMap::new()));
        let mut methods_map = HashMap::new();
        methods_map.insert("bad_method".to_string(), Value::Integer(42)); // Not a closure
        let methods = Arc::new(methods_map);

        let result = interp.eval_class_instance_method_on_class(
            "TestClass",
            &fields,
            &methods,
            "bad_method",
            &[],
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a closure"));
    }

    #[test]
    fn test_class_instance_method_mut_not_class() {
        let mut interp = make_interpreter();
        // Set up a variable that's not a class
        interp.set_variable("NotAClass", Value::Integer(42));

        let obj = HashMap::new();
        let obj_rc = Arc::new(Mutex::new(obj));

        let result = interp.eval_class_instance_method_mut(&obj_rc, "NotAClass", "method", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a class"));
    }
}
