//! Feature Extraction for Error Classification
//!
//! Extracts 73 features from rustc error messages for ML classification.
//!
//! # Feature Groups
//! - Error code indicators (ONE-HOT, 40 features)
//! - Keyword detection (ONE-HOT, 21 features)
//! - Handcrafted features (12 features)
//!
//! # References
//! - [2] Zheng & Casari (2018). Feature Engineering for ML.

use super::ErrorCategory;

/// Total number of features extracted from error messages
pub const FEATURE_COUNT: usize = 73;

/// Error code feature count (ONE-HOT encoded)
pub const ERROR_CODE_FEATURES: usize = 40;

/// Keyword detection feature count
pub const KEYWORD_FEATURES: usize = 21;

/// Handcrafted feature count
pub const HANDCRAFTED_FEATURES: usize = 12;

/// Keywords to detect in error messages
pub const KEYWORDS: [&str; 21] = [
    "type", "borrow", "move", "clone", "mut", "impl", "trait",
    "lifetime", "reference", "ownership", "drop", "copy",
    "string", "str", "vec", "option", "result", "expected",
    "found", "mismatch", "cannot",
];

/// Common rustc error codes for ONE-HOT encoding
pub const ERROR_CODES: [&str; 40] = [
    "E0308", "E0271", "E0382", "E0502", "E0499", "E0505",
    "E0597", "E0716", "E0621", "E0106", "E0277", "E0599",
    "E0609", "E0433", "E0425", "E0412", "E0432", "E0596",
    "E0594", "E0658", "E0061", "E0063", "E0384", "E0507",
    "E0515", "E0373", "E0282", "E0283", "E0369", "E0392",
    "E0404", "E0405", "E0407", "E0408", "E0409", "E0411",
    "E0413", "E0415", "E0416", "E0424",
];

/// Extracted features from an error message
#[derive(Debug, Clone)]
pub struct ErrorFeatures {
    /// Feature vector (73 dimensions)
    pub features: [f32; FEATURE_COUNT],
}

impl ErrorFeatures {
    /// Create empty feature vector
    #[must_use]
    pub fn new() -> Self {
        Self {
            features: [0.0; FEATURE_COUNT],
        }
    }

    /// Extract features from error message and optional error code
    #[must_use]
    pub fn extract(error_message: &str, error_code: Option<&str>) -> Self {
        let mut features = Self::new();

        // 1. Error code ONE-HOT encoding (40 features)
        if let Some(code) = error_code {
            features.encode_error_code(code);
        }

        // 2. Keyword detection (21 features)
        features.detect_keywords(error_message);

        // 3. Handcrafted features (12 features)
        features.extract_handcrafted(error_message);

        features
    }

    /// Encode error code as ONE-HOT vector
    fn encode_error_code(&mut self, code: &str) {
        for (i, known_code) in ERROR_CODES.iter().enumerate() {
            if code == *known_code {
                self.features[i] = 1.0;
                return;
            }
        }
    }

    /// Detect keywords in error message
    fn detect_keywords(&mut self, message: &str) {
        let message_lower = message.to_lowercase();
        let offset = ERROR_CODE_FEATURES;

        for (i, keyword) in KEYWORDS.iter().enumerate() {
            if message_lower.contains(keyword) {
                self.features[offset + i] = 1.0;
            }
        }
    }

    /// Extract handcrafted features
    fn extract_handcrafted(&mut self, message: &str) {
        let offset = ERROR_CODE_FEATURES + KEYWORD_FEATURES;
        let message_lower = message.to_lowercase();

        // Feature 0: Mentions ownership
        self.features[offset] = if message_lower.contains("ownership")
            || message_lower.contains("owned") { 1.0 } else { 0.0 };

        // Feature 1: Mentions lifetime
        self.features[offset + 1] = if message_lower.contains("lifetime")
            || message_lower.contains("'a")
            || message_lower.contains("'static") { 1.0 } else { 0.0 };

        // Feature 2: Mentions type
        self.features[offset + 2] = if message_lower.contains("type") { 1.0 } else { 0.0 };

        // Feature 3: Mentions trait
        self.features[offset + 3] = if message_lower.contains("trait")
            || message_lower.contains("impl") { 1.0 } else { 0.0 };

        // Feature 4: Token count (normalized)
        let token_count = message.split_whitespace().count();
        self.features[offset + 4] = (token_count as f32 / 100.0).min(1.0);

        // Feature 5: Line number present (normalized)
        self.features[offset + 5] = if message.contains(':') { 1.0 } else { 0.0 };

        // Feature 6: Has suggestion
        self.features[offset + 6] = if message_lower.contains("help:")
            || message_lower.contains("suggestion:")
            || message_lower.contains("consider") { 1.0 } else { 0.0 };

        // Feature 7: Suggestion confidence (based on keywords)
        self.features[offset + 7] = if message_lower.contains("try")
            || message_lower.contains("use") { 0.5 } else { 0.0 };

        // Feature 8: Error chain depth (count of "caused by")
        let chain_depth = message_lower.matches("caused by").count();
        self.features[offset + 8] = (chain_depth as f32 / 5.0).min(1.0);

        // Feature 9: Related error count (count of "note:")
        let note_count = message_lower.matches("note:").count();
        self.features[offset + 9] = (note_count as f32 / 5.0).min(1.0);

        // Feature 10: File complexity (based on path depth)
        let path_depth = message.matches('/').count();
        self.features[offset + 10] = (path_depth as f32 / 10.0).min(1.0);

        // Feature 11: Function nesting (count of nested braces mentioned)
        self.features[offset + 11] = if message.contains("in function")
            || message.contains("in method") { 1.0 } else { 0.0 };
    }

    /// Get the feature vector as a slice
    #[must_use]
    pub fn as_slice(&self) -> &[f32] {
        &self.features
    }

    /// Get the feature vector as a Vec for ML libraries
    #[must_use]
    pub fn to_vec(&self) -> Vec<f32> {
        self.features.to_vec()
    }

    /// Predict category using rule-based fallback (no ML model)
    #[must_use]
    pub fn predict_category_rules(&self) -> ErrorCategory {
        // Check error code features first (most reliable)
        for (i, code) in ERROR_CODES.iter().enumerate() {
            if self.features[i] > 0.5 {
                return ErrorCategory::from_error_code(code);
            }
        }

        // Fallback to keyword heuristics
        let offset = ERROR_CODE_FEATURES;

        // Check for type-related keywords
        if self.features[offset] > 0.5 // "type"
            || self.features[offset + 17] > 0.5 // "expected"
            || self.features[offset + 19] > 0.5 // "mismatch"
        {
            return ErrorCategory::TypeMismatch;
        }

        // Check for borrow-related keywords
        if self.features[offset + 1] > 0.5 // "borrow"
            || self.features[offset + 2] > 0.5 // "move"
        {
            return ErrorCategory::BorrowChecker;
        }

        // Check for lifetime keywords
        if self.features[offset + 7] > 0.5 { // "lifetime"
            return ErrorCategory::LifetimeError;
        }

        // Check for trait keywords
        if self.features[offset + 5] > 0.5 // "impl"
            || self.features[offset + 6] > 0.5 // "trait"
        {
            return ErrorCategory::TraitBound;
        }

        // Check for mutability
        if self.features[offset + 4] > 0.5 { // "mut"
            return ErrorCategory::MutabilityError;
        }

        ErrorCategory::Other
    }
}

impl Default for ErrorFeatures {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // EXTREME TDD: Feature Extraction Tests
    // ============================================================================

    #[test]
    fn test_feature_count_constant() {
        assert_eq!(FEATURE_COUNT, 73);
        assert_eq!(ERROR_CODE_FEATURES + KEYWORD_FEATURES + HANDCRAFTED_FEATURES, 73);
    }

    #[test]
    fn test_feature_breakdown() {
        assert_eq!(ERROR_CODE_FEATURES, 40);
        assert_eq!(KEYWORD_FEATURES, 21);
        assert_eq!(HANDCRAFTED_FEATURES, 12);
    }

    #[test]
    fn test_keywords_count() {
        assert_eq!(KEYWORDS.len(), 21);
    }

    #[test]
    fn test_error_codes_count() {
        assert_eq!(ERROR_CODES.len(), 40);
    }

    #[test]
    fn test_new_creates_zero_features() {
        let features = ErrorFeatures::new();
        assert!(features.features.iter().all(|&f| f == 0.0));
    }

    #[test]
    fn test_extract_with_error_code_e0308() {
        let features = ErrorFeatures::extract("mismatched types", Some("E0308"));
        assert_eq!(features.features[0], 1.0); // E0308 is index 0
    }

    #[test]
    fn test_extract_with_error_code_e0382() {
        let features = ErrorFeatures::extract("borrow of moved value", Some("E0382"));
        assert_eq!(features.features[2], 1.0); // E0382 is index 2
    }

    #[test]
    fn test_extract_keyword_type() {
        let features = ErrorFeatures::extract("expected type `i32`", None);
        assert_eq!(features.features[ERROR_CODE_FEATURES], 1.0); // "type" is first keyword
    }

    #[test]
    fn test_extract_keyword_borrow() {
        let features = ErrorFeatures::extract("cannot borrow as mutable", None);
        assert_eq!(features.features[ERROR_CODE_FEATURES + 1], 1.0); // "borrow"
    }

    #[test]
    fn test_extract_keyword_clone() {
        let features = ErrorFeatures::extract("consider using clone()", None);
        assert_eq!(features.features[ERROR_CODE_FEATURES + 3], 1.0); // "clone"
    }

    #[test]
    fn test_extract_handcrafted_ownership() {
        let features = ErrorFeatures::extract("ownership of value", None);
        let offset = ERROR_CODE_FEATURES + KEYWORD_FEATURES;
        assert_eq!(features.features[offset], 1.0); // mentions_ownership
    }

    #[test]
    fn test_extract_handcrafted_lifetime() {
        let features = ErrorFeatures::extract("lifetime 'a not satisfied", None);
        let offset = ERROR_CODE_FEATURES + KEYWORD_FEATURES;
        assert_eq!(features.features[offset + 1], 1.0); // mentions_lifetime
    }

    #[test]
    fn test_extract_handcrafted_suggestion() {
        let features = ErrorFeatures::extract("help: consider adding", None);
        let offset = ERROR_CODE_FEATURES + KEYWORD_FEATURES;
        assert_eq!(features.features[offset + 6], 1.0); // has_suggestion
    }

    #[test]
    fn test_extract_handcrafted_token_count() {
        let features = ErrorFeatures::extract("a b c d e f g h i j", None);
        let offset = ERROR_CODE_FEATURES + KEYWORD_FEATURES;
        // 10 tokens / 100 = 0.1
        assert!((features.features[offset + 4] - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_as_slice_length() {
        let features = ErrorFeatures::new();
        assert_eq!(features.as_slice().len(), FEATURE_COUNT);
    }

    #[test]
    fn test_to_vec_length() {
        let features = ErrorFeatures::new();
        assert_eq!(features.to_vec().len(), FEATURE_COUNT);
    }

    #[test]
    fn test_predict_category_rules_type_mismatch() {
        let features = ErrorFeatures::extract("mismatched types", Some("E0308"));
        assert_eq!(features.predict_category_rules(), ErrorCategory::TypeMismatch);
    }

    #[test]
    fn test_predict_category_rules_borrow_checker() {
        let features = ErrorFeatures::extract("borrow of moved value", Some("E0382"));
        assert_eq!(features.predict_category_rules(), ErrorCategory::BorrowChecker);
    }

    #[test]
    fn test_predict_category_rules_missing_import() {
        let features = ErrorFeatures::extract("cannot find type", Some("E0433"));
        assert_eq!(features.predict_category_rules(), ErrorCategory::MissingImport);
    }

    #[test]
    fn test_predict_category_rules_from_keywords() {
        // No error code, rely on keywords
        let features = ErrorFeatures::extract("expected type String found i32 mismatch", None);
        assert_eq!(features.predict_category_rules(), ErrorCategory::TypeMismatch);
    }

    #[test]
    fn test_predict_category_rules_borrow_keywords() {
        let features = ErrorFeatures::extract("cannot borrow value after move", None);
        assert_eq!(features.predict_category_rules(), ErrorCategory::BorrowChecker);
    }

    #[test]
    fn test_predict_category_rules_unknown() {
        let features = ErrorFeatures::extract("some random error", None);
        assert_eq!(features.predict_category_rules(), ErrorCategory::Other);
    }

    #[test]
    fn test_default_impl() {
        let features = ErrorFeatures::default();
        assert!(features.features.iter().all(|&f| f == 0.0));
    }

    #[test]
    fn test_clone_impl() {
        let features = ErrorFeatures::extract("test", Some("E0308"));
        let cloned = features.clone();
        assert_eq!(features.features, cloned.features);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn prop_feature_vector_length(
            message in ".*",
            code in prop::option::of(prop::sample::select(vec![
                "E0308", "E0382", "E0597", "E0277", "E0433"
            ]))
        ) {
            let features = ErrorFeatures::extract(&message, code);
            prop_assert_eq!(features.as_slice().len(), FEATURE_COUNT);
        }

        #[test]
        fn prop_features_bounded(message in "[a-zA-Z ]{0,100}") {
            let features = ErrorFeatures::extract(&message, None);
            for &f in features.as_slice() {
                prop_assert!((0.0..=1.0).contains(&f),
                    "Feature {} out of bounds [0,1]", f);
            }
        }

        #[test]
        fn prop_extract_deterministic(
            message in "[a-zA-Z ]{0,50}",
            code in prop::option::of("E0[0-9]{3}")
        ) {
            let f1 = ErrorFeatures::extract(&message, code.as_deref());
            let f2 = ErrorFeatures::extract(&message, code.as_deref());
            prop_assert_eq!(f1.features, f2.features);
        }

        #[test]
        fn prop_predict_category_always_valid(message in ".*") {
            let features = ErrorFeatures::extract(&message, None);
            let category = features.predict_category_rules();
            // Category should be one of the 8 valid categories
            prop_assert!(category.to_index() < ErrorCategory::COUNT);
        }
    }
}
