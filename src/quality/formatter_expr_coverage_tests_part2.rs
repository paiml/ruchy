    use super::*;
    use crate::frontend::ast::*;

    fn make_formatter() -> Formatter {
        Formatter::new()
    }

    fn span() -> Span {
        Span::new(0, 0)
    }

    fn int_expr(n: i64) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Integer(n, None)), span())
    }

    fn ident_expr(name: &str) -> Expr {
        Expr::new(ExprKind::Identifier(name.to_string()), span())
    }

    fn str_expr(s: &str) -> Expr {
        Expr::new(ExprKind::Literal(Literal::String(s.to_string())), span())
    }

    fn bool_expr(b: bool) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Bool(b)), span())
    }

    // ============================================================================
    // Call
    // ============================================================================

    #[test]
    fn test_format_expr_call() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Call {
                func: Box::new(ident_expr("foo")),
                args: vec![int_expr(1), int_expr(2)],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "foo(1, 2)");
    }

    #[test]
    fn test_format_expr_call_no_args() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Call {
                func: Box::new(ident_expr("bar")),
                args: vec![],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "bar()");
    }

    // ============================================================================
    // If expression
    // ============================================================================

    #[test]
    fn test_format_expr_if_no_else() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(bool_expr(true)),
                then_branch: Box::new(Expr::new(ExprKind::Block(vec![int_expr(1)]), span())),
                else_branch: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("if true"));
        assert!(!result.contains("else"));
    }

    #[test]
    fn test_format_expr_if_with_else() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(bool_expr(false)),
                then_branch: Box::new(Expr::new(ExprKind::Block(vec![int_expr(1)]), span())),
                else_branch: Some(Box::new(Expr::new(
                    ExprKind::Block(vec![int_expr(2)]),
                    span(),
                ))),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("if false"));
        assert!(result.contains("else"));
    }

    // ============================================================================
    // Block
    // ============================================================================

    #[test]
    fn test_format_expr_block_empty() {
        let f = make_formatter();
        let expr = Expr::new(ExprKind::Block(vec![]), span());
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("{"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_format_expr_block_with_items() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Block(vec![int_expr(1), int_expr(2)]),
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("1"));
        assert!(result.contains("2"));
    }

    // ============================================================================
    // Indentation with tabs
    // ============================================================================

    #[test]
    fn test_format_expr_with_tabs() {
        let config = FormatterConfig {
            use_tabs: true,
            ..FormatterConfig::default()
        };
        let f = Formatter::with_config(config);
        let expr = Expr::new(
            ExprKind::Block(vec![int_expr(42)]),
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("42"));
    }

    // ============================================================================
    // Trailing comment
    // ============================================================================

    #[test]
    fn test_format_expr_with_trailing_comment() {
        let f = make_formatter();
        let mut expr = int_expr(42);
        expr.trailing_comment = Some(Comment::new(
            CommentKind::Line(" important".to_string()),
            span(),
        ));
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("42"));
        assert!(result.contains("// important"));
    }

    // ============================================================================
    // Leading comment
    // ============================================================================

    #[test]
    fn test_format_expr_with_leading_comment() {
        let f = make_formatter();
        let mut expr = int_expr(42);
        expr.leading_comments = vec![Comment::new(
            CommentKind::Line(" a comment".to_string()),
            span(),
        )];
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("// a comment"));
        assert!(result.contains("42"));
    }

    // ============================================================================
    // Function
    // ============================================================================

    #[test]
    fn test_format_expr_function_no_params_no_return() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Function {
                name: "greet".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(Expr::new(ExprKind::Block(vec![int_expr(1)]), span())),
                is_async: false,
                is_pub: false,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("fun greet()"));
    }

    #[test]
    fn test_format_expr_function_with_typed_params_and_return() {
        let f = make_formatter();
        let param = Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: span(),
            },
            span: span(),
            is_mutable: false,
            default_value: None,
        };
        let expr = Expr::new(
            ExprKind::Function {
                name: "add".to_string(),
                type_params: vec![],
                params: vec![param],
                return_type: Some(Type {
                    kind: TypeKind::Named("i32".to_string()),
                    span: span(),
                }),
                body: Box::new(Expr::new(ExprKind::Block(vec![ident_expr("x")]), span())),
                is_async: false,
                is_pub: false,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("fun add(x: i32) -> i32"));
    }

    #[test]
    fn test_format_expr_function_param_any_type_omitted() {
        let f = make_formatter();
        let param = Param {
            pattern: Pattern::Identifier("val".to_string()),
            ty: Type {
                kind: TypeKind::Named("Any".to_string()),
                span: span(),
            },
            span: span(),
            is_mutable: false,
            default_value: None,
        };
        let expr = Expr::new(
            ExprKind::Function {
                name: "identity".to_string(),
                type_params: vec![],
                params: vec![param],
                return_type: None,
                body: Box::new(Expr::new(ExprKind::Block(vec![]), span())),
                is_async: false,
                is_pub: false,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        // "Any" type should NOT appear in the output
        assert!(result.contains("fun identity(val)"));
        assert!(!result.contains("Any"));
    }

    #[test]
    fn test_format_expr_function_param_non_named_type() {
        let f = make_formatter();
        let param = Param {
            pattern: Pattern::Identifier("items".to_string()),
            ty: Type {
                kind: TypeKind::List(Box::new(Type {
                    kind: TypeKind::Named("i32".to_string()),
                    span: span(),
                })),
                span: span(),
            },
            span: span(),
            is_mutable: false,
            default_value: None,
        };
        let expr = Expr::new(
            ExprKind::Function {
                name: "process".to_string(),
                type_params: vec![],
                params: vec![param],
                return_type: None,
                body: Box::new(Expr::new(ExprKind::Block(vec![]), span())),
                is_async: false,
                is_pub: false,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("items:"));
    }

    #[test]
    fn test_format_expr_function_multiple_params() {
        let f = make_formatter();
        let param1 = Param {
            pattern: Pattern::Identifier("a".to_string()),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: span(),
            },
            span: span(),
            is_mutable: false,
            default_value: None,
        };
        let param2 = Param {
            pattern: Pattern::Identifier("b".to_string()),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: span(),
            },
            span: span(),
            is_mutable: false,
            default_value: None,
        };
        let expr = Expr::new(
            ExprKind::Function {
                name: "sum".to_string(),
                type_params: vec![],
                params: vec![param1, param2],
                return_type: None,
                body: Box::new(Expr::new(ExprKind::Block(vec![]), span())),
                is_async: false,
                is_pub: false,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("a: i32, b: i32"));
    }

    // ============================================================================
    // Match
    // ============================================================================

    #[test]
    fn test_format_expr_match() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Match {
                expr: Box::new(ident_expr("x")),
                arms: vec![
                    MatchArm {
                        pattern: Pattern::Literal(Literal::Integer(1, None)),
                        guard: None,
                        body: Box::new(str_expr("one")),
                        span: span(),
                    },
                    MatchArm {
                        pattern: Pattern::Wildcard,
                        guard: None,
                        body: Box::new(str_expr("other")),
                        span: span(),
                    },
                ],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("match x {"));
        assert!(result.contains("\"one\""));
        assert!(result.contains("\"other\""));
        assert!(result.ends_with("}"));
    }

    // ============================================================================
    // ObjectLiteral
    // ============================================================================

    #[test]
    fn test_format_expr_object_literal_empty() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::ObjectLiteral { fields: vec![] },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "{}");
    }

    #[test]
    fn test_format_expr_object_literal_key_value() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::ObjectLiteral {
                fields: vec![
                    ObjectField::KeyValue {
                        key: "name".to_string(),
                        value: str_expr("alice"),
                    },
                    ObjectField::KeyValue {
                        key: "age".to_string(),
                        value: int_expr(30),
                    },
                ],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("name: \"alice\""));
        assert!(result.contains("age: 30"));
    }

    #[test]
    fn test_format_expr_object_literal_spread() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::ObjectLiteral {
                fields: vec![
                    ObjectField::KeyValue {
                        key: "x".to_string(),
                        value: int_expr(1),
                    },
                    ObjectField::Spread {
                        expr: ident_expr("rest"),
                    },
                ],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("x: 1"));
        assert!(result.contains("...rest"));
    }

    // ============================================================================
    // StructLiteral
    // ============================================================================

    #[test]
    fn test_format_expr_struct_literal_no_base() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::StructLiteral {
                name: "Point".to_string(),
                fields: vec![
                    ("x".to_string(), int_expr(1)),
                    ("y".to_string(), int_expr(2)),
                ],
                base: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("Point {"));
        assert!(result.contains("x: 1"));
        assert!(result.contains("y: 2"));
    }

    #[test]
    fn test_format_expr_struct_literal_with_base() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::StructLiteral {
                name: "Point".to_string(),
                fields: vec![("x".to_string(), int_expr(5))],
                base: Some(Box::new(ident_expr("default_point"))),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("Point {"));
        assert!(result.contains("x: 5"));
        assert!(result.contains("..default_point"));
    }

    // ============================================================================
    // TryCatch
    // ============================================================================

    #[test]
    fn test_format_expr_try_catch_basic() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::TryCatch {
                try_block: Box::new(Expr::new(
                    ExprKind::Block(vec![ident_expr("risky")]),
                    span(),
                )),
                catch_clauses: vec![CatchClause {
                    pattern: Pattern::Identifier("e".to_string()),
                    body: Box::new(Expr::new(
                        ExprKind::Block(vec![ident_expr("handle")]),
                        span(),
                    )),
                }],
                finally_block: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("try "));
        assert!(result.contains("catch (e)"));
        assert!(!result.contains("finally"));
    }

    #[test]
    fn test_format_expr_try_catch_with_finally() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::TryCatch {
                try_block: Box::new(Expr::new(
                    ExprKind::Block(vec![ident_expr("risky")]),
                    span(),
                )),
                catch_clauses: vec![CatchClause {
                    pattern: Pattern::Identifier("e".to_string()),
                    body: Box::new(Expr::new(
                        ExprKind::Block(vec![ident_expr("handle")]),
                        span(),
                    )),
                }],
                finally_block: Some(Box::new(Expr::new(
                    ExprKind::Block(vec![ident_expr("cleanup")]),
                    span(),
                ))),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("try "));
        assert!(result.contains("catch (e)"));
        assert!(result.contains("finally"));
    }

    // ============================================================================
    // AsyncLambda
    // ============================================================================

    #[test]
    fn test_format_expr_async_lambda() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::AsyncLambda {
                params: vec!["x".to_string(), "y".to_string()],
                body: Box::new(ident_expr("x")),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "async |x, y| x");
    }

    // ============================================================================
    // IfLet
    // ============================================================================

    #[test]
    fn test_format_expr_if_let_no_else() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::IfLet {
                pattern: Pattern::Some(Box::new(Pattern::Identifier("val".to_string()))),
                expr: Box::new(ident_expr("opt")),
                then_branch: Box::new(Expr::new(
                    ExprKind::Block(vec![ident_expr("val")]),
                    span(),
                )),
                else_branch: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("if let Some(val) = opt"));
        assert!(!result.contains("else"));
    }

    #[test]
    fn test_format_expr_if_let_with_else() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::IfLet {
                pattern: Pattern::Some(Box::new(Pattern::Identifier("val".to_string()))),
                expr: Box::new(ident_expr("opt")),
                then_branch: Box::new(Expr::new(
                    ExprKind::Block(vec![ident_expr("val")]),
                    span(),
                )),
                else_branch: Some(Box::new(Expr::new(
                    ExprKind::Block(vec![int_expr(0)]),
                    span(),
                ))),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("if let Some(val) = opt"));
        assert!(result.contains("else"));
    }

    // ============================================================================
    // Struct
    // ============================================================================

    #[test]
    fn test_format_expr_struct_no_type_params() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Struct {
                name: "Point".to_string(),
                type_params: vec![],
                fields: vec![
                    StructField {
                        name: "x".to_string(),
                        ty: Type {
                            kind: TypeKind::Named("f64".to_string()),
                            span: span(),
                        },
                        visibility: Visibility::Private,
                        is_mut: false,
                        default_value: None,
                        decorators: vec![],
                    },
                    StructField {
                        name: "y".to_string(),
                        ty: Type {
                            kind: TypeKind::Named("f64".to_string()),
                            span: span(),
                        },
                        visibility: Visibility::Private,
                        is_mut: false,
                        default_value: None,
                        decorators: vec![],
                    },
                ],
                methods: vec![],
                derives: vec![],
                is_pub: false,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "struct Point { x: f64, y: f64 }");
    }

    #[test]
    fn test_format_expr_struct_pub_with_type_params() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Struct {
                name: "Container".to_string(),
                type_params: vec!["T".to_string()],
                fields: vec![StructField {
                    name: "value".to_string(),
                    ty: Type {
                        kind: TypeKind::Named("T".to_string()),
                        span: span(),
                    },
                    visibility: Visibility::Private,
                    is_mut: false,
                    default_value: None,
                    decorators: vec![],
                }],
                methods: vec![],
                derives: vec![],
                is_pub: true,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "pub struct Container<T> { value: T }");
    }

    // ============================================================================
    // TupleStruct
    // ============================================================================

    #[test]
    fn test_format_expr_tuple_struct() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::TupleStruct {
                name: "Pair".to_string(),
                type_params: vec![],
                fields: vec![
                    Type {
                        kind: TypeKind::Named("i32".to_string()),
                        span: span(),
                    },
                    Type {
                        kind: TypeKind::Named("String".to_string()),
                        span: span(),
                    },
                ],
                derives: vec![],
                is_pub: false,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "struct Pair(i32, String)");
    }

    #[test]
    fn test_format_expr_tuple_struct_pub_with_type_params() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::TupleStruct {
                name: "Wrapper".to_string(),
                type_params: vec!["T".to_string(), "U".to_string()],
                fields: vec![Type {
                    kind: TypeKind::Named("T".to_string()),
                    span: span(),
                }],
                derives: vec![],
                is_pub: true,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "pub struct Wrapper<T, U>(T)");
    }

    // ============================================================================
    // Enum
    // ============================================================================

    #[test]
    fn test_format_expr_enum() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Enum {
                name: "Color".to_string(),
                type_params: vec![],
                variants: vec![
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
                ],
                is_pub: false,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("enum Color {"));
        assert!(result.contains("Red"));
        assert!(result.contains("Green"));
    }

    #[test]
    fn test_format_expr_enum_pub_with_type_params() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Enum {
                name: "Option".to_string(),
                type_params: vec!["T".to_string()],
                variants: vec![
                    EnumVariant {
                        name: "Some".to_string(),
                        kind: EnumVariantKind::Tuple(vec![Type {
                            kind: TypeKind::Named("T".to_string()),
                            span: span(),
                        }]),
                        discriminant: None,
                    },
                    EnumVariant {
                        name: "None".to_string(),
                        kind: EnumVariantKind::Unit,
                        discriminant: None,
                    },
                ],
                is_pub: true,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("pub enum Option<T>"));
    }

    // ============================================================================
    // Trait
    // ============================================================================

    #[test]
    fn test_format_expr_trait() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Trait {
                name: "Printable".to_string(),
                type_params: vec![],
                associated_types: vec![],
                methods: vec![TraitMethod {
                    name: "print".to_string(),
                    params: vec![],
                    return_type: None,
                    body: None,
                    is_pub: false,
                }],
                is_pub: false,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("trait Printable {"));
        assert!(result.contains("print"));
    }

    #[test]
    fn test_format_expr_trait_pub_with_type_params() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Trait {
                name: "Container".to_string(),
                type_params: vec!["T".to_string()],
                associated_types: vec![],
                methods: vec![],
                is_pub: true,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("pub trait Container<T>"));
    }

    // ============================================================================
    // Impl
    // ============================================================================

    #[test]
    fn test_format_expr_impl_no_trait() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Impl {
                type_params: vec![],
                trait_name: None,
                for_type: "Point".to_string(),
                methods: vec![ImplMethod {
                    name: "new".to_string(),
                    params: vec![],
                    return_type: None,
                    body: Box::new(Expr::new(ExprKind::Block(vec![]), span())),
                    is_pub: false,
                }],
                is_pub: false,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("impl Point"));
        assert!(!result.contains(" for "));
    }

    #[test]
    fn test_format_expr_impl_with_trait() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Impl {
                type_params: vec![],
                trait_name: Some("Display".to_string()),
                for_type: "Point".to_string(),
                methods: vec![],
                is_pub: false,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("impl Display for Point"));
    }

    #[test]
    fn test_format_expr_impl_with_type_params() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Impl {
                type_params: vec!["T".to_string()],
                trait_name: Some("From".to_string()),
                for_type: "Wrapper".to_string(),
                methods: vec![],
                is_pub: false,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("impl<T> From for Wrapper"));
    }

    // ============================================================================
    // Class
    // ============================================================================

    #[test]
    fn test_format_expr_class_no_type_params() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Class {
                name: "Person".to_string(),
                type_params: vec![],
                superclass: None,
                traits: vec![],
                fields: vec![StructField {
                    name: "name".to_string(),
                    ty: Type {
                        kind: TypeKind::Named("String".to_string()),
                        span: span(),
                    },
                    visibility: Visibility::Private,
                    is_mut: false,
                    default_value: None,
                    decorators: vec![],
                }],
                constructors: vec![],
                methods: vec![],
                constants: vec![],
                properties: vec![],
                derives: vec![],
                decorators: vec![],
                is_pub: false,
                is_sealed: false,
                is_abstract: false,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "class Person { name: String }");
    }

    #[test]
    fn test_format_expr_class_with_type_params() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Class {
                name: "Box".to_string(),
                type_params: vec!["T".to_string()],
                superclass: None,
                traits: vec![],
                fields: vec![],
                constructors: vec![],
                methods: vec![],
                constants: vec![],
                properties: vec![],
                derives: vec![],
                decorators: vec![],
                is_pub: false,
                is_sealed: false,
                is_abstract: false,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "class Box<T> {  }");
    }

    // ============================================================================
    // Actor
    // ============================================================================

    #[test]
    fn test_format_expr_actor() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Actor {
                name: "Counter".to_string(),
                state: vec![StructField {
                    name: "count".to_string(),
                    ty: Type {
                        kind: TypeKind::Named("i32".to_string()),
                        span: span(),
                    },
                    visibility: Visibility::Private,
                    is_mut: false,
                    default_value: None,
                    decorators: vec![],
                }],
                handlers: vec![ActorHandler {
                    message_type: "Increment".to_string(),
                    params: vec![],
                    body: Box::new(Expr::new(ExprKind::Block(vec![]), span())),
                }],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("actor Counter {"));
        assert!(result.contains("count: i32"));
        assert!(result.contains("handle Increment"));
    }

    // ============================================================================
    // Effect
    // ============================================================================

    #[test]
    fn test_format_expr_effect() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Effect {
                name: "Console".to_string(),
                operations: vec![EffectOperation {
                    name: "print".to_string(),
                    params: vec![Param {
                        pattern: Pattern::Identifier("msg".to_string()),
                        ty: Type {
                            kind: TypeKind::Named("String".to_string()),
                            span: span(),
                        },
                        span: span(),
                        is_mutable: false,
                        default_value: None,
                    }],
                    return_type: Some(Type {
                        kind: TypeKind::Named("Unit".to_string()),
                        span: span(),
                    }),
                }],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("effect Console {"));
        assert!(result.contains("print("));
    }

    #[test]
    fn test_format_expr_effect_no_return_type() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Effect {
                name: "Log".to_string(),
                operations: vec![EffectOperation {
                    name: "log".to_string(),
                    params: vec![],
                    return_type: None,
                }],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("effect Log {"));
        assert!(result.contains("log()"));
    }

    // ============================================================================
    // Handle (effect handler)
    // ============================================================================

    #[test]
    fn test_format_expr_handle_no_params() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Handle {
                expr: Box::new(ident_expr("computation")),
                handlers: vec![EffectHandler {
                    operation: "print".to_string(),
                    params: vec![],
                    body: Box::new(Expr::new(ExprKind::Block(vec![]), span())),
                }],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("handle computation with {"));
        assert!(result.contains("print =>"));
    }

    #[test]
    fn test_format_expr_handle_with_params() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Handle {
                expr: Box::new(ident_expr("prog")),
                handlers: vec![EffectHandler {
                    operation: "read".to_string(),
                    params: vec![Pattern::Identifier("k".to_string())],
                    body: Box::new(str_expr("value")),
                }],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("handle prog with {"));
        assert!(result.contains("read("));
    }

    // ============================================================================
    // Extension
    // ============================================================================

    #[test]
    fn test_format_expr_extension() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Extension {
                target_type: "String".to_string(),
                methods: vec![ImplMethod {
                    name: "shout".to_string(),
                    params: vec![],
                    return_type: None,
                    body: Box::new(Expr::new(ExprKind::Block(vec![]), span())),
                    is_pub: false,
                }],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("extension String {"));
        assert!(result.contains("fun shout()"));
    }

    // ============================================================================
    // For with non-Identifier pattern (Debug fallback on line 492)
    // ============================================================================

    #[test]
    fn test_format_expr_for_with_tuple_pattern() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::For {
                var: String::new(),
                pattern: Some(Pattern::Tuple(vec![
                    Pattern::Identifier("k".to_string()),
                    Pattern::Identifier("v".to_string()),
                ])),
                iter: Box::new(ident_expr("map")),
                body: Box::new(Expr::new(ExprKind::Block(vec![]), span())),
                label: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("for "));
        assert!(result.contains(" in map"));
    }

    // ============================================================================
    // Let sequential with Call body (non-Block, non-Unit body branch)
    // ============================================================================

    #[test]
    fn test_format_expr_let_sequential_with_call_body() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                value: Box::new(int_expr(1)),
                body: Box::new(Expr::new(
                    ExprKind::Call {
                        func: Box::new(ident_expr("foo")),
                        args: vec![ident_expr("x")],
                    },
                    span(),
                )),
                is_mutable: false,
                type_annotation: None,
                else_block: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("let x = 1"));
        assert!(result.contains("foo(x)"));
    }

    #[test]
    fn test_format_expr_let_sequential_with_method_call_body() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Let {
                name: "v".to_string(),
                value: Box::new(int_expr(42)),
                body: Box::new(Expr::new(
                    ExprKind::MethodCall {
                        receiver: Box::new(ident_expr("v")),
                        method: "push".to_string(),
                        args: vec![int_expr(1)],
                    },
                    span(),
                )),
                is_mutable: false,
                type_annotation: None,
                else_block: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("let v = 42"));
        assert!(result.contains("v.push(1)"));
    }
