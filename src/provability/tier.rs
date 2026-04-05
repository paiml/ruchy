//! Provability tier classification (§14.2).
//!
//! Given a function's attribute list and contract signature, determine
//! which of the four §14.2 tiers applies: Bronze, Silver, Gold, or
//! Platinum. The classification is the input to metrics F1 (non-trivial
//! contract rate), F4 (stdlib Bronze count), and Criterion #13.
//!
//! This module is pure: it takes attribute names + booleans and returns
//! a tier. No file I/O, no parser coupling.

/// §14.2 provability tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// No contract, baseline type safety only. Deprecated after 5.2.
    Bronze,
    /// `requires`/`ensures` present. Default tier.
    Silver,
    /// `@gold` + contracts. SMT-discharged. Required for stdlib `pub fn`.
    Gold,
    /// `@platinum` + YAML + Lean theorem. Safety-critical.
    Platinum,
}

impl Tier {
    /// Numeric strength for ordering (Bronze=0 through Platinum=3).
    #[must_use]
    pub const fn strength(&self) -> u8 {
        match self {
            Self::Bronze => 0,
            Self::Silver => 1,
            Self::Gold => 2,
            Self::Platinum => 3,
        }
    }

    /// Human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Bronze => "bronze",
            Self::Silver => "silver",
            Self::Gold => "gold",
            Self::Platinum => "platinum",
        }
    }

    /// Is this tier allowed in stdlib after release 5.2?
    ///
    /// Per §14.6: Bronze is banned in stdlib after 5.2.
    #[must_use]
    pub const fn is_stdlib_eligible_at_52(&self) -> bool {
        !matches!(self, Self::Bronze)
    }

    /// Does this tier require Kani BMC verification?
    #[must_use]
    pub const fn requires_kani(&self) -> bool {
        matches!(self, Self::Gold | Self::Platinum)
    }

    /// Does this tier require a Lean refinement proof? (Platinum only.)
    #[must_use]
    pub const fn requires_lean_refinement(&self) -> bool {
        matches!(self, Self::Platinum)
    }
}

impl Default for Tier {
    fn default() -> Self {
        Self::Bronze
    }
}

/// Minimal view of a function's provability annotations.
///
/// Constructed by callers from a parsed `Expr` (decorators → strings,
/// contract clauses → bools). Keeps `classify` pure and testable.
#[derive(Debug, Clone, Default)]
pub struct TierInputs<'a> {
    /// Decorator names present on the function (e.g. "bronze", "gold", "platinum", "total").
    pub decorators: Vec<&'a str>,
    /// Whether the function has at least one `requires` clause.
    pub has_requires: bool,
    /// Whether the function has at least one `ensures` clause.
    pub has_ensures: bool,
    /// Whether the function has a linked YAML contract
    /// (`provable-contracts` kernel spec).
    pub has_yaml_contract: bool,
    /// Whether the function has a non-`sorry` Lean refinement proof.
    pub has_lean_proof: bool,
}

/// Classify a function into a tier based on its annotations.
///
/// **Precedence** (§14.2 authoritative):
/// 1. Explicit `@platinum` decorator + YAML contract + Lean proof → Platinum.
/// 2. Explicit `@gold` decorator + contracts → Gold.
/// 3. Explicit `@bronze` decorator → Bronze (migration escape hatch).
/// 4. `requires` or `ensures` present → Silver (default).
/// 5. Otherwise → Bronze (emit warning in Silver+ diagnostic runs).
#[must_use]
pub fn classify(inputs: &TierInputs<'_>) -> Tier {
    let has_bronze = inputs.decorators.iter().any(|d| *d == "bronze");
    let has_gold = inputs.decorators.iter().any(|d| *d == "gold");
    let has_platinum = inputs.decorators.iter().any(|d| *d == "platinum");
    let has_contracts = inputs.has_requires || inputs.has_ensures;

    // Explicit Platinum requires the full stack (§14.2).
    if has_platinum && has_contracts && inputs.has_yaml_contract && inputs.has_lean_proof {
        return Tier::Platinum;
    }
    // A @platinum claim without full stack degrades to Gold — we refuse
    // to give the label without the backing proof.
    if has_gold && has_contracts {
        return Tier::Gold;
    }
    if has_bronze {
        return Tier::Bronze;
    }
    if has_contracts {
        return Tier::Silver;
    }
    Tier::Bronze
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs(decorators: Vec<&'static str>, req: bool, ens: bool) -> TierInputs<'static> {
        TierInputs {
            decorators,
            has_requires: req,
            has_ensures: ens,
            has_yaml_contract: false,
            has_lean_proof: false,
        }
    }

    #[test]
    fn test_tier_order() {
        assert!(Tier::Bronze.strength() < Tier::Silver.strength());
        assert!(Tier::Silver.strength() < Tier::Gold.strength());
        assert!(Tier::Gold.strength() < Tier::Platinum.strength());
    }

    #[test]
    fn test_tier_labels() {
        assert_eq!(Tier::Bronze.label(), "bronze");
        assert_eq!(Tier::Silver.label(), "silver");
        assert_eq!(Tier::Gold.label(), "gold");
        assert_eq!(Tier::Platinum.label(), "platinum");
    }

    #[test]
    fn test_stdlib_eligibility_at_52() {
        assert!(!Tier::Bronze.is_stdlib_eligible_at_52());
        assert!(Tier::Silver.is_stdlib_eligible_at_52());
        assert!(Tier::Gold.is_stdlib_eligible_at_52());
        assert!(Tier::Platinum.is_stdlib_eligible_at_52());
    }

    #[test]
    fn test_kani_requirement() {
        assert!(!Tier::Bronze.requires_kani());
        assert!(!Tier::Silver.requires_kani());
        assert!(Tier::Gold.requires_kani());
        assert!(Tier::Platinum.requires_kani());
    }

    #[test]
    fn test_lean_refinement_only_for_platinum() {
        assert!(!Tier::Bronze.requires_lean_refinement());
        assert!(!Tier::Silver.requires_lean_refinement());
        assert!(!Tier::Gold.requires_lean_refinement());
        assert!(Tier::Platinum.requires_lean_refinement());
    }

    #[test]
    fn test_classify_bare_function_is_bronze() {
        assert_eq!(classify(&inputs(vec![], false, false)), Tier::Bronze);
    }

    #[test]
    fn test_classify_requires_only_is_silver() {
        assert_eq!(classify(&inputs(vec![], true, false)), Tier::Silver);
    }

    #[test]
    fn test_classify_ensures_only_is_silver() {
        assert_eq!(classify(&inputs(vec![], false, true)), Tier::Silver);
    }

    #[test]
    fn test_classify_both_clauses_is_silver() {
        assert_eq!(classify(&inputs(vec![], true, true)), Tier::Silver);
    }

    #[test]
    fn test_classify_gold_decorator_plus_contract_is_gold() {
        assert_eq!(
            classify(&inputs(vec!["gold"], true, true)),
            Tier::Gold
        );
    }

    #[test]
    fn test_classify_gold_decorator_without_contract_is_bronze() {
        // A @gold claim without contracts degrades. We refuse to give
        // the label without backing.
        assert_eq!(
            classify(&inputs(vec!["gold"], false, false)),
            Tier::Bronze
        );
    }

    #[test]
    fn test_classify_platinum_requires_full_stack() {
        let partial = TierInputs {
            decorators: vec!["platinum"],
            has_requires: true,
            has_ensures: true,
            has_yaml_contract: false,  // missing YAML
            has_lean_proof: false,
        };
        // Without YAML, @platinum does NOT upgrade past Silver.
        assert_eq!(classify(&partial), Tier::Silver);

        let full = TierInputs {
            decorators: vec!["platinum"],
            has_requires: true,
            has_ensures: true,
            has_yaml_contract: true,
            has_lean_proof: true,
        };
        assert_eq!(classify(&full), Tier::Platinum);
    }

    #[test]
    fn test_classify_bronze_decorator_wins_over_contracts() {
        // Explicit @bronze is a migration escape hatch: author declares
        // "I know this is Bronze", so we honor it even with contracts.
        assert_eq!(
            classify(&inputs(vec!["bronze"], true, true)),
            Tier::Bronze
        );
    }

    #[test]
    fn test_classify_ignores_unrelated_decorators() {
        // Unrelated decorators don't change classification.
        assert_eq!(
            classify(&inputs(vec!["verified", "total"], true, false)),
            Tier::Silver
        );
    }

    #[test]
    fn test_default_tier_is_bronze() {
        assert_eq!(Tier::default(), Tier::Bronze);
    }
}
