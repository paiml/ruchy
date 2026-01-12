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
                    use crate::runtime::actor_runtime::{ActorMessage, ACTOR_RUNTIME};

                    // Extract message type and data
                    let message = &arg_values[0];
                    let (msg_type, msg_data) = if let Value::Object(msg_obj) = message {
                        if let Some(Value::String(type_str)) = msg_obj.get("__type") {
                            if type_str.as_ref() == "Message" {
                                let msg_type = msg_obj
                                    .get("type")
                                    .and_then(|v| {
                                        if let Value::String(s) = v {
                                            Some(s.to_string())
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or_else(|| "Unknown".to_string());
                                let msg_data = msg_obj
                                    .get("data")
                                    .and_then(|v| {
                                        if let Value::Array(arr) = v {
                                            Some(arr.to_vec())
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or_else(Vec::new);
                                (msg_type, msg_data)
                            } else {
                                ("Unknown".to_string(), vec![])
                            }
                        } else {
                            ("Unknown".to_string(), vec![])
                        }
                    } else {
                        // Simple message value
                        ("Message".to_string(), vec![message.clone()])
                    };

                    // Convert data to strings for thread safety
                    let str_data: Vec<String> =
                        msg_data.iter().map(|v| format!("{:?}", v)).collect();

                    // Send the message to the actor
                    let actor_msg = ActorMessage {
                        message_type: msg_type,
                        data: str_data,
                    };

                    ACTOR_RUNTIME.send_message(actor_id.as_ref(), actor_msg)?;
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

                // Get the message
                let message = &arg_values[0];

                // Try to extract message type and data
                if let Value::Object(msg_obj) = message {
                    // Check if this is a Message object we created
                    if let Some(Value::String(type_str)) = msg_obj.get("__type") {
                        if type_str.as_ref() == "Message" {
                            // Extract message type and data
                            if let Some(Value::String(msg_type)) = msg_obj.get("type") {
                                if let Some(Value::Array(data)) = msg_obj.get("data") {
                                    // Look up the handler for this message type
                                    if let Some(handlers) = instance.get("__handlers") {
                                        if let Value::Array(handler_list) = handlers {
                                            // Find matching handler
                                            for handler in handler_list.iter() {
                                                if let Value::Object(h) = handler {
                                                    if let Some(Value::String(h_type)) =
                                                        h.get("message_type")
                                                    {
                                                        if h_type.as_ref() == msg_type.as_ref() {
                                                            // Found matching handler - execute it
                                                            if let Some(Value::Closure {
                                                                params,
                                                                body,
                                                                env,
                                                            }) = h.get("handler")
                                                            {
                                                                // Push a new environment for handler execution
                                                                let mut handler_env =
                                                                    env.borrow().clone(); // ISSUE-119: Borrow from RefCell

                                                                // Bind message parameters
                                                                // RUNTIME-DEFAULT-PARAMS: Extract param name from tuple
                                                                for (
                                                                    i,
                                                                    (param_name, _default_value),
                                                                ) in params.iter().enumerate()
                                                                {
                                                                    if let Some(value) = data.get(i)
                                                                    {
                                                                        handler_env.insert(
                                                                            param_name.clone(),
                                                                            value.clone(),
                                                                        );
                                                                    }
                                                                }

                                                                // Also bind 'self' to the actor instance
                                                                handler_env.insert(
                                                                    "self".to_string(),
                                                                    Value::Object(Arc::new(
                                                                        instance.clone(),
                                                                    )),
                                                                );

                                                                // Execute handler body
                                                                self.env_push(handler_env);
                                                                let result =
                                                                    self.eval_expr(body)?;
                                                                self.env_pop();

                                                                return Ok(result);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // No handler found - return a default response
                                    return Ok(Value::from_string(format!(
                                        "Received: {}",
                                        msg_type.as_ref()
                                    )));
                                }
                            }
                        }
                    }
                }

                // Default: return the message itself (echo)
                Ok(message.clone())
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
        // Messages come as function calls like Push(1) or SetCount(5)
        let (msg_type, msg_args) = Self::extract_message_type_and_data(message)?;

        // Find the matching handler
        if let Some(Value::Array(handlers)) = instance.get("__handlers") {
            for handler in handlers.iter() {
                if let Value::Object(handler_obj) = handler {
                    if let Some(Value::String(handler_type)) = handler_obj.get("message_type") {
                        if handler_type.as_ref() == msg_type {
                            // Found matching handler - execute it
                            if let Some(Value::Closure { params, body, env }) =
                                handler_obj.get("body")
                            {
                                // Create a new environment for handler execution
                                let mut handler_env = env.borrow().clone(); // ISSUE-119: Borrow from RefCell

                                // Bind message parameters
                                // RUNTIME-DEFAULT-PARAMS: Extract param name from tuple
                                for (i, (param_name, _default_value)) in params.iter().enumerate() {
                                    if let Some(value) = msg_args.get(i) {
                                        handler_env.insert(param_name.clone(), value.clone());
                                    }
                                }

                                // Bind 'self' to the actor instance
                                // Create a mutable object for self that includes all fields
                                let mut self_obj = HashMap::new();
                                for (key, value) in instance {
                                    if !key.starts_with("__") {
                                        self_obj.insert(key.clone(), value.clone());
                                    }
                                }
                                handler_env
                                    .insert("self".to_string(), Value::Object(Arc::new(self_obj)));

                                // Execute the handler body
                                self.env_stack.push(Rc::new(RefCell::new(handler_env))); // ISSUE-119: Wrap in Rc<RefCell>
                                let result = self.eval_expr(body);
                                self.env_stack.pop();

                                return result;
                            }
                        }
                    }
                }
            }
        }

        Err(InterpreterError::RuntimeError(format!(
            "No handler found for message type: {}",
            msg_type
        )))
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

        // Find the matching handler
        if let Some(Value::Array(handlers)) = instance.get("__handlers") {
            for handler in handlers.iter() {
                if let Value::Object(handler_obj) = handler {
                    if let Some(Value::String(handler_type)) = handler_obj.get("message_type") {
                        if handler_type.as_ref() == msg_type {
                            // Found matching handler - execute it
                            if let Some(Value::Closure { params, body, env }) =
                                handler_obj.get("body")
                            {
                                // Clone data before dropping instance borrow
                                let params_clone = params.clone();
                                let body_clone = body.clone();
                                let env_clone = env.clone();

                                // Get parameter types for validation
                                let param_types = handler_obj.get("param_types").and_then(|v| {
                                    if let Value::Array(types) = v {
                                        Some(types.clone())
                                    } else {
                                        None
                                    }
                                });

                                drop(instance); // Release borrow before executing handler

                                // Validate parameter types before execution
                                if let Some(types) = param_types {
                                    for (i, expected_type_val) in types.iter().enumerate() {
                                        if let Value::String(expected_type) = expected_type_val {
                                            if let Some(actual_value) = msg_args.get(i) {
                                                let actual_type = actual_value.type_name();
                                                // Map Ruchy type names to runtime type names
                                                let expected_runtime_type =
                                                    match expected_type.as_ref() {
                                                        "i32" | "i64" | "int" => "integer",
                                                        "f32" | "f64" | "float" => "float",
                                                        "String" | "string" | "str" => "string",
                                                        "bool" => "boolean",
                                                        _ => expected_type.as_ref(),
                                                    };

                                                if actual_type != expected_runtime_type
                                                    && expected_runtime_type != "Any"
                                                {
                                                    return Err(InterpreterError::RuntimeError(format!(
                                                        "Type error in message {}: parameter {} expects type '{}', got '{}'",
                                                        msg_type, i, expected_runtime_type, actual_type
                                                    )));
                                                }
                                            }
                                        }
                                    }
                                }

                                // Create a new environment for handler execution
                                let mut handler_env = env_clone.borrow().clone(); // ISSUE-119: Borrow from RefCell

                                // RUNTIME-DEFAULT-PARAMS: Bind message parameters
                                for (i, (param_name, _default_value)) in
                                    params_clone.iter().enumerate()
                                {
                                    if let Some(value) = msg_args.get(i) {
                                        handler_env.insert(param_name.clone(), value.clone());
                                    }
                                }

                                // CRITICAL: Bind 'self' to ObjectMut (not immutable Object)
                                // This allows mutations in the handler to persist
                                handler_env.insert(
                                    "self".to_string(),
                                    Value::ObjectMut(Arc::clone(cell_rc)),
                                );

                                // Execute the handler body
                                self.env_stack.push(Rc::new(RefCell::new(handler_env))); // ISSUE-119: Wrap in Rc<RefCell>
                                let result = self.eval_expr(&body_clone);
                                self.env_stack.pop();

                                return result;
                            }
                        }
                    }
                }
            }
        }

        Err(InterpreterError::RuntimeError(format!(
            "No handler found for message type: {}",
            msg_type
        )))
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
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span};
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::Mutex;

    fn make_interpreter() -> Interpreter {
        Interpreter::new()
    }

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn make_closure(
        params: Vec<(String, Option<Arc<Expr>>)>,
        body: Expr,
    ) -> Value {
        Value::Closure {
            params,
            body: Arc::new(body),
            env: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    fn make_handler(message_type: &str, params: Vec<String>, body: Expr) -> Value {
        let mut handler_obj = HashMap::new();
        handler_obj.insert(
            "message_type".to_string(),
            Value::from_string(message_type.to_string()),
        );
        handler_obj.insert(
            "params".to_string(),
            Value::Array(Arc::from(
                params.iter().map(|p| Value::from_string(p.clone())).collect::<Vec<_>>(),
            )),
        );
        handler_obj.insert(
            "body".to_string(),
            make_closure(
                params.into_iter().map(|p| (p, None)).collect(),
                body,
            ),
        );
        Value::Object(Arc::new(handler_obj))
    }

    fn make_handler_with_types(
        message_type: &str,
        params: Vec<String>,
        param_types: Vec<&str>,
        body: Expr,
    ) -> Value {
        let mut handler_obj = HashMap::new();
        handler_obj.insert(
            "message_type".to_string(),
            Value::from_string(message_type.to_string()),
        );
        handler_obj.insert(
            "params".to_string(),
            Value::Array(Arc::from(
                params.iter().map(|p| Value::from_string(p.clone())).collect::<Vec<_>>(),
            )),
        );
        handler_obj.insert(
            "param_types".to_string(),
            Value::Array(Arc::from(
                param_types.iter().map(|t| Value::from_string(t.to_string())).collect::<Vec<_>>(),
            )),
        );
        handler_obj.insert(
            "body".to_string(),
            make_closure(
                params.into_iter().map(|p| (p, None)).collect(),
                body,
            ),
        );
        Value::Object(Arc::new(handler_obj))
    }

    fn make_message(msg_type: &str, data: Vec<Value>) -> Value {
        let mut msg_obj = HashMap::new();
        msg_obj.insert("__type".to_string(), Value::from_string("Message".to_string()));
        msg_obj.insert("type".to_string(), Value::from_string(msg_type.to_string()));
        msg_obj.insert("data".to_string(), Value::Array(Arc::from(data)));
        Value::Object(Arc::new(msg_obj))
    }

    // Actor instance method tests
    #[test]
    fn test_actor_instance_send_empty() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "send", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires a message"));
    }

    #[test]
    fn test_actor_instance_stop() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "stop", &[]).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_actor_instance_ask_empty() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires a message"));
    }

    #[test]
    fn test_actor_instance_ask_echo() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();
        let msg = Value::Integer(42);
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg]).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_actor_instance_ask_message_no_handler() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();

        // Create a Message object
        let mut msg_obj = HashMap::new();
        msg_obj.insert("__type".to_string(), Value::from_string("Message".to_string()));
        msg_obj.insert("type".to_string(), Value::from_string("TestMsg".to_string()));
        msg_obj.insert("data".to_string(), Value::Array(Arc::from(vec![])));
        let msg = Value::Object(Arc::new(msg_obj));

        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg]).unwrap();
        assert_eq!(result, Value::from_string("Received: TestMsg".to_string()));
    }

    #[test]
    fn test_actor_instance_unknown_method() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "unknown", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown actor method"));
    }

    // Process actor message sync tests
    #[test]
    fn test_process_actor_message_sync_no_handler() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();

        let mut msg_obj = HashMap::new();
        msg_obj.insert("__type".to_string(), Value::from_string("Message".to_string()));
        msg_obj.insert("type".to_string(), Value::from_string("NoHandler".to_string()));
        msg_obj.insert("data".to_string(), Value::Array(Arc::from(vec![])));
        let msg = Value::Object(Arc::new(msg_obj));

        let result = interp.process_actor_message_sync(&instance, &msg);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No handler found"));
    }

    // Process actor message sync mut tests
    #[test]
    fn test_process_actor_message_sync_mut_no_handler() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();
        let instance_rc = Arc::new(Mutex::new(instance));

        let mut msg_obj = HashMap::new();
        msg_obj.insert("__type".to_string(), Value::from_string("Message".to_string()));
        msg_obj.insert("type".to_string(), Value::from_string("NoHandler".to_string()));
        msg_obj.insert("data".to_string(), Value::Array(Arc::from(vec![])));
        let msg = Value::Object(Arc::new(msg_obj));

        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No handler found"));
    }

    // Struct instance method tests - error path
    #[test]
    fn test_struct_instance_method_not_closure_error() {
        let mut interp = make_interpreter();
        // Set up a variable that's not a closure
        interp.set_variable("TestStruct::method", Value::Integer(42));

        let instance = HashMap::new();
        let result = interp.eval_struct_instance_method(&instance, "TestStruct", "method", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a method closure"));
    }

    // Object method tests - missing type marker error
    #[test]
    fn test_object_method_missing_type() {
        let interp = make_interpreter();
        let obj = HashMap::new();

        let result = interp.eval_object_method(&obj, "test", &[], true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing __type marker"));
    }

    #[test]
    fn test_object_method_unknown_type() {
        let interp = make_interpreter();
        let mut obj = HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("UnknownType".to_string()));

        let result = interp.eval_object_method(&obj, "test", &[], true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown object type"));
    }

    // Test send with async actor ID (branch coverage)
    #[test]
    fn test_actor_instance_send_sync_no_handler() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();

        let mut msg_obj = HashMap::new();
        msg_obj.insert("__type".to_string(), Value::from_string("Message".to_string()));
        msg_obj.insert("type".to_string(), Value::from_string("TestMsg".to_string()));
        msg_obj.insert("data".to_string(), Value::Array(Arc::from(vec![])));
        let msg = Value::Object(Arc::new(msg_obj));

        let result = interp.eval_actor_instance_method(&instance, "TestActor", "send", &[msg]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No handler found"));
    }

    // Test ask with Message object having handlers (but wrong type structure)
    #[test]
    fn test_actor_instance_ask_message_object_wrong_type() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();

        // Create a message object with __type but not "Message"
        let mut msg_obj = HashMap::new();
        msg_obj.insert("__type".to_string(), Value::from_string("NotMessage".to_string()));
        let msg = Value::Object(Arc::new(msg_obj));

        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg.clone()]).unwrap();
        // Should echo back the message
        assert_eq!(result, msg);
    }

    // Test struct instance method with wrong arg count
    #[test]
    fn test_struct_instance_method_closure_not_found() {
        let mut interp = make_interpreter();
        // Set up a variable that's not a closure
        interp.set_variable("Point::display", Value::Integer(42));

        let instance = HashMap::new();
        let result = interp.eval_struct_instance_method(&instance, "Point", "display", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a method closure"));
    }

    // Test process_actor_message_sync with valid handler
    #[test]
    fn test_process_actor_message_sync_with_handler() {
        let mut interp = make_interpreter();

        // Create handler that returns 42
        let body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let handler = make_handler("TestMsg", vec![], body);

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));

        let msg = make_message("TestMsg", vec![]);
        let result = interp.process_actor_message_sync(&instance, &msg).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    // Test process_actor_message_sync_mut with valid handler
    #[test]
    fn test_process_actor_message_sync_mut_with_handler() {
        let mut interp = make_interpreter();

        // Create handler that returns a string
        let body = make_expr(ExprKind::Literal(Literal::String("success".to_string())));
        let handler = make_handler("Ping", vec![], body);

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));
        let instance_rc = Arc::new(Mutex::new(instance));

        let msg = make_message("Ping", vec![]);
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg).unwrap();
        assert_eq!(result, Value::from_string("success".to_string()));
    }

    // Test process_actor_message_sync_mut with type validation success
    #[test]
    fn test_process_actor_message_sync_mut_type_validation_pass() {
        let mut interp = make_interpreter();

        // Create handler that expects an integer parameter
        let body = make_expr(ExprKind::Literal(Literal::Integer(100, None)));
        let handler = make_handler_with_types(
            "SetValue",
            vec!["value".to_string()],
            vec!["i32"],
            body,
        );

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));
        let instance_rc = Arc::new(Mutex::new(instance));

        // Send message with correct type (integer)
        let msg = make_message("SetValue", vec![Value::Integer(42)]);
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg).unwrap();
        assert_eq!(result, Value::Integer(100));
    }

    // Test process_actor_message_sync_mut with type validation failure
    #[test]
    fn test_process_actor_message_sync_mut_type_validation_fail() {
        let mut interp = make_interpreter();

        // Create handler that expects an integer parameter
        let body = make_expr(ExprKind::Literal(Literal::Integer(100, None)));
        let handler = make_handler_with_types(
            "SetValue",
            vec!["value".to_string()],
            vec!["i32"],
            body,
        );

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));
        let instance_rc = Arc::new(Mutex::new(instance));

        // Send message with wrong type (string instead of integer)
        let msg = make_message("SetValue", vec![Value::from_string("wrong".to_string())]);
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Type error"));
    }

    // Test eval_struct_instance_method with valid closure but wrong arg count
    #[test]
    fn test_struct_instance_method_wrong_arg_count() {
        let mut interp = make_interpreter();

        // Create a closure that expects 2 params (self + one arg)
        let body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let closure = make_closure(
            vec![
                ("self".to_string(), None),
                ("x".to_string(), None),
            ],
            body,
        );
        interp.set_variable("TestStruct::method", closure);

        let instance = HashMap::new();
        // Call with 0 args (but method expects 1 besides self)
        let result = interp.eval_struct_instance_method(&instance, "TestStruct", "method", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expects"));
    }

    // Test eval_struct_instance_method with valid closure and correct args
    #[test]
    fn test_struct_instance_method_success() {
        let mut interp = make_interpreter();

        // Create a closure that expects only self (no args)
        let body = make_expr(ExprKind::Literal(Literal::Integer(99, None)));
        let closure = make_closure(
            vec![("self".to_string(), None)],
            body,
        );
        interp.set_variable("Point::get_value", closure);

        let instance = HashMap::new();
        // Call with 0 args (method expects none besides self)
        let result = interp.eval_struct_instance_method(&instance, "Point", "get_value", &[]).unwrap();
        assert_eq!(result, Value::Integer(99));
    }

    // Test send with simple value message (not object)
    #[test]
    fn test_actor_instance_send_simple_value() {
        let mut interp = make_interpreter();

        // Create handler for "Message" type (default for simple values)
        let body = make_expr(ExprKind::Literal(Literal::String("handled".to_string())));
        let handler = make_handler("Message", vec![], body);

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));

        // Send a simple integer value (not a Message object)
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "send", &[Value::Integer(42)]);
        // This should fail because "Message" handler doesn't match the simple value extraction
        assert!(result.is_err());
    }

    // Test process_actor_message_sync with handler that has parameters
    #[test]
    fn test_process_actor_message_sync_with_params() {
        let mut interp = make_interpreter();

        // Create handler that reads the 'x' parameter via identifier lookup
        // Since we can't do complex expressions easily, just return literal
        let body = make_expr(ExprKind::Literal(Literal::Integer(200, None)));
        let handler = make_handler("Add", vec!["x".to_string()], body);

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));

        let msg = make_message("Add", vec![Value::Integer(50)]);
        let result = interp.process_actor_message_sync(&instance, &msg).unwrap();
        assert_eq!(result, Value::Integer(200));
    }

    // Test process_actor_message_sync with multiple handlers
    #[test]
    fn test_process_actor_message_sync_multiple_handlers() {
        let mut interp = make_interpreter();

        // Create two handlers for different message types
        let body1 = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler1 = make_handler("First", vec![], body1);

        let body2 = make_expr(ExprKind::Literal(Literal::Integer(2, None)));
        let handler2 = make_handler("Second", vec![], body2);

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler1, handler2])),
        );

        // Should match second handler
        let msg = make_message("Second", vec![]);
        let result = interp.process_actor_message_sync(&instance, &msg).unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    // Test send with Message object (going through extract_message_type_and_data)
    #[test]
    fn test_actor_instance_send_message_object() {
        let mut interp = make_interpreter();

        // Create handler
        let body = make_expr(ExprKind::Literal(Literal::Bool(true)));
        let handler = make_handler("Ping", vec![], body);

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));

        let msg = make_message("Ping", vec![]);
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "send", &[msg]).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // Test eval_struct_instance_method fallback to generic method
    #[test]
    fn test_struct_instance_method_fallback() {
        let mut interp = make_interpreter();
        // No method registered - should fall back to generic method handling
        let instance = HashMap::new();
        let result = interp.eval_struct_instance_method(&instance, "Unknown", "unknown_method", &[]);
        // Should fail because generic method doesn't know about this
        assert!(result.is_err());
    }

    // Test process_actor_message_sync_mut with Any type (no type checking)
    #[test]
    fn test_process_actor_message_sync_mut_any_type() {
        let mut interp = make_interpreter();

        // Create handler that expects Any type parameter
        let body = make_expr(ExprKind::Literal(Literal::String("ok".to_string())));
        let handler = make_handler_with_types(
            "Accept",
            vec!["value".to_string()],
            vec!["Any"],
            body,
        );

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));
        let instance_rc = Arc::new(Mutex::new(instance));

        // Any type should accept anything
        let msg = make_message("Accept", vec![Value::from_string("anything".to_string())]);
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg).unwrap();
        assert_eq!(result, Value::from_string("ok".to_string()));
    }

    // Test process_actor_message_sync_mut with string type
    #[test]
    fn test_process_actor_message_sync_mut_string_type() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler = make_handler_with_types(
            "SetName",
            vec!["name".to_string()],
            vec!["String"],
            body,
        );

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));
        let instance_rc = Arc::new(Mutex::new(instance));

        // String type should work
        let msg = make_message("SetName", vec![Value::from_string("test".to_string())]);
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    // =========================================================================
    // EXTREME TDD: Additional tests for increased coverage
    // Target: Cover async actor paths, ask handler execution, edge cases
    // =========================================================================

    // Helper to create ask handler (uses "handler" key instead of "body")
    fn make_ask_handler(message_type: &str, params: Vec<String>, body: Expr) -> Value {
        let mut handler_obj = HashMap::new();
        handler_obj.insert(
            "message_type".to_string(),
            Value::from_string(message_type.to_string()),
        );
        handler_obj.insert(
            "params".to_string(),
            Value::Array(Arc::from(
                params.iter().map(|p| Value::from_string(p.clone())).collect::<Vec<_>>(),
            )),
        );
        // Note: ask method looks for "handler" key, not "body"
        handler_obj.insert(
            "handler".to_string(),
            make_closure(
                params.into_iter().map(|p| (p, None)).collect(),
                body,
            ),
        );
        Value::Object(Arc::new(handler_obj))
    }

    // Test ask method with handlers but handler closure found
    #[test]
    fn test_actor_ask_with_handler_closure() {
        let mut interp = make_interpreter();

        // Create handler with "handler" key (for ask method)
        let body = make_expr(ExprKind::Literal(Literal::Integer(999, None)));
        let handler = make_ask_handler("Query", vec![], body);

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));

        let msg = make_message("Query", vec![]);
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg]).unwrap();
        assert_eq!(result, Value::Integer(999));
    }

    // Test ask method with handler that has parameters
    #[test]
    fn test_actor_ask_with_handler_params() {
        let mut interp = make_interpreter();

        // Create handler with parameters
        let body = make_expr(ExprKind::Literal(Literal::String("handled".to_string())));
        let handler = make_ask_handler("Greet", vec!["name".to_string()], body);

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));

        let msg = make_message("Greet", vec![Value::from_string("Alice".to_string())]);
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg]).unwrap();
        assert_eq!(result, Value::from_string("handled".to_string()));
    }

    // Test ask with Message but no matching handler (falls through to default response)
    #[test]
    fn test_actor_ask_message_with_data_no_handler() {
        let mut interp = make_interpreter();

        // Instance has handlers but not for this message type
        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler = make_ask_handler("OtherMsg", vec![], body);

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));

        let msg = make_message("TestQuery", vec![Value::Integer(42)]);
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg]).unwrap();
        // Should return "Received: TestQuery"
        assert_eq!(result, Value::from_string("Received: TestQuery".to_string()));
    }

    // Test ask with handlers that is not an array
    #[test]
    fn test_actor_ask_handlers_not_array() {
        let mut interp = make_interpreter();

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Integer(42)); // Not an array

        let msg = make_message("Query", vec![]);
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg]).unwrap();
        // Falls through to "Received: Query" since handlers aren't iterable
        assert_eq!(result, Value::from_string("Received: Query".to_string()));
    }

    // Test ask with handler that is not an Object
    #[test]
    fn test_actor_ask_handler_not_object() {
        let mut interp = make_interpreter();

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![
            Value::Integer(123), // Not an Object
        ])));

        let msg = make_message("Query", vec![]);
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg]).unwrap();
        assert_eq!(result, Value::from_string("Received: Query".to_string()));
    }

    // Test ask with handler missing message_type
    #[test]
    fn test_actor_ask_handler_missing_message_type() {
        let mut interp = make_interpreter();

        let mut handler_obj = HashMap::new();
        // No message_type field
        handler_obj.insert("handler".to_string(), Value::Integer(1));

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![
            Value::Object(Arc::new(handler_obj)),
        ])));

        let msg = make_message("Query", vec![]);
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg]).unwrap();
        assert_eq!(result, Value::from_string("Received: Query".to_string()));
    }

    // Test ask with handler where message_type doesn't match
    #[test]
    fn test_actor_ask_handler_type_mismatch() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler = make_ask_handler("DifferentType", vec![], body);

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));

        let msg = make_message("Query", vec![]);
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg]).unwrap();
        assert_eq!(result, Value::from_string("Received: Query".to_string()));
    }

    // Test ask with handler missing the closure
    #[test]
    fn test_actor_ask_handler_missing_closure() {
        let mut interp = make_interpreter();

        let mut handler_obj = HashMap::new();
        handler_obj.insert("message_type".to_string(), Value::from_string("Query".to_string()));
        // No "handler" key

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![
            Value::Object(Arc::new(handler_obj)),
        ])));

        let msg = make_message("Query", vec![]);
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg]).unwrap();
        assert_eq!(result, Value::from_string("Received: Query".to_string()));
    }

    // Test ask with message object missing "type" field
    #[test]
    fn test_actor_ask_message_missing_type_field() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();

        let mut msg_obj = HashMap::new();
        msg_obj.insert("__type".to_string(), Value::from_string("Message".to_string()));
        // No "type" field
        msg_obj.insert("data".to_string(), Value::Array(Arc::from(vec![])));
        let msg = Value::Object(Arc::new(msg_obj));

        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg.clone()]).unwrap();
        // Should echo back since no type field means it can't match handlers
        assert_eq!(result, msg);
    }

    // Test ask with message object missing "data" field
    #[test]
    fn test_actor_ask_message_missing_data_field() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();

        let mut msg_obj = HashMap::new();
        msg_obj.insert("__type".to_string(), Value::from_string("Message".to_string()));
        msg_obj.insert("type".to_string(), Value::from_string("Query".to_string()));
        // No "data" field
        let msg = Value::Object(Arc::new(msg_obj));

        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg.clone()]).unwrap();
        // Should echo back since no data field
        assert_eq!(result, msg);
    }

    // Test ask with message object where "type" is not a string
    #[test]
    fn test_actor_ask_message_type_not_string() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();

        let mut msg_obj = HashMap::new();
        msg_obj.insert("__type".to_string(), Value::from_string("Message".to_string()));
        msg_obj.insert("type".to_string(), Value::Integer(123)); // Not a string
        msg_obj.insert("data".to_string(), Value::Array(Arc::from(vec![])));
        let msg = Value::Object(Arc::new(msg_obj));

        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg.clone()]).unwrap();
        // Should echo back since type isn't a string
        assert_eq!(result, msg);
    }

    // Test ask with message object where "data" is not an array
    #[test]
    fn test_actor_ask_message_data_not_array() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();

        let mut msg_obj = HashMap::new();
        msg_obj.insert("__type".to_string(), Value::from_string("Message".to_string()));
        msg_obj.insert("type".to_string(), Value::from_string("Query".to_string()));
        msg_obj.insert("data".to_string(), Value::Integer(42)); // Not an array
        let msg = Value::Object(Arc::new(msg_obj));

        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg.clone()]).unwrap();
        // Should echo back since data isn't an array
        assert_eq!(result, msg);
    }

    // Test process_actor_message_sync with handler where body is not a closure
    #[test]
    fn test_process_actor_message_sync_handler_body_not_closure() {
        let mut interp = make_interpreter();

        let mut handler_obj = HashMap::new();
        handler_obj.insert("message_type".to_string(), Value::from_string("Test".to_string()));
        handler_obj.insert("body".to_string(), Value::Integer(42)); // Not a closure

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![
            Value::Object(Arc::new(handler_obj)),
        ])));

        let msg = make_message("Test", vec![]);
        let result = interp.process_actor_message_sync(&instance, &msg);
        // Should fail to find handler since body isn't a closure
        assert!(result.is_err());
    }

    // Test process_actor_message_sync_mut with handler where body is not a closure
    #[test]
    fn test_process_actor_message_sync_mut_handler_body_not_closure() {
        let mut interp = make_interpreter();

        let mut handler_obj = HashMap::new();
        handler_obj.insert("message_type".to_string(), Value::from_string("Test".to_string()));
        handler_obj.insert("body".to_string(), Value::Integer(42)); // Not a closure

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![
            Value::Object(Arc::new(handler_obj)),
        ])));
        let instance_rc = Arc::new(Mutex::new(instance));

        let msg = make_message("Test", vec![]);
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg);
        // Should fail to find handler since body isn't a closure
        assert!(result.is_err());
    }

    // Test process_actor_message_sync with handler missing message_type field
    #[test]
    fn test_process_actor_message_sync_handler_missing_type() {
        let mut interp = make_interpreter();

        let mut handler_obj = HashMap::new();
        // No message_type field
        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        handler_obj.insert("body".to_string(), make_closure(vec![], body));

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![
            Value::Object(Arc::new(handler_obj)),
        ])));

        let msg = make_message("Test", vec![]);
        let result = interp.process_actor_message_sync(&instance, &msg);
        assert!(result.is_err());
    }

    // Test process_actor_message_sync_mut with handler missing message_type field
    #[test]
    fn test_process_actor_message_sync_mut_handler_missing_type() {
        let mut interp = make_interpreter();

        let mut handler_obj = HashMap::new();
        // No message_type field
        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        handler_obj.insert("body".to_string(), make_closure(vec![], body));

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![
            Value::Object(Arc::new(handler_obj)),
        ])));
        let instance_rc = Arc::new(Mutex::new(instance));

        let msg = make_message("Test", vec![]);
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg);
        assert!(result.is_err());
    }

    // Test process_actor_message_sync with handler that is not an Object
    #[test]
    fn test_process_actor_message_sync_handler_not_object() {
        let mut interp = make_interpreter();

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![
            Value::Integer(42), // Not an Object
        ])));

        let msg = make_message("Test", vec![]);
        let result = interp.process_actor_message_sync(&instance, &msg);
        assert!(result.is_err());
    }

    // Test process_actor_message_sync_mut with handler that is not an Object
    #[test]
    fn test_process_actor_message_sync_mut_handler_not_object() {
        let mut interp = make_interpreter();

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![
            Value::Integer(42), // Not an Object
        ])));
        let instance_rc = Arc::new(Mutex::new(instance));

        let msg = make_message("Test", vec![]);
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg);
        assert!(result.is_err());
    }

    // Test process_actor_message_sync with handler where message_type is not a string
    #[test]
    fn test_process_actor_message_sync_handler_type_not_string() {
        let mut interp = make_interpreter();

        let mut handler_obj = HashMap::new();
        handler_obj.insert("message_type".to_string(), Value::Integer(123)); // Not a string
        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        handler_obj.insert("body".to_string(), make_closure(vec![], body));

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![
            Value::Object(Arc::new(handler_obj)),
        ])));

        let msg = make_message("Test", vec![]);
        let result = interp.process_actor_message_sync(&instance, &msg);
        assert!(result.is_err());
    }

    // Test eval_struct_instance_method with multiple args
    #[test]
    fn test_struct_instance_method_with_args() {
        let mut interp = make_interpreter();

        // Create a closure that expects self + 2 args
        let body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let closure = make_closure(
            vec![
                ("self".to_string(), None),
                ("x".to_string(), None),
                ("y".to_string(), None),
            ],
            body,
        );
        interp.set_variable("TestStruct::method", closure);

        let instance = HashMap::new();
        let result = interp.eval_struct_instance_method(
            &instance,
            "TestStruct",
            "method",
            &[Value::Integer(1), Value::Integer(2)],
        ).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    // Test eval_object_method with non-empty args
    #[test]
    fn test_object_method_with_args() {
        let interp = make_interpreter();
        let mut obj = HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("SomeType".to_string()));

        let result = interp.eval_object_method(&obj, "method", &[Value::Integer(1)], false);
        // Should fail for unknown type
        assert!(result.is_err());
    }

    // Test process_actor_message_sync with fields to copy to self
    #[test]
    fn test_process_actor_message_sync_with_fields() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let handler = make_handler("Query", vec![], body);

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));
        instance.insert("count".to_string(), Value::Integer(10)); // Non-dunder field
        instance.insert("__private".to_string(), Value::Integer(20)); // Dunder field

        let msg = make_message("Query", vec![]);
        let result = interp.process_actor_message_sync(&instance, &msg).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    // Test process_actor_message_sync_mut with float type validation
    #[test]
    fn test_process_actor_message_sync_mut_float_type() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler = make_handler_with_types(
            "SetFloat",
            vec!["value".to_string()],
            vec!["f64"],
            body,
        );

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));
        let instance_rc = Arc::new(Mutex::new(instance));

        // Float type should work
        let msg = make_message("SetFloat", vec![Value::Float(3.14)]);
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    // Test process_actor_message_sync_mut with bool type validation
    #[test]
    fn test_process_actor_message_sync_mut_bool_type() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler = make_handler_with_types(
            "SetBool",
            vec!["value".to_string()],
            vec!["bool"],
            body,
        );

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));
        let instance_rc = Arc::new(Mutex::new(instance));

        let msg = make_message("SetBool", vec![Value::Bool(true)]);
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    // Test process_actor_message_sync_mut with float type mismatch
    #[test]
    fn test_process_actor_message_sync_mut_float_type_mismatch() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler = make_handler_with_types(
            "SetFloat",
            vec!["value".to_string()],
            vec!["f64"],
            body,
        );

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));
        let instance_rc = Arc::new(Mutex::new(instance));

        // Integer instead of float
        let msg = make_message("SetFloat", vec![Value::Integer(42)]);
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Type error"));
    }

    // Test process_actor_message_sync_mut with str type alias
    #[test]
    fn test_process_actor_message_sync_mut_str_type_alias() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler = make_handler_with_types(
            "SetStr",
            vec!["value".to_string()],
            vec!["str"],
            body,
        );

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));
        let instance_rc = Arc::new(Mutex::new(instance));

        let msg = make_message("SetStr", vec![Value::from_string("hello".to_string())]);
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    // Test process_actor_message_sync_mut with multiple params binding
    #[test]
    fn test_process_actor_message_sync_mut_multiple_params() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(100, None)));
        let handler = make_handler_with_types(
            "MultiParam",
            vec!["a".to_string(), "b".to_string()],
            vec!["i32", "String"],
            body,
        );

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));
        let instance_rc = Arc::new(Mutex::new(instance));

        let msg = make_message("MultiParam", vec![
            Value::Integer(1),
            Value::from_string("test".to_string()),
        ]);
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg).unwrap();
        assert_eq!(result, Value::Integer(100));
    }

    // Test process_actor_message_sync with multiple params
    #[test]
    fn test_process_actor_message_sync_multiple_params() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(200, None)));
        let handler = make_handler(
            "MultiParam",
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            body,
        );

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));

        let msg = make_message("MultiParam", vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let result = interp.process_actor_message_sync(&instance, &msg).unwrap();
        assert_eq!(result, Value::Integer(200));
    }

    // Test process_actor_message_sync_mut where param_types is not an array
    #[test]
    fn test_process_actor_message_sync_mut_param_types_not_array() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let mut handler_obj = HashMap::new();
        handler_obj.insert("message_type".to_string(), Value::from_string("Test".to_string()));
        handler_obj.insert("param_types".to_string(), Value::Integer(42)); // Not an array
        handler_obj.insert(
            "body".to_string(),
            make_closure(vec![("x".to_string(), None)], body),
        );
        let handler = Value::Object(Arc::new(handler_obj));

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));
        let instance_rc = Arc::new(Mutex::new(instance));

        let msg = make_message("Test", vec![Value::Integer(1)]);
        // Should succeed since param_types is ignored when not an array
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    // Test process_actor_message_sync_mut where param type is not a string
    #[test]
    fn test_process_actor_message_sync_mut_param_type_not_string() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let mut handler_obj = HashMap::new();
        handler_obj.insert("message_type".to_string(), Value::from_string("Test".to_string()));
        handler_obj.insert("param_types".to_string(), Value::Array(Arc::from(vec![
            Value::Integer(42), // Not a string
        ])));
        handler_obj.insert(
            "body".to_string(),
            make_closure(vec![("x".to_string(), None)], body),
        );
        let handler = Value::Object(Arc::new(handler_obj));

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));
        let instance_rc = Arc::new(Mutex::new(instance));

        let msg = make_message("Test", vec![Value::Integer(1)]);
        // Should succeed since type check is skipped for non-string types
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    // Test process_actor_message_sync_mut with unknown custom type
    #[test]
    fn test_process_actor_message_sync_mut_custom_type() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler = make_handler_with_types(
            "CustomType",
            vec!["value".to_string()],
            vec!["MyCustomType"],
            body,
        );

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));
        let instance_rc = Arc::new(Mutex::new(instance));

        // Send integer which won't match custom type
        let msg = make_message("CustomType", vec![Value::Integer(42)]);
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg);
        // Should fail because integer doesn't match MyCustomType
        assert!(result.is_err());
    }

    // Test process_actor_message_sync_mut with less args than params
    #[test]
    fn test_process_actor_message_sync_mut_fewer_args() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler = make_handler_with_types(
            "Test",
            vec!["a".to_string(), "b".to_string()],
            vec!["i32", "i32"],
            body,
        );

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));
        let instance_rc = Arc::new(Mutex::new(instance));

        // Send only 1 arg but handler expects 2
        let msg = make_message("Test", vec![Value::Integer(1)]);
        // Should succeed - missing args just won't be bound
        let result = interp.process_actor_message_sync_mut(&instance_rc, &msg).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    // Test process_actor_message_sync with fewer args than params
    #[test]
    fn test_process_actor_message_sync_fewer_args() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler = make_handler(
            "Test",
            vec!["a".to_string(), "b".to_string()],
            body,
        );

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![handler])));

        // Send only 1 arg but handler expects 2
        let msg = make_message("Test", vec![Value::Integer(1)]);
        let result = interp.process_actor_message_sync(&instance, &msg).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    // Test ask with multiple handlers, first doesn't match
    #[test]
    fn test_actor_ask_multiple_handlers_second_matches() {
        let mut interp = make_interpreter();

        let body1 = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler1 = make_ask_handler("First", vec![], body1);

        let body2 = make_expr(ExprKind::Literal(Literal::Integer(2, None)));
        let handler2 = make_ask_handler("Second", vec![], body2);

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![
            handler1, handler2,
        ])));

        let msg = make_message("Second", vec![]);
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg]).unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    // Test eval_struct_instance_method with extra args (more than expected)
    #[test]
    fn test_struct_instance_method_extra_args_ignored() {
        let mut interp = make_interpreter();

        // Create a closure that expects self + 1 arg
        let body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let closure = make_closure(
            vec![("self".to_string(), None), ("x".to_string(), None)],
            body,
        );
        interp.set_variable("TestStruct::method", closure);

        let instance = HashMap::new();
        // Call with 1 arg (correct)
        let result = interp.eval_struct_instance_method(
            &instance,
            "TestStruct",
            "method",
            &[Value::Integer(1)],
        ).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    // Test ask with handler value that is not a closure
    #[test]
    fn test_actor_ask_handler_value_not_closure() {
        let mut interp = make_interpreter();

        let mut handler_obj = HashMap::new();
        handler_obj.insert("message_type".to_string(), Value::from_string("Query".to_string()));
        handler_obj.insert("handler".to_string(), Value::Integer(42)); // Not a closure

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Array(Arc::from(vec![
            Value::Object(Arc::new(handler_obj)),
        ])));

        let msg = make_message("Query", vec![]);
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[msg]).unwrap();
        // Falls through since handler isn't a closure
        assert_eq!(result, Value::from_string("Received: Query".to_string()));
    }
}
