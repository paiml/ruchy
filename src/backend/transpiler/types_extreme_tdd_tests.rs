use super::*;
use crate::frontend::ast::{
    EnumVariant, EnumVariantKind, ImplMethod, Span, StructField, TraitMethod, Type, TypeKind,
    Visibility,
};

fn make_type(kind: TypeKind) -> Type {
    Type {
        kind,
        span: Span::new(0, 0),
    }
}

fn make_expr(kind: crate::frontend::ast::ExprKind) -> Expr {
    Expr {
        kind,
        span: Span::new(0, 0),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

fn make_param(name: &str, ty: Type) -> crate::frontend::ast::Param {
    crate::frontend::ast::Param {
        pattern: crate::frontend::ast::Pattern::Identifier(name.to_string()),
        ty,
        span: Span::new(0, 0),
        is_mutable: false,
        default_value: None,
    }
}

fn make_struct_field(name: &str, ty: Type, visibility: Visibility) -> StructField {
    StructField {
        name: name.to_string(),
        ty,
        visibility,
        default_value: None,
        decorators: vec![],
        is_mut: false,
    }
}

// ===== Named Type Tests =====

#[test]
fn test_transpile_named_type_int() {
    let t = Transpiler::new();
    let result = t.transpile_named_type("int").unwrap();
    assert!(result.to_string().contains("i64"));
}

#[test]
fn test_transpile_named_type_float() {
    let t = Transpiler::new();
    let result = t.transpile_named_type("float").unwrap();
    assert!(result.to_string().contains("f64"));
}

#[test]
fn test_transpile_named_type_bool() {
    let t = Transpiler::new();
    let result = t.transpile_named_type("bool").unwrap();
    assert!(result.to_string().contains("bool"));
}

#[test]
fn test_transpile_named_type_str() {
    let t = Transpiler::new();
    let result = t.transpile_named_type("str").unwrap();
    assert!(result.to_string().contains("& str"));
}

#[test]
fn test_transpile_named_type_string() {
    let t = Transpiler::new();
    let result = t.transpile_named_type("string").unwrap();
    assert!(result.to_string().contains("String"));
}

#[test]
fn test_transpile_named_type_char() {
    let t = Transpiler::new();
    let result = t.transpile_named_type("char").unwrap();
    assert!(result.to_string().contains("char"));
}

#[test]
fn test_transpile_named_type_unit() {
    let t = Transpiler::new();
    let result = t.transpile_named_type("()").unwrap();
    assert!(result.to_string().contains("()"));
}

#[test]
fn test_transpile_named_type_any() {
    let t = Transpiler::new();
    let result = t.transpile_named_type("Any").unwrap();
    assert!(result.to_string().contains("_"));
}

#[test]
fn test_transpile_named_type_object() {
    let t = Transpiler::new();
    let result = t.transpile_named_type("Object").unwrap();
    assert!(result.to_string().contains("BTreeMap"));
}

#[test]
fn test_transpile_named_type_namespaced() {
    let t = Transpiler::new();
    let result = t.transpile_named_type("std::io::Error").unwrap();
    let s = result.to_string();
    assert!(s.contains("std"));
    assert!(s.contains("io"));
    assert!(s.contains("Error"));
}

#[test]
fn test_transpile_named_type_custom() {
    let t = Transpiler::new();
    let result = t.transpile_named_type("MyCustomType").unwrap();
    assert!(result.to_string().contains("MyCustomType"));
}

// ===== Generic Type Tests =====

#[test]
fn test_transpile_generic_type_single_param() {
    let t = Transpiler::new();
    let inner = make_type(TypeKind::Named("i32".to_string()));
    let result = t.transpile_generic_type("Vec", &[inner]).unwrap();
    assert!(result.to_string().contains("Vec"));
    assert!(result.to_string().contains("i32"));
}

#[test]
fn test_transpile_generic_type_multiple_params() {
    let t = Transpiler::new();
    let k = make_type(TypeKind::Named("String".to_string()));
    let v = make_type(TypeKind::Named("i32".to_string()));
    let result = t.transpile_generic_type("HashMap", &[k, v]).unwrap();
    let s = result.to_string();
    assert!(s.contains("HashMap"));
    assert!(s.contains("String"));
    assert!(s.contains("i32"));
}

// ===== Optional Type Tests =====

#[test]
fn test_transpile_optional_type() {
    let t = Transpiler::new();
    let inner = make_type(TypeKind::Named("i32".to_string()));
    let result = t.transpile_optional_type(&inner).unwrap();
    let s = result.to_string();
    assert!(s.contains("Option"));
    assert!(s.contains("i32"));
}

// ===== List Type Tests =====

#[test]
fn test_transpile_list_type() {
    let t = Transpiler::new();
    let elem = make_type(TypeKind::Named("String".to_string()));
    let result = t.transpile_list_type(&elem).unwrap();
    let s = result.to_string();
    assert!(s.contains("Vec"));
    assert!(s.contains("String"));
}

// ===== Array Type Tests =====

#[test]
fn test_transpile_array_type() {
    let t = Transpiler::new();
    let elem = make_type(TypeKind::Named("f64".to_string()));
    let result = t.transpile_array_type(&elem, 10).unwrap();
    let s = result.to_string();
    assert!(s.contains("f64"));
    assert!(s.contains("10"));
}

// ===== Tuple Type Tests =====

#[test]
fn test_transpile_tuple_type_empty() {
    let t = Transpiler::new();
    let result = t.transpile_tuple_type(&[]).unwrap();
    assert!(result.to_string().contains("()"));
}

#[test]
fn test_transpile_tuple_type_single() {
    let t = Transpiler::new();
    let elem = make_type(TypeKind::Named("i32".to_string()));
    let result = t.transpile_tuple_type(&[elem]).unwrap();
    assert!(result.to_string().contains("i32"));
}

#[test]
fn test_transpile_tuple_type_multiple() {
    let t = Transpiler::new();
    let e1 = make_type(TypeKind::Named("i32".to_string()));
    let e2 = make_type(TypeKind::Named("String".to_string()));
    let e3 = make_type(TypeKind::Named("bool".to_string()));
    let result = t.transpile_tuple_type(&[e1, e2, e3]).unwrap();
    let s = result.to_string();
    assert!(s.contains("i32"));
    assert!(s.contains("String"));
    assert!(s.contains("bool"));
}

// ===== Function Type Tests =====

#[test]
fn test_transpile_function_type() {
    let t = Transpiler::new();
    let param = make_type(TypeKind::Named("i32".to_string()));
    let ret = make_type(TypeKind::Named("bool".to_string()));
    let result = t.transpile_function_type(&[param], &ret).unwrap();
    let s = result.to_string();
    assert!(s.contains("fn"));
    assert!(s.contains("i32"));
    assert!(s.contains("bool"));
}

// ===== Reference Type Tests =====

#[test]
fn test_transpile_reference_type_immutable() {
    let t = Transpiler::new();
    let inner = make_type(TypeKind::Named("i32".to_string()));
    let result = t.transpile_reference_type(false, None, &inner).unwrap();
    let s = result.to_string();
    assert!(s.contains("&"));
    assert!(s.contains("i32"));
    assert!(!s.contains("mut"));
}

#[test]
fn test_transpile_reference_type_mutable() {
    let t = Transpiler::new();
    let inner = make_type(TypeKind::Named("i32".to_string()));
    let result = t.transpile_reference_type(true, None, &inner).unwrap();
    let s = result.to_string();
    assert!(s.contains("mut"));
    assert!(s.contains("i32"));
}

#[test]
fn test_transpile_reference_type_with_lifetime() {
    let t = Transpiler::new();
    let inner = make_type(TypeKind::Named("i32".to_string()));
    let result = t
        .transpile_reference_type(false, Some("'a"), &inner)
        .unwrap();
    let s = result.to_string();
    assert!(s.contains("'a"));
    assert!(s.contains("i32"));
}

#[test]
fn test_transpile_reference_type_str_special() {
    let t = Transpiler::new();
    let inner = make_type(TypeKind::Named("str".to_string()));
    let result = t.transpile_reference_type(false, None, &inner).unwrap();
    // Should be &str, not &&str
    assert_eq!(result.to_string().matches("&").count(), 1);
}

// ===== Type Parameter Tests =====

#[test]
fn test_parse_type_param_simple() {
    let result = Transpiler::parse_type_param_to_tokens("T");
    assert!(result.to_string().contains("T"));
}

#[test]
fn test_parse_type_param_lifetime() {
    let result = Transpiler::parse_type_param_to_tokens("'a");
    assert!(result.to_string().contains("'a"));
}

#[test]
fn test_parse_type_param_with_bound() {
    let result = Transpiler::parse_type_param_to_tokens("T: Clone");
    let s = result.to_string();
    assert!(s.contains("T"));
    assert!(s.contains("Clone"));
}

// ===== Struct Tests =====

#[test]
fn test_transpile_struct_empty() {
    let t = Transpiler::new();
    let result = t.transpile_struct("Empty", &[], &[], &[], false).unwrap();
    let s = result.to_string();
    assert!(s.contains("struct"));
    assert!(s.contains("Empty"));
}

#[test]
fn test_transpile_struct_with_fields() {
    let t = Transpiler::new();
    let fields = vec![
        make_struct_field(
            "x",
            make_type(TypeKind::Named("i32".to_string())),
            Visibility::Public,
        ),
        make_struct_field(
            "y",
            make_type(TypeKind::Named("i32".to_string())),
            Visibility::Private,
        ),
    ];
    let result = t
        .transpile_struct("Point", &[], &fields, &[], true)
        .unwrap();
    let s = result.to_string();
    assert!(s.contains("pub struct"));
    assert!(s.contains("Point"));
    assert!(s.contains("x"));
    assert!(s.contains("y"));
}

#[test]
fn test_transpile_struct_with_type_params() {
    let t = Transpiler::new();
    let fields = vec![make_struct_field(
        "value",
        make_type(TypeKind::Named("T".to_string())),
        Visibility::Public,
    )];
    let result = t
        .transpile_struct("Container", &["T".to_string()], &fields, &[], true)
        .unwrap();
    let s = result.to_string();
    assert!(s.contains("Container"));
    assert!(s.contains("<"));
    assert!(s.contains("T"));
}

#[test]
fn test_transpile_struct_with_derives() {
    let t = Transpiler::new();
    let result = t
        .transpile_struct(
            "MyStruct",
            &[],
            &[],
            &["Debug".to_string(), "Clone".to_string()],
            false,
        )
        .unwrap();
    let s = result.to_string();
    assert!(s.contains("derive"));
    assert!(s.contains("Debug"));
    assert!(s.contains("Clone"));
}

// ===== Tuple Struct Tests =====

#[test]
fn test_transpile_tuple_struct() {
    let t = Transpiler::new();
    let fields = vec![
        make_type(TypeKind::Named("i32".to_string())),
        make_type(TypeKind::Named("String".to_string())),
    ];
    let result = t
        .transpile_tuple_struct("Pair", &[], &fields, &[], true)
        .unwrap();
    let s = result.to_string();
    assert!(s.contains("pub struct"));
    assert!(s.contains("Pair"));
    assert!(s.contains("i32"));
    assert!(s.contains("String"));
}

// ===== Enum Tests =====

#[test]
fn test_transpile_enum_unit_variants() {
    let t = Transpiler::new();
    let variants = vec![
        EnumVariant {
            name: "Red".to_string(),
            kind: EnumVariantKind::Unit,
            discriminant: None,
        },
        EnumVariant {
            name: "Green".to_string(),
            kind: EnumVariantKind::Unit,
            discriminant: None,
        },
        EnumVariant {
            name: "Blue".to_string(),
            kind: EnumVariantKind::Unit,
            discriminant: None,
        },
    ];
    let result = t.transpile_enum("Color", &[], &variants, true).unwrap();
    let s = result.to_string();
    assert!(s.contains("pub enum"));
    assert!(s.contains("Color"));
    assert!(s.contains("Red"));
    assert!(s.contains("Green"));
    assert!(s.contains("Blue"));
}

#[test]
fn test_transpile_enum_with_discriminants() {
    let t = Transpiler::new();
    let variants = vec![
        EnumVariant {
            name: "A".to_string(),
            kind: EnumVariantKind::Unit,
            discriminant: Some(1),
        },
        EnumVariant {
            name: "B".to_string(),
            kind: EnumVariantKind::Unit,
            discriminant: Some(2),
        },
    ];
    let result = t.transpile_enum("MyEnum", &[], &variants, false).unwrap();
    let s = result.to_string();
    assert!(s.contains("repr"));
    assert!(s.contains("= 1"));
    assert!(s.contains("= 2"));
}

#[test]
fn test_transpile_enum_tuple_variant() {
    let t = Transpiler::new();
    let variants = vec![
        EnumVariant {
            name: "Some".to_string(),
            kind: EnumVariantKind::Tuple(vec![make_type(TypeKind::Named("T".to_string()))]),
            discriminant: None,
        },
        EnumVariant {
            name: "None".to_string(),
            kind: EnumVariantKind::Unit,
            discriminant: None,
        },
    ];
    let result = t
        .transpile_enum("MyOption", &["T".to_string()], &variants, true)
        .unwrap();
    let s = result.to_string();
    assert!(s.contains("MyOption"));
    assert!(s.contains("Some"));
    assert!(s.contains("None"));
}

#[test]
fn test_transpile_enum_struct_variant() {
    let t = Transpiler::new();
    let variants = vec![EnumVariant {
        name: "Move".to_string(),
        kind: EnumVariantKind::Struct(vec![
            make_struct_field(
                "x",
                make_type(TypeKind::Named("i32".to_string())),
                Visibility::Public,
            ),
            make_struct_field(
                "y",
                make_type(TypeKind::Named("i32".to_string())),
                Visibility::Public,
            ),
        ]),
        discriminant: None,
    }];
    let result = t.transpile_enum("Message", &[], &variants, true).unwrap();
    let s = result.to_string();
    assert!(s.contains("Move"));
    assert!(s.contains("x"));
    assert!(s.contains("y"));
}

// ===== Trait Tests =====

#[test]
fn test_transpile_trait_empty() {
    let t = Transpiler::new();
    let result = t.transpile_trait("Empty", &[], &[], &[], true).unwrap();
    let s = result.to_string();
    assert!(s.contains("pub trait"));
    assert!(s.contains("Empty"));
}

#[test]
fn test_transpile_trait_with_method() {
    let t = Transpiler::new();
    let methods = vec![TraitMethod {
        name: "do_something".to_string(),
        params: vec![make_param(
            "self",
            make_type(TypeKind::Reference {
                is_mut: false,
                lifetime: None,
                inner: Box::new(make_type(TypeKind::Named("Self".to_string()))),
            }),
        )],
        return_type: Some(make_type(TypeKind::Named("i32".to_string()))),
        body: None,
        is_pub: false,
    }];
    let result = t
        .transpile_trait("MyTrait", &[], &[], &methods, true)
        .unwrap();
    let s = result.to_string();
    assert!(s.contains("MyTrait"));
    assert!(s.contains("do_something"));
    assert!(s.contains("i32"));
}

#[test]
fn test_transpile_trait_with_associated_type() {
    let t = Transpiler::new();
    let result = t
        .transpile_trait("Iterator", &[], &["Item".to_string()], &[], true)
        .unwrap();
    let s = result.to_string();
    assert!(s.contains("type Item"));
}

// ===== Impl Tests =====

#[test]
fn test_transpile_impl_inherent() {
    let t = Transpiler::new();
    let body = make_expr(crate::frontend::ast::ExprKind::Literal(
        crate::frontend::ast::Literal::Integer(42, None),
    ));
    let methods = vec![ImplMethod {
        name: "answer".to_string(),
        params: vec![],
        return_type: Some(make_type(TypeKind::Named("i32".to_string()))),
        body: Box::new(body),
        is_pub: true,
    }];
    let result = t
        .transpile_impl("MyStruct", &[], None, &methods, true)
        .unwrap();
    let s = result.to_string();
    assert!(s.contains("impl MyStruct"));
    assert!(s.contains("answer"));
}

#[test]
fn test_transpile_impl_trait() {
    let t = Transpiler::new();
    let body = make_expr(crate::frontend::ast::ExprKind::Literal(
        crate::frontend::ast::Literal::Integer(0, None),
    ));
    let methods = vec![ImplMethod {
        name: "default".to_string(),
        params: vec![],
        return_type: Some(make_type(TypeKind::Named("Self".to_string()))),
        body: Box::new(body),
        is_pub: false,
    }];
    let result = t
        .transpile_impl("MyStruct", &[], Some("Default"), &methods, false)
        .unwrap();
    let s = result.to_string();
    assert!(s.contains("impl Default for MyStruct"));
}

// ===== Extend Tests =====

#[test]
fn test_transpile_extend() {
    let t = Transpiler::new();
    let body = make_expr(crate::frontend::ast::ExprKind::Literal(
        crate::frontend::ast::Literal::Bool(true),
    ));
    let methods = vec![ImplMethod {
        name: "is_empty".to_string(),
        params: vec![make_param(
            "self",
            make_type(TypeKind::Reference {
                is_mut: false,
                lifetime: None,
                inner: Box::new(make_type(TypeKind::Named("Self".to_string()))),
            }),
        )],
        return_type: Some(make_type(TypeKind::Named("bool".to_string()))),
        body: Box::new(body),
        is_pub: true,
    }];
    let result = t.transpile_extend("String", &methods).unwrap();
    let s = result.to_string();
    assert!(s.contains("trait StringExt"));
    assert!(s.contains("impl StringExt for String"));
}

// ===== Helper Method Tests =====

#[test]
fn test_has_reference_fields_true() {
    let t = Transpiler::new();
    let fields = vec![StructField {
        name: "data".to_string(),
        ty: make_type(TypeKind::Reference {
            is_mut: false,
            lifetime: None,
            inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
        }),
        visibility: Visibility::Public,
        default_value: None,
        decorators: vec![],
        is_mut: false,
    }];
    assert!(t.has_reference_fields(&fields));
}

#[test]
fn test_has_reference_fields_false() {
    let t = Transpiler::new();
    let fields = vec![StructField {
        name: "data".to_string(),
        ty: make_type(TypeKind::Named("String".to_string())),
        visibility: Visibility::Public,
        default_value: None,
        decorators: vec![],
        is_mut: false,
    }];
    assert!(!t.has_reference_fields(&fields));
}

#[test]
fn test_has_lifetime_params_true() {
    let t = Transpiler::new();
    assert!(t.has_lifetime_params(&["'a".to_string(), "T".to_string()]));
}

#[test]
fn test_has_lifetime_params_false() {
    let t = Transpiler::new();
    assert!(!t.has_lifetime_params(&["T".to_string(), "U".to_string()]));
}

#[test]
fn test_generate_derive_attributes_empty() {
    let t = Transpiler::new();
    let result = t.generate_derive_attributes(&[]);
    assert!(result.is_empty());
}

#[test]
fn test_generate_derive_attributes_multiple() {
    let t = Transpiler::new();
    let result = t.generate_derive_attributes(&["Debug".to_string(), "Clone".to_string()]);
    let s = result.to_string();
    assert!(s.contains("derive"));
    assert!(s.contains("Debug"));
    assert!(s.contains("Clone"));
}

#[test]
fn test_generate_class_type_param_tokens() {
    let t = Transpiler::new();
    let result = t.generate_class_type_param_tokens(&["T".to_string(), "'a".to_string()]);
    assert_eq!(result.len(), 2);
}

// ===== transpile_type Integration Tests =====

#[test]
fn test_transpile_type_named() {
    let t = Transpiler::new();
    let ty = make_type(TypeKind::Named("i32".to_string()));
    let result = t.transpile_type(&ty).unwrap();
    assert!(result.to_string().contains("i32"));
}

#[test]
fn test_transpile_type_optional() {
    let t = Transpiler::new();
    let ty = make_type(TypeKind::Optional(Box::new(make_type(TypeKind::Named(
        "i32".to_string(),
    )))));
    let result = t.transpile_type(&ty).unwrap();
    assert!(result.to_string().contains("Option"));
}

#[test]
fn test_transpile_type_list() {
    let t = Transpiler::new();
    let ty = make_type(TypeKind::List(Box::new(make_type(TypeKind::Named(
        "String".to_string(),
    )))));
    let result = t.transpile_type(&ty).unwrap();
    assert!(result.to_string().contains("Vec"));
}

#[test]
fn test_transpile_type_dataframe() {
    let t = Transpiler::new();
    let ty = make_type(TypeKind::DataFrame { columns: vec![] });
    let result = t.transpile_type(&ty).unwrap();
    assert!(result.to_string().contains("DataFrame"));
}

#[test]
fn test_transpile_type_series() {
    let t = Transpiler::new();
    let ty = make_type(TypeKind::Series {
        dtype: Box::new(make_type(TypeKind::Named("f64".to_string()))),
    });
    let result = t.transpile_type(&ty).unwrap();
    assert!(result.to_string().contains("Series"));
}

#[test]
fn test_transpile_type_refined() {
    let t = Transpiler::new();
    let ty = make_type(TypeKind::Refined {
        base: Box::new(make_type(TypeKind::Named("i32".to_string()))),
        constraint: Box::new(make_expr(crate::frontend::ast::ExprKind::Literal(
            crate::frontend::ast::Literal::Bool(true),
        ))),
    });
    let result = t.transpile_type(&ty).unwrap();
    // Refined types transpile to just the base type
    assert!(result.to_string().contains("i32"));
}

// ===== transpile_struct_with_methods Tests =====

#[test]
fn test_struct_no_methods_no_derives() {
    let t = Transpiler::new();
    let fields = vec![make_struct_field(
        "x",
        make_type(TypeKind::Named("i32".to_string())),
        Visibility::Public,
    )];
    let result = t
        .transpile_struct_with_methods("Point", &[], &fields, &[], &[], false)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("struct Point"), "Should contain struct name");
    assert!(code.contains("pub x"), "Public field should have pub");
    assert!(code.contains("i32"), "Should contain field type");
    // Clone is auto-added
    assert!(code.contains("Clone"), "Clone should be auto-added");
}

#[test]
fn test_struct_with_pub_visibility() {
    let t = Transpiler::new();
    let fields = vec![make_struct_field(
        "value",
        make_type(TypeKind::Named("String".to_string())),
        Visibility::Private,
    )];
    let result = t
        .transpile_struct_with_methods("Config", &[], &fields, &[], &[], true)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("pub struct Config"), "Should be pub struct");
}

#[test]
fn test_struct_with_type_params() {
    let t = Transpiler::new();
    let fields = vec![make_struct_field(
        "data",
        make_type(TypeKind::Named("T".to_string())),
        Visibility::Public,
    )];
    let result = t
        .transpile_struct_with_methods("Container", &["T".to_string()], &fields, &[], &[], false)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("Container"), "Should contain struct name");
    assert!(code.contains('T'), "Should have type parameter");
}

#[test]
fn test_struct_with_methods_no_defaults() {
    let t = Transpiler::new();
    let fields = vec![make_struct_field(
        "x",
        make_type(TypeKind::Named("i32".to_string())),
        Visibility::Public,
    )];
    let method = crate::frontend::ast::ClassMethod {
        name: "get_x".to_string(),
        params: vec![],
        return_type: Some(make_type(TypeKind::Named("i32".to_string()))),
        body: Box::new(make_expr(crate::frontend::ast::ExprKind::Literal(
            crate::frontend::ast::Literal::Integer(0, None),
        ))),
        is_pub: true,
        is_static: false,
        is_override: false,
        is_final: false,
        is_abstract: false,
        is_async: false,
        self_type: crate::frontend::ast::SelfType::Borrowed,
    };
    let result = t
        .transpile_struct_with_methods("Point", &[], &fields, &[method], &[], false)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("struct Point"), "Should contain struct def");
    assert!(code.contains("impl Point"), "Should contain impl block");
}

#[test]
fn test_struct_with_default_values() {
    let t = Transpiler::new();
    let field_with_default = StructField {
        name: "count".to_string(),
        ty: make_type(TypeKind::Named("i32".to_string())),
        visibility: Visibility::Public,
        default_value: Some(make_expr(crate::frontend::ast::ExprKind::Literal(
            crate::frontend::ast::Literal::Integer(0, None),
        ))),
        decorators: vec![],
        is_mut: false,
    };
    let result = t
        .transpile_struct_with_methods("Counter", &[], &[field_with_default], &[], &[], false)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("Default"), "Should generate Default impl");
}

#[test]
fn test_struct_with_default_string_field() {
    let t = Transpiler::new();
    let field_with_default = StructField {
        name: "label".to_string(),
        ty: make_type(TypeKind::Named("String".to_string())),
        visibility: Visibility::Public,
        default_value: Some(make_expr(crate::frontend::ast::ExprKind::Literal(
            crate::frontend::ast::Literal::String("hello".to_string()),
        ))),
        decorators: vec![],
        is_mut: false,
    };
    let result = t
        .transpile_struct_with_methods("Config", &[], &[field_with_default], &[], &[], false)
        .unwrap();
    let code = result.to_string();
    assert!(
        code.contains("to_string"),
        "String default should add .to_string()"
    );
}

#[test]
fn test_struct_defaults_with_methods() {
    let t = Transpiler::new();
    let field_with_default = StructField {
        name: "count".to_string(),
        ty: make_type(TypeKind::Named("i32".to_string())),
        visibility: Visibility::Public,
        default_value: Some(make_expr(crate::frontend::ast::ExprKind::Literal(
            crate::frontend::ast::Literal::Integer(0, None),
        ))),
        decorators: vec![],
        is_mut: false,
    };
    let method = crate::frontend::ast::ClassMethod {
        name: "inc".to_string(),
        params: vec![],
        return_type: None,
        body: Box::new(make_expr(crate::frontend::ast::ExprKind::Literal(
            crate::frontend::ast::Literal::Unit,
        ))),
        is_pub: true,
        is_static: false,
        is_override: false,
        is_final: false,
        is_abstract: false,
        is_async: false,
        self_type: crate::frontend::ast::SelfType::MutBorrowed,
    };
    let result = t
        .transpile_struct_with_methods(
            "Counter",
            &[],
            &[field_with_default],
            &[method],
            &[],
            false,
        )
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("Default"), "Should have Default impl");
    assert!(code.contains("impl Counter"), "Should have methods impl block");
}

#[test]
fn test_struct_generic_with_defaults() {
    let t = Transpiler::new();
    let field = StructField {
        name: "value".to_string(),
        ty: make_type(TypeKind::Named("T".to_string())),
        visibility: Visibility::Public,
        default_value: Some(make_expr(crate::frontend::ast::ExprKind::Literal(
            crate::frontend::ast::Literal::Integer(0, None),
        ))),
        decorators: vec![],
        is_mut: false,
    };
    let result = t
        .transpile_struct_with_methods(
            "Wrapper",
            &["T".to_string()],
            &[field],
            &[],
            &[],
            false,
        )
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("Default"), "Generic struct should get Default impl with bounds");
}

#[test]
fn test_struct_field_visibility_pub_crate() {
    let t = Transpiler::new();
    let fields = vec![make_struct_field(
        "internal",
        make_type(TypeKind::Named("i32".to_string())),
        Visibility::PubCrate,
    )];
    let result = t
        .transpile_struct_with_methods("Inner", &[], &fields, &[], &[], false)
        .unwrap();
    let code = result.to_string();
    assert!(
        code.contains("pub (crate)"),
        "Should have pub(crate) visibility: {code}"
    );
}

#[test]
fn test_struct_field_visibility_pub_super() {
    let t = Transpiler::new();
    let fields = vec![make_struct_field(
        "data",
        make_type(TypeKind::Named("u8".to_string())),
        Visibility::PubSuper,
    )];
    let result = t
        .transpile_struct_with_methods("Sub", &[], &fields, &[], &[], false)
        .unwrap();
    let code = result.to_string();
    assert!(
        code.contains("pub (super)"),
        "Should have pub(super) visibility: {code}"
    );
}

#[test]
fn test_struct_field_no_default_in_default_impl() {
    let t = Transpiler::new();
    // One field has default, one doesn't => Default impl should use Default::default() for missing
    let field_with = StructField {
        name: "a".to_string(),
        ty: make_type(TypeKind::Named("i32".to_string())),
        visibility: Visibility::Public,
        default_value: Some(make_expr(crate::frontend::ast::ExprKind::Literal(
            crate::frontend::ast::Literal::Integer(5, None),
        ))),
        decorators: vec![],
        is_mut: false,
    };
    let field_without = make_struct_field(
        "b",
        make_type(TypeKind::Named("i32".to_string())),
        Visibility::Public,
    );
    let result = t
        .transpile_struct_with_methods("Mixed", &[], &[field_with, field_without], &[], &[], false)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("Default :: default"), "Missing defaults should use Default::default()");
}

#[test]
fn test_struct_clone_not_duplicated() {
    let t = Transpiler::new();
    let fields = vec![make_struct_field(
        "x",
        make_type(TypeKind::Named("i32".to_string())),
        Visibility::Public,
    )];
    let result = t
        .transpile_struct_with_methods(
            "Point",
            &[],
            &fields,
            &[],
            &["Clone".to_string(), "Debug".to_string()],
            false,
        )
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("Debug"), "Should have Debug derive");
    // Clone already provided, should not be duplicated
    assert!(code.contains("Clone"));
}

// ===== transpile_constructor_body Tests =====

#[test]
fn test_constructor_body_single_self_assign() {
    let t = Transpiler::new();
    // self.x = value  =>  Self { x: value }
    let target = make_expr(crate::frontend::ast::ExprKind::FieldAccess {
        object: Box::new(make_expr(crate::frontend::ast::ExprKind::Identifier(
            "self".to_string(),
        ))),
        field: "x".to_string(),
    });
    let value = make_expr(crate::frontend::ast::ExprKind::Literal(
        crate::frontend::ast::Literal::Integer(42, None),
    ));
    let body = make_expr(crate::frontend::ast::ExprKind::Assign {
        target: Box::new(target),
        value: Box::new(value),
    });
    let result = t.transpile_constructor_body(&body).unwrap();
    let code = result.to_string();
    assert!(code.contains("Self"), "Should generate Self struct init");
    assert!(code.contains('x'), "Should contain field name");
    assert!(code.contains("42"), "Should contain field value");
}

#[test]
fn test_constructor_body_block_with_self_assigns() {
    let t = Transpiler::new();
    // Block: { self.x = 1; self.y = 2 }
    let assign1 = make_expr(crate::frontend::ast::ExprKind::Assign {
        target: Box::new(make_expr(crate::frontend::ast::ExprKind::FieldAccess {
            object: Box::new(make_expr(crate::frontend::ast::ExprKind::Identifier(
                "self".to_string(),
            ))),
            field: "x".to_string(),
        })),
        value: Box::new(make_expr(crate::frontend::ast::ExprKind::Literal(
            crate::frontend::ast::Literal::Integer(1, None),
        ))),
    });
    let assign2 = make_expr(crate::frontend::ast::ExprKind::Assign {
        target: Box::new(make_expr(crate::frontend::ast::ExprKind::FieldAccess {
            object: Box::new(make_expr(crate::frontend::ast::ExprKind::Identifier(
                "self".to_string(),
            ))),
            field: "y".to_string(),
        })),
        value: Box::new(make_expr(crate::frontend::ast::ExprKind::Literal(
            crate::frontend::ast::Literal::Integer(2, None),
        ))),
    });
    let body = make_expr(crate::frontend::ast::ExprKind::Block(vec![assign1, assign2]));
    let result = t.transpile_constructor_body(&body).unwrap();
    let code = result.to_string();
    assert!(code.contains("Self"), "Should generate Self init");
    assert!(code.contains('x'), "Should contain x field");
    assert!(code.contains('y'), "Should contain y field");
}

#[test]
fn test_constructor_body_non_self_assign_fallthrough() {
    let t = Transpiler::new();
    // Block with a non-self assignment should fallback to regular transpilation
    let non_self_assign = make_expr(crate::frontend::ast::ExprKind::Assign {
        target: Box::new(make_expr(crate::frontend::ast::ExprKind::Identifier(
            "other".to_string(),
        ))),
        value: Box::new(make_expr(crate::frontend::ast::ExprKind::Literal(
            crate::frontend::ast::Literal::Integer(1, None),
        ))),
    });
    let body = make_expr(crate::frontend::ast::ExprKind::Block(vec![non_self_assign]));
    let result = t.transpile_constructor_body(&body);
    // Should fall through to regular transpilation (still OK)
    assert!(result.is_ok());
}

#[test]
fn test_constructor_body_non_block_non_assign_fallthrough() {
    let t = Transpiler::new();
    // A plain literal body should fall through to regular transpilation
    let body = make_expr(crate::frontend::ast::ExprKind::Literal(
        crate::frontend::ast::Literal::Integer(0, None),
    ));
    let result = t.transpile_constructor_body(&body).unwrap();
    let code = result.to_string();
    assert!(code.contains('0'), "Should transpile literal directly");
}

#[test]
fn test_constructor_body_empty_block() {
    let t = Transpiler::new();
    // Empty block => field_inits is empty => falls through to regular transpilation
    let body = make_expr(crate::frontend::ast::ExprKind::Block(vec![]));
    let result = t.transpile_constructor_body(&body);
    assert!(result.is_ok());
}

#[test]
fn test_constructor_body_block_mixed_self_then_non_self() {
    let t = Transpiler::new();
    // self.x = 1 then regular_var = 2 (non-self-assign breaks out)
    let self_assign = make_expr(crate::frontend::ast::ExprKind::Assign {
        target: Box::new(make_expr(crate::frontend::ast::ExprKind::FieldAccess {
            object: Box::new(make_expr(crate::frontend::ast::ExprKind::Identifier(
                "self".to_string(),
            ))),
            field: "x".to_string(),
        })),
        value: Box::new(make_expr(crate::frontend::ast::ExprKind::Literal(
            crate::frontend::ast::Literal::Integer(1, None),
        ))),
    });
    let non_self = make_expr(crate::frontend::ast::ExprKind::Assign {
        target: Box::new(make_expr(crate::frontend::ast::ExprKind::Identifier(
            "y".to_string(),
        ))),
        value: Box::new(make_expr(crate::frontend::ast::ExprKind::Literal(
            crate::frontend::ast::Literal::Integer(2, None),
        ))),
    });
    let body = make_expr(crate::frontend::ast::ExprKind::Block(vec![
        self_assign, non_self,
    ]));
    let result = t.transpile_constructor_body(&body);
    // Falls through when encountering non-self assignment
    assert!(result.is_ok());
}

// ===== transpile_property_test tests =====

#[test]
fn test_transpile_property_test_basic_function() {
    let t = Transpiler::new();
    let param = make_param(
        "x",
        make_type(crate::frontend::ast::TypeKind::Named("i32".to_string())),
    );
    let func = make_expr(crate::frontend::ast::ExprKind::Function {
        name: "check_positive".to_string(),
        type_params: vec![],
        params: vec![param],
        return_type: None,
        body: Box::new(make_expr(crate::frontend::ast::ExprKind::Literal(
            crate::frontend::ast::Literal::Bool(true),
        ))),
        is_async: false,
        is_pub: false,
    });
    let attr = crate::frontend::ast::Attribute {
        name: "property_test".to_string(),
        args: vec![],
        span: crate::frontend::ast::Span::new(0, 0),
    };
    let result = t.transpile_property_test(&func, &attr);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("proptest"));
    assert!(code.contains("check_positive"));
    assert!(code.contains("i32"));
}

#[test]
fn test_transpile_property_test_multiple_params() {
    let t = Transpiler::new();
    let param1 = make_param(
        "a",
        make_type(crate::frontend::ast::TypeKind::Named("i32".to_string())),
    );
    let param2 = make_param(
        "b",
        make_type(crate::frontend::ast::TypeKind::Named("f64".to_string())),
    );
    let func = make_expr(crate::frontend::ast::ExprKind::Function {
        name: "test_add".to_string(),
        type_params: vec![],
        params: vec![param1, param2],
        return_type: None,
        body: Box::new(make_expr(crate::frontend::ast::ExprKind::Literal(
            crate::frontend::ast::Literal::Bool(true),
        ))),
        is_async: false,
        is_pub: false,
    });
    let attr = crate::frontend::ast::Attribute {
        name: "property_test".to_string(),
        args: vec![],
        span: crate::frontend::ast::Span::new(0, 0),
    };
    let result = t.transpile_property_test(&func, &attr);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("test_add"));
    assert!(code.contains("i32"));
    assert!(code.contains("f64"));
}

#[test]
fn test_transpile_property_test_non_function_error() {
    let t = Transpiler::new();
    let non_func = make_expr(crate::frontend::ast::ExprKind::Literal(
        crate::frontend::ast::Literal::Integer(42, None),
    ));
    let attr = crate::frontend::ast::Attribute {
        name: "property_test".to_string(),
        args: vec![],
        span: crate::frontend::ast::Span::new(0, 0),
    };
    let result = t.transpile_property_test(&non_func, &attr);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Property test attribute can only be applied to functions"));
}

#[test]
fn test_transpile_property_test_no_params() {
    let t = Transpiler::new();
    let func = make_expr(crate::frontend::ast::ExprKind::Function {
        name: "check_invariant".to_string(),
        type_params: vec![],
        params: vec![],
        return_type: None,
        body: Box::new(make_expr(crate::frontend::ast::ExprKind::Literal(
            crate::frontend::ast::Literal::Bool(true),
        ))),
        is_async: false,
        is_pub: false,
    });
    let attr = crate::frontend::ast::Attribute {
        name: "property_test".to_string(),
        args: vec![],
        span: crate::frontend::ast::Span::new(0, 0),
    };
    let result = t.transpile_property_test(&func, &attr);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("proptest"));
    assert!(code.contains("check_invariant"));
}

// ===== transpile_body_with_auto_clone tests =====

#[test]
fn test_transpile_body_with_auto_clone_self_field() {
    let t = Transpiler::new();
    // self.name → self.name.clone()
    let body = make_expr(crate::frontend::ast::ExprKind::FieldAccess {
        object: Box::new(make_expr(crate::frontend::ast::ExprKind::Identifier(
            "self".to_string(),
        ))),
        field: "name".to_string(),
    });
    let result = t.transpile_body_with_auto_clone(&body);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("self . name . clone ()"), "Should auto-clone self.field: {code}");
}

#[test]
fn test_transpile_body_with_auto_clone_non_self_field() {
    let t = Transpiler::new();
    // other.name → transpile_expr (no auto-clone)
    let body = make_expr(crate::frontend::ast::ExprKind::FieldAccess {
        object: Box::new(make_expr(crate::frontend::ast::ExprKind::Identifier(
            "other".to_string(),
        ))),
        field: "name".to_string(),
    });
    let result = t.transpile_body_with_auto_clone(&body);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    // Should NOT have .clone() auto-added (falls through to transpile_expr)
    assert!(code.contains("name"));
}

#[test]
fn test_transpile_body_with_auto_clone_block_with_self_last() {
    let t = Transpiler::new();
    // Block where last expr is self.field → auto-clone on last expr
    let first = make_expr(crate::frontend::ast::ExprKind::Literal(
        crate::frontend::ast::Literal::Integer(1, None),
    ));
    let last = make_expr(crate::frontend::ast::ExprKind::FieldAccess {
        object: Box::new(make_expr(crate::frontend::ast::ExprKind::Identifier(
            "self".to_string(),
        ))),
        field: "value".to_string(),
    });
    let body = make_expr(crate::frontend::ast::ExprKind::Block(vec![first, last]));
    let result = t.transpile_body_with_auto_clone(&body);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("clone"), "Last expr in block should be auto-cloned: {code}");
}

#[test]
fn test_transpile_body_with_auto_clone_non_field_expr() {
    let t = Transpiler::new();
    // Simple literal → falls through to transpile_expr
    let body = make_expr(crate::frontend::ast::ExprKind::Literal(
        crate::frontend::ast::Literal::Integer(42, None),
    ));
    let result = t.transpile_body_with_auto_clone(&body);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("42"));
}

#[test]
fn test_transpile_body_with_auto_clone_empty_block() {
    let t = Transpiler::new();
    // Empty block → falls through to default branch (transpile_expr)
    let body = make_expr(crate::frontend::ast::ExprKind::Block(vec![]));
    let result = t.transpile_body_with_auto_clone(&body);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_body_with_auto_clone_single_elem_block() {
    let t = Transpiler::new();
    // Block with single self.field → auto-clone applied
    let elem = make_expr(crate::frontend::ast::ExprKind::FieldAccess {
        object: Box::new(make_expr(crate::frontend::ast::ExprKind::Identifier(
            "self".to_string(),
        ))),
        field: "data".to_string(),
    });
    let body = make_expr(crate::frontend::ast::ExprKind::Block(vec![elem]));
    let result = t.transpile_body_with_auto_clone(&body);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("clone"), "Single-elem block with self.field should auto-clone: {code}");
}
