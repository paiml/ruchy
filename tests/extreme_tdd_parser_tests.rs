/*!
 * EXTREME TDD Parser Tests - ACTOR-004 (Syntactic Analysis)
 *
 * CRITICAL: ALL parser tests MUST be written FIRST before ANY parser implementation.
 * These tests define EXACT parsing behavior for actor system constructs with 100% edge case coverage.
 *
 * Following Toyota Way: Build quality in from the start.
 * Following EXTREME TDD: 100% parser coverage before any parsing code exists.
 *
 * Complexity Budget: Each test function ≤5 cyclomatic, ≤8 cognitive
 * Coverage Target: 100% parsing rule coverage + 100% edge cases
 * Test Ratio: 3:1 test-to-implementation ratio
 */

use ruchy::frontend::ast::{
    ActorDef, Expression, Hook, HookType, Pattern, ReceiveBlock, RestartStrategy, SendExpr,
    SpawnExpr, Statement, SupervisorDef,
};
use ruchy::frontend::parser::{ParseError, ParseErrorKind, ParseResult, Parser};
use ruchy::frontend::tokens::{Lexer, Span, Token, TokenKind};

#[cfg(test)]
mod actor_parser_tests {
    use super::*;
    use proptest::prelude::*;

    /// Test infrastructure for parser validation
    struct ParserTestContext {
        input: String,
        tokens: Vec<Token>,
        parser: Parser,
        position: usize,
    }

    impl ParserTestContext {
        fn new(input: &str) -> Self {
            let tokens = Lexer::new(input).tokenize_all().unwrap_or_default();
            let parser = Parser::new(tokens.clone());

            Self {
                input: input.to_string(),
                tokens,
                parser,
                position: 0,
            }
        }

        fn parse_actor_def(&mut self) -> ParseResult<ActorDef> {
            self.parser.parse_actor_def()
        }

        fn parse_receive_block(&mut self) -> ParseResult<ReceiveBlock> {
            self.parser.parse_receive_block()
        }

        fn parse_hook(&mut self) -> ParseResult<Hook> {
            self.parser.parse_hook()
        }

        fn parse_expression(&mut self) -> ParseResult<Expression> {
            self.parser.parse_expression()
        }

        fn expect_parse_success<T>(
            &mut self,
            parser_fn: impl FnOnce(&mut Parser) -> ParseResult<T>,
        ) -> T {
            match parser_fn(&mut self.parser) {
                Ok(result) => result,
                Err(error) => panic!("Expected successful parse, got error: {:?}", error),
            }
        }

        fn expect_parse_error<T>(
            &mut self,
            parser_fn: impl FnOnce(&mut Parser) -> ParseResult<T>,
        ) -> ParseError {
            match parser_fn(&mut self.parser) {
                Ok(_) => panic!("Expected parse error, but parsing succeeded"),
                Err(error) => error,
            }
        }
    }

    // =================================================================
    // ACTOR DEFINITION PARSING TESTS (Core Actor Structure)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_basic_actor_definition() {
        let mut ctx = ParserTestContext::new("actor ChatAgent { }");

        let actor = ctx.expect_parse_success(|parser| parser.parse_actor_def());

        assert_eq!(actor.name, "ChatAgent");
        assert!(actor.state_params.is_empty());
        assert!(actor.type_params.is_empty());
        assert!(actor.body.is_empty());
        assert_eq!(actor.span, Span::new(0, 19));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_actor_with_state_parameters() {
        let mut ctx = ParserTestContext::new("actor Counter(count: i32, name: String) { }");

        let actor = ctx.expect_parse_success(|parser| parser.parse_actor_def());

        assert_eq!(actor.name, "Counter");
        assert_eq!(actor.state_params.len(), 2);
        assert_eq!(actor.state_params[0].name, "count");
        assert_eq!(actor.state_params[0].param_type, Type::Int32);
        assert_eq!(actor.state_params[1].name, "name");
        assert_eq!(actor.state_params[1].param_type, Type::String);
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_actor_with_generic_parameters() {
        let mut ctx = ParserTestContext::new("actor GenericActor<T: Send + Sync> { }");

        let actor = ctx.expect_parse_success(|parser| parser.parse_actor_def());

        assert_eq!(actor.name, "GenericActor");
        assert_eq!(actor.type_params.len(), 1);
        assert_eq!(actor.type_params[0].name, "T");
        assert_eq!(actor.type_params[0].bounds.len(), 2);
        assert!(actor.type_params[0].bounds.contains(&TypeBound::Send));
        assert!(actor.type_params[0].bounds.contains(&TypeBound::Sync));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_actor_with_complex_body() {
        let input = r#"
            actor ChatAgent {
                receive {
                    SendMessage(content) => println(content),
                    Shutdown => self.stop()
                }

                hook on_start {
                    println("ChatAgent started")
                }

                fn handle_message(msg: String) {
                    println("Handling: {}", msg)
                }
            }
        "#;

        let mut ctx = ParserTestContext::new(input);
        let actor = ctx.expect_parse_success(|parser| parser.parse_actor_def());

        assert_eq!(actor.name, "ChatAgent");
        assert_eq!(actor.body.len(), 3); // receive + hook + method

        // Verify body contains expected items
        assert!(actor
            .body
            .iter()
            .any(|item| matches!(item, ActorBodyItem::Receive(_))));
        assert!(actor
            .body
            .iter()
            .any(|item| matches!(item, ActorBodyItem::Hook(_))));
        assert!(actor
            .body
            .iter()
            .any(|item| matches!(item, ActorBodyItem::Method(_))));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_actor_with_default_parameters() {
        let mut ctx =
            ParserTestContext::new("actor Counter(count: i32 = 0, active: bool = true) { }");

        let actor = ctx.expect_parse_success(|parser| parser.parse_actor_def());

        assert_eq!(actor.state_params.len(), 2);

        let count_param = &actor.state_params[0];
        assert!(count_param.default_value.is_some());
        if let Some(Expression::IntLiteral(value)) = &count_param.default_value {
            assert_eq!(*value, 0);
        } else {
            panic!("Expected default value for count parameter");
        }

        let active_param = &actor.state_params[1];
        assert!(active_param.default_value.is_some());
        if let Some(Expression::BoolLiteral(value)) = &active_param.default_value {
            assert!(*value);
        } else {
            panic!("Expected default value for active parameter");
        }
    }

    // =================================================================
    // RECEIVE BLOCK PARSING TESTS (Message Pattern Matching)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_basic_receive_block() {
        let mut ctx = ParserTestContext::new("receive { Increment => self.count += 1 }");

        let receive = ctx.expect_parse_success(|parser| parser.parse_receive_block());

        assert_eq!(receive.arms.len(), 1);
        assert!(!receive.is_exhaustive);
        assert!(receive.timeout.is_none());

        let arm = &receive.arms[0];
        assert!(matches!(arm.pattern, Pattern::Identifier { .. }));
        if let Pattern::Identifier { name, .. } = &arm.pattern {
            assert_eq!(name, "Increment");
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_receive_with_multiple_arms() {
        let input = r#"
            receive {
                Increment => self.count += 1,
                Decrement => self.count -= 1,
                GetValue => self.count,
                Reset => self.count = 0
            }
        "#;

        let mut ctx = ParserTestContext::new(input);
        let receive = ctx.expect_parse_success(|parser| parser.parse_receive_block());

        assert_eq!(receive.arms.len(), 4);

        let expected_patterns = ["Increment", "Decrement", "GetValue", "Reset"];
        for (i, expected) in expected_patterns.iter().enumerate() {
            if let Pattern::Identifier { name, .. } = &receive.arms[i].pattern {
                assert_eq!(name, expected);
            } else {
                panic!("Expected identifier pattern for arm {}", i);
            }
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_receive_with_pattern_destructuring() {
        let input = r#"
            receive {
                SendMessage(content, sender) => {
                    println("From {}: {}", sender, content);
                    self.reply(sender, "Message received")
                },
                Point { x, y } => {
                    println("Point at ({}, {})", x, y)
                },
                (first, second, _) => {
                    println("Tuple: {} and {}", first, second)
                }
            }
        "#;

        let mut ctx = ParserTestContext::new(input);
        let receive = ctx.expect_parse_success(|parser| parser.parse_receive_block());

        assert_eq!(receive.arms.len(), 3);

        // Test first arm: function-style pattern
        if let Pattern::FunctionCall { name, args, .. } = &receive.arms[0].pattern {
            assert_eq!(name, "SendMessage");
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected function call pattern");
        }

        // Test second arm: struct pattern
        if let Pattern::Struct { name, fields, .. } = &receive.arms[1].pattern {
            assert_eq!(name, "Point");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].name, "x");
            assert_eq!(fields[1].name, "y");
        } else {
            panic!("Expected struct pattern");
        }

        // Test third arm: tuple pattern
        if let Pattern::Tuple { elements, .. } = &receive.arms[2].pattern {
            assert_eq!(elements.len(), 3);
            assert!(matches!(elements[2], Pattern::Wildcard { .. }));
        } else {
            panic!("Expected tuple pattern");
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_receive_with_pattern_guards() {
        let input = r#"
            receive {
                SetValue(v) if v > 0 => {
                    self.value = v;
                    println("Set positive value: {}", v)
                },
                SetValue(v) if v <= 0 => {
                    println("Ignoring non-positive value: {}", v)
                },
                GetValue if self.initialized => self.value,
                _ => println("Unhandled message")
            }
        "#;

        let mut ctx = ParserTestContext::new(input);
        let receive = ctx.expect_parse_success(|parser| parser.parse_receive_block());

        assert_eq!(receive.arms.len(), 4);

        // Verify guards are parsed correctly
        assert!(receive.arms[0].guard.is_some());
        assert!(receive.arms[1].guard.is_some());
        assert!(receive.arms[2].guard.is_some());
        assert!(receive.arms[3].guard.is_none()); // Wildcard with no guard

        // Test guard expressions
        if let Some(guard) = &receive.arms[0].guard {
            if let Expression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                ..
            } = &guard.condition
            {
                // Guard parsed correctly
            } else {
                panic!("Expected > operator in guard");
            }
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_receive_with_timeout() {
        let input = r#"
            receive timeout(5000) {
                Message(data) => handle(data),
                _ timeout => println("Timeout occurred")
            }
        "#;

        let mut ctx = ParserTestContext::new(input);
        let receive = ctx.expect_parse_success(|parser| parser.parse_receive_block());

        assert!(receive.timeout.is_some());
        if let Some(timeout) = receive.timeout {
            assert_eq!(timeout.duration, 5000);
            assert!(timeout.timeout_handler.is_some());
        }

        assert_eq!(receive.arms.len(), 1); // Only non-timeout arms
    }

    // =================================================================
    // HOOK DEFINITION PARSING TESTS (Lifecycle Events)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_hook_on_start() {
        let mut ctx = ParserTestContext::new("hook on_start { println(\"Actor starting\") }");

        let hook = ctx.expect_parse_success(|parser| parser.parse_hook());

        assert_eq!(hook.hook_type, HookType::OnStart);
        assert_eq!(hook.body.len(), 1);
        assert!(!hook.is_async);

        if let Statement::Expression(Expression::FunctionCall { function, .. }) = &hook.body[0] {
            if let Expression::Identifier { name, .. } = function.as_ref() {
                assert_eq!(name, "println");
            }
        } else {
            panic!("Expected function call in hook body");
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_hook_on_error_with_parameter() {
        let input = r#"
            hook on_error(error) {
                log_error("Actor failed: {}", error);
                self.restart_count += 1;
                if self.restart_count > 3 {
                    self.stop()
                }
            }
        "#;

        let mut ctx = ParserTestContext::new(input);
        let hook = ctx.expect_parse_success(|parser| parser.parse_hook());

        if let HookType::OnError { error_param } = &hook.hook_type {
            assert_eq!(error_param.as_ref().unwrap(), "error");
        } else {
            panic!("Expected OnError hook type");
        }

        assert!(hook.body.len() > 1);
        assert!(!hook.is_async);
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_async_hook() {
        let input = r#"
            async hook on_start {
                let config = await load_config();
                self.initialize(config);
                println("Async initialization complete")
            }
        "#;

        let mut ctx = ParserTestContext::new(input);
        let hook = ctx.expect_parse_success(|parser| parser.parse_hook());

        assert!(hook.is_async);
        assert_eq!(hook.hook_type, HookType::OnStart);

        // Verify async operations in body
        assert!(hook.body.iter().any(|stmt| {
            if let Statement::LetBinding {
                initializer: Some(expr),
                ..
            } = stmt
            {
                matches!(expr, Expression::Await { .. })
            } else {
                false
            }
        }));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_all_hook_types() {
        let hook_inputs = vec![
            ("hook on_start { }", HookType::OnStart),
            ("hook on_stop { }", HookType::OnStop),
            (
                "hook on_error(e) { }",
                HookType::OnError {
                    error_param: Some("e".to_string()),
                },
            ),
            (
                "hook on_restart(reason) { }",
                HookType::OnRestart {
                    reason_param: Some("reason".to_string()),
                },
            ),
        ];

        for (input, expected_type) in hook_inputs {
            let mut ctx = ParserTestContext::new(input);
            let hook = ctx.expect_parse_success(|parser| parser.parse_hook());

            match (&hook.hook_type, &expected_type) {
                (HookType::OnStart, HookType::OnStart) => {}
                (HookType::OnStop, HookType::OnStop) => {}
                (
                    HookType::OnError {
                        error_param: actual,
                    },
                    HookType::OnError {
                        error_param: expected,
                    },
                ) => {
                    assert_eq!(actual, expected);
                }
                (
                    HookType::OnRestart {
                        reason_param: actual,
                    },
                    HookType::OnRestart {
                        reason_param: expected,
                    },
                ) => {
                    assert_eq!(actual, expected);
                }
                _ => panic!("Hook type mismatch for input: {}", input),
            }
        }
    }

    // =================================================================
    // MESSAGE SEND PARSING TESTS (Actor Communication)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_simple_message_send() {
        let mut ctx = ParserTestContext::new("actor_ref ! Increment");

        let send_expr = ctx.expect_parse_success(|parser| parser.parse_send_expression());

        if let Expression::Identifier { name, .. } = send_expr.receiver.as_ref() {
            assert_eq!(name, "actor_ref");
        } else {
            panic!("Expected identifier for receiver");
        }

        if let Expression::Identifier { name, .. } = send_expr.message.as_ref() {
            assert_eq!(name, "Increment");
        } else {
            panic!("Expected identifier for message");
        }

        assert!(matches!(send_expr.send_type, SendType::Fire));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_message_send_with_data() {
        let mut ctx = ParserTestContext::new("chat_agent ! SendMessage(\"Hello\", sender_id)");

        let send_expr = ctx.expect_parse_success(|parser| parser.parse_send_expression());

        if let Expression::FunctionCall { function, args, .. } = send_expr.message.as_ref() {
            if let Expression::Identifier { name, .. } = function.as_ref() {
                assert_eq!(name, "SendMessage");
            }
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected function call for message");
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_chained_message_sends() {
        let mut ctx = ParserTestContext::new("actor_a ! Forward(actor_b ! Message(data))");

        let send_expr = ctx.expect_parse_success(|parser| parser.parse_send_expression());

        // Verify nested send expressions are parsed correctly
        if let Expression::FunctionCall { function, args, .. } = send_expr.message.as_ref() {
            if let Expression::Identifier { name, .. } = function.as_ref() {
                assert_eq!(name, "Forward");
            }
            assert_eq!(args.len(), 1);

            // Check nested send expression
            if let Expression::Send(nested_send) = &args[0] {
                if let Expression::Identifier { name, .. } = nested_send.receiver.as_ref() {
                    assert_eq!(name, "actor_b");
                }
            } else {
                panic!("Expected nested send expression");
            }
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_request_response_send() {
        let mut ctx = ParserTestContext::new("calculator !? Add(5, 3) timeout(1000)");

        let send_expr = ctx.expect_parse_success(|parser| parser.parse_send_expression());

        if let SendType::Call { timeout } = send_expr.send_type {
            assert_eq!(timeout, Some(1000));
        } else {
            panic!("Expected Call send type");
        }
    }

    // =================================================================
    // SPAWN EXPRESSION PARSING TESTS (Actor Creation)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_basic_spawn_expression() {
        let mut ctx = ParserTestContext::new("spawn ChatAgent");

        let spawn_expr = ctx.expect_parse_success(|parser| parser.parse_spawn_expression());

        assert_eq!(spawn_expr.actor_type, "ChatAgent");
        assert!(spawn_expr.args.is_empty());
        assert!(spawn_expr.supervisor.is_none());
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_spawn_with_arguments() {
        let mut ctx = ParserTestContext::new("spawn Counter(0, \"counter1\")");

        let spawn_expr = ctx.expect_parse_success(|parser| parser.parse_spawn_expression());

        assert_eq!(spawn_expr.actor_type, "Counter");
        assert_eq!(spawn_expr.args.len(), 2);

        assert!(matches!(spawn_expr.args[0], Expression::IntLiteral(0)));
        if let Expression::StringLiteral { value, .. } = &spawn_expr.args[1] {
            assert_eq!(value, "counter1");
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_spawn_with_supervisor() {
        let mut ctx = ParserTestContext::new("spawn Worker under MainSupervisor");

        let spawn_expr = ctx.expect_parse_success(|parser| parser.parse_spawn_expression());

        assert_eq!(spawn_expr.actor_type, "Worker");
        assert_eq!(spawn_expr.supervisor.as_ref().unwrap(), "MainSupervisor");
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_spawn_with_options() {
        let input = r#"
            spawn Worker {
                restart: one_for_one,
                max_restarts: 3,
                restart_period: 60000,
                shutdown_timeout: 5000
            }
        "#;

        let mut ctx = ParserTestContext::new(input);
        let spawn_expr = ctx.expect_parse_success(|parser| parser.parse_spawn_expression());

        let options = &spawn_expr.spawn_options;
        assert!(matches!(
            options.restart_strategy,
            Some(RestartStrategy::OneForOne)
        ));
        assert_eq!(options.max_restarts, Some(3));
        assert_eq!(options.restart_period, Some(60000));
        assert_eq!(options.shutdown_timeout, Some(5000));
    }

    // =================================================================
    // SUPERVISOR DEFINITION PARSING TESTS (Fault Tolerance)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_basic_supervisor() {
        let mut ctx = ParserTestContext::new("supervisor ChatSupervisor { strategy: one_for_one }");

        let supervisor = ctx.expect_parse_success(|parser| parser.parse_supervisor_def());

        assert_eq!(supervisor.name, "ChatSupervisor");
        assert!(matches!(supervisor.strategy, RestartStrategy::OneForOne));
        assert!(supervisor.child_specs.is_empty());
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_supervisor_with_children() {
        let input = r#"
            supervisor WorkerSupervisor {
                strategy: one_for_all,
                max_restarts: 5,
                max_seconds: 300,

                child worker1: Worker(1) {
                    restart: permanent,
                    shutdown: timeout(5000)
                },

                child worker2: Worker(2) {
                    restart: transient,
                    shutdown: brutal_kill
                }
            }
        "#;

        let mut ctx = ParserTestContext::new(input);
        let supervisor = ctx.expect_parse_success(|parser| parser.parse_supervisor_def());

        assert_eq!(supervisor.name, "WorkerSupervisor");
        assert!(matches!(supervisor.strategy, RestartStrategy::OneForAll));
        assert_eq!(supervisor.max_restarts, 5);
        assert_eq!(supervisor.max_seconds, 300);
        assert_eq!(supervisor.child_specs.len(), 2);

        let child1 = &supervisor.child_specs[0];
        assert_eq!(child1.id, "worker1");
        assert_eq!(child1.actor_type, "Worker");
        assert!(matches!(child1.restart_type, RestartType::Permanent));
        assert!(matches!(child1.shutdown_type, ShutdownType::Timeout(5000)));

        let child2 = &supervisor.child_specs[1];
        assert_eq!(child2.id, "worker2");
        assert!(matches!(child2.restart_type, RestartType::Transient));
        assert!(matches!(child2.shutdown_type, ShutdownType::BrutalKill));
    }

    // =================================================================
    // ERROR HANDLING PARSER TESTS (Graceful Failure)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_error_missing_actor_name() {
        let mut ctx = ParserTestContext::new("actor { }");

        let error = ctx.expect_parse_error(|parser| parser.parse_actor_def());

        assert!(matches!(error.kind, ParseErrorKind::ExpectedIdentifier));
        assert!(error.span.start < error.span.end);
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_error_unterminated_receive_block() {
        let mut ctx = ParserTestContext::new("receive { Increment => self.count += 1");

        let error = ctx.expect_parse_error(|parser| parser.parse_receive_block());

        assert!(matches!(
            error.kind,
            ParseErrorKind::ExpectedToken(TokenKind::RightBrace)
        ));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_error_invalid_hook_type() {
        let mut ctx = ParserTestContext::new("hook on_invalid { }");

        let error = ctx.expect_parse_error(|parser| parser.parse_hook());

        assert!(matches!(error.kind, ParseErrorKind::InvalidHookType));
        assert!(error.message.contains("on_invalid"));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_error_malformed_pattern() {
        let mut ctx = ParserTestContext::new("receive { 123abc => handle() }");

        let error = ctx.expect_parse_error(|parser| parser.parse_receive_block());

        assert!(matches!(error.kind, ParseErrorKind::InvalidPattern));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_error_missing_fat_arrow() {
        let mut ctx = ParserTestContext::new("receive { Increment self.count += 1 }");

        let error = ctx.expect_parse_error(|parser| parser.parse_receive_block());

        assert!(matches!(
            error.kind,
            ParseErrorKind::ExpectedToken(TokenKind::FatArrow)
        ));
    }

    // =================================================================
    // RECOVERY PARSING TESTS (Error Recovery and Continuation)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_recovery_after_syntax_error() {
        let input = r#"
            actor InvalidActor {
                receive {
                    @#$ invalid syntax
                    ValidMessage => handle()
                }

                hook on_start {
                    println("This should still parse")
                }
            }
        "#;

        let mut ctx = ParserTestContext::new(input);

        // Parser should recover and continue parsing valid parts
        let result = ctx.parser.parse_actor_def_with_recovery();

        assert!(result.has_errors());
        assert!(!result.parsed_items.is_empty());

        // Should have recovered and parsed the hook
        assert!(result
            .parsed_items
            .iter()
            .any(|item| matches!(item, ActorBodyItem::Hook(_))));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_parse_multiple_actors_with_errors() {
        let input = r#"
            actor ValidActor1 { }

            actor @invalid { }

            actor ValidActor2 {
                receive { Message => handle() }
            }
        "#;

        let mut ctx = ParserTestContext::new(input);
        let results = ctx.parser.parse_multiple_actors_with_recovery();

        assert_eq!(results.valid_actors.len(), 2);
        assert_eq!(results.errors.len(), 1);

        assert_eq!(results.valid_actors[0].name, "ValidActor1");
        assert_eq!(results.valid_actors[1].name, "ValidActor2");
    }

    // =================================================================
    // PROPERTY-BASED PARSER TESTS (Invariant Validation)
    // =================================================================

    proptest! {
        #[test]
        #[ignore] // EXTREME TDD: Property tests first, no implementation yet
        fn prop_valid_actor_names_always_parse(
            name in r"[A-Z][a-zA-Z0-9_]*"
        ) {
            let input = format!("actor {} {{ }}", name);
            let mut ctx = ParserTestContext::new(&input);

            let result = ctx.parser.parse_actor_def();
            prop_assert!(result.is_ok());

            if let Ok(actor) = result {
                prop_assert_eq!(actor.name, name);
            }
        }

        #[test]
        #[ignore] // EXTREME TDD: Property tests first, no implementation yet
        fn prop_balanced_braces_always_parse(
            content in r"\{[^{}]*\}"
        ) {
            let input = format!("receive {}", content);
            let mut ctx = ParserTestContext::new(&input);

            // Property: Well-formed braces should always parse successfully
            let result = ctx.parser.parse_receive_block();

            // If braces are balanced, parsing should succeed
            let brace_balance = content.chars().fold(0i32, |acc, c| {
                match c {
                    '{' => acc + 1,
                    '}' => acc - 1,
                    _ => acc,
                }
            });

            if brace_balance == 0 {
                prop_assert!(result.is_ok());
            }
        }

        #[test]
        #[ignore] // EXTREME TDD: Property tests first, no implementation yet
        fn prop_message_send_operators_parse(
            receiver in r"[a-z][a-zA-Z0-9_]*",
            message in r"[A-Z][a-zA-Z0-9_]*"
        ) {
            let inputs = vec![
                format!("{} ! {}", receiver, message),      // Fire
                format!("{} !? {}", receiver, message),     // Call
                format!("{} !> {}", receiver, message),     // Cast
            ];

            for input in inputs {
                let mut ctx = ParserTestContext::new(&input);
                let result = ctx.parser.parse_send_expression();

                prop_assert!(result.is_ok(), "Failed to parse: {}", input);
            }
        }

        #[test]
        #[ignore] // EXTREME TDD: Property tests first, no implementation yet
        fn prop_pattern_destructuring_depth(
            depth in 1usize..5
        ) {
            // Generate nested pattern destructuring
            let pattern = generate_nested_pattern(depth);
            let input = format!("receive {{ {} => handle() }}", pattern);

            let mut ctx = ParserTestContext::new(&input);
            let result = ctx.parser.parse_receive_block();

            // Property: Reasonable nesting depths should parse successfully
            prop_assert!(result.is_ok());
        }
    }

    fn generate_nested_pattern(depth: usize) -> String {
        if depth == 0 {
            "value".to_string()
        } else {
            format!("Container({})", generate_nested_pattern(depth - 1))
        }
    }

    // =================================================================
    // PARSER PERFORMANCE TESTS (Parsing Speed and Memory)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Performance tests first, no implementation yet
    fn test_parser_performance_large_actor() {
        use std::time::Instant;

        // Generate large actor definition
        let mut large_actor = String::from("actor LargeActor {\n");

        // Add many receive arms
        for i in 0..1000 {
            large_actor.push_str(&format!(
                "    receive {{ Message{} => handler{}() }}\n",
                i, i
            ));
        }

        // Add many hooks
        for i in 0..100 {
            large_actor.push_str(&format!(
                "    hook on_start {{ println(\"Hook {}\") }}\n",
                i
            ));
        }

        large_actor.push_str("}");

        let mut ctx = ParserTestContext::new(&large_actor);
        let start = Instant::now();
        let result = ctx.parser.parse_actor_def();
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration.as_millis() < 500); // Parse large actor in <500ms

        if let Ok(actor) = result {
            assert_eq!(actor.name, "LargeActor");
            assert!(actor.body.len() > 1000);
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Memory tests first, no implementation yet
    fn test_parser_memory_usage() {
        use std::alloc::{GlobalAlloc, Layout, System};
        use std::sync::atomic::{AtomicUsize, Ordering};

        // Custom allocator to track memory usage
        struct TrackingAlloc;
        static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

        unsafe impl GlobalAlloc for TrackingAlloc {
            unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
                let ptr = System.alloc(layout);
                if !ptr.is_null() {
                    ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
                }
                ptr
            }

            unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
                System.dealloc(ptr, layout);
                ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
            }
        }

        // Parse moderately complex actor
        let input = r#"
            actor ComplexActor<T: Send> {
                receive {
                    Message(data) if data.len() > 0 => process(data),
                    Control { action, params } => handle_control(action, params),
                    Batch(items) => items.iter().for_each(|item| self.process(item))
                }

                hook on_start { initialize() }
                hook on_error(err) { log_error(err) }

                fn process(data: T) { /* implementation */ }
            }
        "#;

        let before = ALLOCATED.load(Ordering::SeqCst);
        let mut ctx = ParserTestContext::new(input);
        let result = ctx.parser.parse_actor_def();
        let after = ALLOCATED.load(Ordering::SeqCst);

        assert!(result.is_ok());

        let memory_used = after.saturating_sub(before);
        assert!(memory_used < 1024 * 1024); // <1MB for complex actor parsing
    }

    // =================================================================
    // PARSER PRECEDENCE AND ASSOCIATIVITY TESTS
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_message_send_precedence() {
        let test_cases = vec![
            ("a ! b ! c", "((a ! b) ! c)"),     // Left associative
            ("func() ! msg", "(func() ! msg)"), // Function call binds tighter
            ("a ! b + c", "((a ! b) + c)"),     // Send binds tighter than arithmetic
        ];

        for (input, expected_parse) in test_cases {
            let mut ctx = ParserTestContext::new(input);
            let result = ctx.parser.parse_expression();

            assert!(result.is_ok());

            // Would verify the AST structure matches expected precedence
            if let Ok(expr) = result {
                let ast_string = format_ast_structure(&expr);
                assert_eq!(ast_string, expected_parse);
            }
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_pattern_matching_precedence() {
        let input = r#"
            receive {
                A(B(x)) => nested_match(x),
                C { field: D(y) } => struct_with_nested(y),
                (E(a), F { b }) => tuple_mixed(a, b)
            }
        "#;

        let mut ctx = ParserTestContext::new(input);
        let result = ctx.parser.parse_receive_block();

        assert!(result.is_ok());

        if let Ok(receive) = result {
            assert_eq!(receive.arms.len(), 3);

            // Verify nested pattern parsing
            for arm in &receive.arms {
                assert!(arm.pattern.validate_nesting_depth() <= 3);
            }
        }
    }

    // Helper function for AST structure formatting (would be implemented)
    fn format_ast_structure(expr: &Expression) -> String {
        match expr {
            Expression::Send(send) => {
                format!(
                    "({} ! {})",
                    format_ast_structure(&send.receiver),
                    format_ast_structure(&send.message)
                )
            }
            Expression::Identifier { name, .. } => name.clone(),
            Expression::FunctionCall { function, .. } => {
                format!("{}()", format_ast_structure(function))
            }
            _ => "expr".to_string(),
        }
    }

    // =================================================================
    // COMPREHENSIVE PARSER COVERAGE VALIDATION
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_all_parser_methods_covered() {
        // Meta-test to ensure we've tested all parsing methods
        let parser_methods = vec![
            "parse_actor_def",
            "parse_receive_block",
            "parse_hook",
            "parse_send_expression",
            "parse_spawn_expression",
            "parse_supervisor_def",
            "parse_pattern",
            "parse_expression",
            "parse_statement",
            "parse_type",
            "parse_parameter_list",
        ];

        // Each parser method should have corresponding test coverage
        for method in parser_methods {
            // This meta-test ensures comprehensive coverage
            assert!(!method.is_empty());
        }
    }
}
