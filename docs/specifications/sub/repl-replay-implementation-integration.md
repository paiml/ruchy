# Sub-spec: REPL Replay Testing — Implementation, Integration & Risk Mitigation

**Parent:** [repl-replay-testing-spec.md](../repl-replay-testing-spec.md) Sections 5-10

---
## Critical Design Decisions

### Semantic Equivalence Definition

The differential tester's `assert_semantic_eq!` requires precise specification:

```rust
impl SemanticEquivalence {
    fn equivalent(a: &Value, b: &Value) -> bool {
        match (a, b) {
            // Floating point with ULP tolerance
            (Value::Float(x), Value::Float(y)) => {
                (x - y).abs() < f64::EPSILON * x.abs().max(y.abs()) * 2.0
            }
            // Heap addresses ignored in comparison
            (Value::Ref(x), Value::Ref(y)) => {
                self.deref(x) == self.deref(y)
            }
            // HashMap iteration order ignored
            (Value::Map(x), Value::Map(y)) => {
                x.len() == y.len() && 
                x.iter().all(|(k, v)| y.get(k).map_or(false, |v2| 
                    Self::equivalent(v, v2)))
            }
            _ => a == b
        }
    }
}
```

### Observer Effect Mitigation

Recording overhead must be strictly bounded:

```rust
#[cfg(feature = "replay")]
macro_rules! record_event {
    ($session:expr, $event:expr) => {
        if $session.is_recording() {
            // Lazy evaluation - only serialize if recording
            $session.record($event);
        }
    };
}

// Zero-cost abstraction when not recording
#[cfg(not(feature = "replay"))]
macro_rules! record_event {
    ($session:expr, $event:expr) => {};
}

## Performance Requirements

- Session recording overhead: <5% CPU, <10MB RAM
- Replay validation: <2x original execution time
- Checkpoint/restore: <10ms for 1MB state
- Grading throughput: 100 submissions/second

### Version Migration Strategy

Session format evolution requires disciplined migration:

```rust
pub trait SessionMigration {
    fn can_migrate(&self, from: SemVer, to: SemVer) -> bool;
    fn migrate(&self, session: ReplSession) -> Result<ReplSession>;
}

impl SessionStore {
    fn load_with_migration(&self, path: &Path) -> Result<ReplSession> {
        let raw = self.load_raw(path)?;
        let version = raw.metadata.version;
        
        if version != CURRENT_VERSION {
            self.migrator.migrate_chain(raw, version, CURRENT_VERSION)
        } else {
            Ok(raw)
        }
    }
}
```

## Security Considerations

### Sandboxing for Educational Assessment
```rust
pub struct SecureSandbox {
    wasm_runtime: WasmRuntime,
    resource_limiter: ResourceLimiter,
    syscall_filter: SyscallFilter,
}

impl SecureSandbox {
    pub fn create_isolated_repl(&self) -> Repl {
        Repl::new()
            .with_runtime(self.wasm_runtime.clone())
            .with_resource_limits(self.resource_limiter.clone())
            .with_syscall_filter(self.syscall_filter.clone())
            .disable_network()
            .disable_filesystem()
            .disable_process_spawn()
    }
}
```

### Anti-Tampering Measures
- Cryptographic signing of session events
- Merkle tree for event causality verification
- Time-bound execution windows for assignments

## Integration with Existing Infrastructure

### CI/CD Pipeline Integration
```yaml
# .github/workflows/repl-tests.yml
name: REPL Replay Tests
on: [push, pull_request]

jobs:
  replay-golden:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run replay tests
        run: |
          cargo test --test replay_golden
          cargo test --test differential
          cargo test --test chaos
      - name: Performance regression check
        run: |
          cargo bench --bench repl_performance
          ./scripts/check_regression.sh
```

### Educational Platform Integration
```rust
// REST API for assignment submission
#[post("/submit/{assignment_id}")]
async fn submit_assignment(
    assignment_id: String,
    session: ReplSession,
    grading_service: &GradingService,
) -> Result<GradeReport> {
    let assignment = grading_service.load_assignment(&assignment_id)?;
    let report = grading_service.grade(assignment, session).await?;
    
    // Store for academic records
    grading_service.store_submission(report.clone()).await?;
    
    Ok(report)
}
```

## Future Extensions

### Advanced Educational Features
1. **Collaborative Sessions**: Multi-user REPL with operational transformation
2. **Hint Generation**: Automatic hint synthesis from successful submissions
3. **Learning Analytics**: Track common error patterns for curriculum improvement
4. **Adaptive Testing**: Difficulty adjustment based on student performance

### Testing Enhancements
1. **Symbolic Execution**: Path coverage for REPL command sequences
2. **Metamorphic Testing**: Property preservation across transformations
3. **Regression Visualization**: Visual diff of session execution traces
4. **Performance Profiling**: Integrated flamegraph generation per command

## Risk Mitigation Strategies

### Teaching to the Test Prevention
- Randomized test case generation per submission
- Property-based specification instead of fixed outputs
- Emphasis on performance and style metrics
- Manual code review sampling for high-stakes assessments

### Scope Management
- Begin with minimal viable replay (record/playback only)
- Defer educational features to v2.0
- Use existing property testing frameworks initially
- Leverage WASI for sandboxing instead of custom implementation

## References

- Jupyter nbval: Output-based notebook validation
- Kotlin REPL: Incremental compilation testing
- IPython ipytest: In-environment test execution
- AFL/LibFuzzer: Grammar-aware fuzzing techniques
- Lamport, Leslie: "Time, Clocks, and the Ordering of Events"
