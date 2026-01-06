//! Tests for type transpilation
//! EXTREME TDD Round 84: Extracted from types.rs
//!
//! This module contains comprehensive tests for type handling.

use crate::backend::transpiler::Transpiler;
use crate::frontend::ast::{ClassMethod, Constructor, Expr, ExprKind, Literal, Param, Span, StructField, Type, TypeKind};
use crate::frontend::parser::Parser;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

fn create_transpiler() -> Transpiler {
    Transpiler::new()
}


    #[test]
    fn test_transpile_result_helpers() {
        let helpers = Transpiler::generate_result_helpers();
        let code = helpers.to_string();
        // Check that the ResultExt trait is generated
        assert!(code.contains("trait ResultExt"));
        assert!(code.contains("map_err_with"));
        assert!(code.contains("unwrap_or_else_with"));
        assert!(code.contains("and_then_with"));
        assert!(code.contains("or_else_with"));
    }

    #[test]
    fn test_transpile_named_types() {
        let transpiler = Transpiler::new();

        // Test int type
        let int_type = Type {
            kind: crate::frontend::ast::TypeKind::Named("int".to_string()),
            span: crate::frontend::ast::Span::new(0, 3),
        };
        let result = transpiler
            .transpile_type(&int_type)
            .expect("operation should succeed in test");
        assert_eq!(result.to_string(), "i64");

        // Test float type
        let float_type = Type {
            kind: crate::frontend::ast::TypeKind::Named("float".to_string()),
            span: crate::frontend::ast::Span::new(0, 5),
        };
        let result = transpiler
            .transpile_type(&float_type)
            .expect("operation should succeed in test");
        assert_eq!(result.to_string(), "f64");

        // Test bool type
        let bool_type = Type {
            kind: crate::frontend::ast::TypeKind::Named("bool".to_string()),
            span: crate::frontend::ast::Span::new(0, 4),
        };
        let result = transpiler
            .transpile_type(&bool_type)
            .expect("operation should succeed in test");
        assert_eq!(result.to_string(), "bool");

        // Test String type
        let string_type = Type {
            kind: crate::frontend::ast::TypeKind::Named("String".to_string()),
            span: crate::frontend::ast::Span::new(0, 6),
        };
        let result = transpiler
            .transpile_type(&string_type)
            .expect("operation should succeed in test");
        assert_eq!(result.to_string(), "String");

        // Test custom type
        let custom_type = Type {
            kind: crate::frontend::ast::TypeKind::Named("MyType".to_string()),
            span: crate::frontend::ast::Span::new(0, 6),
        };
        let result = transpiler
            .transpile_type(&custom_type)
            .expect("operation should succeed in test");
        assert_eq!(result.to_string(), "MyType");
    }

    #[test]
    fn test_transpile_optional_type() {
        let transpiler = Transpiler::new();

        let inner_type = Type {
            kind: crate::frontend::ast::TypeKind::Named("int".to_string()),
            span: crate::frontend::ast::Span::new(0, 3),
        };

        let optional_type = Type {
            kind: crate::frontend::ast::TypeKind::Optional(Box::new(inner_type)),
            span: crate::frontend::ast::Span::new(0, 10),
        };

        let result = transpiler
            .transpile_type(&optional_type)
            .expect("operation should succeed in test");
        assert!(result.to_string().contains("Option"));
        assert!(result.to_string().contains("i64"));
    }

    #[test]
    fn test_transpile_list_type() {
        let transpiler = Transpiler::new();

        let elem_type = Type {
            kind: crate::frontend::ast::TypeKind::Named("int".to_string()),
            span: crate::frontend::ast::Span::new(0, 3),
        };

        let list_type = Type {
            kind: crate::frontend::ast::TypeKind::List(Box::new(elem_type)),
            span: crate::frontend::ast::Span::new(0, 10),
        };

        let result = transpiler
            .transpile_type(&list_type)
            .expect("operation should succeed in test");
        assert!(result.to_string().contains("Vec"));
        assert!(result.to_string().contains("i64"));
    }

    #[test]
    fn test_transpile_tuple_type() {
        let transpiler = Transpiler::new();

        let types = vec![
            Type {
                kind: crate::frontend::ast::TypeKind::Named("int".to_string()),
                span: crate::frontend::ast::Span::new(0, 3),
            },
            Type {
                kind: crate::frontend::ast::TypeKind::Named("bool".to_string()),
                span: crate::frontend::ast::Span::new(0, 4),
            },
        ];

        let tuple_type = Type {
            kind: crate::frontend::ast::TypeKind::Tuple(types),
            span: crate::frontend::ast::Span::new(0, 10),
        };

        let result = transpiler
            .transpile_type(&tuple_type)
            .expect("operation should succeed in test");
        let code = result.to_string();
        assert!(code.contains("i64"));
        assert!(code.contains("bool"));
        assert!(code.contains('(') && code.contains(')'));
    }

    #[test]
    fn test_transpile_array_type() {
        let transpiler = Transpiler::new();

        let elem_type = Type {
            kind: crate::frontend::ast::TypeKind::Named("int".to_string()),
            span: crate::frontend::ast::Span::new(0, 3),
        };

        let array_type = Type {
            kind: crate::frontend::ast::TypeKind::Array {
                elem_type: Box::new(elem_type),
                size: 10,
            },
            span: crate::frontend::ast::Span::new(0, 10),
        };

        let result = transpiler
            .transpile_type(&array_type)
            .expect("operation should succeed in test");
        let code = result.to_string();
        assert!(code.contains('['));
        assert!(code.contains("i64"));
        assert!(code.contains("10"));
    }

    #[test]
    fn test_transpile_reference_type() {
        let transpiler = Transpiler::new();

        let inner_type = Type {
            kind: crate::frontend::ast::TypeKind::Named("String".to_string()),
            span: crate::frontend::ast::Span::new(0, 6),
        };

        // Immutable reference
        let ref_type = Type {
            kind: crate::frontend::ast::TypeKind::Reference {
                is_mut: false,
                lifetime: None,
                inner: Box::new(inner_type.clone()),
            },
            span: crate::frontend::ast::Span::new(0, 10),
        };

        let result = transpiler
            .transpile_type(&ref_type)
            .expect("operation should succeed in test");
        let code = result.to_string();
        assert!(code.contains('&'));
        assert!(code.contains("String"));
        assert!(!code.contains("mut"));

        // Mutable reference
        let mut_ref_type = Type {
            kind: crate::frontend::ast::TypeKind::Reference {
                is_mut: true,
                lifetime: None,
                inner: Box::new(inner_type),
            },
            span: crate::frontend::ast::Span::new(0, 10),
        };

        let result = transpiler
            .transpile_type(&mut_ref_type)
            .expect("operation should succeed in test");
        let code = result.to_string();
        assert!(code.contains('&'));
        assert!(code.contains("mut"));
        assert!(code.contains("String"));
    }

    #[test]
    fn test_transpile_dataframe_series_types() {
        let transpiler = Transpiler::new();

        // DataFrame type
        let df_type = Type {
            kind: crate::frontend::ast::TypeKind::DataFrame { columns: vec![] },
            span: crate::frontend::ast::Span::new(0, 10),
        };

        let result = transpiler
            .transpile_type(&df_type)
            .expect("operation should succeed in test");
        assert!(result.to_string().contains("DataFrame"));

        // Series type
        let series_type = Type {
            kind: crate::frontend::ast::TypeKind::Series {
                dtype: Box::new(Type {
                    kind: crate::frontend::ast::TypeKind::Named("int".to_string()),
                    span: crate::frontend::ast::Span::new(0, 3),
                }),
            },
            span: crate::frontend::ast::Span::new(0, 10),
        };

        let result = transpiler
            .transpile_type(&series_type)
            .expect("operation should succeed in test");
        assert!(result.to_string().contains("Series"));
    }

    #[test]
    fn test_transpile_generic_type() {
        let transpiler = Transpiler::new();

        let params = vec![Type {
            kind: crate::frontend::ast::TypeKind::Named("int".to_string()),
            span: crate::frontend::ast::Span::new(0, 3),
        }];

        let generic_type = Type {
            kind: crate::frontend::ast::TypeKind::Generic {
                base: "Vec".to_string(),
                params,
            },
            span: crate::frontend::ast::Span::new(0, 10),
        };

        let result = transpiler
            .transpile_type(&generic_type)
            .expect("operation should succeed in test");
        let code = result.to_string();
        assert!(code.contains("Vec"));
        assert!(code.contains("i64"));
        assert!(code.contains('<') && code.contains('>'));
    }

    #[test]

    fn test_transpile_function_type() {
        let transpiler = Transpiler::new();

        let params = vec![
            Type {
                kind: crate::frontend::ast::TypeKind::Named("int".to_string()),
                span: crate::frontend::ast::Span::new(0, 3),
            },
            Type {
                kind: crate::frontend::ast::TypeKind::Named("int".to_string()),
                span: crate::frontend::ast::Span::new(0, 3),
            },
        ];

        let ret = Type {
            kind: crate::frontend::ast::TypeKind::Named("bool".to_string()),
            span: crate::frontend::ast::Span::new(0, 4),
        };

        let func_type = Type {
            kind: crate::frontend::ast::TypeKind::Function {
                params,
                ret: Box::new(ret),
            },
            span: crate::frontend::ast::Span::new(0, 20),
        };

        let result = transpiler
            .transpile_type(&func_type)
            .expect("operation should succeed in test");
        let code = result.to_string();
        assert!(code.contains("fn"));
        assert!(code.contains("i64"));
        assert!(code.contains("bool"));
    }

    // Helper: Create Type from TypeKind
    fn make_type(kind: TypeKind) -> Type {
        Type {
            kind,
            span: crate::frontend::ast::Span::new(0, 0),
        }
    }

    // Helper: Create StructField
    fn make_field(name: &str, type_name: &str) -> StructField {
        use crate::frontend::ast::Visibility;
        StructField {
            name: name.to_string(),
            ty: make_type(TypeKind::Named(type_name.to_string())),
            visibility: Visibility::Private,
            is_mut: false,
            default_value: None,
            decorators: vec![],
        }
    }

    // Test 11: has_reference_fields - no references
    #[test]
    fn test_has_reference_fields_none() {
        let transpiler = Transpiler::new();
        let fields = vec![make_field("x", "i32"), make_field("y", "String")];
        assert!(!transpiler.has_reference_fields(&fields));
    }

    // Test 12: has_reference_fields - with reference
    #[test]
    fn test_has_reference_fields_with_ref() {
        use crate::frontend::ast::Visibility;
        let transpiler = Transpiler::new();
        let ref_field = StructField {
            name: "data".to_string(),
            ty: make_type(TypeKind::Reference {
                is_mut: false,
                lifetime: None,
                inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
            }),
            visibility: Visibility::Private,
            is_mut: false,
            default_value: None,
            decorators: vec![],
        };
        assert!(transpiler.has_reference_fields(&[ref_field]));
    }

    // Test 13: has_lifetime_params - no lifetimes
    #[test]
    fn test_has_lifetime_params_none() {
        let transpiler = Transpiler::new();
        let type_params = vec!["T".to_string(), "U".to_string()];
        assert!(!transpiler.has_lifetime_params(&type_params));
    }

    // Test 14: has_lifetime_params - with lifetime
    #[test]
    fn test_has_lifetime_params_with_lifetime() {
        let transpiler = Transpiler::new();
        let type_params = vec!["'a".to_string(), "T".to_string()];
        assert!(transpiler.has_lifetime_params(&type_params));
    }

    // Test 15: generate_derive_attributes - empty
    #[test]
    fn test_generate_derive_attributes_empty() {
        let transpiler = Transpiler::new();
        let result = transpiler.generate_derive_attributes(&[]);
        assert_eq!(result.to_string(), "");
    }

    // Test 16: generate_derive_attributes - single derive
    #[test]
    fn test_generate_derive_attributes_single() {
        let transpiler = Transpiler::new();
        let derives = vec!["Debug".to_string()];
        let result = transpiler.generate_derive_attributes(&derives);
        let code = result.to_string();
        assert!(code.contains("derive"));
        assert!(code.contains("Debug"));
    }

    // Test 17: generate_derive_attributes - multiple derives
    #[test]
    fn test_generate_derive_attributes_multiple() {
        let transpiler = Transpiler::new();
        let derives = vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "PartialEq".to_string(),
        ];
        let result = transpiler.generate_derive_attributes(&derives);
        let code = result.to_string();
        assert!(code.contains("Debug"));
        assert!(code.contains("Clone"));
        assert!(code.contains("PartialEq"));
    }

    // Test 18: generate_class_type_param_tokens - empty
    #[test]
    fn test_generate_class_type_param_tokens_empty() {
        let transpiler = Transpiler::new();
        let result = transpiler.generate_class_type_param_tokens(&[]);
        assert_eq!(result.len(), 0);
    }

    // Test 19: generate_class_type_param_tokens - single param
    #[test]
    fn test_generate_class_type_param_tokens_single() {
        let transpiler = Transpiler::new();
        let type_params = vec!["T".to_string()];
        let result = transpiler.generate_class_type_param_tokens(&type_params);
        assert_eq!(result.len(), 1);
        assert!(result[0].to_string().contains('T'));
    }

    // Test 20: generate_class_type_param_tokens - with lifetime
    #[test]
    fn test_generate_class_type_param_tokens_with_lifetime() {
        let transpiler = Transpiler::new();
        let type_params = vec!["'a".to_string(), "T".to_string()];
        let result = transpiler.generate_class_type_param_tokens(&type_params);
        assert_eq!(result.len(), 2);
        // Lifetime should be first
        assert!(result[0].to_string().contains("'a"));
    }

    // Test 21: transpile_params - empty
    #[test]
    fn test_transpile_params_empty() {
        let transpiler = Transpiler::new();
        let result = transpiler
            .transpile_params(&[])
            .expect("operation should succeed in test");
        assert_eq!(result.len(), 0);
    }

    // Test 22: transpile_params - single param
    #[test]
    fn test_transpile_params_single() {
        let transpiler = Transpiler::new();
        let params = vec![crate::frontend::ast::Param {
            pattern: crate::frontend::ast::Pattern::Identifier("x".to_string()),
            ty: make_type(TypeKind::Named("i32".to_string())),
            span: crate::frontend::ast::Span::new(0, 0),
            is_mutable: false,
            default_value: None,
        }];
        let result = transpiler
            .transpile_params(&params)
            .expect("operation should succeed in test");
        assert_eq!(result.len(), 1);
        let code = result[0].to_string();
        assert!(code.contains('x'));
        assert!(code.contains("i32"));
    }

    // Test 23: transpile_params - multiple params
    #[test]
    fn test_transpile_params_multiple() {
        let transpiler = Transpiler::new();
        let params = vec![
            crate::frontend::ast::Param {
                pattern: crate::frontend::ast::Pattern::Identifier("x".to_string()),
                ty: make_type(TypeKind::Named("i32".to_string())),
                span: crate::frontend::ast::Span::new(0, 0),
                is_mutable: false,
                default_value: None,
            },
            crate::frontend::ast::Param {
                pattern: crate::frontend::ast::Pattern::Identifier("y".to_string()),
                ty: make_type(TypeKind::Named("String".to_string())),
                span: crate::frontend::ast::Span::new(0, 0),
                is_mutable: false,
                default_value: None,
            },
        ];
        let result = transpiler
            .transpile_params(&params)
            .expect("operation should succeed in test");
        assert_eq!(result.len(), 2);
    }

    // Test 24: transpile_params - with mutable param
    #[test]
    fn test_transpile_params_mutable() {
        let transpiler = Transpiler::new();
        let params = vec![crate::frontend::ast::Param {
            pattern: crate::frontend::ast::Pattern::Identifier("x".to_string()),
            ty: make_type(TypeKind::Named("i32".to_string())),
            span: crate::frontend::ast::Span::new(0, 0),
            is_mutable: true,
            default_value: None,
        }];
        let result = transpiler
            .transpile_params(&params)
            .expect("operation should succeed in test");
        let code = result[0].to_string();
        assert!(code.contains("mut"));
        assert!(code.contains('x'));
    }

    // Test 25: transpile_struct - basic struct
    #[test]
    fn test_transpile_struct_basic() {
        let transpiler = Transpiler::new();
        let fields = vec![make_field("x", "i32"), make_field("y", "String")];
        let result = transpiler.transpile_struct("Point", &[], &fields, &[], false);
        assert!(result.is_ok());
        let code = result.expect("result should be Ok in test").to_string();
        assert!(code.contains("struct"));
        assert!(code.contains("Point"));
        assert!(code.contains('x'));
        assert!(code.contains('y'));
    }

    // Test 26: transpile_struct - with derives
    #[test]
    fn test_transpile_struct_with_derives() {
        let transpiler = Transpiler::new();
        let fields = vec![make_field("value", "i32")];
        let derives = vec!["Debug".to_string(), "Clone".to_string()];
        let result = transpiler.transpile_struct("Data", &[], &fields, &derives, false);
        assert!(result.is_ok());
        let code = result.expect("result should be Ok in test").to_string();
        assert!(code.contains("derive"));
        assert!(code.contains("Debug"));
        assert!(code.contains("Clone"));
    }

    // Test 27: transpile_tuple_struct - basic
    #[test]
    fn test_transpile_tuple_struct_basic() {
        let transpiler = Transpiler::new();
        let field_types = vec![
            make_type(TypeKind::Named("i32".to_string())),
            make_type(TypeKind::Named("String".to_string())),
        ];
        let result = transpiler.transpile_tuple_struct("Wrapper", &[], &field_types, &[], false);
        assert!(result.is_ok());
        let code = result.expect("result should be Ok in test").to_string();
        assert!(code.contains("struct"));
        assert!(code.contains("Wrapper"));
        assert!(code.contains("i32"));
        assert!(code.contains("String"));
    }

    // Test 28: transpile_struct_field_type_with_lifetime - with reference type
    #[test]
    fn test_transpile_struct_field_type_with_lifetime_reference() {
        let transpiler = Transpiler::new();
        let ref_type = make_type(TypeKind::Reference {
            is_mut: false,
            lifetime: Some("'old".to_string()),
            inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
        });
        let result = transpiler
            .transpile_struct_field_type_with_lifetime(&ref_type, "'a")
            .expect("operation should succeed in test");
        let code = result.to_string();
        assert!(code.contains("'a")); // Should use new lifetime
        assert!(code.contains("str"));
    }

    // Test 29: transpile_struct_field_type_with_lifetime - with non-reference type
    #[test]
    fn test_transpile_struct_field_type_with_lifetime_non_reference() {
        let transpiler = Transpiler::new();
        let non_ref_type = make_type(TypeKind::Named("String".to_string()));
        let result = transpiler
            .transpile_struct_field_type_with_lifetime(&non_ref_type, "'a")
            .expect("operation should succeed in test");
        let code = result.to_string();
        assert_eq!(code, "String");
        assert!(!code.contains("'a")); // Non-reference type shouldn't get lifetime
    }

    // Test 30: transpile_named_type - with namespaced type (std::io::Error)
    #[test]
    fn test_transpile_named_type_namespaced() {
        let transpiler = Transpiler::new();
        let result = transpiler
            .transpile_named_type("std::io::Error")
            .expect("operation should succeed in test");
        let code = result.to_string();
        assert!(code.contains("std"));
        assert!(code.contains("io"));
        assert!(code.contains("Error"));
    }

    // Test 31: transpile_named_type - with nested namespace (trace::Sampler)
    #[test]
    fn test_transpile_named_type_nested_namespace() {
        let transpiler = Transpiler::new();
        let result = transpiler
            .transpile_named_type("trace::Sampler")
            .expect("operation should succeed in test");
        let code = result.to_string();
        assert!(code.contains("trace"));
        assert!(code.contains("Sampler"));
    }

    // Test 32: transpile_reference_type - with lifetime
    #[test]
    fn test_transpile_reference_type_with_lifetime() {
        let transpiler = Transpiler::new();
        let inner = make_type(TypeKind::Named("String".to_string()));
        let result = transpiler
            .transpile_reference_type(false, Some("'a"), &inner)
            .expect("operation should succeed in test");
        let code = result.to_string();
        assert!(code.contains("'a"));
        assert!(code.contains("String"));
        assert!(!code.contains("mut"));
    }

    // Test 33: transpile_reference_type - mut with lifetime
    #[test]
    fn test_transpile_reference_type_mut_with_lifetime() {
        let transpiler = Transpiler::new();
        // Use Generic instead of Named for Vec<i32>
        let inner = make_type(TypeKind::Generic {
            base: "Vec".to_string(),
            params: vec![make_type(TypeKind::Named("i32".to_string()))],
        });
        let result = transpiler
            .transpile_reference_type(true, Some("'b"), &inner)
            .expect("operation should succeed in test");
        let code = result.to_string();
        assert!(code.contains("'b"));
        assert!(code.contains("mut"));
        assert!(code.contains("Vec"));
    }

    // Test 34: transpile_generic_type - with multiple type params
    #[test]
    fn test_transpile_generic_type_multiple_params() {
        let transpiler = Transpiler::new();
        let params = vec![
            make_type(TypeKind::Named("String".to_string())),
            make_type(TypeKind::Named("i32".to_string())),
            make_type(TypeKind::Named("bool".to_string())),
        ];
        let result = transpiler
            .transpile_generic_type("HashMap", &params)
            .expect("operation should succeed in test");
        let code = result.to_string();
        assert!(code.contains("HashMap"));
        assert!(code.contains("String"));
        assert!(code.contains("i32"));
        assert!(code.contains("bool"));
    }

    // Helper: Create Constructor for testing
    fn make_constructor(name: Option<&str>, return_type: Option<Type>) -> Constructor {
        use crate::frontend::ast::{Expr, ExprKind, Literal};
        Constructor {
            name: name.map(std::string::ToString::to_string),
            params: vec![],
            body: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(0, None)),
                span: crate::frontend::ast::Span::new(0, 0),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            return_type,
            is_pub: true,
        }
    }

    // Test 35: transpile_constructors - with named constructor
    #[test]
    fn test_transpile_constructors_named() {
        let transpiler = Transpiler::new();
        let ctors = vec![make_constructor(Some("from_string"), None)];
        let result = transpiler
            .transpile_constructors(&ctors)
            .expect("operation should succeed in test");
        assert_eq!(result.len(), 1);
        let code = result[0].to_string();
        assert!(code.contains("from_string"));
        assert!(code.contains("pub"));
        assert!(code.contains("Self"));
    }

    // Test 36: transpile_constructors - with return type
    #[test]
    fn test_transpile_constructors_with_return_type() {
        let transpiler = Transpiler::new();
        // Use Generic instead of Named for Result<Self, Error>
        let ret_type = make_type(TypeKind::Generic {
            base: "Result".to_string(),
            params: vec![
                make_type(TypeKind::Named("Self".to_string())),
                make_type(TypeKind::Named("Error".to_string())),
            ],
        });
        let ctors = vec![make_constructor(None, Some(ret_type))];
        let result = transpiler
            .transpile_constructors(&ctors)
            .expect("operation should succeed in test");
        assert_eq!(result.len(), 1);
        let code = result[0].to_string();
        assert!(code.contains("Result"));
        assert!(code.contains("new")); // Default constructor name
    }

    // Helper: Create ClassMethod for testing
    fn make_class_method(name: &str, is_pub: bool) -> ClassMethod {
        use crate::frontend::ast::{Expr, ExprKind, Literal, SelfType};
        ClassMethod {
            name: name.to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(42, None)),
                span: crate::frontend::ast::Span::new(0, 0),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            is_pub,
            is_async: false,
            is_static: false,
            is_override: false,
            is_final: false,
            is_abstract: false,
            self_type: SelfType::None,
        }
    }

    // Test 37: transpile_class_methods - single method
    #[test]
    fn test_transpile_class_methods_single() {
        let transpiler = Transpiler::new();
        let methods = vec![make_class_method("compute", true)];
        let result = transpiler
            .transpile_class_methods(&methods)
            .expect("operation should succeed in test");
        assert_eq!(result.len(), 1);
        let code = result[0].to_string();
        assert!(code.contains("compute"));
        assert!(code.contains("pub"));
        assert!(code.contains("42"));
    }

    // Helper: Create ClassConstant for testing
    fn make_class_constant(name: &str, is_pub: bool) -> crate::frontend::ast::ClassConstant {
        use crate::frontend::ast::{Expr, ExprKind, Literal};
        crate::frontend::ast::ClassConstant {
            name: name.to_string(),
            ty: make_type(TypeKind::Named("i32".to_string())),
            value: Expr {
                kind: ExprKind::Literal(Literal::Integer(100, None)),
                span: crate::frontend::ast::Span::new(0, 0),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            },
            is_pub,
        }
    }

    // Test 38: transpile_class_constants - single constant
    #[test]
    fn test_transpile_class_constants_single() {
        let transpiler = Transpiler::new();
        let constants = vec![make_class_constant("MAX_SIZE", true)];
        let result = transpiler
            .transpile_class_constants(&constants)
            .expect("operation should succeed in test");
        assert_eq!(result.len(), 1);
        let code = result[0].to_string();
        assert!(code.contains("MAX_SIZE"));
        assert!(code.contains("const"));
        assert!(code.contains("pub"));
        assert!(code.contains("100"));
    }

    // Test 39: generate_impl_block - without type params
    #[test]
    fn test_generate_impl_block_no_type_params() {
        let transpiler = Transpiler::new();
        let struct_name = format_ident!("MyStruct");
        let result = transpiler.generate_impl_block(&struct_name, &[], &[], &[], &[]);
        let code = result.to_string();
        assert!(code.contains("impl"));
        assert!(code.contains("MyStruct"));
        assert!(!code.contains('<')); // No angle brackets for type params
    }

    // Test 40: generate_impl_block - with type params
    #[test]
    fn test_generate_impl_block_with_type_params() {
        let transpiler = Transpiler::new();
        let struct_name = format_ident!("MyStruct");
        let type_params = vec![quote! { T }, quote! { U }];
        let result = transpiler.generate_impl_block(&struct_name, &type_params, &[], &[], &[]);
        let code = result.to_string();
        assert!(code.contains("impl"));
        assert!(code.contains('<')); // Has type params
        assert!(code.contains('T'));
        assert!(code.contains('U'));
    }

    // Test 41: generate_default_impl - no defaults (returns empty)
    #[test]
    fn test_generate_default_impl_no_defaults() {
        let transpiler = Transpiler::new();
        let struct_name = format_ident!("NoDefaults");
        let fields = vec![make_field("x", "i32")]; // No default values
        let result = transpiler
            .generate_default_impl(&fields, &struct_name, &[])
            .expect("operation should succeed in test");
        let code = result.to_string();
        assert!(code.is_empty()); // Should return empty TokenStream
    }

    // Test 42: generate_default_impl - with defaults
    #[test]
    fn test_generate_default_impl_with_defaults() {
        use crate::frontend::ast::{Expr, ExprKind, Literal, Visibility};
        let transpiler = Transpiler::new();
        let struct_name = format_ident!("WithDefaults");
        let field_with_default = StructField {
            name: "count".to_string(),
            ty: make_type(TypeKind::Named("i32".to_string())),
            visibility: Visibility::Private,
            is_mut: false,
            default_value: Some(Expr {
                kind: ExprKind::Literal(Literal::Integer(10, None)),
                span: crate::frontend::ast::Span::new(0, 0),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            decorators: vec![],
        };
        let result = transpiler
            .generate_default_impl(&[field_with_default], &struct_name, &[])
            .expect("operation should succeed in test");
        let code = result.to_string();
        assert!(code.contains("impl"));
        assert!(code.contains("Default"));
        assert!(code.contains("default"));
        assert!(code.contains("count"));
        assert!(code.contains("10"));
    }

