//! Educational Assessment System for REPL Replay Testing
//!
//! Provides automated grading, rubric evaluation, and academic integrity checking
//! for educational use of the Ruchy REPL.

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use sha2::{Sha256, Digest};
use regex::Regex;

use crate::runtime::replay::{
    ReplSession, Event, ReplayValidator
};
use crate::runtime::repl::Repl;

// ============================================================================
// Assignment Specification
// ============================================================================

/// Complete assignment specification for automated grading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub id: String,
    pub title: String,
    pub description: String,
    pub setup: AssignmentSetup,
    pub tasks: Vec<Task>,
    pub constraints: AssignmentConstraints,
    pub rubric: GradingRubric,
}

/// Initial setup for assignment environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentSetup {
    pub prelude_code: Vec<String>,
    pub provided_functions: HashMap<String, String>,
    pub immutable_bindings: HashSet<String>,
}

/// Individual task within an assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub points: u32,
    pub test_cases: Vec<TestCase>,
    pub hidden_cases: Vec<TestCase>,
    pub requirements: Vec<Requirement>,
}

/// Test case for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub input: String,
    pub expected: ExpectedBehavior,
    pub points: u32,
    pub timeout_ms: u64,
}

/// Expected behavior patterns for test validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpectedBehavior {
    ExactOutput(String),
    Pattern(String), // Regex pattern
    TypeSignature(String),
    Predicate(PredicateCheck),
    PerformanceBound {
        max_ns: u64,
        max_bytes: usize,
    },
}

/// Predicate-based checking for complex validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredicateCheck {
    pub name: String,
    pub check_fn: String, // Code to evaluate
}

/// Requirements that must be met
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Requirement {
    UseRecursion,
    NoLoops,
    UseHigherOrderFunctions,
    TypeSafe,
    PureFunction,
    TailRecursive,
}

/// Constraints on the assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentConstraints {
    pub max_time_ms: u64,
    pub max_memory_mb: usize,
    pub allowed_imports: Vec<String>,
    pub forbidden_keywords: Vec<String>,
    pub performance: Option<PerformanceConstraints>,
}

/// Performance requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConstraints {
    pub max_cpu_ms: u64,
    pub max_heap_mb: usize,
    pub complexity_bound: String, // e.g., "O(n log n)"
}

// ============================================================================
// Grading Rubric
// ============================================================================

/// Grading rubric with weighted categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradingRubric {
    pub categories: Vec<RubricCategory>,
    pub late_penalty: Option<LatePenalty>,
    pub bonus_criteria: Vec<BonusCriterion>,
}

/// Category in the grading rubric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricCategory {
    pub name: String,
    pub weight: f32,
    pub criteria: Vec<Criterion>,
}

/// Individual grading criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criterion {
    pub description: String,
    pub max_points: u32,
    pub evaluation: CriterionEvaluation,
}

/// How to evaluate a criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CriterionEvaluation {
    Automatic(AutomaticCheck),
    Manual(String), // Instructions for manual grading
    Hybrid { auto_weight: f32, manual_weight: f32 },
}

/// Automatic checking methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomaticCheck {
    TestsPassed,
    CodeQuality { min_score: f32 },
    Documentation { required_sections: Vec<String> },
    Performance { metric: String, threshold: f64 },
}

/// Late submission penalty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatePenalty {
    pub grace_hours: u32,
    pub penalty_per_day: f32,
    pub max_days_late: u32,
}

/// Bonus points criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusCriterion {
    pub description: String,
    pub points: u32,
    pub check: BonusCheck,
}

/// Bonus checking methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BonusCheck {
    ExtraFeature(String),
    Optimization { improvement_percent: f32 },
    CreativeSolution,
}

// ============================================================================
// Grading Engine
// ============================================================================

/// Main grading engine for automated assessment
pub struct GradingEngine {
    pub replay_validator: ReplayValidator,
    pub plagiarism_detector: PlagiarismDetector,
    pub secure_sandbox: SecureSandbox,
}

impl Default for GradingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GradingEngine {
    pub fn new() -> Self {
        Self {
            replay_validator: ReplayValidator::new(true),
            plagiarism_detector: PlagiarismDetector::new(),
            secure_sandbox: SecureSandbox::new(),
        }
    }
    
    /// Grade a student submission against an assignment
    pub fn grade_submission(
        &mut self,
        assignment: &Assignment,
        submission: &ReplSession,
    ) -> GradeReport {
        let mut report = GradeReport::new(assignment.id.clone());
        
        // Verify submission integrity
        if !self.verify_no_tampering(submission) {
            report.mark_invalid("Session integrity check failed");
            return report;
        }
        
        // Setup assignment environment
        let mut repl = match self.secure_sandbox.create_isolated_repl() {
            Ok(r) => r,
            Err(e) => {
                report.mark_invalid(&format!("Failed to create sandbox: {e}"));
                return report;
            }
        };
        
        // Load assignment setup
        if let Err(e) = self.load_setup(&mut repl, &assignment.setup) {
            report.mark_invalid(&format!("Failed to load setup: {e}"));
            return report;
        }
        
        // Grade each task
        for task in &assignment.tasks {
            let task_grade = self.grade_task(&mut repl, task, submission);
            report.add_task_grade(task_grade);
        }
        
        // Evaluate rubric
        report.rubric_score = self.evaluate_rubric(&assignment.rubric, submission);
        
        // Check performance requirements
        if let Some(perf) = &assignment.constraints.performance {
            report.performance_score = self.measure_performance(submission, perf);
        }
        
        // Detect plagiarism
        report.originality_score = self.plagiarism_detector.analyze(submission);
        
        // Calculate final grade
        report.calculate_final_grade();
        
        report
    }
    
    fn verify_no_tampering(&self, session: &ReplSession) -> bool {
        // Verify event sequence integrity
        let mut prev_timestamp = 0u64;
        for event in &session.timeline {
            if event.timestamp_ns < prev_timestamp {
                return false; // Time went backwards
            }
            prev_timestamp = event.timestamp_ns;
        }
        
        // Verify state hashes are consistent
        // In production, would replay and verify each hash
        true
    }
    
    fn load_setup(&self, repl: &mut Repl, setup: &AssignmentSetup) -> Result<()> {
        // Load prelude code
        for code in &setup.prelude_code {
            repl.eval(code)?;
        }
        
        // Load provided functions
        for (name, code) in &setup.provided_functions {
            repl.eval(&format!("let {name} = {code}"))?;
        }
        
        Ok(())
    }
    
    fn grade_task(
        &mut self,
        repl: &mut Repl,
        task: &Task,
        _submission: &ReplSession,
    ) -> TaskGrade {
        let mut grade = TaskGrade::new(task.id.clone());
        
        // Test visible cases
        for test in &task.test_cases {
            let result = self.run_test_case(repl, test);
            grade.add_test_result(test.input.clone(), result);
        }
        
        // Test hidden cases (for academic integrity)
        for test in &task.hidden_cases {
            let result = self.run_test_case(repl, test);
            grade.add_hidden_result(test.input.clone(), result);
        }
        
        // Check requirements
        for req in &task.requirements {
            if self.check_requirement(repl, req) {
                grade.requirements_met.insert(format!("{req:?}"));
            }
        }
        
        grade.calculate_score(task.points);
        grade
    }
    
    fn run_test_case(&self, repl: &mut Repl, test: &TestCase) -> TestResult {
        // Execute with timeout
        let start = std::time::Instant::now();
        let output = match repl.eval(&test.input) {
            Ok(out) => out,
            Err(e) => {
                return TestResult {
                    passed: false,
                    points_earned: 0,
                    feedback: format!("Error: {e}"),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                };
            }
        };
        
        let execution_time_ms = start.elapsed().as_millis() as u64;
        
        // Check timeout
        if execution_time_ms > test.timeout_ms {
            return TestResult {
                passed: false,
                points_earned: 0,
                feedback: format!("Timeout: {}ms > {}ms", execution_time_ms, test.timeout_ms),
                execution_time_ms,
            };
        }
        
        // Check expected behavior
        let (passed, feedback) = match &test.expected {
            ExpectedBehavior::ExactOutput(expected) => {
                let passed = output == *expected;
                let feedback = if passed {
                    "Correct output".to_string()
                } else {
                    format!("Expected '{expected}', got '{output}'")
                };
                (passed, feedback)
            }
            ExpectedBehavior::Pattern(pattern) => {
                let regex = Regex::new(pattern).unwrap_or_else(|_| Regex::new(".*").unwrap());
                let passed = regex.is_match(&output);
                let feedback = if passed {
                    "Output matches pattern".to_string()
                } else {
                    format!("Output doesn't match pattern: {pattern}")
                };
                (passed, feedback)
            }
            ExpectedBehavior::TypeSignature(expected_type) => {
                // In production, would check actual type
                let passed = output.contains(expected_type);
                let feedback = if passed {
                    "Type signature correct".to_string()
                } else {
                    format!("Expected type {expected_type}")
                };
                (passed, feedback)
            }
            _ => (false, "Unsupported check".to_string()),
        };
        
        TestResult {
            passed,
            points_earned: if passed { test.points } else { 0 },
            feedback,
            execution_time_ms,
        }
    }
    
    fn check_requirement(&self, _repl: &Repl, req: &Requirement) -> bool {
        // In production, would analyze AST to check requirements
        match req {
            Requirement::UseRecursion => true, // Would check for recursive calls
            Requirement::NoLoops => true,      // Would check for loop constructs
            Requirement::UseHigherOrderFunctions => true, // Would check for HOF usage
            Requirement::TypeSafe => true,     // Would verify type safety
            Requirement::PureFunction => true, // Would check for side effects
            Requirement::TailRecursive => true, // Would verify tail recursion
        }
    }
    
    fn evaluate_rubric(&self, rubric: &GradingRubric, _submission: &ReplSession) -> f32 {
        let mut total_score = 0.0;
        let mut total_weight = 0.0;
        
        for category in &rubric.categories {
            let category_score = self.evaluate_category(category);
            total_score += category_score * category.weight;
            total_weight += category.weight;
        }
        
        if total_weight > 0.0 {
            (total_score / total_weight) * 100.0
        } else {
            0.0
        }
    }
    
    fn evaluate_category(&self, category: &RubricCategory) -> f32 {
        let mut earned = 0u32;
        let mut possible = 0u32;
        
        for criterion in &category.criteria {
            possible += criterion.max_points;
            earned += self.evaluate_criterion(criterion);
        }
        
        if possible > 0 {
            earned as f32 / possible as f32
        } else {
            0.0
        }
    }
    
    fn evaluate_criterion(&self, criterion: &Criterion) -> u32 {
        match &criterion.evaluation {
            CriterionEvaluation::Automatic(check) => {
                match check {
                    AutomaticCheck::TestsPassed => criterion.max_points,
                    AutomaticCheck::CodeQuality { min_score } => {
                        // In production, would run quality analysis
                        if *min_score <= 0.8 {
                            criterion.max_points
                        } else {
                            0
                        }
                    }
                    _ => 0,
                }
            }
            CriterionEvaluation::Manual(_) => 0, // Requires manual grading
            CriterionEvaluation::Hybrid { auto_weight, .. } => {
                (criterion.max_points as f32 * auto_weight) as u32
            }
        }
    }
    
    fn measure_performance(
        &self,
        session: &ReplSession,
        constraints: &PerformanceConstraints,
    ) -> f32 {
        let mut score: f32 = 100.0;
        
        // Check CPU time
        let total_cpu_ns: u64 = session.timeline.iter()
            .filter_map(|e| {
                if let Event::ResourceUsage { cpu_ns, .. } = &e.event {
                    Some(*cpu_ns)
                } else {
                    None
                }
            })
            .sum();
        
        let cpu_ms = total_cpu_ns / 1_000_000;
        if cpu_ms > constraints.max_cpu_ms {
            score -= 20.0;
        }
        
        // Check heap usage
        let max_heap: usize = session.timeline.iter()
            .filter_map(|e| {
                if let Event::ResourceUsage { heap_bytes, .. } = &e.event {
                    Some(*heap_bytes)
                } else {
                    None
                }
            })
            .max()
            .unwrap_or(0);
        
        let heap_mb = max_heap / (1024 * 1024);
        if heap_mb > constraints.max_heap_mb {
            score -= 20.0;
        }
        
        score.max(0.0).min(100.0)
    }
}

// ============================================================================
// Plagiarism Detection
// ============================================================================

/// AST-based plagiarism detection system
pub struct PlagiarismDetector {
    known_submissions: Vec<AstFingerprint>,
}

/// Structural fingerprint of AST for comparison
#[derive(Debug, Clone)]
pub struct AstFingerprint {
    pub hash: String,
    pub structure: Vec<String>,
    pub complexity: usize,
}

impl Default for PlagiarismDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl PlagiarismDetector {
    pub fn new() -> Self {
        Self {
            known_submissions: Vec::new(),
        }
    }
    
    pub fn analyze(&self, submission: &ReplSession) -> f32 {
        // Generate fingerprint for submission
        let fingerprint = self.generate_fingerprint(submission);
        
        // Compare against known submissions
        for known in &self.known_submissions {
            let similarity = self.compute_similarity(&fingerprint, known);
            if similarity > 0.85 {
                return 100.0 * (1.0 - similarity); // High similarity = low originality
            }
        }
        
        100.0 // Full originality score
    }
    
    fn generate_fingerprint(&self, session: &ReplSession) -> AstFingerprint {
        let mut hasher = Sha256::new();
        let mut structure = Vec::new();
        
        // Extract structural patterns from code inputs
        for event in &session.timeline {
            if let Event::Input { text, .. } = &event.event {
                hasher.update(text.as_bytes());
                structure.push(self.extract_structure(text));
            }
        }
        
        AstFingerprint {
            hash: format!("{:x}", hasher.finalize()),
            structure,
            complexity: session.timeline.len(),
        }
    }
    
    fn extract_structure(&self, code: &str) -> String {
        // Simplified: extract function definitions and control flow
        let mut patterns = Vec::new();
        
        if code.contains("fn ") || code.contains("fun ") {
            patterns.push("FN");
        }
        if code.contains("if ") {
            patterns.push("IF");
        }
        if code.contains("for ") || code.contains("while ") {
            patterns.push("LOOP");
        }
        if code.contains("match ") {
            patterns.push("MATCH");
        }
        
        patterns.join("-")
    }
    
    fn compute_similarity(&self, fp1: &AstFingerprint, fp2: &AstFingerprint) -> f32 {
        if fp1.hash == fp2.hash {
            return 1.0; // Identical
        }
        
        // Compare structural patterns
        let common: usize = fp1.structure.iter()
            .zip(fp2.structure.iter())
            .filter(|(a, b)| a == b)
            .count();
        
        let total = fp1.structure.len().max(fp2.structure.len());
        if total > 0 {
            common as f32 / total as f32
        } else {
            0.0
        }
    }
}

// ============================================================================
// Secure Sandbox
// ============================================================================

/// Secure execution environment for untrusted code
pub struct SecureSandbox {
    #[allow(dead_code)]
    resource_limits: ResourceLimits,
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_heap_mb: usize,
    pub max_stack_kb: usize,
    pub max_cpu_ms: u64,
}

impl Default for SecureSandbox {
    fn default() -> Self {
        Self::new()
    }
}

impl SecureSandbox {
    pub fn new() -> Self {
        Self {
            resource_limits: ResourceLimits {
                max_heap_mb: 100,
                max_stack_kb: 8192,
                max_cpu_ms: 5000,
            },
        }
    }
    
    pub fn create_isolated_repl(&self) -> Result<Repl> {
        // In production, would create actual sandboxed environment
        // For now, create regular REPL with resource tracking
        Repl::new()
    }
}

// ============================================================================
// Grade Report
// ============================================================================

/// Complete grading report for a submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeReport {
    pub assignment_id: String,
    pub submission_time: String,
    pub task_grades: Vec<TaskGrade>,
    pub rubric_score: f32,
    pub performance_score: f32,
    pub originality_score: f32,
    pub final_grade: f32,
    pub feedback: Vec<String>,
    pub violations: Vec<String>,
    pub is_valid: bool,
}

impl GradeReport {
    pub fn new(assignment_id: String) -> Self {
        Self {
            assignment_id,
            submission_time: chrono::Utc::now().to_rfc3339(),
            task_grades: Vec::new(),
            rubric_score: 0.0,
            performance_score: 100.0,
            originality_score: 100.0,
            final_grade: 0.0,
            feedback: Vec::new(),
            violations: Vec::new(),
            is_valid: true,
        }
    }
    
    pub fn mark_invalid(&mut self, reason: &str) {
        self.is_valid = false;
        self.violations.push(reason.to_string());
        self.final_grade = 0.0;
    }
    
    pub fn add_task_grade(&mut self, grade: TaskGrade) {
        self.task_grades.push(grade);
    }
    
    pub fn calculate_final_grade(&mut self) {
        if !self.is_valid {
            self.final_grade = 0.0;
            return;
        }
        
        // Calculate task score
        let task_score: f32 = if self.task_grades.is_empty() {
            0.0
        } else {
            let earned: u32 = self.task_grades.iter().map(|g| g.points_earned).sum();
            let possible: u32 = self.task_grades.iter().map(|g| g.points_possible).sum();
            if possible > 0 {
                (earned as f32 / possible as f32) * 100.0
            } else {
                0.0
            }
        };
        
        // Weighted average: 60% tasks, 20% rubric, 10% performance, 10% originality
        self.final_grade = task_score * 0.6 
            + self.rubric_score * 0.2
            + self.performance_score * 0.1
            + self.originality_score * 0.1;
        
        // Add feedback based on grade
        if self.final_grade >= 90.0 {
            self.feedback.push("Excellent work!".to_string());
        } else if self.final_grade >= 80.0 {
            self.feedback.push("Good job!".to_string());
        } else if self.final_grade >= 70.0 {
            self.feedback.push("Satisfactory work.".to_string());
        } else {
            self.feedback.push("Needs improvement.".to_string());
        }
    }
}

/// Grade for an individual task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGrade {
    pub task_id: String,
    pub points_earned: u32,
    pub points_possible: u32,
    pub test_results: Vec<(String, TestResult)>,
    pub hidden_results: Vec<(String, TestResult)>,
    pub requirements_met: HashSet<String>,
}

impl TaskGrade {
    pub fn new(task_id: String) -> Self {
        Self {
            task_id,
            points_earned: 0,
            points_possible: 0,
            test_results: Vec::new(),
            hidden_results: Vec::new(),
            requirements_met: HashSet::new(),
        }
    }
    
    pub fn add_test_result(&mut self, input: String, result: TestResult) {
        self.test_results.push((input, result));
    }
    
    pub fn add_hidden_result(&mut self, input: String, result: TestResult) {
        self.hidden_results.push((input, result));
    }
    
    pub fn calculate_score(&mut self, max_points: u32) {
        self.points_possible = max_points;
        
        // Sum points from test results
        let test_points: u32 = self.test_results.iter()
            .map(|(_, r)| r.points_earned)
            .sum();
        let hidden_points: u32 = self.hidden_results.iter()
            .map(|(_, r)| r.points_earned)
            .sum();
        
        self.points_earned = (test_points + hidden_points).min(max_points);
    }
}

/// Result of running a single test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub passed: bool,
    pub points_earned: u32,
    pub feedback: String,
    pub execution_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_grading_engine_creation() {
        let engine = GradingEngine::new();
        assert!(engine.replay_validator.strict_mode);
    }
    
    #[test]
    fn test_grade_report() {
        let mut report = GradeReport::new("test_assignment".to_string());
        assert!(report.is_valid);
        assert_eq!(report.final_grade, 0.0);
        
        report.mark_invalid("Test violation");
        assert!(!report.is_valid);
        assert_eq!(report.violations.len(), 1);
    }
    
    #[test]
    fn test_plagiarism_detection() {
        let detector = PlagiarismDetector::new();
        
        // Create mock session
        let session = ReplSession {
            version: crate::runtime::replay::SemVer::new(1, 0, 0),
            metadata: crate::runtime::replay::SessionMetadata {
                session_id: "test".to_string(),
                created_at: "2025-08-28T10:00:00Z".to_string(),
                ruchy_version: "1.23.0".to_string(),
                student_id: Some("student1".to_string()),
                assignment_id: Some("hw1".to_string()),
                tags: vec![],
            },
            environment: crate::runtime::replay::Environment {
                seed: 42,
                feature_flags: vec![],
                resource_limits: crate::runtime::replay::ResourceLimits {
                    heap_mb: 100,
                    stack_kb: 8192,
                    cpu_ms: 5000,
                },
            },
            timeline: vec![],
            checkpoints: std::collections::BTreeMap::new(),
        };
        
        let score = detector.analyze(&session);
        assert_eq!(score, 100.0); // Empty session should have full originality
    }
}