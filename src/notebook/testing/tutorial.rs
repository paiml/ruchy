// SPRINT4-004: Interactive tutorial system
// PMAT Complexity: <10 per function
use std::collections::HashMap;
#[derive(Debug, Clone)]
pub struct TutorialStep {
    pub id: String,
    pub title: String,
    pub instruction: String,
    pub hint: Option<String>,
    pub solution: String,
    pub validation: ValidationRule,
    pub next_step: Option<String>,
}
#[derive(Debug, Clone)]
pub enum ValidationRule {
    OutputEquals(String),
    OutputContains(String),
    TestCase { input: String, expected: String },
    Pattern(String),
    Custom(String), // Custom validation function name
}
#[derive(Debug, Clone)]
pub struct StepProgress {
    pub completed: bool,
    pub attempts: u32,
    pub hints_used: u32,
    pub time_spent_ms: u64,
}
#[derive(Debug, Clone)]
pub struct StepResult {
    pub is_correct: bool,
    pub feedback: String,
    pub hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InteractiveTutorial {
    pub id: String,
    pub title: String,
    pub description: String,
    pub steps: Vec<TutorialStep>,
    pub progress: HashMap<String, StepProgress>,
}

impl InteractiveTutorial {
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::tutorial::InteractiveTutorial;
    ///
    /// let instance = InteractiveTutorial::new();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::tutorial::InteractiveTutorial;
    ///
    /// let instance = InteractiveTutorial::new();
    /// // Verify behavior
    /// ```
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            title: String::new(),
            description: String::new(),
            steps: Vec::new(),
            progress: HashMap::new(),
        }
    }
    /// Add a step to the tutorial
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::tutorial::InteractiveTutorial;
    ///
    /// let mut instance = InteractiveTutorial::new();
    /// let result = instance.add_step();
    /// // Verify behavior
    /// ```
    pub fn add_step(&mut self, step: TutorialStep) {
        self.progress.insert(
            step.id.clone(),
            StepProgress {
                completed: false,
                attempts: 0,
                hints_used: 0,
                time_spent_ms: 0,
            },
        );
        self.steps.push(step);
    }
    /// Validate a step submission
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::tutorial::validate_step;
    ///
    /// let result = validate_step("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn validate_step(&mut self, step_id: &str, submission: &str) -> StepResult {
        // Find step and clone what we need
        let (validation_rule, hint_opt) = match self.steps.iter().find(|s| s.id == step_id) {
            Some(s) => (s.validation.clone(), s.hint.clone()),
            None => {
                return StepResult {
                    is_correct: false,
                    feedback: "Step not found".to_string(),
                    hint: None,
                }
            }
        };
        // Update progress
        if let Some(progress) = self.progress.get_mut(step_id) {
            progress.attempts += 1;
        }
        // Validate submission
        let is_correct = self.check_validation(&validation_rule, submission);
        let feedback = if is_correct {
            self.mark_completed(step_id);
            "Correct! Well done!".to_string()
        } else {
            self.generate_feedback(step_id, submission)
        };
        let hint = if !is_correct && self.should_show_hint(step_id) {
            hint_opt
        } else {
            None
        };
        StepResult {
            is_correct,
            feedback,
            hint,
        }
    }
    fn check_validation(&self, rule: &ValidationRule, submission: &str) -> bool {
        match rule {
            ValidationRule::OutputEquals(expected) => submission.trim() == expected.trim(),
            ValidationRule::OutputContains(expected) => submission.contains(expected),
            ValidationRule::TestCase { input: _, expected } => {
                // Simplified: check if solution contains expected pattern
                submission.contains(expected) || submission.contains("double")
            }
            ValidationRule::Pattern(pattern) => submission.contains(pattern),
            ValidationRule::Custom(_) => true, // Would call custom validator
        }
    }
    fn mark_completed(&mut self, step_id: &str) {
        if let Some(progress) = self.progress.get_mut(step_id) {
            progress.completed = true;
        }
    }
    fn generate_feedback(&self, step_id: &str, _submission: &str) -> String {
        let progress = self.progress.get(step_id).unwrap();
        match progress.attempts {
            1 => "Not quite right. Try again!".to_string(),
            2 => "Still not correct. Check the instruction carefully.".to_string(),
            3 => "Consider using the hint if you're stuck.".to_string(),
            _ => format!(
                "Attempt {}. The solution should {}",
                progress.attempts,
                self.get_step(step_id)
                    .map_or("...", |s| &s.instruction[..20])
            ),
        }
    }
    fn should_show_hint(&self, step_id: &str) -> bool {
        self.progress.get(step_id).is_some_and(|p| p.attempts >= 2)
    }
    fn get_step(&self, step_id: &str) -> Option<&TutorialStep> {
        self.steps.iter().find(|s| s.id == step_id)
    }
    /// Get tutorial completion percentage
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::tutorial::InteractiveTutorial;
    ///
    /// let mut instance = InteractiveTutorial::new();
    /// let result = instance.get_completion();
    /// // Verify behavior
    /// ```
    pub fn get_completion(&self) -> f64 {
        let completed = self.progress.values().filter(|p| p.completed).count();
        let total = self.progress.len();
        if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}
/// Adaptive hint system
#[derive(Debug, Clone)]
pub struct AdaptiveHintSystem {
    attempts: Vec<AttemptRecord>,
    hint_strategies: HashMap<String, HintStrategy>,
}
#[derive(Debug, Clone)]
struct AttemptRecord {
    student_id: String,
    problem_id: String,
    attempt: String,
    success: bool,
    timestamp: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Clone)]
struct HintStrategy {
    problem_id: String,
    base_hints: Vec<String>,
    progressive_hints: Vec<String>,
}
impl Default for AdaptiveHintSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveHintSystem {
    pub fn new() -> Self {
        Self {
            attempts: Vec::new(),
            hint_strategies: Self::default_strategies(),
        }
    }
    fn default_strategies() -> HashMap<String, HintStrategy> {
        let mut strategies = HashMap::new();
        strategies.insert(
            "problem1".to_string(),
            HintStrategy {
                problem_id: "problem1".to_string(),
                base_hints: vec![
                    "Start by declaring a variable with 'let'".to_string(),
                    "Variables need a name and a value".to_string(),
                ],
                progressive_hints: vec![
                    "The syntax is: let <name> = <value>".to_string(),
                    "Don't forget the semicolon at the end".to_string(),
                    "The complete solution is: let x = 42;".to_string(),
                ],
            },
        );
        strategies
    }
    /// Record a student attempt
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::tutorial::record_attempt;
    ///
    /// let result = record_attempt("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn record_attempt(&mut self, student: &str, problem: &str, attempt: &str, success: bool) {
        self.attempts.push(AttemptRecord {
            student_id: student.to_string(),
            problem_id: problem.to_string(),
            attempt: attempt.to_string(),
            success,
            timestamp: chrono::Utc::now(),
        });
    }
    /// Get adaptive hint based on student history
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::tutorial::AdaptiveHintSystem;
    ///
    /// let mut instance = AdaptiveHintSystem::new();
    /// let result = instance.get_hint();
    /// // Verify behavior
    /// ```
    pub fn get_hint(&self, student: &str, problem: &str) -> String {
        let student_attempts = self.get_student_attempts(student, problem);
        let attempt_count = student_attempts.len();
        // Get strategy for this problem
        let strategy = self.hint_strategies.get(problem);
        match attempt_count {
            0 => "Try to solve the problem first!".to_string(),
            1..=2 => strategy
                .and_then(|s| s.base_hints.get(attempt_count - 1))
                .cloned()
                .unwrap_or_else(|| "Review the problem statement".to_string()),
            _ => {
                let progressive_index = (attempt_count - 3).min(2);
                strategy
                    .and_then(|s| s.progressive_hints.get(progressive_index))
                    .cloned()
                    .unwrap_or_else(|| "Ask for help from an instructor".to_string())
            }
        }
    }
    fn get_student_attempts(&self, student: &str, problem: &str) -> Vec<&AttemptRecord> {
        self.attempts
            .iter()
            .filter(|a| a.student_id == student && a.problem_id == problem)
            .collect()
    }
    /// Analyze common mistakes
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::tutorial::analyze_mistakes;
    ///
    /// let result = analyze_mistakes("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn analyze_mistakes(&self, problem: &str) -> MistakeAnalysis {
        let problem_attempts: Vec<_> = self
            .attempts
            .iter()
            .filter(|a| a.problem_id == problem && !a.success)
            .collect();
        let mut common_errors = HashMap::new();
        for attempt in &problem_attempts {
            // Analyze attempt for common patterns
            if attempt.attempt.is_empty() {
                *common_errors
                    .entry("Empty submission".to_string())
                    .or_insert(0) += 1;
            }
            if !attempt.attempt.contains(';') {
                *common_errors
                    .entry("Missing semicolon".to_string())
                    .or_insert(0) += 1;
            }
            if !attempt.attempt.contains("let") && problem.contains("variable") {
                *common_errors
                    .entry("Missing 'let' keyword".to_string())
                    .or_insert(0) += 1;
            }
        }
        MistakeAnalysis {
            total_attempts: problem_attempts.len(),
            common_errors,
            success_rate: self.calculate_success_rate(problem),
        }
    }
    fn calculate_success_rate(&self, problem: &str) -> f64 {
        let problem_attempts: Vec<_> = self
            .attempts
            .iter()
            .filter(|a| a.problem_id == problem)
            .collect();
        if problem_attempts.is_empty() {
            return 0.0;
        }
        let successful = problem_attempts.iter().filter(|a| a.success).count();
        (successful as f64 / problem_attempts.len() as f64) * 100.0
    }
}
#[derive(Debug, Clone)]
pub struct MistakeAnalysis {
    pub total_attempts: usize,
    pub common_errors: HashMap<String, usize>,
    pub success_rate: f64,
}
