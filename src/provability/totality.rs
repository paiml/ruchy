//! Totality markers (§14.10.3, TOTAL-XXX).
//!
//! `@total` and `@partial` decorators tag functions for the totality checker
//! (future sprint TOTAL-002). Today this module provides the `Totality` enum
//! that the parser/transpiler populates from the decorator; the analyzer
//! that enforces termination using the `decreases` keyword is implemented
//! incrementally.
//!
//! Reference: Idris 2 totality checker, ATS dependent-type obligations.

/// Totality classification for a function.
///
/// Every Ruchy function carries a Totality tag by 5.2. Default is `Partial`
/// (Bronze/Silver); Gold requires `Total`; Platinum requires `Total` or
/// `Corecursive` with a justification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Totality {
    /// `@total`: terminates for all inputs satisfying `requires`. Every
    /// recursive call must strictly decrease the `decreases` measure and
    /// every match must be exhaustive.
    Total,
    /// `@partial`: no termination guarantee. Banned in Gold and Platinum.
    Partial,
    /// `@corecursive`: intentional non-termination (event loop, server).
    /// Must carry a human-readable justification string.
    Corecursive(&'static str),
    /// No explicit marker. Defaults to `Partial` but emits a warning
    /// in Silver tier and higher.
    Unknown,
}

impl Totality {
    /// Is this function proved to terminate?
    #[must_use]
    pub const fn is_total(&self) -> bool {
        matches!(self, Self::Total)
    }

    /// Is this tier-Gold-eligible?
    #[must_use]
    pub const fn is_gold_eligible(&self) -> bool {
        matches!(self, Self::Total | Self::Corecursive(_))
    }

    /// Human-readable label for diagnostics.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Total => "@total",
            Self::Partial => "@partial",
            Self::Corecursive(_) => "@corecursive",
            Self::Unknown => "(unmarked)",
        }
    }

    /// Parse a decorator name into a Totality. Returns None for
    /// decorators that are not totality markers.
    #[must_use]
    pub fn from_decorator(name: &str) -> Option<Self> {
        match name {
            "total" => Some(Self::Total),
            "partial" => Some(Self::Partial),
            // `@corecursive(reason = "...")` uses from_decorator_with_justification.
            _ => None,
        }
    }
}

impl Default for Totality {
    fn default() -> Self {
        Self::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totality_is_total() {
        assert!(Totality::Total.is_total());
        assert!(!Totality::Partial.is_total());
        assert!(!Totality::Corecursive("server loop").is_total());
        assert!(!Totality::Unknown.is_total());
    }

    #[test]
    fn test_gold_eligibility() {
        assert!(Totality::Total.is_gold_eligible());
        assert!(Totality::Corecursive("event loop").is_gold_eligible());
        assert!(!Totality::Partial.is_gold_eligible());
        assert!(!Totality::Unknown.is_gold_eligible());
    }

    #[test]
    fn test_label() {
        assert_eq!(Totality::Total.label(), "@total");
        assert_eq!(Totality::Partial.label(), "@partial");
        assert_eq!(Totality::Corecursive("x").label(), "@corecursive");
        assert_eq!(Totality::Unknown.label(), "(unmarked)");
    }

    #[test]
    fn test_from_decorator() {
        assert_eq!(Totality::from_decorator("total"), Some(Totality::Total));
        assert_eq!(
            Totality::from_decorator("partial"),
            Some(Totality::Partial)
        );
        assert_eq!(Totality::from_decorator("verified"), None);
        assert_eq!(Totality::from_decorator("gold"), None);
    }

    #[test]
    fn test_default_is_unknown() {
        assert_eq!(Totality::default(), Totality::Unknown);
    }
}
