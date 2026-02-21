use super::*;
use crate::frontend::ast::{
    ClassConstant, ClassMethod, Constructor, Literal, Param, Pattern, SelfType, Span, StructField,
    Type, TypeKind, Visibility,
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

fn make_expr(kind: ExprKind) -> Expr {
    Expr::new(kind, Span::default())
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

fn make_struct_field_with_default(name: &str, ty: Type, default: Expr) -> StructField {
    StructField {
        name: name.to_string(),
        ty,
        default_value: Some(default),
        is_mut: true,
        visibility: Visibility::Public,
        decorators: vec![],
    }
}

fn make_param(name: &str) -> Param {
    Param {
        pattern: Pattern::Identifier(name.to_string()),
        ty: make_type("Any"),
        span: Span::default(),
        is_mutable: false,
        default_value: None,
    }
}

fn make_constructor(name: Option<&str>, params: Vec<Param>, body: Expr) -> Constructor {
    Constructor {
        name: name.map(|s| s.to_string()),
        params,
        return_type: None,
        body: Box::new(body),
        is_pub: true,
    }
}

fn make_method(name: &str, params: Vec<Param>, body: Expr, is_static: bool) -> ClassMethod {
    ClassMethod {
        name: name.to_string(),
        params,
        return_type: Some(make_type("Any")),
        body: Box::new(body),
        is_pub: true,
        is_static,
        is_override: false,
        is_final: false,
        is_abstract: false,
        is_async: false,
        self_type: if is_static {
            SelfType::None
        } else {
            SelfType::Borrowed
        },
    }
}

fn make_constant(name: &str, value: Expr) -> ClassConstant {
    ClassConstant {
        name: name.to_string(),
        ty: make_type("i32"),
        value,
        is_pub: true,
    }
}

fn make_struct_literal(name: &str, fields: Vec<(&str, Expr)>) -> Expr {
    make_expr(ExprKind::StructLiteral {
        name: name.to_string(),
        fields: fields
            .into_iter()
            .map(|(n, e)| (n.to_string(), e))
            .collect(),
        base: None,
    })
}

#[test]
fn test_eval_class_definition_empty() {
    let mut interp = make_interpreter();
    let result = interp
        .eval_class_definition("Empty", &[], None, &[], &[], &[], &[], &[], &[], false)
        .unwrap();

    if let Value::Object(obj) = result {
        assert_eq!(
            obj.get("__type"),
            Some(&Value::from_string("Class".to_string()))
        );
        assert_eq!(
            obj.get("__name"),
            Some(&Value::from_string("Empty".to_string()))
        );
        // Should have default "new" constructor
        if let Some(Value::Object(ctors)) = obj.get("__constructors") {
            assert!(ctors.contains_key("new"));
        }
    } else {
        panic!("Expected Object");
    }
}

#[test]
fn test_eval_class_definition_with_superclass() {
    let mut interp = make_interpreter();
    let parent = "ParentClass".to_string();
    let result = interp
        .eval_class_definition(
            "ChildClass",
            &[],
            Some(&parent),
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            false,
        )
        .unwrap();

    if let Value::Object(obj) = result {
        assert_eq!(
            obj.get("__superclass"),
            Some(&Value::from_string("ParentClass".to_string()))
        );
    } else {
        panic!("Expected Object");
    }
}

#[test]
fn test_instantiate_class_not_class() {
    let mut interp = make_interpreter();
    interp.set_variable("NotClass", Value::Integer(42));
    let result = interp.instantiate_class_with_constructor("NotClass", "new", &[]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("not a class definition"));
}

#[test]
fn test_instantiate_class_wrong_type() {
    let mut interp = make_interpreter();
    let mut obj = HashMap::new();
    obj.insert(
        "__type".to_string(),
        Value::from_string("Struct".to_string()),
    );
    interp.set_variable("WrongType", Value::Object(Arc::new(obj)));

    let result = interp.instantiate_class_with_constructor("WrongType", "new", &[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not a class"));
}

#[test]
fn test_instantiate_class_with_args_not_class() {
    let mut interp = make_interpreter();
    interp.set_variable("NotClass", Value::Integer(42));
    let result = interp.instantiate_class_with_args("NotClass", &[]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("not a class definition"));
}

#[test]
fn test_instantiate_class_with_args_wrong_type() {
    let mut interp = make_interpreter();
    let mut obj = HashMap::new();
    obj.insert(
        "__type".to_string(),
        Value::from_string("Struct".to_string()),
    );
    interp.set_variable("WrongType", Value::Object(Arc::new(obj)));

    let result = interp.instantiate_class_with_args("WrongType", &[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not a class"));
}

#[test]
fn test_eval_class_instance_method_not_class() {
    let mut interp = make_interpreter();
    interp.set_variable("NotClass", Value::Integer(42));
    let result = interp.eval_class_instance_method(&HashMap::new(), "NotClass", "method", &[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not a class"));
}

#[test]
fn test_call_static_method_not_class() {
    let mut interp = make_interpreter();
    interp.set_variable("NotClass", Value::Integer(42));
    let result = interp.call_static_method("NotClass", "method", &[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not a class"));
}

#[test]
fn test_call_static_method_not_found() {
    let mut interp = make_interpreter();
    interp
        .eval_class_definition("TestClass", &[], None, &[], &[], &[], &[], &[], &[], false)
        .unwrap();
    let result = interp.call_static_method("TestClass", "nonexistent", &[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_eval_class_instance_method_not_found() {
    let mut interp = make_interpreter();
    interp
        .eval_class_definition("TestClass", &[], None, &[], &[], &[], &[], &[], &[], false)
        .unwrap();
    let result =
        interp.eval_class_instance_method(&HashMap::new(), "TestClass", "nonexistent", &[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no method named"));
}

// =========================================================================
// Additional tests for coverage improvement
// =========================================================================

#[test]
fn test_eval_class_definition_with_fields() {
    let mut interp = make_interpreter();
    let fields = vec![
        make_struct_field("x", make_type("i32")),
        make_struct_field("y", make_type("String")),
    ];

    let result = interp
        .eval_class_definition("Point", &[], None, &[], &fields, &[], &[], &[], &[], false)
        .unwrap();

    if let Value::Object(obj) = result {
        if let Some(Value::Object(field_defs)) = obj.get("__fields") {
            assert!(field_defs.contains_key("x"));
            assert!(field_defs.contains_key("y"));
        } else {
            panic!("Expected __fields");
        }
    } else {
        panic!("Expected Object");
    }
}

#[test]
fn test_eval_class_definition_with_field_defaults() {
    let mut interp = make_interpreter();
    let fields = vec![make_struct_field_with_default(
        "count",
        make_type("i32"),
        make_expr(ExprKind::Literal(Literal::Integer(42, None))),
    )];

    let result = interp
        .eval_class_definition(
            "Counter",
            &[],
            None,
            &[],
            &fields,
            &[],
            &[],
            &[],
            &[],
            false,
        )
        .unwrap();

    if let Value::Object(obj) = result {
        if let Some(Value::Object(field_defs)) = obj.get("__fields") {
            if let Some(Value::Object(count_field)) = field_defs.get("count") {
                assert_eq!(count_field.get("default"), Some(&Value::Integer(42)));
                assert_eq!(count_field.get("is_mut"), Some(&Value::Bool(true)));
            } else {
                panic!("Expected count field");
            }
        }
    } else {
        panic!("Expected Object");
    }
}

#[test]
fn test_eval_class_definition_with_constructor() {
    let mut interp = make_interpreter();
    let constructors = vec![make_constructor(
        Some("new"),
        vec![make_param("value")],
        make_expr(ExprKind::Block(vec![])),
    )];

    let result = interp
        .eval_class_definition(
            "MyClass",
            &[],
            None,
            &[],
            &[],
            &constructors,
            &[],
            &[],
            &[],
            false,
        )
        .unwrap();

    if let Value::Object(obj) = result {
        if let Some(Value::Object(ctors)) = obj.get("__constructors") {
            assert!(ctors.contains_key("new"));
            if let Some(Value::Closure { params, .. }) = ctors.get("new") {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].0, "value");
            }
        }
    } else {
        panic!("Expected Object");
    }
}

#[test]
fn test_eval_class_definition_with_methods() {
    let mut interp = make_interpreter();
    let methods = vec![
        make_method(
            "get_value",
            vec![make_param("self")],
            make_expr(ExprKind::Literal(Literal::Integer(100, None))),
            false,
        ),
        make_method(
            "static_method",
            vec![],
            make_expr(ExprKind::Literal(Literal::Integer(200, None))),
            true,
        ),
    ];

    let result = interp
        .eval_class_definition(
            "MyClass",
            &[],
            None,
            &[],
            &[],
            &[],
            &methods,
            &[],
            &[],
            false,
        )
        .unwrap();

    if let Value::Object(obj) = result {
        if let Some(Value::Object(method_defs)) = obj.get("__methods") {
            assert!(method_defs.contains_key("get_value"));
            assert!(method_defs.contains_key("static_method"));

            // Check static flag
            if let Some(Value::Object(static_meta)) = method_defs.get("static_method") {
                assert_eq!(static_meta.get("is_static"), Some(&Value::Bool(true)));
            }
        }
    } else {
        panic!("Expected Object");
    }
}

#[test]
fn test_eval_class_definition_with_constants() {
    let mut interp = make_interpreter();
    let constants = vec![make_constant(
        "MAX_VALUE",
        make_expr(ExprKind::Literal(Literal::Integer(1000, None))),
    )];

    let result = interp
        .eval_class_definition(
            "Config",
            &[],
            None,
            &[],
            &[],
            &[],
            &[],
            &constants,
            &[],
            false,
        )
        .unwrap();

    if let Value::Object(obj) = result {
        if let Some(Value::Object(const_defs)) = obj.get("__constants") {
            assert!(const_defs.contains_key("MAX_VALUE"));
        }
    }

    // Also check that constant is accessible via qualified name
    let const_val = interp.lookup_variable("Config::MAX_VALUE").unwrap();
    assert_eq!(const_val, Value::Integer(1000));
}

#[test]
fn test_instantiate_class_with_constructor_success() {
    let mut interp = make_interpreter();

    // Define a class with a field and constructor that returns a struct literal
    let fields = vec![make_struct_field("value", make_type("i32"))];

    let constructors = vec![make_constructor(
        Some("new"),
        vec![],
        make_struct_literal(
            "Simple",
            vec![(
                "value",
                make_expr(ExprKind::Literal(Literal::Integer(0, None))),
            )],
        ),
    )];

    interp
        .eval_class_definition(
            "Simple",
            &[],
            None,
            &[],
            &fields,
            &constructors,
            &[],
            &[],
            &[],
            false,
        )
        .unwrap();

    // Instantiate it
    let result = interp
        .instantiate_class_with_constructor("Simple", "new", &[])
        .unwrap();

    // Should be ObjectMut
    if let Value::ObjectMut(cell) = result {
        let obj = cell.lock().unwrap();
        assert_eq!(
            obj.get("__class"),
            Some(&Value::from_string("Simple".to_string()))
        );
    } else {
        panic!("Expected ObjectMut, got {:?}", result);
    }
}

#[test]
fn test_instantiate_class_with_constructor_and_args() {
    let mut interp = make_interpreter();

    // Define a class with fields and a constructor
    let fields = vec![
        make_struct_field("x", make_type("i32")),
        make_struct_field("y", make_type("i32")),
    ];
    let constructors = vec![make_constructor(
        Some("new"),
        vec![make_param("x"), make_param("y")],
        make_struct_literal(
            "Point",
            vec![
                ("x", make_expr(ExprKind::Identifier("x".to_string()))),
                ("y", make_expr(ExprKind::Identifier("y".to_string()))),
            ],
        ),
    )];

    interp
        .eval_class_definition(
            "Point",
            &[],
            None,
            &[],
            &fields,
            &constructors,
            &[],
            &[],
            &[],
            false,
        )
        .unwrap();

    // Instantiate with arguments
    let result = interp
        .instantiate_class_with_constructor(
            "Point",
            "new",
            &[Value::Integer(10), Value::Integer(20)],
        )
        .unwrap();

    if let Value::ObjectMut(cell) = result {
        let obj = cell.lock().unwrap();
        assert_eq!(
            obj.get("__class"),
            Some(&Value::from_string("Point".to_string()))
        );
    } else {
        panic!("Expected ObjectMut");
    }
}

#[test]
fn test_instantiate_class_with_constructor_wrong_arg_count() {
    let mut interp = make_interpreter();

    let constructors = vec![make_constructor(
        Some("new"),
        vec![make_param("x")],
        make_expr(ExprKind::Block(vec![])),
    )];

    interp
        .eval_class_definition(
            "OneArg",
            &[],
            None,
            &[],
            &[],
            &constructors,
            &[],
            &[],
            &[],
            false,
        )
        .unwrap();

    // Wrong number of arguments
    let result = interp.instantiate_class_with_constructor("OneArg", "new", &[]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("expects 1 arguments"));
}

#[test]
fn test_instantiate_class_with_args_success() {
    let mut interp = make_interpreter();

    let fields = vec![make_struct_field("count", make_type("i32"))];
    let constructors = vec![make_constructor(
        Some("init"),
        vec![],
        make_expr(ExprKind::Block(vec![])),
    )];

    interp
        .eval_class_definition(
            "Counter",
            &[],
            None,
            &[],
            &fields,
            &constructors,
            &[],
            &[],
            &[],
            false,
        )
        .unwrap();

    let result = interp.instantiate_class_with_args("Counter", &[]).unwrap();

    // Should be Value::Class
    if let Value::Class { class_name, .. } = result {
        assert_eq!(class_name, "Counter");
    } else {
        panic!("Expected Value::Class, got {:?}", result);
    }
}

#[test]
fn test_instantiate_class_with_args_wrong_count() {
    let mut interp = make_interpreter();

    let constructors = vec![make_constructor(
        Some("init"),
        vec![make_param("value")],
        make_expr(ExprKind::Block(vec![])),
    )];

    interp
        .eval_class_definition(
            "NeedsArg",
            &[],
            None,
            &[],
            &[],
            &constructors,
            &[],
            &[],
            &[],
            false,
        )
        .unwrap();

    let result = interp.instantiate_class_with_args("NeedsArg", &[]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("expects 1 arguments"));
}

#[test]
fn test_eval_class_instance_method_success() {
    let mut interp = make_interpreter();

    // Create a method that returns a literal
    let methods = vec![make_method(
        "get_value",
        vec![], // self is filtered out automatically
        make_expr(ExprKind::Literal(Literal::Integer(42, None))),
        false,
    )];

    interp
        .eval_class_definition(
            "Getter",
            &[],
            None,
            &[],
            &[],
            &[],
            &methods,
            &[],
            &[],
            false,
        )
        .unwrap();

    let instance = HashMap::new();
    let result = interp
        .eval_class_instance_method(&instance, "Getter", "get_value", &[])
        .unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_eval_class_instance_method_with_args() {
    let mut interp = make_interpreter();

    // Method that takes an argument and returns it
    let methods = vec![make_method(
        "echo",
        vec![make_param("x")],
        make_expr(ExprKind::Identifier("x".to_string())),
        false,
    )];

    interp
        .eval_class_definition("Echo", &[], None, &[], &[], &[], &methods, &[], &[], false)
        .unwrap();

    let instance = HashMap::new();
    let result = interp
        .eval_class_instance_method(&instance, "Echo", "echo", &[Value::Integer(99)])
        .unwrap();
    assert_eq!(result, Value::Integer(99));
}

#[test]
fn test_eval_class_instance_method_wrong_arg_count() {
    let mut interp = make_interpreter();

    let methods = vec![make_method(
        "need_one",
        vec![make_param("x")],
        make_expr(ExprKind::Literal(Literal::Integer(0, None))),
        false,
    )];

    interp
        .eval_class_definition(
            "NeedOne",
            &[],
            None,
            &[],
            &[],
            &[],
            &methods,
            &[],
            &[],
            false,
        )
        .unwrap();

    let instance = HashMap::new();
    let result = interp.eval_class_instance_method(&instance, "NeedOne", "need_one", &[]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("expects 1 arguments"));
}

#[test]
fn test_eval_class_instance_method_on_static() {
    let mut interp = make_interpreter();

    let methods = vec![make_method(
        "static_fn",
        vec![],
        make_expr(ExprKind::Literal(Literal::Integer(0, None))),
        true, // is_static = true
    )];

    interp
        .eval_class_definition(
            "HasStatic",
            &[],
            None,
            &[],
            &[],
            &[],
            &methods,
            &[],
            &[],
            false,
        )
        .unwrap();

    let instance = HashMap::new();
    let result = interp.eval_class_instance_method(&instance, "HasStatic", "static_fn", &[]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Cannot call static method"));
}

#[test]
fn test_call_static_method_success() {
    let mut interp = make_interpreter();

    let methods = vec![make_method(
        "create",
        vec![],
        make_expr(ExprKind::Literal(Literal::Integer(999, None))),
        true, // is_static = true
    )];

    interp
        .eval_class_definition(
            "Factory",
            &[],
            None,
            &[],
            &[],
            &[],
            &methods,
            &[],
            &[],
            false,
        )
        .unwrap();

    let result = interp.call_static_method("Factory", "create", &[]).unwrap();
    assert_eq!(result, Value::Integer(999));
}

#[test]
fn test_call_static_method_with_args() {
    let mut interp = make_interpreter();

    let methods = vec![make_method(
        "add",
        vec![make_param("a"), make_param("b")],
        make_expr(ExprKind::Identifier("a".to_string())), // Just return a for simplicity
        true,
    )];

    interp
        .eval_class_definition("Math", &[], None, &[], &[], &[], &methods, &[], &[], false)
        .unwrap();

    let result = interp
        .call_static_method("Math", "add", &[Value::Integer(10), Value::Integer(20)])
        .unwrap();
    assert_eq!(result, Value::Integer(10));
}

#[test]
fn test_call_static_method_wrong_arg_count() {
    let mut interp = make_interpreter();

    let methods = vec![make_method(
        "need_two",
        vec![make_param("a"), make_param("b")],
        make_expr(ExprKind::Literal(Literal::Integer(0, None))),
        true,
    )];

    interp
        .eval_class_definition(
            "NeedTwo",
            &[],
            None,
            &[],
            &[],
            &[],
            &methods,
            &[],
            &[],
            false,
        )
        .unwrap();

    let result = interp.call_static_method("NeedTwo", "need_two", &[Value::Integer(1)]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("expects 2 arguments"));
}

#[test]
fn test_call_static_method_on_non_static() {
    let mut interp = make_interpreter();

    let methods = vec![make_method(
        "instance_method",
        vec![],
        make_expr(ExprKind::Literal(Literal::Integer(0, None))),
        false, // is_static = false
    )];

    interp
        .eval_class_definition(
            "HasInstance",
            &[],
            None,
            &[],
            &[],
            &[],
            &methods,
            &[],
            &[],
            false,
        )
        .unwrap();

    let result = interp.call_static_method("HasInstance", "instance_method", &[]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("not a static method"));
}

#[test]
fn test_instantiate_class_with_superclass_fields() {
    let mut interp = make_interpreter();

    // Define parent class with a field
    let parent_fields = vec![make_struct_field_with_default(
        "parent_val",
        make_type("i32"),
        make_expr(ExprKind::Literal(Literal::Integer(100, None))),
    )];
    // Parent constructor returns struct literal
    let parent_constructors = vec![make_constructor(
        Some("new"),
        vec![],
        make_struct_literal(
            "Parent",
            vec![(
                "parent_val",
                make_expr(ExprKind::Literal(Literal::Integer(100, None))),
            )],
        ),
    )];
    interp
        .eval_class_definition(
            "Parent",
            &[],
            None,
            &[],
            &parent_fields,
            &parent_constructors,
            &[],
            &[],
            &[],
            false,
        )
        .unwrap();

    // Define child class with both parent and child fields (inheritance not fully supported)
    let child_fields = vec![
        make_struct_field_with_default(
            "parent_val",
            make_type("i32"),
            make_expr(ExprKind::Literal(Literal::Integer(100, None))),
        ),
        make_struct_field_with_default(
            "child_val",
            make_type("i32"),
            make_expr(ExprKind::Literal(Literal::Integer(200, None))),
        ),
    ];
    let parent_name = "Parent".to_string();
    // Child constructor returns struct literal with both fields
    let child_constructors = vec![make_constructor(
        Some("new"),
        vec![],
        make_struct_literal(
            "Child",
            vec![
                (
                    "parent_val",
                    make_expr(ExprKind::Literal(Literal::Integer(100, None))),
                ),
                (
                    "child_val",
                    make_expr(ExprKind::Literal(Literal::Integer(200, None))),
                ),
            ],
        ),
    )];
    interp
        .eval_class_definition(
            "Child",
            &[],
            Some(&parent_name),
            &[],
            &child_fields,
            &child_constructors,
            &[],
            &[],
            &[],
            false,
        )
        .unwrap();

    // Instantiate child
    let result = interp
        .instantiate_class_with_constructor("Child", "new", &[])
        .unwrap();

    if let Value::ObjectMut(cell) = result {
        let obj = cell.lock().unwrap();
        // Should have both parent and child fields
        assert_eq!(obj.get("parent_val"), Some(&Value::Integer(100)));
        assert_eq!(obj.get("child_val"), Some(&Value::Integer(200)));
    } else {
        panic!("Expected ObjectMut");
    }
}

#[test]
fn test_eval_class_with_method_override_flag() {
    let mut interp = make_interpreter();

    let mut override_method = make_method(
        "overridden",
        vec![],
        make_expr(ExprKind::Literal(Literal::Integer(0, None))),
        false,
    );
    override_method.is_override = true;

    let methods = vec![override_method];

    let result = interp
        .eval_class_definition(
            "Subclass",
            &[],
            None,
            &[],
            &[],
            &[],
            &methods,
            &[],
            &[],
            false,
        )
        .unwrap();

    if let Value::Object(obj) = result {
        if let Some(Value::Object(method_defs)) = obj.get("__methods") {
            if let Some(Value::Object(method_meta)) = method_defs.get("overridden") {
                assert_eq!(method_meta.get("is_override"), Some(&Value::Bool(true)));
            }
        }
    }
}

#[test]
fn test_instantiate_class_with_args_has_methods() {
    let mut interp = make_interpreter();

    let methods = vec![make_method(
        "do_something",
        vec![],
        make_expr(ExprKind::Literal(Literal::Integer(42, None))),
        false,
    )];

    interp
        .eval_class_definition(
            "WithMethods",
            &[],
            None,
            &[],
            &[],
            &[],
            &methods,
            &[],
            &[],
            false,
        )
        .unwrap();

    let result = interp
        .instantiate_class_with_args("WithMethods", &[])
        .unwrap();

    if let Value::Class { methods: m, .. } = result {
        assert!(m.contains_key("do_something"));
    } else {
        panic!("Expected Value::Class");
    }
}

#[test]
fn test_instantiate_class_with_args_has_field_defaults() {
    let mut interp = make_interpreter();

    let fields = vec![make_struct_field_with_default(
        "initialized",
        make_type("i32"),
        make_expr(ExprKind::Literal(Literal::Integer(777, None))),
    )];

    interp
        .eval_class_definition(
            "WithDefaults",
            &[],
            None,
            &[],
            &fields,
            &[],
            &[],
            &[],
            &[],
            false,
        )
        .unwrap();

    let result = interp
        .instantiate_class_with_args("WithDefaults", &[])
        .unwrap();

    if let Value::Class { fields: f, .. } = result {
        let fields_guard = f.read().unwrap();
        assert_eq!(fields_guard.get("initialized"), Some(&Value::Integer(777)));
    } else {
        panic!("Expected Value::Class");
    }
}
