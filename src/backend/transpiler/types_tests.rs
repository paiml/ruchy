//! Tests for type transpilation
//! EXTREME TDD Round 84: Extracted from types.rs
//!
//! This module contains comprehensive tests for type handling.

use crate::backend::transpiler::Transpiler;
use crate::frontend::ast::{
    ClassMethod, Constructor, Param, Span, StructField, Type, TypeKind,
};
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

// ============================================================
// EXTREME TDD: Direct tests for pub(crate) type helper methods
// ============================================================

// Test 43: transpile_optional_type - direct call with int inner
#[test]
fn test_direct_transpile_optional_type_int() {
    let transpiler = Transpiler::new();
    let inner = make_type(TypeKind::Named("int".to_string()));
    let result = transpiler
        .transpile_optional_type(&inner)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("Option"));
    assert!(code.contains("i64"));
}

// Test 44: transpile_optional_type - direct call with String inner
#[test]
fn test_direct_transpile_optional_type_string() {
    let transpiler = Transpiler::new();
    let inner = make_type(TypeKind::Named("String".to_string()));
    let result = transpiler
        .transpile_optional_type(&inner)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("Option"));
    assert!(code.contains("String"));
}

// Test 45: transpile_optional_type - direct call with nested generic
#[test]
fn test_direct_transpile_optional_type_nested_generic() {
    let transpiler = Transpiler::new();
    let inner = make_type(TypeKind::Generic {
        base: "Vec".to_string(),
        params: vec![make_type(TypeKind::Named("bool".to_string()))],
    });
    let result = transpiler
        .transpile_optional_type(&inner)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("Option"));
    assert!(code.contains("Vec"));
    assert!(code.contains("bool"));
}

// Test 46: transpile_list_type - direct call with int element
#[test]
fn test_direct_transpile_list_type_int() {
    let transpiler = Transpiler::new();
    let elem = make_type(TypeKind::Named("int".to_string()));
    let result = transpiler
        .transpile_list_type(&elem)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("Vec"));
    assert!(code.contains("i64"));
}

// Test 47: transpile_list_type - direct call with String element
#[test]
fn test_direct_transpile_list_type_string() {
    let transpiler = Transpiler::new();
    let elem = make_type(TypeKind::Named("String".to_string()));
    let result = transpiler
        .transpile_list_type(&elem)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("Vec"));
    assert!(code.contains("String"));
}

// Test 48: transpile_list_type - direct call with custom type element
#[test]
fn test_direct_transpile_list_type_custom() {
    let transpiler = Transpiler::new();
    let elem = make_type(TypeKind::Named("MyStruct".to_string()));
    let result = transpiler
        .transpile_list_type(&elem)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("Vec"));
    assert!(code.contains("MyStruct"));
}

// Test 49: transpile_array_type - direct call with int element and size 5
#[test]
fn test_direct_transpile_array_type_int_5() {
    let transpiler = Transpiler::new();
    let elem = make_type(TypeKind::Named("int".to_string()));
    let result = transpiler
        .transpile_array_type(&elem, 5)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains('['));
    assert!(code.contains("i64"));
    assert!(code.contains('5'));
}

// Test 50: transpile_array_type - direct call with float element and size 100
#[test]
fn test_direct_transpile_array_type_float_100() {
    let transpiler = Transpiler::new();
    let elem = make_type(TypeKind::Named("float".to_string()));
    let result = transpiler
        .transpile_array_type(&elem, 100)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains('['));
    assert!(code.contains("f64"));
    assert!(code.contains("100"));
}

// Test 51: transpile_array_type - direct call with zero size
#[test]
fn test_direct_transpile_array_type_zero_size() {
    let transpiler = Transpiler::new();
    let elem = make_type(TypeKind::Named("bool".to_string()));
    let result = transpiler
        .transpile_array_type(&elem, 0)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains('['));
    assert!(code.contains("bool"));
    assert!(code.contains('0'));
}

// Test 52: transpile_tuple_type - direct call with two elements
#[test]
fn test_direct_transpile_tuple_type_two() {
    let transpiler = Transpiler::new();
    let types = vec![
        make_type(TypeKind::Named("int".to_string())),
        make_type(TypeKind::Named("String".to_string())),
    ];
    let result = transpiler
        .transpile_tuple_type(&types)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains('('));
    assert!(code.contains(')'));
    assert!(code.contains("i64"));
    assert!(code.contains("String"));
}

// Test 53: transpile_tuple_type - direct call with three elements
#[test]
fn test_direct_transpile_tuple_type_three() {
    let transpiler = Transpiler::new();
    let types = vec![
        make_type(TypeKind::Named("int".to_string())),
        make_type(TypeKind::Named("bool".to_string())),
        make_type(TypeKind::Named("char".to_string())),
    ];
    let result = transpiler
        .transpile_tuple_type(&types)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains('('));
    assert!(code.contains("i64"));
    assert!(code.contains("bool"));
    assert!(code.contains("char"));
}

// Test 54: transpile_tuple_type - direct call with empty (unit type)
#[test]
fn test_direct_transpile_tuple_type_empty() {
    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile_tuple_type(&[])
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains('('));
    assert!(code.contains(')'));
}

// Test 55: transpile_function_type - direct call with single param
#[test]
fn test_direct_transpile_function_type_single_param() {
    let transpiler = Transpiler::new();
    let params = vec![make_type(TypeKind::Named("int".to_string()))];
    let ret = make_type(TypeKind::Named("bool".to_string()));
    let result = transpiler
        .transpile_function_type(&params, &ret)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("fn"));
    assert!(code.contains("i64"));
    assert!(code.contains("bool"));
}

// Test 56: transpile_function_type - direct call with no params
#[test]
fn test_direct_transpile_function_type_no_params() {
    let transpiler = Transpiler::new();
    let ret = make_type(TypeKind::Named("String".to_string()));
    let result = transpiler
        .transpile_function_type(&[], &ret)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("fn"));
    assert!(code.contains("String"));
    assert!(code.contains("()"));
}

// Test 57: transpile_function_type - direct call with multiple params
#[test]
fn test_direct_transpile_function_type_multiple_params() {
    let transpiler = Transpiler::new();
    let params = vec![
        make_type(TypeKind::Named("int".to_string())),
        make_type(TypeKind::Named("String".to_string())),
        make_type(TypeKind::Named("bool".to_string())),
    ];
    let ret = make_type(TypeKind::Named("float".to_string()));
    let result = transpiler
        .transpile_function_type(&params, &ret)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("fn"));
    assert!(code.contains("i64"));
    assert!(code.contains("String"));
    assert!(code.contains("bool"));
    assert!(code.contains("f64"));
}

// Test 58: parse_type_param_to_tokens - simple type param
#[test]
fn test_direct_parse_type_param_simple() {
    let result = Transpiler::parse_type_param_to_tokens("T");
    let code = result.to_string();
    assert!(code.contains('T'));
}

// Test 59: parse_type_param_to_tokens - lifetime param
#[test]
fn test_direct_parse_type_param_lifetime() {
    let result = Transpiler::parse_type_param_to_tokens("'a");
    let code = result.to_string();
    assert!(code.contains("'a"));
}

// Test 60: parse_type_param_to_tokens - type param with single bound
#[test]
fn test_direct_parse_type_param_with_bound() {
    let result = Transpiler::parse_type_param_to_tokens("T: Clone");
    let code = result.to_string();
    assert!(code.contains('T'));
    assert!(code.contains("Clone"));
}

// Test 61: parse_type_param_to_tokens - type param with multiple bounds
#[test]
fn test_direct_parse_type_param_multiple_bounds() {
    let result = Transpiler::parse_type_param_to_tokens("T: Clone + Debug");
    let code = result.to_string();
    assert!(code.contains('T'));
    // The result should contain some bounds info
}

// Test 62: parse_type_param_to_tokens - complex lifetime param
#[test]
fn test_direct_parse_type_param_complex_lifetime() {
    let result = Transpiler::parse_type_param_to_tokens("'static");
    let code = result.to_string();
    assert!(code.contains("'static"));
}

// Test 63: transpile_optional_type - with reference inner
#[test]
fn test_direct_transpile_optional_type_reference() {
    let transpiler = Transpiler::new();
    let inner = make_type(TypeKind::Reference {
        is_mut: false,
        lifetime: None,
        inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
    });
    let result = transpiler
        .transpile_optional_type(&inner)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("Option"));
    assert!(code.contains('&'));
    assert!(code.contains("str"));
}

// Test 64: transpile_list_type - with tuple element
#[test]
fn test_direct_transpile_list_type_tuple() {
    let transpiler = Transpiler::new();
    let elem = make_type(TypeKind::Tuple(vec![
        make_type(TypeKind::Named("int".to_string())),
        make_type(TypeKind::Named("String".to_string())),
    ]));
    let result = transpiler
        .transpile_list_type(&elem)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("Vec"));
    assert!(code.contains('('));
    assert!(code.contains("i64"));
    assert!(code.contains("String"));
}

// Test 65: transpile_array_type - with large size
#[test]
fn test_direct_transpile_array_type_large() {
    let transpiler = Transpiler::new();
    let elem = make_type(TypeKind::Named("u8".to_string()));
    let result = transpiler
        .transpile_array_type(&elem, 1024)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains('['));
    assert!(code.contains("u8"));
    assert!(code.contains("1024"));
}

// Test 66: transpile_tuple_type - with nested tuple
#[test]
fn test_direct_transpile_tuple_type_nested() {
    let transpiler = Transpiler::new();
    let inner_tuple = make_type(TypeKind::Tuple(vec![
        make_type(TypeKind::Named("int".to_string())),
        make_type(TypeKind::Named("int".to_string())),
    ]));
    let types = vec![
        make_type(TypeKind::Named("String".to_string())),
        inner_tuple,
    ];
    let result = transpiler
        .transpile_tuple_type(&types)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("String"));
    assert!(code.contains("i64"));
}

// Test 67: transpile_function_type - with function return type
#[test]
fn test_direct_transpile_function_type_returning_function() {
    let transpiler = Transpiler::new();
    let params = vec![make_type(TypeKind::Named("int".to_string()))];
    let inner_ret = make_type(TypeKind::Named("bool".to_string()));
    let ret = make_type(TypeKind::Function {
        params: vec![make_type(TypeKind::Named("String".to_string()))],
        ret: Box::new(inner_ret),
    });
    let result = transpiler
        .transpile_function_type(&params, &ret)
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("fn"));
    assert!(code.contains("i64"));
}

// Test 68: parse_type_param_to_tokens - with where-style bound
#[test]
fn test_direct_parse_type_param_complex_bound() {
    let result = Transpiler::parse_type_param_to_tokens("T: Iterator<Item = u8>");
    let code = result.to_string();
    assert!(code.contains('T'));
    // Complex bounds should still produce a valid identifier
}

// ============================================================
// EXTREME TDD: More direct tests for enum, trait, impl methods
// ============================================================

// Helper: Create EnumVariant
fn make_enum_variant(
    name: &str,
    kind: crate::frontend::ast::EnumVariantKind,
) -> crate::frontend::ast::EnumVariant {
    crate::frontend::ast::EnumVariant {
        name: name.to_string(),
        kind,
        discriminant: None,
    }
}

// Test 69: transpile_enum - unit variants
#[test]
fn test_transpile_enum_unit_variants() {
    let transpiler = Transpiler::new();
    use crate::frontend::ast::EnumVariantKind;
    let variants = vec![
        make_enum_variant("Red", EnumVariantKind::Unit),
        make_enum_variant("Green", EnumVariantKind::Unit),
        make_enum_variant("Blue", EnumVariantKind::Unit),
    ];
    let result = transpiler.transpile_enum("Color", &[], &variants, true);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("enum"));
    assert!(code.contains("Color"));
    assert!(code.contains("Red"));
    assert!(code.contains("Green"));
    assert!(code.contains("Blue"));
    assert!(code.contains("derive"));
}

// Test 70: transpile_enum - tuple variants
#[test]
fn test_transpile_enum_tuple_variants() {
    let transpiler = Transpiler::new();
    use crate::frontend::ast::EnumVariantKind;
    let variants = vec![make_enum_variant(
        "Point",
        EnumVariantKind::Tuple(vec![
            make_type(TypeKind::Named("i32".to_string())),
            make_type(TypeKind::Named("i32".to_string())),
        ]),
    )];
    let result = transpiler.transpile_enum("Shape", &[], &variants, true);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("Point"));
    assert!(code.contains("i32"));
}

// Test 71: transpile_enum - struct variants
#[test]
fn test_transpile_enum_struct_variants() {
    let transpiler = Transpiler::new();
    use crate::frontend::ast::{EnumVariantKind, Visibility};
    let fields = vec![
        StructField {
            name: "x".to_string(),
            ty: make_type(TypeKind::Named("i32".to_string())),
            visibility: Visibility::Public,
            is_mut: false,
            default_value: None,
            decorators: vec![],
        },
        StructField {
            name: "y".to_string(),
            ty: make_type(TypeKind::Named("i32".to_string())),
            visibility: Visibility::Public,
            is_mut: false,
            default_value: None,
            decorators: vec![],
        },
    ];
    let variants = vec![make_enum_variant("Move", EnumVariantKind::Struct(fields))];
    let result = transpiler.transpile_enum("Command", &[], &variants, true);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("Move"));
    assert!(code.contains('x'));
    assert!(code.contains('y'));
}

// Test 72: transpile_enum - with type params
#[test]
fn test_transpile_enum_with_type_params() {
    let transpiler = Transpiler::new();
    use crate::frontend::ast::EnumVariantKind;
    let variants = vec![
        make_enum_variant(
            "Some",
            EnumVariantKind::Tuple(vec![make_type(TypeKind::Named("T".to_string()))]),
        ),
        make_enum_variant("None", EnumVariantKind::Unit),
    ];
    let result = transpiler.transpile_enum("Option", &["T".to_string()], &variants, true);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("Option"));
    assert!(code.contains('<'));
    assert!(code.contains('>'));
    assert!(code.contains('T'));
}

// Test 73: transpile_enum - with discriminants
#[test]
fn test_transpile_enum_with_discriminants() {
    let transpiler = Transpiler::new();
    use crate::frontend::ast::EnumVariantKind;
    let mut v1 = make_enum_variant("A", EnumVariantKind::Unit);
    v1.discriminant = Some(10);
    let mut v2 = make_enum_variant("B", EnumVariantKind::Unit);
    v2.discriminant = Some(20);
    let variants = vec![v1, v2];
    let result = transpiler.transpile_enum("Values", &[], &variants, false);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("repr"));
    assert!(code.contains("10"));
    assert!(code.contains("20"));
}

// Helper: Create TraitMethod
fn make_trait_method(name: &str, has_body: bool) -> crate::frontend::ast::TraitMethod {
    use crate::frontend::ast::{Expr, ExprKind, Literal};
    crate::frontend::ast::TraitMethod {
        name: name.to_string(),
        params: vec![],
        return_type: Some(make_type(TypeKind::Named("i32".to_string()))),
        body: if has_body {
            Some(Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(42, None)),
                span: Span::new(0, 0),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }))
        } else {
            None
        },
        is_pub: true,
    }
}

// Test 74: transpile_trait - simple trait with methods
#[test]
fn test_transpile_trait_simple() {
    let transpiler = Transpiler::new();
    let methods = vec![make_trait_method("calculate", false)];
    let result = transpiler.transpile_trait("Calculator", &[], &[], &methods, true);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("trait"));
    assert!(code.contains("Calculator"));
    assert!(code.contains("calculate"));
    assert!(code.contains("pub"));
}

// Test 75: transpile_trait - with default implementation
#[test]
fn test_transpile_trait_with_default_impl() {
    let transpiler = Transpiler::new();
    let methods = vec![make_trait_method("get_value", true)];
    let result = transpiler.transpile_trait("Getter", &[], &[], &methods, true);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("trait"));
    assert!(code.contains("Getter"));
    assert!(code.contains("get_value"));
    assert!(code.contains("42")); // Default implementation body
}

// Test 76: transpile_trait - with type params
#[test]
fn test_transpile_trait_with_type_params() {
    let transpiler = Transpiler::new();
    let methods = vec![make_trait_method("process", false)];
    let result = transpiler.transpile_trait("Processor", &["T".to_string()], &[], &methods, true);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("trait"));
    assert!(code.contains("Processor"));
    assert!(code.contains('<'));
    assert!(code.contains('T'));
    assert!(code.contains('>'));
}

// Test 77: transpile_trait - with associated types
#[test]
fn test_transpile_trait_with_associated_types() {
    let transpiler = Transpiler::new();
    let methods = vec![make_trait_method("next", false)];
    let associated = vec!["Item".to_string()];
    let result = transpiler.transpile_trait("Iterator", &[], &associated, &methods, true);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("type Item"));
}

// Helper: Create ImplMethod
fn make_impl_method(name: &str, is_pub: bool) -> crate::frontend::ast::ImplMethod {
    use crate::frontend::ast::{Expr, ExprKind, Literal};
    crate::frontend::ast::ImplMethod {
        name: name.to_string(),
        params: vec![],
        return_type: Some(make_type(TypeKind::Named("i32".to_string()))),
        body: Box::new(Expr {
            kind: ExprKind::Literal(Literal::Integer(100, None)),
            span: Span::new(0, 0),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }),
        is_pub,
    }
}

// Test 78: transpile_impl - inherent impl
#[test]
fn test_transpile_impl_inherent() {
    let transpiler = Transpiler::new();
    let methods = vec![make_impl_method("compute", true)];
    let result = transpiler.transpile_impl("MyStruct", &[], None, &methods, false);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("impl"));
    assert!(code.contains("MyStruct"));
    assert!(code.contains("compute"));
    assert!(code.contains("100"));
}

// Test 79: transpile_impl - trait impl
#[test]
fn test_transpile_impl_trait() {
    let transpiler = Transpiler::new();
    let methods = vec![make_impl_method("get", true)];
    let result = transpiler.transpile_impl("MyStruct", &[], Some("Getter"), &methods, false);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("impl"));
    assert!(code.contains("Getter"));
    assert!(code.contains("for"));
    assert!(code.contains("MyStruct"));
}

// Test 80: transpile_impl - with type params
#[test]
fn test_transpile_impl_with_type_params() {
    let transpiler = Transpiler::new();
    let methods = vec![make_impl_method("inner", true)];
    let result = transpiler.transpile_impl("Wrapper", &["T".to_string()], None, &methods, false);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("impl"));
    assert!(code.contains('<'));
    assert!(code.contains('T'));
    assert!(code.contains('>'));
    assert!(code.contains("Wrapper"));
}

// Test 81: transpile_impl - trait impl with type params
#[test]
fn test_transpile_impl_trait_with_type_params() {
    let transpiler = Transpiler::new();
    let methods = vec![make_impl_method("convert", true)];
    let result = transpiler.transpile_impl(
        "Container",
        &["T".to_string()],
        Some("Converter"),
        &methods,
        false,
    );
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("impl"));
    assert!(code.contains("Converter"));
    assert!(code.contains("for"));
    assert!(code.contains("Container"));
}

// Test 82: transpile_extend - extension trait
#[test]
fn test_transpile_extend() {
    let transpiler = Transpiler::new();
    let methods = vec![make_impl_method("is_empty", true)];
    let result = transpiler.transpile_extend("String", &methods);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("trait"));
    assert!(code.contains("StringExt"));
    assert!(code.contains("impl"));
    assert!(code.contains("for"));
    assert!(code.contains("String"));
    assert!(code.contains("is_empty"));
}

// Test 83: transpile_params - with self parameter (immutable reference)
#[test]
fn test_transpile_params_self_ref() {
    let transpiler = Transpiler::new();
    let params = vec![Param {
        pattern: crate::frontend::ast::Pattern::Identifier("self".to_string()),
        ty: make_type(TypeKind::Reference {
            is_mut: false,
            lifetime: None,
            inner: Box::new(make_type(TypeKind::Named("Self".to_string()))),
        }),
        span: Span::new(0, 0),
        is_mutable: false,
        default_value: None,
    }];
    let result = transpiler.transpile_params(&params).unwrap();
    let code = result[0].to_string();
    assert!(code.contains('&'));
    assert!(code.contains("self"));
    assert!(!code.contains("mut"));
}

// Test 84: transpile_params - with self parameter (mutable reference)
#[test]
fn test_transpile_params_self_mut_ref() {
    let transpiler = Transpiler::new();
    let params = vec![Param {
        pattern: crate::frontend::ast::Pattern::Identifier("self".to_string()),
        ty: make_type(TypeKind::Reference {
            is_mut: true,
            lifetime: None,
            inner: Box::new(make_type(TypeKind::Named("Self".to_string()))),
        }),
        span: Span::new(0, 0),
        is_mutable: false,
        default_value: None,
    }];
    let result = transpiler.transpile_params(&params).unwrap();
    let code = result[0].to_string();
    assert!(code.contains('&'));
    assert!(code.contains("mut"));
    assert!(code.contains("self"));
}

// Test 85: transpile_params - with self parameter (owned)
#[test]
fn test_transpile_params_self_owned() {
    let transpiler = Transpiler::new();
    let params = vec![Param {
        pattern: crate::frontend::ast::Pattern::Identifier("self".to_string()),
        ty: make_type(TypeKind::Named("Self".to_string())),
        span: Span::new(0, 0),
        is_mutable: false,
        default_value: None,
    }];
    let result = transpiler.transpile_params(&params).unwrap();
    let code = result[0].to_string();
    assert!(code.contains("self"));
    assert!(!code.contains('&'));
}

// Test 86: transpile_params - with mutable owned self
#[test]
fn test_transpile_params_self_mut_owned() {
    let transpiler = Transpiler::new();
    let params = vec![Param {
        pattern: crate::frontend::ast::Pattern::Identifier("self".to_string()),
        ty: make_type(TypeKind::Named("Self".to_string())),
        span: Span::new(0, 0),
        is_mutable: true,
        default_value: None,
    }];
    let result = transpiler.transpile_params(&params).unwrap();
    let code = result[0].to_string();
    assert!(code.contains("mut"));
    assert!(code.contains("self"));
}

// Test 87: transpile_struct_with_methods - struct with methods
#[test]
fn test_transpile_struct_with_methods() {
    use crate::frontend::ast::{ClassMethod, Expr, ExprKind, Literal, SelfType, Visibility};
    let transpiler = Transpiler::new();
    let fields = vec![StructField {
        name: "value".to_string(),
        ty: make_type(TypeKind::Named("i32".to_string())),
        visibility: Visibility::Public,
        is_mut: false,
        default_value: None,
        decorators: vec![],
    }];
    let methods = vec![ClassMethod {
        name: "get_value".to_string(),
        params: vec![],
        return_type: Some(make_type(TypeKind::Named("i32".to_string()))),
        body: Box::new(Expr {
            kind: ExprKind::Literal(Literal::Integer(42, None)),
            span: Span::new(0, 0),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }),
        is_pub: true,
        is_async: false,
        is_static: false,
        is_override: false,
        is_final: false,
        is_abstract: false,
        self_type: SelfType::None,
    }];
    let result = transpiler.transpile_struct_with_methods(
        "Counter",
        &[],
        &fields,
        &methods,
        &["Debug".to_string()],
        true,
    );
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("struct"));
    assert!(code.contains("Counter"));
    assert!(code.contains("value"));
    assert!(code.contains("impl"));
    assert!(code.contains("get_value"));
}

// Test 88: transpile_struct - with visibility variants
#[test]
fn test_transpile_struct_field_visibility() {
    use crate::frontend::ast::Visibility;
    let transpiler = Transpiler::new();
    let fields = vec![
        StructField {
            name: "public_field".to_string(),
            ty: make_type(TypeKind::Named("i32".to_string())),
            visibility: Visibility::Public,
            is_mut: false,
            default_value: None,
            decorators: vec![],
        },
        StructField {
            name: "crate_field".to_string(),
            ty: make_type(TypeKind::Named("i32".to_string())),
            visibility: Visibility::PubCrate,
            is_mut: false,
            default_value: None,
            decorators: vec![],
        },
        StructField {
            name: "super_field".to_string(),
            ty: make_type(TypeKind::Named("i32".to_string())),
            visibility: Visibility::PubSuper,
            is_mut: false,
            default_value: None,
            decorators: vec![],
        },
        StructField {
            name: "private_field".to_string(),
            ty: make_type(TypeKind::Named("i32".to_string())),
            visibility: Visibility::Private,
            is_mut: false,
            default_value: None,
            decorators: vec![],
        },
    ];
    let result = transpiler.transpile_struct("VisibleStruct", &[], &fields, &[], true);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("pub public_field"));
    assert!(code.contains("pub (crate) crate_field"));
    assert!(code.contains("pub (super) super_field"));
    // private_field should NOT have pub prefix
}

// Test 89: transpile_class - full class with everything
#[test]
fn test_transpile_class_full() {
    use crate::frontend::ast::{
        ClassConstant, ClassMethod, Expr, ExprKind, Literal, SelfType, Visibility,
    };
    let transpiler = Transpiler::new();
    let fields = vec![StructField {
        name: "count".to_string(),
        ty: make_type(TypeKind::Named("i32".to_string())),
        visibility: Visibility::Private,
        is_mut: false,
        default_value: None,
        decorators: vec![],
    }];
    let constructors = vec![Constructor {
        name: None,
        params: vec![],
        body: Box::new(Expr {
            kind: ExprKind::Literal(Literal::Integer(0, None)),
            span: Span::new(0, 0),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }),
        return_type: None,
        is_pub: true,
    }];
    let methods = vec![ClassMethod {
        name: "increment".to_string(),
        params: vec![],
        return_type: None,
        body: Box::new(Expr {
            kind: ExprKind::Literal(Literal::Integer(1, None)),
            span: Span::new(0, 0),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }),
        is_pub: true,
        is_async: false,
        is_static: false,
        is_override: false,
        is_final: false,
        is_abstract: false,
        self_type: SelfType::None,
    }];
    let constants = vec![ClassConstant {
        name: "MAX".to_string(),
        ty: make_type(TypeKind::Named("i32".to_string())),
        value: Expr {
            kind: ExprKind::Literal(Literal::Integer(100, None)),
            span: Span::new(0, 0),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        },
        is_pub: true,
    }];
    let result = transpiler.transpile_class(
        "Counter",
        &[],
        &[],
        &fields,
        &constructors,
        &methods,
        &constants,
        &["Debug".to_string()],
        true,
    );
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("struct"));
    assert!(code.contains("Counter"));
    assert!(code.contains("impl"));
    assert!(code.contains("new"));
    assert!(code.contains("increment"));
    assert!(code.contains("MAX"));
    assert!(code.contains("100"));
}

// Test 90: transpile_named_type - special type mappings
#[test]
fn test_transpile_named_type_special_mappings() {
    let transpiler = Transpiler::new();

    // Test unit type
    let result = transpiler.transpile_named_type("()").unwrap();
    assert_eq!(result.to_string(), "()");

    // Test Any type
    let result = transpiler.transpile_named_type("Any").unwrap();
    assert_eq!(result.to_string(), "_");

    // Test underscore
    let result = transpiler.transpile_named_type("_").unwrap();
    assert_eq!(result.to_string(), "_");

    // Test Object type
    let result = transpiler.transpile_named_type("Object").unwrap();
    assert!(result.to_string().contains("BTreeMap"));
}

// Test 91: transpile_generic_type - direct call
#[test]
fn test_direct_transpile_generic_type() {
    let transpiler = Transpiler::new();
    let params = vec![
        make_type(TypeKind::Named("String".to_string())),
        make_type(TypeKind::Named("i32".to_string())),
    ];
    let result = transpiler
        .transpile_generic_type("HashMap", &params)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("HashMap"));
    assert!(code.contains("String"));
    assert!(code.contains("i32"));
    assert!(code.contains('<'));
    assert!(code.contains('>'));
}

// Test 92: transpile_reference_type - direct call with str
#[test]
fn test_direct_transpile_reference_type_str() {
    let transpiler = Transpiler::new();
    let inner = make_type(TypeKind::Named("str".to_string()));
    let result = transpiler
        .transpile_reference_type(false, None, &inner)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains('&'));
    assert!(code.contains("str"));
    // Should not be &&str
    assert!(!code.contains("& & str"));
}

// Test 93: transpile_reference_type - mutable str
#[test]
fn test_direct_transpile_reference_type_mut_str() {
    let transpiler = Transpiler::new();
    let inner = make_type(TypeKind::Named("str".to_string()));
    let result = transpiler
        .transpile_reference_type(true, None, &inner)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains('&'));
    assert!(code.contains("mut"));
    assert!(code.contains("str"));
}

// === EXTREME TDD Round 124 tests ===

// Test 94: transpile_named_type - bool type
#[test]
fn test_transpile_named_type_bool_r124() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_named_type("bool").unwrap();
    assert_eq!(result.to_string(), "bool");
}

// Test 95: transpile_named_type - char type
#[test]
fn test_transpile_named_type_char_r124() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_named_type("char").unwrap();
    assert_eq!(result.to_string(), "char");
}

// Test 96: transpile_named_type - usize type
#[test]
fn test_transpile_named_type_usize_r124() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_named_type("usize").unwrap();
    assert_eq!(result.to_string(), "usize");
}

// Test 97: transpile_named_type - isize type
#[test]
fn test_transpile_named_type_isize_r124() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_named_type("isize").unwrap();
    assert_eq!(result.to_string(), "isize");
}

// Test 98: transpile_named_type - i8 type
#[test]
fn test_transpile_named_type_i8_r124() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_named_type("i8").unwrap();
    assert_eq!(result.to_string(), "i8");
}

// Test 99: transpile_named_type - u8 type
#[test]
fn test_transpile_named_type_u8_r124() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_named_type("u8").unwrap();
    assert_eq!(result.to_string(), "u8");
}

// Test 100: transpile_named_type - i16 type
#[test]
fn test_transpile_named_type_i16_r124() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_named_type("i16").unwrap();
    assert_eq!(result.to_string(), "i16");
}

// Test 101: transpile_named_type - u16 type
#[test]
fn test_transpile_named_type_u16_r124() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_named_type("u16").unwrap();
    assert_eq!(result.to_string(), "u16");
}

// Test 102: transpile_named_type - i64 type
#[test]
fn test_transpile_named_type_i64_r124() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_named_type("i64").unwrap();
    assert_eq!(result.to_string(), "i64");
}

// Test 103: transpile_named_type - u64 type
#[test]
fn test_transpile_named_type_u64_r124() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_named_type("u64").unwrap();
    assert_eq!(result.to_string(), "u64");
}

// Test 104: transpile_named_type - f32 type
#[test]
fn test_transpile_named_type_f32_r124() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_named_type("f32").unwrap();
    assert_eq!(result.to_string(), "f32");
}

// Test 105: transpile_named_type - custom type
#[test]
fn test_transpile_named_type_custom_r124() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_named_type("MyCustomType").unwrap();
    assert_eq!(result.to_string(), "MyCustomType");
}
