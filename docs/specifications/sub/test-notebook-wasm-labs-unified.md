# Sub-spec: Test Notebook Framework — WASM Educational Labs and Unified Architecture

**Parent:** [ruchy-test-notebook-framework.md](../ruchy-test-notebook-framework.md) Sections 12-15

---

## WASM Educational Labs

### Lab Notebook Architecture

Educational notebooks run in WASM sandbox with progressive test disclosure and automated feedback.

```rust
#[derive(Serialize, Deserialize)]
struct LabNotebook {
    instructions: Vec<MarkdownCell>,
    exercises: Vec<Exercise>,
    hidden_tests: Vec<HiddenTest>,
    grading_rubric: GradingRubric,
}

struct Exercise {
    id: ExerciseId,
    prompt: String,
    starter_code: String,
    visible_tests: Vec<TestCase>,
    hidden_tests: Vec<TestCase>,
    hints: Vec<Hint>,
    max_attempts: Option<u32>,
}

struct TestCase {
    input: TestInput,
    expected: ExpectedOutput,
    feedback: FeedbackStrategy,
    points: u32,
}

enum FeedbackStrategy {
    Immediate(String),
    Differential { show_expected: bool, show_diff: bool },
    Hints { progressive: Vec<String> },
    Adaptive { based_on: ErrorPattern },
}
```

### WASM Sandbox Execution

```rust
impl WasmLabRunner {
    pub fn run_exercise(&self, submission: &str, exercise: &Exercise) -> LabResult {
        // Compile to WASM with restricted capabilities
        let module = self.compile_sandboxed(submission)?;
        
        // Resource limits for student code
        let limits = ResourceLimits {
            max_memory: 64 * MB,
            max_execution_time: Duration::from_secs(5),
            max_stack_depth: 1000,
            allowed_imports: &["std", "ruchy_lab"],  // No filesystem/network
        };
        
        let sandbox = WasmSandbox::new(module, limits);
        
        // Run visible tests first
        let visible_results = self.run_test_suite(&sandbox, &exercise.visible_tests);
        
        // Run hidden tests only if visible pass
        let hidden_results = if visible_results.all_pass() {
            self.run_test_suite(&sandbox, &exercise.hidden_tests)
        } else {
            TestResults::Skipped
        };
        
        // Generate feedback based on results
        let feedback = self.generate_feedback(&visible_results, &hidden_results, exercise);
        
        LabResult {
            score: self.calculate_score(&visible_results, &hidden_results),
            feedback,
            next_hint: self.select_hint(&visible_results, exercise),
        }
    }
}
```

### Automated Feedback Generation

```rust
impl FeedbackGenerator {
    fn generate_feedback(&self, results: &TestResults, exercise: &Exercise) -> Feedback {
        match self.classify_error(results) {
            ErrorClass::Syntax(err) => {
                Feedback::Immediate(format!(
                    "Syntax error at line {}: {}. Try checking your parentheses.",
                    err.line, err.message
                ))
            }
            ErrorClass::TypeError(expected, actual) => {
                Feedback::Differential {
                    message: format!("Type mismatch: expected {}, got {}", expected, actual),
                    suggestion: self.suggest_type_fix(expected, actual),
                }
            }
            ErrorClass::LogicError(pattern) => {
                // Adaptive feedback based on common mistakes
                let hint = self.mistake_database.lookup(pattern);
                Feedback::Adaptive {
                    explanation: hint.explanation,
                    example: hint.counter_example,
                }
            }
            ErrorClass::Performance(metric) => {
                Feedback::Performance {
                    your_time: metric.execution_time,
                    target_time: exercise.performance_target,
                    suggestion: "Consider using a more efficient data structure",
                }
            }
        }
    }
}
```

### Progressive Hint System

```rust
impl HintEngine {
    fn select_hint(&self, attempt: u32, error_pattern: &ErrorPattern) -> Option<Hint> {
        // Start with high-level hints, progressively more specific
        let hint_level = match attempt {
            1..=2 => HintLevel::Conceptual,
            3..=4 => HintLevel::Structural,
            5..=6 => HintLevel::Specific,
            _ => HintLevel::Solution,
        };
        
        self.hints
            .iter()
            .filter(|h| h.level == hint_level && h.matches(error_pattern))
            .next()
            .cloned()
    }
}

struct Hint {
    level: HintLevel,
    text: String,
    example: Option<String>,
    related_concept: Option<ConceptLink>,
}
```

### Anti-Cheating Measures

```rust
impl SubmissionValidator {
    fn validate_submission(&self, submission: &Submission) -> ValidationResult {
        // Structural similarity detection
        if self.detect_plagiarism(submission) > SIMILARITY_THRESHOLD {
            return ValidationResult::SuspiciousSimilarity;
        }
        
        // Timing analysis
        if submission.time_between_attempts < MIN_ATTEMPT_INTERVAL {
            return ValidationResult::TooFastSubmission;
        }
        
        // Solution pattern matching
        if self.matches_known_solution(submission) {
            // Require explanation
            return ValidationResult::RequiresExplanation;
        }
        
        ValidationResult::Valid
    }
    
    fn generate_unique_exercise(&self, student_id: &str, template: &ExerciseTemplate) -> Exercise {
        // Parameterized problems with student-specific values
        let seed = self.hash_student_id(student_id);
        let mut rng = StdRng::seed_from_u64(seed);
        
        Exercise {
            starter_code: template.generate_variant(&mut rng),
            test_inputs: template.generate_test_inputs(&mut rng),
            ..template.base_exercise()
        }
    }
}
```

### Lab Authoring DSL

```rust
// instructor.ruchy - Define labs with simple DSL
#[lab(title = "Introduction to DataFrames")]
mod dataframe_lab {
    #[exercise(points = 10)]
    fn load_and_filter() -> Exercise {
        prompt: "Load the iris dataset and filter for sepal_length > 5.0"
        
        starter: ```
        let df = // Your code here
        df.filter(|row| // Complete the filter)
        ```
        
        test_visible: {
            assert_eq!(result.len(), 118)
        }
        
        test_hidden: {
            assert!(result.columns().contains("sepal_length"))
            assert!(result["sepal_length"].min() >= 5.0)
        }
        
        hint[1]: "Use read_csv() to load the data"
        hint[2]: "The filter closure receives each row"
        hint[3]: "Access sepal_length with row['sepal_length']"
    }
}
```

### Learning Analytics

```rust
impl AnalyticsCollector {
    fn track_learning_progress(&self, attempt: &Attempt) -> LearningMetrics {
        LearningMetrics {
            error_evolution: self.analyze_error_patterns(attempt),
            concept_mastery: self.estimate_understanding(attempt),
            time_to_solution: attempt.duration,
            hint_effectiveness: self.measure_hint_impact(attempt),
            struggle_points: self.identify_confusion_areas(attempt),
        }
    }
    
    fn generate_instructor_report(&self, cohort: &[StudentProgress]) -> Report {
        Report {
            common_misconceptions: self.cluster_errors(cohort),
            difficulty_ranking: self.rank_exercises(cohort),
            recommended_interventions: self.suggest_curriculum_changes(cohort),
        }
    }
}
```

### Deployment Configuration

```yaml
# lab-config.yaml
lab:
  mode: coursera_compatible
  grading:
    visible_tests_weight: 0.4
    hidden_tests_weight: 0.6
    max_attempts: unlimited
    late_penalty: 0.1_per_day
  
  sandbox:
    wasm_memory_limit: 64MB
    execution_timeout: 5s
    allowed_imports:
      - ruchy_std
      - ruchy_lab
    blocked_features:
      - filesystem
      - network
      - unsafe
  
  feedback:
    immediate: true
    show_test_cases: visible_only
    hints_after_attempts: 2
```

## Unified Testing Architecture

### Execution Pipeline

All testing modes share a common execution pipeline with progressive enhancement.

```rust
pub struct UnifiedTestRunner {
    basic: BasicValidator,      // Always runs
    property: PropertyTester,    // User opt-in
    formal: FormalVerifier,     // Advanced users + compiler team
    differential: BackendVerifier, // Compiler canaries
}

impl UnifiedTestRunner {
    pub fn execute(&self, notebook: &Notebook, config: &TestConfig) -> TestResults {
        // Base validation for all users
        let mut results = self.basic.validate(notebook)?;
        
        // Progressive enhancement based on annotations
        for cell in notebook.cells() {
            match cell.test_type() {
                TestType::Assert => {
                    // Simple assertion, already handled
                }
                TestType::Property if config.enable_property => {
                    results.merge(self.property.test_cell(cell)?);
                }
                TestType::Prove if config.enable_smt => {
                    results.merge(self.formal.verify_cell(cell)?);
                }
                TestType::Complexity => {
                    results.merge(self.analyze_complexity(cell)?);
                }
                TestType::Canary => {
                    // Internal compiler testing
                    results.merge(self.differential.test_backends(cell)?);
                }
            }
        }
        
        results
    }
}
```

### Pragmatic SMT Integration

Formal verification with escape hatches for tractability.

```rust
impl FormalVerifier {
    pub fn verify_cell(&self, cell: &Cell) -> VerificationResult {
        // Quick syntactic check first
        if !self.has_tractable_structure(cell) {
            return VerificationResult::Skipped("Complex control flow");
        }
        
        // Bounded verification for practicality
        let config = SmtConfig {
            timeout: Duration::from_secs(5),
            max_iterations: 100,
            abstraction_level: AbstractionLevel::Linear,
        };
        
        match self.smt_engine.verify_with_config(cell, config) {
            Ok(proof) => VerificationResult::Proved(proof),
            Err(Timeout) => VerificationResult::BoundedSafe(100),
            Err(CounterExample(cex)) => VerificationResult::Failed(cex),
        }
    }
    
    fn has_tractable_structure(&self, cell: &Cell) -> bool {
        // Limit to functions we can reasonably verify
        cell.cyclomatic_complexity() < 10 &&
        !cell.has_recursion() &&
        cell.loop_depth() <= 2
    }
}
```

### Incremental Complexity Analysis

Static analysis with empirical validation fallback.

```rust
impl ComplexityAnalyzer {
    pub fn analyze(&self, func: &Function) -> ComplexityResult {
        // Try static analysis first (fast path)
        if let Some(recurrence) = self.extract_simple_recurrence(func) {
            return self.solve_closed_form(recurrence);
        }
        
        // Fall back to empirical analysis
        let samples = self.benchmark_with_sizes(&[10, 100, 1000, 10000]);
        let fitted = self.fit_curve(samples);
        
        // Validate claimed complexity if present
        if let Some(claimed) = func.complexity_annotation() {
            let verified = fitted.matches(claimed, tolerance = 0.1);
            ComplexityResult::Empirical { fitted, verified }
        } else {
            ComplexityResult::Inferred(fitted)
        }
    }
}
```

### Runtime Acceptance Testing Integration

Notebooks as compiler test harness with minimal overhead.

```rust
#[cfg(test)]
mod compiler_acceptance {
    use ruchy_test::canary::*;
    
    #[canary_test]
    fn type_system_soundness() {
        let notebook = include_notebook!("canaries/types.ruchynb");
        
        // Execute on all backends in parallel
        let results = notebook.run_differential(&[
            Backend::Interpreter,
            Backend::JIT,
            Backend::Transpiled,
            Backend::WASM,
        ]);
        
        // Semantic equivalence is non-negotiable
        assert!(results.all_equivalent());
        
        // Performance bounds
        assert!(results.jit_speedup() > 10.0);
        assert!(results.transpiled_speedup() > 50.0);
    }
}
```

### WASM Lab Deployment

Production configuration for educational deployment.

```rust
pub struct LabDeployment {
    cdn_url: Url,  // Static WASM modules
    grading_api: Url,  // Submission processing
    analytics_sink: Url,  // Learning metrics
}

impl LabDeployment {
    pub async fn process_submission(&self, submission: Submission) -> GradingResult {
        // Compile to WASM with deterministic build
        let wasm = self.compile_deterministic(submission.code)?;
        
        // Execute in isolated worker
        let worker = Worker::spawn(wasm, ResourceLimits::educational());
        let output = worker.execute_timeout(Duration::from_secs(5)).await?;
        
        // Grade against test suite
        let score = self.grade(output, submission.exercise)?;
        
        // Store for analytics (async, non-blocking)
        tokio::spawn(self.record_metrics(submission.student_id, score));
        
        GradingResult { score, feedback: self.generate_feedback(output) }
    }
}
```

## Implementation Strategy

### Phase 1: Core Testing (Weeks 1-4)
- Basic assertion testing
- Snapshot management
- State preservation
- Coverage tracking

### Phase 2: Property & Differential (Weeks 5-8)
- Property-based generators
- Backend differential testing
- Mutation framework
- Performance baselines

### Phase 3: Formal Methods (Weeks 9-12)
- SMT integration (Z3)
- Complexity analyzer
- Bounded verification
- Proof cache

### Phase 4: Educational Platform (Weeks 13-16)
- WASM sandbox
- Feedback engine
- Anti-cheating
- Analytics pipeline

### Phase 5: Production Hardening (Weeks 17-20)
- CI/CD integration
- Performance optimization
- Documentation
- Migration tools

## Performance Targets

- **Simple assertion**: <1ms overhead per cell
- **Property test**: <100ms for 100 iterations
- **SMT verification**: <5s timeout, cached results
- **Complexity analysis**: <500ms static, <2s empirical
- **WASM compilation**: <200ms for typical exercise
- **Differential test**: <10s for 4 backends

## Key Design Decisions

1. **Test metadata lives in notebook** - No separate test files to maintain
2. **Multiple validation strategies** - Deterministic, property, differential, formal
3. **State preservation** - Tests see accumulated notebook state
4. **Efficient golden storage** - Format-appropriate serialization
5. **Incremental testing** - Only re-run affected cells on change
6. **WASM sandboxing** - Safe execution of untrusted student code
7. **Progressive disclosure** - Hints and feedback adapt to student progress
8. **Anti-cheating** - Parameterized problems and plagiarism detection
9. **Unified pipeline** - Single framework serves users and compiler team
10. **Pragmatic verification** - Bounded proofs with tractability checks

This framework exceeds nbval by integrating formal methods, supports Coursera-style education via WASM sandboxing, and serves as the acceptance testing harness for the Ruchy compiler itself. The architecture scales from simple assertions to SMT-backed proofs while maintaining sub-second feedback for common operations.