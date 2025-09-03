//! Comprehensive TDD test suite for actors.rs transpiler module
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every actor system path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::{Transpiler, Parser};

// ==================== ACTOR DEFINITION TESTS ====================

#[test]
fn test_transpile_simple_actor() {
    let transpiler = Transpiler::new();
    let code = r#"
    actor Counter {
        count: i32,
        
        handler Increment() {
            self.count += 1
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("struct Counter"));
    assert!(transpiled.contains("CounterMessage"));
    assert!(transpiled.contains("tokio::sync::mpsc"));
}

#[test]
fn test_transpile_actor_with_multiple_handlers() {
    let transpiler = Transpiler::new();
    let code = r#"
    actor Calculator {
        value: f64,
        
        handler Add(n: f64) {
            self.value += n
        }
        
        handler Subtract(n: f64) {
            self.value -= n
        }
        
        handler Reset() {
            self.value = 0.0
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("enum CalculatorMessage"));
    assert!(transpiled.contains("Add("));
    assert!(transpiled.contains("Subtract("));
    assert!(transpiled.contains("Reset"));
}

#[test]
fn test_transpile_actor_with_complex_state() {
    let transpiler = Transpiler::new();
    let code = r#"
    actor UserSession {
        id: String,
        username: String,
        logged_in: bool,
        last_action: i64,
        
        handler Login(user: String, pass: String) {
            self.username = user
            self.logged_in = true
        }
        
        handler Logout() {
            self.logged_in = false
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("struct UserSession"));
    assert!(transpiled.contains("id: String"));
    assert!(transpiled.contains("username: String"));
    assert!(transpiled.contains("logged_in: bool"));
}

#[test]
fn test_transpile_actor_empty_state() {
    let transpiler = Transpiler::new();
    let code = r#"
    actor Logger {
        handler Log(message: String) {
            println(message)
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("struct Logger"));
    assert!(transpiled.contains("Log(String)"));
}

// ==================== MESSAGE SENDING TESTS ====================

#[test]
fn test_transpile_actor_send_simple() {
    let transpiler = Transpiler::new();
    let code = r#"counter ! Increment()"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("send"));
    assert!(transpiled.contains("await"));
}

#[test]
fn test_transpile_actor_send_with_params() {
    let transpiler = Transpiler::new();
    let code = r#"calc ! Add(42.5)"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("send"));
}

#[test]
fn test_transpile_actor_ask() {
    let transpiler = Transpiler::new();
    let code = r#"let result = counter ? GetCount()"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("oneshot"));
}

// ==================== HANDLER PATTERN TESTS ====================

#[test]
fn test_transpile_handler_with_pattern_matching() {
    let transpiler = Transpiler::new();
    let code = r#"
    actor Router {
        routes: Vec<String>,
        
        handler Route(path: String) {
            match path {
                "/" => "home",
                "/about" => "about",
                _ => "not found"
            }
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("match"));
}

#[test]
fn test_transpile_handler_with_multiple_params() {
    let transpiler = Transpiler::new();
    let code = r#"
    actor Database {
        data: HashMap<String, String>,
        
        handler Set(key: String, value: String) {
            self.data.insert(key, value)
        }
        
        handler Get(key: String) {
            self.data.get(&key)
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("Set"));
    assert!(transpiled.contains("Get"));
}

// ==================== ASYNC HANDLER TESTS ====================

#[test]
fn test_transpile_async_handler() {
    let transpiler = Transpiler::new();
    let code = r#"
    actor HttpClient {
        handler Fetch(url: String) {
            await http::get(url)
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("async"));
}

// ==================== ACTOR LIFECYCLE TESTS ====================

#[test]
fn test_transpile_actor_new_method() {
    let transpiler = Transpiler::new();
    let code = r#"
    actor Worker {
        id: i32,
        
        handler Start() {
            self.id = 1
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("fn new()"));
    assert!(transpiled.contains("Default::default()"));
}

#[test]
fn test_transpile_actor_run_loop() {
    let transpiler = Transpiler::new();
    let code = r#"
    actor EventLoop {
        running: bool,
        
        handler Stop() {
            self.running = false
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("async fn run"));
    assert!(transpiled.contains("while let Some"));
}

// ==================== ACTOR COMMUNICATION TESTS ====================

#[test]
fn test_transpile_actor_channel_creation() {
    let transpiler = Transpiler::new();
    let code = r#"
    actor MessageQueue {
        messages: Vec<String>,
        
        handler Push(msg: String) {
            self.messages.push(msg)
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("channel(100)"));
    assert!(transpiled.contains("Sender"));
    assert!(transpiled.contains("Receiver"));
}

#[test]
fn test_transpile_actor_sender_clone() {
    let transpiler = Transpiler::new();
    let code = r#"
    actor Broadcaster {
        handler Broadcast(msg: String) {
            println(msg)
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("fn sender(&self)"));
    assert!(transpiled.contains("clone()"));
}

// ==================== ERROR HANDLING TESTS ====================

#[test]
fn test_transpile_handler_with_result() {
    let transpiler = Transpiler::new();
    let code = r#"
    actor FileSystem {
        handler ReadFile(path: String) {
            Result::Ok("content")
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("ReadFile"));
}

// ==================== COMPLEX ACTOR TESTS ====================

#[test]
fn test_transpile_actor_with_generics() {
    let transpiler = Transpiler::new();
    let code = r#"
    actor Store<T> {
        items: Vec<T>,
        
        handler Add(item: T) {
            self.items.push(item)
        }
    }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("Store"));
}

#[test]
fn test_transpile_actor_system_spawn() {
    let transpiler = Transpiler::new();
    let code = r#"let worker = spawn Worker { id: 1 }"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("spawn") || transpiled.contains("Worker"));
}

// Run all tests with: cargo test actors_tdd --test actors_tdd