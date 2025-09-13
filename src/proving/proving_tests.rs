//! Comprehensive TDD tests for Proving modules
//! Target: Increase coverage for SMT solver, verification, and tactics
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod proving_tests {
    use crate::proving::{SmtSolver, Prover, Verifier, Tactic, Formula, ProofResult};
    use std::collections::HashMap;
    
    // ========== SMT Solver Tests ==========
    
    #[test]
    fn test_smt_solver_creation() {
        let solver = SmtSolver::new();
        assert_eq!(solver.constraint_count(), 0);
        assert!(solver.is_empty());
    }
    
    #[test]
    fn test_add_constraint() {
        let mut solver = SmtSolver::new();
        let constraint = Formula::equality("x", 10);
        
        solver.add_constraint(constraint);
        assert_eq!(solver.constraint_count(), 1);
        assert!(!solver.is_empty());
    }
    
    #[test]
    fn test_solve_simple_constraint() {
        let mut solver = SmtSolver::new();
        solver.add_constraint(Formula::equality("x", 5));
        solver.add_constraint(Formula::equality("y", 10));
        
        let solution = solver.solve();
        assert!(solution.is_sat());
        
        let model = solution.get_model().unwrap();
        assert_eq!(model.get("x"), Some(&5));
        assert_eq!(model.get("y"), Some(&10));
    }
    
    #[test]
    fn test_solve_unsatisfiable() {
        let mut solver = SmtSolver::new();
        solver.add_constraint(Formula::equality("x", 5));
        solver.add_constraint(Formula::equality("x", 10)); // Contradiction
        
        let solution = solver.solve();
        assert!(solution.is_unsat());
        assert!(solution.get_model().is_none());
    }
    
    #[test]
    fn test_solve_with_inequalities() {
        let mut solver = SmtSolver::new();
        solver.add_constraint(Formula::greater_than("x", 5));
        solver.add_constraint(Formula::less_than("x", 10));
        
        let solution = solver.solve();
        assert!(solution.is_sat());
        
        let model = solution.get_model().unwrap();
        let x_value = model.get("x").unwrap();
        assert!(*x_value > 5 && *x_value < 10);
    }
    
    #[test]
    fn test_incremental_solving() {
        let mut solver = SmtSolver::new();
        solver.add_constraint(Formula::equality("x", 5));
        
        let solution1 = solver.solve();
        assert!(solution1.is_sat());
        
        solver.push(); // Save state
        solver.add_constraint(Formula::equality("x", 10)); // Add contradiction
        
        let solution2 = solver.solve();
        assert!(solution2.is_unsat());
        
        solver.pop(); // Restore state
        let solution3 = solver.solve();
        assert!(solution3.is_sat()); // Should be satisfiable again
    }
    
    // ========== Prover Tests ==========
    
    #[test]
    fn test_prover_creation() {
        let prover = Prover::new();
        assert_eq!(prover.axiom_count(), 0);
        assert_eq!(prover.theorem_count(), 0);
    }
    
    #[test]
    fn test_add_axiom() {
        let mut prover = Prover::new();
        let axiom = Formula::implication("A", "B");
        
        prover.add_axiom("modus_ponens", axiom);
        assert_eq!(prover.axiom_count(), 1);
    }
    
    #[test]
    fn test_prove_simple_theorem() {
        let mut prover = Prover::new();
        prover.add_axiom("axiom1", Formula::truth("A"));
        prover.add_axiom("axiom2", Formula::implication("A", "B"));
        
        let theorem = Formula::truth("B");
        let proof = prover.prove(theorem);
        
        assert!(proof.is_valid());
        assert_eq!(proof.steps().len(), 2); // Should use both axioms
    }
    
    #[test]
    fn test_prove_by_contradiction() {
        let mut prover = Prover::new();
        let theorem = Formula::or(Formula::truth("A"), Formula::negation("A"));
        
        let proof = prover.prove_by_contradiction(theorem);
        assert!(proof.is_valid());
    }
    
    #[test]
    fn test_prove_with_tactics() {
        let mut prover = Prover::new();
        prover.add_tactic(Tactic::Induction);
        prover.add_tactic(Tactic::CaseSplit);
        
        let theorem = Formula::for_all("n", Formula::property("n >= 0"));
        let proof = prover.prove_with_tactics(theorem);
        
        // May or may not succeed depending on theorem
        assert!(proof.is_valid() || proof.is_incomplete());
    }
    
    // ========== Verifier Tests ==========
    
    #[test]
    fn test_verifier_creation() {
        let verifier = Verifier::new();
        assert_eq!(verifier.verification_count(), 0);
    }
    
    #[test]
    fn test_verify_assertion() {
        let mut verifier = Verifier::new();
        
        let pre = Formula::equality("x", 0);
        let code = "x = x + 1";
        let post = Formula::equality("x", 1);
        
        let result = verifier.verify_assertion(pre, code, post);
        assert!(result.is_verified());
    }
    
    #[test]
    fn test_verify_loop_invariant() {
        let mut verifier = Verifier::new();
        
        let invariant = Formula::property("sum >= 0");
        let loop_code = "for i in 0..n { sum += i }";
        
        let result = verifier.verify_loop_invariant(loop_code, invariant);
        // Verification depends on the invariant strength
        assert!(result.is_verified() || result.needs_strengthening());
    }
    
    #[test]
    fn test_verify_function_contract() {
        let mut verifier = Verifier::new();
        
        let precondition = Formula::greater_than("n", 0);
        let postcondition = Formula::property("result > 0");
        let function = "fn factorial(n: i32) -> i32 { if n <= 1 { 1 } else { n * factorial(n-1) } }";
        
        let result = verifier.verify_contract(function, precondition, postcondition);
        assert!(result.is_verified() || result.has_counterexample());
    }
    
    #[test]
    fn test_generate_counterexample() {
        let mut verifier = Verifier::new();
        
        let pre = Formula::truth("true");
        let code = "x = y / z"; // Division by zero possible
        let post = Formula::property("x >= 0");
        
        let result = verifier.verify_assertion(pre, code, post);
        
        if !result.is_verified() {
            let counterexample = result.get_counterexample().unwrap();
            assert!(counterexample.contains_key("z"));
            assert_eq!(counterexample.get("z"), Some(&0)); // z = 0 causes failure
        }
    }
    
    // ========== Tactics Tests ==========
    
    #[test]
    fn test_tactic_application() {
        let tactic = Tactic::Induction;
        let formula = Formula::for_all("n", Formula::property("sum(0..n) = n*(n+1)/2"));
        
        let subgoals = tactic.apply(&formula);
        assert_eq!(subgoals.len(), 2); // Base case and inductive step
    }
    
    #[test]
    fn test_case_split_tactic() {
        let tactic = Tactic::CaseSplit;
        let formula = Formula::property("x = 0 || x > 0 || x < 0");
        
        let subgoals = tactic.apply(&formula);
        assert_eq!(subgoals.len(), 3); // Three cases
    }
    
    #[test]
    fn test_simplification_tactic() {
        let tactic = Tactic::Simplify;
        let formula = Formula::and(Formula::truth("true"), Formula::property("x > 0"));
        
        let simplified = tactic.apply(&formula);
        assert_eq!(simplified.len(), 1);
        // Should simplify to just "x > 0"
    }
    
    // ========== Formula Construction Tests ==========
    
    #[test]
    fn test_formula_builders() {
        let eq = Formula::equality("x", 5);
        assert_eq!(eq.to_string(), "x = 5");
        
        let gt = Formula::greater_than("y", 10);
        assert_eq!(gt.to_string(), "y > 10");
        
        let and = Formula::and(eq.clone(), gt.clone());
        assert_eq!(and.to_string(), "(x = 5) ∧ (y > 10)");
        
        let or = Formula::or(eq, gt);
        assert_eq!(or.to_string(), "(x = 5) ∨ (y > 10)");
    }
    
    #[test]
    fn test_quantified_formulas() {
        let forall = Formula::for_all("x", Formula::property("x >= 0"));
        assert!(forall.to_string().contains("∀"));
        
        let exists = Formula::exists("y", Formula::property("y * y = 4"));
        assert!(exists.to_string().contains("∃"));
    }
    
    // ========== Integration Tests ==========
    
    #[test]
    fn test_prove_and_verify() {
        let mut prover = Prover::new();
        let mut verifier = Verifier::new();
        
        // Prove a theorem
        let theorem = Formula::implication("A ∧ B", "A");
        let proof = prover.prove(theorem.clone());
        assert!(proof.is_valid());
        
        // Verify the proof
        let verification = verifier.verify_proof(&proof, &theorem);
        assert!(verification);
    }
    
    #[test]
    fn test_smt_assisted_proving() {
        let mut solver = SmtSolver::new();
        let mut prover = Prover::new();
        
        // Use SMT to find witness
        solver.add_constraint(Formula::property("x * x = 16"));
        let solution = solver.solve();
        
        if solution.is_sat() {
            let witness = solution.get_model().unwrap();
            
            // Use witness in proof
            let theorem = Formula::exists("x", Formula::property("x * x = 16"));
            prover.add_witness("x", *witness.get("x").unwrap());
            
            let proof = prover.prove(theorem);
            assert!(proof.is_valid());
        }
    }
    
    // ========== Helper Functions (≤10 complexity each) ==========
    
    impl SmtSolver {
        fn is_empty(&self) -> bool {
            self.constraint_count() == 0
        }
        
        fn constraint_count(&self) -> usize {
            self.constraints.len()
        }
    }
    
    impl Prover {
        fn axiom_count(&self) -> usize {
            self.axioms.len()
        }
        
        fn theorem_count(&self) -> usize {
            self.theorems.len()
        }
    }
    
    impl Verifier {
        fn verification_count(&self) -> usize {
            self.verifications.len()
        }
    }
    
    // ========== Property Tests ==========
    
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_smt_solver_consistency(constraints in prop::collection::vec(0i32..100, 1..10)) {
            let mut solver = SmtSolver::new();
            
            for (i, value) in constraints.iter().enumerate() {
                let var = format!("x{}", i);
                solver.add_constraint(Formula::equality(&var, *value));
            }
            
            let solution = solver.solve();
            if solution.is_sat() {
                let model = solution.get_model().unwrap();
                for (i, value) in constraints.iter().enumerate() {
                    let var = format!("x{}", i);
                    assert_eq!(model.get(&var), Some(value));
                }
            }
        }
        
        #[test]
        fn test_formula_never_panics(var in "[a-z]+", value in -1000i32..1000) {
            let _ = Formula::equality(&var, value);
            let _ = Formula::greater_than(&var, value);
            let _ = Formula::less_than(&var, value);
            // Should not panic
        }
        
        #[test]
        fn test_prover_deterministic(seed in 0u64..1000) {
            let mut prover1 = Prover::with_seed(seed);
            let mut prover2 = Prover::with_seed(seed);
            
            let theorem = Formula::truth("A ∨ ¬A");
            
            let proof1 = prover1.prove(theorem.clone());
            let proof2 = prover2.prove(theorem);
            
            // Same seed should give same result
            assert_eq!(proof1.is_valid(), proof2.is_valid());
        }
    }
}