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
