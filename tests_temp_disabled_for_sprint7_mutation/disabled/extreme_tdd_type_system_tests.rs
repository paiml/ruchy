/*!
 * EXTREME TDD Type System Tests - ACTOR-005 (Type Safety & Inference)
 *
 * CRITICAL: ALL type system tests MUST be written FIRST before ANY type checking implementation.
 * These tests define EXACT type safety requirements for actor system with complete coverage.
 *
 * Following Toyota Way: Build quality in from the start.
 * Following EXTREME TDD: 100% type system coverage before any type checking code exists.
 *
 * Complexity Budget: Each test function ≤5 cyclomatic, ≤8 cognitive
 * Coverage Target: 100% type rule coverage + 100% edge cases
 * Test Ratio: 3:1 test-to-implementation ratio
 */

use ruchy::common::span::Span;
use ruchy::frontend::ast::{ActorDef, Expression, Pattern, ReceiveBlock, SendExpr, SpawnExpr};
use ruchy::middleend::type_system::{
    ActorType, MessageType, Substitution, SupervisorType, Type, TypeChecker, TypeConstraint,
    TypeContext, TypeError, TypeErrorKind, TypeInference, TypeResult, TypeScheme, TypeVar,
};
use ruchy::middleend::types::{
    ActorRef, MessageProtocol, ReceiveCapability, RestartPolicy, SendCapability,
    SupervisionCapability, TypeSafety,
};

#[cfg(test)]
mod actor_type_system_tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashMap;

    /// Test infrastructure for type system validation
    struct TypeTestContext {
        type_checker: TypeChecker,
        context: TypeContext,
        next_type_var: u32,
        actor_registry: HashMap<String, ActorType>,
        message_protocols: HashMap<String, MessageProtocol>,
    }

    impl TypeTestContext {
        fn new() -> Self {
            Self {
                type_checker: TypeChecker::new(),
                context: TypeContext::new(),
                next_type_var: 0,
                actor_registry: HashMap::new(),
                message_protocols: HashMap::new(),
            }
        }

        fn fresh_type_var(&mut self) -> TypeVar {
            let var = TypeVar::new(self.next_type_var);
            self.next_type_var += 1;
            var
        }

        fn register_actor(&mut self, name: &str, actor_type: ActorType) {
            self.actor_registry.insert(name.to_string(), actor_type);
            self.context
                .bind_type(name, Type::Actor(actor_type.clone()));
        }

        fn register_message_protocol(&mut self, name: &str, protocol: MessageProtocol) {
            self.message_protocols.insert(name.to_string(), protocol);
        }

        fn type_check_expression(&mut self, expr: &Expression) -> TypeResult<Type> {
            self.type_checker
                .infer_expression_type(expr, &mut self.context)
        }

        fn type_check_send(&mut self, send_expr: &SendExpr) -> TypeResult<Type> {
            self.type_checker
                .type_check_send(send_expr, &mut self.context)
        }

        fn expect_type_success<T>(&mut self, result: TypeResult<T>) -> T {
            match result {
                Ok(value) => value,
                Err(error) => panic!("Expected type checking success, got error: {:?}", error),
            }
        }

        fn expect_type_error<T>(&mut self, result: TypeResult<T>) -> TypeError {
            match result {
                Ok(_) => panic!("Expected type error, but type checking succeeded"),
                Err(error) => error,
            }
        }
    }

    // =================================================================
    // ACTOR TYPE SYSTEM TESTS (Core Actor Typing)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_actor_ref_type_basic() {
        let mut ctx = TypeTestContext::new();

        let chat_agent_type = ActorType {
            name: "ChatAgent".to_string(),
            state_type: Type::Struct {
                name: "ChatAgentState".to_string(),
                fields: vec![
                    ("message_count".to_string(), Type::Int32),
                    ("active".to_string(), Type::Bool),
                ],
            },
            message_types: vec![
                MessageType::simple("SendMessage", vec![Type::String]),
                MessageType::simple("GetCount", vec![]),
                MessageType::simple("Shutdown", vec![]),
            ],
            spawn_params: vec![],
        };

        ctx.register_actor("ChatAgent", chat_agent_type.clone());

        let actor_ref_type = Type::ActorRef {
            actor_type: Box::new(Type::Actor(chat_agent_type)),
            capabilities: vec![SendCapability::Fire, SendCapability::Call],
        };

        // Test: ActorRef type should be well-formed
        assert!(ctx
            .type_checker
            .is_well_formed_type(&actor_ref_type, &ctx.context));

        // Test: ActorRef should support expected operations
        assert!(ctx.type_checker.supports_send(&actor_ref_type));
        assert!(!ctx.type_checker.supports_receive(&actor_ref_type)); // Only actors receive
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_actor_state_type_inference() {
        let mut ctx = TypeTestContext::new();

        let actor_def = ActorDef {
            name: "Counter".to_string(),
            type_params: vec![],
            state_params: vec![StateParam {
                name: "count".to_string(),
                param_type: Type::Int32,
                default_value: Some(Expression::IntLiteral(0)),
                mutability: Mutability::Mutable,
                span: Span::new(0, 10),
            }],
            body: vec![],
            span: Span::new(0, 30),
            visibility: Visibility::Public,
            attributes: vec![],
        };

        let inferred_type = ctx.expect_type_success(
            ctx.type_checker
                .infer_actor_type(&actor_def, &mut ctx.context),
        );

        if let Type::Actor(actor_type) = inferred_type {
            assert_eq!(actor_type.name, "Counter");

            if let Type::Struct { fields, .. } = actor_type.state_type {
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].0, "count");
                assert_eq!(fields[0].1, Type::Int32);
            } else {
                panic!("Expected struct type for actor state");
            }
        } else {
            panic!("Expected Actor type");
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_generic_actor_type_instantiation() {
        let mut ctx = TypeTestContext::new();

        let generic_actor_def = ActorDef {
            name: "GenericActor".to_string(),
            type_params: vec![TypeParam {
                name: "T".to_string(),
                bounds: vec![TypeBound::Send, TypeBound::Clone],
                default_type: None,
                span: Span::new(0, 1),
            }],
            state_params: vec![StateParam {
                name: "data".to_string(),
                param_type: Type::TypeVar(TypeVar::new(0)), // T
                default_value: None,
                mutability: Mutability::Immutable,
                span: Span::new(0, 4),
            }],
            body: vec![],
            span: Span::new(0, 50),
            visibility: Visibility::Public,
            attributes: vec![],
        };

        let inferred_type = ctx.expect_type_success(
            ctx.type_checker
                .infer_actor_type(&generic_actor_def, &mut ctx.context),
        );

        // Test instantiation with String
        let string_instantiation =
            ctx.expect_type_success(ctx.type_checker.instantiate_actor_type(
                &inferred_type,
                &[Type::String],
                &mut ctx.context,
            ));

        if let Type::Actor(actor_type) = string_instantiation {
            if let Type::Struct { fields, .. } = actor_type.state_type {
                assert_eq!(fields[0].1, Type::String); // T -> String
            }
        } else {
            panic!("Expected instantiated Actor type");
        }
    }

    // =================================================================
    // MESSAGE TYPE SAFETY TESTS (Communication Type Safety)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_message_type_compatibility() {
        let mut ctx = TypeTestContext::new();

        let message_protocol = MessageProtocol {
            messages: vec![
                MessageType {
                    name: "SendMessage".to_string(),
                    params: vec![
                        Type::String,
                        Type::ActorRef {
                            actor_type: Box::new(Type::Any),
                            capabilities: vec![SendCapability::Fire],
                        },
                    ],
                    return_type: None,
                    is_async: false,
                },
                MessageType {
                    name: "GetCount".to_string(),
                    params: vec![],
                    return_type: Some(Type::Int32),
                    is_async: false,
                },
            ],
        };

        ctx.register_message_protocol("ChatAgent", message_protocol);

        // Test: Valid message should type check
        let valid_send = SendExpr {
            receiver: Box::new(Expression::Identifier {
                name: "chat_agent".to_string(),
                span: Span::new(0, 10),
            }),
            message: Box::new(Expression::FunctionCall {
                function: Box::new(Expression::Identifier {
                    name: "SendMessage".to_string(),
                    span: Span::new(0, 11),
                }),
                args: vec![
                    Expression::StringLiteral {
                        value: "Hello".to_string(),
                        span: Span::new(0, 7),
                    },
                    Expression::Identifier {
                        name: "sender".to_string(),
                        span: Span::new(0, 6),
                    },
                ],
                span: Span::new(0, 20),
            }),
            send_type: SendType::Fire,
            span: Span::new(0, 30),
        };

        let result = ctx.type_check_send(&valid_send);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_message_type_mismatch_error() {
        let mut ctx = TypeTestContext::new();

        // Register actor that expects String messages
        let actor_type = ActorType {
            name: "StringProcessor".to_string(),
            state_type: Type::Unit,
            message_types: vec![MessageType::simple("Process", vec![Type::String])],
            spawn_params: vec![],
        };

        ctx.register_actor("StringProcessor", actor_type);

        // Test: Sending wrong type should fail
        let invalid_send = SendExpr {
            receiver: Box::new(Expression::Identifier {
                name: "processor".to_string(),
                span: Span::new(0, 9),
            }),
            message: Box::new(Expression::FunctionCall {
                function: Box::new(Expression::Identifier {
                    name: "Process".to_string(),
                    span: Span::new(0, 7),
                }),
                args: vec![
                    Expression::IntLiteral(42), // Wrong type: should be String
                ],
                span: Span::new(0, 15),
            }),
            send_type: SendType::Fire,
            span: Span::new(0, 25),
        };

        let error = ctx.expect_type_error(ctx.type_check_send(&invalid_send));

        assert!(matches!(
            error.kind,
            TypeErrorKind::MessageTypeMismatch { .. }
        ));
        assert!(error.message.contains("Process"));
        assert!(error.message.contains("Int32"));
        assert!(error.message.contains("String"));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_request_response_type_safety() {
        let mut ctx = TypeTestContext::new();

        let calculator_type = ActorType {
            name: "Calculator".to_string(),
            state_type: Type::Unit,
            message_types: vec![
                MessageType {
                    name: "Add".to_string(),
                    params: vec![Type::Int32, Type::Int32],
                    return_type: Some(Type::Int32),
                    is_async: false,
                },
                MessageType {
                    name: "Divide".to_string(),
                    params: vec![Type::Int32, Type::Int32],
                    return_type: Some(Type::Result {
                        ok_type: Box::new(Type::Int32),
                        err_type: Box::new(Type::String),
                    }),
                    is_async: false,
                },
            ],
            spawn_params: vec![],
        };

        ctx.register_actor("Calculator", calculator_type);

        // Test: Request-response call should return correct type
        let request_response_send = SendExpr {
            receiver: Box::new(Expression::Identifier {
                name: "calculator".to_string(),
                span: Span::new(0, 10),
            }),
            message: Box::new(Expression::FunctionCall {
                function: Box::new(Expression::Identifier {
                    name: "Add".to_string(),
                    span: Span::new(0, 3),
                }),
                args: vec![Expression::IntLiteral(5), Expression::IntLiteral(3)],
                span: Span::new(0, 10),
            }),
            send_type: SendType::Call {
                timeout: Some(5000),
            },
            span: Span::new(0, 25),
        };

        let result_type = ctx.expect_type_success(ctx.type_check_send(&request_response_send));

        // Should return Future<Int32> for async call
        if let Type::Future(inner_type) = result_type {
            assert_eq!(*inner_type, Type::Int32);
        } else {
            panic!("Expected Future type for request-response send");
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_message_protocol_inheritance() {
        let mut ctx = TypeTestContext::new();

        let base_protocol = MessageProtocol {
            messages: vec![
                MessageType::simple("Start", vec![]),
                MessageType::simple("Stop", vec![]),
            ],
        };

        let extended_protocol = MessageProtocol {
            messages: vec![
                MessageType::simple("Start", vec![]),
                MessageType::simple("Stop", vec![]),
                MessageType::simple("Pause", vec![]),
                MessageType::simple("Resume", vec![]),
            ],
        };

        ctx.register_message_protocol("Controllable", base_protocol);
        ctx.register_message_protocol("ExtendedControllable", extended_protocol);

        // Test: Extended protocol should be compatible with base
        let is_compatible = ctx.type_checker.is_protocol_compatible(
            "ExtendedControllable",
            "Controllable",
            &ctx.context,
        );

        assert!(is_compatible);

        // Test: Base protocol should NOT be compatible with extended
        let is_reverse_compatible = ctx.type_checker.is_protocol_compatible(
            "Controllable",
            "ExtendedControllable",
            &ctx.context,
        );

        assert!(!is_reverse_compatible);
    }

    // =================================================================
    // PATTERN TYPE CHECKING TESTS (Receive Block Type Safety)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_pattern_type_checking_basic() {
        let mut ctx = TypeTestContext::new();

        // Define message types that can be received
        ctx.context.bind_type("Increment", Type::Unit);
        ctx.context.bind_type(
            "SetValue",
            Type::Function {
                params: vec![Type::Int32],
                return_type: Box::new(Type::Unit),
            },
        );

        let receive_block = ReceiveBlock {
            arms: vec![ReceiveArm {
                pattern: Pattern::Identifier {
                    name: "Increment".to_string(),
                    span: Span::new(0, 9),
                },
                guard: None,
                body: Expression::Assignment {
                    target: Box::new(Expression::FieldAccess {
                        object: Box::new(Expression::Identifier {
                            name: "self".to_string(),
                            span: Span::new(0, 4),
                        }),
                        field: "count".to_string(),
                        span: Span::new(0, 10),
                    }),
                    value: Box::new(Expression::BinaryOp {
                        op: BinaryOperator::Add,
                        left: Box::new(Expression::FieldAccess {
                            object: Box::new(Expression::Identifier {
                                name: "self".to_string(),
                                span: Span::new(0, 4),
                            }),
                            field: "count".to_string(),
                            span: Span::new(0, 10),
                        }),
                        right: Box::new(Expression::IntLiteral(1)),
                        span: Span::new(0, 15),
                    }),
                    span: Span::new(0, 20),
                },
                span: Span::new(0, 25),
            }],
            is_exhaustive: false,
            timeout: None,
            span: Span::new(0, 30),
        };

        let result = ctx
            .type_checker
            .type_check_receive_block(&receive_block, &mut ctx.context);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_pattern_destructuring_type_checking() {
        let mut ctx = TypeTestContext::new();

        // Define complex message type
        ctx.context.bind_type(
            "SendMessage",
            Type::Function {
                params: vec![
                    Type::String, // content
                    Type::ActorRef {
                        actor_type: Box::new(Type::Any),
                        capabilities: vec![SendCapability::Fire],
                    }, // sender
                ],
                return_type: Box::new(Type::Unit),
            },
        );

        let receive_block = ReceiveBlock {
            arms: vec![ReceiveArm {
                pattern: Pattern::FunctionCall {
                    name: "SendMessage".to_string(),
                    args: vec![
                        Pattern::Identifier {
                            name: "content".to_string(),
                            span: Span::new(0, 7),
                        },
                        Pattern::Identifier {
                            name: "sender".to_string(),
                            span: Span::new(0, 6),
                        },
                    ],
                    span: Span::new(0, 20),
                },
                guard: None,
                body: Expression::FunctionCall {
                    function: Box::new(Expression::Identifier {
                        name: "handle_message".to_string(),
                        span: Span::new(0, 14),
                    }),
                    args: vec![
                        Expression::Identifier {
                            name: "content".to_string(),
                            span: Span::new(0, 7),
                        },
                        Expression::Identifier {
                            name: "sender".to_string(),
                            span: Span::new(0, 6),
                        },
                    ],
                    span: Span::new(0, 25),
                },
                span: Span::new(0, 30),
            }],
            is_exhaustive: false,
            timeout: None,
            span: Span::new(0, 35),
        };

        let result = ctx
            .type_checker
            .type_check_receive_block(&receive_block, &mut ctx.context);
        assert!(result.is_ok());

        // Verify that bound variables have correct types in the body
        let arm_context = result.unwrap();
        assert_eq!(arm_context.get_type("content"), Some(&Type::String));
        assert!(matches!(
            arm_context.get_type("sender"),
            Some(Type::ActorRef { .. })
        ));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_pattern_guard_type_checking() {
        let mut ctx = TypeTestContext::new();

        ctx.context.bind_type(
            "SetValue",
            Type::Function {
                params: vec![Type::Int32],
                return_type: Box::new(Type::Unit),
            },
        );

        let receive_arm = ReceiveArm {
            pattern: Pattern::FunctionCall {
                name: "SetValue".to_string(),
                args: vec![Pattern::Identifier {
                    name: "value".to_string(),
                    span: Span::new(0, 5),
                }],
                span: Span::new(0, 12),
            },
            guard: Some(PatternGuard {
                condition: Expression::BinaryOp {
                    op: BinaryOperator::GreaterThan,
                    left: Box::new(Expression::Identifier {
                        name: "value".to_string(),
                        span: Span::new(0, 5),
                    }),
                    right: Box::new(Expression::IntLiteral(0)),
                    span: Span::new(0, 10),
                },
                span: Span::new(0, 10),
            }),
            body: Expression::Assignment {
                target: Box::new(Expression::FieldAccess {
                    object: Box::new(Expression::Identifier {
                        name: "self".to_string(),
                        span: Span::new(0, 4),
                    }),
                    field: "value".to_string(),
                    span: Span::new(0, 10),
                }),
                value: Box::new(Expression::Identifier {
                    name: "value".to_string(),
                    span: Span::new(0, 5),
                }),
                span: Span::new(0, 15),
            },
            span: Span::new(0, 20),
        };

        let result = ctx
            .type_checker
            .type_check_receive_arm(&receive_arm, &mut ctx.context);
        assert!(result.is_ok());

        // Guard condition should type check to bool
        if let Some(guard) = &receive_arm.guard {
            let guard_type = ctx.expect_type_success(
                ctx.type_checker
                    .infer_expression_type(&guard.condition, &mut ctx.context),
            );
            assert_eq!(guard_type, Type::Bool);
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_exhaustiveness_checking() {
        let mut ctx = TypeTestContext::new();

        // Define enum-like message types
        ctx.context.bind_type(
            "Command",
            Type::Sum {
                variants: vec![
                    ("Start".to_string(), vec![]),
                    ("Stop".to_string(), vec![]),
                    ("Pause".to_string(), vec![]),
                    ("Resume".to_string(), vec![]),
                ],
            },
        );

        // Test: Non-exhaustive pattern matching
        let non_exhaustive_receive = ReceiveBlock {
            arms: vec![
                ReceiveArm {
                    pattern: Pattern::Identifier {
                        name: "Start".to_string(),
                        span: Span::new(0, 5),
                    },
                    guard: None,
                    body: Expression::Unit,
                    span: Span::new(0, 10),
                },
                ReceiveArm {
                    pattern: Pattern::Identifier {
                        name: "Stop".to_string(),
                        span: Span::new(0, 4),
                    },
                    guard: None,
                    body: Expression::Unit,
                    span: Span::new(0, 8),
                },
            ],
            is_exhaustive: false,
            timeout: None,
            span: Span::new(0, 15),
        };

        let result = ctx
            .type_checker
            .check_exhaustiveness(&non_exhaustive_receive, &ctx.context);

        assert!(result.is_err());
        if let Err(error) = result {
            assert!(matches!(error.kind, TypeErrorKind::NonExhaustivePatterns));
            assert!(error.missing_patterns.contains(&"Pause".to_string()));
            assert!(error.missing_patterns.contains(&"Resume".to_string()));
        }

        // Test: Exhaustive pattern matching with wildcard
        let exhaustive_receive = ReceiveBlock {
            arms: vec![
                ReceiveArm {
                    pattern: Pattern::Identifier {
                        name: "Start".to_string(),
                        span: Span::new(0, 5),
                    },
                    guard: None,
                    body: Expression::Unit,
                    span: Span::new(0, 10),
                },
                ReceiveArm {
                    pattern: Pattern::Wildcard {
                        span: Span::new(0, 1),
                    },
                    guard: None,
                    body: Expression::Unit,
                    span: Span::new(0, 5),
                },
            ],
            is_exhaustive: true,
            timeout: None,
            span: Span::new(0, 10),
        };

        let result = ctx
            .type_checker
            .check_exhaustiveness(&exhaustive_receive, &ctx.context);
        assert!(result.is_ok());
    }

    // =================================================================
    // ACTOR INTRINSICS TYPE TESTS (Built-in Actor Operations)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_actor_self_intrinsic_typing() {
        let mut ctx = TypeTestContext::new();

        // Context: Inside ChatAgent actor
        let chat_agent_type = ActorType {
            name: "ChatAgent".to_string(),
            state_type: Type::Struct {
                name: "ChatAgentState".to_string(),
                fields: vec![("message_count".to_string(), Type::Int32)],
            },
            message_types: vec![],
            spawn_params: vec![],
        };

        ctx.context
            .bind_actor_context("ChatAgent", chat_agent_type.clone());

        // Test: self should have correct type
        let self_expr = Expression::Identifier {
            name: "self".to_string(),
            span: Span::new(0, 4),
        };

        let self_type = ctx.expect_type_success(ctx.type_check_expression(&self_expr));

        if let Type::ActorSelf(actor_type) = self_type {
            assert_eq!(actor_type.name, "ChatAgent");
        } else {
            panic!("Expected ActorSelf type for 'self'");
        }

        // Test: self field access
        let field_access = Expression::FieldAccess {
            object: Box::new(self_expr),
            field: "message_count".to_string(),
            span: Span::new(0, 17),
        };

        let field_type = ctx.expect_type_success(ctx.type_check_expression(&field_access));
        assert_eq!(field_type, Type::Int32);
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_spawn_intrinsic_typing() {
        let mut ctx = TypeTestContext::new();

        let counter_type = ActorType {
            name: "Counter".to_string(),
            state_type: Type::Struct {
                name: "CounterState".to_string(),
                fields: vec![("count".to_string(), Type::Int32)],
            },
            message_types: vec![],
            spawn_params: vec![Type::Int32], // Initial count
        };

        ctx.register_actor("Counter", counter_type.clone());

        // Test: spawn expression with correct arguments
        let spawn_expr = SpawnExpr {
            actor_type: "Counter".to_string(),
            args: vec![Expression::IntLiteral(0)],
            supervisor: None,
            spawn_options: SpawnOptions::default(),
            span: Span::new(0, 15),
        };

        let spawn_result_type = ctx.expect_type_success(
            ctx.type_checker
                .type_check_spawn(&spawn_expr, &mut ctx.context),
        );

        // Should return ActorRef<Counter>
        if let Type::ActorRef {
            actor_type,
            capabilities,
        } = spawn_result_type
        {
            if let Type::Actor(actor) = *actor_type {
                assert_eq!(actor.name, "Counter");
            }
            assert!(capabilities.contains(&SendCapability::Fire));
            assert!(capabilities.contains(&SendCapability::Call));
        } else {
            panic!("Expected ActorRef type from spawn");
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_spawn_argument_type_mismatch() {
        let mut ctx = TypeTestContext::new();

        let counter_type = ActorType {
            name: "Counter".to_string(),
            state_type: Type::Int32,
            message_types: vec![],
            spawn_params: vec![Type::Int32],
        };

        ctx.register_actor("Counter", counter_type);

        // Test: spawn with wrong argument type
        let invalid_spawn = SpawnExpr {
            actor_type: "Counter".to_string(),
            args: vec![Expression::StringLiteral {
                value: "not_a_number".to_string(),
                span: Span::new(0, 13),
            }],
            supervisor: None,
            spawn_options: SpawnOptions::default(),
            span: Span::new(0, 20),
        };

        let error = ctx.expect_type_error(
            ctx.type_checker
                .type_check_spawn(&invalid_spawn, &mut ctx.context),
        );

        assert!(matches!(
            error.kind,
            TypeErrorKind::ArgumentTypeMismatch { .. }
        ));
        assert!(error.message.contains("String"));
        assert!(error.message.contains("Int32"));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_actor_stop_intrinsic() {
        let mut ctx = TypeTestContext::new();

        // Context: Inside an actor
        let actor_type = ActorType {
            name: "TestActor".to_string(),
            state_type: Type::Unit,
            message_types: vec![],
            spawn_params: vec![],
        };

        ctx.context.bind_actor_context("TestActor", actor_type);

        // Test: self.stop() should be valid
        let stop_call = Expression::MethodCall {
            object: Box::new(Expression::Identifier {
                name: "self".to_string(),
                span: Span::new(0, 4),
            }),
            method: "stop".to_string(),
            args: vec![],
            span: Span::new(0, 11),
        };

        let result_type = ctx.expect_type_success(ctx.type_check_expression(&stop_call));
        assert_eq!(result_type, Type::Unit);
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_actor_reply_intrinsic() {
        let mut ctx = TypeTestContext::new();

        let chat_agent_type = ActorType {
            name: "ChatAgent".to_string(),
            state_type: Type::Unit,
            message_types: vec![MessageType {
                name: "GetStatus".to_string(),
                params: vec![],
                return_type: Some(Type::String),
                is_async: false,
            }],
            spawn_params: vec![],
        };

        ctx.context.bind_actor_context("ChatAgent", chat_agent_type);

        // Test: self.reply() in request-response context
        let reply_call = Expression::MethodCall {
            object: Box::new(Expression::Identifier {
                name: "self".to_string(),
                span: Span::new(0, 4),
            }),
            method: "reply".to_string(),
            args: vec![Expression::StringLiteral {
                value: "Active".to_string(),
                span: Span::new(0, 8),
            }],
            span: Span::new(0, 15),
        };

        let result_type = ctx.expect_type_success(ctx.type_check_expression(&reply_call));
        assert_eq!(result_type, Type::Unit);
    }

    // =================================================================
    // SUPERVISION TYPE TESTS (Fault Tolerance Typing)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_supervisor_type_checking() {
        let mut ctx = TypeTestContext::new();

        let supervisor_type = SupervisorType {
            name: "WorkerSupervisor".to_string(),
            strategy: RestartStrategy::OneForOne,
            child_types: vec![ChildType {
                id: "worker1".to_string(),
                actor_type: "Worker".to_string(),
                restart_policy: RestartPolicy::Permanent,
                capabilities: vec![SupervisionCapability::Restart, SupervisionCapability::Stop],
            }],
        };

        ctx.context.bind_type(
            "WorkerSupervisor",
            Type::Supervisor(supervisor_type.clone()),
        );

        // Test: Supervisor type should be well-formed
        assert!(ctx
            .type_checker
            .is_well_formed_type(&Type::Supervisor(supervisor_type), &ctx.context));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_supervision_capability_checking() {
        let mut ctx = TypeTestContext::new();

        let supervisor_ref_type = Type::SupervisorRef {
            supervisor_type: Box::new(Type::Supervisor(SupervisorType {
                name: "TestSupervisor".to_string(),
                strategy: RestartStrategy::OneForOne,
                child_types: vec![],
            })),
            capabilities: vec![
                SupervisionCapability::Restart,
                SupervisionCapability::Stop,
                SupervisionCapability::AddChild,
            ],
        };

        // Test: Supervisor should support restart operations
        assert!(ctx.type_checker.supports_restart(&supervisor_ref_type));
        assert!(ctx
            .type_checker
            .supports_child_management(&supervisor_ref_type));

        // Test: Regular actor ref should not support supervision
        let actor_ref_type = Type::ActorRef {
            actor_type: Box::new(Type::Any),
            capabilities: vec![SendCapability::Fire],
        };

        assert!(!ctx.type_checker.supports_restart(&actor_ref_type));
        assert!(!ctx.type_checker.supports_child_management(&actor_ref_type));
    }

    // =================================================================
    // TYPE INFERENCE TESTS (Automatic Type Deduction)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_message_type_inference() {
        let mut ctx = TypeTestContext::new();

        // Test: Infer message type from receive pattern
        let receive_arm = ReceiveArm {
            pattern: Pattern::FunctionCall {
                name: "ProcessData".to_string(),
                args: vec![Pattern::Identifier {
                    name: "data".to_string(),
                    span: Span::new(0, 4),
                }],
                span: Span::new(0, 15),
            },
            guard: None,
            body: Expression::FunctionCall {
                function: Box::new(Expression::Identifier {
                    name: "process".to_string(),
                    span: Span::new(0, 7),
                }),
                args: vec![Expression::Identifier {
                    name: "data".to_string(),
                    span: Span::new(0, 4),
                }],
                span: Span::new(0, 12),
            },
            span: Span::new(0, 20),
        };

        // Infer that ProcessData takes one parameter
        let inferred_message_type =
            ctx.expect_type_success(ctx.type_checker.infer_message_type_from_pattern(
                &receive_arm.pattern,
                &receive_arm.body,
                &mut ctx.context,
            ));

        if let Type::Function {
            params,
            return_type,
        } = inferred_message_type
        {
            assert_eq!(params.len(), 1);
            assert_eq!(*return_type, Type::Unit);
        } else {
            panic!("Expected function type for inferred message");
        }
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_actor_state_type_inference_from_usage() {
        let mut ctx = TypeTestContext::new();

        // Actor with untyped state parameters
        let actor_def = ActorDef {
            name: "InferredActor".to_string(),
            type_params: vec![],
            state_params: vec![StateParam {
                name: "counter".to_string(),
                param_type: Type::Unknown, // To be inferred
                default_value: Some(Expression::IntLiteral(0)),
                mutability: Mutability::Mutable,
                span: Span::new(0, 7),
            }],
            body: vec![ActorBodyItem::Receive(ReceiveBlock {
                arms: vec![ReceiveArm {
                    pattern: Pattern::Identifier {
                        name: "Increment".to_string(),
                        span: Span::new(0, 9),
                    },
                    guard: None,
                    body: Expression::Assignment {
                        target: Box::new(Expression::FieldAccess {
                            object: Box::new(Expression::Identifier {
                                name: "self".to_string(),
                                span: Span::new(0, 4),
                            }),
                            field: "counter".to_string(),
                            span: Span::new(0, 12),
                        }),
                        value: Box::new(Expression::BinaryOp {
                            op: BinaryOperator::Add,
                            left: Box::new(Expression::FieldAccess {
                                object: Box::new(Expression::Identifier {
                                    name: "self".to_string(),
                                    span: Span::new(0, 4),
                                }),
                                field: "counter".to_string(),
                                span: Span::new(0, 12),
                            }),
                            right: Box::new(Expression::IntLiteral(1)),
                            span: Span::new(0, 17),
                        }),
                        span: Span::new(0, 20),
                    },
                    span: Span::new(0, 25),
                }],
                is_exhaustive: false,
                timeout: None,
                span: Span::new(0, 30),
            })],
            span: Span::new(0, 40),
            visibility: Visibility::Public,
            attributes: vec![],
        };

        let inferred_type = ctx.expect_type_success(
            ctx.type_checker
                .infer_actor_type_with_usage_analysis(&actor_def, &mut ctx.context),
        );

        if let Type::Actor(actor_type) = inferred_type {
            if let Type::Struct { fields, .. } = actor_type.state_type {
                assert_eq!(fields[0].1, Type::Int32); // Inferred from arithmetic usage
            }
        }
    }

    // =================================================================
    // PROPERTY-BASED TYPE SYSTEM TESTS (Type Safety Invariants)
    // =================================================================

    proptest! {
        #[test]
        #[ignore] // EXTREME TDD: Property tests first, no implementation yet
        fn prop_message_send_preserves_type_safety(
            receiver_name in r"[a-z][a-zA-Z0-9_]*",
            message_name in r"[A-Z][a-zA-Z0-9_]*"
        ) {
            let mut ctx = TypeTestContext::new();

            // Property: Well-typed message sends should always type check
            let send_expr = SendExpr {
                receiver: Box::new(Expression::Identifier {
                    name: receiver_name.clone(),
                    span: Span::new(0, receiver_name.len()),
                }),
                message: Box::new(Expression::Identifier {
                    name: message_name.clone(),
                    span: Span::new(0, message_name.len()),
                }),
                send_type: SendType::Fire,
                span: Span::new(0, 20),
            };

            // Mock valid types in context
            ctx.context.bind_type(&receiver_name, Type::ActorRef {
                actor_type: Box::new(Type::Any),
                capabilities: vec![SendCapability::Fire],
            });
            ctx.context.bind_type(&message_name, Type::Unit);

            let result = ctx.type_check_send(&send_expr);
            prop_assert!(result.is_ok());
        }

        #[test]
        #[ignore] // EXTREME TDD: Property tests first, no implementation yet
        fn prop_actor_spawn_type_consistency(
            actor_name in r"[A-Z][a-zA-Z0-9_]*"
        ) {
            let mut ctx = TypeTestContext::new();

            let actor_type = ActorType {
                name: actor_name.clone(),
                state_type: Type::Unit,
                message_types: vec![],
                spawn_params: vec![],
            };

            ctx.register_actor(&actor_name, actor_type);

            let spawn_expr = SpawnExpr {
                actor_type: actor_name.clone(),
                args: vec![],
                supervisor: None,
                spawn_options: SpawnOptions::default(),
                span: Span::new(0, 15),
            };

            let result = ctx.type_checker.type_check_spawn(&spawn_expr, &mut ctx.context);
            prop_assert!(result.is_ok());

            if let Ok(Type::ActorRef { actor_type, .. }) = result {
                if let Type::Actor(spawned_actor) = *actor_type {
                    prop_assert_eq!(spawned_actor.name, actor_name);
                }
            }
        }

        #[test]
        #[ignore] // EXTREME TDD: Property tests first, no implementation yet
        fn prop_pattern_bindings_are_well_typed(
            var_names in prop::collection::vec(r"[a-z][a-zA-Z0-9_]*", 1..5)
        ) {
            let mut ctx = TypeTestContext::new();

            // Generate pattern with variable bindings
            let patterns: Vec<Pattern> = var_names.iter().map(|name| {
                Pattern::Identifier {
                    name: name.clone(),
                    span: Span::new(0, name.len()),
                }
            }).collect();

            // Property: Pattern bindings should introduce correctly typed variables
            for (i, pattern) in patterns.iter().enumerate() {
                if let Pattern::Identifier { name, .. } = pattern {
                    let binding_type = Type::TypeVar(TypeVar::new(i as u32));
                    ctx.context.bind_type(name, binding_type.clone());

                    let retrieved_type = ctx.context.get_type(name);
                    prop_assert_eq!(retrieved_type, Some(&binding_type));
                }
            }
        }

        #[test]
        #[ignore] // EXTREME TDD: Property tests first, no implementation yet
        fn prop_type_substitution_preserves_well_formedness(
            type_var_count in 1usize..5
        ) {
            let mut ctx = TypeTestContext::new();

            // Generate type with variables
            let type_vars: Vec<TypeVar> = (0..type_var_count)
                .map(|i| TypeVar::new(i as u32))
                .collect();

            let complex_type = Type::Function {
                params: type_vars.iter().map(|&var| Type::TypeVar(var)).collect(),
                return_type: Box::new(Type::TypeVar(type_vars[0])),
            };

            prop_assert!(ctx.type_checker.is_well_formed_type(&complex_type, &ctx.context));

            // Create substitution
            let mut substitution = Substitution::new();
            for &type_var in &type_vars {
                substitution.insert(type_var, Type::Int32);
            }

            // Apply substitution
            let substituted_type = ctx.type_checker.apply_substitution(&complex_type, &substitution);

            // Property: Substitution preserves well-formedness
            prop_assert!(ctx.type_checker.is_well_formed_type(&substituted_type, &ctx.context));

            // Property: All type variables should be eliminated
            prop_assert!(!ctx.type_checker.contains_type_variables(&substituted_type));
        }
    }

    // =================================================================
    // TYPE ERROR REPORTING TESTS (Error Quality and Recovery)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_detailed_type_error_messages() {
        let mut ctx = TypeTestContext::new();

        // Test: Detailed error for message type mismatch
        let invalid_send = SendExpr {
            receiver: Box::new(Expression::Identifier {
                name: "string_processor".to_string(),
                span: Span::new(5, 21),
            }),
            message: Box::new(Expression::FunctionCall {
                function: Box::new(Expression::Identifier {
                    name: "Process".to_string(),
                    span: Span::new(25, 32),
                }),
                args: vec![
                    Expression::IntLiteral(42), // Line 1, column 33-35
                ],
                span: Span::new(25, 38),
            }),
            send_type: SendType::Fire,
            span: Span::new(0, 40),
        };

        // Register actor that expects String
        ctx.register_actor(
            "StringProcessor",
            ActorType {
                name: "StringProcessor".to_string(),
                state_type: Type::Unit,
                message_types: vec![MessageType::simple("Process", vec![Type::String])],
                spawn_params: vec![],
            },
        );

        ctx.context.bind_type(
            "string_processor",
            Type::ActorRef {
                actor_type: Box::new(Type::Actor(ctx.actor_registry["StringProcessor"].clone())),
                capabilities: vec![SendCapability::Fire],
            },
        );

        let error = ctx.expect_type_error(ctx.type_check_send(&invalid_send));

        // Verify error has detailed information
        assert!(matches!(
            error.kind,
            TypeErrorKind::MessageTypeMismatch { .. }
        ));
        assert!(error.message.contains("Process"));
        assert!(error.message.contains("expected String"));
        assert!(error.message.contains("found Int32"));
        assert!(error.span == Span::new(25, 38)); // Points to message call
        assert!(error.help_text.is_some());
        assert!(error.help_text.as_ref().unwrap().contains("convert"));
    }

    #[test]
    #[ignore] // EXTREME TDD: Test first, no implementation yet
    fn test_type_error_recovery_suggestions() {
        let mut ctx = TypeTestContext::new();

        // Test various error scenarios and their recovery suggestions
        let error_cases = vec![
            (
                TypeErrorKind::UndefinedActor("ChatAgent".to_string()),
                vec![
                    "Did you mean 'ChatActor'?",
                    "Import the actor definition",
                    "Define the actor first",
                ],
            ),
            (
                TypeErrorKind::MessageTypeMismatch {
                    message_name: "SendMessage".to_string(),
                    expected: Type::String,
                    found: Type::Int32,
                },
                vec![
                    "Convert Int32 to String using .to_string()",
                    "Use a different message type",
                    "Check the message definition",
                ],
            ),
            (
                TypeErrorKind::NonExhaustivePatterns,
                vec![
                    "Add patterns for missing cases",
                    "Use a wildcard pattern (_)",
                    "Make the receive block exhaustive",
                ],
            ),
        ];

        for (error_kind, expected_suggestions) in error_cases {
            let suggestions = ctx
                .type_checker
                .get_error_recovery_suggestions(&error_kind, &ctx.context);

            for expected in expected_suggestions {
                assert!(suggestions.iter().any(|s| s.contains(expected)));
            }
        }
    }

    // =================================================================
    // TYPE SYSTEM PERFORMANCE TESTS (Inference Speed)
    // =================================================================

    #[test]
    #[ignore] // EXTREME TDD: Performance tests first, no implementation yet
    fn test_type_inference_performance() {
        use std::time::Instant;

        let mut ctx = TypeTestContext::new();

        // Create complex actor with many message types
        let complex_actor_type = ActorType {
            name: "ComplexActor".to_string(),
            state_type: Type::Struct {
                name: "ComplexState".to_string(),
                fields: (0..100)
                    .map(|i| (format!("field_{}", i), Type::Int32))
                    .collect(),
            },
            message_types: (0..100)
                .map(|i| {
                    MessageType::simple(&format!("Message_{}", i), vec![Type::Int32, Type::String])
                })
                .collect(),
            spawn_params: vec![],
        };

        ctx.register_actor("ComplexActor", complex_actor_type);

        // Generate large receive block
        let receive_arms: Vec<ReceiveArm> = (0..100)
            .map(|i| ReceiveArm {
                pattern: Pattern::FunctionCall {
                    name: format!("Message_{}", i),
                    args: vec![
                        Pattern::Identifier {
                            name: format!("param1_{}", i),
                            span: Span::new(0, 10),
                        },
                        Pattern::Identifier {
                            name: format!("param2_{}", i),
                            span: Span::new(0, 10),
                        },
                    ],
                    span: Span::new(0, 20),
                },
                guard: None,
                body: Expression::FunctionCall {
                    function: Box::new(Expression::Identifier {
                        name: format!("handler_{}", i),
                        span: Span::new(0, 10),
                    }),
                    args: vec![
                        Expression::Identifier {
                            name: format!("param1_{}", i),
                            span: Span::new(0, 10),
                        },
                        Expression::Identifier {
                            name: format!("param2_{}", i),
                            span: Span::new(0, 10),
                        },
                    ],
                    span: Span::new(0, 25),
                },
                span: Span::new(0, 30),
            })
            .collect();

        let large_receive_block = ReceiveBlock {
            arms: receive_arms,
            is_exhaustive: false,
            timeout: None,
            span: Span::new(0, 500),
        };

        let start = Instant::now();
        let result = ctx
            .type_checker
            .type_check_receive_block(&large_receive_block, &mut ctx.context);
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration.as_millis() < 1000); // Type check large receive block in <1s
    }
}
