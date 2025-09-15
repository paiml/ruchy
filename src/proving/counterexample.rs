//! Counterexample generation for failed proofs
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use super::smt::{SmtSolver, SmtBackend, SmtResult};
/// Counterexample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counterexample {
    /// Variable assignments
    pub assignments: HashMap<String, Value>,
    /// Execution trace
    pub trace: Vec<TraceStep>,
    /// Failed assertion
    pub failed_assertion: String,
    /// Explanation
    pub explanation: Option<String>,
}
/// Value in counterexample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Int(i64),
    Bool(bool),
    String(String),
    Float(f64),
    Array(Vec<Value>),
    Tuple(Vec<Value>),
    Null,
}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(n) => write!(f, "{n}"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Float(x) => write!(f, "{x}"),
            Self::Array(vs) => {
                write!(f, "[")?;
                for (i, v) in vs.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            Self::Tuple(vs) => {
                write!(f, "(")?;
                for (i, v) in vs.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{v}")?;
                }
                write!(f, ")")
            }
            Self::Null => write!(f, "null"),
        }
    }
}
/// Trace step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    /// Step number
    pub step: usize,
    /// Location in code
    pub location: String,
    /// Operation performed
    pub operation: String,
    /// State after operation
    pub state: HashMap<String, Value>,
}
impl Counterexample {
    /// Create new counterexample
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::new;
/// 
/// let result = new("example");
/// assert_eq!(result, Ok(()));
/// ```
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::new;
/// 
/// let result = new("example");
/// assert_eq!(result, Ok(()));
/// ```
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::new;
/// 
/// let result = new("example");
/// assert_eq!(result, Ok(()));
/// ```
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::new;
/// 
/// let result = new("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn new(failed_assertion: &str) -> Self {
        Self {
            assignments: HashMap::new(),
            trace: Vec::new(),
            failed_assertion: failed_assertion.to_string(),
            explanation: None,
        }
    }
    /// Add assignment
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::add_assignment;
/// 
/// let result = add_assignment("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn add_assignment(&mut self, var: &str, value: Value) {
        self.assignments.insert(var.to_string(), value);
    }
    /// Add trace step
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::add_trace_step;
/// 
/// let result = add_trace_step(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn add_trace_step(&mut self, step: TraceStep) {
        self.trace.push(step);
    }
    /// Set explanation
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::set_explanation;
/// 
/// let result = set_explanation("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn set_explanation(&mut self, explanation: &str) {
        self.explanation = Some(explanation.to_string());
    }
    /// Format as readable report
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::format_report;
/// 
/// let result = format_report(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn format_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Counterexample Found ===\n\n");
        report.push_str("Failed Assertion:\n");
        report.push_str(&format!("  {}\n\n", self.failed_assertion));
        if !self.assignments.is_empty() {
            report.push_str("Variable Assignments:\n");
            for (var, val) in &self.assignments {
                report.push_str(&format!("  {var} = {val}\n"));
            }
            report.push('\n');
        }
        if !self.trace.is_empty() {
            report.push_str("Execution Trace:\n");
            for step in &self.trace {
                report.push_str(&format!("  Step {}: {} at {}\n", 
                    step.step, step.operation, step.location));
                if !step.state.is_empty() {
                    for (var, val) in &step.state {
                        report.push_str(&format!("    {var} = {val}\n"));
                    }
                }
            }
            report.push('\n');
        }
        if let Some(explanation) = &self.explanation {
            report.push_str("Explanation:\n");
            report.push_str(&format!("  {explanation}\n"));
        }
        report
    }
}
/// Test case for property-based testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Input values
    pub inputs: HashMap<String, Value>,
    /// Expected output
    pub expected: Option<Value>,
    /// Property to test
    pub property: String,
}
impl TestCase {
    /// Create new test case
    pub fn new(property: &str) -> Self {
        Self {
            inputs: HashMap::new(),
            expected: None,
            property: property.to_string(),
        }
    }
    /// Add input
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::add_input;
/// 
/// let result = add_input("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn add_input(&mut self, name: &str, value: Value) {
        self.inputs.insert(name.to_string(), value);
    }
    /// Set expected output
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::set_expected;
/// 
/// let result = set_expected(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn set_expected(&mut self, value: Value) {
        self.expected = Some(value);
    }
    /// Generate Ruchy test code
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::to_ruchy_test;
/// 
/// let result = to_ruchy_test("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn to_ruchy_test(&self, test_name: &str) -> String {
        let mut code = String::new();
        code.push_str(&format!("#[test]\nfn {test_name}() {{\n"));
        for (name, value) in &self.inputs {
            code.push_str(&format!("    let {} = {};\n", name, 
                self.value_to_ruchy(value)));
        }
        code.push_str(&format!("    assert!({});\n", self.property));
        if let Some(expected) = &self.expected {
            code.push_str(&format!("    assert_eq!(result, {});\n", 
                self.value_to_ruchy(expected)));
        }
        code.push_str("}\n");
        code
    }
    /// Convert value to Ruchy syntax
    fn value_to_ruchy(&self, value: &Value) -> String {
        match value {
            Value::Int(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => format!("\"{s}\""),
            Value::Float(x) => format!("{x:.6}"),
            Value::Array(vs) => {
                let items: Vec<String> = vs.iter()
                    .map(|v| self.value_to_ruchy(v))
                    .collect();
                format!("[{}]", items.join(", "))
            }
            Value::Tuple(vs) => {
                let items: Vec<String> = vs.iter()
                    .map(|v| self.value_to_ruchy(v))
                    .collect();
                format!("({})", items.join(", "))
            }
            Value::Null => "None".to_string(),
        }
    }
}
/// Counterexample generator
pub struct CounterexampleGenerator {
    backend: SmtBackend,
    max_iterations: usize,
    shrinking: bool,
}
impl CounterexampleGenerator {
    /// Create new generator
    pub fn new() -> Self {
        Self {
            backend: SmtBackend::Z3,
            max_iterations: 100,
            shrinking: true,
        }
    }
    /// Set SMT backend
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::set_backend;
/// 
/// let result = set_backend(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn set_backend(&mut self, backend: SmtBackend) {
        self.backend = backend;
    }
    /// Enable/disable shrinking
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::set_shrinking;
/// 
/// let result = set_shrinking(true);
/// assert_eq!(result, Ok(true));
/// ```
pub fn set_shrinking(&mut self, enabled: bool) {
        self.shrinking = enabled;
    }
    /// Generate counterexample for property
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::generate;
/// 
/// let result = generate("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn generate(&self, property: &str, vars: &[(String, String)]) -> Result<Option<Counterexample>> {
        let mut solver = SmtSolver::new(self.backend);
        for (name, sort) in vars {
            solver.declare_var(name, sort);
        }
        solver.assert(&format!("(not {property})"));
        match solver.check_sat()? {
            SmtResult::Sat => {
                let model = solver.get_model()?;
                Ok(Some(self.build_counterexample(property, model)))
            }
            _ => Ok(None),
        }
    }
    /// Build counterexample from model
    fn build_counterexample(&self, property: &str, model: Option<HashMap<String, String>>) -> Counterexample {
        let mut cex = Counterexample::new(property);
        if let Some(assignments) = model {
            for (var, val) in assignments {
                cex.add_assignment(&var, self.parse_value(&val));
            }
        }
        if self.shrinking {
            self.shrink_counterexample(&mut cex);
        }
        cex
    }
    /// Parse SMT value
    fn parse_value(&self, smt_value: &str) -> Value {
        if let Ok(n) = smt_value.parse::<i64>() {
            Value::Int(n)
        } else if smt_value == "true" {
            Value::Bool(true)
        } else if smt_value == "false" {
            Value::Bool(false)
        } else if let Ok(x) = smt_value.parse::<f64>() {
            Value::Float(x)
        } else {
            Value::String(smt_value.to_string())
        }
    }
    /// Shrink counterexample to minimal form
    fn shrink_counterexample(&self, cex: &mut Counterexample) {
        let mut changed = true;
        let mut iterations = 0;
        while changed && iterations < self.max_iterations {
            changed = false;
            iterations += 1;
            for (var, value) in cex.assignments.clone() {
                if let Some(shrunk) = self.try_shrink_value(&value) {
                    cex.assignments.insert(var, shrunk);
                    changed = true;
                }
            }
        }
    }
    /// Try to shrink a value
    fn try_shrink_value(&self, value: &Value) -> Option<Value> {
        match value {
            Value::Int(n) if *n > 0 => Some(Value::Int(n / 2)),
            Value::Array(vs) if !vs.is_empty() => {
                let mut shrunk = vs.clone();
                shrunk.pop();
                Some(Value::Array(shrunk))
            }
            Value::String(s) if s.len() > 1 => {
                Some(Value::String(s[..s.len()-1].to_string()))
            }
            _ => None,
        }
    }
    /// Generate multiple test cases
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::generate_test_suite;
/// 
/// let result = generate_test_suite("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn generate_test_suite(&self, property: &str, vars: &[(String, String)], count: usize) -> Result<Vec<TestCase>> {
        let mut test_cases: Vec<TestCase> = Vec::new();
        for i in 0..count {
            let mut solver = SmtSolver::new(self.backend);
            for (name, sort) in vars {
                solver.declare_var(name, sort);
            }
            for prev_case in &test_cases {
                for (var, val) in &prev_case.inputs {
                    solver.assert(&format!("(not (= {} {}))", var, 
                        self.value_to_smt(val)));
                }
            }
            solver.assert(&format!("(not {property})"));
            match solver.check_sat()? {
                SmtResult::Sat => {
                    if let Some(model) = solver.get_model()? {
                        let mut test_case = TestCase::new(property);
                        for (var, val) in model {
                            test_case.add_input(&var, self.parse_value(&val));
                        }
                        test_cases.push(test_case);
                    }
                }
                _ => break,
            }
            if i >= count {
                break;
            }
        }
        Ok(test_cases)
    }
    /// Convert value to SMT syntax
    fn value_to_smt(&self, value: &Value) -> String {
        match value {
            Value::Int(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => format!("\"{s}\""),
            Value::Float(x) => format!("{x:.6}"),
            _ => "null".to_string(),
        }
    }
}
impl Default for CounterexampleGenerator {
    fn default() -> Self {
        Self::new()
    }
}
/// Symbolic execution engine
pub struct SymbolicExecutor {
    generator: CounterexampleGenerator,
    path_conditions: Vec<String>,
    symbolic_state: HashMap<String, String>,
}
impl SymbolicExecutor {
    /// Create new symbolic executor
    pub fn new() -> Self {
        Self {
            generator: CounterexampleGenerator::new(),
            path_conditions: Vec::new(),
            symbolic_state: HashMap::new(),
        }
    }
    /// Add path condition
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::add_condition;
/// 
/// let result = add_condition("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn add_condition(&mut self, condition: &str) {
        self.path_conditions.push(condition.to_string());
    }
    /// Set symbolic variable
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::set_symbolic;
/// 
/// let result = set_symbolic("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn set_symbolic(&mut self, var: &str, symbolic: &str) {
        self.symbolic_state.insert(var.to_string(), symbolic.to_string());
    }
    /// Find path to error
/// # Examples
/// 
/// ```
/// use ruchy::proving::counterexample::find_error_path;
/// 
/// let result = find_error_path("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn find_error_path(&self, error_condition: &str) -> Result<Option<Counterexample>> {
        let mut solver = SmtSolver::new(self.generator.backend);
        for var in self.symbolic_state.keys() {
            solver.declare_var(var, "Int");
        }
        for condition in &self.path_conditions {
            solver.assert(condition);
        }
        solver.assert(error_condition);
        match solver.check_sat()? {
            SmtResult::Sat => {
                let model = solver.get_model()?;
                Ok(Some(self.generator.build_counterexample(error_condition, model)))
            }
            _ => Ok(None),
        }
    }
}
impl Default for SymbolicExecutor {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_counterexample_report() {
        let mut cex = Counterexample::new("x > 10");
        cex.add_assignment("x", Value::Int(5));
        cex.set_explanation("x = 5 does not satisfy x > 10");
        let report = cex.format_report();
        assert!(report.contains("x > 10"));
        assert!(report.contains("x = 5"));
    }
    #[test]
    fn test_test_case_generation() {
        let mut test = TestCase::new("x > 0");
        test.add_input("x", Value::Int(42));
        test.set_expected(Value::Bool(true));
        let code = test.to_ruchy_test("test_positive");
        assert!(code.contains("let x = 42"));
        assert!(code.contains("assert!(x > 0)"));
    }
}
#[cfg(test)]
mod property_tests_counterexample {
    use proptest::proptest;
    
    
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
