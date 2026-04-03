# Sub-spec: Demo-Driven Actor Chat — EXTREME-TDD Specification

**Parent:** [demo-driven-actor-chat.md](../demo-driven-actor-chat.md) Section 3

---
## EXTREME-TDD Specification

### Phase 1: Parser Tests (Write First, 0% Implementation)

```rust
#[test]
fn test_parse_actor_definition() {
    let input = "actor ChatAgent { state: String }";
    let ast = parse(input).unwrap();
    assert_matches!(ast, 
        Program::Actor(Actor {
            name: "ChatAgent",
            fields: vec![Field { name: "state", ty: Type::String }],
            ..
        })
    );
}

#[test]
fn test_parse_receive_block() {
    let input = r#"
        actor Agent {
            receive process(msg: String) {
                println(msg)
            }
        }
    "#;
    let ast = parse(input).unwrap();
    assert_eq!(ast.actor().receives().len(), 1);
    assert_eq!(ast.actor().receives()[0].name, "process");
}

#[test]
fn test_parse_send_operator() {
    let input = "agent ! message";
    let ast = parse_expr(input).unwrap();
    assert_matches!(ast, 
        Expr::Send { 
            actor: box Expr::Ident("agent"),
            message: box Expr::Ident("message")
        }
    );
}

#[test]
fn test_parse_ask_operator_with_timeout() {
    let input = "agent ? get_state() timeout 5s";
    let ast = parse_expr(input).unwrap();
    assert_matches!(ast,
        Expr::Ask {
            actor: _,
            message: _,
            timeout: Some(Duration::Seconds(5))
        }
    );
}

#[test]
fn test_parse_supervision_hooks() {
    let input = r#"
        actor Supervised {
            on_restart() {
                self.reinitialize()
            }
            on_child_failure(error: Error) {
                log_error(error)
            }
        }
    "#;
    let ast = parse(input).unwrap();
    assert!(ast.actor().has_hook("on_restart"));
    assert!(ast.actor().has_hook("on_child_failure"));
}
```

### Phase 2: Type System Tests (Isolated Components)

```rust
#[test]
fn test_actor_ref_type_inference() {
    let program = r#"
        actor Worker {
            receive work(task: String) -> Result<String, Error>
        }
        
        fn main() {
            let worker = spawn Worker::new();  // Inferred: ActorRef<Worker>
            let result = worker ? work("task"); // Inferred: Future<Result<String, Error>>
        }
    "#;
    
    let typed = type_check(program).unwrap();
    assert_eq!(
        typed.get_type("worker"),
        Type::ActorRef(box Type::Actor("Worker"))
    );
    assert_eq!(
        typed.get_type("result"),
        Type::Future(box Type::Result(
            box Type::String,
            box Type::Error
        ))
    );
}

#[test]
fn test_message_type_safety() {
    let program = r#"
        actor TypedActor {
            receive process(n: i32) -> String
        }
        
        fn main() {
            let actor = spawn TypedActor::new();
            actor ! process("wrong");  // Type error: expected i32, found String
        }
    "#;
    
    let result = type_check(program);
    assert_matches!(result, 
        Err(TypeError::MessageTypeMismatch { 
            expected: Type::I32, 
            found: Type::String,
            ..
        })
    );
}
```

### Phase 3: Transpiler Tests (Rust Code Generation)

```rust
#[test]
fn test_actor_transpiles_to_rust_struct_with_tokio() {
    let ruchy_code = r#"
        actor SimpleActor {
            count: i32,
            receive increment() {
                self.count += 1
            }
        }
    "#;
    
    let rust_code = transpile(ruchy_code).unwrap();
    
    assert_contains!(rust_code, "use tokio::sync::mpsc;");
    assert_contains!(rust_code, "struct SimpleActor {");
    assert_contains!(rust_code, "    count: i32,");
    assert_contains!(rust_code, "enum SimpleActorMessage {");
    assert_contains!(rust_code, "    Increment,");
    assert_contains!(rust_code, "impl SimpleActor {");
    assert_contains!(rust_code, "    async fn handle_message(&mut self");
}

#[test]
fn test_supervision_generates_restart_logic() {
    let ruchy_code = r#"
        actor Supervised {
            on_restart() {
                self.state = initial_state()
            }
        }
    "#;
    
    let rust_code = transpile(ruchy_code).unwrap();
    
    assert_contains!(rust_code, "trait Supervisable {");
    assert_contains!(rust_code, "    fn on_restart(&mut self);");
    assert_contains!(rust_code, "impl Supervisable for Supervised {");
    assert_contains!(rust_code, "    fn on_restart(&mut self) {");
    assert_contains!(rust_code, "        self.state = initial_state()");
}
```

### Phase 4: Integration Tests (End-to-End Behavior)

```rust
#[test]
async fn test_actor_actually_processes_messages() {
    let program = r#"
        actor Counter {
            value: i32,
            
            receive increment() {
                self.value += 1
            }
            
            receive get() -> i32 {
                self.value
            }
        }
        
        async fn test_counter() -> i32 {
            let counter = spawn Counter { value: 0 };
            counter ! increment();
            counter ! increment();
            counter ? get()
        }
    "#;
    
    let result = compile_and_run(program).await;
    assert_eq!(result, Value::Int(2));
}

#[test]
async fn test_supervisor_restarts_failed_actor() {
    let program = r#"
        actor Unreliable {
            attempts: i32,
            
            receive risky_operation() -> Result<String, Error> {
                self.attempts += 1;
                if self.attempts < 3 {
                    panic!("Simulated failure")
                }
                Ok("Success on third try")
            }
            
            on_restart() {
                // Don't reset attempts - we want to eventually succeed
            }
        }
        
        async fn test_supervision() -> String {
            // spawn_supervised creates actor with implicit supervisor
            // Retry mechanism built into ask operator when supervised
            let actor = spawn_supervised Unreliable { attempts: 0 };
            // Should fail twice, restart, then succeed
            actor ? risky_operation() retry 3
        }
    "#;
    
    let result = compile_and_run(program).await;
    assert_eq!(result, Value::String("Success on third try"));
}
```

### Phase 5: Property-Based Tests (Correctness Invariants)

```rust
#[property]
fn actor_message_ordering_preserved(messages: Vec<TestMessage>) {
    // Property: Messages from single sender arrive in order
    let actor = spawn TestActor::new();
    let sent_order = messages.clone();
    
    for msg in messages {
        actor ! msg;
    }
    
    let received_order = actor ? get_all_received();
    assert_eq!(sent_order, received_order);
}

#[property]
fn supervision_tree_never_loses_messages(
    messages: Vec<TestMessage>,
    failure_points: Vec<usize>
) {
    // Property: Every sent message is eventually processed
    let supervisor = spawn_supervisor();
    let actor = supervisor.spawn_child(TestActor::new());
    
    for (i, msg) in messages.iter().enumerate() {
        if failure_points.contains(&i) {
            actor ! trigger_failure();
        }
        actor ! msg.clone();
    }
    
    thread::sleep(Duration::from_secs(1));
    
    let processed = actor ? get_processed_messages();
    assert_eq!(messages.len(), processed.len());
    assert_eq!(messages.to_set(), processed.to_set());
}

#[property]
fn actor_state_isolation(
    actor_count: u8,  // 1-255 actors
    operations_per_actor: Vec<Operation>
) {
    // Property: Actor state changes don't affect other actors
    let actors: Vec<_> = (0..actor_count)
        .map(|_| spawn IsolatedActor::new())
        .collect();
    
    // Parallel operations
    actors.par_iter().zip(operations_per_actor).for_each(|(actor, ops)| {
        for op in ops {
            actor ! op;
        }
    });
    
    // Each actor should only reflect its own operations
    for (i, actor) in actors.iter().enumerate() {
        let state = actor ? get_state();
        assert_eq!(state, expected_state_for(operations_per_actor[i]));
    }
}
```

