//! Actor instance methods and message processing
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
    /// Evaluates actor instance methods like `send()` and `ask()`.
    ///
    /// This method handles message passing to actors using the `!` (send) and `<?` (ask) operators.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::frontend::parser::Parser;
    /// use ruchy::runtime::interpreter::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// let code = r#"
    ///     actor Counter {
    ///         count: i32 = 0
    ///
    ///         receive {
    ///             Increment => 42
    ///         }
    ///     }
    ///
    ///     fn main() {
    ///         let counter = spawn Counter
    ///         counter ! Increment
    ///         counter
    ///     }
    /// "#;
    ///
    /// let mut parser = Parser::new(code);
    /// let expr = parser.parse().expect("parse should succeed in doctest");
    /// interpreter.eval_expr(&expr).expect("eval_expr should succeed in doctest");
    /// let main_call = Parser::new("main()").parse().expect("parse should succeed in doctest");
    /// let result = interpreter.eval_expr(&main_call).expect("eval_expr should succeed in doctest");
    /// // Actor instance returned
    /// ```
    pub(crate) fn eval_actor_instance_method(
        &mut self,
        instance: &std::collections::HashMap<String, Value>,
        _actor_name: &str,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        match method {
            "send" => {
                // Send a message to the actor (fire-and-forget)
                if arg_values.is_empty() {
                    return Err(InterpreterError::RuntimeError(
                        "send() requires a message argument".to_string(),
                    ));
                }

                // Check if this is an async actor with runtime ID
                if let Some(Value::String(actor_id)) = instance.get("__actor_id") {
                    use crate::runtime::actor_runtime::{ActorMessage, actor_runtime};

                    // Extract message type and data
                    let message = &arg_values[0];
                    let (msg_type, msg_data) =
                        Self::extract_send_message_parts(message);

                    // Convert data to strings for thread safety
                    let str_data: Vec<String> =
                        msg_data.iter().map(|v| format!("{:?}", v)).collect();

                    // Send the message to the actor
                    let actor_msg = ActorMessage {
                        message_type: msg_type,
                        data: str_data,
                    };

                    actor_runtime().send_message(actor_id.as_ref(), actor_msg)?;
                    return Ok(Value::Nil);
                }

                // Synchronous actor - process message immediately
                self.process_actor_message_sync(instance, &arg_values[0])
            }
            "stop" => {
                // Stop the actor
                // In a real actor system, this would terminate the actor's mailbox processing
                Ok(Value::Bool(true))
            }
            "ask" => {
                // Send a message and wait for response
                // For now, we'll process the message synchronously
                if arg_values.is_empty() {
                    return Err(InterpreterError::RuntimeError(
                        "ask() requires a message argument".to_string(),
                    ));
                }

                let message = &arg_values[0];
                self.execute_ask_handler(instance, message)
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Unknown actor method: {}",
                method
            ))),
        }
    }

    /// Process a message for a synchronous (interpreted) actor.
    ///
    /// This method executes the appropriate message handler based on the message type.
    /// Complexity: 9
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::frontend::parser::Parser;
    /// use ruchy::runtime::interpreter::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// let code = r#"
    ///     actor Greeter {
    ///         greeting: String = "Hello"
    ///
    ///         receive {
    ///             Greet(name: String) => {
    ///                 "Hello, World!"
    ///             }
    ///         }
    ///     }
    ///
    ///     fn main() {
    ///         let greeter = spawn Greeter
    ///         greeter ! Greet("Alice")
    ///         greeter
    ///     }
    /// "#;
    ///
    /// let mut parser = Parser::new(code);
    /// let expr = parser.parse().expect("parse should succeed in doctest");
    /// interpreter.eval_expr(&expr).expect("eval_expr should succeed in doctest");
    /// let main_call = Parser::new("main()").parse().expect("parse should succeed in doctest");
    /// let result = interpreter.eval_expr(&main_call);
    /// assert!(result.is_ok());
    /// ```
    pub(crate) fn process_actor_message_sync(
        &mut self,
        instance: &std::collections::HashMap<String, Value>,
        message: &Value,
    ) -> Result<Value, InterpreterError> {
        // Parse the message to extract type and arguments
        let (msg_type, msg_args) = Self::extract_message_type_and_data(message)?;

        // Find the matching handler closure
        let handler_closure = Self::find_handler_closure(instance, &msg_type, "body");
        let Some((params, body, env)) = handler_closure else {
            return Err(InterpreterError::RuntimeError(format!(
                "No handler found for message type: {msg_type}"
            )));
        };

        // Build handler environment
        let mut handler_env = env.borrow().clone(); // ISSUE-119

        // RUNTIME-DEFAULT-PARAMS: Bind message parameters
        Self::bind_params(&mut handler_env, &params, &msg_args);

        // Bind 'self' -- create an object with public fields only
        let self_obj: HashMap<String, Value> = instance
            .iter()
            .filter(|(k, _)| !k.starts_with("__"))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        handler_env.insert("self".to_string(), Value::Object(Arc::new(self_obj)));

        // Execute the handler body
        self.env_stack.push(Rc::new(RefCell::new(handler_env)));
        let result = self.eval_expr(&body);
        self.env_stack.pop();

        result
    }

    /// Process a message for a synchronous (interpreted) actor with mutable state.
    ///
    /// This version accepts `Arc<Mutex<HashMap>>` and passes `ObjectMut` as self to enable mutations.
    /// Complexity: 9
    pub(crate) fn process_actor_message_sync_mut(
        &mut self,
        cell_rc: &Arc<std::sync::Mutex<std::collections::HashMap<String, Value>>>,
        message: &Value,
    ) -> Result<Value, InterpreterError> {
        let instance = cell_rc
            .lock()
            .expect("Mutex poisoned: instance lock is corrupted");

        // Parse the message to extract type and arguments
        let (msg_type, msg_args) = Self::extract_message_type_and_data(message)?;

        // Find matching handler and extract cloned data (before releasing mutex)
        let found = Self::find_handler_closure_with_types(&instance, &msg_type);
        let Some((params_clone, body_clone, env_clone, param_types)) = found else {
            return Err(InterpreterError::RuntimeError(format!(
                "No handler found for message type: {msg_type}"
            )));
        };

        drop(instance); // Release mutex before executing handler

        // Validate parameter types before execution
        if let Some(types) = param_types {
            Self::validate_param_types(&msg_type, &types, &msg_args)?;
        }

        // Build handler environment
        let mut handler_env = env_clone.borrow().clone(); // ISSUE-119
        Self::bind_params(&mut handler_env, &params_clone, &msg_args);

        // CRITICAL: Bind 'self' to ObjectMut (not immutable Object)
        handler_env.insert(
            "self".to_string(),
            Value::ObjectMut(Arc::clone(cell_rc)),
        );

        // Execute the handler body
        self.env_stack.push(Rc::new(RefCell::new(handler_env)));
        let result = self.eval_expr(&body_clone);
        self.env_stack.pop();

        result
    }

    /// Extract message type and data from a send-style message Value.
    fn extract_send_message_parts(message: &Value) -> (String, Vec<Value>) {
        let Value::Object(msg_obj) = message else {
            return ("Message".to_string(), vec![message.clone()]);
        };
        let Some(Value::String(type_str)) = msg_obj.get("__type") else {
            return ("Unknown".to_string(), vec![]);
        };
        if type_str.as_ref() != "Message" {
            return ("Unknown".to_string(), vec![]);
        }
        let msg_type = match msg_obj.get("type") {
            Some(Value::String(s)) => s.to_string(),
            _ => "Unknown".to_string(),
        };
        let msg_data = match msg_obj.get("data") {
            Some(Value::Array(arr)) => arr.to_vec(),
            _ => vec![],
        };
        (msg_type, msg_data)
    }

    /// Execute the `ask` handler lookup and dispatch.
    ///
    /// Extracted from `eval_actor_instance_method` to reduce nesting depth.
    fn execute_ask_handler(
        &mut self,
        instance: &std::collections::HashMap<String, Value>,
        message: &Value,
    ) -> Result<Value, InterpreterError> {
        // Try to extract structured message fields
        let Some((msg_type, data)) = Self::extract_ask_message(message) else {
            return Ok(message.clone());
        };

        // Try to find and execute a matching handler
        if let Some(Value::Array(handler_list)) = instance.get("__handlers") {
            for handler in handler_list.iter() {
                let Value::Object(h) = handler else { continue };
                let Some(Value::String(h_type)) = h.get("message_type") else {
                    continue;
                };
                if h_type.as_ref() != msg_type.as_ref() {
                    continue;
                }
                let Some(Value::Closure { params, body, env }) = h.get("handler") else {
                    continue;
                };

                // Build handler environment
                let mut handler_env = env.borrow().clone(); // ISSUE-119

                // RUNTIME-DEFAULT-PARAMS: Bind message parameters
                for (i, (param_name, _default)) in params.iter().enumerate() {
                    if let Some(value) = data.get(i) {
                        handler_env.insert(param_name.clone(), value.clone());
                    }
                }

                // Bind 'self' to the actor instance
                handler_env.insert(
                    "self".to_string(),
                    Value::Object(Arc::new(instance.clone())),
                );

                // Execute handler body
                self.env_push(handler_env);
                let result = self.eval_expr(body)?;
                self.env_pop();

                return Ok(result);
            }
        }

        // No handler found - return a default response
        Ok(Value::from_string(format!("Received: {}", msg_type.as_ref())))
    }

    /// Extract message type and data from a structured ask message.
    ///
    /// Returns `None` when the message is not a structured Message object.
    fn extract_ask_message(message: &Value) -> Option<(&Arc<str>, &Arc<[Value]>)> {
        let Value::Object(msg_obj) = message else { return None };
        let Some(Value::String(type_str)) = msg_obj.get("__type") else {
            return None;
        };
        if type_str.as_ref() != "Message" {
            return None;
        }
        let Some(Value::String(msg_type)) = msg_obj.get("type") else {
            return None;
        };
        let Some(Value::Array(data)) = msg_obj.get("data") else {
            return None;
        };
        Some((msg_type, data))
    }

    /// Validate parameter types against expected types from handler definition.
    fn validate_param_types(
        msg_type: &str,
        types: &Arc<[Value]>,
        msg_args: &[Value],
    ) -> Result<(), InterpreterError> {
        for (i, expected_type_val) in types.iter().enumerate() {
            let Value::String(expected_type) = expected_type_val else {
                continue;
            };
            let Some(actual_value) = msg_args.get(i) else {
                continue;
            };
            let actual_type = actual_value.type_name();
            let expected_runtime_type = match expected_type.as_ref() {
                "i32" | "i64" | "int" => "integer",
                "f32" | "f64" | "float" => "float",
                "String" | "string" | "str" => "string",
                "bool" => "boolean",
                _ => expected_type.as_ref(),
            };
            if actual_type != expected_runtime_type && expected_runtime_type != "Any" {
                return Err(InterpreterError::RuntimeError(format!(
                    "Type error in message {msg_type}: parameter {i} \
                     expects type '{expected_runtime_type}', got '{actual_type}'"
                )));
            }
        }
        Ok(())
    }

    /// Find a matching handler closure from the __handlers list by message type.
    ///
    /// `body_key` is the key within the handler object that contains the closure
    /// (e.g., "body" for sync handlers, "handler" for ask handlers).
    fn find_handler_closure<'a>(
        instance: &'a std::collections::HashMap<String, Value>,
        msg_type: &str,
        body_key: &str,
    ) -> Option<(
        &'a Vec<(String, Option<Arc<crate::frontend::ast::Expr>>)>,
        &'a Arc<crate::frontend::ast::Expr>,
        &'a Rc<RefCell<HashMap<String, Value>>>,
    )> {
        let Value::Array(handlers) = instance.get("__handlers")? else {
            return None;
        };
        for handler in handlers.iter() {
            let Value::Object(handler_obj) = handler else { continue };
            let Some(Value::String(handler_type)) = handler_obj.get("message_type") else {
                continue;
            };
            if handler_type.as_ref() != msg_type {
                continue;
            }
            let Some(Value::Closure { params, body, env }) = handler_obj.get(body_key) else {
                continue;
            };
            return Some((params, body, env));
        }
        None
    }

    /// Find a handler closure and clone its data (for use with mutable actors where
    /// the mutex guard must be dropped before handler execution).
    ///
    /// Also extracts optional param_types for validation.
    #[allow(clippy::type_complexity)]
    fn find_handler_closure_with_types(
        instance: &std::collections::HashMap<String, Value>,
        msg_type: &str,
    ) -> Option<(
        Vec<(String, Option<Arc<crate::frontend::ast::Expr>>)>,
        Arc<crate::frontend::ast::Expr>,
        Rc<RefCell<HashMap<String, Value>>>,
        Option<Arc<[Value]>>,
    )> {
        let Value::Array(handlers) = instance.get("__handlers")? else {
            return None;
        };
        for handler in handlers.iter() {
            let Value::Object(handler_obj) = handler else { continue };
            let Some(Value::String(handler_type)) = handler_obj.get("message_type") else {
                continue;
            };
            if handler_type.as_ref() != msg_type {
                continue;
            }
            let Some(Value::Closure { params, body, env }) = handler_obj.get("body") else {
                continue;
            };
            let param_types = match handler_obj.get("param_types") {
                Some(Value::Array(types)) => Some(types.clone()),
                _ => None,
            };
            return Some((params.clone(), body.clone(), env.clone(), param_types));
        }
        None
    }

    /// Bind message parameters into a handler environment.
    fn bind_params(
        env: &mut HashMap<String, Value>,
        params: &[(String, Option<Arc<crate::frontend::ast::Expr>>)],
        args: &[Value],
    ) {
        for (i, (param_name, _default)) in params.iter().enumerate() {
            if let Some(value) = args.get(i) {
                env.insert(param_name.clone(), value.clone());
            }
        }
    }

    pub(crate) fn eval_struct_instance_method(
        &mut self,
        instance: &std::collections::HashMap<String, Value>,
        struct_name: &str,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
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
                let mut new_env = env.borrow().clone(); // ISSUE-119: Borrow from RefCell

                // RUNTIME-DEFAULT-PARAMS: Bind self parameter (first parameter)
                // RUNTIME-094: Bind as Value::Struct to preserve struct type for nested method calls
                if let Some((self_param_name, _default_value)) = params.first() {
                    new_env.insert(
                        self_param_name.clone(),
                        Value::Struct {
                            name: struct_name.to_string(),
                            fields: std::sync::Arc::new(instance.clone()),
                        },
                    );
                }

                // RUNTIME-DEFAULT-PARAMS: Bind other parameters
                for (i, arg_value) in arg_values.iter().enumerate() {
                    if let Some((param_name, _default_value)) = params.get(i + 1) {
                        // +1 to skip self
                        new_env.insert(param_name.clone(), arg_value.clone());
                    }
                }

                // Execute method body with new environment
                self.env_stack.push(Rc::new(RefCell::new(new_env))); // ISSUE-119: Wrap in Rc<RefCell>
                let result = self.eval_expr(&body);
                self.env_stack.pop();

                result
            } else {
                Err(InterpreterError::RuntimeError(format!(
                    "Found {} but it's not a method closure",
                    qualified_method_name
                )))
            }
        } else {
            // Fall back to generic method handling
            self.eval_generic_method(
                &Value::Object(std::sync::Arc::new(instance.clone())),
                method,
                arg_values.is_empty(),
            )
        }
    }

    pub(crate) fn eval_object_method(
        &self,
        obj: &std::collections::HashMap<String, Value>,
        method: &str,
        arg_values: &[Value],
        args_empty: bool,
    ) -> Result<Value, InterpreterError> {
        use crate::runtime::eval_method_dispatch;
        eval_method_dispatch::eval_method_call(
            &Value::Object(std::sync::Arc::new(obj.clone())),
            method,
            arg_values,
            args_empty,
            |_receiver, _args| {
                Err(InterpreterError::RuntimeError(
                    "Function call not implemented in actor context".to_string(),
                ))
            },
            |_receiver, _args| {
                Err(InterpreterError::RuntimeError(
                    "DataFrame filter not implemented in actor context".to_string(),
                ))
            },
            |_expr, _columns, _index| {
                Err(InterpreterError::RuntimeError(
                    "Column context not implemented in actor context".to_string(),
                ))
            },
        )
    }
}

#[cfg(test)]
#[path = "interpreter_methods_actor_tests.rs"]
mod tests;
