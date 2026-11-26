//! Ruchy error taxonomy for the error oracle (paiml/ruchy#164, Step 3).
//!
//! Distinct from [`super::ErrorCategory`], which classifies rustc errors from
//! generated Rust. This enum classifies Ruchy-level errors: parse, type, name,
//! function, control, async, transpile, runtime and effect failures.
//!
//! Contract: `contracts/error-oracle-taxonomy-v1.yaml`.

/// Ruchy error classes. Discriminants are the stable ML labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RuchyErrorCategory {
    ParseUnexpectedToken = 0,
    ParseUnterminatedString = 1,
    ParseInvalidNumber = 2,
    ParseMissingSemicolon = 3,
    ParseUnmatchedBracket = 4,
    ParseInvalidOperator = 5,

    TypeMismatch = 10,
    TypeInferenceFailed = 11,
    TypeAnnotationRequired = 12,
    TypeNotCallable = 13,
    TypeNotIndexable = 14,
    TypeNotIterable = 15,

    NameNotFound = 20,
    NameAlreadyDefined = 21,
    NameShadowWarning = 22,
    NameImportFailed = 23,
    NamePrivateAccess = 24,

    FuncWrongArgCount = 30,
    FuncWrongArgType = 31,
    FuncMissingReturn = 32,
    FuncReturnTypeMismatch = 33,
    FuncRecursionLimit = 34,

    ControlBreakOutside = 40,
    ControlContinueOutside = 41,
    ControlUnreachableCode = 42,
    ControlMissingCase = 43,

    AsyncAwaitOutside = 50,
    AsyncMissingAwait = 51,
    ConcurrencyDataRace = 52,
    ConcurrencyDeadlock = 53,

    TranspileUnsupported = 60,
    TranspileRustcError = 61,
    TranspileBorrowError = 62,
    TranspileLifetimeError = 63,

    RuntimeDivisionByZero = 70,
    RuntimeIndexOutOfBounds = 71,
    RuntimeNullReference = 72,
    RuntimeStackOverflow = 73,

    EffectNotHandled = 80,
    EffectHandlerMismatch = 81,
    EffectResumeMissing = 82,

    /// Fallback for unmapped labels. Not a trained output class.
    Unknown = 255,
}

impl RuchyErrorCategory {
    /// Number of trained output classes (every variant except `Unknown`).
    pub const COUNT: usize = 41;

    /// Every trained class, in label order.
    pub const ALL: [Self; 41] = [
        Self::ParseUnexpectedToken,
        Self::ParseUnterminatedString,
        Self::ParseInvalidNumber,
        Self::ParseMissingSemicolon,
        Self::ParseUnmatchedBracket,
        Self::ParseInvalidOperator,
        Self::TypeMismatch,
        Self::TypeInferenceFailed,
        Self::TypeAnnotationRequired,
        Self::TypeNotCallable,
        Self::TypeNotIndexable,
        Self::TypeNotIterable,
        Self::NameNotFound,
        Self::NameAlreadyDefined,
        Self::NameShadowWarning,
        Self::NameImportFailed,
        Self::NamePrivateAccess,
        Self::FuncWrongArgCount,
        Self::FuncWrongArgType,
        Self::FuncMissingReturn,
        Self::FuncReturnTypeMismatch,
        Self::FuncRecursionLimit,
        Self::ControlBreakOutside,
        Self::ControlContinueOutside,
        Self::ControlUnreachableCode,
        Self::ControlMissingCase,
        Self::AsyncAwaitOutside,
        Self::AsyncMissingAwait,
        Self::ConcurrencyDataRace,
        Self::ConcurrencyDeadlock,
        Self::TranspileUnsupported,
        Self::TranspileRustcError,
        Self::TranspileBorrowError,
        Self::TranspileLifetimeError,
        Self::RuntimeDivisionByZero,
        Self::RuntimeIndexOutOfBounds,
        Self::RuntimeNullReference,
        Self::RuntimeStackOverflow,
        Self::EffectNotHandled,
        Self::EffectHandlerMismatch,
        Self::EffectResumeMissing,
    ];

    /// Stable ML label for this class.
    pub fn label(self) -> u8 {
        self as u8
    }

    /// Inverse of [`Self::label`]. Unmapped labels yield `Unknown`.
    pub fn from_label(label: u8) -> Self {
        Self::ALL
            .iter()
            .copied()
            .find(|c| c.label() == label)
            .unwrap_or(Self::Unknown)
    }

    /// Short fix suggestion shown to the user. Never empty.
    pub fn fix_suggestion(self) -> &'static str {
        match self {
            Self::ParseUnexpectedToken => "Check syntax near the reported location",
            Self::ParseUnterminatedString => "Add closing quote to string literal",
            Self::ParseInvalidNumber => "Check numeric literal format (e.g., 1.0, 0x1F)",
            Self::ParseMissingSemicolon => "Add semicolon at end of statement",
            Self::ParseUnmatchedBracket => "Match opening and closing brackets",
            Self::ParseInvalidOperator => "Use valid operator (+, -, *, /, ==, etc.)",
            Self::TypeMismatch => "Convert type with .into() or explicit cast",
            Self::TypeInferenceFailed => "Add explicit type annotation: let x: Type = ...",
            Self::TypeAnnotationRequired => "Specify type for function parameter or return",
            Self::TypeNotCallable => "Ensure value is a function before calling",
            Self::TypeNotIndexable => "Use array or map type for indexing",
            Self::TypeNotIterable => "Implement IntoIterator or use .iter()",
            Self::NameNotFound => "Define variable or import from module",
            Self::NameAlreadyDefined => "Rename to avoid duplicate definition",
            Self::NameShadowWarning => "Consider renaming to avoid shadowing",
            Self::NameImportFailed => "Check module path and availability",
            Self::NamePrivateAccess => "Use pub keyword or access via public API",
            Self::FuncWrongArgCount => "Provide correct number of arguments",
            Self::FuncWrongArgType => "Convert argument to expected type",
            Self::FuncMissingReturn => "Add return statement or change return type to ()",
            Self::FuncReturnTypeMismatch => "Match return value with declared type",
            Self::FuncRecursionLimit => "Add base case or use iteration",
            Self::ControlBreakOutside => "Use break only inside loop",
            Self::ControlContinueOutside => "Use continue only inside loop",
            Self::ControlUnreachableCode => "Remove code after return/break/continue",
            Self::ControlMissingCase => "Add missing match arms or use _ => wildcard",
            Self::AsyncAwaitOutside => "Mark function as async or remove await",
            Self::AsyncMissingAwait => "Add .await to Future value",
            Self::ConcurrencyDataRace => "Use Arc<Mutex<T>> for shared state",
            Self::ConcurrencyDeadlock => "Review lock ordering, use try_lock",
            Self::TranspileUnsupported => "Use alternative Ruchy idiom",
            Self::TranspileRustcError => "Review generated Rust code for issues",
            Self::TranspileBorrowError => "Clone value or restructure ownership",
            Self::TranspileLifetimeError => "Add lifetime annotations or use owned types",
            Self::RuntimeDivisionByZero => "Check denominator is non-zero",
            Self::RuntimeIndexOutOfBounds => "Validate index against length",
            Self::RuntimeNullReference => "Use Option<T> and handle None case",
            Self::RuntimeStackOverflow => "Reduce recursion depth or use iteration",
            Self::EffectNotHandled => "Wrap in handle block with appropriate handler",
            Self::EffectHandlerMismatch => "Match handler signature with effect",
            Self::EffectResumeMissing => "Add resume call in effect handler",
            Self::Unknown => "Review the error message for details",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RuchyErrorCategory;
    use std::collections::HashSet;

    #[test]
    fn test_164_label_round_trip_all_classes() {
        for c in RuchyErrorCategory::ALL {
            assert_eq!(RuchyErrorCategory::from_label(c.label()), c, "{c:?}");
        }
    }

    #[test]
    fn test_164_labels_are_unique() {
        let labels: HashSet<u8> = RuchyErrorCategory::ALL.iter().map(|c| c.label()).collect();
        assert_eq!(labels.len(), RuchyErrorCategory::ALL.len());
        assert!(!labels.contains(&RuchyErrorCategory::Unknown.label()));
    }

    #[test]
    fn test_164_unmapped_label_is_unknown() {
        let mapped: HashSet<u8> = RuchyErrorCategory::ALL.iter().map(|c| c.label()).collect();
        for l in 0..=u8::MAX {
            if !mapped.contains(&l) {
                assert_eq!(
                    RuchyErrorCategory::from_label(l),
                    RuchyErrorCategory::Unknown,
                    "label {l}"
                );
            }
        }
    }

    #[test]
    fn test_164_class_count_is_41() {
        assert_eq!(RuchyErrorCategory::COUNT, 41);
        assert_eq!(RuchyErrorCategory::ALL.len(), RuchyErrorCategory::COUNT);
    }

    #[test]
    fn test_164_every_class_has_non_empty_fix() {
        for c in RuchyErrorCategory::ALL
            .iter()
            .copied()
            .chain([RuchyErrorCategory::Unknown])
        {
            assert!(!c.fix_suggestion().is_empty(), "{c:?}");
        }
    }
}
