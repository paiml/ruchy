//! AST → Tier classification bridge.
//!
//! Takes a parsed [`Expr`] function definition and returns its §14.2 tier.
//! Pure extraction layer — all classification logic lives in [`super::tier`].
//!
//! Ticket: TIER-002 (AST-to-Tier bridge).

use crate::frontend::ast::{ContractClause, Expr, ExprKind};
use crate::provability::tier::{classify, Tier, TierInputs};

/// Extract decorator names from an expression's attributes.
fn decorator_names(expr: &Expr) -> Vec<&str> {
    expr.attributes.iter().map(|a| a.name.as_str()).collect()
}

/// Check whether a contracts list contains at least one `Requires` clause.
fn has_requires_clause(contracts: &[ContractClause]) -> bool {
    contracts
        .iter()
        .any(|c| matches!(c, ContractClause::Requires(_)))
}

/// Check whether a contracts list contains at least one `Ensures` clause.
fn has_ensures_clause(contracts: &[ContractClause]) -> bool {
    contracts
        .iter()
        .any(|c| matches!(c, ContractClause::Ensures(_)))
}

/// Classify a function-defining expression into its §14.2 tier.
///
/// Returns `None` if the expression is not a function definition. Does NOT
/// consult YAML contracts or Lean proofs — those are file-system artifacts
/// outside the AST. Callers that need the full Platinum check must populate
/// `TierInputs::has_yaml_contract` / `has_lean_proof` themselves and call
/// [`classify`] directly.
///
/// This means: a `@platinum` decorator in the AST alone can only resolve to
/// Silver via [`tier_of_function`]. The full Platinum elevation requires
/// out-of-band verification.
#[must_use]
pub fn tier_of_function(expr: &Expr) -> Option<Tier> {
    if !matches!(expr.kind, ExprKind::Function { .. }) {
        return None;
    }
    let inputs = TierInputs {
        decorators: decorator_names(expr),
        has_requires: has_requires_clause(&expr.contracts),
        has_ensures: has_ensures_clause(&expr.contracts),
        has_yaml_contract: false,
        has_lean_proof: false,
    };
    Some(classify(&inputs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    fn parse(src: &str) -> Expr {
        Parser::new(src).parse().expect("test source must parse")
    }

    fn first_function_in_block(expr: Expr) -> Expr {
        // Parser returns a single expr for a single function, or a Block
        // for multiple top-level items. Grab the first function either way.
        match expr.kind {
            ExprKind::Function { .. } => expr,
            ExprKind::Block(exprs) => exprs
                .into_iter()
                .find(|e| matches!(e.kind, ExprKind::Function { .. }))
                .expect("test must produce at least one function"),
            _ => panic!("unexpected top-level expr kind"),
        }
    }

    #[test]
    fn test_tier_of_function_bare_is_bronze() {
        let f = first_function_in_block(parse("fun f() { 1 }"));
        assert_eq!(tier_of_function(&f), Some(Tier::Bronze));
    }

    #[test]
    fn test_tier_of_function_with_attribute_decorator() {
        // #[bronze] explicit escape hatch
        let f = first_function_in_block(parse("#[bronze]\nfun f() { 1 }"));
        assert_eq!(tier_of_function(&f), Some(Tier::Bronze));
    }

    #[test]
    fn test_tier_of_function_with_gold_attribute_but_no_contract_is_bronze() {
        // @gold decorator without contracts degrades to Bronze (claim
        // must be backed).
        let f = first_function_in_block(parse("#[gold]\nfun f() { 1 }"));
        assert_eq!(tier_of_function(&f), Some(Tier::Bronze));
    }

    #[test]
    fn test_tier_of_function_non_function_returns_none() {
        // Literal expression is not a function.
        let e = parse("42");
        assert_eq!(tier_of_function(&e), None);
    }

    #[test]
    fn test_decorator_names_reads_all_attributes() {
        let f = first_function_in_block(parse(
            "#[bronze]\n#[verified]\n#[total]\nfun f() { 1 }",
        ));
        let names = decorator_names(&f);
        assert!(names.contains(&"bronze"));
        assert!(names.contains(&"verified"));
        assert!(names.contains(&"total"));
    }

    #[test]
    fn test_has_requires_empty_is_false() {
        assert!(!has_requires_clause(&[]));
    }

    #[test]
    fn test_has_ensures_empty_is_false() {
        assert!(!has_ensures_clause(&[]));
    }

    #[test]
    fn test_tier_of_function_platinum_without_full_stack_degrades() {
        // @platinum decorator alone cannot resolve to Platinum via the AST
        // bridge; YAML/Lean checks happen elsewhere. Without contracts it
        // should fall to Bronze.
        let f = first_function_in_block(parse("#[platinum]\nfun f() { 1 }"));
        assert_eq!(tier_of_function(&f), Some(Tier::Bronze));
    }
}
