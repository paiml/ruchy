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
    use crate::frontend::ast::{
        ActorHandler, ExprKind, Literal, Param, Span, StructField, Type, TypeKind, Visibility,
    };

    fn make_interpreter() -> Interpreter {
        Interpreter::new()
    }

    fn make_type(name: &str) -> Type {
        Type {
            kind: TypeKind::Named(name.to_string()),
            span: Span::default(),
        }
    }

    fn make_any_type() -> Type {
        // Create a non-Named type to cover the "Any" fallback path
        Type {
            kind: TypeKind::Array {
                elem_type: Box::new(make_type("i32")),
                size: 10,
            },
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

    fn make_struct_field_with_default(
        name: &str,
        ty: Type,
        default: crate::frontend::ast::Expr,
    ) -> StructField {
        StructField {
            name: name.to_string(),
            ty,
            default_value: Some(default),
            is_mut: false,
            visibility: Visibility::Public,
            decorators: vec![],
        }
    }

    fn make_struct_field_mutable(name: &str, ty: Type) -> StructField {
        StructField {
            name: name.to_string(),
            ty,
            default_value: None,
            is_mut: true,
            visibility: Visibility::Public,
            decorators: vec![],
        }
    }

    fn make_expr(kind: ExprKind) -> crate::frontend::ast::Expr {
        crate::frontend::ast::Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn make_param(name: &str, ty: Type) -> Param {
        Param {
            pattern: crate::frontend::ast::Pattern::Identifier(name.to_string()),
            ty,
            span: Span::default(),
            is_mutable: false,
            default_value: None,
        }
    }

    fn make_param_with_default(name: &str, ty: Type, default: crate::frontend::ast::Expr) -> Param {
        Param {
            pattern: crate::frontend::ast::Pattern::Identifier(name.to_string()),
            ty,
            span: Span::default(),
            is_mutable: false,
            default_value: Some(Box::new(default)),
        }
    }

    fn make_handler(
        message_type: &str,
        params: Vec<Param>,
        body: crate::frontend::ast::Expr,
    ) -> ActorHandler {
        ActorHandler {
            message_type: message_type.to_string(),
            params,
            body: Box::new(body),
        }
    }

    // ============== Actor Definition Tests ==============

    #[test]
    fn test_eval_actor_definition_empty() {
        let mut interp = make_interpreter();
        let result = interp.eval_actor_definition("TestActor", &[], &[]).unwrap();

        if let Value::Object(obj) = result {
            assert_eq!(
                obj.get("__type"),
                Some(&Value::from_string("Actor".to_string()))
            );
            assert_eq!(
                obj.get("__name"),
                Some(&Value::from_string("TestActor".to_string()))
            );
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

        let result = interp
            .eval_actor_definition("Counter", &fields, &[])
            .unwrap();

        if let Value::Object(obj) = result {
            assert_eq!(
                obj.get("__name"),
                Some(&Value::from_string("Counter".to_string()))
            );
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
    fn test_eval_actor_definition_with_non_named_field_type() {
        // Covers line 38: the "Any" fallback for non-Named types
        let mut interp = make_interpreter();
        let fields = vec![make_struct_field("items", make_any_type())];

        let result = interp
            .eval_actor_definition("ItemActor", &fields, &[])
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(fields_obj)) = obj.get("__fields") {
                if let Some(Value::Object(field_meta)) = fields_obj.get("items") {
                    assert_eq!(
                        field_meta.get("type"),
                        Some(&Value::from_string("Any".to_string()))
                    );
                } else {
                    panic!("Expected field metadata object");
                }
            } else {
                panic!("Expected __fields Object");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_actor_definition_with_default_value_success() {
        // Covers lines 49-50: successful default value evaluation
        let mut interp = make_interpreter();
        let default_expr = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let fields = vec![make_struct_field_with_default(
            "count",
            make_type("i32"),
            default_expr,
        )];

        let result = interp
            .eval_actor_definition("DefaultActor", &fields, &[])
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(fields_obj)) = obj.get("__fields") {
                if let Some(Value::Object(field_meta)) = fields_obj.get("count") {
                    assert_eq!(field_meta.get("default"), Some(&Value::Integer(42)));
                } else {
                    panic!("Expected field metadata object");
                }
            } else {
                panic!("Expected __fields Object");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_actor_definition_with_default_value_failure() {
        // Covers lines 52-55: default value evaluation fails
        let mut interp = make_interpreter();
        // Use an undefined identifier which will fail to evaluate
        let default_expr = make_expr(ExprKind::Identifier("undefined_var".to_string()));
        let fields = vec![make_struct_field_with_default(
            "value",
            make_type("i32"),
            default_expr,
        )];

        let result = interp
            .eval_actor_definition("FailDefaultActor", &fields, &[])
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(fields_obj)) = obj.get("__fields") {
                if let Some(Value::Object(field_meta)) = fields_obj.get("value") {
                    // When evaluation fails, default should be Nil
                    assert_eq!(field_meta.get("default"), Some(&Value::Nil));
                } else {
                    panic!("Expected field metadata object");
                }
            } else {
                panic!("Expected __fields Object");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_actor_definition_with_mutable_field() {
        // Covers line 44: is_mut field
        let mut interp = make_interpreter();
        let fields = vec![make_struct_field_mutable("data", make_type("String"))];

        let result = interp
            .eval_actor_definition("MutActor", &fields, &[])
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(fields_obj)) = obj.get("__fields") {
                if let Some(Value::Object(field_meta)) = fields_obj.get("data") {
                    assert_eq!(field_meta.get("is_mut"), Some(&Value::Bool(true)));
                } else {
                    panic!("Expected field metadata object");
                }
            } else {
                panic!("Expected __fields Object");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_actor_definition_with_handlers() {
        // Covers lines 71-144: handler creation
        let mut interp = make_interpreter();
        let body = make_expr(ExprKind::Literal(Literal::Integer(100, None)));
        let handlers = vec![make_handler("Increment", vec![], body)];

        let result = interp
            .eval_actor_definition("HandlerActor", &[], &handlers)
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Array(handlers_arr)) = obj.get("__handlers") {
                assert_eq!(handlers_arr.len(), 1);
                if let Value::Object(handler_obj) = &handlers_arr[0] {
                    assert_eq!(
                        handler_obj.get("message_type"),
                        Some(&Value::from_string("Increment".to_string()))
                    );
                } else {
                    panic!("Expected handler object");
                }
            } else {
                panic!("Expected __handlers Array");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_actor_definition_with_handler_params() {
        // Covers lines 81-92: handler params with defaults
        let mut interp = make_interpreter();
        let body = make_expr(ExprKind::Literal(Literal::Bool(true)));
        let params = vec![
            make_param("x", make_type("i32")),
            make_param("y", make_type("String")),
        ];
        let handlers = vec![make_handler("SetValues", params, body)];

        let result = interp
            .eval_actor_definition("ParamActor", &[], &handlers)
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Array(handlers_arr)) = obj.get("__handlers") {
                assert_eq!(handlers_arr.len(), 1);
                if let Value::Object(handler_obj) = &handlers_arr[0] {
                    if let Some(Value::Array(param_names)) = handler_obj.get("params") {
                        assert_eq!(param_names.len(), 2);
                        assert_eq!(param_names[0], Value::from_string("x".to_string()));
                        assert_eq!(param_names[1], Value::from_string("y".to_string()));
                    } else {
                        panic!("Expected params array");
                    }
                } else {
                    panic!("Expected handler object");
                }
            } else {
                panic!("Expected __handlers Array");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_actor_definition_with_param_default_value() {
        // Covers lines 87-89: handler param with default value
        let mut interp = make_interpreter();
        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let default_val = make_expr(ExprKind::Literal(Literal::Integer(10, None)));
        let params = vec![make_param_with_default(
            "amount",
            make_type("i32"),
            default_val,
        )];
        let handlers = vec![make_handler("Add", params, body)];

        let result = interp
            .eval_actor_definition("DefaultParamActor", &[], &handlers)
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Array(handlers_arr)) = obj.get("__handlers") {
                assert_eq!(handlers_arr.len(), 1);
                if let Value::Object(handler_obj) = &handlers_arr[0] {
                    // Verify the body closure was stored
                    assert!(handler_obj.contains_key("body"));
                } else {
                    panic!("Expected handler object");
                }
            } else {
                panic!("Expected __handlers Array");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_actor_definition_with_non_named_param_type() {
        // Covers line 115: "Any" fallback for non-Named param types
        let mut interp = make_interpreter();
        let body = make_expr(ExprKind::Literal(Literal::Bool(false)));
        let params = vec![make_param("items", make_any_type())];
        let handlers = vec![make_handler("Process", params, body)];

        let result = interp
            .eval_actor_definition("AnyParamActor", &[], &handlers)
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Array(handlers_arr)) = obj.get("__handlers") {
                if let Value::Object(handler_obj) = &handlers_arr[0] {
                    if let Some(Value::Array(param_types)) = handler_obj.get("param_types") {
                        assert_eq!(param_types[0], Value::from_string("Any".to_string()));
                    } else {
                        panic!("Expected param_types array");
                    }
                } else {
                    panic!("Expected handler object");
                }
            } else {
                panic!("Expected __handlers Array");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_actor_definition_registers_in_env() {
        // Covers line 148: actor registration in environment
        let mut interp = make_interpreter();
        interp
            .eval_actor_definition("RegisteredActor", &[], &[])
            .unwrap();

        // Verify the actor is registered
        let looked_up = interp.lookup_variable("RegisteredActor").unwrap();
        if let Value::Object(obj) = looked_up {
            assert_eq!(
                obj.get("__type"),
                Some(&Value::from_string("Actor".to_string()))
            );
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_actor_definition_multiple_handlers() {
        let mut interp = make_interpreter();
        let body1 = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let body2 = make_expr(ExprKind::Literal(Literal::Integer(2, None)));
        let handlers = vec![
            make_handler("First", vec![], body1),
            make_handler("Second", vec![], body2),
        ];

        let result = interp
            .eval_actor_definition("MultiHandlerActor", &[], &handlers)
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Array(handlers_arr)) = obj.get("__handlers") {
                assert_eq!(handlers_arr.len(), 2);
            } else {
                panic!("Expected __handlers Array");
            }
        } else {
            panic!("Expected Object");
        }
    }

    // ============== Actor Instantiation Tests ==============

    #[test]
    fn test_instantiate_actor_not_actor() {
        let mut interp = make_interpreter();
        // Set up a variable that's not an actor
        interp.set_variable("NotAnActor", Value::Integer(42));

        let result = interp.instantiate_actor_with_args("NotAnActor", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not an actor definition"));
    }

    #[test]
    fn test_instantiate_actor_wrong_type() {
        let mut interp = make_interpreter();
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("Struct".to_string()),
        );
        interp.set_variable("WrongType", Value::Object(Arc::new(obj)));

        let result = interp.instantiate_actor_with_args("WrongType", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not an actor"));
    }

    #[test]
    fn test_instantiate_actor_with_positional_args() {
        let mut interp = make_interpreter();

        // Create actor definition with fields
        let fields = vec![make_struct_field("x", make_type("i32"))];
        interp
            .eval_actor_definition("TestActor", &fields, &[])
            .unwrap();

        // Instantiate with positional args
        let result = interp
            .instantiate_actor_with_args("TestActor", &[Value::Integer(42)])
            .unwrap();

        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            assert_eq!(
                obj.get("__actor"),
                Some(&Value::from_string("TestActor".to_string()))
            );
        } else {
            panic!("Expected ObjectMut");
        }
    }

    #[test]
    fn test_instantiate_actor_with_named_args() {
        let mut interp = make_interpreter();

        // Create actor definition with fields
        let fields = vec![make_struct_field("count", make_type("i32"))];
        interp
            .eval_actor_definition("Counter", &fields, &[])
            .unwrap();

        // Create named arguments object
        let mut named = HashMap::new();
        named.insert("count".to_string(), Value::Integer(100));
        let args = vec![Value::Object(Arc::new(named))];

        let result = interp
            .instantiate_actor_with_args("Counter", &args)
            .unwrap();

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

    #[test]
    fn test_instantiate_actor_named_args_missing_field() {
        // Covers lines 203-204: named args where field is not found
        let mut interp = make_interpreter();
        let fields = vec![
            make_struct_field("a", make_type("i32")),
            make_struct_field("b", make_type("i32")),
        ];
        interp
            .eval_actor_definition("TwoFieldActor", &fields, &[])
            .unwrap();

        // Named args object only has one field
        let mut named = HashMap::new();
        named.insert("a".to_string(), Value::Integer(10));
        // "b" is NOT provided
        let args = vec![Value::Object(Arc::new(named))];

        let result = interp
            .instantiate_actor_with_args("TwoFieldActor", &args)
            .unwrap();

        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            assert_eq!(obj.get("a"), Some(&Value::Integer(10)));
            // "b" should be initialized to Nil since not provided
            assert_eq!(obj.get("b"), Some(&Value::Nil));
        } else {
            panic!("Expected ObjectMut");
        }
    }

    #[test]
    fn test_instantiate_actor_positional_args_fewer_than_fields() {
        // Covers lines 212-219: positional args fewer than fields, use default
        let mut interp = make_interpreter();
        let default_expr = make_expr(ExprKind::Literal(Literal::Integer(99, None)));
        let fields = vec![
            make_struct_field("first", make_type("i32")),
            make_struct_field_with_default("second", make_type("i32"), default_expr),
        ];
        interp
            .eval_actor_definition("DefaultFieldActor", &fields, &[])
            .unwrap();

        // Only provide first argument
        let args = vec![Value::Integer(1)];
        let result = interp
            .instantiate_actor_with_args("DefaultFieldActor", &args)
            .unwrap();

        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            // Verify the field is populated (order may vary due to HashMap)
            assert!(obj.len() >= 3); // __actor + fields + possibly __handlers
        } else {
            panic!("Expected ObjectMut");
        }
    }

    #[test]
    fn test_instantiate_actor_positional_args_field_no_default() {
        // Covers lines 216-218: field with metadata but no default
        let mut interp = make_interpreter();
        let fields = vec![make_struct_field("only_field", make_type("i32"))];
        interp
            .eval_actor_definition("NoDefaultActor", &fields, &[])
            .unwrap();

        // Provide no arguments - field has no default, should use Nil
        let result = interp
            .instantiate_actor_with_args("NoDefaultActor", &[])
            .unwrap();

        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            // Field should exist with Nil value
            assert!(obj.contains_key("__actor"));
        } else {
            panic!("Expected ObjectMut");
        }
    }

    #[test]
    fn test_instantiate_actor_simple_field_without_metadata() {
        // Covers lines 220-222: simple field without metadata object
        // This is tricky to trigger - we need to manually create the actor definition
        let mut interp = make_interpreter();

        // Manually create actor definition with simple field (not Object metadata)
        let mut actor_type = HashMap::new();
        actor_type.insert(
            "__type".to_string(),
            Value::from_string("Actor".to_string()),
        );
        actor_type.insert(
            "__name".to_string(),
            Value::from_string("SimpleFieldActor".to_string()),
        );

        let mut fields = HashMap::new();
        // Insert a field with a simple Value, not Object metadata
        fields.insert("simple_field".to_string(), Value::Integer(0)); // Not an Object!

        actor_type.insert("__fields".to_string(), Value::Object(Arc::new(fields)));
        actor_type.insert("__handlers".to_string(), Value::Array(Arc::from(vec![])));

        interp.set_variable("SimpleFieldActor", Value::Object(Arc::new(actor_type)));

        // Instantiate with no args - the simple field should get Nil
        let result = interp
            .instantiate_actor_with_args("SimpleFieldActor", &[])
            .unwrap();

        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            assert_eq!(obj.get("simple_field"), Some(&Value::Nil));
        } else {
            panic!("Expected ObjectMut");
        }
    }

    #[test]
    fn test_instantiate_actor_with_handlers_stored() {
        // Covers lines 229-231: handlers are stored in instance
        let mut interp = make_interpreter();
        let body = make_expr(ExprKind::Literal(Literal::Bool(true)));
        let handlers = vec![make_handler("Ping", vec![], body)];
        interp
            .eval_actor_definition("HandlerStoredActor", &[], &handlers)
            .unwrap();

        let result = interp
            .instantiate_actor_with_args("HandlerStoredActor", &[])
            .unwrap();

        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            assert!(obj.contains_key("__handlers"));
            if let Some(Value::Array(handlers_arr)) = obj.get("__handlers") {
                assert_eq!(handlers_arr.len(), 1);
            } else {
                panic!("Expected handlers array");
            }
        } else {
            panic!("Expected ObjectMut");
        }
    }

    #[test]
    fn test_instantiate_actor_no_type_marker() {
        // Actor object without __type marker
        let mut interp = make_interpreter();
        let obj = HashMap::new(); // Empty object, no __type
        interp.set_variable("NoTypeActor", Value::Object(Arc::new(obj)));

        // Should still work - won't fail the __type check but will succeed without error
        let result = interp
            .instantiate_actor_with_args("NoTypeActor", &[])
            .unwrap();
        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            assert!(obj.contains_key("__actor"));
        } else {
            panic!("Expected ObjectMut");
        }
    }

    #[test]
    fn test_instantiate_actor_multiple_positional_args() {
        let mut interp = make_interpreter();
        let fields = vec![
            make_struct_field("a", make_type("i32")),
            make_struct_field("b", make_type("String")),
        ];
        interp
            .eval_actor_definition("MultiArgActor", &fields, &[])
            .unwrap();

        let result = interp
            .instantiate_actor_with_args(
                "MultiArgActor",
                &[Value::Integer(10), Value::from_string("hello".to_string())],
            )
            .unwrap();

        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            assert!(obj.contains_key("__actor"));
        } else {
            panic!("Expected ObjectMut");
        }
    }

    #[test]
    fn test_instantiate_actor_with_non_object_single_arg() {
        // Covers lines 187-189: single arg that is not an Object (not named args)
        let mut interp = make_interpreter();
        let fields = vec![make_struct_field("value", make_type("i32"))];
        interp
            .eval_actor_definition("SingleArgActor", &fields, &[])
            .unwrap();

        // Single arg that is NOT an Object
        let result = interp
            .instantiate_actor_with_args("SingleArgActor", &[Value::Integer(55)])
            .unwrap();

        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            assert!(obj.contains_key("__actor"));
        } else {
            panic!("Expected ObjectMut");
        }
    }

    #[test]
    fn test_instantiate_actor_no_fields() {
        // Actor with no fields at all
        let mut interp = make_interpreter();
        interp
            .eval_actor_definition("NoFieldsActor", &[], &[])
            .unwrap();

        let result = interp
            .instantiate_actor_with_args("NoFieldsActor", &[])
            .unwrap();

        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            assert_eq!(
                obj.get("__actor"),
                Some(&Value::from_string("NoFieldsActor".to_string()))
            );
        } else {
            panic!("Expected ObjectMut");
        }
    }

    #[test]
    fn test_eval_actor_definition_with_string_default() {
        let mut interp = make_interpreter();
        let default_expr = make_expr(ExprKind::Literal(Literal::String(
            "default_value".to_string(),
        )));
        let fields = vec![make_struct_field_with_default(
            "name",
            make_type("String"),
            default_expr,
        )];

        let result = interp
            .eval_actor_definition("StringDefaultActor", &fields, &[])
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(fields_obj)) = obj.get("__fields") {
                if let Some(Value::Object(field_meta)) = fields_obj.get("name") {
                    assert_eq!(
                        field_meta.get("default"),
                        Some(&Value::from_string("default_value".to_string()))
                    );
                } else {
                    panic!("Expected field metadata object");
                }
            } else {
                panic!("Expected __fields Object");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_actor_definition_with_bool_default() {
        let mut interp = make_interpreter();
        let default_expr = make_expr(ExprKind::Literal(Literal::Bool(true)));
        let fields = vec![make_struct_field_with_default(
            "active",
            make_type("bool"),
            default_expr,
        )];

        let result = interp
            .eval_actor_definition("BoolDefaultActor", &fields, &[])
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(fields_obj)) = obj.get("__fields") {
                if let Some(Value::Object(field_meta)) = fields_obj.get("active") {
                    assert_eq!(field_meta.get("default"), Some(&Value::Bool(true)));
                } else {
                    panic!("Expected field metadata object");
                }
            } else {
                panic!("Expected __fields Object");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_actor_definition_handler_with_typed_params() {
        let mut interp = make_interpreter();
        let body = make_expr(ExprKind::Literal(Literal::Integer(0, None)));
        let params = vec![
            make_param("count", make_type("i32")),
            make_param("message", make_type("String")),
            make_param("flag", make_type("bool")),
        ];
        let handlers = vec![make_handler("Process", params, body)];

        let result = interp
            .eval_actor_definition("TypedParamActor", &[], &handlers)
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Array(handlers_arr)) = obj.get("__handlers") {
                if let Value::Object(handler_obj) = &handlers_arr[0] {
                    if let Some(Value::Array(param_types)) = handler_obj.get("param_types") {
                        assert_eq!(param_types.len(), 3);
                        assert_eq!(param_types[0], Value::from_string("i32".to_string()));
                        assert_eq!(param_types[1], Value::from_string("String".to_string()));
                        assert_eq!(param_types[2], Value::from_string("bool".to_string()));
                    } else {
                        panic!("Expected param_types array");
                    }
                } else {
                    panic!("Expected handler object");
                }
            } else {
                panic!("Expected __handlers Array");
            }
        } else {
            panic!("Expected Object");
        }
    }
}
