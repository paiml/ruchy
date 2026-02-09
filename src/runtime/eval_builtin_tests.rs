//! Tests for eval_builtin module
//!
//! EXTREME TDD Round 86: Comprehensive tests for builtin functions
//! Coverage target: 95% for eval_builtin module
//!
//! These tests use the REPL to evaluate builtin functions end-to-end.
//! Requires `repl` feature since they use the REPL for evaluation.

#[cfg(all(test, feature = "repl"))]
mod tests {
    use crate::runtime::Repl;

    // Helper to create a REPL and evaluate an expression
    fn eval(code: &str) -> String {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed");
        repl.eval(code).expect("eval should succeed")
    }

    // Helper that may or may not succeed
    fn try_eval(code: &str) -> Option<String> {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed");
        repl.eval(code).ok()
    }

    #[path = "eval_builtin_tests_part1.rs"]
    mod part1;
    #[path = "eval_builtin_tests_part2.rs"]
    mod part2;
}
