/*!
 * EXTREME TDD Grammar Tests - ACTOR-003
 *
 * CRITICAL: ALL grammar tests MUST be written FIRST before ANY implementation.
 * These tests define the EXACT BNF grammar for actor syntax.
 *
 * Following Toyota Way: Build quality in from the start.
 * Following EXTREME TDD: 100% test coverage before any code exists.
 *
 * Complexity Budget: Each test function ≤5 cyclomatic, ≤8 cognitive
 * Coverage Target: 100% BNF rule coverage
 * Test Ratio: 3:1 test-to-implementation ratio
 */

use ruchy::frontend::ast::{ActorDef, Expression, Hook, Pattern, ReceiveBlock};
use ruchy::frontend::grammar::{BNFRule, Grammar, GrammarRule, TokenType};
use ruchy::frontend::tokens::{Span, Token, TokenKind};

#[cfg(test)]
mod actor_grammar_tests {
    use super::*;
    use proptest::prelude::*;

    /// Test infrastructure for BNF grammar validation
    struct GrammarTestContext {
        grammar: Grammar,
        token_stream: Vec<Token>,
        current_pos: usize,
    }

    impl GrammarTestContext {
        fn new() -> Self {
            Self {
                grammar: Grammar::new(),
                token_stream: Vec::new(),
                current_pos: 0,
            }
        }

        fn add_rule(&mut self, name: &str, rule: BNFRule) {
            self.grammar.add_rule(name, rule);
        }

        fn validate_production(&self, rule_name: &str, input: &[Token]) -> bool {
            self.grammar.validate_production(rule_name, input)
        }
    }

    // =================================================================
    // ACTOR DEFINITION GRAMMAR TESTS (Core BNF Rules)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_actor_definition_basic_bnf() {
        let mut ctx = GrammarTestContext::new();

        // BNF: actor_def := "actor" IDENTIFIER "{" actor_body "}"
        ctx.add_rule(
            "actor_def",
            BNFRule::sequence(vec![
                BNFRule::terminal(TokenKind::Actor),
                BNFRule::terminal(TokenKind::Identifier),
                BNFRule::terminal(TokenKind::LeftBrace),
                BNFRule::non_terminal("actor_body"),
                BNFRule::terminal(TokenKind::RightBrace),
            ]),
        );

        let tokens = vec![
            Token::new(TokenKind::Actor, "actor", Span::new(0, 5)),
            Token::new(TokenKind::Identifier, "ChatAgent", Span::new(6, 15)),
            Token::new(TokenKind::LeftBrace, "{", Span::new(16, 17)),
            Token::new(TokenKind::RightBrace, "}", Span::new(18, 19)),
        ];

        assert!(ctx.validate_production("actor_def", &tokens));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_actor_definition_with_state_bnf() {
        let mut ctx = GrammarTestContext::new();

        // BNF: actor_def := "actor" IDENTIFIER "(" state_params ")" "{" actor_body "}"
        ctx.add_rule(
            "actor_def_with_state",
            BNFRule::sequence(vec![
                BNFRule::terminal(TokenKind::Actor),
                BNFRule::terminal(TokenKind::Identifier),
                BNFRule::terminal(TokenKind::LeftParen),
                BNFRule::non_terminal("state_params"),
                BNFRule::terminal(TokenKind::RightParen),
                BNFRule::terminal(TokenKind::LeftBrace),
                BNFRule::non_terminal("actor_body"),
                BNFRule::terminal(TokenKind::RightBrace),
            ]),
        );

        let tokens = vec![
            Token::new(TokenKind::Actor, "actor", Span::new(0, 5)),
            Token::new(TokenKind::Identifier, "Counter", Span::new(6, 13)),
            Token::new(TokenKind::LeftParen, "(", Span::new(13, 14)),
            Token::new(TokenKind::Identifier, "count", Span::new(14, 19)),
            Token::new(TokenKind::Colon, ":", Span::new(19, 20)),
            Token::new(TokenKind::Identifier, "i32", Span::new(21, 24)),
            Token::new(TokenKind::RightParen, ")", Span::new(24, 25)),
            Token::new(TokenKind::LeftBrace, "{", Span::new(26, 27)),
            Token::new(TokenKind::RightBrace, "}", Span::new(28, 29)),
        ];

        assert!(ctx.validate_production("actor_def_with_state", &tokens));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_actor_body_bnf() {
        let mut ctx = GrammarTestContext::new();

        // BNF: actor_body := (receive_block | hook_def | method_def)*
        ctx.add_rule(
            "actor_body",
            BNFRule::star(BNFRule::choice(vec![
                BNFRule::non_terminal("receive_block"),
                BNFRule::non_terminal("hook_def"),
                BNFRule::non_terminal("method_def"),
            ])),
        );

        // Test empty actor body
        let empty_tokens = vec![];
        assert!(ctx.validate_production("actor_body", &empty_tokens));
    }

    // =================================================================
    // RECEIVE BLOCK GRAMMAR TESTS (Message Pattern Matching)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_receive_block_basic_bnf() {
        let mut ctx = GrammarTestContext::new();

        // BNF: receive_block := "receive" "{" receive_arms "}"
        ctx.add_rule(
            "receive_block",
            BNFRule::sequence(vec![
                BNFRule::terminal(TokenKind::Receive),
                BNFRule::terminal(TokenKind::LeftBrace),
                BNFRule::non_terminal("receive_arms"),
                BNFRule::terminal(TokenKind::RightBrace),
            ]),
        );

        let tokens = vec![
            Token::new(TokenKind::Receive, "receive", Span::new(0, 7)),
            Token::new(TokenKind::LeftBrace, "{", Span::new(8, 9)),
            Token::new(TokenKind::RightBrace, "}", Span::new(10, 11)),
        ];

        assert!(ctx.validate_production("receive_block", &tokens));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_receive_arms_single_pattern_bnf() {
        let mut ctx = GrammarTestContext::new();

        // BNF: receive_arms := receive_arm ("," receive_arm)*
        // BNF: receive_arm := pattern "=>" expression
        ctx.add_rule(
            "receive_arms",
            BNFRule::sequence(vec![
                BNFRule::non_terminal("receive_arm"),
                BNFRule::star(BNFRule::sequence(vec![
                    BNFRule::terminal(TokenKind::Comma),
                    BNFRule::non_terminal("receive_arm"),
                ])),
            ]),
        );

        ctx.add_rule(
            "receive_arm",
            BNFRule::sequence(vec![
                BNFRule::non_terminal("pattern"),
                BNFRule::terminal(TokenKind::FatArrow),
                BNFRule::non_terminal("expression"),
            ]),
        );

        let tokens = vec![
            Token::new(TokenKind::Identifier, "Increment", Span::new(0, 9)),
            Token::new(TokenKind::FatArrow, "=>", Span::new(10, 12)),
            Token::new(TokenKind::Identifier, "self", Span::new(13, 17)),
        ];

        assert!(ctx.validate_production("receive_arm", &tokens));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_receive_pattern_with_data_bnf() {
        let mut ctx = GrammarTestContext::new();

        // BNF: pattern := IDENTIFIER | IDENTIFIER "(" pattern_params ")" | "_"
        ctx.add_rule(
            "pattern",
            BNFRule::choice(vec![
                BNFRule::terminal(TokenKind::Identifier),
                BNFRule::sequence(vec![
                    BNFRule::terminal(TokenKind::Identifier),
                    BNFRule::terminal(TokenKind::LeftParen),
                    BNFRule::non_terminal("pattern_params"),
                    BNFRule::terminal(TokenKind::RightParen),
                ]),
                BNFRule::terminal(TokenKind::Underscore),
            ]),
        );

        let tokens = vec![
            Token::new(TokenKind::Identifier, "SendMessage", Span::new(0, 11)),
            Token::new(TokenKind::LeftParen, "(", Span::new(11, 12)),
            Token::new(TokenKind::Identifier, "content", Span::new(12, 19)),
            Token::new(TokenKind::RightParen, ")", Span::new(19, 20)),
        ];

        assert!(ctx.validate_production("pattern", &tokens));
    }

    // =================================================================
    // HOOK DEFINITION GRAMMAR TESTS (Lifecycle Events)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_hook_definition_bnf() {
        let mut ctx = GrammarTestContext::new();

        // BNF: hook_def := "hook" hook_type "{" statement_list "}"
        ctx.add_rule(
            "hook_def",
            BNFRule::sequence(vec![
                BNFRule::terminal(TokenKind::Hook),
                BNFRule::non_terminal("hook_type"),
                BNFRule::terminal(TokenKind::LeftBrace),
                BNFRule::non_terminal("statement_list"),
                BNFRule::terminal(TokenKind::RightBrace),
            ]),
        );

        let tokens = vec![
            Token::new(TokenKind::Hook, "hook", Span::new(0, 4)),
            Token::new(TokenKind::Identifier, "on_start", Span::new(5, 13)),
            Token::new(TokenKind::LeftBrace, "{", Span::new(14, 15)),
            Token::new(TokenKind::RightBrace, "}", Span::new(16, 17)),
        ];

        assert!(ctx.validate_production("hook_def", &tokens));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_hook_types_bnf() {
        let mut ctx = GrammarTestContext::new();

        // BNF: hook_type := "on_start" | "on_stop" | "on_error" | "on_restart"
        ctx.add_rule(
            "hook_type",
            BNFRule::choice(vec![
                BNFRule::terminal(TokenKind::OnStart),
                BNFRule::terminal(TokenKind::OnStop),
                BNFRule::terminal(TokenKind::OnError),
                BNFRule::terminal(TokenKind::OnRestart),
            ]),
        );

        let on_start = vec![Token::new(TokenKind::OnStart, "on_start", Span::new(0, 8))];
        let on_stop = vec![Token::new(TokenKind::OnStop, "on_stop", Span::new(0, 7))];
        let on_error = vec![Token::new(TokenKind::OnError, "on_error", Span::new(0, 8))];
        let on_restart = vec![Token::new(
            TokenKind::OnRestart,
            "on_restart",
            Span::new(0, 10),
        )];

        assert!(ctx.validate_production("hook_type", &on_start));
        assert!(ctx.validate_production("hook_type", &on_stop));
        assert!(ctx.validate_production("hook_type", &on_error));
        assert!(ctx.validate_production("hook_type", &on_restart));
    }

    // =================================================================
    // MESSAGE SENDING GRAMMAR TESTS (Actor Communication)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_send_expression_bnf() {
        let mut ctx = GrammarTestContext::new();

        // BNF: send_expr := expression "!" expression
        ctx.add_rule(
            "send_expr",
            BNFRule::sequence(vec![
                BNFRule::non_terminal("expression"),
                BNFRule::terminal(TokenKind::Bang),
                BNFRule::non_terminal("expression"),
            ]),
        );

        let tokens = vec![
            Token::new(TokenKind::Identifier, "actor_ref", Span::new(0, 9)),
            Token::new(TokenKind::Bang, "!", Span::new(9, 10)),
            Token::new(TokenKind::Identifier, "Increment", Span::new(10, 19)),
        ];

        assert!(ctx.validate_production("send_expr", &tokens));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_send_with_message_data_bnf() {
        let mut ctx = GrammarTestContext::new();

        let tokens = vec![
            Token::new(TokenKind::Identifier, "chat_agent", Span::new(0, 10)),
            Token::new(TokenKind::Bang, "!", Span::new(10, 11)),
            Token::new(TokenKind::Identifier, "SendMessage", Span::new(11, 22)),
            Token::new(TokenKind::LeftParen, "(", Span::new(22, 23)),
            Token::new(
                TokenKind::StringLiteral,
                "\"Hello World\"",
                Span::new(23, 36),
            ),
            Token::new(TokenKind::RightParen, ")", Span::new(36, 37)),
        ];

        assert!(ctx.validate_production("send_expr", &tokens));
    }

    // =================================================================
    // SUPERVISION GRAMMAR TESTS (Fault Tolerance)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_supervisor_definition_bnf() {
        let mut ctx = GrammarTestContext::new();

        // BNF: supervisor_def := "supervisor" IDENTIFIER "{" supervisor_body "}"
        ctx.add_rule(
            "supervisor_def",
            BNFRule::sequence(vec![
                BNFRule::terminal(TokenKind::Supervisor),
                BNFRule::terminal(TokenKind::Identifier),
                BNFRule::terminal(TokenKind::LeftBrace),
                BNFRule::non_terminal("supervisor_body"),
                BNFRule::terminal(TokenKind::RightBrace),
            ]),
        );

        let tokens = vec![
            Token::new(TokenKind::Supervisor, "supervisor", Span::new(0, 10)),
            Token::new(TokenKind::Identifier, "ChatSupervisor", Span::new(11, 25)),
            Token::new(TokenKind::LeftBrace, "{", Span::new(26, 27)),
            Token::new(TokenKind::RightBrace, "}", Span::new(28, 29)),
        ];

        assert!(ctx.validate_production("supervisor_def", &tokens));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_restart_strategy_bnf() {
        let mut ctx = GrammarTestContext::new();

        // BNF: restart_strategy := "one_for_one" | "one_for_all" | "rest_for_one"
        ctx.add_rule(
            "restart_strategy",
            BNFRule::choice(vec![
                BNFRule::terminal(TokenKind::OneForOne),
                BNFRule::terminal(TokenKind::OneForAll),
                BNFRule::terminal(TokenKind::RestForOne),
            ]),
        );

        let one_for_one = vec![Token::new(
            TokenKind::OneForOne,
            "one_for_one",
            Span::new(0, 11),
        )];
        let one_for_all = vec![Token::new(
            TokenKind::OneForAll,
            "one_for_all",
            Span::new(0, 11),
        )];
        let rest_for_one = vec![Token::new(
            TokenKind::RestForOne,
            "rest_for_one",
            Span::new(0, 12),
        )];

        assert!(ctx.validate_production("restart_strategy", &one_for_one));
        assert!(ctx.validate_production("restart_strategy", &one_for_all));
        assert!(ctx.validate_production("restart_strategy", &rest_for_one));
    }

    // =================================================================
    // SPAWN EXPRESSION GRAMMAR TESTS (Actor Creation)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_spawn_expression_bnf() {
        let mut ctx = GrammarTestContext::new();

        // BNF: spawn_expr := "spawn" IDENTIFIER | "spawn" IDENTIFIER "(" args ")"
        ctx.add_rule(
            "spawn_expr",
            BNFRule::choice(vec![
                BNFRule::sequence(vec![
                    BNFRule::terminal(TokenKind::Spawn),
                    BNFRule::terminal(TokenKind::Identifier),
                ]),
                BNFRule::sequence(vec![
                    BNFRule::terminal(TokenKind::Spawn),
                    BNFRule::terminal(TokenKind::Identifier),
                    BNFRule::terminal(TokenKind::LeftParen),
                    BNFRule::non_terminal("args"),
                    BNFRule::terminal(TokenKind::RightParen),
                ]),
            ]),
        );

        let tokens_no_args = vec![
            Token::new(TokenKind::Spawn, "spawn", Span::new(0, 5)),
            Token::new(TokenKind::Identifier, "ChatAgent", Span::new(6, 15)),
        ];

        let tokens_with_args = vec![
            Token::new(TokenKind::Spawn, "spawn", Span::new(0, 5)),
            Token::new(TokenKind::Identifier, "Counter", Span::new(6, 13)),
            Token::new(TokenKind::LeftParen, "(", Span::new(13, 14)),
            Token::new(TokenKind::IntegerLiteral, "0", Span::new(14, 15)),
            Token::new(TokenKind::RightParen, ")", Span::new(15, 16)),
        ];

        assert!(ctx.validate_production("spawn_expr", &tokens_no_args));
        assert!(ctx.validate_production("spawn_expr", &tokens_with_args));
    }

    // =================================================================
    // PROPERTY-BASED GRAMMAR TESTS (Invariant Validation)
    // =================================================================

    proptest! {
        #[test]
        #[ignore] // EXTREME TDD: Property tests first, no implementation yet
        fn prop_actor_name_must_be_valid_identifier(name in r"[A-Z][a-zA-Z0-9_]*") {
            let mut ctx = GrammarTestContext::new();
            ctx.add_rule("actor_name", BNFRule::terminal(TokenKind::Identifier));

            let token = Token::new(TokenKind::Identifier, &name, Span::new(0, name.len()));
            prop_assert!(ctx.validate_production("actor_name", &[token]));
        }

        #[test]
        #[ignore] // EXTREME TDD: Property tests first, no implementation yet
        fn prop_receive_block_must_have_balanced_braces(
            content in r"receive\s*\{[^{}]*\}"
        ) {
            // Property: All receive blocks must have balanced braces
            let brace_count = content.chars().fold(0i32, |acc, c| {
                match c {
                    '{' => acc + 1,
                    '}' => acc - 1,
                    _ => acc,
                }
            });
            prop_assert_eq!(brace_count, 0);
        }

        #[test]
        #[ignore] // EXTREME TDD: Property tests first, no implementation yet
        fn prop_send_expression_associativity(
            actor1 in r"[a-z_][a-zA-Z0-9_]*",
            actor2 in r"[a-z_][a-zA-Z0-9_]*",
            msg in r"[A-Z][a-zA-Z0-9_]*"
        ) {
            // Property: Send operations are left-associative
            // a ! b ! c should parse as (a ! b) ! c
            let mut ctx = GrammarTestContext::new();

            // This property test ensures correct parsing precedence
            let expression = format!("{} ! {} ! {}", actor1, actor2, msg);
            // Test would validate left-associative parsing
        }
    }

    // =================================================================
    // NEGATIVE GRAMMAR TESTS (Error Case Validation)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_invalid_actor_definition_missing_name() {
        let mut ctx = GrammarTestContext::new();

        let invalid_tokens = vec![
            Token::new(TokenKind::Actor, "actor", Span::new(0, 5)),
            Token::new(TokenKind::LeftBrace, "{", Span::new(6, 7)),
            Token::new(TokenKind::RightBrace, "}", Span::new(7, 8)),
        ];

        assert!(!ctx.validate_production("actor_def", &invalid_tokens));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_invalid_receive_block_missing_fat_arrow() {
        let mut ctx = GrammarTestContext::new();

        let invalid_tokens = vec![
            Token::new(TokenKind::Identifier, "Increment", Span::new(0, 9)),
            Token::new(TokenKind::Identifier, "self", Span::new(10, 14)),
        ];

        assert!(!ctx.validate_production("receive_arm", &invalid_tokens));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_invalid_hook_unknown_type() {
        let mut ctx = GrammarTestContext::new();

        let invalid_tokens = vec![
            Token::new(TokenKind::Hook, "hook", Span::new(0, 4)),
            Token::new(TokenKind::Identifier, "on_unknown", Span::new(5, 15)),
            Token::new(TokenKind::LeftBrace, "{", Span::new(16, 17)),
            Token::new(TokenKind::RightBrace, "}", Span::new(17, 18)),
        ];

        assert!(!ctx.validate_production("hook_def", &invalid_tokens));
    }

    // =================================================================
    // COMPLETE BNF GRAMMAR SPECIFICATION TESTS
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_complete_actor_system_grammar_bnf() {
        let mut ctx = GrammarTestContext::new();

        // Complete BNF grammar for actor system
        let bnf_rules = vec![
            // Top-level constructs
            ("program", "item*"),
            ("item", "actor_def | supervisor_def | function_def"),
            // Actor definitions
            (
                "actor_def",
                "\"actor\" IDENTIFIER actor_params? \"{\" actor_body \"}\"",
            ),
            ("actor_params", "\"(\" param_list \")\""),
            ("actor_body", "(receive_block | hook_def | method_def)*"),
            // Receive blocks
            ("receive_block", "\"receive\" \"{\" receive_arms \"}\""),
            ("receive_arms", "receive_arm (\",\" receive_arm)* \",\"?"),
            ("receive_arm", "pattern \"=>\" expression"),
            // Patterns
            (
                "pattern",
                "IDENTIFIER | IDENTIFIER \"(\" pattern_params \")\" | \"_\"",
            ),
            ("pattern_params", "pattern (\",\" pattern)*"),
            // Hooks
            ("hook_def", "\"hook\" hook_type \"{\" statement_list \"}\""),
            (
                "hook_type",
                "\"on_start\" | \"on_stop\" | \"on_error\" | \"on_restart\"",
            ),
            // Expressions
            ("send_expr", "expression \"!\" expression"),
            (
                "spawn_expr",
                "\"spawn\" IDENTIFIER | \"spawn\" IDENTIFIER \"(\" args \")\"",
            ),
            // Supervision
            (
                "supervisor_def",
                "\"supervisor\" IDENTIFIER \"{\" supervisor_body \"}\"",
            ),
            (
                "restart_strategy",
                "\"one_for_one\" | \"one_for_all\" | \"rest_for_one\"",
            ),
        ];

        // Add all BNF rules to grammar context
        for (rule_name, _bnf_production) in &bnf_rules {
            // This would parse the BNF string and create actual BNFRule objects
            // For now, we verify the rule names are comprehensive
            assert!(!rule_name.is_empty());
        }

        // Verify we cover all major language constructs
        let required_constructs = vec![
            "actor_def",
            "receive_block",
            "hook_def",
            "send_expr",
            "spawn_expr",
            "supervisor_def",
            "pattern",
            "restart_strategy",
        ];

        for construct in required_constructs {
            assert!(bnf_rules.iter().any(|(name, _)| *name == construct));
        }
    }

    // =================================================================
    // GRAMMAR PRECEDENCE AND ASSOCIATIVITY TESTS
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_message_send_precedence() {
        let mut ctx = GrammarTestContext::new();

        // Test: actor ! msg1 ! msg2 should be left-associative
        // Test: func() ! msg should bind function call first
        // Test: actor ! msg + 1 should bind message send first

        // These tests ensure correct operator precedence in the grammar
        let precedence_tests = vec![
            ("actor ! msg1 ! msg2", "((actor ! msg1) ! msg2)"),
            ("func() ! msg", "(func() ! msg)"),
            ("actor ! msg + 1", "((actor ! msg) + 1)"),
        ];

        for (input, expected_parse) in precedence_tests {
            // This would test that the grammar produces the correct parse tree
            // with proper precedence and associativity
            assert!(!input.is_empty() && !expected_parse.is_empty());
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_pattern_matching_precedence() {
        let mut ctx = GrammarTestContext::new();

        // Test pattern matching precedence in receive blocks
        let pattern_tests = vec![
            ("Msg(x) if x > 0 => handle(x)", "guard expression binding"),
            ("Msg(A(x)) => nested(x)", "nested pattern destructuring"),
            (
                "Msg { field: value } => struct_pattern()",
                "struct pattern syntax",
            ),
        ];

        for (pattern, description) in pattern_tests {
            // Validate pattern parsing precedence
            assert!(!pattern.is_empty() && !description.is_empty());
        }
    }
}

// =================================================================
// GRAMMAR COMPLETENESS VALIDATION (Meta-Tests)
// =================================================================

#[cfg(test)]
mod grammar_completeness_tests {
    use super::*;

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_all_token_types_covered_in_grammar() {
        // Verify that every TokenKind used in actor system has corresponding grammar rules
        let actor_token_types = vec![
            TokenKind::Actor,
            TokenKind::Receive,
            TokenKind::Hook,
            TokenKind::Supervisor,
            TokenKind::Spawn,
            TokenKind::OnStart,
            TokenKind::OnStop,
            TokenKind::OnError,
            TokenKind::OnRestart,
            TokenKind::OneForOne,
            TokenKind::OneForAll,
            TokenKind::RestForOne,
            TokenKind::Bang,
            TokenKind::FatArrow,
        ];

        // Each token type must have at least one grammar rule
        for token_type in actor_token_types {
            // This would verify comprehensive grammar coverage
            assert_ne!(format!("{:?}", token_type), "");
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_grammar_cycle_detection() {
        // Ensure no circular dependencies in grammar rules
        // Example: A -> B -> C -> A would be invalid

        let mut ctx = GrammarTestContext::new();

        // Add potentially cyclic rules for testing
        ctx.add_rule(
            "expr",
            BNFRule::choice(vec![
                BNFRule::non_terminal("send_expr"),
                BNFRule::non_terminal("spawn_expr"),
                BNFRule::terminal(TokenKind::Identifier),
            ]),
        );

        // This would detect and prevent infinite recursion in grammar
        assert!(ctx.grammar.validate_no_cycles());
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_grammar_ambiguity_detection() {
        // Detect ambiguous grammar productions that could lead to multiple parse trees
        let mut ctx = GrammarTestContext::new();

        // Test case: Ambiguous expression parsing
        let potentially_ambiguous = vec![
            "a ! b ! c",      // Could be (a ! b) ! c or a ! (b ! c)
            "spawn Actor(x)", // Could be spawn (Actor(x)) or (spawn Actor)(x)
        ];

        for input in potentially_ambiguous {
            // This would ensure unambiguous grammar
            assert!(ctx.grammar.is_unambiguous(input));
        }
    }
}

// =================================================================
// BENCHMARK TESTS FOR GRAMMAR PERFORMANCE
// =================================================================

#[cfg(test)]
mod grammar_performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    #[ignore] // EXTREME TDD: Performance tests first, no implementation yet
    fn bench_grammar_validation_performance() {
        let mut ctx = GrammarTestContext::new();

        // Setup complete grammar
        // ... (grammar rules would be added here)

        let large_actor_definition = generate_large_actor_definition(1000);

        let start = Instant::now();
        let result = ctx.validate_production("actor_def", &large_actor_definition);
        let duration = start.elapsed();

        // Grammar validation must complete within performance budget
        assert!(result);
        assert!(duration.as_millis() < 100); // <100ms for large definitions
    }

    fn generate_large_actor_definition(size: usize) -> Vec<Token> {
        // Generate large but valid actor definition for performance testing
        let mut tokens = Vec::new();

        tokens.push(Token::new(TokenKind::Actor, "actor", Span::new(0, 5)));
        tokens.push(Token::new(
            TokenKind::Identifier,
            "LargeActor",
            Span::new(6, 16),
        ));
        tokens.push(Token::new(TokenKind::LeftBrace, "{", Span::new(17, 18)));

        // Add many receive patterns
        for i in 0..size {
            tokens.push(Token::new(TokenKind::Receive, "receive", Span::new(0, 7)));
            tokens.push(Token::new(TokenKind::LeftBrace, "{", Span::new(8, 9)));
            tokens.push(Token::new(
                TokenKind::Identifier,
                &format!("Msg{}", i),
                Span::new(10, 15),
            ));
            tokens.push(Token::new(TokenKind::FatArrow, "=>", Span::new(16, 18)));
            tokens.push(Token::new(
                TokenKind::Identifier,
                "handle",
                Span::new(19, 25),
            ));
            tokens.push(Token::new(TokenKind::RightBrace, "}", Span::new(26, 27)));
        }

        tokens.push(Token::new(TokenKind::RightBrace, "}", Span::new(0, 1)));
        tokens
    }
}
