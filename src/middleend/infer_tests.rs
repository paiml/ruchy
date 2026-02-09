//! EXTREME TDD tests for infer.rs
//!
//! Coverage target: 95% for type inference module
//! Tests cover InferenceContext, TypeConstraint, all infer methods

#[cfg(test)]
mod tests {
    use crate::frontend::ast::Expr;
    use crate::frontend::parser::Parser;
    use crate::middleend::environment::TypeEnv;
    use crate::middleend::infer::{InferenceContext, TypeConstraint};
    use crate::middleend::types::{MonoType, TyVar, TypeScheme};

    // Helper to create expression from code
    fn parse_code(code: &str) -> Expr {
        let mut parser = Parser::new(code);
        parser.parse().expect("should parse")
    }

    #[path = "infer_tests_part1.rs"]
    mod part1;
    #[path = "infer_tests_part2.rs"]
    mod part2;
}
