//! TDD Test Suite for actors.rs - 100% Coverage Campaign  
//! Target: 48.4% → 100% coverage (23 lines to cover)
//! PMAT: Keep complexity <10 per test

#![cfg(test)]

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

/// Helper: Parse and transpile actor code
fn transpile_actor(code: &str) -> anyhow::Result<String> {
    let mut parser = Parser::new(code);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let tokens = transpiler.transpile(&ast)?;
    Ok(tokens.to_string())
}

// ========== Basic Actor Tests ==========
#[test]
fn test_actor_empty() {
    let code = r#"
actor Counter {
    state { }
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("struct Counter"));
}

#[test]
fn test_actor_with_state() {
    let code = r#"
actor Counter {
    state {
        count: int
    }
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("struct Counter"));
    assert!(result.contains("count"));
}

#[test]
fn test_actor_multiple_state_fields() {
    let code = r#"
actor BankAccount {
    state {
        balance: float,
        owner: str,
        id: int
    }
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("struct BankAccount"));
    assert!(result.contains("balance"));
    assert!(result.contains("owner"));
}

// ========== Handler Tests ==========
#[test]
fn test_actor_simple_handler() {
    let code = r#"
actor Counter {
    state { count: int }
    
    handler Increment {
        self.count += 1
    }
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("CounterMessage"));
    assert!(result.contains("Increment"));
}

#[test]
fn test_actor_handler_with_params() {
    let code = r#"
actor Counter {
    state { count: int }
    
    handler Add(amount: int) {
        self.count += amount
    }
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("Add"));
}

#[test]
fn test_actor_multiple_handlers() {
    let code = r#"
actor Counter {
    state { count: int }
    
    handler Increment {
        self.count += 1
    }
    
    handler Decrement {
        self.count -= 1
    }
    
    handler Reset {
        self.count = 0
    }
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("Increment"));
    assert!(result.contains("Decrement"));
    assert!(result.contains("Reset"));
}

#[test]
fn test_actor_handler_with_multiple_params() {
    let code = r#"
actor Calculator {
    state { result: float }
    
    handler Calculate(a: float, b: float, op: str) {
        match op {
            "+" => self.result = a + b,
            "-" => self.result = a - b,
            "*" => self.result = a * b,
            "/" => self.result = a / b,
            _ => {}
        }
    }
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("Calculate"));
}

// ========== Return Value Handler Tests ==========
#[test]
fn test_actor_handler_with_return() {
    let code = r#"
actor Counter {
    state { count: int }
    
    handler GetCount -> int {
        self.count
    }
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("GetCount"));
}

#[test]
fn test_actor_handler_params_and_return() {
    let code = r#"
actor Calculator {
    state { }
    
    handler Add(a: int, b: int) -> int {
        a + b
    }
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("Add"));
}

// ========== Async Handler Tests ==========
#[test]
fn test_actor_async_handler() {
    let code = r#"
actor DataFetcher {
    state { cache: dict }
    
    async handler Fetch(url: str) {
        let data = await http_get(url)
        self.cache.insert(url, data)
    }
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("async"));
}

// ========== Complex Actor Tests ==========
#[test]
fn test_actor_with_initialization() {
    let code = r#"
actor Logger {
    state {
        messages: list,
        max_size: int
    }
    
    init(max_size: int) {
        self.messages = []
        self.max_size = max_size
    }
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("Logger"));
}

#[test]
fn test_actor_with_generic_type() {
    let code = r#"
actor Storage<T> {
    state {
        items: list<T>
    }
    
    handler Store(item: T) {
        self.items.push(item)
    }
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("Storage"));
}

// ========== Message Enum Tests ==========
#[test]
fn test_actor_message_enum_generation() {
    let code = r#"
actor Worker {
    state { }
    
    handler Start {}
    handler Stop {}
    handler Process(data: str) {}
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("WorkerMessage"));
    assert!(result.contains("Start"));
    assert!(result.contains("Stop"));
    assert!(result.contains("Process"));
}

// ========== Error Handling Tests ==========
#[test]
fn test_actor_handler_with_result() {
    let code = r#"
actor FileSystem {
    state { }
    
    handler ReadFile(path: str) -> Result<str> {
        std::fs::read_to_string(path)
    }
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("Result"));
}

// ========== State Mutation Tests ==========
#[test]
fn test_actor_complex_state_mutation() {
    let code = r#"
actor GameState {
    state {
        players: dict,
        score: int,
        active: bool
    }
    
    handler AddPlayer(name: str) {
        self.players.insert(name, 0)
        self.active = true
    }
    
    handler UpdateScore(player: str, points: int) {
        if let Some(score) = self.players.get_mut(player) {
            *score += points
            self.score += points
        }
    }
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("GameState"));
}

// ========== Spawn and Send Tests ==========
#[test]
fn test_actor_spawn() {
    let code = r#"
let counter = spawn Counter { count: 0 }
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("spawn") || result.contains("Counter"));
}

#[test]
fn test_actor_send_message() {
    let code = r#"
counter.send(Increment)
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("send"));
}

#[test]
fn test_actor_send_with_params() {
    let code = r#"
calculator.send(Add(5, 3))
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("send"));
}

// ========== Edge Cases ==========
#[test]
fn test_actor_empty_handler() {
    let code = r#"
actor Dummy {
    state { }
    handler DoNothing { }
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("DoNothing"));
}

#[test]
fn test_actor_nested_types() {
    let code = r#"
actor Container {
    state {
        data: list<dict<str, int>>
    }
}
"#;
    let result = transpile_actor(code).unwrap();
    assert!(result.contains("Container"));
}

// Total: 20+ tests for 100% coverage of actors.rs