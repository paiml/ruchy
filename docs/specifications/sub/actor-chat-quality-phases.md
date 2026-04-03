# Sub-spec: Actor Chat -- Quality, Implementation Phases, and Appendices

**Parent:** [demo-driven-actor-chat.md](../demo-driven-actor-chat.md) Sections 6-10

---

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
    p50: 10us
    p99: 100us
    
  message_send:
    p50: 100ns
    p99: 1us
    
  supervision_restart:
    p50: 50us
    p99: 500us
    
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
/// - Spawn: 10us
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
+-- unit/
|   +-- parser/
|   |   +-- actor_definition_test.rs
|   |   +-- receive_block_test.rs
|   |   +-- message_operators_test.rs
|   |   +-- supervision_hooks_test.rs
|   +-- typechecker/
|   |   +-- actor_ref_inference_test.rs
|   |   +-- message_type_safety_test.rs
|   |   +-- supervision_constraints_test.rs
|   +-- transpiler/
|       +-- actor_to_rust_test.rs
|       +-- supervision_generation_test.rs
|       +-- runtime_integration_test.rs
+-- integration/
|   +-- simple_actor_test.rs
|   +-- supervision_restart_test.rs
|   +-- message_ordering_test.rs
|   +-- chat_demo_test.rs
+-- property/
|   +-- message_delivery_props.rs
|   +-- supervision_props.rs
|   +-- isolation_props.rs
+-- benchmarks/
    +-- actor_spawn_bench.rs
    +-- message_throughput_bench.rs
    +-- supervision_overhead_bench.rs
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
