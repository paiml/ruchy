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
    // Literal coverage: Char, Byte, Unit, Null, Atom
    // ============================================================================

    #[test]
    fn test_format_expr_literal_char() {
        let f = make_formatter();
        let expr = Expr::new(ExprKind::Literal(Literal::Char('A')), span());
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "'A'");
    }

    #[test]
    fn test_format_expr_literal_byte() {
        let f = make_formatter();
        let expr = Expr::new(ExprKind::Literal(Literal::Byte(b'x')), span());
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "b'x'");
    }

    #[test]
    fn test_format_expr_literal_unit() {
        let f = make_formatter();
        let expr = Expr::new(ExprKind::Literal(Literal::Unit), span());
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "()");
    }

    #[test]
    fn test_format_expr_literal_null() {
        let f = make_formatter();
        let expr = Expr::new(ExprKind::Literal(Literal::Null), span());
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "null");
    }

    #[test]
    fn test_format_expr_literal_atom() {
        let f = make_formatter();
        let expr = Expr::new(ExprKind::Literal(Literal::Atom("ok".to_string())), span());
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, ":ok");
    }

    #[test]
    fn test_format_expr_string_with_quotes() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Literal(Literal::String("say \"hi\"".to_string())),
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, r#""say \"hi\"""#);
    }

    // ============================================================================
    // MethodCall
    // ============================================================================

    #[test]
    fn test_format_expr_method_call() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(ident_expr("arr")),
                method: "push".to_string(),
                args: vec![int_expr(42)],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "arr.push(42)");
    }

    #[test]
    fn test_format_expr_method_call_no_args() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(ident_expr("vec")),
                method: "len".to_string(),
                args: vec![],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "vec.len()");
    }

    #[test]
    fn test_format_expr_method_call_multiple_args() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(ident_expr("s")),
                method: "replace".to_string(),
                args: vec![str_expr("a"), str_expr("b")],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, r#"s.replace("a", "b")"#);
    }

    // ============================================================================
    // IndexAccess
    // ============================================================================

    #[test]
    fn test_format_expr_index_access() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(ident_expr("arr")),
                index: Box::new(int_expr(0)),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "arr[0]");
    }

    // ============================================================================
    // Assign
    // ============================================================================

    #[test]
    fn test_format_expr_assign() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Assign {
                target: Box::new(ident_expr("x")),
                value: Box::new(int_expr(42)),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "x = 42");
    }

    // ============================================================================
    // Return
    // ============================================================================

    #[test]
    fn test_format_expr_return_value() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Return {
                value: Some(Box::new(int_expr(42))),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "return 42");
    }

    #[test]
    fn test_format_expr_return_void() {
        let f = make_formatter();
        let expr = Expr::new(ExprKind::Return { value: None }, span());
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "return");
    }

    // ============================================================================
    // FieldAccess
    // ============================================================================

    #[test]
    fn test_format_expr_field_access() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::FieldAccess {
                object: Box::new(ident_expr("point")),
                field: "x".to_string(),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "point.x");
    }

    // ============================================================================
    // While
    // ============================================================================

    #[test]
    fn test_format_expr_while() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::While {
                condition: Box::new(bool_expr(true)),
                body: Box::new(Expr::new(
                    ExprKind::Block(vec![int_expr(1)]),
                    span(),
                )),
                label: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("while true"));
    }

    // ============================================================================
    // Break, Continue
    // ============================================================================

    #[test]
    fn test_format_expr_break_value() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Break {
                value: Some(Box::new(int_expr(42))),
                label: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "break 42");
    }

    #[test]
    fn test_format_expr_break_no_value() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Break {
                value: None,
                label: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "break");
    }

    #[test]
    fn test_format_expr_continue() {
        let f = make_formatter();
        let expr = Expr::new(ExprKind::Continue { label: None }, span());
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "continue");
    }

    // ============================================================================
    // Range
    // ============================================================================

    #[test]
    fn test_format_expr_range_exclusive() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Range {
                start: Box::new(int_expr(0)),
                end: Box::new(int_expr(10)),
                inclusive: false,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "0..10");
    }

    #[test]
    fn test_format_expr_range_inclusive() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Range {
                start: Box::new(int_expr(0)),
                end: Box::new(int_expr(10)),
                inclusive: true,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "0..=10");
    }

    // ============================================================================
    // Module, ModuleDeclaration
    // ============================================================================

    #[test]
    fn test_format_expr_module() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Module {
                name: "mymod".to_string(),
                body: Box::new(Expr::new(
                    ExprKind::Block(vec![int_expr(1)]),
                    span(),
                )),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("mod mymod"));
    }

    #[test]
    fn test_format_expr_module_declaration() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::ModuleDeclaration {
                name: "utils".to_string(),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "mod utils;");
    }

    // ============================================================================
    // Import, ImportAll, ImportDefault
    // ============================================================================

    #[test]
    fn test_format_expr_import_with_items() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Import {
                module: "std::io".to_string(),
                items: Some(vec!["Read".to_string(), "Write".to_string()]),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "import std::io::{Read, Write}");
    }

    #[test]
    fn test_format_expr_import_no_items() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Import {
                module: "std::fs".to_string(),
                items: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "import std::fs");
    }

    #[test]
    fn test_format_expr_import_all() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::ImportAll {
                module: "std::collections".to_string(),
                alias: String::new(),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "import std::collections::*");
    }

    #[test]
    fn test_format_expr_import_default() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::ImportDefault {
                module: "mylib".to_string(),
                name: "MyClass".to_string(),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "import default MyClass from mylib");
    }

    // ============================================================================
    // Export, ExportList, ExportDefault
    // ============================================================================

    #[test]
    fn test_format_expr_export() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Export {
                expr: Box::new(ident_expr("foo")),
                is_default: false,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "export foo");
    }

    #[test]
    fn test_format_expr_export_default() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Export {
                expr: Box::new(ident_expr("bar")),
                is_default: true,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "export default bar");
    }

    #[test]
    fn test_format_expr_export_list() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::ExportList {
                names: vec!["foo".to_string(), "bar".to_string()],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "export { foo, bar }");
    }

    #[test]
    fn test_format_expr_export_default_variant() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::ExportDefault {
                expr: Box::new(ident_expr("MyClass")),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "export default MyClass");
    }

    // ============================================================================
    // LetPattern
    // ============================================================================

    #[test]
    fn test_format_expr_let_pattern() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::LetPattern {
                pattern: Pattern::Tuple(vec![
                    Pattern::Identifier("a".to_string()),
                    Pattern::Identifier("b".to_string()),
                ]),
                type_annotation: None,
                value: Box::new(ident_expr("pair")),
                body: Box::new(ident_expr("a")),
                is_mutable: false,
                else_block: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "let (a, b) = pair in a");
    }

    // ============================================================================
    // WhileLet
    // ============================================================================

    #[test]
    fn test_format_expr_while_let() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::WhileLet {
                pattern: Pattern::Some(Box::new(Pattern::Identifier("v".to_string()))),
                expr: Box::new(ident_expr("iter")),
                body: Box::new(Expr::new(ExprKind::Block(vec![]), span())),
                label: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("while let Some(v) = iter"));
    }

    // ============================================================================
    // StringInterpolation
    // ============================================================================

    #[test]
    fn test_format_expr_string_interpolation_text_only() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::StringInterpolation {
                parts: vec![StringPart::Text("hello world".to_string())],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "f\"hello world\"");
    }

    #[test]
    fn test_format_expr_string_interpolation_with_expr() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::StringInterpolation {
                parts: vec![
                    StringPart::Text("x = ".to_string()),
                    StringPart::Expr(Box::new(ident_expr("x"))),
                ],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "f\"x = {x}\"");
    }

    #[test]
    fn test_format_expr_string_interpolation_with_format_spec() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::StringInterpolation {
                parts: vec![
                    StringPart::Text("pi = ".to_string()),
                    StringPart::ExprWithFormat {
                        expr: Box::new(ident_expr("pi")),
                        format_spec: ".2".to_string(),
                    },
                ],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "f\"pi = {pi:.2}\"");
    }

    // ============================================================================
    // Send
    // ============================================================================

    #[test]
    fn test_format_expr_send() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Send {
                actor: Box::new(ident_expr("worker")),
                message: Box::new(str_expr("ping")),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "send(worker, \"ping\")");
    }

    // ============================================================================
    // Loop
    // ============================================================================

    #[test]
    fn test_format_expr_loop() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Loop {
                body: Box::new(Expr::new(
                    ExprKind::Break {
                        value: None,
                        label: None,
                    },
                    span(),
                )),
                label: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("loop {"));
        assert!(result.contains("break"));
    }

    // ============================================================================
    // Pipeline
    // ============================================================================

    #[test]
    fn test_format_expr_pipeline() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Pipeline {
                expr: Box::new(ident_expr("data")),
                stages: vec![
                    PipelineStage {
                        op: Box::new(ident_expr("filter")),
                        span: span(),
                    },
                    PipelineStage {
                        op: Box::new(ident_expr("map")),
                        span: span(),
                    },
                ],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "data |> filter |> map");
    }

    // ============================================================================
    // Pre/Post Increment/Decrement
    // ============================================================================

    #[test]
    fn test_format_expr_pre_increment() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::PreIncrement {
                target: Box::new(ident_expr("i")),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "++i");
    }

    #[test]
    fn test_format_expr_post_increment() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::PostIncrement {
                target: Box::new(ident_expr("i")),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "i++");
    }

    #[test]
    fn test_format_expr_pre_decrement() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::PreDecrement {
                target: Box::new(ident_expr("i")),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "--i");
    }

    #[test]
    fn test_format_expr_post_decrement() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::PostDecrement {
                target: Box::new(ident_expr("i")),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "i--");
    }

    // ============================================================================
    // ActorSend, ActorQuery, Ask
    // ============================================================================

    #[test]
    fn test_format_expr_actor_send() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::ActorSend {
                actor: Box::new(ident_expr("worker")),
                message: Box::new(str_expr("task")),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "worker <- \"task\"");
    }

    #[test]
    fn test_format_expr_actor_query() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::ActorQuery {
                actor: Box::new(ident_expr("db")),
                message: Box::new(str_expr("status")),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "db <? \"status\"");
    }

    #[test]
    fn test_format_expr_ask() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Ask {
                actor: Box::new(ident_expr("server")),
                message: Box::new(str_expr("ping")),
                timeout: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "ask server \"ping\"");
    }

    // ============================================================================
    // ListComprehension
    // ============================================================================

    #[test]
    fn test_format_expr_list_comprehension() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::ListComprehension {
                element: Box::new(ident_expr("x")),
                clauses: vec![ComprehensionClause {
                    variable: "x".to_string(),
                    iterable: Box::new(ident_expr("items")),
                    condition: None,
                }],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "[x for x in items]");
    }

    #[test]
    fn test_format_expr_list_comprehension_with_condition() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::ListComprehension {
                element: Box::new(ident_expr("x")),
                clauses: vec![ComprehensionClause {
                    variable: "x".to_string(),
                    iterable: Box::new(ident_expr("items")),
                    condition: Some(Box::new(Expr::new(
                        ExprKind::Binary {
                            left: Box::new(ident_expr("x")),
                            op: BinaryOp::Greater,
                            right: Box::new(int_expr(0)),
                        },
                        span(),
                    ))),
                }],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "[x for x in items if x > 0]");
    }

    // ============================================================================
    // DictComprehension
    // ============================================================================

    #[test]
    fn test_format_expr_dict_comprehension() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::DictComprehension {
                key: Box::new(ident_expr("k")),
                value: Box::new(ident_expr("v")),
                clauses: vec![ComprehensionClause {
                    variable: "kv".to_string(),
                    iterable: Box::new(ident_expr("pairs")),
                    condition: None,
                }],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "{k: v for kv in pairs}");
    }

    // ============================================================================
    // SetComprehension
    // ============================================================================

    #[test]
    fn test_format_expr_set_comprehension() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::SetComprehension {
                element: Box::new(ident_expr("x")),
                clauses: vec![ComprehensionClause {
                    variable: "x".to_string(),
                    iterable: Box::new(ident_expr("items")),
                    condition: None,
                }],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "{x for x in items}");
    }

    // ============================================================================
    // Command
    // ============================================================================

    #[test]
    fn test_format_expr_command_no_args() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Command {
                program: "ls".to_string(),
                args: vec![],
                env: vec![],
                working_dir: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "`ls`");
    }

    #[test]
    fn test_format_expr_command_with_args() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Command {
                program: "git".to_string(),
                args: vec!["status".to_string(), "-s".to_string()],
                env: vec![],
                working_dir: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "`git status -s`");
    }

    // ============================================================================
    // QualifiedName
    // ============================================================================

    #[test]
    fn test_format_expr_qualified_name() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::QualifiedName {
                module: "std".to_string(),
                name: "io".to_string(),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "std::io");
    }

    // ============================================================================
    // TypeAlias
    // ============================================================================

    #[test]
    fn test_format_expr_type_alias() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::TypeAlias {
                name: "MyInt".to_string(),
                target_type: Type {
                    kind: TypeKind::Named("i64".to_string()),
                    span: span(),
                },
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "type MyInt = i64");
    }

    // ============================================================================
    // Spread
    // ============================================================================

    #[test]
    fn test_format_expr_spread() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Spread {
                expr: Box::new(ident_expr("args")),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "...args");
    }

    // ============================================================================
    // OptionalMethodCall
    // ============================================================================

    #[test]
    fn test_format_expr_optional_method_call() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::OptionalMethodCall {
                receiver: Box::new(ident_expr("obj")),
                method: "get".to_string(),
                args: vec![str_expr("key")],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "obj?.get(\"key\")");
    }

    // ============================================================================
    // ReExport
    // ============================================================================

    #[test]
    fn test_format_expr_re_export() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::ReExport {
                items: vec!["Foo".to_string(), "Bar".to_string()],
                module: "lib".to_string(),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "export { Foo, Bar } from lib");
    }

    // ============================================================================
    // Macro, MacroInvocation
    // ============================================================================

    #[test]
    fn test_format_expr_macro() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Macro {
                name: "my_macro".to_string(),
                args: vec![ident_expr("x")],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "macro my_macro(x) { }");
    }

    #[test]
    fn test_format_expr_macro_invocation() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::MacroInvocation {
                name: "println".to_string(),
                args: vec![str_expr("hello")],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "println!(\"hello\")");
    }

    // ============================================================================
    // VecRepeat
    // ============================================================================

    #[test]
    fn test_format_expr_vec_repeat() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::VecRepeat {
                value: Box::new(int_expr(0)),
                count: Box::new(int_expr(10)),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "vec![0; 10]");
    }

    // ============================================================================
    // DataFrame
    // ============================================================================

    #[test]
    fn test_format_expr_dataframe() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::DataFrame {
                columns: vec![
                    DataFrameColumn {
                        name: "x".to_string(),
                        values: vec![int_expr(1), int_expr(2)],
                    },
                    DataFrameColumn {
                        name: "y".to_string(),
                        values: vec![int_expr(3), int_expr(4)],
                    },
                ],
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "df![\"x\" => [1, 2], \"y\" => [3, 4]]");
    }

    // ============================================================================
    // DataFrameOperation
    // ============================================================================

    #[test]
    fn test_format_expr_dataframe_operation() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::DataFrameOperation {
                source: Box::new(ident_expr("df")),
                operation: DataFrameOp::Limit(10),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("df."));
        assert!(result.contains("Limit"));
    }

    // ============================================================================
    // Lazy
    // ============================================================================

    #[test]
    fn test_format_expr_lazy() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Lazy {
                expr: Box::new(int_expr(42)),
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "lazy 42");
    }

    // ============================================================================
    // Set (unimplemented)
    // ============================================================================

    #[test]
    fn test_format_expr_set() {
        let f = make_formatter();
        let expr = Expr::new(ExprKind::Set(vec![int_expr(1)]), span());
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("UNIMPLEMENTED"));
    }

    // ============================================================================
    // For expression
    // ============================================================================

    #[test]
    fn test_format_expr_for_with_var() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::For {
                var: "i".to_string(),
                pattern: None,
                iter: Box::new(Expr::new(
                    ExprKind::Range {
                        start: Box::new(int_expr(0)),
                        end: Box::new(int_expr(10)),
                        inclusive: false,
                    },
                    span(),
                )),
                body: Box::new(Expr::new(ExprKind::Block(vec![]), span())),
                label: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("for i in 0..10"));
    }

    #[test]
    fn test_format_expr_for_with_pattern() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::For {
                var: String::new(),
                pattern: Some(Pattern::Identifier("x".to_string())),
                iter: Box::new(ident_expr("items")),
                body: Box::new(Expr::new(ExprKind::Block(vec![]), span())),
                label: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("for x in items"));
    }

    // ============================================================================
    // Let (sequential statement style)
    // ============================================================================

    #[test]
    fn test_format_expr_let_sequential_unit_body() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                value: Box::new(int_expr(42)),
                body: Box::new(Expr::new(ExprKind::Literal(Literal::Unit), span())),
                is_mutable: false,
                type_annotation: None,
                else_block: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert_eq!(result, "let x = 42");
    }

    #[test]
    fn test_format_expr_let_sequential_with_block_body() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                value: Box::new(int_expr(1)),
                body: Box::new(Expr::new(
                    ExprKind::Block(vec![ident_expr("x")]),
                    span(),
                )),
                is_mutable: false,
                type_annotation: None,
                else_block: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.starts_with("let x = 1"));
        assert!(result.contains("x"));
    }

    #[test]
    fn test_format_expr_let_sequential_nested_let_body() {
        let f = make_formatter();
        let inner_let = Expr::new(
            ExprKind::Let {
                name: "y".to_string(),
                value: Box::new(int_expr(2)),
                body: Box::new(Expr::new(ExprKind::Literal(Literal::Unit), span())),
                is_mutable: false,
                type_annotation: None,
                else_block: None,
            },
            span(),
        );
        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                value: Box::new(int_expr(1)),
                body: Box::new(inner_let),
                is_mutable: false,
                type_annotation: None,
                else_block: None,
            },
            span(),
        );
        let result = f.format_expr(&expr, 0);
        assert!(result.contains("let x = 1"));
        assert!(result.contains("let y = 2"));
    }

    #[test]
    fn test_format_expr_let_in_expression() {
        let f = make_formatter();
        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                value: Box::new(int_expr(10)),
                body: Box::new(Expr::new(
                    ExprKind::Binary {
                        left: Box::new(ident_expr("x")),
                        op: BinaryOp::Add,
                        right: Box::new(int_expr(1)),
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
        assert_eq!(result, "let x = 10 in x + 1");
    }

