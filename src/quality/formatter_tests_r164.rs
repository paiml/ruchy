    use super::*;
    use crate::frontend::ast::*;

    fn create_simple_literal(value: i64) -> Expr {
        Expr::new(
            ExprKind::Literal(Literal::Integer(value, None)),
            Default::default(),
        )
    }

    fn create_identifier(name: &str) -> Expr {
        Expr::new(ExprKind::Identifier(name.to_string()), Default::default())
    }

    fn create_bool_literal(b: bool) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Bool(b)), Default::default())
    }

    // Test 65: Format while loop
    #[test]
    fn test_format_while_loop_r164() {
        let formatter = Formatter::new();
        let condition = create_bool_literal(true);
        let body = Expr::new(
            ExprKind::Block(vec![create_simple_literal(1)]),
            Default::default(),
        );
        let expr = Expr::new(
            ExprKind::While {
                condition: Box::new(condition),
                body: Box::new(body),
                label: None,
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("while"));
    }

    // Test 66: Format for loop
    #[test]
    fn test_format_for_loop_r164() {
        let formatter = Formatter::new();
        let iter = create_identifier("items");
        let body = Expr::new(
            ExprKind::Block(vec![create_simple_literal(1)]),
            Default::default(),
        );
        let expr = Expr::new(
            ExprKind::For {
                var: "x".to_string(),
                pattern: Some(Pattern::Identifier("x".to_string())),
                iter: Box::new(iter),
                body: Box::new(body),
                label: None,
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("for"));
        assert!(result.contains("in"));
    }

    // Test 67: Format method call
    #[test]
    fn test_format_method_call_r164() {
        let formatter = Formatter::new();
        let receiver = create_identifier("obj");
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(receiver),
                method: "get".to_string(),
                args: vec![create_simple_literal(0)],
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("obj"));
        assert!(result.contains("get"));
    }

    // Test 68: Format lambda expression
    #[test]
    fn test_format_lambda_r164() {
        let formatter = Formatter::new();
        let body = create_simple_literal(42);
        let param = Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type {
                kind: TypeKind::Named("Any".to_string()),
                span: Default::default(),
            },
            span: Default::default(),
            is_mutable: false,
            default_value: None,
        };
        let expr = Expr::new(
            ExprKind::Lambda {
                params: vec![param],
                body: Box::new(body),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("|"));
        assert!(result.contains("42"));
    }

    // Test 69: Format ternary expression
    #[test]
    fn test_format_ternary_r164() {
        let formatter = Formatter::new();
        let condition = create_bool_literal(true);
        let true_expr = create_simple_literal(1);
        let false_expr = create_simple_literal(2);
        let expr = Expr::new(
            ExprKind::Ternary {
                condition: Box::new(condition),
                true_expr: Box::new(true_expr),
                false_expr: Box::new(false_expr),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("?"));
        assert!(result.contains(":"));
    }

    // Test 70: Format assign expression
    #[test]
    fn test_format_assign_r164() {
        let formatter = Formatter::new();
        let target = create_identifier("x");
        let value = create_simple_literal(42);
        let expr = Expr::new(
            ExprKind::Assign {
                target: Box::new(target),
                value: Box::new(value),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("x"));
        assert!(result.contains("="));
        assert!(result.contains("42"));
    }

    // Test 71: Format compound assign
    #[test]
    fn test_format_compound_assign_r164() {
        let formatter = Formatter::new();
        let target = create_identifier("x");
        let value = create_simple_literal(1);
        let expr = Expr::new(
            ExprKind::CompoundAssign {
                target: Box::new(target),
                op: BinaryOp::Add,
                value: Box::new(value),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("+=") || (result.contains("x") && result.contains("1")));
    }

    // Test 72: Format return without value
    #[test]
    fn test_format_return_no_value_r164() {
        let formatter = Formatter::new();
        let expr = Expr::new(ExprKind::Return { value: None }, Default::default());
        let result = formatter.format(&expr).expect("should format");
        assert_eq!(result, "return");
    }

    // Test 73: Format break with value
    #[test]
    fn test_format_break_with_value_r164() {
        let formatter = Formatter::new();
        let value = create_simple_literal(42);
        let expr = Expr::new(
            ExprKind::Break {
                label: None,
                value: Some(Box::new(value)),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("break"));
        assert!(result.contains("42"));
    }

    // Test 74: Format list literal
    #[test]
    fn test_format_list_literal_r164() {
        let formatter = Formatter::new();
        let items = vec![
            create_simple_literal(1),
            create_simple_literal(2),
            create_simple_literal(3),
        ];
        let expr = Expr::new(ExprKind::List(items), Default::default());
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("["));
        assert!(result.contains("]"));
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }

    // Test 75: Format tuple literal
    #[test]
    fn test_format_tuple_literal_direct_r164() {
        let formatter = Formatter::new();
        let items = vec![create_simple_literal(1), create_simple_literal(2)];
        let expr = Expr::new(ExprKind::Tuple(items), Default::default());
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("("));
        assert!(result.contains(")"));
    }

    // Test 76: Format range exclusive
    #[test]
    fn test_format_range_exclusive_r164() {
        let formatter = Formatter::new();
        let start = create_simple_literal(0);
        let end = create_simple_literal(10);
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
        assert!(!result.contains("..="));
    }

    // Test 77: Format range inclusive
    #[test]
    fn test_format_range_inclusive_r164() {
        let formatter = Formatter::new();
        let start = create_simple_literal(0);
        let end = create_simple_literal(10);
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

    // Test 78: Format throw expression
    #[test]
    fn test_format_throw_r164() {
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

    // Test 79: Format await expression
    #[test]
    fn test_format_await_r164() {
        let formatter = Formatter::new();
        let inner = create_identifier("future");
        let expr = Expr::new(
            ExprKind::Await {
                expr: Box::new(inner),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("await"));
        assert!(result.contains("future"));
    }

    // Test 80: Format async block
    #[test]
    fn test_format_async_block_r164() {
        let formatter = Formatter::new();
        let body = create_simple_literal(42);
        let expr = Expr::new(
            ExprKind::AsyncBlock {
                body: Box::new(body),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("async"));
    }

    // Test 81: Format Ok variant
    #[test]
    fn test_format_ok_variant_r164() {
        let formatter = Formatter::new();
        let value = create_simple_literal(42);
        let expr = Expr::new(
            ExprKind::Ok {
                value: Box::new(value),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("Ok"));
        assert!(result.contains("42"));
    }

    // Test 82: Format Err variant
    #[test]
    fn test_format_err_variant_r164() {
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

    // Test 83: Format Some variant
    #[test]
    fn test_format_some_variant_r164() {
        let formatter = Formatter::new();
        let value = create_simple_literal(42);
        let expr = Expr::new(
            ExprKind::Some {
                value: Box::new(value),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("Some"));
    }

    // Test 84: Format None variant
    #[test]
    fn test_format_none_variant_r164() {
        let formatter = Formatter::new();
        let expr = Expr::new(ExprKind::None, Default::default());
        let result = formatter.format(&expr).expect("should format");
        assert_eq!(result, "None");
    }

    // Test 85: Format try expression
    #[test]
    fn test_format_try_expr_r164() {
        let formatter = Formatter::new();
        let inner = create_identifier("result");
        let expr = Expr::new(
            ExprKind::Try {
                expr: Box::new(inner),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("?"));
    }

    // Test 86: Format spawn expression
    #[test]
    fn test_format_spawn_r164() {
        let formatter = Formatter::new();
        let actor = create_identifier("my_actor");
        let expr = Expr::new(
            ExprKind::Spawn {
                actor: Box::new(actor),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("spawn"));
    }

    // Test 87: Format optional field access
    #[test]
    fn test_format_optional_field_access_r164() {
        let formatter = Formatter::new();
        let obj = create_identifier("maybe_obj");
        let expr = Expr::new(
            ExprKind::OptionalFieldAccess {
                object: Box::new(obj),
                field: "value".to_string(),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("?."));
    }

    // Test 88: Format type cast
    #[test]
    fn test_format_type_cast_r164() {
        let formatter = Formatter::new();
        let value = create_simple_literal(42);
        let expr = Expr::new(
            ExprKind::TypeCast {
                expr: Box::new(value),
                target_type: "f64".to_string(),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("as"));
        assert!(result.contains("f64"));
    }

    // Test 89: Format array init
    #[test]
    fn test_format_array_init_r164() {
        let formatter = Formatter::new();
        let value = create_simple_literal(0);
        let size = create_simple_literal(10);
        let expr = Expr::new(
            ExprKind::ArrayInit {
                value: Box::new(value),
                size: Box::new(size),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("["));
        assert!(result.contains(";"));
        assert!(result.contains("]"));
    }

    // Test 90: Format qualified name
    #[test]
    fn test_format_qualified_name_r164() {
        let formatter = Formatter::new();
        let expr = Expr::new(
            ExprKind::QualifiedName {
                module: "std".to_string(),
                name: "println".to_string(),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("std"));
        assert!(result.contains("::"));
        assert!(result.contains("println"));
    }

    // Test 91: Format spread
    #[test]
    fn test_format_spread_r164() {
        let formatter = Formatter::new();
        let inner = create_identifier("arr");
        let expr = Expr::new(
            ExprKind::Spread {
                expr: Box::new(inner),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("..."));
    }

    // Test 92: Format pre-increment
    #[test]
    fn test_format_pre_increment_r164() {
        let formatter = Formatter::new();
        let target = create_identifier("x");
        let expr = Expr::new(
            ExprKind::PreIncrement {
                target: Box::new(target),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("++"));
    }

    // Test 93: Format post-increment
    #[test]
    fn test_format_post_increment_r164() {
        let formatter = Formatter::new();
        let target = create_identifier("x");
        let expr = Expr::new(
            ExprKind::PostIncrement {
                target: Box::new(target),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("++"));
    }

    // Test 94: Format pre-decrement
    #[test]
    fn test_format_pre_decrement_r164() {
        let formatter = Formatter::new();
        let target = create_identifier("x");
        let expr = Expr::new(
            ExprKind::PreDecrement {
                target: Box::new(target),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("--"));
    }

    // Test 95: Format post-decrement
    #[test]
    fn test_format_post_decrement_r164() {
        let formatter = Formatter::new();
        let target = create_identifier("x");
        let expr = Expr::new(
            ExprKind::PostDecrement {
                target: Box::new(target),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("--"));
    }

    // Test 96: Format import
    #[test]
    fn test_format_import_r164() {
        let formatter = Formatter::new();
        let expr = Expr::new(
            ExprKind::Import {
                module: "std::io".to_string(),
                items: Some(vec!["read".to_string(), "write".to_string()]),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("import"));
    }

    // Test 97: Format module declaration
    #[test]
    fn test_format_module_declaration_r164() {
        let formatter = Formatter::new();
        let expr = Expr::new(
            ExprKind::ModuleDeclaration {
                name: "utils".to_string(),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("mod"));
        assert!(result.contains("utils"));
    }

    // Test 98: Format export
    #[test]
    fn test_format_export_r164() {
        let formatter = Formatter::new();
        let inner = create_identifier("my_fn");
        let expr = Expr::new(
            ExprKind::Export {
                expr: Box::new(inner),
                is_default: false,
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("export"));
    }

    // Test 99: Format export default
    #[test]
    fn test_format_export_default_r164() {
        let formatter = Formatter::new();
        let inner = create_identifier("main");
        let expr = Expr::new(
            ExprKind::Export {
                expr: Box::new(inner),
                is_default: true,
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("export"));
        assert!(result.contains("default"));
    }

    // Test 100: Format loop
    #[test]
    fn test_format_loop_r164() {
        let formatter = Formatter::new();
        let body = create_simple_literal(1);
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

    // Test 101: Format binary subtraction
    #[test]
    fn test_format_binary_sub_r164() {
        let formatter = Formatter::new();
        let left = create_simple_literal(10);
        let right = create_simple_literal(5);
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Subtract,
                right: Box::new(right),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("-"));
    }

    // Test 102: Format binary multiply
    #[test]
    fn test_format_binary_mul_r164() {
        let formatter = Formatter::new();
        let left = create_simple_literal(3);
        let right = create_simple_literal(4);
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Multiply,
                right: Box::new(right),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("*"));
    }

    // Test 103: Format binary divide
    #[test]
    fn test_format_binary_div_r164() {
        let formatter = Formatter::new();
        let left = create_simple_literal(10);
        let right = create_simple_literal(2);
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Divide,
                right: Box::new(right),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("/"));
    }

    // Test 104: Format binary modulo
    #[test]
    fn test_format_binary_mod_r164() {
        let formatter = Formatter::new();
        let left = create_simple_literal(10);
        let right = create_simple_literal(3);
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Modulo,
                right: Box::new(right),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("%"));
    }

    // Test 105: Format binary equality
    #[test]
    fn test_format_binary_eq_r164() {
        let formatter = Formatter::new();
        let left = create_simple_literal(1);
        let right = create_simple_literal(1);
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Equal,
                right: Box::new(right),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("=="));
    }

    // Test 106: Format binary not equal
    #[test]
    fn test_format_binary_ne_r164() {
        let formatter = Formatter::new();
        let left = create_simple_literal(1);
        let right = create_simple_literal(2);
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::NotEqual,
                right: Box::new(right),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("!="));
    }

    // Test 107: Format binary less than
    #[test]
    fn test_format_binary_lt_r164() {
        let formatter = Formatter::new();
        let left = create_simple_literal(1);
        let right = create_simple_literal(2);
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Less,
                right: Box::new(right),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("<"));
    }

    // Test 108: Format binary greater than
    #[test]
    fn test_format_binary_gt_r164() {
        let formatter = Formatter::new();
        let left = create_simple_literal(2);
        let right = create_simple_literal(1);
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Greater,
                right: Box::new(right),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains(">"));
    }

    // Test 109: Format binary and
    #[test]
    fn test_format_binary_and_r164() {
        let formatter = Formatter::new();
        let left = create_bool_literal(true);
        let right = create_bool_literal(false);
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("&&"));
    }

    // Test 110: Format binary or
    #[test]
    fn test_format_binary_or_r164() {
        let formatter = Formatter::new();
        let left = create_bool_literal(true);
        let right = create_bool_literal(false);
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Or,
                right: Box::new(right),
            },
            Default::default(),
        );
        let result = formatter.format(&expr).expect("should format");
        assert!(result.contains("||"));
    }
