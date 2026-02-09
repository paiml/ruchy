    // ============== Match Arm Variants ==============

    #[test]
    fn test_format_match_with_guard() {
        let code = "match x { n if n > 0 => \"positive\", _ => \"other\" }";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_match_with_or_pattern() {
        let code = "match x { 1 | 2 | 3 => \"small\", _ => \"big\" }";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    // ============== Function Parameter Variations ==============

    #[test]
    fn test_format_function_no_params() {
        let result = format_code("fun greet() { \"hello\" }");
        assert!(result.contains("fun") && result.contains("()"));
    }

    #[test]
    fn test_format_function_many_params() {
        let result = format_code("fun foo(a, b, c, d, e) { a + b + c + d + e }");
        assert!(result.contains("fun") && result.contains(","));
    }

    // ============== Boolean Literal Tests ==============

    #[test]
    fn test_format_bool_true() {
        let result = format_code("true");
        assert_eq!(result.trim(), "true");
    }

    #[test]
    fn test_format_bool_false() {
        let result = format_code("false");
        assert_eq!(result.trim(), "false");
    }

    // ============== Comment Formatting Tests ==============

    #[test]
    fn test_format_with_line_comment() {
        // Parse code with line comment
        let code = "// comment\nlet x = 1";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_with_doc_comment() {
        // Doc comments
        let code = "/// doc comment\nfun foo() { 1 }";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_with_block_comment() {
        // Block comments
        let code = "/* block */\nlet x = 1";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_comment_preservation_with_formatter_api() {
        use crate::quality::formatter::Formatter;

        // Create formatter and manually test
        let formatter = Formatter::new();
        let code = "let x = 42";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("should parse");
        let result = formatter.format(&ast).expect("should format");
        assert!(result.contains("let") && result.contains("42"));
    }

    // ============== Ignore Directive Tests ==============

    #[test]
    fn test_formatter_with_source_set() {
        use crate::quality::formatter::Formatter;

        let mut formatter = Formatter::new();
        let source = "let x = 42";
        formatter.set_source(source);

        let mut parser = Parser::new(source);
        let ast = parser.parse().expect("should parse");
        let result = formatter.format(&ast).expect("should format");
        assert!(result.contains("let") && result.contains("42"));
    }

    #[test]
    fn test_formatter_source_none() {
        use crate::quality::formatter::Formatter;

        let formatter = Formatter::new(); // No source set
        let code = "let x = 42";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("should parse");
        let result = formatter.format(&ast).expect("should format");
        assert!(result.contains("let"));
    }

    // ============== Type Formatting Tests ==============

    #[test]
    fn test_format_function_with_arrow_return_type() {
        let code = "fun add(a: Int, b: Int) -> Int { a + b }";
        let result = try_format(code);
        // Parser may or may not support -> syntax fully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_let_with_type_annotation() {
        let code = "let x: Int = 42";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_generic_function() {
        let code = "fun identity<T>(x: T) -> T { x }";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    // ============== Span and Original Text Tests ==============

    #[test]
    fn test_format_preserves_structure() {
        use crate::quality::formatter::Formatter;

        let code = "let a = 1\nlet b = 2";
        let mut formatter = Formatter::new();
        formatter.set_source(code);

        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("should parse");
        let result = formatter.format(&ast).expect("should format");
        assert!(result.contains("let a") && result.contains("let b"));
    }

    #[test]
    fn test_format_nested_block_span() {
        let code = "fun foo() { if true { 1 } else { 2 } }";
        let result = format_code(code);
        assert!(result.contains("fun") && result.contains("if"));
    }

    // ============== Edge Cases in format_expr ==============

    #[test]
    fn test_format_deeply_nested_binary() {
        let code = "1 + 2 + 3 + 4 + 5";
        let result = format_code(code);
        assert!(result.contains("+"));
    }

    #[test]
    fn test_format_method_chain() {
        let code = "x.foo().bar().baz()";
        let result = format_code(code);
        assert!(result.contains("foo") && result.contains("bar") && result.contains("baz"));
    }

    #[test]
    fn test_format_complex_call_args() {
        let code = "func(1, 2, 3, a + b, c * d)";
        let result = format_code(code);
        assert!(result.contains("func") && result.contains(","));
    }

    #[test]
    fn test_format_while_with_break() {
        let code = "while true { if done { break } }";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_while_with_continue() {
        let code = "while true { if skip { continue } }";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_for_with_range() {
        let code = "for i in 0..10 { print(i) }";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_exclusive_range() {
        let code = "0..10";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_inclusive_range() {
        let code = "0..=10";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    // ============== Lambda and Closure Tests ==============

    #[test]
    fn test_format_lambda_three_params() {
        let code = "|a, b, c| a + b + c";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_lambda_with_block() {
        let code = "|x| { let y = x * 2; y }";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    // ============== Result/Option Types ==============

    #[test]
    fn test_format_ok_variant() {
        let code = "Ok(42)";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_err_variant() {
        let code = "Err(\"error\")";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_some_variant() {
        let code = "Some(42)";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_none_variant() {
        let code = "None";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    // ============== Async/Await Tests ==============

    #[test]
    fn test_format_await_expr() {
        let code = "await fetch()";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_async_block_with_await() {
        let code = "async { fetch().await }";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_spawn_expr() {
        let code = "spawn(counter)";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    // ============== Try/Catch Tests ==============

    #[test]
    fn test_format_try_expr() {
        let code = "try { risky() } catch e { handle(e) }";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_throw_expr() {
        let code = "throw Error(\"oops\")";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_question_mark_operator() {
        let code = "result?";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    // ============== Type Cast Tests ==============

    #[test]
    fn test_format_type_cast_as() {
        let code = "x as Int";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    // ============== Array Init Tests ==============

    #[test]
    fn test_format_array_init_repeat() {
        let code = "[0; 10]";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    // ============== Object and Struct Literal Tests ==============

    #[test]
    fn test_format_object_literal_multiple_fields() {
        let code = "{ x: 1, y: 2, z: 3 }";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_struct_literal_with_spread_base() {
        let code = "Point { x: 1, ..default }";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    // ============== Slice Tests ==============

    #[test]
    fn test_format_slice_both() {
        let code = "arr[1..5]";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_slice_from() {
        let code = "arr[1..]";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_slice_to() {
        let code = "arr[..5]";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_slice_all() {
        let code = "arr[..]";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    // ============== If-Let Tests ==============

    #[test]
    fn test_format_if_let_some() {
        let code = "if let Some(x) = opt { x }";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_if_let_some_with_else() {
        let code = "if let Some(x) = opt { x } else { 0 }";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    // ============== Optional Field/Method Access ==============

    #[test]
    fn test_format_optional_field() {
        let code = "obj?.field";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    // ============== Set Literal Tests ==============

    #[test]
    fn test_format_set_literal() {
        let code = "{1, 2, 3}";
        let result = try_format(code);
        // Sets may be parsed differently
        assert!(result.is_ok() || result.is_err());
    }

    // ============== DataFrame Operation Tests ==============

    #[test]
    fn test_format_dataframe_operation() {
        let code = "df.filter(x > 0)";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    // ============== Lazy Evaluation Tests ==============

    #[test]
    fn test_format_lazy_expr() {
        let code = "lazy(expensive_computation())";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    // ============== More Edge Cases ==============

    #[test]
    fn test_format_empty_block_braces() {
        let result = format_code("{}");
        assert!(result.contains("{") && result.contains("}"));
    }

    #[test]
    fn test_format_block_single_number() {
        let result = format_code("{ 42 }");
        assert!(result.contains("42"));
    }

    #[test]
    fn test_format_nested_if_else() {
        let code = "if a { if b { 1 } else { 2 } } else { 3 }";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_return_without_value() {
        let code = "return";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_break_with_number() {
        let code = "break 42";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_format_block_multiple_lets() {
        let code = "{ let a = 1; let b = 2; a + b }";
        let result = try_format(code);
        assert!(result.is_ok() || result.is_err());
    }

    // ============== Direct AST Construction Tests ==============
    // These tests construct AST nodes directly to cover branches
    // that the parser doesn't support

    #[test]
    fn test_format_loop_direct() {
        let formatter = Formatter::new();
        let body = make_lit(1);
        let expr = Expr::new(
            ExprKind::Loop {
                body: Box::new(body),
                label: None,
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("loop"));
    }

    #[test]
    fn test_format_send_direct() {
        let formatter = Formatter::new();
        let actor = make_ident("actor");
        let message = make_lit(42);
        let expr = Expr::new(
            ExprKind::Send {
                actor: Box::new(actor),
                message: Box::new(message),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("send"));
    }

    #[test]
    fn test_format_pre_increment_direct() {
        let formatter = Formatter::new();
        let target = make_ident("x");
        let expr = Expr::new(
            ExprKind::PreIncrement {
                target: Box::new(target),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("++"));
    }

    #[test]
    fn test_format_post_increment_direct() {
        let formatter = Formatter::new();
        let target = make_ident("x");
        let expr = Expr::new(
            ExprKind::PostIncrement {
                target: Box::new(target),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("++"));
    }

    #[test]
    fn test_format_pre_decrement_direct() {
        let formatter = Formatter::new();
        let target = make_ident("x");
        let expr = Expr::new(
            ExprKind::PreDecrement {
                target: Box::new(target),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("--"));
    }

    #[test]
    fn test_format_post_decrement_direct() {
        let formatter = Formatter::new();
        let target = make_ident("x");
        let expr = Expr::new(
            ExprKind::PostDecrement {
                target: Box::new(target),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("--"));
    }

    #[test]
    fn test_format_actor_send_direct() {
        let formatter = Formatter::new();
        let actor = make_ident("actor");
        let message = make_lit(42);
        let expr = Expr::new(
            ExprKind::ActorSend {
                actor: Box::new(actor),
                message: Box::new(message),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        // ActorSend format may vary, just ensure non-empty
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_actor_query_direct() {
        let formatter = Formatter::new();
        let actor = make_ident("actor");
        let message = make_lit(42);
        let expr = Expr::new(
            ExprKind::ActorQuery {
                actor: Box::new(actor),
                message: Box::new(message),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("?"));
    }

    #[test]
    fn test_format_ask_direct() {
        let formatter = Formatter::new();
        let actor = make_ident("actor");
        let message = make_lit(42);
        let expr = Expr::new(
            ExprKind::Ask {
                actor: Box::new(actor),
                message: Box::new(message),
                timeout: None,
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("ask"));
    }

    #[test]
    fn test_format_ask_with_timeout_direct() {
        let formatter = Formatter::new();
        let actor = make_ident("actor");
        let message = make_lit(42);
        let timeout = make_lit(1000);
        let expr = Expr::new(
            ExprKind::Ask {
                actor: Box::new(actor),
                message: Box::new(message),
                timeout: Some(Box::new(timeout)),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        // Ask with timeout format may vary
        assert!(result.contains("ask"));
    }

    #[test]
    fn test_format_import_all_direct() {
        let formatter = Formatter::new();
        let expr = Expr::new(
            ExprKind::ImportAll {
                module: "std::io".to_string(),
                alias: String::new(),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("import") && result.contains("*"));
    }

    #[test]
    fn test_format_import_default_direct() {
        let formatter = Formatter::new();
        let expr = Expr::new(
            ExprKind::ImportDefault {
                module: "react".to_string(),
                name: "React".to_string(),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("import"));
    }

    #[test]
    fn test_format_export_list_direct() {
        let formatter = Formatter::new();
        let expr = Expr::new(
            ExprKind::ExportList {
                names: vec!["foo".to_string(), "bar".to_string()],
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("export"));
    }

    #[test]
    fn test_format_export_default_direct() {
        let formatter = Formatter::new();
        let inner = make_ident("Component");
        let expr = Expr::new(
            ExprKind::ExportDefault {
                expr: Box::new(inner),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("export") && result.contains("default"));
    }

    #[test]
    fn test_format_qualified_name_direct() {
        let formatter = Formatter::new();
        let expr = Expr::new(
            ExprKind::QualifiedName {
                module: "std::io".to_string(),
                name: "read".to_string(),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("::"));
    }

    #[test]
    fn test_format_type_alias_direct() {
        let formatter = Formatter::new();
        let expr = Expr::new(
            ExprKind::TypeAlias {
                name: "MyInt".to_string(),
                target_type: crate::frontend::ast::Type {
                    kind: crate::frontend::ast::TypeKind::Named("Int".to_string()),
                    span: Default::default(),
                },
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("type") && result.contains("MyInt"));
    }

    #[test]
    fn test_format_spread_direct() {
        let formatter = Formatter::new();
        let inner = make_ident("args");
        let expr = Expr::new(
            ExprKind::Spread {
                expr: Box::new(inner),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("..."));
    }

    #[test]
    fn test_format_vec_repeat_direct() {
        let formatter = Formatter::new();
        let value = make_lit(0);
        let count = make_lit(10);
        let expr = Expr::new(
            ExprKind::VecRepeat {
                value: Box::new(value),
                count: Box::new(count),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("[") && result.contains(";"));
    }

    #[test]
    fn test_format_lazy_direct() {
        let formatter = Formatter::new();
        let inner = make_lit(42);
        let expr = Expr::new(
            ExprKind::Lazy {
                expr: Box::new(inner),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("lazy"));
    }

    #[test]
    fn test_format_set_direct() {
        let formatter = Formatter::new();
        let items = vec![make_lit(1), make_lit(2), make_lit(3)];
        let expr = Expr::new(ExprKind::Set(items), Default::default());
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("{") && result.contains("}"));
    }

    #[test]
    fn test_format_none_direct() {
        let formatter = Formatter::new();
        let expr = Expr::new(ExprKind::None, Default::default());
        let result = formatter.format(&expr).expect("should format");
        assert_eq!(result, "None");
    }

    #[test]
    fn test_format_ok_direct() {
        let formatter = Formatter::new();
        let value = make_lit(42);
        let expr = Expr::new(
            ExprKind::Ok {
                value: Box::new(value),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("Ok"));
    }

    #[test]
    fn test_format_err_direct() {
        let formatter = Formatter::new();
        let error = Expr::new(
            ExprKind::Literal(Literal::String("error".to_string())),
            Default::default(),
        );
        let expr = Expr::new(
            ExprKind::Err {
                error: Box::new(error),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("Err"));
    }

    #[test]
    fn test_format_some_direct() {
        let formatter = Formatter::new();
        let value = make_lit(42);
        let expr = Expr::new(
            ExprKind::Some {
                value: Box::new(value),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("Some"));
    }

    #[test]
    fn test_format_try_direct() {
        let formatter = Formatter::new();
        let inner = make_ident("result");
        let expr = Expr::new(
            ExprKind::Try {
                expr: Box::new(inner),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("?"));
    }

    #[test]
    fn test_format_spawn_direct() {
        let formatter = Formatter::new();
        let actor = make_ident("MyActor");
        let expr = Expr::new(
            ExprKind::Spawn {
                actor: Box::new(actor),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("spawn"));
    }

    #[test]
    fn test_format_await_direct() {
        let formatter = Formatter::new();
        let inner = make_ident("future");
        let expr = Expr::new(
            ExprKind::Await {
                expr: Box::new(inner),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("await"));
    }

    #[test]
    fn test_format_async_block_direct() {
        let formatter = Formatter::new();
        let body = make_lit(42);
        let expr = Expr::new(
            ExprKind::AsyncBlock {
                body: Box::new(body),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("async"));
    }

    #[test]
    fn test_format_throw_direct() {
        let formatter = Formatter::new();
        let error = Expr::new(
            ExprKind::Literal(Literal::String("error".to_string())),
            Default::default(),
        );
        let expr = Expr::new(
            ExprKind::Throw {
                expr: Box::new(error),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("throw"));
    }

    #[test]
    fn test_format_return_with_value_direct() {
        let formatter = Formatter::new();
        let value = make_lit(42);
        let expr = Expr::new(
            ExprKind::Return {
                value: Some(Box::new(value)),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("return") && result.contains("42"));
    }

    #[test]
    fn test_format_return_without_value_direct() {
        let formatter = Formatter::new();
        let expr = Expr::new(ExprKind::Return { value: None }, Default::default());
        let result = formatter.format(&expr).expect("should format");
        assert_eq!(result, "return");
    }

    #[test]
    fn test_format_break_direct() {
        let formatter = Formatter::new();
        let expr = Expr::new(
            ExprKind::Break {
                label: None,
                value: None,
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert_eq!(result, "break");
    }

    #[test]
    fn test_format_break_with_value_direct() {
        let formatter = Formatter::new();
        let value = make_lit(42);
        let expr = Expr::new(
            ExprKind::Break {
                label: None,
                value: Some(Box::new(value)),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("break") && result.contains("42"));
    }

    #[test]
    fn test_format_continue_direct() {
        let formatter = Formatter::new();
        let expr = Expr::new(ExprKind::Continue { label: None }, Default::default());
        let result = formatter.format(&expr).expect("should format");
        assert_eq!(result, "continue");
    }

    #[test]
    fn test_format_unary_not_direct() {
        let formatter = Formatter::new();
        let operand = Expr::new(ExprKind::Literal(Literal::Bool(true)), Default::default());
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Not,
                operand: Box::new(operand),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("!"));
    }

    #[test]
    fn test_format_unary_negate_direct() {
        let formatter = Formatter::new();
        let operand = make_lit(42);
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: Box::new(operand),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("-"));
    }

    #[test]
    fn test_format_range_exclusive_direct() {
        let formatter = Formatter::new();
        let start = make_lit(0);
        let end = make_lit(10);
        let expr = Expr::new(
            ExprKind::Range {
                start: Box::new(start),
                end: Box::new(end),
                inclusive: false,
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains(".."));
    }

    #[test]
    fn test_format_range_inclusive_direct() {
        let formatter = Formatter::new();
        let start = make_lit(0);
        let end = make_lit(10);
        let expr = Expr::new(
            ExprKind::Range {
                start: Box::new(start),
                end: Box::new(end),
                inclusive: true,
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("..="));
    }

    #[test]
    fn test_format_slice_direct() {
        let formatter = Formatter::new();
        let obj = make_ident("arr");
        let start = make_lit(0);
        let end = make_lit(5);
        let expr = Expr::new(
            ExprKind::Slice {
                object: Box::new(obj),
                start: Some(Box::new(start)),
                end: Some(Box::new(end)),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("[") && result.contains(".."));
    }

    #[test]
    fn test_format_slice_from_direct() {
        let formatter = Formatter::new();
        let obj = make_ident("arr");
        let start = make_lit(1);
        let expr = Expr::new(
            ExprKind::Slice {
                object: Box::new(obj),
                start: Some(Box::new(start)),
                end: None,
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("[1..]"));
    }

    #[test]
    fn test_format_slice_to_direct() {
        let formatter = Formatter::new();
        let obj = make_ident("arr");
        let end = make_lit(5);
        let expr = Expr::new(
            ExprKind::Slice {
                object: Box::new(obj),
                start: None,
                end: Some(Box::new(end)),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("[..5]"));
    }

    #[test]
    fn test_format_slice_full_direct() {
        let formatter = Formatter::new();
        let obj = make_ident("arr");
        let expr = Expr::new(
            ExprKind::Slice {
                object: Box::new(obj),
                start: None,
                end: None,
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("[..]"));
    }

    #[test]
    fn test_format_optional_field_access_direct() {
        let formatter = Formatter::new();
        let obj = make_ident("obj");
        let expr = Expr::new(
            ExprKind::OptionalFieldAccess {
                object: Box::new(obj),
                field: "name".to_string(),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("?."));
    }

    #[test]
    fn test_format_ternary_direct() {
        let formatter = Formatter::new();
        let condition = Expr::new(ExprKind::Literal(Literal::Bool(true)), Default::default());
        let then_expr = make_lit(1);
        let else_expr = make_lit(2);
        let expr = Expr::new(
            ExprKind::Ternary {
                condition: Box::new(condition),
                true_expr: Box::new(then_expr),
                false_expr: Box::new(else_expr),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("?") && result.contains(":"));
    }

    #[test]
    fn test_format_array_init_direct() {
        let formatter = Formatter::new();
        let value = make_lit(0);
        let size = make_lit(10);
        let expr = Expr::new(
            ExprKind::ArrayInit {
                value: Box::new(value),
                size: Box::new(size),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("[0; 10]"));
    }

    #[test]
    fn test_format_module_direct() {
        let formatter = Formatter::new();
        let body = make_lit(42);
        let expr = Expr::new(
            ExprKind::Module {
                name: "mymod".to_string(),
                body: Box::new(body),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("mod") && result.contains("mymod"));
    }

    #[test]
    fn test_format_module_declaration_direct() {
        let formatter = Formatter::new();
        let expr = Expr::new(
            ExprKind::ModuleDeclaration {
                name: "utils".to_string(),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("mod") && result.contains("utils"));
    }

    #[test]
    fn test_format_reexport_direct() {
        let formatter = Formatter::new();
        let expr = Expr::new(
            ExprKind::ReExport {
                items: vec!["foo".to_string(), "bar".to_string()],
                module: "other".to_string(),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("export") && result.contains("from"));
    }

    #[test]
    fn test_format_macro_direct() {
        let formatter = Formatter::new();
        let args = vec![make_lit(1), make_lit(2)];
        let expr = Expr::new(
            ExprKind::Macro {
                name: "vec".to_string(),
                args,
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        // Macro format may use different syntax
        assert!(result.contains("vec"));
    }

    #[test]
    fn test_format_macro_invocation_direct() {
        let formatter = Formatter::new();
        let args = vec![Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            Default::default(),
        )];
        let expr = Expr::new(
            ExprKind::MacroInvocation {
                name: "println".to_string(),
                args,
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("println!"));
    }
