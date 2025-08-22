#![allow(clippy::unwrap_used, clippy::panic)]
//! Comprehensive tests for the Actor System implementation

#![allow(clippy::unwrap_used)] // Tests are allowed to unwrap
#![allow(clippy::expect_used)] // Tests are allowed to expect
#![allow(clippy::panic)] // Tests use panic for assertions

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::ast::ExprKind;
use ruchy::frontend::parser::Parser;

#[test]
fn test_parse_simple_actor() {
    let input = r"
        actor Counter {
            count: i32,
            
            receive {
                Increment => { 1 }
                Decrement => { 2 }
                GetCount => { 3 }
            }
        }
    ";

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse actor: {result:?}");
    let expr = result.unwrap();

    if let ExprKind::Actor {
        name,
        state,
        handlers,
    } = &expr.kind
    {
        assert_eq!(name, "Counter");
        assert_eq!(state.len(), 1);
        assert_eq!(state[0].name, "count");
        assert_eq!(handlers.len(), 3);
        assert_eq!(handlers[0].message_type, "Increment");
        assert_eq!(handlers[1].message_type, "Decrement");
        assert_eq!(handlers[2].message_type, "GetCount");
    } else {
        panic!("Expected Actor expression, got {:?}", expr.kind);
    }
}

#[test]
fn test_parse_actor_with_message_params() {
    let input = r"
        actor Calculator {
            result: f64,
            
            receive {
                Add(value: f64) => { 
                    value 
                }
                Multiply(factor: f64) => { 
                    factor 
                }
                Clear => { 
                    0.0 
                }
            }
        }
    ";

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Failed to parse actor with params: {result:?}"
    );
    let expr = result.unwrap();

    if let ExprKind::Actor { name, handlers, .. } = &expr.kind {
        assert_eq!(name, "Calculator");
        assert_eq!(handlers[0].params.len(), 1);
        assert_eq!(handlers[0].params[0].name(), "value");
        assert_eq!(handlers[1].params.len(), 1);
        assert_eq!(handlers[1].params[0].name(), "factor");
        assert_eq!(handlers[2].params.len(), 0);
    } else {
        panic!("Expected Actor expression");
    }
}

#[test]
fn test_parse_send_operation() {
    let input = "counter <- Increment";

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse send: {result:?}");
    let expr = result.unwrap();

    if let ExprKind::ActorSend { actor, message } = &expr.kind {
        if let ExprKind::Identifier(name) = &actor.kind {
            assert_eq!(name, "counter");
        } else {
            panic!("Expected actor to be identifier");
        }
        if let ExprKind::Identifier(msg) = &message.kind {
            assert_eq!(msg, "Increment");
        } else {
            panic!("Expected message to be identifier");
        }
    } else {
        panic!("Expected Send expression, got {:?}", expr.kind);
    }
}

#[test]
fn test_parse_ask_operation() {
    let input = "calculator <? GetResult";

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse ask: {result:?}");
    let expr = result.unwrap();

    if let ExprKind::ActorQuery {
        actor,
        message,
    } = &expr.kind
    {
        if let ExprKind::Identifier(name) = &actor.kind {
            assert_eq!(name, "calculator");
        } else {
            panic!("Expected actor to be identifier");
        }
        if let ExprKind::Identifier(msg) = &message.kind {
            assert_eq!(msg, "GetResult");
        } else {
            panic!("Expected message to be identifier");
        }
    } else {
        panic!("Expected ActorQuery expression, got {:?}", expr.kind);
    }
}

#[test]
fn test_transpile_actor() {
    let input = r"
        actor Counter {
            count: i32,
            
            receive {
                Increment => { 1 }
                GetCount => { 2 }
            }
        }
    ";

    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&expr);

    assert!(result.is_ok(), "Failed to transpile actor: {result:?}");
    let rust_code = result.unwrap().to_string();

    // Check that the generated code contains expected elements
    assert!(rust_code.contains("struct Counter"));
    assert!(rust_code.contains("enum CounterMessage"));
    assert!(rust_code.contains("Increment"));
    assert!(rust_code.contains("GetCount"));
    assert!(rust_code.contains("async fn handle_message"));
    assert!(rust_code.contains("tokio :: sync :: mpsc"));
}

#[test]
fn test_transpile_send() {
    let input = "counter <- Increment";

    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&expr);

    assert!(result.is_ok(), "Failed to transpile send: {result:?}");
    let rust_code = result.unwrap().to_string();

    assert!(rust_code.contains("counter"));
    assert!(rust_code.contains("send"));
    assert!(rust_code.contains("Increment"));
    assert!(rust_code.contains("await"));
}

#[test]
fn test_transpile_ask() {
    let input = "calculator <? GetResult";

    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&expr);

    assert!(result.is_ok(), "Failed to transpile ask: {result:?}");
    let rust_code = result.unwrap().to_string();

    assert!(rust_code.contains("calculator"));
    assert!(rust_code.contains("ask"));
    assert!(rust_code.contains("GetResult"));
    assert!(rust_code.contains("await"));
}

#[test]
fn test_actor_with_multiple_state_fields() {
    let input = r"
        actor BankAccount {
            balance: f64,
            owner: String,
            is_frozen: bool,
            
            receive {
                Deposit(amount: f64) => {
                    amount
                }
                Withdraw(amount: f64) => {
                    amount
                }
                Freeze => {
                    1.0
                }
                Unfreeze => {
                    0.0
                }
            }
        }
    ";

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse complex actor: {result:?}");
    let expr = result.unwrap();

    if let ExprKind::Actor {
        name,
        state,
        handlers,
    } = &expr.kind
    {
        assert_eq!(name, "BankAccount");
        assert_eq!(state.len(), 3);
        assert_eq!(handlers.len(), 4);

        // Verify state fields
        assert_eq!(state[0].name, "balance");
        assert_eq!(state[1].name, "owner");
        assert_eq!(state[2].name, "is_frozen");

        // Verify handlers
        assert_eq!(handlers[0].message_type, "Deposit");
        assert_eq!(handlers[0].params.len(), 1);
        assert_eq!(handlers[1].message_type, "Withdraw");
        assert_eq!(handlers[1].params.len(), 1);
        assert_eq!(handlers[2].message_type, "Freeze");
        assert_eq!(handlers[3].message_type, "Unfreeze");
    } else {
        panic!("Expected Actor expression");
    }
}
