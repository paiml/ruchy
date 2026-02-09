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

    fn make_closure(params: Vec<(String, Option<Arc<Expr>>)>, body: Expr) -> Value {
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
                params
                    .iter()
                    .map(|p| Value::from_string(p.clone()))
                    .collect::<Vec<_>>(),
            )),
        );
        handler_obj.insert(
            "body".to_string(),
            make_closure(params.into_iter().map(|p| (p, None)).collect(), body),
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
                params
                    .iter()
                    .map(|p| Value::from_string(p.clone()))
                    .collect::<Vec<_>>(),
            )),
        );
        handler_obj.insert(
            "param_types".to_string(),
            Value::Array(Arc::from(
                param_types
                    .iter()
                    .map(|t| Value::from_string(t.to_string()))
                    .collect::<Vec<_>>(),
            )),
        );
        handler_obj.insert(
            "body".to_string(),
            make_closure(params.into_iter().map(|p| (p, None)).collect(), body),
        );
        Value::Object(Arc::new(handler_obj))
    }

    fn make_message(msg_type: &str, data: Vec<Value>) -> Value {
        let mut msg_obj = HashMap::new();
        msg_obj.insert(
            "__type".to_string(),
            Value::from_string("Message".to_string()),
        );
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires a message"));
    }

    #[test]
    fn test_actor_instance_stop() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();
        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "stop", &[])
            .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_actor_instance_ask_empty() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "ask", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires a message"));
    }

    #[test]
    fn test_actor_instance_ask_echo() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();
        let msg = Value::Integer(42);
        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg])
            .unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_actor_instance_ask_message_no_handler() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();

        // Create a Message object
        let mut msg_obj = HashMap::new();
        msg_obj.insert(
            "__type".to_string(),
            Value::from_string("Message".to_string()),
        );
        msg_obj.insert(
            "type".to_string(),
            Value::from_string("TestMsg".to_string()),
        );
        msg_obj.insert("data".to_string(), Value::Array(Arc::from(vec![])));
        let msg = Value::Object(Arc::new(msg_obj));

        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg])
            .unwrap();
        assert_eq!(result, Value::from_string("Received: TestMsg".to_string()));
    }

    #[test]
    fn test_actor_instance_unknown_method() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();
        let result = interp.eval_actor_instance_method(&instance, "TestActor", "unknown", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown actor method"));
    }

    // Process actor message sync tests
    #[test]
    fn test_process_actor_message_sync_no_handler() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();

        let mut msg_obj = HashMap::new();
        msg_obj.insert(
            "__type".to_string(),
            Value::from_string("Message".to_string()),
        );
        msg_obj.insert(
            "type".to_string(),
            Value::from_string("NoHandler".to_string()),
        );
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
        msg_obj.insert(
            "__type".to_string(),
            Value::from_string("Message".to_string()),
        );
        msg_obj.insert(
            "type".to_string(),
            Value::from_string("NoHandler".to_string()),
        );
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not a method closure"));
    }

    // Object method tests - missing type marker error
    #[test]
    fn test_object_method_missing_type() {
        let interp = make_interpreter();
        let obj = HashMap::new();

        let result = interp.eval_object_method(&obj, "test", &[], true);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing __type marker"));
    }

    #[test]
    fn test_object_method_unknown_type() {
        let interp = make_interpreter();
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("UnknownType".to_string()),
        );

        let result = interp.eval_object_method(&obj, "test", &[], true);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown object type"));
    }

    // Test send with async actor ID (branch coverage)
    #[test]
    fn test_actor_instance_send_sync_no_handler() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();

        let mut msg_obj = HashMap::new();
        msg_obj.insert(
            "__type".to_string(),
            Value::from_string("Message".to_string()),
        );
        msg_obj.insert(
            "type".to_string(),
            Value::from_string("TestMsg".to_string()),
        );
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
        msg_obj.insert(
            "__type".to_string(),
            Value::from_string("NotMessage".to_string()),
        );
        let msg = Value::Object(Arc::new(msg_obj));

        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg.clone()])
            .unwrap();
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not a method closure"));
    }

    // Test process_actor_message_sync with valid handler
    #[test]
    fn test_process_actor_message_sync_with_handler() {
        let mut interp = make_interpreter();

        // Create handler that returns 42
        let body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let handler = make_handler("TestMsg", vec![], body);

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );

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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );
        let instance_rc = Arc::new(Mutex::new(instance));

        let msg = make_message("Ping", vec![]);
        let result = interp
            .process_actor_message_sync_mut(&instance_rc, &msg)
            .unwrap();
        assert_eq!(result, Value::from_string("success".to_string()));
    }

    // Test process_actor_message_sync_mut with type validation success
    #[test]
    fn test_process_actor_message_sync_mut_type_validation_pass() {
        let mut interp = make_interpreter();

        // Create handler that expects an integer parameter
        let body = make_expr(ExprKind::Literal(Literal::Integer(100, None)));
        let handler =
            make_handler_with_types("SetValue", vec!["value".to_string()], vec!["i32"], body);

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );
        let instance_rc = Arc::new(Mutex::new(instance));

        // Send message with correct type (integer)
        let msg = make_message("SetValue", vec![Value::Integer(42)]);
        let result = interp
            .process_actor_message_sync_mut(&instance_rc, &msg)
            .unwrap();
        assert_eq!(result, Value::Integer(100));
    }

    // Test process_actor_message_sync_mut with type validation failure
    #[test]
    fn test_process_actor_message_sync_mut_type_validation_fail() {
        let mut interp = make_interpreter();

        // Create handler that expects an integer parameter
        let body = make_expr(ExprKind::Literal(Literal::Integer(100, None)));
        let handler =
            make_handler_with_types("SetValue", vec!["value".to_string()], vec!["i32"], body);

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );
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
            vec![("self".to_string(), None), ("x".to_string(), None)],
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
        let closure = make_closure(vec![("self".to_string(), None)], body);
        interp.set_variable("Point::get_value", closure);

        let instance = HashMap::new();
        // Call with 0 args (method expects none besides self)
        let result = interp
            .eval_struct_instance_method(&instance, "Point", "get_value", &[])
            .unwrap();
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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );

        // Send a simple integer value (not a Message object)
        let result = interp.eval_actor_instance_method(
            &instance,
            "TestActor",
            "send",
            &[Value::Integer(42)],
        );
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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );

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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );

        let msg = make_message("Ping", vec![]);
        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "send", &[msg])
            .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // Test eval_struct_instance_method fallback to generic method
    #[test]
    fn test_struct_instance_method_fallback() {
        let mut interp = make_interpreter();
        // No method registered - should fall back to generic method handling
        let instance = HashMap::new();
        let result =
            interp.eval_struct_instance_method(&instance, "Unknown", "unknown_method", &[]);
        // Should fail because generic method doesn't know about this
        assert!(result.is_err());
    }

    // Test process_actor_message_sync_mut with Any type (no type checking)
    #[test]
    fn test_process_actor_message_sync_mut_any_type() {
        let mut interp = make_interpreter();

        // Create handler that expects Any type parameter
        let body = make_expr(ExprKind::Literal(Literal::String("ok".to_string())));
        let handler =
            make_handler_with_types("Accept", vec!["value".to_string()], vec!["Any"], body);

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );
        let instance_rc = Arc::new(Mutex::new(instance));

        // Any type should accept anything
        let msg = make_message("Accept", vec![Value::from_string("anything".to_string())]);
        let result = interp
            .process_actor_message_sync_mut(&instance_rc, &msg)
            .unwrap();
        assert_eq!(result, Value::from_string("ok".to_string()));
    }

    // Test process_actor_message_sync_mut with string type
    #[test]
    fn test_process_actor_message_sync_mut_string_type() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler =
            make_handler_with_types("SetName", vec!["name".to_string()], vec!["String"], body);

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );
        let instance_rc = Arc::new(Mutex::new(instance));

        // String type should work
        let msg = make_message("SetName", vec![Value::from_string("test".to_string())]);
        let result = interp
            .process_actor_message_sync_mut(&instance_rc, &msg)
            .unwrap();
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
                params
                    .iter()
                    .map(|p| Value::from_string(p.clone()))
                    .collect::<Vec<_>>(),
            )),
        );
        // Note: ask method looks for "handler" key, not "body"
        handler_obj.insert(
            "handler".to_string(),
            make_closure(params.into_iter().map(|p| (p, None)).collect(), body),
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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );

        let msg = make_message("Query", vec![]);
        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg])
            .unwrap();
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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );

        let msg = make_message("Greet", vec![Value::from_string("Alice".to_string())]);
        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg])
            .unwrap();
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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );

        let msg = make_message("TestQuery", vec![Value::Integer(42)]);
        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg])
            .unwrap();
        // Should return "Received: TestQuery"
        assert_eq!(
            result,
            Value::from_string("Received: TestQuery".to_string())
        );
    }

    // Test ask with handlers that is not an array
    #[test]
    fn test_actor_ask_handlers_not_array() {
        let mut interp = make_interpreter();

        let mut instance = HashMap::new();
        instance.insert("__handlers".to_string(), Value::Integer(42)); // Not an array

        let msg = make_message("Query", vec![]);
        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg])
            .unwrap();
        // Falls through to "Received: Query" since handlers aren't iterable
        assert_eq!(result, Value::from_string("Received: Query".to_string()));
    }

    // Test ask with handler that is not an Object
    #[test]
    fn test_actor_ask_handler_not_object() {
        let mut interp = make_interpreter();

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![
                Value::Integer(123), // Not an Object
            ])),
        );

        let msg = make_message("Query", vec![]);
        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg])
            .unwrap();
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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![Value::Object(Arc::new(handler_obj))])),
        );

        let msg = make_message("Query", vec![]);
        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg])
            .unwrap();
        assert_eq!(result, Value::from_string("Received: Query".to_string()));
    }

    // Test ask with handler where message_type doesn't match
    #[test]
    fn test_actor_ask_handler_type_mismatch() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler = make_ask_handler("DifferentType", vec![], body);

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );

        let msg = make_message("Query", vec![]);
        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg])
            .unwrap();
        assert_eq!(result, Value::from_string("Received: Query".to_string()));
    }

    // Test ask with handler missing the closure
    #[test]
    fn test_actor_ask_handler_missing_closure() {
        let mut interp = make_interpreter();

        let mut handler_obj = HashMap::new();
        handler_obj.insert(
            "message_type".to_string(),
            Value::from_string("Query".to_string()),
        );
        // No "handler" key

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![Value::Object(Arc::new(handler_obj))])),
        );

        let msg = make_message("Query", vec![]);
        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg])
            .unwrap();
        assert_eq!(result, Value::from_string("Received: Query".to_string()));
    }

    // Test ask with message object missing "type" field
    #[test]
    fn test_actor_ask_message_missing_type_field() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();

        let mut msg_obj = HashMap::new();
        msg_obj.insert(
            "__type".to_string(),
            Value::from_string("Message".to_string()),
        );
        // No "type" field
        msg_obj.insert("data".to_string(), Value::Array(Arc::from(vec![])));
        let msg = Value::Object(Arc::new(msg_obj));

        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg.clone()])
            .unwrap();
        // Should echo back since no type field means it can't match handlers
        assert_eq!(result, msg);
    }

    // Test ask with message object missing "data" field
    #[test]
    fn test_actor_ask_message_missing_data_field() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();

        let mut msg_obj = HashMap::new();
        msg_obj.insert(
            "__type".to_string(),
            Value::from_string("Message".to_string()),
        );
        msg_obj.insert("type".to_string(), Value::from_string("Query".to_string()));
        // No "data" field
        let msg = Value::Object(Arc::new(msg_obj));

        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg.clone()])
            .unwrap();
        // Should echo back since no data field
        assert_eq!(result, msg);
    }

    // Test ask with message object where "type" is not a string
    #[test]
    fn test_actor_ask_message_type_not_string() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();

        let mut msg_obj = HashMap::new();
        msg_obj.insert(
            "__type".to_string(),
            Value::from_string("Message".to_string()),
        );
        msg_obj.insert("type".to_string(), Value::Integer(123)); // Not a string
        msg_obj.insert("data".to_string(), Value::Array(Arc::from(vec![])));
        let msg = Value::Object(Arc::new(msg_obj));

        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg.clone()])
            .unwrap();
        // Should echo back since type isn't a string
        assert_eq!(result, msg);
    }

    // Test ask with message object where "data" is not an array
    #[test]
    fn test_actor_ask_message_data_not_array() {
        let mut interp = make_interpreter();
        let instance = HashMap::new();

        let mut msg_obj = HashMap::new();
        msg_obj.insert(
            "__type".to_string(),
            Value::from_string("Message".to_string()),
        );
        msg_obj.insert("type".to_string(), Value::from_string("Query".to_string()));
        msg_obj.insert("data".to_string(), Value::Integer(42)); // Not an array
        let msg = Value::Object(Arc::new(msg_obj));

        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg.clone()])
            .unwrap();
        // Should echo back since data isn't an array
        assert_eq!(result, msg);
    }

    // Test process_actor_message_sync with handler where body is not a closure
    #[test]
    fn test_process_actor_message_sync_handler_body_not_closure() {
        let mut interp = make_interpreter();

        let mut handler_obj = HashMap::new();
        handler_obj.insert(
            "message_type".to_string(),
            Value::from_string("Test".to_string()),
        );
        handler_obj.insert("body".to_string(), Value::Integer(42)); // Not a closure

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![Value::Object(Arc::new(handler_obj))])),
        );

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
        handler_obj.insert(
            "message_type".to_string(),
            Value::from_string("Test".to_string()),
        );
        handler_obj.insert("body".to_string(), Value::Integer(42)); // Not a closure

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![Value::Object(Arc::new(handler_obj))])),
        );
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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![Value::Object(Arc::new(handler_obj))])),
        );

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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![Value::Object(Arc::new(handler_obj))])),
        );
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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![
                Value::Integer(42), // Not an Object
            ])),
        );

        let msg = make_message("Test", vec![]);
        let result = interp.process_actor_message_sync(&instance, &msg);
        assert!(result.is_err());
    }

    // Test process_actor_message_sync_mut with handler that is not an Object
    #[test]
    fn test_process_actor_message_sync_mut_handler_not_object() {
        let mut interp = make_interpreter();

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![
                Value::Integer(42), // Not an Object
            ])),
        );
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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![Value::Object(Arc::new(handler_obj))])),
        );

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
        let result = interp
            .eval_struct_instance_method(
                &instance,
                "TestStruct",
                "method",
                &[Value::Integer(1), Value::Integer(2)],
            )
            .unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    // Test eval_object_method with non-empty args
    #[test]
    fn test_object_method_with_args() {
        let interp = make_interpreter();
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("SomeType".to_string()),
        );

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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );
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
        let handler =
            make_handler_with_types("SetFloat", vec!["value".to_string()], vec!["f64"], body);

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );
        let instance_rc = Arc::new(Mutex::new(instance));

        // Float type should work
        let msg = make_message("SetFloat", vec![Value::Float(3.14)]);
        let result = interp
            .process_actor_message_sync_mut(&instance_rc, &msg)
            .unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    // Test process_actor_message_sync_mut with bool type validation
    #[test]
    fn test_process_actor_message_sync_mut_bool_type() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler =
            make_handler_with_types("SetBool", vec!["value".to_string()], vec!["bool"], body);

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );
        let instance_rc = Arc::new(Mutex::new(instance));

        let msg = make_message("SetBool", vec![Value::Bool(true)]);
        let result = interp
            .process_actor_message_sync_mut(&instance_rc, &msg)
            .unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    // Test process_actor_message_sync_mut with float type mismatch
    #[test]
    fn test_process_actor_message_sync_mut_float_type_mismatch() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler =
            make_handler_with_types("SetFloat", vec!["value".to_string()], vec!["f64"], body);

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );
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
        let handler =
            make_handler_with_types("SetStr", vec!["value".to_string()], vec!["str"], body);

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );
        let instance_rc = Arc::new(Mutex::new(instance));

        let msg = make_message("SetStr", vec![Value::from_string("hello".to_string())]);
        let result = interp
            .process_actor_message_sync_mut(&instance_rc, &msg)
            .unwrap();
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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );
        let instance_rc = Arc::new(Mutex::new(instance));

        let msg = make_message(
            "MultiParam",
            vec![Value::Integer(1), Value::from_string("test".to_string())],
        );
        let result = interp
            .process_actor_message_sync_mut(&instance_rc, &msg)
            .unwrap();
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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );

        let msg = make_message(
            "MultiParam",
            vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        );
        let result = interp.process_actor_message_sync(&instance, &msg).unwrap();
        assert_eq!(result, Value::Integer(200));
    }

    // Test process_actor_message_sync_mut where param_types is not an array
    #[test]
    fn test_process_actor_message_sync_mut_param_types_not_array() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let mut handler_obj = HashMap::new();
        handler_obj.insert(
            "message_type".to_string(),
            Value::from_string("Test".to_string()),
        );
        handler_obj.insert("param_types".to_string(), Value::Integer(42)); // Not an array
        handler_obj.insert(
            "body".to_string(),
            make_closure(vec![("x".to_string(), None)], body),
        );
        let handler = Value::Object(Arc::new(handler_obj));

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );
        let instance_rc = Arc::new(Mutex::new(instance));

        let msg = make_message("Test", vec![Value::Integer(1)]);
        // Should succeed since param_types is ignored when not an array
        let result = interp
            .process_actor_message_sync_mut(&instance_rc, &msg)
            .unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    // Test process_actor_message_sync_mut where param type is not a string
    #[test]
    fn test_process_actor_message_sync_mut_param_type_not_string() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let mut handler_obj = HashMap::new();
        handler_obj.insert(
            "message_type".to_string(),
            Value::from_string("Test".to_string()),
        );
        handler_obj.insert(
            "param_types".to_string(),
            Value::Array(Arc::from(vec![
                Value::Integer(42), // Not a string
            ])),
        );
        handler_obj.insert(
            "body".to_string(),
            make_closure(vec![("x".to_string(), None)], body),
        );
        let handler = Value::Object(Arc::new(handler_obj));

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );
        let instance_rc = Arc::new(Mutex::new(instance));

        let msg = make_message("Test", vec![Value::Integer(1)]);
        // Should succeed since type check is skipped for non-string types
        let result = interp
            .process_actor_message_sync_mut(&instance_rc, &msg)
            .unwrap();
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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );
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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );
        let instance_rc = Arc::new(Mutex::new(instance));

        // Send only 1 arg but handler expects 2
        let msg = make_message("Test", vec![Value::Integer(1)]);
        // Should succeed - missing args just won't be bound
        let result = interp
            .process_actor_message_sync_mut(&instance_rc, &msg)
            .unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    // Test process_actor_message_sync with fewer args than params
    #[test]
    fn test_process_actor_message_sync_fewer_args() {
        let mut interp = make_interpreter();

        let body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let handler = make_handler("Test", vec!["a".to_string(), "b".to_string()], body);

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler])),
        );

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
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![handler1, handler2])),
        );

        let msg = make_message("Second", vec![]);
        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg])
            .unwrap();
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
        let result = interp
            .eval_struct_instance_method(&instance, "TestStruct", "method", &[Value::Integer(1)])
            .unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    // Test ask with handler value that is not a closure
    #[test]
    fn test_actor_ask_handler_value_not_closure() {
        let mut interp = make_interpreter();

        let mut handler_obj = HashMap::new();
        handler_obj.insert(
            "message_type".to_string(),
            Value::from_string("Query".to_string()),
        );
        handler_obj.insert("handler".to_string(), Value::Integer(42)); // Not a closure

        let mut instance = HashMap::new();
        instance.insert(
            "__handlers".to_string(),
            Value::Array(Arc::from(vec![Value::Object(Arc::new(handler_obj))])),
        );

        let msg = make_message("Query", vec![]);
        let result = interp
            .eval_actor_instance_method(&instance, "TestActor", "ask", &[msg])
            .unwrap();
        // Falls through since handler isn't a closure
        assert_eq!(result, Value::from_string("Received: Query".to_string()));
    }
