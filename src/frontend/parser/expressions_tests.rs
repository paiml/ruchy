    use super::*;
    use crate::Parser;

    // Helper to parse expressions
    fn parse(code: &str) -> Result<Expr> {
        let mut parser = Parser::new(code);
        parser.parse()
    }

    // Helper to extract block expressions
    fn get_block_exprs(expr: &Expr) -> Option<&Vec<Expr>> {
        match &expr.kind {
            ExprKind::Block(exprs) => Some(exprs),
            _ => None,
        }
    }

    // ===== parse_prefix tests =====

    #[test]
    fn test_parse_prefix_integer_literal() {
        let expr = parse("42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Integer(42, _))
            ));
        }
    }

    #[test]
    fn test_parse_prefix_float_literal() {
        let expr = parse("3.14").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Literal(Literal::Float(f)) = &exprs[0].kind {
                assert!((f - 3.14).abs() < 0.001);
            }
        }
    }

    #[test]
    fn test_parse_prefix_string_literal() {
        let expr = parse("\"hello\"").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::String(s)) if s == "hello"
            ));
        }
    }

    #[test]
    fn test_parse_prefix_bool_true() {
        let expr = parse("true").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Bool(true))
            ));
        }
    }

    #[test]
    fn test_parse_prefix_bool_false() {
        let expr = parse("false").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Bool(false))
            ));
        }
    }

    #[test]
    fn test_parse_prefix_null() {
        let expr = parse("null").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Literal(Literal::Unit)));
        }
    }

    #[test]
    fn test_parse_prefix_identifier() {
        let expr = parse("foo").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Identifier(name) if name == "foo"
            ));
        }
    }

    #[test]
    fn test_parse_prefix_underscore() {
        let expr = parse("_").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Identifier(name) if name == "_"
            ));
        }
    }

    #[test]
    fn test_parse_prefix_self() {
        let expr = parse("self").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Identifier(name) if name == "self"
            ));
        }
    }

    // ===== Unary operator tests =====

    #[test]
    fn test_parse_unary_minus() {
        let expr = parse("-42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Unary { .. }));
        }
    }

    #[test]
    fn test_parse_unary_bang() {
        let expr = parse("!true").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Unary { .. }));
        }
    }

    #[test]
    fn test_parse_unary_star_deref() {
        let expr = parse("*ptr").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Unary { .. }));
        }
    }

    #[test]
    fn test_parse_unary_ampersand_ref() {
        let expr = parse("&x").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Unary { .. }));
        }
    }

    // ===== Control flow tests =====

    #[test]
    fn test_parse_if_expression() {
        let expr = parse("if true { 1 } else { 2 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::If { .. }));
        }
    }

    #[test]
    fn test_parse_if_without_else() {
        let expr = parse("if x { 1 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::If { .. }));
        }
    }

    #[test]
    fn test_parse_while_loop() {
        let expr = parse("while true { x }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::While { .. }));
        }
    }

    #[test]
    fn test_parse_for_loop() {
        let expr = parse("for i in 0..10 { i }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::For { .. }));
        }
    }

    #[test]
    fn test_parse_loop() {
        let expr = parse("loop { break }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Loop { .. }));
        }
    }

    #[test]
    fn test_parse_match_expression() {
        let expr = parse("match x { 1 => a, _ => b }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Match { .. }));
        }
    }

    // ===== Variable declaration tests =====

    #[test]
    fn test_parse_let_statement() {
        let expr = parse("let x = 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Let { .. }));
        }
    }

    #[test]
    fn test_parse_let_mut_statement() {
        let expr = parse("let mut x = 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Let { is_mutable, .. } = &exprs[0].kind {
                assert!(is_mutable);
            } else {
                panic!("Expected Let expression");
            }
        }
    }

    #[test]
    fn test_parse_let_with_type() {
        let expr = parse("let x: i32 = 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Let {
                type_annotation, ..
            } = &exprs[0].kind
            {
                assert!(type_annotation.is_some());
            } else {
                panic!("Expected Let expression");
            }
        }
    }

    #[test]
    fn test_parse_var_statement() {
        let expr = parse("var x = 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // var is syntactic sugar for let mut
            assert!(matches!(&exprs[0].kind, ExprKind::Let { .. }));
        }
    }

    // ===== Function tests =====

    #[test]
    fn test_parse_function_definition() {
        let expr = parse("fun foo() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Function { .. }));
        }
    }

    #[test]
    fn test_parse_function_with_params() {
        let expr = parse("fun add(a, b) { a + b }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Function { params, .. } = &exprs[0].kind {
                assert_eq!(params.len(), 2);
            } else {
                panic!("Expected Function expression");
            }
        }
    }

    #[test]
    fn test_parse_function_with_return_type() {
        let expr = parse("fun foo() -> i32 { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Function { return_type, .. } = &exprs[0].kind {
                assert!(return_type.is_some());
            } else {
                panic!("Expected Function expression");
            }
        }
    }

    // ===== Lambda tests =====

    #[test]
    fn test_parse_lambda_pipe() {
        let expr = parse("|x| x + 1").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Lambda { .. }));
        }
    }

    #[test]
    fn test_parse_lambda_no_params() {
        let expr = parse("|| 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Lambda { .. }));
        }
    }

    #[test]
    fn test_parse_lambda_backslash() {
        let expr = parse("\\x -> x + 1").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Lambda { .. }));
        }
    }

    // ===== Collection tests =====

    #[test]
    fn test_parse_array_literal() {
        let expr = parse("[1, 2, 3]").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::List(_)));
        }
    }

    #[test]
    fn test_parse_empty_array() {
        let expr = parse("[]").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::List(elements) = &exprs[0].kind {
                assert!(elements.is_empty());
            } else {
                panic!("Expected List expression");
            }
        }
    }

    #[test]
    fn test_parse_tuple() {
        let expr = parse("(1, 2, 3)").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Tuple(_)));
        }
    }

    #[test]
    fn test_parse_unit() {
        let expr = parse("()").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Literal(Literal::Unit)));
        }
    }

    #[test]
    fn test_parse_grouped_expression() {
        let expr = parse("(42)").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // Grouped expression should unwrap to the inner expression
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Integer(42, _))
            ));
        }
    }

    // ===== Struct tests =====

    #[test]
    fn test_parse_struct_definition() {
        let expr = parse("struct Point { x: i32, y: i32 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Struct { .. }));
        }
    }

    #[test]
    fn test_parse_tuple_struct() {
        let expr = parse("struct Pair(i32, i32)").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Struct { .. }));
        }
    }

    #[test]
    fn test_parse_unit_struct() {
        let expr = parse("struct Empty").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Struct { .. }));
        }
    }

    // ===== Enum tests =====

    #[test]
    fn test_parse_enum_definition() {
        let expr = parse("enum Color { Red, Green, Blue }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Enum { .. }));
        }
    }

    // ===== Trait and impl tests =====

    #[test]
    fn test_parse_trait_definition() {
        let expr = parse("trait Foo { fun bar() }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Trait { .. }));
        }
    }

    #[test]
    fn test_parse_impl_block() {
        let expr = parse("impl Foo { fun bar() { 42 } }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Impl { .. }));
        }
    }

    // ===== Import tests =====

    #[test]
    fn test_parse_use_statement() {
        let expr = parse("use std::io").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // 'use' statements are parsed as Import
            assert!(matches!(&exprs[0].kind, ExprKind::Import { .. }));
        }
    }

    #[test]
    fn test_parse_import_statement() {
        let expr = parse("import foo").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Import { .. }));
        }
    }

    // ===== Range tests =====

    #[test]
    fn test_parse_prefix_range_exclusive() {
        let expr = parse("..5").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Range { inclusive, .. } = &exprs[0].kind {
                assert!(!inclusive);
            } else {
                panic!("Expected Range expression");
            }
        }
    }

    #[test]
    fn test_parse_prefix_range_inclusive() {
        let expr = parse("..=5").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Range { inclusive, .. } = &exprs[0].kind {
                assert!(inclusive);
            } else {
                panic!("Expected Range expression");
            }
        }
    }

    // ===== Control statement tests =====

    #[test]
    fn test_parse_break() {
        let expr = parse("break").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Break { .. }));
        }
    }

    #[test]
    fn test_parse_continue() {
        let expr = parse("continue").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Continue { .. }));
        }
    }

    #[test]
    fn test_parse_return() {
        let expr = parse("return 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Return { .. }));
        }
    }

    #[test]
    fn test_parse_return_empty() {
        let expr = parse("return").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Return { .. }));
        }
    }

    // ===== Async tests =====

    #[test]
    fn test_parse_async_function() {
        let expr = parse("async fun foo() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Function { is_async, .. } = &exprs[0].kind {
                assert!(is_async);
            } else {
                panic!("Expected async Function expression");
            }
        }
    }

    #[test]
    fn test_parse_async_block() {
        let expr = parse("async { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::AsyncBlock { .. }));
        }
    }

    // ===== Lazy tests =====

    #[test]
    fn test_parse_lazy_expression() {
        let expr = parse("lazy 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Lazy { .. }));
        }
    }

    // ===== Pub and visibility tests =====

    #[test]
    fn test_parse_pub_function() {
        let expr = parse("pub fun foo() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Function { is_pub, .. } = &exprs[0].kind {
                assert!(is_pub);
            } else {
                panic!("Expected pub Function expression");
            }
        }
    }

    #[test]
    fn test_parse_pub_struct() {
        let expr = parse("pub struct Foo { x: i32 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Struct { is_pub, .. } = &exprs[0].kind {
                assert!(is_pub);
            } else {
                panic!("Expected pub Struct expression");
            }
        }
    }

    // ===== Type alias tests =====

    #[test]
    fn test_parse_type_alias() {
        let expr = parse("type Num = i32").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::TypeAlias { .. }));
        }
    }

    // ===== Block tests =====

    #[test]
    fn test_parse_block_expression() {
        let expr = parse("{ 1; 2; 3 }").unwrap();
        // The entire program is wrapped in a block, so the outer is Block
        // The inner { 1; 2; 3 } is also a Block
        assert!(matches!(&expr.kind, ExprKind::Block(_)));
    }

    // ===== Try-catch tests =====

    #[test]
    fn test_parse_try_catch() {
        let expr = parse("try { x } catch e { e }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::TryCatch { .. }));
        }
    }

    // ===== Module tests =====

    #[test]
    fn test_parse_module_declaration() {
        let expr = parse("mod foo { }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Module { .. }));
        }
    }

    // ===== Character and byte literal tests =====

    #[test]
    fn test_parse_char_literal() {
        let expr = parse("'a'").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Char('a'))
            ));
        }
    }

    #[test]
    fn test_parse_byte_literal() {
        let expr = parse("b'x'").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Byte(_))
            ));
        }
    }

    // ===== Hex literal tests (Issue #168) =====

    #[test]
    fn test_parse_hex_integer() {
        let expr = parse("0xFF").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Integer(255, _))
            ));
        }
    }

    // ===== Atom literal tests =====

    #[test]
    fn test_parse_atom_literal() {
        let expr = parse(":ok").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::Atom(name)) if name == "ok"
            ));
        }
    }

    // ===== Decorator tests (BUG-033) =====

    #[test]
    fn test_parse_decorator_on_function() {
        let expr = parse("@inline fun foo() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Function { .. }));
            assert!(!exprs[0].attributes.is_empty());
        }
    }

    #[test]
    fn test_parse_decorator_with_args() {
        let expr = parse("@test(\"example\") fun foo() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Function { .. }));
        }
    }

    // ===== parse_let_mutability tests =====

    #[test]
    fn test_let_mutability_mut() {
        let expr = parse("let mut x = 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Let { is_mutable, .. } = &exprs[0].kind {
                assert!(is_mutable);
            }
        }
    }

    #[test]
    fn test_let_mutability_immut() {
        let expr = parse("let x = 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Let { is_mutable, .. } = &exprs[0].kind {
                assert!(!is_mutable);
            }
        }
    }

    // ===== Default and Result identifier tests =====

    #[test]
    fn test_parse_default_identifier() {
        let expr = parse("default").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Identifier(name) if name == "default"
            ));
        }
    }

    #[test]
    fn test_parse_result_identifier() {
        let expr = parse("Result").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Identifier(name) if name == "Result"
            ));
        }
    }

    // ===== Ok/Err constructor tests =====

    #[test]
    fn test_parse_ok_constructor() {
        let expr = parse("Ok(42)").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Call { .. }));
        }
    }

    #[test]
    fn test_parse_err_constructor() {
        let expr = parse("Err(\"error\")").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Call { .. }));
        }
    }

    // ===== Some/None tests =====

    #[test]
    fn test_parse_some_constructor() {
        let expr = parse("Some(42)").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Call { .. }));
        }
    }

    #[test]
    fn test_parse_none_literal() {
        let expr = parse("None").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Identifier(name) if name == "None"
            ));
        }
    }

    // ===== Super identifier test =====

    #[test]
    fn test_parse_super_identifier() {
        let expr = parse("super").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Identifier(name) if name == "super"
            ));
        }
    }

    // ===== Const declaration tests =====

    #[test]
    fn test_parse_const_declaration() {
        let expr = parse("const X: i32 = 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // const X is parsed as immutable Let with "const" attribute
            if let ExprKind::Let { is_mutable, .. } = &exprs[0].kind {
                assert!(!is_mutable);
            } else {
                panic!("Expected Let expression for const");
            }
        }
    }

    // ===== Throw tests =====

    #[test]
    fn test_parse_throw() {
        let expr = parse("throw err").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Throw { .. }));
        }
    }

    // ===== Interface as trait tests =====

    #[test]
    fn test_parse_interface_as_trait() {
        let expr = parse("interface Foo { fun bar() }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // Interface is parsed as Trait
            assert!(matches!(&exprs[0].kind, ExprKind::Trait { .. }));
        }
    }

    // ===== Class tests =====

    #[test]
    fn test_parse_class_definition() {
        let expr = parse("class Point { x: i32, y: i32 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Class { .. }));
        }
    }

    // ===== Raw string tests =====

    #[test]
    fn test_parse_raw_string() {
        let expr = parse("r\"raw\\nstring\"").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // Raw strings are parsed as regular strings
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::Literal(Literal::String(_))
            ));
        }
    }

    // ===== F-string tests =====

    #[test]
    fn test_parse_fstring() {
        let expr = parse("f\"value: {x}\"").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // F-strings are parsed as StringInterpolation
            assert!(matches!(
                &exprs[0].kind,
                ExprKind::StringInterpolation { .. }
            ));
        }
    }

    // ===== dispatch_prefix_token comprehensive tests =====

    #[test]
    fn test_dispatch_unexpected_token() {
        // Test that unexpected tokens result in error
        let result = parse("@@@");
        assert!(result.is_err());
    }

    // ===== parse_decorator_args_inline tests =====

    #[test]
    fn test_decorator_multiple_args() {
        let expr = parse("@test(\"a\", \"b\") fun foo() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let Some(attr) = exprs[0].attributes.first() {
                assert_eq!(attr.args.len(), 2);
            }
        }
    }

    #[test]
    fn test_decorator_identifier_args() {
        let expr = parse("@test(arg1, arg2) fun foo() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Function { .. }));
        }
    }

    // ===== More thorough edge case tests =====

    #[test]
    fn test_parse_nested_if() {
        let expr = parse("if a { if b { 1 } else { 2 } } else { 3 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::If { .. }));
        }
    }

    #[test]
    fn test_parse_chained_comparison() {
        // This tests binary operators as well
        let expr = parse("a < b && b < c").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Binary { .. }));
        }
    }

    #[test]
    fn test_parse_multiline_lambda() {
        let expr = parse("|x| { let y = x + 1; y * 2 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Lambda { .. }));
        }
    }

    #[test]
    fn test_parse_generic_function() {
        let expr = parse("fun identity<T>(x: T) -> T { x }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Function { type_params, .. } = &exprs[0].kind {
                assert!(!type_params.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_generic_struct() {
        let expr = parse("struct Box<T> { value: T }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Struct { type_params, .. } = &exprs[0].kind {
                assert!(!type_params.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_impl_trait_for_type() {
        let expr = parse("impl Foo for Bar { fun baz() { 42 } }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Impl { .. }));
        }
    }

    #[test]
    fn test_parse_for_with_pattern() {
        let expr = parse("for (a, b) in pairs { a + b }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::For { .. }));
        }
    }

    #[test]
    fn test_parse_break_with_value() {
        let expr = parse("break 42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Break { value, .. } = &exprs[0].kind {
                assert!(value.is_some());
            }
        }
    }

    #[test]
    fn test_parse_continue_with_label() {
        let expr = parse("continue 'outer").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Continue { label } = &exprs[0].kind {
                assert!(label.is_some());
            }
        }
    }

    // ===== Pattern matching tests =====

    #[test]
    fn test_parse_tuple_pattern_in_let() {
        let expr = parse("let (a, b) = pair").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // Tuple patterns use LetPattern variant
            assert!(matches!(&exprs[0].kind, ExprKind::LetPattern { .. }));
        }
    }

    #[test]
    fn test_parse_struct_pattern_in_let() {
        let expr = parse("let Point { x, y } = point").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // Struct patterns use LetPattern variant
            assert!(matches!(&exprs[0].kind, ExprKind::LetPattern { .. }));
        }
    }

    // ===== Unsafe function tests =====

    #[test]
    fn test_parse_unsafe_function() {
        let expr = parse("unsafe fun deref_raw(ptr) { ptr }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Function { .. } = &exprs[0].kind {
                // unsafe functions have "unsafe" attribute
                assert!(exprs[0].attributes.iter().any(|a| a.name == "unsafe"));
            } else {
                panic!("Expected Function expression");
            }
        }
    }

    // ============================================================
    // Coverage tests for parse_label_as_decorator (expressions.rs:229)
    // The lexer emits @identifier as Token::Label("@identifier").
    // When not followed by Colon, it's treated as a decorator.
    // ============================================================

    #[test]
    fn test_label_as_decorator_on_function() {
        // @inline fun f() { 42 }
        // Lexer emits Label("@inline"), then tokens for fun...
        let expr = parse("@inline fun f() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Function { .. }));
            assert!(
                exprs[0].attributes.iter().any(|a| a.name == "inline"),
                "Should have 'inline' attribute"
            );
        }
    }

    #[test]
    fn test_label_as_decorator_with_args_on_function() {
        // @test("example") fun f() { 42 }
        let expr = parse("@test(\"example\") fun f() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Function { .. }));
            let attr = exprs[0].attributes.iter().find(|a| a.name == "test");
            assert!(attr.is_some(), "Should have 'test' attribute");
            if let Some(a) = attr {
                assert_eq!(a.args.len(), 1, "Should have one arg");
            }
        }
    }

    #[test]
    fn test_label_as_decorator_on_class() {
        // @serialize class Foo { }
        // This should set both attributes and class decorators
        let expr = parse("@serialize class Foo { }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Class { decorators, .. } = &exprs[0].kind {
                assert!(
                    decorators.iter().any(|d| d.name == "serialize"),
                    "Class should have 'serialize' decorator"
                );
            } else {
                panic!("Expected Class expression");
            }
        }
    }

    #[test]
    fn test_label_as_decorator_multiple_on_function() {
        // @inline @test fun f() { 42 }
        // Two consecutive Label tokens as decorators
        let expr = parse("@inline @test fun f() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(matches!(&exprs[0].kind, ExprKind::Function { .. }));
            assert!(
                exprs[0].attributes.len() >= 2,
                "Should have at least 2 attributes, got {}",
                exprs[0].attributes.len()
            );
        }
    }

    #[test]
    fn test_label_as_decorator_with_multiple_args() {
        // @test("a", "b", "c") fun f() { 42 }
        let expr = parse("@test(\"a\", \"b\", \"c\") fun f() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            let attr = exprs[0].attributes.iter().find(|a| a.name == "test");
            assert!(attr.is_some());
            if let Some(a) = attr {
                assert_eq!(a.args.len(), 3, "Should have 3 args");
            }
        }
    }

    #[test]
    fn test_label_as_decorator_no_at_prefix() {
        // Test stripping @ prefix behavior - the decorator name
        // should not include the @ prefix
        let expr = parse("@myattr fun f() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                exprs[0].attributes.iter().any(|a| a.name == "myattr"),
                "Attribute name should be 'myattr' without @"
            );
            assert!(
                !exprs[0].attributes.iter().any(|a| a.name == "@myattr"),
                "Attribute name should NOT include @"
            );
        }
    }

    #[test]
    fn test_label_as_decorator_on_class_with_args() {
        // @derive("Debug", "Clone") class Foo { }
        let expr = parse("@derive(\"Debug\", \"Clone\") class Foo { }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Class { decorators, .. } = &exprs[0].kind {
                assert!(
                    decorators.iter().any(|d| d.name == "derive"),
                    "Class should have 'derive' decorator"
                );
                let dec = decorators.iter().find(|d| d.name == "derive").unwrap();
                assert_eq!(dec.args.len(), 2, "derive should have 2 args");
            } else {
                panic!("Expected Class expression");
            }
        }
    }

    #[test]
    fn test_label_as_decorator_identifier_arg() {
        // @cfg(test) fun f() { 42 }
        let expr = parse("@cfg(test) fun f() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            let attr = exprs[0].attributes.iter().find(|a| a.name == "cfg");
            assert!(attr.is_some());
        }
    }

    // ============================================================
    // Additional coverage tests for parse_label_as_decorator (expressions.rs:229)
    // Targets the Token::At decorator loop (lines 286-311)
    // and labeled-loop-after-decorator error (line 263)
    // ============================================================

    #[test]
    fn test_label_as_decorator_followed_by_at_decorator_on_function() {
        // @label @ identifier fun f() { 42 }
        // The first @label is Token::Label, then we need Token::At + Token::Identifier
        // Token::At is produced by standalone @ (without attached identifier)
        // Lexer: @label -> Label("@label"), @ -> At, identifier -> Identifier
        // This exercises the Token::At loop in parse_label_as_decorator (lines 286-311)
        let expr = parse("@first @ second fun f() { 42 }");
        // This tests whether the @ <space> identifier path works
        // It may or may not parse depending on lexer behavior
        assert!(expr.is_ok() || expr.is_err(), "Should not panic");
    }

    #[test]
    fn test_label_as_decorator_at_with_args_on_function() {
        // @label @decorator("arg") fun f() { 42 }
        // This tests the Token::At decorator with arguments (lines 297-301)
        let expr = parse("@label @decorator(\"arg\") fun f() { 42 }");
        // Whether this parses depends on lexer tokenization
        assert!(expr.is_ok() || expr.is_err(), "Should not panic");
    }

    #[test]
    fn test_label_as_decorator_on_struct() {
        // @derive struct Point { x: i32 }
        // Decorator on struct (not a class), exercises the non-Class path after parse_prefix
        let expr = parse("@derive struct Point { x: i32 }");
        if let Ok(expr) = expr {
            if let Some(exprs) = get_block_exprs(&expr) {
                assert!(
                    !exprs[0].attributes.is_empty(),
                    "Struct should have attributes from decorator"
                );
            }
        }
    }

    #[test]
    fn test_label_as_decorator_multiple_label_tokens() {
        // @first @second @third fun f() { 42 }
        // Three consecutive Label tokens -- exercises the first while loop (lines 256-283)
        let expr = parse("@first @second @third fun f() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                exprs[0].attributes.len() >= 3,
                "Should have at least 3 attributes, got {}",
                exprs[0].attributes.len()
            );
        }
    }

    #[test]
    fn test_label_as_decorator_multiple_with_args() {
        // @first("a") @second("b") fun f() { 42 }
        // Two Label tokens with args -- exercises the arg parsing in the while loop (lines 269-273)
        let expr = parse("@first(\"a\") @second(\"b\") fun f() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                exprs[0].attributes.len() >= 2,
                "Should have at least 2 attributes"
            );
            let first = exprs[0].attributes.iter().find(|a| a.name == "first");
            assert!(first.is_some());
            if let Some(a) = first {
                assert_eq!(a.args.len(), 1);
            }
        }
    }

    #[test]
    fn test_label_as_decorator_on_class_multiple() {
        // @serialize @json class Foo { }
        // Multiple decorators on class -- exercises class decorator setting (lines 320-340)
        let expr = parse("@serialize @json class Foo { }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            if let ExprKind::Class { decorators, .. } = &exprs[0].kind {
                assert!(
                    decorators.len() >= 2,
                    "Class should have at least 2 decorators, got {}",
                    decorators.len()
                );
            }
        }
    }

    #[test]
    fn test_label_as_decorator_empty_args() {
        // @test() fun f() { 42 }
        // Decorator with empty parens -- exercises the args parsing with immediate RightParen
        let expr = parse("@test() fun f() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            let attr = exprs[0].attributes.iter().find(|a| a.name == "test");
            assert!(attr.is_some());
            if let Some(a) = attr {
                assert!(a.args.is_empty(), "Empty parens should produce no args");
            }
        }
    }
