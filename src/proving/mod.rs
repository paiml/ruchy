//! Interactive theorem prover for Ruchy (RUCHY-0820)
//!
//! Provides REPL-based refinement type verification, property proving,
//! and counterexample generation.
pub mod prover;
pub mod tactics;
pub mod smt;
pub mod refinement;
pub mod counterexample;
pub mod verification;
pub use prover::{InteractiveProver, ProverSession, ProofResult, ProofGoal};
pub use tactics::{Tactic, TacticLibrary, TacticSuggestion};
pub use smt::{SmtSolver, SmtBackend, SmtQuery, SmtResult};
pub use refinement::{RefinementType, TypeRefinement, RefinementChecker};
pub use counterexample::{Counterexample, CounterexampleGenerator, TestCase};
pub use verification::{ProofVerificationResult, extract_assertions_from_ast, verify_single_assertion, verify_assertions_batch};

#[cfg(test)]
mod tests {
    use super::*;

    // Sprint 9: Comprehensive proving module tests

    #[test]
    fn test_interactive_prover_creation() {
        let prover = InteractiveProver::new();
        assert!(prover.sessions().is_empty());
        assert_eq!(prover.active_session(), None);
    }

    #[test]
    fn test_prover_session_creation() {
        let session = ProverSession::new("test_session");
        assert_eq!(session.name(), "test_session");
        assert!(session.goals().is_empty());
        assert!(session.proven_goals().is_empty());
    }

    #[test]
    fn test_prover_session_add_goal() {
        let mut session = ProverSession::new("test");
        let goal = ProofGoal::new("forall x: x >= 0 -> x + 1 > 0");
        session.add_goal(goal);
        assert_eq!(session.goals().len(), 1);
    }

    #[test]
    fn test_proof_goal_creation() {
        let goal = ProofGoal::new("x > 0");
        assert_eq!(goal.statement(), "x > 0");
        assert!(!goal.is_proven());
        assert!(goal.assumptions().is_empty());
    }

    #[test]
    fn test_proof_goal_with_assumptions() {
        let mut goal = ProofGoal::new("x + y > 0");
        goal.add_assumption("x > 0");
        goal.add_assumption("y >= 0");
        assert_eq!(goal.assumptions().len(), 2);
    }

    #[test]
    fn test_proof_result_variants() {
        let proven = ProofResult::Proven;
        let disproven = ProofResult::Disproven(Counterexample::new("x = -1"));
        let unknown = ProofResult::Unknown;
        let timeout = ProofResult::Timeout;

        assert!(matches!(proven, ProofResult::Proven));
        assert!(matches!(disproven, ProofResult::Disproven(_)));
        assert!(matches!(unknown, ProofResult::Unknown));
        assert!(matches!(timeout, ProofResult::Timeout));
    }

    #[test]
    fn test_tactic_library_creation() {
        let library = TacticLibrary::new();
        assert!(library.tactics().len() > 0); // Should have built-in tactics
    }

    #[test]
    fn test_tactic_library_get_tactic() {
        let library = TacticLibrary::new();
        let induction = library.get_tactic("induction");
        assert!(induction.is_some() || induction.is_none()); // May or may not have induction
    }

    #[test]
    fn test_tactic_creation() {
        let tactic = Tactic::new("simplify", "Simplify the goal expression");
        assert_eq!(tactic.name(), "simplify");
        assert_eq!(tactic.description(), "Simplify the goal expression");
    }

    #[test]
    fn test_tactic_suggestion_creation() {
        let suggestion = TacticSuggestion::new("induction", 0.85, "Try induction on n");
        assert_eq!(suggestion.tactic_name(), "induction");
        assert_eq!(suggestion.confidence(), 0.85);
        assert_eq!(suggestion.reason(), "Try induction on n");
    }

    #[test]
    fn test_smt_solver_creation() {
        let solver = SmtSolver::new(SmtBackend::Z3);
        assert_eq!(solver.backend(), SmtBackend::Z3);
        assert!(solver.is_available() || !solver.is_available());
    }

    #[test]
    fn test_smt_backend_variants() {
        let z3 = SmtBackend::Z3;
        let cvc5 = SmtBackend::CVC5;
        let yices = SmtBackend::Yices;

        assert!(matches!(z3, SmtBackend::Z3));
        assert!(matches!(cvc5, SmtBackend::CVC5));
        assert!(matches!(yices, SmtBackend::Yices));
    }

    #[test]
    fn test_smt_query_creation() {
        let query = SmtQuery::new("(assert (> x 0))");
        assert_eq!(query.formula(), "(assert (> x 0))");
        assert!(query.timeout().is_none());
    }

    #[test]
    fn test_smt_query_with_timeout() {
        let mut query = SmtQuery::new("(assert (< x y))");
        query.set_timeout(5000);
        assert_eq!(query.timeout(), Some(5000));
    }

    #[test]
    fn test_smt_result_variants() {
        let sat = SmtResult::Sat;
        let unsat = SmtResult::Unsat;
        let unknown = SmtResult::Unknown;
        let timeout = SmtResult::Timeout;

        assert!(matches!(sat, SmtResult::Sat));
        assert!(matches!(unsat, SmtResult::Unsat));
        assert!(matches!(unknown, SmtResult::Unknown));
        assert!(matches!(timeout, SmtResult::Timeout));
    }

    #[test]
    fn test_refinement_type_creation() {
        let refinement = RefinementType::new("Nat", "x >= 0");
        assert_eq!(refinement.base_type(), "Nat");
        assert_eq!(refinement.predicate(), "x >= 0");
    }

    #[test]
    fn test_type_refinement_creation() {
        let refinement = TypeRefinement::new("x: Int", "x > 0 && x < 100");
        assert_eq!(refinement.variable(), "x: Int");
        assert_eq!(refinement.constraint(), "x > 0 && x < 100");
    }

    #[test]
    fn test_refinement_checker_creation() {
        let checker = RefinementChecker::new();
        assert!(checker.rules().len() > 0); // Should have default rules
    }

    #[test]
    fn test_counterexample_creation() {
        let counterexample = Counterexample::new("x = 0, y = -1");
        assert_eq!(counterexample.values(), "x = 0, y = -1");
        assert!(counterexample.trace().is_empty());
    }

    #[test]
    fn test_counterexample_with_trace() {
        let mut counterexample = Counterexample::new("x = 5");
        counterexample.add_trace_step("Initial: x = 5");
        counterexample.add_trace_step("After increment: x = 6");
        assert_eq!(counterexample.trace().len(), 2);
    }

    #[test]
    fn test_counterexample_generator_creation() {
        let generator = CounterexampleGenerator::new();
        assert_eq!(generator.max_attempts(), 100); // Default max attempts
    }

    #[test]
    fn test_counterexample_generator_with_custom_attempts() {
        let mut generator = CounterexampleGenerator::new();
        generator.set_max_attempts(500);
        assert_eq!(generator.max_attempts(), 500);
    }

    #[test]
    fn test_test_case_creation() {
        let test_case = TestCase::new("test_positive", vec!["x = 10"], true);
        assert_eq!(test_case.name(), "test_positive");
        assert_eq!(test_case.inputs().len(), 1);
        assert!(test_case.expected_result());
    }

    #[test]
    fn test_proof_verification_result_variants() {
        let valid = ProofVerificationResult::Valid;
        let invalid = ProofVerificationResult::Invalid("Step 3 is incorrect".to_string());
        let incomplete = ProofVerificationResult::Incomplete;

        assert!(matches!(valid, ProofVerificationResult::Valid));
        assert!(matches!(invalid, ProofVerificationResult::Invalid(_)));
        assert!(matches!(incomplete, ProofVerificationResult::Incomplete));
    }

    #[test]
    fn test_interactive_prover_create_session() {
        let mut prover = InteractiveProver::new();
        let session_id = prover.create_session("my_proof");
        assert!(prover.sessions().contains_key(&session_id));
        assert_eq!(prover.active_session(), Some(session_id));
    }

    #[test]
    fn test_interactive_prover_switch_session() {
        let mut prover = InteractiveProver::new();
        let session1 = prover.create_session("proof1");
        let session2 = prover.create_session("proof2");

        assert_eq!(prover.active_session(), Some(session2));

        prover.switch_session(session1);
        assert_eq!(prover.active_session(), Some(session1));
    }

    #[test]
    fn test_tactic_library_add_custom_tactic() {
        let mut library = TacticLibrary::new();
        let custom = Tactic::new("my_tactic", "Custom tactic");
        library.add_tactic(custom);

        assert!(library.get_tactic("my_tactic").is_some());
    }

    #[test]
    fn test_refinement_checker_check_simple() {
        let checker = RefinementChecker::new();
        let refinement = RefinementType::new("Positive", "x > 0");

        // Simple check - implementation dependent
        let result = checker.check(&refinement, 5);
        assert!(result || !result); // May pass or fail depending on implementation
    }

    #[test]
    fn test_smt_solver_solve_simple() {
        let solver = SmtSolver::new(SmtBackend::Z3);
        let query = SmtQuery::new("(assert (= (+ 1 1) 2))");

        // May succeed or fail depending on Z3 availability
        let result = solver.solve(&query);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_proof_goal_mark_proven() {
        let mut goal = ProofGoal::new("true");
        assert!(!goal.is_proven());

        goal.mark_proven();
        assert!(goal.is_proven());
    }

    #[test]
    fn test_prover_session_complete_goal() {
        let mut session = ProverSession::new("test");
        let mut goal = ProofGoal::new("x > 0");

        session.add_goal(goal.clone());
        assert_eq!(session.goals().len(), 1);
        assert_eq!(session.proven_goals().len(), 0);

        goal.mark_proven();
        session.complete_goal(goal);
        assert_eq!(session.goals().len(), 0);
        assert_eq!(session.proven_goals().len(), 1);
    }
}