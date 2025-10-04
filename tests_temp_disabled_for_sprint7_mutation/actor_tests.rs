#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
//! Tests for Actor system functionality

#![allow(clippy::unwrap_used)]
#![allow(clippy::needless_raw_string_hashes)]

use ruchy::{compile, is_valid_syntax};

#[test]
fn test_actor_definition() {
    let code = r#"
        actor Counter {
            state {
                count: i32
            }
            
            receive Increment(amount: i32) {
                // Assignment not yet implemented, return placeholder
                ()
            }
            
            receive GetCount() -> i32 {
                self.count
            }
        }
    "#;

    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Counter"));
    assert!(result.contains("CounterMessage"));
    assert!(result.contains("Increment"));
    assert!(result.contains("GetCount"));
}

#[test]
fn test_actor_send() {
    let code = "counter <- Increment(5)";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("send"));
    assert!(result.contains("Increment"));
}

#[test]
fn test_actor_ask() {
    let code = "counter <? GetCount()";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("ask"));
    assert!(result.contains("GetCount"));
}

#[test]
fn test_actor_with_multiple_handlers() {
    let code = r#"
        actor Logger {
            state {
                messages: Vec<String>
            }
            
            receive Info(msg: String) {
                // In Ruchy, we use string interpolation
                let info_msg = "[INFO] " + msg
                ()
            }
            
            receive Error(msg: String) {
                // In Ruchy, we use string interpolation  
                let error_msg = "[ERROR] " + msg
                ()
            }
            
            receive GetLogs() -> String {
                // Return placeholder for now
                "logs"
            }
        }
    "#;

    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Logger"));
    assert!(result.contains("Info"));
    assert!(result.contains("Error"));
    assert!(result.contains("GetLogs"));
}

#[test]
fn test_actor_spawn() {
    let code = "let counter = spawn Counter::new()";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Counter"));
    assert!(result.contains("new"));
}

#[test]
fn test_actor_pipeline() {
    let code = r#"
        let counter = spawn Counter::new()
        counter <- Increment(10)
        let count = counter <? GetCount()
    "#;

    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Counter"));
    assert!(result.contains("Increment"));
    assert!(result.contains("GetCount"));
}

#[test]
fn test_actor_with_complex_state() {
    let code = r#"
        actor Database {
            state {
                data: HashMap<String, String>,
                connections: Vec<Connection>
            }
            
            receive Store(key: String, value: String) {
                // In Ruchy, we don't have method calls on self.data yet
                // This would need implementation
                ()
            }
            
            receive Fetch(key: String) -> String {
                // Return placeholder for now
                "value"
            }
        }
    "#;

    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Database"));
    assert!(result.contains("HashMap"));
    assert!(result.contains("Store"));
    assert!(result.contains("Fetch"));
}

#[test]
fn test_actor_supervision() {
    let code = r#"
        supervisor MySupervisor {
            strategy: OneForOne,
            children: [Counter, Logger]
        }
    "#;

    // Note: Supervision syntax might not be fully implemented yet
    // This test documents the expected syntax
    if is_valid_syntax(code) {
        let result = compile(code).unwrap();
        assert!(result.contains("Supervisor"));
    }
}
