# Demo-Driven Actor Chat Specification
## Ruchy Actor Concurrency for Agentic AI

### Executive Summary
This specification drives the next phase of Ruchy development through a concrete chatbot demo that showcases:
- **Actor-based concurrency** with supervision trees
- **MCP integration** for LLM communication
- **EXTREME-TDD** with 100% test-first development
- **Production quality** from day one

### Demo: Multi-Agent Chat System

```rust
// The demo that will sell Ruchy to the AI/autonomous dev community
actor ChatOrchestrator {
    agents: Map<AgentId, ActorRef<ChatAgent>>,
    supervisor: SupervisorRef,
    
    receive start_conversation(topic: String, participants: Vec<AgentType>) {
        // Spawn specialized agents under supervision
        // Note: supervisor.spawn_child creates supervised actors
        // Alternative: spawn_supervised creates actor with implicit supervisor
        let agents = participants.map(|type| {
            supervisor.spawn_child(ChatAgent::new(type))
        });
        
        // Broadcast topic to all agents
        for agent in agents {
            agent ! think_about(topic)
        }
        
        // Start conversation rounds
        self ! facilitate_round(1)
    }
    
    receive facilitate_round(n: i32) {
        // Collect thoughts from all agents (with timeout)
        let thoughts = agents.map(|agent| {
            agent ? get_thought() timeout 5s
        }).collect();
        
        // Synthesize responses using LLM or consensus algorithm
        // Returns Continue(insight) if discussion should proceed
        // Returns Conclude(summary) if consensus reached or max rounds
        match synthesize(thoughts) {
            Continue(insight) => {
                broadcast ! share_insight(insight);
                self ! facilitate_round(n + 1)
            },
            Conclude(summary) => {
                self ! end_conversation(summary)
            }
        }
    }
    
    receive agent_failed(id: AgentId, error: Error) {
        // Supervision: restart failed agent with backoff
        supervisor.restart_child(id, ExponentialBackoff(1s, 30s))
    }
}

actor ChatAgent {
    agent_type: AgentType,
    llm_connection: MCPConnection,
    memory: ConversationMemory,
    thinking: Option<String>,
    
    receive think_about(topic: String) {
        // Non-blocking LLM call via MCP
        // The !> operator sends a message and pipes the response as a new message
        // This allows async processing without blocking the actor's message loop
        let prompt = build_prompt(agent_type, topic, memory);
        llm_connection !> generate(prompt) |> store_thought
        // Equivalent to: llm_connection ! generate(prompt, reply_to: self.store_thought)
    }
    
    receive store_thought(response: LLMResponse) {
        self.thinking = Some(response.content);
        self.memory.add(response);
    }
    
    receive get_thought() -> String {
        self.thinking.take().unwrap_or("Still thinking...")
    }
    
    // Supervision hooks
    on_restart() {
        // Restore state from event log
        // Events are automatically logged for all state changes
        // self.id is an intrinsic property available to all actors
        self.memory = EventLog::replay_for(self.id);
    }
    
    on_child_failure(error: Error) {
        // Escalate to parent supervisor
        // parent is implicitly available to supervised actors
        parent ! agent_failed(self.id, error)
    }
}

// The demo in action
fn demo_autonomous_discussion() {
    let orchestrator = spawn ChatOrchestrator::new();
    
    orchestrator ! start_conversation(
        "How can we improve code review with AI?",
        vec![
            AgentType::Architect,    // Focuses on design
            AgentType::SecurityAuditor,  // Focuses on vulnerabilities  
            AgentType::TestEngineer,  // Focuses on coverage
            AgentType::Refactorer     // Focuses on clean code
        ]
    );
    
    // Agents discuss autonomously, supervised and fault-tolerant
}
```

## Key Language Features Clarification

### Actor Spawning Semantics
- **`spawn Actor::new()`** - Creates unsupervised actor
- **`spawn_supervised Actor::new()`** - Creates actor with implicit OneForOne supervisor
- **`supervisor.spawn_child(Actor::new())`** - Creates actor under explicit supervisor
- **`actor ? message retry N`** - Built-in retry logic with supervision

### Pipeline Operator (`!>` and `|>`)
The pipeline operator enables non-blocking async message flows:
```rust
// Sends message and routes response to another handler
actor !> async_operation() |> handle_result

// Transpiles to:
actor ! async_operation(reply_to: self.handle_result)
```

### Event Sourcing for State Recovery
- All state mutations automatically logged to event store
- `EventLog::replay_for(actor_id)` reconstructs state from events
- Configurable persistence backends (in-memory, disk, distributed)

### Intrinsic Actor Properties
Every actor has access to:
- `self.id` - Unique actor identifier  
- `parent` - Reference to supervisor (if supervised)
- `children` - List of supervised children (if supervisor)

### Synthesize Function
The `synthesize` function in the demo uses pluggable strategies:
- **LLM-based**: Sends all thoughts to LLM for insight extraction
- **Consensus**: Uses voting or agreement algorithms
- **Rule-based**: Applies domain-specific logic
Returns `Continue(insight)` to proceed or `Conclude(summary)` when done.

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

## Quality Specifications

### 1. Coverage Requirements

```yaml
coverage_requirements:
  overall: 95%
  
  per_module:
    parser/actor.rs: 100%  # Critical path
    typechecker/actor.rs: 100%  # Type safety
    transpiler/actor.rs: 100%  # Correctness
    runtime/supervision.rs: 95%  # Allow for edge cases
    
  mutation_testing:
    kill_rate: 90%  # Via mutants
    
  property_coverage:
    message_delivery: 100%
    supervision_restarts: 100%
    state_isolation: 100%
```

### 2. Performance Requirements

```yaml
performance_requirements:
  actor_spawn: 
    p50: 10µs
    p99: 100µs
    
  message_send:
    p50: 100ns
    p99: 1µs
    
  supervision_restart:
    p50: 50µs
    p99: 500µs
    
  memory_per_actor:
    idle: <1KB
    active: <10KB
    
  throughput:
    single_actor: 1M messages/sec
    actor_system: 10M messages/sec (aggregate)
```

### 3. Complexity Limits

```toml
[complexity_limits]
# Enforced by clippy and PMAT

[actor_implementation]
cyclomatic_complexity = 5  # Simple message handlers
cognitive_complexity = 8   # Understandable logic
nesting_depth = 3          # Flat structure

[supervision_tree]
max_depth = 5              # Manageable hierarchy
max_children = 100         # Reasonable fan-out
restart_strategy_complexity = 3  # Simple patterns
```

### 4. Documentation Requirements

Every actor component must have:

```rust
/// Counter actor maintains a mutable count.
/// 
/// # Example
/// ```
/// let counter = spawn Counter { value: 0 };
/// counter ! increment();
/// let value = counter ? get_value();
/// assert_eq!(value, 1);
/// ```
/// 
/// # Supervision
/// Restarts with initial value on failure.
/// 
/// # Performance
/// - Spawn: 10µs
/// - Message: 100ns
/// - Memory: 64 bytes
actor Counter {
    /// Current count value
    value: i32,
    
    /// Increment the counter by 1.
    /// Never fails.
    receive increment() {
        self.value += 1
    }
}
```

## Implementation Phases

### Week 1: Parser + AST (TDD)
1. Write all parser tests (Day 1)
2. Implement minimal parser to pass tests (Day 2-3)
3. Add AST nodes for actors (Day 4)
4. Parse supervision constructs (Day 5)
5. **Deliverable**: 100% parser test coverage

### Week 2: Type System (TDD)
1. Write type checking tests (Day 1)
2. Add ActorRef type (Day 2)
3. Message type validation (Day 3)
4. Supervision type constraints (Day 4)
5. **Deliverable**: Type-safe actor compilation

### Week 3: Transpiler (TDD)
1. Write transpilation tests (Day 1)
2. Generate Tokio boilerplate (Day 2-3)
3. Supervision tree generation (Day 4)
4. Runtime integration (Day 5)
5. **Deliverable**: Working actor execution

### Week 4: Quality + Demo (TDD)
1. Property test implementation (Day 1-2)
2. Performance benchmarks (Day 3)
3. Build chat demo (Day 4)
4. Polish and documentation (Day 5)
5. **Deliverable**: Production-ready demo

## Acceptance Criteria

### Functional Requirements
- [ ] Parse all actor syntax correctly
- [ ] Type check message passing
- [ ] Generate valid Rust code
- [ ] Execute with Tokio runtime
- [ ] Supervision tree restarts failed actors
- [ ] Chat demo runs for 1 hour without crashes

### Quality Requirements  
- [ ] 95% test coverage
- [ ] 90% mutation score
- [ ] All property tests pass
- [ ] Performance meets p99 targets
- [ ] Zero clippy warnings
- [ ] Zero SATD comments

### Demo Requirements
- [ ] 4 agents discuss autonomously
- [ ] Graceful handling of LLM timeouts
- [ ] Automatic restart on failures
- [ ] Real-time conversation flow
- [ ] MCP integration functional

## Risk Mitigation

### Technical Risks
1. **Tokio complexity**: Start with simplest runtime integration
2. **Supervision overhead**: Benchmark early, optimize later
3. **Type system complexity**: Use existing Hindley-Milner base

### Schedule Risks
1. **Scope creep**: Demo features frozen at spec time
2. **Test complexity**: Use property testing to reduce manual tests
3. **Performance issues**: Profile daily, fix immediately

## Success Metrics

### Demo Impact
- 1000+ GitHub stars within 1 month
- Featured in 3+ AI/Rust newsletters
- 10+ community contributors
- Used in 5+ production projects

### Code Quality
- A+ on code quality reports
- Featured as TDD case study
- Zero production bugs in first month
- Sub-second compile times

## Appendix A: Complete Test Suite Structure

```
tests/
├── unit/
│   ├── parser/
│   │   ├── actor_definition_test.rs
│   │   ├── receive_block_test.rs
│   │   ├── message_operators_test.rs
│   │   └── supervision_hooks_test.rs
│   ├── typechecker/
│   │   ├── actor_ref_inference_test.rs
│   │   ├── message_type_safety_test.rs
│   │   └── supervision_constraints_test.rs
│   └── transpiler/
│       ├── actor_to_rust_test.rs
│       ├── supervision_generation_test.rs
│       └── runtime_integration_test.rs
├── integration/
│   ├── simple_actor_test.rs
│   ├── supervision_restart_test.rs
│   ├── message_ordering_test.rs
│   └── chat_demo_test.rs
├── property/
│   ├── message_delivery_props.rs
│   ├── supervision_props.rs
│   └── isolation_props.rs
└── benchmarks/
    ├── actor_spawn_bench.rs
    ├── message_throughput_bench.rs
    └── supervision_overhead_bench.rs
```

## Appendix B: MCP Integration Points

```rust
// How actors integrate with MCP for LLM communication
actor LLMAgent {
    mcp_client: MCPConnection,
    
    #[mcp_tool("analyze_code")]
    receive analyze(code: String) -> Analysis {
        let prompt = format!("Analyze: {}", code);
        let response = self.mcp_client ? complete(prompt);
        Analysis::from(response)
    }
    
    #[mcp_subscribe("code_review_requested")]
    receive on_review_request(pr: PullRequest) {
        let analysis = self ! analyze(pr.diff);
        github ! post_comment(pr.id, analysis)
    }
}
```

## Appendix C: Supervision Patterns

```rust
// One-for-one: Restart only the failed child
supervisor OneForOne {
    strategy: RestartStrategy::OneForOne,
    max_restarts: 3,
    time_window: 60s,
}

// All-for-one: Restart all children if one fails
supervisor AllForOne {
    strategy: RestartStrategy::AllForOne,
    max_restarts: 1,
    time_window: 60s,
}

// Rest-for-one: Restart failed child and all started after it
supervisor RestForOne {
    strategy: RestartStrategy::RestForOne,
    max_restarts: 2,
    time_window: 60s,
}
```
