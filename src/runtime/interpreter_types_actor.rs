//! Actor definition and instantiation
//!
//! Extracted from interpreter_types_impl.rs for coverage attribution.

#![allow(clippy::unused_self)]
#![allow(clippy::expect_used)]

use crate::frontend::ast::Expr;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::{InterpreterError, Value};
use std::collections::HashMap;
use std::sync::Arc;

impl Interpreter {
    pub(crate) fn eval_actor_definition(
        &mut self,
        name: &str,
        state: &[crate::frontend::ast::StructField],
        handlers: &[crate::frontend::ast::ActorHandler],
    ) -> Result<Value, InterpreterError> {
        use std::collections::HashMap;

        // Create an actor type object
        let mut actor_type = HashMap::new();

        // Store actor metadata
        actor_type.insert(
            "__type".to_string(),
            Value::from_string("Actor".to_string()),
        );
        actor_type.insert("__name".to_string(), Value::from_string(name.to_string()));

        // Store state field definitions with default values
        let mut fields = HashMap::new();
        for field in state {
            let type_name = match &field.ty.kind {
                crate::frontend::ast::TypeKind::Named(n) => n.clone(),
                _ => "Any".to_string(),
            };

            // Create field metadata object
            let mut field_meta = HashMap::new();
            field_meta.insert("type".to_string(), Value::from_string(type_name));
            field_meta.insert("is_mut".to_string(), Value::Bool(field.is_mut));

            // Evaluate default value if present
            if let Some(ref default_expr) = field.default_value {
                match self.eval_expr(default_expr) {
                    Ok(default_val) => {
                        field_meta.insert("default".to_string(), default_val);
                    }
                    Err(_) => {
                        // If evaluation fails, use type default
                        field_meta.insert("default".to_string(), Value::Nil);
                    }
                }
            } else {
                // No default value specified, use Nil
                field_meta.insert("default".to_string(), Value::Nil);
            }

            fields.insert(field.name.clone(), Value::Object(Arc::new(field_meta)));
        }
        actor_type.insert(
            "__fields".to_string(),
            Value::Object(std::sync::Arc::new(fields)),
        );

        // Store message handlers as closures
        let mut handlers_array = Vec::new();
        for handler in handlers {
            // Create a closure for each handler
            let mut handler_obj = HashMap::new();
            handler_obj.insert(
                "message_type".to_string(),
                Value::from_string(handler.message_type.clone()),
            );

            // Store params as strings
            // RUNTIME-DEFAULT-PARAMS: Extract both param names AND default values
            let params_with_defaults: Vec<(String, Option<Arc<Expr>>)> = handler
                .params
                .iter()
                .map(|p| {
                    (
                        p.name(),
                        p.default_value
                            .clone()
                            .map(|expr| Arc::new((*expr).clone())),
                    )
                })
                .collect();

            let param_names: Vec<String> = params_with_defaults
                .iter()
                .map(|(name, _)| name.clone())
                .collect();

            handler_obj.insert(
                "params".to_string(),
                Value::Array(Arc::from(
                    param_names
                        .iter()
                        .map(|n| Value::from_string(n.clone()))
                        .collect::<Vec<_>>(),
                )),
            );

            // Store parameter types for runtime type checking
            let param_types: Vec<String> = handler
                .params
                .iter()
                .map(|p| match &p.ty.kind {
                    crate::frontend::ast::TypeKind::Named(name) => name.clone(),
                    _ => "Any".to_string(),
                })
                .collect();
            handler_obj.insert(
                "param_types".to_string(),
                Value::Array(Arc::from(
                    param_types
                        .iter()
                        .map(|t| Value::from_string(t.clone()))
                        .collect::<Vec<_>>(),
                )),
            );

            // Store the handler body AST node (we'll evaluate it later)
            // For now, store as a closure with the current environment
            handler_obj.insert(
                "body".to_string(),
                Value::Closure {
                    params: params_with_defaults,
                    body: Arc::new(*handler.body.clone()),
                    env: self.current_env().clone(), // ISSUE-119: Rc::clone (shallow copy)
                },
            );

            handlers_array.push(Value::Object(Arc::new(handler_obj)));
        }
        actor_type.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(handlers_array)),
        );

        // Register this actor type in the environment
        let actor_obj = Value::Object(std::sync::Arc::new(actor_type));
        self.set_variable(name, actor_obj.clone());

        Ok(actor_obj)
    }

    /// Instantiates an actor with initial field values.
    ///
    /// This method creates a new actor instance, initializes fields with default or provided values,
    /// and stores the message handlers for later use.
    pub(crate) fn instantiate_actor_with_args(
        &mut self,
        actor_name: &str,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Look up the actor definition
        let actor_def = self.lookup_variable(actor_name)?;

        if let Value::Object(ref actor_info) = actor_def {
            // Verify this is an actor
            if let Some(Value::String(ref type_str)) = actor_info.get("__type") {
                if type_str.as_ref() != "Actor" {
                    return Err(InterpreterError::RuntimeError(format!(
                        "{} is not an actor",
                        actor_name
                    )));
                }
            }

            // Create actor instance
            let mut instance = HashMap::new();
            instance.insert(
                "__actor".to_string(),
                Value::from_string(actor_name.to_string()),
            );

            // Check if args is a single object literal (named arguments)
            let named_args = if args.len() == 1 {
                if let Value::Object(ref obj) = args[0] {
                    Some(obj)
                } else {
                    None
                }
            } else {
                None
            };

            // Initialize state fields with default values
            // Actors use __fields just like structs
            if let Some(Value::Object(ref fields)) = actor_info.get("__fields") {
                if let Some(named) = named_args {
                    // Use named arguments
                    for (field_name, _field_info) in fields.iter() {
                        if let Some(value) = named.get(field_name) {
                            instance.insert(field_name.clone(), value.clone());
                        } else {
                            // Initialize with default for type
                            instance.insert(field_name.clone(), Value::Nil);
                        }
                    }
                } else {
                    // Map positional arguments to fields (assuming order matches definition)
                    for (i, (field_name, field_info)) in fields.iter().enumerate() {
                        if i < args.len() {
                            instance.insert(field_name.clone(), args[i].clone());
                        } else if let Value::Object(ref field_meta) = field_info {
                            // Use default value if present
                            if let Some(default) = field_meta.get("default") {
                                instance.insert(field_name.clone(), default.clone());
                            } else {
                                // Initialize with default for type
                                instance.insert(field_name.clone(), Value::Nil);
                            }
                        } else {
                            // Simple field without metadata
                            instance.insert(field_name.clone(), Value::Nil);
                        }
                    }
                }
            }

            // Store the actor's handlers for later message processing
            if let Some(handlers) = actor_info.get("__handlers") {
                instance.insert("__handlers".to_string(), handlers.clone());
            }

            // For simple interpreted actors, don't use async runtime - just store state directly
            // This allows synchronous message processing which is simpler and works for tests
            // Return ObjectMut for mutable actor state
            Ok(crate::runtime::object_helpers::new_mutable_object(instance))
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "{} is not an actor definition",
                actor_name
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Span, StructField, Type, TypeKind, Visibility};

    fn make_interpreter() -> Interpreter {
        Interpreter::new()
    }

    fn make_type(name: &str) -> Type {
        Type {
            kind: TypeKind::Named(name.to_string()),
            span: Span::default(),
        }
    }

    fn make_struct_field(name: &str, ty: Type) -> StructField {
        StructField {
            name: name.to_string(),
            ty,
            default_value: None,
            is_mut: false,
            visibility: Visibility::Public,
            decorators: vec![],
        }
    }

    #[test]
    fn test_eval_actor_definition_empty() {
        let mut interp = make_interpreter();
        let result = interp.eval_actor_definition("TestActor", &[], &[]).unwrap();

        if let Value::Object(obj) = result {
            assert_eq!(obj.get("__type"), Some(&Value::from_string("Actor".to_string())));
            assert_eq!(obj.get("__name"), Some(&Value::from_string("TestActor".to_string())));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_actor_definition_with_fields() {
        let mut interp = make_interpreter();
        let fields = vec![
            make_struct_field("count", make_type("i32")),
            make_struct_field("name", make_type("String")),
        ];

        let result = interp.eval_actor_definition("Counter", &fields, &[]).unwrap();

        if let Value::Object(obj) = result {
            assert_eq!(obj.get("__name"), Some(&Value::from_string("Counter".to_string())));
            if let Some(Value::Object(fields_obj)) = obj.get("__fields") {
                assert!(fields_obj.contains_key("count"));
                assert!(fields_obj.contains_key("name"));
            } else {
                panic!("Expected __fields Object");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_instantiate_actor_not_actor() {
        let mut interp = make_interpreter();
        // Set up a variable that's not an actor
        interp.set_variable("NotAnActor", Value::Integer(42));

        let result = interp.instantiate_actor_with_args("NotAnActor", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not an actor definition"));
    }

    #[test]
    fn test_instantiate_actor_wrong_type() {
        let mut interp = make_interpreter();
        let mut obj = HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("Struct".to_string()));
        interp.set_variable("WrongType", Value::Object(Arc::new(obj)));

        let result = interp.instantiate_actor_with_args("WrongType", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not an actor"));
    }

    #[test]
    fn test_instantiate_actor_with_positional_args() {
        let mut interp = make_interpreter();

        // Create actor definition with fields
        let fields = vec![
            make_struct_field("x", make_type("i32")),
        ];
        interp.eval_actor_definition("TestActor", &fields, &[]).unwrap();

        // Instantiate with positional args
        let result = interp.instantiate_actor_with_args("TestActor", &[Value::Integer(42)]).unwrap();

        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            assert_eq!(obj.get("__actor"), Some(&Value::from_string("TestActor".to_string())));
        } else {
            panic!("Expected ObjectMut");
        }
    }

    #[test]
    fn test_instantiate_actor_with_named_args() {
        let mut interp = make_interpreter();

        // Create actor definition with fields
        let fields = vec![
            make_struct_field("count", make_type("i32")),
        ];
        interp.eval_actor_definition("Counter", &fields, &[]).unwrap();

        // Create named arguments object
        let mut named = HashMap::new();
        named.insert("count".to_string(), Value::Integer(100));
        let args = vec![Value::Object(Arc::new(named))];

        let result = interp.instantiate_actor_with_args("Counter", &args).unwrap();

        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            assert_eq!(obj.get("count"), Some(&Value::Integer(100)));
        } else {
            panic!("Expected ObjectMut");
        }
    }

    #[test]
    fn test_instantiate_actor_undefined() {
        let mut interp = make_interpreter();
        let result = interp.instantiate_actor_with_args("UndefinedActor", &[]);
        assert!(result.is_err());
    }
}
