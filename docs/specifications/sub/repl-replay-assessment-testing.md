# Sub-spec: REPL Replay Testing — Educational Assessment & Testing Methodologies

**Parent:** [repl-replay-testing-spec.md](../repl-replay-testing-spec.md) Sections 3-4

---
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

