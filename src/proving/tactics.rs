//! Proof tactics library with ML-powered suggestions
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::prover::{ProofGoal, ProofContext, StepResult};
/// A proof tactic
pub trait Tactic: Send + Sync {
    /// Get tactic name
    fn name(&self) -> &str;
    /// Get tactic description
    fn description(&self) -> &str;
    /// Apply the tactic to a goal
    fn apply(&self, goal: &ProofGoal, args: &[&str], context: &ProofContext) -> Result<StepResult>;
    /// Check if tactic is applicable
    fn is_applicable(&self, goal: &ProofGoal, context: &ProofContext) -> bool;
}
/// Library of available tactics
pub struct TacticLibrary {
    /// Available tactics
    tactics: HashMap<String, Box<dyn Tactic>>,
    /// ML model for suggestions (placeholder)
    _suggestion_model: SuggestionModel,
}
/// ML model for tactic suggestions
struct SuggestionModel {
    // Placeholder for ML model
}
/// Tactic suggestion with confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticSuggestion {
    /// Tactic name
    pub tactic_name: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Reason for suggestion
    pub reason: Option<String>,
    /// Suggested arguments
    pub arguments: Vec<String>,
}
impl TacticLibrary {
    /// Create default tactic library
/// # Examples
/// 
/// ```
/// use ruchy::proving::tactics::TacticLibrary;
/// 
/// let mut instance = TacticLibrary::new();
/// let result = instance.default();
/// // Verify behavior
/// ```
pub fn default() -> Self {
        let mut tactics = HashMap::new();
        // Add basic tactics
        tactics.insert("intro".to_string(), Box::new(IntroTactic) as Box<dyn Tactic>);
        tactics.insert("split".to_string(), Box::new(SplitTactic) as Box<dyn Tactic>);
        tactics.insert("induction".to_string(), Box::new(InductionTactic) as Box<dyn Tactic>);
        tactics.insert("contradiction".to_string(), Box::new(ContradictionTactic) as Box<dyn Tactic>);
        tactics.insert("reflexivity".to_string(), Box::new(ReflexivityTactic) as Box<dyn Tactic>);
        tactics.insert("simplify".to_string(), Box::new(SimplifyTactic) as Box<dyn Tactic>);
        tactics.insert("unfold".to_string(), Box::new(UnfoldTactic) as Box<dyn Tactic>);
        tactics.insert("rewrite".to_string(), Box::new(RewriteTactic) as Box<dyn Tactic>);
        tactics.insert("apply".to_string(), Box::new(ApplyTactic) as Box<dyn Tactic>);
        tactics.insert("assumption".to_string(), Box::new(AssumptionTactic) as Box<dyn Tactic>);
        Self {
            tactics,
            _suggestion_model: SuggestionModel {},
        }
    }
    /// Get all tactics
/// # Examples
/// 
/// ```
/// use ruchy::proving::tactics::TacticLibrary;
/// 
/// let mut instance = TacticLibrary::new();
/// let result = instance.all_tactics();
/// // Verify behavior
/// ```
pub fn all_tactics(&self) -> Vec<&dyn Tactic> {
        self.tactics.values().map(std::convert::AsRef::as_ref).collect()
    }
    /// Get a specific tactic
/// # Examples
/// 
/// ```ignore
/// use ruchy::proving::tactics::get_tactic;
/// 
/// let result = get_tactic("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn get_tactic(&self, name: &str) -> Result<&dyn Tactic> {
        self.tactics.get(name)
            .map(std::convert::AsRef::as_ref)
            .ok_or_else(|| anyhow::anyhow!("Unknown tactic: {}", name))
    }
    /// Suggest tactics for a goal
/// # Examples
/// 
/// ```ignore
/// use ruchy::proving::tactics::suggest_tactics;
/// 
/// let result = suggest_tactics(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn suggest_tactics(&self, goal: &ProofGoal, context: &ProofContext) -> Result<Vec<TacticSuggestion>> {
        let mut suggestions = Vec::new();
        // Check each tactic's applicability
        for (name, tactic) in &self.tactics {
            if tactic.is_applicable(goal, context) {
                let confidence = self.calculate_confidence(goal, tactic.as_ref());
                suggestions.push(TacticSuggestion {
                    tactic_name: name.clone(),
                    confidence,
                    reason: Some(format!("Pattern matches {}", tactic.description())),
                    arguments: Vec::new(),
                });
            }
        }
        // Sort by confidence
        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        Ok(suggestions)
    }
    /// Calculate confidence for a tactic
    fn calculate_confidence(&self, goal: &ProofGoal, _tactic: &dyn Tactic) -> f64 {
        // Simple heuristic-based confidence
        match &goal.statement {
            s if s.contains("->") => 0.8,  // Implication
            s if s.contains("&&") => 0.7,  // Conjunction
            s if s.contains("||") => 0.6,  // Disjunction
            s if s.contains("==") => 0.9,  // Equality
            _ => 0.5,
        }
    }
}
// Basic tactic implementations
/// Introduction tactic (for implications)
struct IntroTactic;
impl Tactic for IntroTactic {
    fn name(&self) -> &'static str { "intro" }
    fn description(&self) -> &'static str { "Introduce hypothesis from implication" }
    fn apply(&self, goal: &ProofGoal, _args: &[&str], _context: &ProofContext) -> Result<StepResult> {
        if goal.statement.contains("->") {
            let parts: Vec<&str> = goal.statement.split("->").collect();
            if parts.len() == 2 {
                return Ok(StepResult::Simplified(parts[1].trim().to_string()));
            }
        }
        Ok(StepResult::Failed("Cannot apply intro".to_string()))
    }
    fn is_applicable(&self, goal: &ProofGoal, _context: &ProofContext) -> bool {
        goal.statement.contains("->")
    }
}
/// Split tactic (for conjunctions)
struct SplitTactic;
impl Tactic for SplitTactic {
    fn name(&self) -> &'static str { "split" }
    fn description(&self) -> &'static str { "Split conjunction into subgoals" }
    fn apply(&self, goal: &ProofGoal, _args: &[&str], _context: &ProofContext) -> Result<StepResult> {
        if goal.statement.contains("&&") {
            let parts: Vec<&str> = goal.statement.split("&&").collect();
            let subgoals: Vec<String> = parts.iter().map(|p| p.trim().to_string()).collect();
            return Ok(StepResult::Subgoals(subgoals));
        }
        Ok(StepResult::Failed("Cannot apply split".to_string()))
    }
    fn is_applicable(&self, goal: &ProofGoal, _context: &ProofContext) -> bool {
        goal.statement.contains("&&")
    }
}
/// Induction tactic
struct InductionTactic;
impl Tactic for InductionTactic {
    fn name(&self) -> &'static str { "induction" }
    fn description(&self) -> &'static str { "Proof by induction" }
    fn apply(&self, goal: &ProofGoal, args: &[&str], _context: &ProofContext) -> Result<StepResult> {
        if args.is_empty() {
            return Ok(StepResult::Failed("Induction requires a variable".to_string()));
        }
        let var = args[0];
        Ok(StepResult::Subgoals(vec![
            format!("Base case: {} when {} = 0", goal.statement, var),
            format!("Inductive step: {} implies {}", 
                goal.statement.replace(var, &var.to_string()),
                goal.statement.replace(var, &format!("{var}+1"))
            ),
        ]))
    }
    fn is_applicable(&self, goal: &ProofGoal, _context: &ProofContext) -> bool {
        goal.statement.contains("forall") || goal.statement.contains('n')
    }
}
/// Contradiction tactic
struct ContradictionTactic;
impl Tactic for ContradictionTactic {
    fn name(&self) -> &'static str { "contradiction" }
    fn description(&self) -> &'static str { "Proof by contradiction" }
    fn apply(&self, _goal: &ProofGoal, _args: &[&str], context: &ProofContext) -> Result<StepResult> {
        // Check for contradictory assumptions
        for assumption in &context.assumptions {
            if assumption.contains('!') {
                let negated = assumption.replace('!', "");
                if context.assumptions.contains(&negated) {
                    return Ok(StepResult::Solved);
                }
            }
        }
        Ok(StepResult::Failed("No contradiction found".to_string()))
    }
    fn is_applicable(&self, _goal: &ProofGoal, context: &ProofContext) -> bool {
        context.assumptions.len() >= 2
    }
}
/// Reflexivity tactic
struct ReflexivityTactic;
impl Tactic for ReflexivityTactic {
    fn name(&self) -> &'static str { "reflexivity" }
    fn description(&self) -> &'static str { "Prove equality by reflexivity" }
    fn apply(&self, goal: &ProofGoal, _args: &[&str], _context: &ProofContext) -> Result<StepResult> {
        if goal.statement.contains("==") {
            let parts: Vec<&str> = goal.statement.split("==").collect();
            if parts.len() == 2 && parts[0].trim() == parts[1].trim() {
                return Ok(StepResult::Solved);
            }
        }
        Ok(StepResult::Failed("Terms are not equal".to_string()))
    }
    fn is_applicable(&self, goal: &ProofGoal, _context: &ProofContext) -> bool {
        goal.statement.contains("==")
    }
}
/// Simplify tactic
struct SimplifyTactic;
impl Tactic for SimplifyTactic {
    fn name(&self) -> &'static str { "simplify" }
    fn description(&self) -> &'static str { "Simplify expression" }
    fn apply(&self, goal: &ProofGoal, _args: &[&str], _context: &ProofContext) -> Result<StepResult> {
        let mut simplified = goal.statement.clone();
        // Basic simplifications
        simplified = simplified.replace("true && ", "");
        simplified = simplified.replace(" && true", "");
        simplified = simplified.replace("false || ", "");
        simplified = simplified.replace(" || false", "");
        simplified = simplified.replace("!!!", "!");
        simplified = simplified.replace("!!", "");
        if simplified == goal.statement {
            Ok(StepResult::Failed("No simplification possible".to_string()))
        } else {
            Ok(StepResult::Simplified(simplified))
        }
    }
    fn is_applicable(&self, _goal: &ProofGoal, _context: &ProofContext) -> bool {
        true
    }
}
/// Unfold tactic
struct UnfoldTactic;
impl Tactic for UnfoldTactic {
    fn name(&self) -> &'static str { "unfold" }
    fn description(&self) -> &'static str { "Unfold definition" }
    fn apply(&self, goal: &ProofGoal, args: &[&str], context: &ProofContext) -> Result<StepResult> {
        if args.is_empty() {
            return Ok(StepResult::Failed("Unfold requires a definition name".to_string()));
        }
        let def_name = args[0];
        if let Some(definition) = context.definitions.get(def_name) {
            let unfolded = goal.statement.replace(def_name, definition);
            Ok(StepResult::Simplified(unfolded))
        } else {
            Ok(StepResult::Failed(format!("Unknown definition: {def_name}")))
        }
    }
    fn is_applicable(&self, _goal: &ProofGoal, context: &ProofContext) -> bool {
        !context.definitions.is_empty()
    }
}
/// Rewrite tactic
struct RewriteTactic;
impl Tactic for RewriteTactic {
    fn name(&self) -> &'static str { "rewrite" }
    fn description(&self) -> &'static str { "Rewrite using equality" }
    fn apply(&self, goal: &ProofGoal, args: &[&str], context: &ProofContext) -> Result<StepResult> {
        if args.is_empty() {
            return Ok(StepResult::Failed("Rewrite requires an equality".to_string()));
        }
        // Find equality in assumptions
        for assumption in &context.assumptions {
            if assumption.contains("==") {
                let parts: Vec<&str> = assumption.split("==").collect();
                if parts.len() == 2 {
                    let lhs = parts[0].trim();
                    let rhs = parts[1].trim();
                    let rewritten = goal.statement.replace(lhs, rhs);
                    if rewritten != goal.statement {
                        return Ok(StepResult::Simplified(rewritten));
                    }
                }
            }
        }
        Ok(StepResult::Failed("No applicable rewrite found".to_string()))
    }
    fn is_applicable(&self, _goal: &ProofGoal, context: &ProofContext) -> bool {
        context.assumptions.iter().any(|a| a.contains("=="))
    }
}
/// Apply tactic
struct ApplyTactic;
impl Tactic for ApplyTactic {
    fn name(&self) -> &'static str { "apply" }
    fn description(&self) -> &'static str { "Apply theorem or lemma" }
    fn apply(&self, _goal: &ProofGoal, args: &[&str], _context: &ProofContext) -> Result<StepResult> {
        if args.is_empty() {
            return Ok(StepResult::Failed("Apply requires a theorem name".to_string()));
        }
        // In a real implementation, this would look up theorems
        Ok(StepResult::Failed(format!("Cannot apply theorem: {}", args[0])))
    }
    fn is_applicable(&self, _goal: &ProofGoal, _context: &ProofContext) -> bool {
        true
    }
}
/// Assumption tactic
struct AssumptionTactic;
impl Tactic for AssumptionTactic {
    fn name(&self) -> &'static str { "assumption" }
    fn description(&self) -> &'static str { "Prove using an assumption" }
    fn apply(&self, goal: &ProofGoal, _args: &[&str], context: &ProofContext) -> Result<StepResult> {
        if context.assumptions.contains(&goal.statement) {
            Ok(StepResult::Solved)
        } else {
            Ok(StepResult::Failed("Goal not in assumptions".to_string()))
        }
    }
    fn is_applicable(&self, goal: &ProofGoal, context: &ProofContext) -> bool {
        context.assumptions.contains(&goal.statement)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_goal(statement: &str) -> ProofGoal {
        ProofGoal {
            statement: statement.to_string(),
        }
    }

    fn create_test_context() -> ProofContext {
        let mut context = ProofContext::new();
        context.assumptions.push("x > 0".to_string());
        context.assumptions.push("y < 10".to_string());
        context.definitions.insert("double".to_string(), "x * 2".to_string());
        context
    }

    #[test]
    fn test_tactic_library_default() {
        let library = TacticLibrary::default();
        assert!(library.tactics.len() >= 10); // Should have basic tactics
        assert!(library.tactics.contains_key("intro"));
        assert!(library.tactics.contains_key("split"));
        assert!(library.tactics.contains_key("reflexivity"));
    }

    #[test]
    fn test_tactic_library_all_tactics() {
        let library = TacticLibrary::default();
        let tactics = library.all_tactics();
        assert!(tactics.len() >= 10);

        let names: Vec<&str> = tactics.iter().map(|t| t.name()).collect();
        assert!(names.contains(&"intro"));
        assert!(names.contains(&"split"));
        assert!(names.contains(&"assumption"));
    }

    #[test]
    fn test_tactic_library_get_tactic() {
        let library = TacticLibrary::default();

        let intro = library.get_tactic("intro").unwrap();
        assert_eq!(intro.name(), "intro");

        let result = library.get_tactic("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_intro_tactic_applicable() {
        let tactic = IntroTactic;
        let goal = create_test_goal("A -> B");
        let context = create_test_context();

        assert!(tactic.is_applicable(&goal, &context));

        let goal2 = create_test_goal("A && B");
        assert!(!tactic.is_applicable(&goal2, &context));
    }

    #[test]
    fn test_intro_tactic_apply() {
        let tactic = IntroTactic;
        let goal = create_test_goal("A -> B");
        let context = create_test_context();

        let result = tactic.apply(&goal, &[], &context).unwrap();
        match result {
            StepResult::Simplified(s) => assert_eq!(s, "B"),
            _ => panic!("Expected simplified result"),
        }
    }

    #[test]
    fn test_intro_tactic_fail() {
        let tactic = IntroTactic;
        let goal = create_test_goal("A && B");
        let context = create_test_context();

        let result = tactic.apply(&goal, &[], &context).unwrap();
        match result {
            StepResult::Failed(_) => {},
            _ => panic!("Expected failed result"),
        }
    }

    #[test]
    fn test_split_tactic_applicable() {
        let tactic = SplitTactic;
        let goal = create_test_goal("A && B");
        let context = create_test_context();

        assert!(tactic.is_applicable(&goal, &context));

        let goal2 = create_test_goal("A -> B");
        assert!(!tactic.is_applicable(&goal2, &context));
    }

    #[test]
    fn test_split_tactic_apply() {
        let tactic = SplitTactic;
        let goal = create_test_goal("A && B");
        let context = create_test_context();

        let result = tactic.apply(&goal, &[], &context).unwrap();
        match result {
            StepResult::Subgoals(subgoals) => {
                assert_eq!(subgoals.len(), 2);
                assert_eq!(subgoals[0], "A");
                assert_eq!(subgoals[1], "B");
            },
            _ => panic!("Expected subgoals result"),
        }
    }

    #[test]
    fn test_split_tactic_multiple_conjunctions() {
        let tactic = SplitTactic;
        let goal = create_test_goal("A && B && C");
        let context = create_test_context();

        let result = tactic.apply(&goal, &[], &context).unwrap();
        match result {
            StepResult::Subgoals(subgoals) => {
                assert_eq!(subgoals.len(), 3);
                assert_eq!(subgoals[0], "A");
                assert_eq!(subgoals[1], "B");
                assert_eq!(subgoals[2], "C");
            },
            _ => panic!("Expected subgoals result"),
        }
    }

    #[test]
    fn test_induction_tactic_applicable() {
        let tactic = InductionTactic;
        let goal1 = create_test_goal("forall n. P(n)");
        let goal2 = create_test_goal("P(n) for all n");
        let goal3 = create_test_goal("A && B");
        let context = create_test_context();

        assert!(tactic.is_applicable(&goal1, &context));
        assert!(tactic.is_applicable(&goal2, &context));
        assert!(!tactic.is_applicable(&goal3, &context));
    }

    #[test]
    fn test_induction_tactic_apply_with_var() {
        let tactic = InductionTactic;
        let goal = create_test_goal("P(n)");
        let context = create_test_context();

        let result = tactic.apply(&goal, &["n"], &context).unwrap();
        match result {
            StepResult::Subgoals(subgoals) => {
                assert_eq!(subgoals.len(), 2);
                assert!(subgoals[0].contains("Base case"));
                assert!(subgoals[1].contains("Inductive step"));
            },
            _ => panic!("Expected subgoals result"),
        }
    }

    #[test]
    fn test_induction_tactic_no_var() {
        let tactic = InductionTactic;
        let goal = create_test_goal("P(n)");
        let context = create_test_context();

        let result = tactic.apply(&goal, &[], &context).unwrap();
        match result {
            StepResult::Failed(msg) => assert!(msg.contains("requires a variable")),
            _ => panic!("Expected failed result"),
        }
    }

    #[test]
    fn test_contradiction_tactic_applicable() {
        let tactic = ContradictionTactic;
        let goal = create_test_goal("false");
        let context = create_test_context();

        assert!(tactic.is_applicable(&goal, &context));

        let empty_context = ProofContext::new();
        assert!(!tactic.is_applicable(&goal, &empty_context));
    }

    #[test]
    fn test_contradiction_tactic_apply() {
        let tactic = ContradictionTactic;
        let goal = create_test_goal("false");
        let mut context = ProofContext::new();
        context.assumptions.push("A".to_string());
        context.assumptions.push("!A".to_string());

        let result = tactic.apply(&goal, &[], &context).unwrap();
        match result {
            StepResult::Solved => {},
            _ => panic!("Expected solved result"),
        }
    }

    #[test]
    fn test_reflexivity_tactic_applicable() {
        let tactic = ReflexivityTactic;
        let goal1 = create_test_goal("x == x");
        let goal2 = create_test_goal("A && B");
        let context = create_test_context();

        assert!(tactic.is_applicable(&goal1, &context));
        assert!(!tactic.is_applicable(&goal2, &context));
    }

    #[test]
    fn test_reflexivity_tactic_apply_success() {
        let tactic = ReflexivityTactic;
        let goal = create_test_goal("x == x");
        let context = create_test_context();

        let result = tactic.apply(&goal, &[], &context).unwrap();
        match result {
            StepResult::Solved => {},
            _ => panic!("Expected solved result"),
        }
    }

    #[test]
    fn test_reflexivity_tactic_apply_fail() {
        let tactic = ReflexivityTactic;
        let goal = create_test_goal("x == y");
        let context = create_test_context();

        let result = tactic.apply(&goal, &[], &context).unwrap();
        match result {
            StepResult::Failed(_) => {},
            _ => panic!("Expected failed result"),
        }
    }

    #[test]
    fn test_simplify_tactic_applicable() {
        let tactic = SimplifyTactic;
        let goal = create_test_goal("anything");
        let context = create_test_context();

        assert!(tactic.is_applicable(&goal, &context));
    }

    #[test]
    fn test_simplify_tactic_true_and() {
        let tactic = SimplifyTactic;
        let goal = create_test_goal("true && A");
        let context = create_test_context();

        let result = tactic.apply(&goal, &[], &context).unwrap();
        match result {
            StepResult::Simplified(s) => assert_eq!(s, "A"),
            _ => panic!("Expected simplified result"),
        }
    }

    #[test]
    fn test_simplify_tactic_false_or() {
        let tactic = SimplifyTactic;
        let goal = create_test_goal("false || B");
        let context = create_test_context();

        let result = tactic.apply(&goal, &[], &context).unwrap();
        match result {
            StepResult::Simplified(s) => assert_eq!(s, "B"),
            _ => panic!("Expected simplified result"),
        }
    }

    #[test]
    fn test_simplify_tactic_double_negation() {
        let tactic = SimplifyTactic;
        let goal = create_test_goal("!!A");
        let context = create_test_context();

        let result = tactic.apply(&goal, &[], &context).unwrap();
        match result {
            StepResult::Simplified(s) => assert_eq!(s, "A"),
            _ => panic!("Expected simplified result"),
        }
    }

    #[test]
    fn test_simplify_tactic_no_change() {
        let tactic = SimplifyTactic;
        let goal = create_test_goal("A");
        let context = create_test_context();

        let result = tactic.apply(&goal, &[], &context).unwrap();
        match result {
            StepResult::Failed(_) => {},
            _ => panic!("Expected failed result when no simplification possible"),
        }
    }

    #[test]
    fn test_unfold_tactic_applicable() {
        let tactic = UnfoldTactic;
        let goal = create_test_goal("double(x)");
        let context = create_test_context();

        assert!(tactic.is_applicable(&goal, &context));

        let empty_context = ProofContext::new();
        assert!(!tactic.is_applicable(&goal, &empty_context));
    }

    #[test]
    fn test_unfold_tactic_apply() {
        let tactic = UnfoldTactic;
        let goal = create_test_goal("double(x) > 0");
        let context = create_test_context();

        let result = tactic.apply(&goal, &["double"], &context).unwrap();
        match result {
            StepResult::Simplified(s) => assert_eq!(s, "x * 2(x) > 0"),
            _ => panic!("Expected simplified result"),
        }
    }

    #[test]
    fn test_unfold_tactic_no_args() {
        let tactic = UnfoldTactic;
        let goal = create_test_goal("double(x)");
        let context = create_test_context();

        let result = tactic.apply(&goal, &[], &context).unwrap();
        match result {
            StepResult::Failed(msg) => assert!(msg.contains("requires a definition name")),
            _ => panic!("Expected failed result"),
        }
    }

    #[test]
    fn test_rewrite_tactic_applicable() {
        let tactic = RewriteTactic;
        let goal = create_test_goal("x + y");
        let mut context = ProofContext::new();
        context.assumptions.push("x == 5".to_string());

        assert!(tactic.is_applicable(&goal, &context));

        let empty_context = ProofContext::new();
        assert!(!tactic.is_applicable(&goal, &empty_context));
    }

    #[test]
    fn test_rewrite_tactic_apply() {
        let tactic = RewriteTactic;
        let goal = create_test_goal("x + y");
        let mut context = ProofContext::new();
        context.assumptions.push("x == 5".to_string());

        let result = tactic.apply(&goal, &["x"], &context).unwrap();
        match result {
            StepResult::Simplified(s) => assert_eq!(s, "5 + y"),
            _ => panic!("Expected simplified result"),
        }
    }

    #[test]
    fn test_apply_tactic_applicable() {
        let tactic = ApplyTactic;
        let goal = create_test_goal("P(x)");
        let context = create_test_context();

        assert!(tactic.is_applicable(&goal, &context));
    }

    #[test]
    fn test_apply_tactic_no_args() {
        let tactic = ApplyTactic;
        let goal = create_test_goal("P(x)");
        let context = create_test_context();

        let result = tactic.apply(&goal, &[], &context).unwrap();
        match result {
            StepResult::Failed(msg) => assert!(msg.contains("requires a theorem name")),
            _ => panic!("Expected failed result"),
        }
    }

    #[test]
    fn test_assumption_tactic_applicable() {
        let tactic = AssumptionTactic;
        let goal = create_test_goal("x > 0");
        let context = create_test_context();

        assert!(tactic.is_applicable(&goal, &context));

        let goal2 = create_test_goal("z < 0");
        assert!(!tactic.is_applicable(&goal2, &context));
    }

    #[test]
    fn test_assumption_tactic_apply_success() {
        let tactic = AssumptionTactic;
        let goal = create_test_goal("x > 0");
        let context = create_test_context();

        let result = tactic.apply(&goal, &[], &context).unwrap();
        match result {
            StepResult::Solved => {},
            _ => panic!("Expected solved result"),
        }
    }

    #[test]
    fn test_assumption_tactic_apply_fail() {
        let tactic = AssumptionTactic;
        let goal = create_test_goal("z < 0");
        let context = create_test_context();

        let result = tactic.apply(&goal, &[], &context).unwrap();
        match result {
            StepResult::Failed(_) => {},
            _ => panic!("Expected failed result"),
        }
    }

    #[test]
    fn test_tactic_suggestion_creation() {
        let suggestion = TacticSuggestion {
            tactic_name: "intro".to_string(),
            confidence: 0.8,
            reason: Some("Pattern matches implication".to_string()),
            arguments: vec!["arg1".to_string()],
        };

        assert_eq!(suggestion.tactic_name, "intro");
        assert_eq!(suggestion.confidence, 0.8);
        assert_eq!(suggestion.arguments.len(), 1);
    }

    #[test]
    fn test_library_suggest_tactics() {
        let library = TacticLibrary::default();
        let goal = create_test_goal("A -> B");
        let context = create_test_context();

        let suggestions = library.suggest_tactics(&goal, &context).unwrap();
        assert!(!suggestions.is_empty());

        // Should suggest intro for implication
        let intro_suggestion = suggestions.iter()
            .find(|s| s.tactic_name == "intro");
        assert!(intro_suggestion.is_some());
    }

    #[test]
    fn test_library_suggest_tactics_sorting() {
        let library = TacticLibrary::default();
        let goal = create_test_goal("x == x");
        let context = create_test_context();

        let suggestions = library.suggest_tactics(&goal, &context).unwrap();

        // Should be sorted by confidence (highest first)
        for i in 1..suggestions.len() {
            assert!(suggestions[i-1].confidence >= suggestions[i].confidence);
        }
    }

    #[test]
    fn test_calculate_confidence_patterns() {
        let library = TacticLibrary::default();
        let dummy_tactic = &IntroTactic as &dyn Tactic;

        let goal1 = create_test_goal("A -> B");
        let confidence1 = library.calculate_confidence(&goal1, dummy_tactic);
        assert_eq!(confidence1, 0.8);

        let goal2 = create_test_goal("A && B");
        let confidence2 = library.calculate_confidence(&goal2, dummy_tactic);
        assert_eq!(confidence2, 0.7);

        let goal3 = create_test_goal("A == B");
        let confidence3 = library.calculate_confidence(&goal3, dummy_tactic);
        assert_eq!(confidence3, 0.9);
    }
}

#[cfg(test)]
mod property_tests_tactics {
    use super::*;
    use proptest::proptest;

    proptest! {
        /// Property: TacticLibrary::default never panics
        #[test]
        fn test_default_never_panics(_input: String) {
            let _ = TacticLibrary::default();
        }

        /// Property: is_applicable never panics on any goal statement
        #[test]
        fn test_is_applicable_never_panics(statement: String) {
            let goal = ProofGoal { statement };
            let context = ProofContext::new();
            let tactic = IntroTactic;

            let _ = tactic.is_applicable(&goal, &context);
        }

        /// Property: Tactic names are consistent
        #[test]
        fn test_tactic_names_consistent(_input: String) {
            let tactics = [
                &IntroTactic as &dyn Tactic,
                &SplitTactic as &dyn Tactic,
                &ReflexivityTactic as &dyn Tactic,
            ];

            for tactic in &tactics {
                let name = tactic.name();
                assert!(!name.is_empty());
                assert!(name.chars().all(|c| c.is_alphanumeric() || c == '_'));
            }
        }
    }
}
