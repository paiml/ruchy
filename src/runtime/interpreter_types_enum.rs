//! Enum and impl block definitions
//!
//! Extracted from interpreter_types_impl.rs for coverage attribution.

#![allow(clippy::unused_self)]
#![allow(clippy::expect_used)]

use crate::frontend::ast::Expr;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::{InterpreterError, Value};
use std::cell::RefCell;
use std::rc::Rc;

impl Interpreter {
    /// Evaluate enum definition
    /// Stores enum type with variant definitions in the environment
    /// Complexity: 6
    pub(crate) fn eval_enum_definition(
        &mut self,
        name: &str,
        _type_params: &[String], // Generic type parameters (not yet used in runtime)
        variants: &[crate::frontend::ast::EnumVariant],
        _is_pub: bool,
    ) -> Result<Value, InterpreterError> {
        use std::collections::HashMap;

        // Create an enum type object
        let mut enum_type = HashMap::new();

        // Store enum metadata
        enum_type.insert("__type".to_string(), Value::from_string("Enum".to_string()));
        enum_type.insert("__name".to_string(), Value::from_string(name.to_string()));

        // Store variant definitions
        let mut variant_defs = HashMap::new();
        for variant in variants {
            let mut variant_info = HashMap::new();

            // Store variant kind
            let kind_str = match &variant.kind {
                crate::frontend::ast::EnumVariantKind::Unit => "Unit",
                crate::frontend::ast::EnumVariantKind::Tuple(_) => "Tuple",
                crate::frontend::ast::EnumVariantKind::Struct(_) => "Struct",
            };
            variant_info.insert("kind".to_string(), Value::from_string(kind_str.to_string()));

            // Store discriminant if present
            if let Some(disc) = variant.discriminant {
                variant_info.insert("discriminant".to_string(), Value::Integer(disc));
            }

            variant_defs.insert(
                variant.name.clone(),
                Value::Object(std::sync::Arc::new(variant_info)),
            );
        }

        enum_type.insert(
            "__variants".to_string(),
            Value::Object(std::sync::Arc::new(variant_defs)),
        );

        // Register this enum type in the environment
        let enum_obj = Value::Object(std::sync::Arc::new(enum_type));
        self.set_variable(name, enum_obj.clone());

        Ok(enum_obj)
    }

    pub(crate) fn eval_impl_block(
        &mut self,
        for_type: &str,
        methods: &[crate::frontend::ast::ImplMethod],
    ) -> Result<Value, InterpreterError> {
        use std::collections::HashMap;
        use std::sync::Arc;

        // For struct impl blocks, we need to register methods that can be called on instances
        // We'll store them in a special registry keyed by type name
        let mut impl_methods = HashMap::new();

        for method in methods {
            // RUNTIME-DEFAULT-PARAMS: Extract both param names AND default values
            let params_with_defaults: Vec<(String, Option<Arc<Expr>>)> = method
                .params
                .iter()
                .map(|p| {
                    let name = match &p.pattern {
                        crate::frontend::ast::Pattern::Identifier(name) => name.clone(),
                        _ => "_".to_string(), // For other patterns, use placeholder
                    };
                    let default = p
                        .default_value
                        .clone()
                        .map(|expr| Arc::new((*expr).clone()));
                    (name, default)
                })
                .collect();

            // Convert ImplMethod to a Value::Closure
            let closure = Value::Closure {
                params: params_with_defaults,
                body: Arc::new(*method.body.clone()),
                env: Rc::new(RefCell::new(HashMap::new())), // ISSUE-119: Empty environment
            };
            impl_methods.insert(method.name.clone(), closure);
        }

        // Store the impl methods in a global registry
        // For now, we'll just add them to the environment with qualified names
        for (method_name, method_closure) in impl_methods {
            let qualified_name = format!("{}::{}", for_type, method_name);
            self.set_variable(&qualified_name, method_closure);
        }

        Ok(Value::Nil) // impl blocks don't return values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{
        EnumVariant, EnumVariantKind, ImplMethod, Param, Pattern, Span, Type, TypeKind,
    };

    fn make_interpreter() -> Interpreter {
        Interpreter::new()
    }

    fn make_unit_variant(name: &str) -> EnumVariant {
        EnumVariant {
            name: name.to_string(),
            kind: EnumVariantKind::Unit,
            discriminant: None,
        }
    }

    fn make_variant_with_discriminant(name: &str, disc: i64) -> EnumVariant {
        EnumVariant {
            name: name.to_string(),
            kind: EnumVariantKind::Unit,
            discriminant: Some(disc),
        }
    }

    fn make_type(name: &str) -> Type {
        Type {
            kind: TypeKind::Named(name.to_string()),
            span: Span::default(),
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

    #[test]
    fn test_eval_enum_definition_empty() {
        let mut interp = make_interpreter();
        let result = interp
            .eval_enum_definition("Empty", &[], &[], false)
            .unwrap();

        if let Value::Object(obj) = result {
            assert_eq!(
                obj.get("__type"),
                Some(&Value::from_string("Enum".to_string()))
            );
            assert_eq!(
                obj.get("__name"),
                Some(&Value::from_string("Empty".to_string()))
            );
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_enum_definition_with_variants() {
        let mut interp = make_interpreter();
        let variants = vec![
            make_unit_variant("Red"),
            make_unit_variant("Green"),
            make_unit_variant("Blue"),
        ];

        let result = interp
            .eval_enum_definition("Color", &[], &variants, false)
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(variants_obj)) = obj.get("__variants") {
                assert!(variants_obj.contains_key("Red"));
                assert!(variants_obj.contains_key("Green"));
                assert!(variants_obj.contains_key("Blue"));
            } else {
                panic!("Expected __variants Object");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_enum_definition_with_discriminants() {
        let mut interp = make_interpreter();
        let variants = vec![
            make_variant_with_discriminant("A", 0),
            make_variant_with_discriminant("B", 10),
            make_variant_with_discriminant("C", 20),
        ];

        let result = interp
            .eval_enum_definition("TestEnum", &[], &variants, false)
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(variants_obj)) = obj.get("__variants") {
                if let Some(Value::Object(b_info)) = variants_obj.get("B") {
                    assert_eq!(b_info.get("discriminant"), Some(&Value::Integer(10)));
                } else {
                    panic!("Expected B variant");
                }
            } else {
                panic!("Expected __variants Object");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_impl_block_empty() {
        let mut interp = make_interpreter();
        let result = interp.eval_impl_block("TestType", &[]).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_impl_block_with_method() {
        let mut interp = make_interpreter();

        // Create a simple method
        let method = ImplMethod {
            name: "foo".to_string(),
            params: vec![make_param("self")],
            return_type: None,
            body: Box::new(Expr::new(
                crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Integer(
                    42, None,
                )),
                Span::default(),
            )),
            is_pub: true,
        };

        let result = interp.eval_impl_block("TestType", &[method]).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify method was registered
        let method_val = interp.lookup_variable("TestType::foo");
        assert!(method_val.is_ok());
        assert!(matches!(method_val.unwrap(), Value::Closure { .. }));
    }

    #[test]
    fn test_eval_impl_block_multiple_methods() {
        let mut interp = make_interpreter();

        let method1 = ImplMethod {
            name: "method1".to_string(),
            params: vec![make_param("self")],
            return_type: None,
            body: Box::new(Expr::new(
                crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Integer(
                    1, None,
                )),
                Span::default(),
            )),
            is_pub: true,
        };

        let method2 = ImplMethod {
            name: "method2".to_string(),
            params: vec![make_param("self"), make_param("x")],
            return_type: None,
            body: Box::new(Expr::new(
                crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Integer(
                    2, None,
                )),
                Span::default(),
            )),
            is_pub: true,
        };

        let result = interp
            .eval_impl_block("MyStruct", &[method1, method2])
            .unwrap();
        assert_eq!(result, Value::Nil);

        // Verify both methods were registered
        assert!(interp.lookup_variable("MyStruct::method1").is_ok());
        assert!(interp.lookup_variable("MyStruct::method2").is_ok());
    }

    #[test]
    fn test_eval_enum_tuple_variant() {
        let mut interp = make_interpreter();
        let variants = vec![EnumVariant {
            name: "Point".to_string(),
            kind: EnumVariantKind::Tuple(vec![make_type("i32"), make_type("i32")]),
            discriminant: None,
        }];

        let result = interp
            .eval_enum_definition("MyEnum", &[], &variants, false)
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(variants_obj)) = obj.get("__variants") {
                if let Some(Value::Object(point_info)) = variants_obj.get("Point") {
                    assert_eq!(
                        point_info.get("kind"),
                        Some(&Value::from_string("Tuple".to_string()))
                    );
                } else {
                    panic!("Expected Point variant");
                }
            } else {
                panic!("Expected __variants Object");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_enum_struct_variant() {
        let mut interp = make_interpreter();

        use crate::frontend::ast::{StructField, Visibility};
        let struct_fields = vec![StructField {
            name: "x".to_string(),
            ty: make_type("i32"),
            default_value: None,
            is_mut: false,
            visibility: Visibility::Public,
            decorators: vec![],
        }];

        let variants = vec![EnumVariant {
            name: "Named".to_string(),
            kind: EnumVariantKind::Struct(struct_fields),
            discriminant: None,
        }];

        let result = interp
            .eval_enum_definition("MyEnum", &[], &variants, false)
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(variants_obj)) = obj.get("__variants") {
                if let Some(Value::Object(named_info)) = variants_obj.get("Named") {
                    assert_eq!(
                        named_info.get("kind"),
                        Some(&Value::from_string("Struct".to_string()))
                    );
                } else {
                    panic!("Expected Named variant");
                }
            } else {
                panic!("Expected __variants Object");
            }
        } else {
            panic!("Expected Object");
        }
    }

    // =========================================================================
    // Additional coverage tests
    // =========================================================================

    #[test]
    fn test_eval_enum_mixed_variants() {
        use crate::frontend::ast::{StructField, Visibility};

        let mut interp = make_interpreter();

        // Mix of Unit, Tuple, and Struct variants
        let variants = vec![
            EnumVariant {
                name: "None".to_string(),
                kind: EnumVariantKind::Unit,
                discriminant: None,
            },
            EnumVariant {
                name: "Point".to_string(),
                kind: EnumVariantKind::Tuple(vec![make_type("i32"), make_type("i32")]),
                discriminant: None,
            },
            EnumVariant {
                name: "Named".to_string(),
                kind: EnumVariantKind::Struct(vec![StructField {
                    name: "value".to_string(),
                    ty: make_type("String"),
                    default_value: None,
                    is_mut: false,
                    visibility: Visibility::Public,
                    decorators: vec![],
                }]),
                discriminant: None,
            },
        ];

        let result = interp
            .eval_enum_definition("MixedEnum", &[], &variants, false)
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(variants_obj)) = obj.get("__variants") {
                // Verify all three kinds are present
                if let Some(Value::Object(none_info)) = variants_obj.get("None") {
                    assert_eq!(
                        none_info.get("kind"),
                        Some(&Value::from_string("Unit".to_string()))
                    );
                }
                if let Some(Value::Object(point_info)) = variants_obj.get("Point") {
                    assert_eq!(
                        point_info.get("kind"),
                        Some(&Value::from_string("Tuple".to_string()))
                    );
                }
                if let Some(Value::Object(named_info)) = variants_obj.get("Named") {
                    assert_eq!(
                        named_info.get("kind"),
                        Some(&Value::from_string("Struct".to_string()))
                    );
                }
            }
        }
    }

    #[test]
    fn test_eval_enum_registered_in_environment() {
        let mut interp = make_interpreter();
        let variants = vec![make_unit_variant("Value")];

        interp
            .eval_enum_definition("MyEnum", &[], &variants, false)
            .unwrap();

        // Verify enum is accessible from environment
        let result = interp.lookup_variable("MyEnum");
        assert!(result.is_ok());
        if let Value::Object(obj) = result.unwrap() {
            assert_eq!(
                obj.get("__type"),
                Some(&Value::from_string("Enum".to_string()))
            );
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_enum_with_type_params() {
        let mut interp = make_interpreter();
        let variants = vec![
            EnumVariant {
                name: "Some".to_string(),
                kind: EnumVariantKind::Tuple(vec![make_type("T")]),
                discriminant: None,
            },
            make_unit_variant("None"),
        ];

        // Even though type params aren't used at runtime, they should be accepted
        let result = interp
            .eval_enum_definition("Option", &["T".to_string()], &variants, false)
            .unwrap();

        if let Value::Object(obj) = result {
            assert_eq!(
                obj.get("__name"),
                Some(&Value::from_string("Option".to_string()))
            );
        }
    }

    #[test]
    fn test_eval_impl_method_with_wildcard_pattern() {
        let mut interp = make_interpreter();

        // Method with wildcard pattern (non-Identifier) - tests the "_" fallback branch
        let method = ImplMethod {
            name: "wildcard_method".to_string(),
            params: vec![Param {
                pattern: Pattern::Wildcard,
                ty: make_type("Any"),
                span: Span::default(),
                is_mutable: false,
                default_value: None,
            }],
            return_type: None,
            body: Box::new(Expr::new(
                crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Integer(
                    0, None,
                )),
                Span::default(),
            )),
            is_pub: true,
        };

        let result = interp.eval_impl_block("WildcardType", &[method]).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify method was registered
        let method_val = interp.lookup_variable("WildcardType::wildcard_method");
        assert!(method_val.is_ok());
    }

    #[test]
    fn test_eval_impl_method_with_default_param() {
        let mut interp = make_interpreter();

        // Method with default parameter value
        let default_expr = Box::new(Expr::new(
            crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Integer(
                42, None,
            )),
            Span::default(),
        ));

        let method = ImplMethod {
            name: "with_default".to_string(),
            params: vec![
                make_param("self"),
                Param {
                    pattern: Pattern::Identifier("x".to_string()),
                    ty: make_type("i32"),
                    span: Span::default(),
                    is_mutable: false,
                    default_value: Some(default_expr),
                },
            ],
            return_type: None,
            body: Box::new(Expr::new(
                crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Integer(
                    0, None,
                )),
                Span::default(),
            )),
            is_pub: true,
        };

        let result = interp.eval_impl_block("DefaultParam", &[method]).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify method was registered with params
        let method_val = interp.lookup_variable("DefaultParam::with_default").unwrap();
        if let Value::Closure { params, .. } = method_val {
            assert_eq!(params.len(), 2);
            // Second param should have a default
            assert!(params[1].1.is_some());
        } else {
            panic!("Expected Closure");
        }
    }

    #[test]
    fn test_eval_enum_variant_no_discriminant() {
        let mut interp = make_interpreter();
        let variants = vec![make_unit_variant("NoDisc")];

        let result = interp
            .eval_enum_definition("NoDiscEnum", &[], &variants, false)
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(variants_obj)) = obj.get("__variants") {
                if let Some(Value::Object(info)) = variants_obj.get("NoDisc") {
                    // Should not have discriminant key
                    assert!(info.get("discriminant").is_none());
                }
            }
        }
    }

    #[test]
    fn test_eval_impl_is_pub_flag() {
        let mut interp = make_interpreter();

        // Test that is_pub flag is accepted (even if not used at runtime)
        let method = ImplMethod {
            name: "private_method".to_string(),
            params: vec![make_param("self")],
            return_type: None,
            body: Box::new(Expr::new(
                crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Integer(
                    0, None,
                )),
                Span::default(),
            )),
            is_pub: false, // Private method
        };

        let result = interp.eval_impl_block("PrivType", &[method]).unwrap();
        assert_eq!(result, Value::Nil);

        // Method should still be registered
        assert!(interp.lookup_variable("PrivType::private_method").is_ok());
    }

    #[test]
    fn test_eval_enum_variant_count() {
        let mut interp = make_interpreter();
        let variants = vec![
            make_unit_variant("A"),
            make_unit_variant("B"),
            make_unit_variant("C"),
            make_unit_variant("D"),
            make_unit_variant("E"),
        ];

        let result = interp
            .eval_enum_definition("FiveVariants", &[], &variants, false)
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(variants_obj)) = obj.get("__variants") {
                assert_eq!(variants_obj.len(), 5);
            }
        }
    }

    #[test]
    fn test_eval_impl_method_with_return_type() {
        let mut interp = make_interpreter();

        let method = ImplMethod {
            name: "typed_return".to_string(),
            params: vec![make_param("self")],
            return_type: Some(make_type("i32")), // Explicit return type
            body: Box::new(Expr::new(
                crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Integer(
                    42, None,
                )),
                Span::default(),
            )),
            is_pub: true,
        };

        let result = interp.eval_impl_block("TypedReturn", &[method]).unwrap();
        assert_eq!(result, Value::Nil);

        // Method should still be registered
        assert!(interp.lookup_variable("TypedReturn::typed_return").is_ok());
    }
}
