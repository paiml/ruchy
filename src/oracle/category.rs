//! Error Category Classification
//!
//! Defines the 8 error categories for Rust compilation errors.
//! Based on analysis of rustc error codes and transpilation failure patterns.
//!
//! # References
//! - [1] Rust Compiler Error Index (2024)
//! - [8] Ko & Myers (2005). Error causation framework.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Error categories for Rust compilation errors
///
/// Based on rustc error code analysis and transpilation patterns.
/// Each category maps to specific error codes and fix strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ErrorCategory {
    /// Type mismatch errors (E0308, E0271)
    /// Examples: expected &str found String, mismatched types
    TypeMismatch = 0,

    /// Borrow checker violations (E0382, E0502, E0499)
    /// Examples: value borrowed after move, cannot borrow as mutable
    BorrowChecker = 1,

    /// Lifetime annotation errors (E0597, E0716, E0621)
    /// Examples: borrowed value does not live long enough
    LifetimeError = 2,

    /// Missing trait implementations (E0277, E0599)
    /// Examples: the trait Debug is not implemented
    TraitBound = 3,

    /// Unresolved imports (E0433, E0425, E0412)
    /// Examples: cannot find type `HashMap` in this scope
    MissingImport = 4,

    /// Mutability errors (E0596, E0594)
    /// Examples: cannot borrow as mutable, cannot assign to immutable
    MutabilityError = 5,

    /// Syntax and parsing errors (E0658, parser errors)
    /// Examples: expected expression, unexpected token
    SyntaxError = 6,

    /// Uncategorized errors requiring human review
    Other = 7,
}

impl ErrorCategory {
    /// Number of categories
    pub const COUNT: usize = 8;

    /// All categories in order
    pub const ALL: [ErrorCategory; 8] = [
        ErrorCategory::TypeMismatch,
        ErrorCategory::BorrowChecker,
        ErrorCategory::LifetimeError,
        ErrorCategory::TraitBound,
        ErrorCategory::MissingImport,
        ErrorCategory::MutabilityError,
        ErrorCategory::SyntaxError,
        ErrorCategory::Other,
    ];

    /// Priority level (P0 = highest, P3 = lowest)
    #[must_use]
    pub fn priority(&self) -> u8 {
        match self {
            ErrorCategory::TypeMismatch | ErrorCategory::BorrowChecker => 0,
            ErrorCategory::LifetimeError | ErrorCategory::TraitBound => 1,
            ErrorCategory::MissingImport | ErrorCategory::MutabilityError => 2,
            ErrorCategory::SyntaxError | ErrorCategory::Other => 3,
        }
    }

    /// Associated rustc error codes
    #[must_use]
    pub fn error_codes(&self) -> &'static [&'static str] {
        match self {
            ErrorCategory::TypeMismatch => &["E0308", "E0271"],
            ErrorCategory::BorrowChecker => &["E0382", "E0502", "E0499", "E0505"],
            ErrorCategory::LifetimeError => &["E0597", "E0716", "E0621", "E0106"],
            ErrorCategory::TraitBound => &["E0277", "E0599", "E0609"],
            ErrorCategory::MissingImport => &["E0433", "E0425", "E0412", "E0432"],
            ErrorCategory::MutabilityError => &["E0596", "E0594"],
            ErrorCategory::SyntaxError => &["E0658", "E0061", "E0063"],
            ErrorCategory::Other => &[],
        }
    }

    /// Classify from rustc error code
    #[must_use]
    pub fn from_error_code(code: &str) -> Self {
        for category in Self::ALL {
            if category.error_codes().contains(&code) {
                return category;
            }
        }
        ErrorCategory::Other
    }

    /// Human-readable description
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            ErrorCategory::TypeMismatch => "Type mismatch between expected and actual types",
            ErrorCategory::BorrowChecker => "Borrow checker violation (ownership/borrowing)",
            ErrorCategory::LifetimeError => "Lifetime annotation missing or incorrect",
            ErrorCategory::TraitBound => "Missing trait implementation",
            ErrorCategory::MissingImport => "Unresolved import or undefined type",
            ErrorCategory::MutabilityError => "Mutability constraint violation",
            ErrorCategory::SyntaxError => "Syntax or parsing error",
            ErrorCategory::Other => "Uncategorized error requiring human review",
        }
    }

    /// Convert from numeric index
    #[must_use]
    pub fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }

    /// Convert to numeric index
    #[must_use]
    pub fn to_index(self) -> usize {
        self as usize
    }

    /// Convert to label for ML training (alias for `to_index`)
    #[must_use]
    pub fn to_label(self) -> usize {
        self.to_index()
    }

    /// Create from label
    #[must_use]
    pub fn from_label(label: usize) -> Self {
        Self::from_index(label).unwrap_or(Self::Other)
    }
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // EXTREME TDD: ErrorCategory Tests (8 categories, full coverage)
    // ============================================================================

    #[test]
    fn test_category_count() {
        assert_eq!(ErrorCategory::COUNT, 8);
        assert_eq!(ErrorCategory::ALL.len(), 8);
    }

    #[test]
    fn test_category_repr_values() {
        assert_eq!(ErrorCategory::TypeMismatch as u8, 0);
        assert_eq!(ErrorCategory::BorrowChecker as u8, 1);
        assert_eq!(ErrorCategory::LifetimeError as u8, 2);
        assert_eq!(ErrorCategory::TraitBound as u8, 3);
        assert_eq!(ErrorCategory::MissingImport as u8, 4);
        assert_eq!(ErrorCategory::MutabilityError as u8, 5);
        assert_eq!(ErrorCategory::SyntaxError as u8, 6);
        assert_eq!(ErrorCategory::Other as u8, 7);
    }

    #[test]
    fn test_priority_p0_categories() {
        assert_eq!(ErrorCategory::TypeMismatch.priority(), 0);
        assert_eq!(ErrorCategory::BorrowChecker.priority(), 0);
    }

    #[test]
    fn test_priority_p1_categories() {
        assert_eq!(ErrorCategory::LifetimeError.priority(), 1);
        assert_eq!(ErrorCategory::TraitBound.priority(), 1);
    }

    #[test]
    fn test_priority_p2_categories() {
        assert_eq!(ErrorCategory::MissingImport.priority(), 2);
        assert_eq!(ErrorCategory::MutabilityError.priority(), 2);
    }

    #[test]
    fn test_priority_p3_categories() {
        assert_eq!(ErrorCategory::SyntaxError.priority(), 3);
        assert_eq!(ErrorCategory::Other.priority(), 3);
    }

    #[test]
    fn test_from_error_code_type_mismatch() {
        assert_eq!(
            ErrorCategory::from_error_code("E0308"),
            ErrorCategory::TypeMismatch
        );
        assert_eq!(
            ErrorCategory::from_error_code("E0271"),
            ErrorCategory::TypeMismatch
        );
    }

    #[test]
    fn test_from_error_code_borrow_checker() {
        assert_eq!(
            ErrorCategory::from_error_code("E0382"),
            ErrorCategory::BorrowChecker
        );
        assert_eq!(
            ErrorCategory::from_error_code("E0502"),
            ErrorCategory::BorrowChecker
        );
        assert_eq!(
            ErrorCategory::from_error_code("E0499"),
            ErrorCategory::BorrowChecker
        );
    }

    #[test]
    fn test_from_error_code_lifetime() {
        assert_eq!(
            ErrorCategory::from_error_code("E0597"),
            ErrorCategory::LifetimeError
        );
        assert_eq!(
            ErrorCategory::from_error_code("E0716"),
            ErrorCategory::LifetimeError
        );
    }

    #[test]
    fn test_from_error_code_trait_bound() {
        assert_eq!(
            ErrorCategory::from_error_code("E0277"),
            ErrorCategory::TraitBound
        );
        assert_eq!(
            ErrorCategory::from_error_code("E0599"),
            ErrorCategory::TraitBound
        );
    }

    #[test]
    fn test_from_error_code_missing_import() {
        assert_eq!(
            ErrorCategory::from_error_code("E0433"),
            ErrorCategory::MissingImport
        );
        assert_eq!(
            ErrorCategory::from_error_code("E0425"),
            ErrorCategory::MissingImport
        );
    }

    #[test]
    fn test_from_error_code_mutability() {
        assert_eq!(
            ErrorCategory::from_error_code("E0596"),
            ErrorCategory::MutabilityError
        );
        assert_eq!(
            ErrorCategory::from_error_code("E0594"),
            ErrorCategory::MutabilityError
        );
    }

    #[test]
    fn test_from_error_code_syntax() {
        assert_eq!(
            ErrorCategory::from_error_code("E0658"),
            ErrorCategory::SyntaxError
        );
    }

    #[test]
    fn test_from_error_code_unknown() {
        assert_eq!(
            ErrorCategory::from_error_code("E9999"),
            ErrorCategory::Other
        );
        assert_eq!(ErrorCategory::from_error_code(""), ErrorCategory::Other);
        assert_eq!(
            ErrorCategory::from_error_code("invalid"),
            ErrorCategory::Other
        );
    }

    #[test]
    fn test_from_index_valid() {
        assert_eq!(
            ErrorCategory::from_index(0),
            Some(ErrorCategory::TypeMismatch)
        );
        assert_eq!(
            ErrorCategory::from_index(1),
            Some(ErrorCategory::BorrowChecker)
        );
        assert_eq!(ErrorCategory::from_index(7), Some(ErrorCategory::Other));
    }

    #[test]
    fn test_from_index_invalid() {
        assert_eq!(ErrorCategory::from_index(8), None);
        assert_eq!(ErrorCategory::from_index(100), None);
    }

    #[test]
    fn test_to_index_roundtrip() {
        for (i, category) in ErrorCategory::ALL.iter().enumerate() {
            assert_eq!(category.to_index(), i);
            assert_eq!(ErrorCategory::from_index(i), Some(*category));
        }
    }

    #[test]
    fn test_error_codes_not_empty_except_other() {
        for category in ErrorCategory::ALL {
            if category != ErrorCategory::Other {
                assert!(
                    !category.error_codes().is_empty(),
                    "{category:?} should have error codes"
                );
            }
        }
    }

    #[test]
    fn test_description_not_empty() {
        for category in ErrorCategory::ALL {
            assert!(
                !category.description().is_empty(),
                "{category:?} should have description"
            );
        }
    }

    #[test]
    fn test_display_impl() {
        assert_eq!(format!("{}", ErrorCategory::TypeMismatch), "TypeMismatch");
        assert_eq!(format!("{}", ErrorCategory::BorrowChecker), "BorrowChecker");
        assert_eq!(format!("{}", ErrorCategory::Other), "Other");
    }

    #[test]
    fn test_clone_and_copy() {
        let cat = ErrorCategory::TypeMismatch;
        let cloned = cat;
        let copied = cat;
        assert_eq!(cat, cloned);
        assert_eq!(cat, copied);
    }

    #[test]
    fn test_hash_consistency() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for category in ErrorCategory::ALL {
            assert!(set.insert(category), "Categories should be unique");
        }
        assert_eq!(set.len(), 8);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn prop_from_index_roundtrip(index in 0usize..8) {
            let category = ErrorCategory::from_index(index).unwrap();
            prop_assert_eq!(category.to_index(), index);
        }

        #[test]
        fn prop_priority_bounded(index in 0usize..8) {
            let category = ErrorCategory::from_index(index).unwrap();
            prop_assert!(category.priority() <= 3);
        }

        #[test]
        fn prop_error_code_classification_deterministic(
            code in prop::sample::select(vec![
                "E0308", "E0382", "E0597", "E0277", "E0433", "E0596", "E0658", "E9999"
            ])
        ) {
            let cat1 = ErrorCategory::from_error_code(code);
            let cat2 = ErrorCategory::from_error_code(code);
            prop_assert_eq!(cat1, cat2);
        }

        #[test]
        fn prop_description_not_empty(index in 0usize..8) {
            let category = ErrorCategory::from_index(index).unwrap();
            prop_assert!(!category.description().is_empty());
        }
    }
}
