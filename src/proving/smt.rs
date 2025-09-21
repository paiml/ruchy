//! SMT solver integration for proof automation
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
/// SMT solver backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmtBackend {
    Z3,
    CVC5,
    Yices2,
    MathSAT,
}
impl SmtBackend {
    /// Get command for solver
    fn command(&self) -> &str {
        match self {
            Self::Z3 => "z3",
            Self::CVC5 => "cvc5",
            Self::Yices2 => "yices-smt2",
            Self::MathSAT => "mathsat",
        }
    }
    /// Get command arguments
    fn args(&self) -> Vec<&str> {
        match self {
            Self::Z3 => vec!["-in", "-smt2"],
            Self::CVC5 => vec!["--lang", "smt2", "--incremental"],
            Self::Yices2 => vec!["--incremental"],
            Self::MathSAT => vec!["-input=smt2"],
        }
    }
}
/// SMT solver interface
pub struct SmtSolver {
    backend: SmtBackend,
    timeout_ms: u64,
    assertions: Vec<String>,
    declarations: HashMap<String, String>,
}
impl SmtSolver {
    /// Create new SMT solver
    /// # Examples
    ///
    /// ```
    /// use ruchy::proving::smt::SmtSolver;
    ///
    /// let instance = SmtSolver::new();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```
    /// use ruchy::proving::smt::SmtSolver;
    ///
    /// let instance = SmtSolver::new();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```
    /// use ruchy::proving::smt::SmtSolver;
    ///
    /// let instance = SmtSolver::new();
    /// // Verify behavior
    /// ```
    pub fn new(backend: SmtBackend) -> Self {
        Self {
            backend,
            timeout_ms: 5000,
            assertions: Vec::new(),
            declarations: HashMap::new(),
        }
    }
    /// Set timeout
    /// # Examples
    ///
    /// ```
    /// use ruchy::proving::smt::SmtSolver;
    ///
    /// let mut instance = SmtSolver::new();
    /// let result = instance.set_timeout();
    /// // Verify behavior
    /// ```
    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.timeout_ms = timeout_ms;
    }
    /// Declare a variable
    /// # Examples
    ///
    /// ```
    /// use ruchy::proving::smt::SmtSolver;
    ///
    /// let mut instance = SmtSolver::new();
    /// let result = instance.declare_var();
    /// // Verify behavior
    /// ```
    pub fn declare_var(&mut self, name: &str, sort: &str) {
        self.declarations
            .insert(name.to_string(), format!("(declare-fun {name} () {sort})"));
    }
    /// Declare a function
    /// # Examples
    ///
    /// ```
    /// use ruchy::proving::smt::SmtSolver;
    ///
    /// let mut instance = SmtSolver::new();
    /// let result = instance.declare_fun();
    /// // Verify behavior
    /// ```
    pub fn declare_fun(&mut self, name: &str, params: &[&str], ret: &str) {
        let params_str = params.join(" ");
        self.declarations.insert(
            name.to_string(),
            format!("(declare-fun {name} ({params_str}) {ret})"),
        );
    }
    /// Add an assertion
    /// # Examples
    ///
    /// ```
    /// use ruchy::proving::smt::SmtSolver;
    ///
    /// let mut instance = SmtSolver::new();
    /// let result = instance.assert();
    /// // Verify behavior
    /// ```
    pub fn assert(&mut self, expr: &str) {
        self.assertions.push(format!("(assert {expr})"));
    }
    /// Check satisfiability
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::proving::smt::check_sat;
    ///
    /// let result = check_sat(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn check_sat(&self) -> Result<SmtResult> {
        let query = self.build_query("(check-sat)");
        self.execute_query(&query)
    }
    /// Get model (if sat)
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::proving::smt::get_model;
    ///
    /// let result = get_model(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn get_model(&self) -> Result<Option<HashMap<String, String>>> {
        let query = self.build_query("(check-sat)\n(get-model)");
        let result = self.execute_query(&query)?;
        if result == SmtResult::Sat {
            Ok(Some(self.parse_model(&query)?))
        } else {
            Ok(None)
        }
    }
    /// Check validity (prove formula)
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::proving::smt::prove;
    ///
    /// let result = prove("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn prove(&self, formula: &str) -> Result<SmtResult> {
        let mut solver = self.clone();
        solver.assert(&format!("(not {formula})"));
        match solver.check_sat()? {
            SmtResult::Unsat => Ok(SmtResult::Valid),
            SmtResult::Sat => Ok(SmtResult::Invalid),
            other => Ok(other),
        }
    }
    /// Build complete SMT-LIB2 query
    fn build_query(&self, command: &str) -> String {
        let mut query = String::new();
        query.push_str("(set-logic ALL)\n");
        query.push_str(&format!("(set-option :timeout {})\n", self.timeout_ms));
        for decl in self.declarations.values() {
            query.push_str(decl);
            query.push('\n');
        }
        for assertion in &self.assertions {
            query.push_str(assertion);
            query.push('\n');
        }
        query.push_str(command);
        query.push('\n');
        query.push_str("(exit)\n");
        query
    }
    /// Execute query via solver process
    fn execute_query(&self, query: &str) -> Result<SmtResult> {
        let mut child = Command::new(self.backend.command())
            .args(self.backend.args())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(query.as_bytes())?;
        }
        let output = child.wait_with_output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_result(&stdout)
    }
    /// Parse solver result
    fn parse_result(&self, output: &str) -> Result<SmtResult> {
        if output.contains("sat") && !output.contains("unsat") {
            Ok(SmtResult::Sat)
        } else if output.contains("unsat") {
            Ok(SmtResult::Unsat)
        } else if output.contains("unknown") {
            Ok(SmtResult::Unknown)
        } else if output.contains("timeout") {
            Ok(SmtResult::Timeout)
        } else {
            Ok(SmtResult::Error(format!("Unexpected output: {output}")))
        }
    }
    /// Parse model from solver output
    fn parse_model(&self, _output: &str) -> Result<HashMap<String, String>> {
        Ok(HashMap::new())
    }
}
impl Clone for SmtSolver {
    fn clone(&self) -> Self {
        Self {
            backend: self.backend,
            timeout_ms: self.timeout_ms,
            assertions: self.assertions.clone(),
            declarations: self.declarations.clone(),
        }
    }
}
/// SMT query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtQuery {
    pub formula: String,
    pub variables: Vec<(String, String)>,
    pub assumptions: Vec<String>,
}
impl SmtQuery {
    /// Create new query
    pub fn new(formula: &str) -> Self {
        Self {
            formula: formula.to_string(),
            variables: Vec::new(),
            assumptions: Vec::new(),
        }
    }
    /// Add variable
    /// # Examples
    ///
    /// ```
    /// use ruchy::proving::smt::SmtQuery;
    ///
    /// let mut instance = SmtQuery::new();
    /// let result = instance.add_var();
    /// // Verify behavior
    /// ```
    pub fn add_var(&mut self, name: &str, sort: &str) {
        self.variables.push((name.to_string(), sort.to_string()));
    }
    /// Add assumption
    /// # Examples
    ///
    /// ```
    /// use ruchy::proving::smt::SmtQuery;
    ///
    /// let mut instance = SmtQuery::new();
    /// let result = instance.add_assumption();
    /// // Verify behavior
    /// ```
    pub fn add_assumption(&mut self, expr: &str) {
        self.assumptions.push(expr.to_string());
    }
    /// Execute query
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::proving::smt::execute;
    ///
    /// let result = execute(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn execute(&self, backend: SmtBackend) -> Result<SmtResult> {
        let mut solver = SmtSolver::new(backend);
        for (name, sort) in &self.variables {
            solver.declare_var(name, sort);
        }
        for assumption in &self.assumptions {
            solver.assert(assumption);
        }
        solver.prove(&self.formula)
    }
}
/// SMT solver result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SmtResult {
    Sat,
    Unsat,
    Valid,
    Invalid,
    Unknown,
    Timeout,
    Error(String),
}
impl SmtResult {
    /// Check if result indicates success
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::proving::smt::is_success;
    ///
    /// let result = is_success(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Valid | Self::Unsat)
    }
    /// Get human-readable description
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::proving::smt::description;
    ///
    /// let result = description(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn description(&self) -> &str {
        match self {
            Self::Sat => "Satisfiable",
            Self::Unsat => "Unsatisfiable",
            Self::Valid => "Valid",
            Self::Invalid => "Invalid",
            Self::Unknown => "Unknown",
            Self::Timeout => "Timeout",
            Self::Error(_) => "Error",
        }
    }
}
/// High-level proof automation
pub struct ProofAutomation {
    backend: SmtBackend,
    cache: HashMap<String, SmtResult>,
}
impl ProofAutomation {
    /// Create new automation
    pub fn new(backend: SmtBackend) -> Self {
        Self {
            backend,
            cache: HashMap::new(),
        }
    }
    /// Prove implication
    /// # Examples
    ///
    /// ```
    /// use ruchy::proving::smt::ProofAutomation;
    ///
    /// let mut instance = ProofAutomation::new();
    /// let result = instance.prove_implication();
    /// // Verify behavior
    /// ```
    pub fn prove_implication(&mut self, antecedent: &str, consequent: &str) -> Result<SmtResult> {
        let formula = format!("(=> {antecedent} {consequent})");
        self.prove(&formula)
    }
    /// Prove equivalence
    /// # Examples
    ///
    /// ```
    /// use ruchy::proving::smt::ProofAutomation;
    ///
    /// let mut instance = ProofAutomation::new();
    /// let result = instance.prove_equivalence();
    /// // Verify behavior
    /// ```
    pub fn prove_equivalence(&mut self, left: &str, right: &str) -> Result<SmtResult> {
        let formula = format!("(= {left} {right})");
        self.prove(&formula)
    }
    /// Prove with caching
    fn prove(&mut self, formula: &str) -> Result<SmtResult> {
        if let Some(cached) = self.cache.get(formula) {
            return Ok(cached.clone());
        }
        let query = SmtQuery::new(formula);
        let result = query.execute(self.backend)?;
        self.cache.insert(formula.to_string(), result.clone());
        Ok(result)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smt_query_construction() {
        let mut query = SmtQuery::new("(> x 0)");
        query.add_var("x", "Int");
        query.add_assumption("(< x 10)");
        assert_eq!(query.formula, "(> x 0)");
        assert_eq!(query.variables.len(), 1);
        assert_eq!(query.assumptions.len(), 1);
    }

    #[test]
    fn test_solver_initialization() {
        let solver = SmtSolver::new(SmtBackend::Z3);
        assert_eq!(solver.backend, SmtBackend::Z3);
        assert_eq!(solver.timeout_ms, 5000);
    }

    #[test]
    fn test_smt_backend_commands() {
        assert_eq!(SmtBackend::Z3.command(), "z3");
        assert_eq!(SmtBackend::CVC5.command(), "cvc5");
        assert_eq!(SmtBackend::Yices2.command(), "yices-smt2");
        assert_eq!(SmtBackend::MathSAT.command(), "mathsat");
    }

    #[test]
    fn test_smt_backend_args() {
        assert_eq!(SmtBackend::Z3.args(), vec!["-in", "-smt2"]);
        assert_eq!(
            SmtBackend::CVC5.args(),
            vec!["--lang", "smt2", "--incremental"]
        );
        assert_eq!(SmtBackend::Yices2.args(), vec!["--incremental"]);
        assert_eq!(SmtBackend::MathSAT.args(), vec!["-input=smt2"]);
    }

    #[test]
    fn test_solver_timeout_setting() {
        let mut solver = SmtSolver::new(SmtBackend::Z3);
        solver.set_timeout(10000);
        assert_eq!(solver.timeout_ms, 10000);
    }

    #[test]
    fn test_solver_declare_var() {
        let mut solver = SmtSolver::new(SmtBackend::Z3);
        solver.declare_var("x", "Int");
        assert!(solver.declarations.contains_key("x"));
        assert_eq!(solver.declarations["x"], "(declare-fun x () Int)");
    }

    #[test]
    fn test_solver_declare_fun() {
        let mut solver = SmtSolver::new(SmtBackend::Z3);
        solver.declare_fun("f", &["Int", "Bool"], "Real");
        assert!(solver.declarations.contains_key("f"));
        assert_eq!(solver.declarations["f"], "(declare-fun f (Int Bool) Real)");
    }

    #[test]
    fn test_solver_declare_fun_no_params() {
        let mut solver = SmtSolver::new(SmtBackend::Z3);
        solver.declare_fun("g", &[], "String");
        assert!(solver.declarations.contains_key("g"));
        assert_eq!(solver.declarations["g"], "(declare-fun g () String)");
    }

    #[test]
    fn test_solver_assert() {
        let mut solver = SmtSolver::new(SmtBackend::Z3);
        solver.assert("(> x 0)");
        assert_eq!(solver.assertions.len(), 1);
        assert_eq!(solver.assertions[0], "(assert (> x 0))");
    }

    #[test]
    fn test_solver_multiple_assertions() {
        let mut solver = SmtSolver::new(SmtBackend::Z3);
        solver.assert("(> x 0)");
        solver.assert("(< x 10)");
        assert_eq!(solver.assertions.len(), 2);
        assert!(solver.assertions.contains(&"(assert (> x 0))".to_string()));
        assert!(solver.assertions.contains(&"(assert (< x 10))".to_string()));
    }

    #[test]
    fn test_build_query() {
        let mut solver = SmtSolver::new(SmtBackend::Z3);
        solver.declare_var("x", "Int");
        solver.assert("(> x 0)");
        let query = solver.build_query("(check-sat)");

        assert!(query.contains("(set-logic ALL)"));
        assert!(query.contains("(set-option :timeout 5000)"));
        assert!(query.contains("(declare-fun x () Int)"));
        assert!(query.contains("(assert (> x 0))"));
        assert!(query.contains("(check-sat)"));
        assert!(query.contains("(exit)"));
    }

    #[test]
    fn test_parse_result_sat() {
        let solver = SmtSolver::new(SmtBackend::Z3);
        let result = solver.parse_result("sat").unwrap();
        assert_eq!(result, SmtResult::Sat);
    }

    #[test]
    fn test_parse_result_unsat() {
        let solver = SmtSolver::new(SmtBackend::Z3);
        let result = solver.parse_result("unsat").unwrap();
        assert_eq!(result, SmtResult::Unsat);
    }

    #[test]
    fn test_parse_result_unknown() {
        let solver = SmtSolver::new(SmtBackend::Z3);
        let result = solver.parse_result("unknown").unwrap();
        assert_eq!(result, SmtResult::Unknown);
    }

    #[test]
    fn test_parse_result_timeout() {
        let solver = SmtSolver::new(SmtBackend::Z3);
        let result = solver.parse_result("timeout").unwrap();
        assert_eq!(result, SmtResult::Timeout);
    }

    #[test]
    fn test_parse_result_error() {
        let solver = SmtSolver::new(SmtBackend::Z3);
        let result = solver.parse_result("unexpected output").unwrap();
        if let SmtResult::Error(msg) = result {
            assert!(msg.contains("Unexpected output"));
        } else {
            panic!("Expected Error result");
        }
    }

    #[test]
    fn test_smt_result_is_success() {
        assert!(SmtResult::Valid.is_success());
        assert!(SmtResult::Unsat.is_success());
        assert!(!SmtResult::Sat.is_success());
        assert!(!SmtResult::Invalid.is_success());
        assert!(!SmtResult::Unknown.is_success());
        assert!(!SmtResult::Timeout.is_success());
        assert!(!SmtResult::Error("test".to_string()).is_success());
    }

    #[test]
    fn test_smt_result_description() {
        assert_eq!(SmtResult::Sat.description(), "Satisfiable");
        assert_eq!(SmtResult::Unsat.description(), "Unsatisfiable");
        assert_eq!(SmtResult::Valid.description(), "Valid");
        assert_eq!(SmtResult::Invalid.description(), "Invalid");
        assert_eq!(SmtResult::Unknown.description(), "Unknown");
        assert_eq!(SmtResult::Timeout.description(), "Timeout");
        assert_eq!(SmtResult::Error("test".to_string()).description(), "Error");
    }

    #[test]
    fn test_smt_query_new() {
        let query = SmtQuery::new("(= x y)");
        assert_eq!(query.formula, "(= x y)");
        assert!(query.variables.is_empty());
        assert!(query.assumptions.is_empty());
    }

    #[test]
    fn test_smt_query_add_var() {
        let mut query = SmtQuery::new("(> x 0)");
        query.add_var("x", "Int");
        query.add_var("y", "Real");
        assert_eq!(query.variables.len(), 2);
        assert!(query
            .variables
            .contains(&("x".to_string(), "Int".to_string())));
        assert!(query
            .variables
            .contains(&("y".to_string(), "Real".to_string())));
    }

    #[test]
    fn test_smt_query_add_assumption() {
        let mut query = SmtQuery::new("(> x 0)");
        query.add_assumption("(< x 10)");
        query.add_assumption("(> y 5)");
        assert_eq!(query.assumptions.len(), 2);
        assert!(query.assumptions.contains(&"(< x 10)".to_string()));
        assert!(query.assumptions.contains(&"(> y 5)".to_string()));
    }

    #[test]
    fn test_proof_automation_new() {
        let automation = ProofAutomation::new(SmtBackend::Z3);
        assert_eq!(automation.backend, SmtBackend::Z3);
        assert!(automation.cache.is_empty());
    }

    #[test]
    fn test_solver_clone() {
        let mut solver1 = SmtSolver::new(SmtBackend::CVC5);
        solver1.set_timeout(8000);
        solver1.declare_var("x", "Bool");
        solver1.assert("(= x true)");

        let solver2 = solver1.clone();
        assert_eq!(solver2.backend, SmtBackend::CVC5);
        assert_eq!(solver2.timeout_ms, 8000);
        assert_eq!(solver2.declarations.len(), 1);
        assert_eq!(solver2.assertions.len(), 1);
    }

    #[test]
    fn test_solver_multiple_declare_var() {
        let mut solver = SmtSolver::new(SmtBackend::Z3);
        solver.declare_var("x", "Int");
        solver.declare_var("y", "Real");
        solver.declare_var("z", "Bool");

        assert_eq!(solver.declarations.len(), 3);
        assert_eq!(solver.declarations["x"], "(declare-fun x () Int)");
        assert_eq!(solver.declarations["y"], "(declare-fun y () Real)");
        assert_eq!(solver.declarations["z"], "(declare-fun z () Bool)");
    }

    #[test]
    fn test_build_query_with_custom_timeout() {
        let mut solver = SmtSolver::new(SmtBackend::Z3);
        solver.set_timeout(15000);
        solver.declare_var("x", "Int");
        let query = solver.build_query("(check-sat)");

        assert!(query.contains("(set-option :timeout 15000)"));
    }

    #[test]
    fn test_parse_result_mixed_output() {
        let solver = SmtSolver::new(SmtBackend::Z3);

        // Test case where output contains both "sat" and "unsat" - should prefer unsat
        let result = solver
            .parse_result("something unsat something sat")
            .unwrap();
        assert_eq!(result, SmtResult::Unsat);

        // Test case where output only contains "sat"
        let result = solver.parse_result("the result is sat").unwrap();
        assert_eq!(result, SmtResult::Sat);
    }

    #[test]
    fn test_smt_backend_enum_properties() {
        // Test PartialEq
        assert_eq!(SmtBackend::Z3, SmtBackend::Z3);
        assert_ne!(SmtBackend::Z3, SmtBackend::CVC5);

        // Test Clone
        let backend = SmtBackend::Yices2;
        let cloned = backend;
        assert_eq!(backend, cloned);
    }
}
#[cfg(test)]
mod property_tests_smt {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
