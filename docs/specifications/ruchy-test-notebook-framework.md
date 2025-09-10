# Ruchy Test: Native Notebook Testing Framework

## Command Architecture

`ruchy test` executes notebook cells as test cases, validating outputs against expectations with property-based and deterministic verification.

```bash
# Test all notebooks in project
ruchy test

# Test specific notebook
ruchy test notebooks/analysis.ruchynb

# Test with coverage
ruchy test --coverage

# Test with mutation
ruchy test --mutate

# Regenerate golden outputs
ruchy test --update-golden
```

## Cell Test Annotations

Notebooks contain test metadata inline with cells.

```rust
// Cell metadata in .ruchynb format
{
  "cell_type": "code",
  "source": "let df = read_csv('data.csv')\ndf.describe()",
  "metadata": {
    "test": {
      "type": "deterministic",
      "golden": "outputs/df_describe.golden",
      "tolerance": 1e-6
    }
  }
}
```

### Test Types

```rust
enum CellTestType {
    Deterministic {
        golden: PathBuf,
        tolerance: Option<f64>,
    },
    Property {
        invariants: Vec<String>,
        generators: HashMap<String, Generator>,
    },
    Regression {
        baseline: PathBuf,
        max_time_factor: f64,
        max_memory_factor: f64,
    },
    Differential {
        backends: Vec<Backend>,
        equivalence: EquivalenceType,
    },
    Skip,
}
```

## Output Validation Engine

### Deterministic Testing (nbval-style)

```rust
impl NotebookTester {
    fn test_deterministic(&self, cell: &Cell) -> TestResult {
        let actual = self.execute_cell(cell)?;
        let expected = self.load_golden(&cell.metadata.golden)?;
        
        match (&actual, &expected) {
            (Output::DataFrame(a), Output::DataFrame(e)) => {
                self.compare_dataframes(a, e, cell.metadata.tolerance)
            }
            (Output::Plot(a), Output::Plot(e)) => {
                self.compare_plots(a, e)  // Perceptual hashing
            }
            (Output::Value(a), Output::Value(e)) => {
                self.compare_values(a, e, cell.metadata.tolerance)
            }
            _ => TestResult::TypeMismatch
        }
    }
    
    fn compare_dataframes(&self, actual: &DataFrame, expected: &DataFrame, tol: f64) -> TestResult {
        // Shape check
        if actual.shape() != expected.shape() {
            return TestResult::ShapeMismatch;
        }
        
        // Numeric columns: use tolerance
        for col in actual.numeric_columns() {
            if !actual[col].approx_eq(&expected[col], tol) {
                return TestResult::NumericDivergence { col, max_delta };
            }
        }
        
        // Categorical columns: exact match
        for col in actual.categorical_columns() {
            if actual[col] != expected[col] {
                return TestResult::CategoricalMismatch { col };
            }
        }
        
        TestResult::Pass
    }
}
```

### Property-Based Cell Testing

```rust
// In-notebook property specification
#[test_cell(property)]
fn test_transformation_properties() {
    // Generator configuration
    let df = generate_dataframe(rows: 100..1000, cols: 5..20);
    
    // Properties that must hold
    assert_property!(df.normalize().mean() ~= 0.0);
    assert_property!(df.normalize().std() ~= 1.0);
    assert_property!(df.filter(|r| r.valid()).len() <= df.len());
}

impl PropertyTester {
    fn test_property_cell(&self, cell: &PropertyCell) -> TestResult {
        let mut failures = vec![];
        
        for _ in 0..self.num_iterations {
            let inputs = self.generate_inputs(&cell.generators);
            let env = self.create_environment(inputs);
            
            match self.execute_with_env(cell, env) {
                Ok(_) => continue,
                Err(PropertyViolation { property, counterexample }) => {
                    failures.push((property, self.shrink(counterexample)));
                }
            }
        }
        
        if failures.is_empty() {
            TestResult::Pass
        } else {
            TestResult::PropertyFailures(failures)
        }
    }
}
```

## State Management Across Cells

Notebooks maintain state between cells. Test framework must handle this correctly.

```rust
struct NotebookTestSession {
    state: NotebookState,
    cell_outputs: Vec<CellOutput>,
    checkpoints: HashMap<CellId, NotebookState>,
}

impl NotebookTestSession {
    fn run_notebook_test(&mut self, notebook: &Notebook) -> TestReport {
        let mut results = vec![];
        
        for cell in &notebook.cells {
            // Checkpoint before potentially failing cell
            if cell.has_test() {
                self.checkpoints.insert(cell.id, self.state.clone());
            }
            
            // Execute cell with accumulated state
            let output = self.execute_with_state(cell, &mut self.state)?;
            
            // Validate if test metadata present
            if let Some(test) = &cell.metadata.test {
                let result = self.validate_output(&output, test);
                
                if result.is_failure() && test.stop_on_failure {
                    // Restore checkpoint for debugging
                    self.state = self.checkpoints[&cell.id].clone();
                    break;
                }
                
                results.push(result);
            }
            
            self.cell_outputs.push(output);
        }
        
        TestReport { results, coverage: self.compute_coverage() }
    }
}
```

## Formal Verification and Complexity Analysis

### Provability Annotations

Cells annotated with formal specifications for SMT-based verification.

```rust
#[test_cell(prove)]
fn verified_sort() {
    #[requires(arr.len() > 0)]
    #[ensures(result.is_sorted() && result.len() == arr.len())]
    #[invariant(partition_point >= 0 && partition_point <= arr.len())]
    fn quicksort(arr: Vec<i32>) -> Vec<i32> {
        // Implementation
    }
    
    // Z3 proves correctness for all inputs up to size N
    verify_exhaustive!(quicksort, max_size: 100);
}

impl ProofEngine {
    fn verify_cell(&self, cell: &ProofCell) -> VerificationResult {
        let smt_context = Z3Context::new();
        
        // Extract pre/post conditions
        let spec = self.extract_specification(cell);
        
        // Generate verification conditions
        let vcs = self.wp_transform(cell.ast, spec.postcondition);
        
        // Prove each VC
        for vc in vcs {
            let formula = self.vc_to_smt(vc, &spec.precondition);
            match smt_context.check_sat(formula) {
                SatResult::Unsat => continue,  // Proved
                SatResult::Sat => {
                    return VerificationResult::CounterExample(
                        smt_context.get_model()
                    );
                }
                SatResult::Unknown(reason) => {
                    return VerificationResult::Timeout(reason);
                }
            }
        }
        
        VerificationResult::Proved
    }
}
```

### Complexity Verification

Static analysis to verify Big-O claims.

```rust
#[test_cell(complexity)]
fn algorithm_complexity() {
    #[complexity(O("n log n"))]
    fn merge_sort<T: Ord>(arr: &mut [T]) {
        // Implementation
    }
    
    #[complexity(O("n^2"), worst_case)]
    #[complexity(O("n log n"), average_case)]
    fn quicksort<T: Ord>(arr: &mut [T]) {
        // Implementation
    }
}

impl ComplexityAnalyzer {
    fn verify_complexity(&self, func: &Function, claimed: &Complexity) -> ComplexityResult {
        // Build recurrence relations from AST
        let recurrence = self.extract_recurrence(func);
        
        // Solve using master theorem or substitution
        let actual = match recurrence {
            Recurrence::DivideConquer { a, b, work } => {
                self.master_theorem(a, b, work)
            }
            Recurrence::Linear { base, recursive } => {
                self.solve_linear(base, recursive)
            }
            Recurrence::Complex(system) => {
                self.solve_system(system)
            }
        };
        
        // Compare with claimed complexity
        if actual.dominates(&claimed) {
            ComplexityResult::Verified
        } else {
            ComplexityResult::Violation {
                claimed: claimed.clone(),
                actual,
                witness: self.generate_witness(&actual, &claimed),
            }
        }
    }
    
    fn empirical_verification(&self, func: &Function, claimed: &Complexity) -> EmpiricResult {
        let sizes = [10, 100, 1000, 10000];
        let mut timings = vec![];
        
        for n in sizes {
            let input = self.generate_worst_case_input(func, n);
            let time = self.measure_execution(func, input);
            timings.push((n, time));
        }
        
        // Fit curve and extract growth rate
        let fitted = self.fit_complexity_curve(&timings);
        
        EmpiricResult {
            fitted_complexity: fitted,
            confidence: self.goodness_of_fit(&timings, &fitted),
            matches_claim: fitted.equivalent(&claimed),
        }
    }
}
```

## Runtime Acceptance Testing

Use notebooks as canary tests for Ruchy compiler backends.

```rust
#[canary_notebook("compiler/type_system.ruchynb")]
mod type_system_canary {
    #[runtime_test(backends = [Interpreter, JIT, Transpiled, WASM])]
    fn test_hindley_milner_inference() {
        // Cell 1: Basic type inference
        let x = 42;  // Should infer i32
        let y = x + 1;  // Should propagate type
        assert_type!(y, i32);
        
        // Cell 2: Polymorphic functions
        fn identity<T>(x: T) -> T { x }
        assert_type!(identity(5), i32);
        assert_type!(identity("hello"), &str);
        
        // Cell 3: Row polymorphism
        let record = { x: 10, y: 20 };
        fn get_x<R: { x: i32, .. }>(r: R) -> i32 { r.x }
        assert_eq!(get_x(record), 10);
    }
}

impl RuntimeAcceptanceTester {
    fn run_canary_notebook(&self, notebook: &Path) -> CanaryResult {
        let notebook = self.load_notebook(notebook)?;
        let mut results = HashMap::new();
        
        for backend in &[Interpreter, JIT, Transpiled, WASM] {
            let output = self.execute_on_backend(&notebook, backend)?;
            results.insert(backend, output);
        }
        
        // Verify semantic equivalence across all backends
        let reference = &results[&Interpreter];
        for (backend, output) in &results {
            if backend == &Interpreter { continue; }
            
            if !self.semantically_equivalent(reference, output) {
                return CanaryResult::BackendDivergence {
                    reference: Interpreter,
                    divergent: backend.clone(),
                    diff: self.compute_diff(reference, output),
                };
            }
        }
        
        CanaryResult::Pass
    }
    
    fn performance_regression_check(&self, notebook: &Path) -> RegressionResult {
        let baseline = self.load_baseline(notebook)?;
        let current = self.benchmark_notebook(notebook)?;
        
        // Check each backend independently
        for backend in &[JIT, Transpiled, WASM] {
            let baseline_perf = &baseline[backend];
            let current_perf = &current[backend];
            
            if current_perf.execution_time > baseline_perf.execution_time * 1.1 {
                return RegressionResult::TimeRegression {
                    backend: backend.clone(),
                    baseline: baseline_perf.execution_time,
                    current: current_perf.execution_time,
                };
            }
            
            if current_perf.memory_peak > baseline_perf.memory_peak * 1.2 {
                return RegressionResult::MemoryRegression {
                    backend: backend.clone(),
                    baseline: baseline_perf.memory_peak,
                    current: current_perf.memory_peak,
                };
            }
        }
        
        RegressionResult::NoRegression
    }
}
```

### Canary Notebook Structure

```yaml
# canary.ruchynb
metadata:
  type: canary
  features: [type_inference, row_polymorphism, effects]
  backends: [all]
  performance_critical: true

cells:
  - id: setup
    source: |
      // Test environment setup
      use ruchy::testing::*;
      set_strict_mode(true);
    
  - id: feature_test
    source: |
      // Specific language feature under test
      let result = complex_type_inference_case();
    test:
      type: differential
      backends: [Interpreter, JIT, Transpiled]
      
  - id: performance_test
    source: |
      // Performance-sensitive operation
      let df = generate_dataframe(1_000_000);
      df.groupby("category").agg("mean")
    test:
      type: regression
      baseline: canary_baselines/perf_001.baseline
      thresholds:
        time: 1.1x
        memory: 1.2x
```

### CI Integration for Compiler Development

```yaml
# .github/workflows/compiler-canary.yml
name: Compiler Canary Tests

on:
  pull_request:
    paths:
      - 'src/compiler/**'
      - 'src/runtime/**'

jobs:
  canary:
    runs-on: ubuntu-latest
    steps:
      - name: Run Canary Notebooks
        run: |
          ruchy test \
            --canary-mode \
            --notebooks "canaries/*.ruchynb" \
            --backends all \
            --differential \
            --fail-on-divergence
            
      - name: Performance Regression Check
        run: |
          ruchy test \
            --performance-mode \
            --baseline main \
            --threshold-time 1.1 \
            --threshold-memory 1.2
```

## Coverage Analysis

Track which code paths in cells are exercised.

```rust
impl CoverageTracker {
    fn instrument_cell(&self, cell: &Cell) -> InstrumentedCell {
        let ast = parse(&cell.source)?;
        
        // Insert coverage probes
        let instrumented = ast.transform(|node| {
            match node {
                Node::Branch(condition, then_branch, else_branch) => {
                    Node::Branch(
                        condition,
                        self.probe(then_branch, BranchId::new()),
                        self.probe(else_branch, BranchId::new()),
                    )
                }
                Node::FunctionCall(name, args) => {
                    self.probe(Node::FunctionCall(name, args), CallId::new())
                }
                _ => node
            }
        });
        
        InstrumentedCell { ast: instrumented, probes: self.probes.clone() }
    }
    
    fn report_coverage(&self) -> CoverageReport {
        CoverageReport {
            line_coverage: self.covered_lines as f64 / self.total_lines as f64,
            branch_coverage: self.covered_branches as f64 / self.total_branches as f64,
            uncovered_sections: self.find_uncovered_sections(),
        }
    }
}
```

## Mutation Testing for Notebooks

Validate test quality by mutating notebook code.

```rust
impl NotebookMutator {
    fn mutate_and_test(&self, notebook: &Notebook) -> MutationScore {
        let mutations = self.generate_mutations(notebook);
        let mut killed = 0;
        
        for mutation in mutations {
            let mutated = notebook.apply_mutation(&mutation);
            let result = self.run_tests(&mutated);
            
            if result.has_failures() {
                killed += 1;
            } else {
                println!("SURVIVED: {:?} at cell {}", mutation.kind, mutation.cell);
            }
        }
        
        MutationScore {
            kill_rate: killed as f64 / mutations.len() as f64,
            survived: mutations.len() - killed,
        }
    }
}

enum NotebookMutation {
    // Data mutations
    SwapDataFile { cell: CellId, original: Path, replacement: Path },
    CorruptDataSample { cell: CellId, corruption_rate: f64 },
    
    // Code mutations  
    AlterNumericConstant { cell: CellId, location: Location, delta: f64 },
    SwapOperator { cell: CellId, location: Location, op: Operator },
    RemoveValidation { cell: CellId, check: ValidationCheck },
    
    // Control flow mutations
    ReorderCells { swap: (CellId, CellId) },
    SkipCell { cell: CellId },
    DuplicateExecution { cell: CellId },
}
```

## Golden Output Management

Maintain and version golden outputs efficiently.

```rust
impl GoldenManager {
    fn update_golden(&self, notebook: &Notebook) -> Result<()> {
        let session = NotebookTestSession::new();
        
        for cell in &notebook.cells {
            if let Some(test) = &cell.metadata.test {
                if let CellTestType::Deterministic { golden, .. } = test {
                    let output = session.execute_cell(cell)?;
                    self.save_golden(&golden, &output)?;
                    
                    // Track in version control
                    self.git_add(&golden)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn save_golden(&self, path: &Path, output: &CellOutput) -> Result<()> {
        match output {
            CellOutput::DataFrame(df) => {
                // Parquet for dataframes (efficient, preserves types)
                df.to_parquet(path)?;
            }
            CellOutput::Plot(plot) => {
                // Perceptual hash for plots
                let hash = self.perceptual_hash(plot);
                fs::write(path, hash.to_bytes())?;
            }
            CellOutput::Value(val) => {
                // MessagePack for general values
                let bytes = rmp_serde::to_vec(val)?;
                fs::write(path, bytes)?;
            }
        }
        
        Ok(())
    }
}
```

## Configuration

```toml
# ruchy.toml
[test.notebook]
golden_dir = "test/golden"
coverage_threshold = 80.0
mutation_threshold = 75.0
property_iterations = 100
tolerance = 1e-6

[test.notebook.backends]
# For differential testing
enabled = ["interpreter", "jit", "transpiled"]

[test.notebook.resources]
max_memory = "512MB"
max_time = "30s"
allow_network = false
```

## CLI Output

```bash
$ ruchy test notebooks/analysis.ruchynb
Running notebook tests...

✓ Cell 1: Data loading [23ms]
✓ Cell 2: Normalization (property: mean=0) [145ms]
✗ Cell 3: Regression output mismatch
  Expected: R²=0.923
  Actual:   R²=0.891
  Tolerance exceeded: 0.032 > 0.01
  
✓ Cell 4: Plot generation (perceptual match) [89ms]
⊘ Cell 5: Skipped (no test metadata)

Coverage: 87% (lines), 72% (branches)
Mutation Score: 81% (17/21 killed)

Tests: 4 passed, 1 failed, 1 skipped
Time: 0.28s
```

## Integration with CI

```yaml
# .github/workflows/notebook-tests.yml
- name: Test Notebooks
  run: |
    ruchy test \
      --coverage-threshold 80 \
      --mutation-threshold 70 \
      --differential-backends all \
      --fail-fast
```

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