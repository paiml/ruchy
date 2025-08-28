# REPL Replay Testing and Educational Assessment Specification

## Executive Summary

REPL replay testing serves dual purposes: ensuring implementation correctness and enabling automated educational assessment. This specification defines a deterministic replay system that captures full session semantics, enabling both regression testing and student evaluation through identical infrastructure.

## Core Architecture

### 1. Session Recording Format

```rust
#[derive(Serialize, Deserialize)]
pub struct ReplSession {
    version: SemVer,
    metadata: SessionMetadata,
    environment: Environment,
    timeline: Vec<TimestampedEvent>,
    checkpoints: BTreeMap<EventId, StateCheckpoint>,
}

#[derive(Serialize, Deserialize)]
pub struct TimestampedEvent {
    id: EventId,
    timestamp_ns: u64,
    event: Event,
    causality: Vec<EventId>, // Lamport clock for distributed replay
}

#[derive(Serialize, Deserialize)]
pub enum Event {
    Input { 
        text: String,
        mode: InputMode, // Interactive vs Paste vs File
    },
    Output {
        result: EvalResult,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    StateChange {
        bindings_delta: HashMap<Ident, Value>,
        type_env_delta: TypeEnvDelta,
        compilation_cache_delta: CacheDelta,
    },
    ResourceUsage {
        heap_bytes: usize,
        stack_depth: usize,
        cpu_ns: u64,
    },
}
```

### 2. Deterministic Execution Model

```rust
pub trait DeterministicRepl {
    fn execute_with_seed(&mut self, input: &str, seed: u64) -> ReplayResult;
    
    fn checkpoint(&self) -> StateCheckpoint;
    
    fn restore(&mut self, checkpoint: &StateCheckpoint) -> Result<()>;
    
    fn validate_determinism(&self, other: &Self) -> ValidationResult;
}

impl DeterministicRepl for Repl {
    fn execute_with_seed(&mut self, input: &str, seed: u64) -> ReplayResult {
        // Fix all sources of non-determinism
        self.rng.seed(seed);
        self.time_source = MockTime::new();
        self.allocator = DeterministicAllocator::new();
        
        let result = self.execute(input);
        
        ReplayResult {
            output: result,
            state_hash: self.compute_hash(),
            resource_usage: self.measure_resources(),
        }
    }
}
```

### 3. Replay Validation Engine

```rust
pub struct ReplayValidator {
    strict_mode: bool,
    tolerance: ResourceTolerance,
}

impl ReplayValidator {
    pub fn validate_session(&self, 
        recorded: &ReplSession, 
        implementation: &mut impl DeterministicRepl
    ) -> ValidationReport {
        let mut report = ValidationReport::new();
        let mut last_checkpoint = StateCheckpoint::initial();
        
        for event in &recorded.timeline {
            match event.event {
                Event::Input { ref text, .. } => {
                    let result = implementation.execute_with_seed(text, 0);
                    
                    // Validate output equivalence
                    let expected = recorded.next_output(event.id);
                    if !self.outputs_equivalent(&result.output, &expected) {
                        report.add_divergence(event.id, Divergence::Output);
                    }
                    
                    // Validate resource bounds
                    if !self.tolerance.accepts(&result.resource_usage) {
                        report.add_divergence(event.id, Divergence::Resources);
                    }
                    
                    // Store checkpoint for rollback testing
                    if recorded.checkpoints.contains_key(&event.id) {
                        last_checkpoint = implementation.checkpoint();
                    }
                }
                _ => {}
            }
        }
        
        report
    }
}
```

### 4. Differential Testing Harness

```rust
pub struct DifferentialTester {
    reference: ReferenceRepl,
    optimized: OptimizedRepl,
    session_generator: SessionGenerator,
}

impl DifferentialTester {
    pub fn test_equivalence(&mut self, iterations: usize) -> TestReport {
        let mut report = TestReport::new();
        
        for _ in 0..iterations {
            let session = self.session_generator.generate();
            
            let ref_result = replay_on(&mut self.reference, &session);
            let opt_result = replay_on(&mut self.optimized, &session);
            
            // Semantic equivalence, not bit-identical
            assert_semantic_eq!(ref_result, opt_result);
            
            // Performance must not regress
            assert!(
                opt_result.total_ns <= ref_result.total_ns * 1.1,
                "Performance regression detected"
            );
        }
        
        report
    }
}
```

## Educational Assessment Framework

### 5. Assignment Specification Format

```rust
#[derive(Serialize, Deserialize)]
pub struct Assignment {
    id: AssignmentId,
    metadata: AssignmentMetadata,
    setup: SessionSetup,
    tasks: Vec<Task>,
    constraints: Constraints,
    rubric: Rubric,
}

#[derive(Serialize, Deserialize)]
pub struct Task {
    description: String,
    test_cases: Vec<TestCase>,
    hidden_cases: Vec<TestCase>, // Not shown to students
    performance_targets: Option<PerformanceTargets>,
}

#[derive(Serialize, Deserialize)]
pub struct TestCase {
    input: String,
    expected: ExpectedBehavior,
    points: u32,
}

#[derive(Serialize, Deserialize)]
pub enum ExpectedBehavior {
    ExactOutput(String),
    Pattern(Regex),
    Predicate(PredicateSpec),
    TypeSignature(Type),
    PerformanceBound {
        max_ns: u64,
        max_bytes: usize,
    },
}
```

### 6. Automated Grading Engine

```rust
pub struct GradingEngine {
    sandbox: SecureSandbox,
    replay_validator: ReplayValidator,
    plagiarism_detector: AstStructuralComparator,
}

impl GradingEngine {
    pub fn grade_submission(
        &mut self,
        assignment: &Assignment,
        submission: &ReplSession,
    ) -> GradeReport {
        let mut report = GradeReport::new();
        
        // Verify submission integrity
        if !self.verify_no_tampering(submission) {
            return report.mark_invalid("Session tampered");
        }
        
        // Setup assignment environment
        let mut repl = self.sandbox.create_isolated_repl();
        repl.load_setup(&assignment.setup);
        
        // Grade each task
        for task in &assignment.tasks {
            let task_grade = self.grade_task(&mut repl, task, submission);
            report.add_task_grade(task_grade);
            
            // Test hidden cases for academic integrity
            for hidden in &task.hidden_cases {
                let result = self.test_hidden_case(&mut repl, hidden);
                report.add_hidden_result(result);
            }
        }
        
        // Check performance requirements
        if let Some(perf) = &assignment.constraints.performance {
            report.performance_score = self.measure_performance(submission, perf);
        }
        
        // Detect plagiarism via AST structural comparison
        report.originality_score = self.plagiarism_detector.analyze(submission);
        
        report
    }
    
    fn grade_task(&mut self, 
        repl: &mut Repl, 
        task: &Task, 
        submission: &ReplSession
    ) -> TaskGrade {
        let mut grade = TaskGrade::new();
        
        for test in &task.test_cases {
            let student_output = repl.execute(&test.input);
            
            let points = match test.expected {
                ExpectedBehavior::ExactOutput(ref expected) => {
                    if student_output == *expected {
                        test.points
                    } else {
                        0
                    }
                }
                ExpectedBehavior::Pattern(ref regex) => {
                    if regex.is_match(&student_output) {
                        test.points
                    } else {
                        0
                    }
                }
                ExpectedBehavior::TypeSignature(ref ty) => {
                    if repl.type_of_last() == *ty {
                        test.points
                    } else {
                        0
                    }
                }
                _ => 0
            };
            
            grade.add_test_result(test, points);
        }
        
        grade
    }
}
```

### 7. Session Storage Format

```toml
# example_session.toml
[metadata]
version = "1.0.0"
ruchy_version = "1.0.0"
created_at = "2024-12-01T10:00:00Z"
student_id = "s123456"  # Optional, for educational use
assignment_id = "fp101_week3"  # Optional

[environment]
seed = 42
feature_flags = ["incremental_compilation", "type_inference_v2"]
resource_limits = { heap_mb = 100, stack_kb = 8192, cpu_ms = 5000 }

[[events]]
id = 1
timestamp_ns = 1000000
type = "input"
text = "let fibonacci = fun(n) -> if n <= 1 then n else fibonacci(n-1) + fibonacci(n-2)"
mode = "interactive"

[[events]]
id = 2
timestamp_ns = 1500000
type = "output"
result = { success = true }
stdout = ""
stderr = ""
state_hash = "a3f2b8c9d4e5f6789abcdef0123456789abcdef0"

[[events]]
id = 3
timestamp_ns = 2000000
type = "input"
text = "fibonacci(10)"
mode = "interactive"
depends_on = [1]

[[events]]
id = 4
timestamp_ns = 2100000
type = "output"
result = { value = "55" }
stdout = ""
stderr = ""
state_hash = "b4f3c9d0e5f6789abcdef0123456789abcdef012"
resource_usage = { heap_bytes = 4096, stack_depth = 12, cpu_ns = 95000 }

[[checkpoints]]
event_id = 2
bindings = { fibonacci = { type = "Function", arity = 1 } }
type_env = { fibonacci = "(Int) -> Int" }
```

## Testing Methodologies

### 8. Property-Based Replay Testing

```rust
#[proptest]
fn replay_preserves_semantics(
    session: ReplSession,
    permutation: Vec<usize>
) {
    // Pure computations yield identical results regardless of execution order
    let original = replay_deterministic(&session);
    
    let reordered = session.reorder_pure_computations(permutation);
    let reordered_result = replay_deterministic(&reordered);
    
    prop_assert_eq!(
        original.pure_outputs(),
        reordered_result.pure_outputs()
    );
}

#[proptest]
fn incremental_matches_batch(commands: Vec<String>) {
    let mut incremental_repl = Repl::new();
    let mut results_incremental = vec![];
    
    for cmd in &commands {
        results_incremental.push(incremental_repl.execute(cmd));
    }
    
    let batch_result = Repl::new().execute_batch(&commands);
    
    prop_assert_eq!(results_incremental, batch_result);
}
```

### 9. Mutation Testing for Error Recovery

```rust
#[mutate]
fn test_recovery_paths(session: &ReplSession) -> Result<()> {
    let mutator = SessionMutator::new();
    
    // Inject syntax errors
    let with_syntax_errors = mutator.inject_syntax_errors(session);
    assert!(can_recover_from(&with_syntax_errors));
    
    // Inject type errors
    let with_type_errors = mutator.inject_type_errors(session);
    assert!(can_recover_from(&with_type_errors));
    
    // Inject resource exhaustion
    let with_oom = mutator.inject_memory_exhaustion(session);
    assert!(can_recover_from(&with_oom));
    
    Ok(())
}
```

### 10. Chaos Engineering

```rust
pub struct ChaosRepl {
    inner: Repl,
    chaos_config: ChaosConfig,
}

impl ChaosRepl {
    pub fn inject_fault(&mut self) {
        match self.chaos_config.select_fault() {
            Fault::MemoryPressure => {
                self.inner.shrink_heap(0.5);
            }
            Fault::SlowCompilation => {
                thread::sleep(Duration::from_millis(500));
            }
            Fault::CacheInvalidation => {
                self.inner.invalidate_random_cache_entries(0.2);
            }
            Fault::NetworkPartition => {
                self.inner.disconnect_mcp();
            }
        }
    }
}

#[test]
fn survives_chaos() {
    let mut chaos_repl = ChaosRepl::new(ChaosConfig::aggressive());
    let session = load_golden_session("stress_test.toml");
    
    for event in session.events() {
        if chaos_repl.chaos_config.should_inject() {
            chaos_repl.inject_fault();
        }
        
        let result = chaos_repl.execute(event);
        assert!(result.is_recoverable());
    }
}
```

## Implementation Roadmap

### Phase 1: Core Infrastructure (Weeks 1-2)
- Implement `ReplSession` serialization format
- Build deterministic execution mode
- Create basic replay validator

### Phase 2: Testing Harness (Weeks 3-4)
- Property-based testing integration
- Differential testing framework
- Session mutation engine

### Phase 3: Educational Features (Weeks 5-6)
- Assignment specification DSL
- Automated grading engine
- Plagiarism detection via AST similarity

### Phase 4: Chaos Engineering (Weeks 7-8)
- Fault injection framework
- Resource exhaustion testing
- Recovery validation

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
- Cook, Byron et al.: "Proving Program Termination" (for symbolic execution)