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
