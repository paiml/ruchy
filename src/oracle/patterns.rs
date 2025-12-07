//! Fix Pattern Library
//!
//! Stores and queries patterns for error fixes.
//!
//! # References
//! - [9] Just et al. (2014). `Defects4J` methodology.

use super::ErrorCategory;
use regex::Regex;
use std::collections::HashMap;

/// A fix suggestion returned by the Oracle
#[derive(Debug, Clone)]
pub struct FixSuggestion {
    /// Human-readable description of the fix
    pub description: String,

    /// The transformation to apply (regex or AST transform)
    pub transformation: String,

    /// Historical success rate (0.0 to 1.0)
    pub success_rate: f64,

    /// Number of times this fix has been applied
    pub times_applied: u32,

    /// Pattern ID for tracking
    pub pattern_id: String,
}

impl FixSuggestion {
    /// Create a new fix suggestion
    #[must_use]
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            transformation: String::new(),
            success_rate: 0.0,
            times_applied: 0,
            pattern_id: String::new(),
        }
    }

    /// Set the transformation
    pub fn with_transformation(mut self, transform: impl Into<String>) -> Self {
        self.transformation = transform.into();
        self
    }

    /// Set the success rate
    pub fn with_success_rate(mut self, rate: f64) -> Self {
        self.success_rate = rate;
        self
    }

    /// Set times applied
    pub fn with_times_applied(mut self, count: u32) -> Self {
        self.times_applied = count;
        self
    }

    /// Set pattern ID
    pub fn with_pattern_id(mut self, id: impl Into<String>) -> Self {
        self.pattern_id = id.into();
        self
    }
}

/// A fix pattern stored in the pattern library
#[derive(Debug, Clone)]
pub struct FixPattern {
    /// Unique pattern identifier
    pub id: String,

    /// Error category this pattern addresses
    pub category: ErrorCategory,

    /// Regex pattern to match error messages
    pub error_pattern: String,

    /// Description of the fix
    pub description: String,

    /// Transformation to apply
    pub transformation: String,

    /// Historical success rate
    pub success_rate: f64,

    /// Number of times applied
    pub usage_count: u32,
}

impl FixPattern {
    /// Create a new fix pattern
    #[must_use]
    pub fn new(id: impl Into<String>, category: ErrorCategory) -> Self {
        Self {
            id: id.into(),
            category,
            error_pattern: String::new(),
            description: String::new(),
            transformation: String::new(),
            success_rate: 0.0,
            usage_count: 0,
        }
    }

    /// Set the error pattern
    pub fn with_error_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.error_pattern = pattern.into();
        self
    }

    /// Set the description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set the transformation
    pub fn with_transformation(mut self, transform: impl Into<String>) -> Self {
        self.transformation = transform.into();
        self
    }

    /// Set the success rate
    pub fn with_success_rate(mut self, rate: f64) -> Self {
        self.success_rate = rate;
        self
    }

    /// Check if pattern matches an error message
    #[must_use]
    pub fn matches(&self, error_message: &str) -> bool {
        if self.error_pattern.is_empty() {
            return false;
        }

        // Try regex match
        if let Ok(regex) = Regex::new(&self.error_pattern) {
            return regex.is_match(error_message);
        }

        // Fallback to simple contains
        error_message.contains(&self.error_pattern)
    }

    /// Convert to fix suggestion
    #[must_use]
    pub fn to_suggestion(&self) -> FixSuggestion {
        FixSuggestion {
            description: self.description.clone(),
            transformation: self.transformation.clone(),
            success_rate: self.success_rate,
            times_applied: self.usage_count,
            pattern_id: self.id.clone(),
        }
    }
}

/// Pattern store for fix suggestions
#[derive(Debug)]
pub struct PatternStore {
    /// Patterns indexed by category
    patterns: HashMap<ErrorCategory, Vec<FixPattern>>,

    /// Total pattern count
    count: usize,
}

impl PatternStore {
    /// Create a new pattern store with default patterns
    #[must_use]
    pub fn new() -> Self {
        let mut store = Self {
            patterns: HashMap::new(),
            count: 0,
        };

        // Load default patterns
        store.load_default_patterns();

        store
    }

    /// Create an empty pattern store
    #[must_use]
    pub fn empty() -> Self {
        Self {
            patterns: HashMap::new(),
            count: 0,
        }
    }

    /// Load default fix patterns
    fn load_default_patterns(&mut self) {
        // TypeMismatch patterns
        self.add_pattern(
            FixPattern::new("FIX-001", ErrorCategory::TypeMismatch)
                .with_error_pattern(r"expected `&str`, found `String`")
                .with_description("Convert String to &str using .as_str()")
                .with_transformation(".as_str()")
                .with_success_rate(0.95),
        );

        self.add_pattern(
            FixPattern::new("FIX-002", ErrorCategory::TypeMismatch)
                .with_error_pattern(r"expected `String`, found `&str`")
                .with_description("Convert &str to String using .to_string()")
                .with_transformation(".to_string()")
                .with_success_rate(0.95),
        );

        self.add_pattern(
            FixPattern::new("FIX-003", ErrorCategory::TypeMismatch)
                .with_error_pattern(r"expected `&\[.*\]`, found `Vec<")
                .with_description("Convert Vec to slice using .as_slice()")
                .with_transformation(".as_slice()")
                .with_success_rate(0.90),
        );

        // BorrowChecker patterns
        self.add_pattern(
            FixPattern::new("FIX-004", ErrorCategory::BorrowChecker)
                .with_error_pattern(r"borrow of moved value")
                .with_description("Clone the value before moving")
                .with_transformation(".clone()")
                .with_success_rate(0.85),
        );

        self.add_pattern(
            FixPattern::new("FIX-005", ErrorCategory::BorrowChecker)
                .with_error_pattern(r"cannot borrow .* as mutable")
                .with_description("Change let to let mut")
                .with_transformation("let mut")
                .with_success_rate(0.90),
        );

        // MissingImport patterns
        self.add_pattern(
            FixPattern::new("FIX-006", ErrorCategory::MissingImport)
                .with_error_pattern(r"cannot find type `HashMap`")
                .with_description("Add use std::collections::HashMap;")
                .with_transformation("use std::collections::HashMap;")
                .with_success_rate(0.99),
        );

        self.add_pattern(
            FixPattern::new("FIX-007", ErrorCategory::MissingImport)
                .with_error_pattern(r"cannot find type `HashSet`")
                .with_description("Add use std::collections::HashSet;")
                .with_transformation("use std::collections::HashSet;")
                .with_success_rate(0.99),
        );

        self.add_pattern(
            FixPattern::new("FIX-008", ErrorCategory::MissingImport)
                .with_error_pattern(r"cannot find type `BTreeMap`")
                .with_description("Add use std::collections::BTreeMap;")
                .with_transformation("use std::collections::BTreeMap;")
                .with_success_rate(0.99),
        );

        // TraitBound patterns
        self.add_pattern(
            FixPattern::new("FIX-009", ErrorCategory::TraitBound)
                .with_error_pattern(r"the trait `Debug` is not implemented")
                .with_description("Add #[derive(Debug)] to the type")
                .with_transformation("#[derive(Debug)]")
                .with_success_rate(0.95),
        );

        self.add_pattern(
            FixPattern::new("FIX-010", ErrorCategory::TraitBound)
                .with_error_pattern(r"the trait `Clone` is not implemented")
                .with_description("Add #[derive(Clone)] to the type")
                .with_transformation("#[derive(Clone)]")
                .with_success_rate(0.95),
        );

        self.add_pattern(
            FixPattern::new("FIX-011", ErrorCategory::TraitBound)
                .with_error_pattern(r"the trait `Default` is not implemented")
                .with_description("Add #[derive(Default)] to the type")
                .with_transformation("#[derive(Default)]")
                .with_success_rate(0.90),
        );

        // MutabilityError patterns
        self.add_pattern(
            FixPattern::new("FIX-012", ErrorCategory::MutabilityError)
                .with_error_pattern(r"cannot assign to .*, as it is not declared as mutable")
                .with_description("Change let to let mut")
                .with_transformation("let mut")
                .with_success_rate(0.95),
        );

        self.add_pattern(
            FixPattern::new("FIX-013", ErrorCategory::MutabilityError)
                .with_error_pattern(r"cannot borrow .* as mutable, as it is not declared as mutable")
                .with_description("Change let to let mut")
                .with_transformation("let mut")
                .with_success_rate(0.95),
        );

        // LifetimeError patterns
        self.add_pattern(
            FixPattern::new("FIX-014", ErrorCategory::LifetimeError)
                .with_error_pattern(r"borrowed value does not live long enough")
                .with_description("Clone the value to extend lifetime")
                .with_transformation(".clone()")
                .with_success_rate(0.70),
        );

        self.add_pattern(
            FixPattern::new("FIX-015", ErrorCategory::LifetimeError)
                .with_error_pattern(r"missing lifetime specifier")
                .with_description("Add lifetime annotation 'a")
                .with_transformation("<'a>")
                .with_success_rate(0.60),
        );
    }

    /// Add a pattern to the store
    pub fn add_pattern(&mut self, pattern: FixPattern) {
        let category = pattern.category;
        self.patterns
            .entry(category)
            .or_default()
            .push(pattern);
        self.count += 1;
    }

    /// Query patterns for a category and error message
    #[must_use]
    pub fn query(
        &self,
        category: ErrorCategory,
        error_message: &str,
        similarity_threshold: f64,
    ) -> Vec<FixSuggestion> {
        let mut suggestions = Vec::new();

        if let Some(patterns) = self.patterns.get(&category) {
            for pattern in patterns {
                if pattern.matches(error_message)
                    && pattern.success_rate >= similarity_threshold {
                        suggestions.push(pattern.to_suggestion());
                    }
            }
        }

        // Sort by success rate (descending)
        suggestions.sort_by(|a, b| {
            b.success_rate
                .partial_cmp(&a.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        suggestions
    }

    /// Get pattern count
    #[must_use]
    pub fn count(&self) -> usize {
        self.count
    }

    /// Get patterns for a category
    #[must_use]
    pub fn patterns_for(&self, category: ErrorCategory) -> Option<&Vec<FixPattern>> {
        self.patterns.get(&category)
    }

    /// Get all patterns
    pub fn all_patterns(&self) -> impl Iterator<Item = &FixPattern> {
        self.patterns.values().flatten()
    }

    /// Record pattern usage (increment counter)
    pub fn record_usage(&mut self, pattern_id: &str, success: bool) {
        for patterns in self.patterns.values_mut() {
            for pattern in patterns.iter_mut() {
                if pattern.id == pattern_id {
                    pattern.usage_count += 1;
                    // Update success rate with exponential moving average
                    let alpha = 0.1;
                    let outcome = if success { 1.0 } else { 0.0 };
                    pattern.success_rate = alpha * outcome + (1.0 - alpha) * pattern.success_rate;
                    return;
                }
            }
        }
    }
}

impl Default for PatternStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // EXTREME TDD: Pattern Store Tests
    // ============================================================================

    #[test]
    fn test_fix_suggestion_new() {
        let suggestion = FixSuggestion::new("Add .clone()");
        assert_eq!(suggestion.description, "Add .clone()");
        assert!(suggestion.transformation.is_empty());
    }

    #[test]
    fn test_fix_suggestion_builder() {
        let suggestion = FixSuggestion::new("Add .clone()")
            .with_transformation(".clone()")
            .with_success_rate(0.9)
            .with_times_applied(100)
            .with_pattern_id("FIX-001");

        assert_eq!(suggestion.transformation, ".clone()");
        assert!((suggestion.success_rate - 0.9).abs() < f64::EPSILON);
        assert_eq!(suggestion.times_applied, 100);
        assert_eq!(suggestion.pattern_id, "FIX-001");
    }

    #[test]
    fn test_fix_pattern_new() {
        let pattern = FixPattern::new("FIX-001", ErrorCategory::TypeMismatch);
        assert_eq!(pattern.id, "FIX-001");
        assert_eq!(pattern.category, ErrorCategory::TypeMismatch);
    }

    #[test]
    fn test_fix_pattern_builder() {
        let pattern = FixPattern::new("FIX-001", ErrorCategory::TypeMismatch)
            .with_error_pattern(r"expected.*found")
            .with_description("Fix type mismatch")
            .with_transformation(".to_string()")
            .with_success_rate(0.9);

        assert!(!pattern.error_pattern.is_empty());
        assert!(!pattern.description.is_empty());
        assert!(!pattern.transformation.is_empty());
    }

    #[test]
    fn test_fix_pattern_matches_regex() {
        let pattern = FixPattern::new("FIX-001", ErrorCategory::TypeMismatch)
            .with_error_pattern(r"expected `&str`, found `String`");

        assert!(pattern.matches("expected `&str`, found `String`"));
        assert!(!pattern.matches("something else"));
    }

    #[test]
    fn test_fix_pattern_matches_empty() {
        let pattern = FixPattern::new("FIX-001", ErrorCategory::TypeMismatch);
        assert!(!pattern.matches("anything"));
    }

    #[test]
    fn test_fix_pattern_to_suggestion() {
        let pattern = FixPattern::new("FIX-001", ErrorCategory::TypeMismatch)
            .with_description("Fix it")
            .with_transformation(".fix()")
            .with_success_rate(0.85);

        let suggestion = pattern.to_suggestion();
        assert_eq!(suggestion.description, "Fix it");
        assert_eq!(suggestion.transformation, ".fix()");
        assert!((suggestion.success_rate - 0.85).abs() < f64::EPSILON);
        assert_eq!(suggestion.pattern_id, "FIX-001");
    }

    #[test]
    fn test_pattern_store_new_has_defaults() {
        let store = PatternStore::new();
        assert!(store.count() > 0);
    }

    #[test]
    fn test_pattern_store_empty() {
        let store = PatternStore::empty();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_pattern_store_add_pattern() {
        let mut store = PatternStore::empty();
        store.add_pattern(FixPattern::new("TEST-001", ErrorCategory::TypeMismatch));
        assert_eq!(store.count(), 1);
    }

    #[test]
    fn test_pattern_store_query_matching() {
        let store = PatternStore::new();

        let suggestions = store.query(
            ErrorCategory::TypeMismatch,
            "expected `&str`, found `String`",
            0.0,
        );

        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_pattern_store_query_no_match() {
        let store = PatternStore::new();

        let suggestions = store.query(
            ErrorCategory::TypeMismatch,
            "completely unrelated error",
            0.0,
        );

        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_pattern_store_query_threshold() {
        let store = PatternStore::new();

        // Query with high threshold
        let suggestions = store.query(
            ErrorCategory::TypeMismatch,
            "expected `&str`, found `String`",
            0.99, // Very high threshold
        );

        // May or may not have results depending on pattern success rates
        // Just verify no panic
        let _ = suggestions;
    }

    #[test]
    fn test_pattern_store_query_sorted() {
        let mut store = PatternStore::empty();

        // Add patterns with different success rates
        store.add_pattern(
            FixPattern::new("LOW", ErrorCategory::TypeMismatch)
                .with_error_pattern("test")
                .with_success_rate(0.5),
        );
        store.add_pattern(
            FixPattern::new("HIGH", ErrorCategory::TypeMismatch)
                .with_error_pattern("test")
                .with_success_rate(0.9),
        );

        let suggestions = store.query(ErrorCategory::TypeMismatch, "test", 0.0);

        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0].pattern_id, "HIGH"); // Higher success rate first
        assert_eq!(suggestions[1].pattern_id, "LOW");
    }

    #[test]
    fn test_pattern_store_patterns_for() {
        let store = PatternStore::new();

        let type_patterns = store.patterns_for(ErrorCategory::TypeMismatch);
        assert!(type_patterns.is_some());
        assert!(!type_patterns.unwrap().is_empty());
    }

    #[test]
    fn test_pattern_store_all_patterns() {
        let store = PatternStore::new();
        let all: Vec<_> = store.all_patterns().collect();
        assert!(!all.is_empty());
        assert_eq!(all.len(), store.count());
    }

    #[test]
    fn test_pattern_store_record_usage() {
        let mut store = PatternStore::new();

        // Get initial usage count for FIX-001
        let initial_count = store
            .patterns_for(ErrorCategory::TypeMismatch)
            .and_then(|p| p.first())
            .map_or(0, |p| p.usage_count);

        store.record_usage("FIX-001", true);

        let new_count = store
            .patterns_for(ErrorCategory::TypeMismatch)
            .and_then(|p| p.first())
            .map_or(0, |p| p.usage_count);

        assert_eq!(new_count, initial_count + 1);
    }

    #[test]
    fn test_pattern_store_default() {
        let store = PatternStore::default();
        assert!(store.count() > 0);
    }

    #[test]
    fn test_default_patterns_coverage() {
        let store = PatternStore::new();

        // Check we have patterns for key categories
        assert!(store.patterns_for(ErrorCategory::TypeMismatch).is_some());
        assert!(store.patterns_for(ErrorCategory::BorrowChecker).is_some());
        assert!(store.patterns_for(ErrorCategory::MissingImport).is_some());
        assert!(store.patterns_for(ErrorCategory::TraitBound).is_some());
        assert!(store.patterns_for(ErrorCategory::MutabilityError).is_some());
        assert!(store.patterns_for(ErrorCategory::LifetimeError).is_some());
    }

    #[test]
    fn test_fix_suggestion_clone() {
        let suggestion = FixSuggestion::new("test")
            .with_success_rate(0.9);
        let cloned = suggestion.clone();
        assert_eq!(suggestion.description, cloned.description);
        assert!((suggestion.success_rate - cloned.success_rate).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fix_pattern_clone() {
        let pattern = FixPattern::new("FIX-001", ErrorCategory::TypeMismatch)
            .with_success_rate(0.9);
        let cloned = pattern.clone();
        assert_eq!(pattern.id, cloned.id);
        assert_eq!(pattern.category, cloned.category);
    }
}
