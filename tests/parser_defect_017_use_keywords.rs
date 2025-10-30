// DEFECT-PARSER-017: Keywords in use statement paths
//
// ROOT CAUSE: Use statement path parser doesn't accept keyword tokens in paths
// Error: "Expected identifier, 'super', 'self', '*', or '{' after '::'"
//
// TEST STRATEGY:
// 1. RED tests for broken keyword paths in use statements
// 2. Regression tests for existing use statement functionality
//
// From: ruchy-book/test/extracted-examples/appendix-b-syntax-reference_example_28.ruchy

use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

fn test_code(code: &str) {
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::thread;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let thread_id = thread::current().id();
    let temp_file = PathBuf::from(format!(
        "/tmp/test_use_keywords_{timestamp}_{thread_id:?}.ruchy"
    ));

    fs::write(&temp_file, code).expect("Failed to write test file");

    let result = ruchy_cmd()
        .arg("check")
        .arg(&temp_file)
        .assert();

    // Clean up
    let _ = fs::remove_file(&temp_file);

    result.success();
}

// ============================================================================
// RED TESTS: These should fail initially (the actual defect)
// ============================================================================

#[test]
fn test_use_module_keyword() {
    // use path::module - 'module' is a keyword
    // From example 28: use unix_specific::module
    test_code("use unix_specific::module");
}

#[test]
fn test_use_type_keyword() {
    // use path::type - 'type' is a keyword
    test_code("use core::type");
}

#[test]
fn test_use_fn_keyword() {
    // use path::fn - 'fn' is a keyword
    test_code("use helpers::fn");
}

#[test]
fn test_use_const_keyword() {
    // use path::const - 'const' is a keyword
    test_code("use config::const");
}

#[test]
fn test_use_trait_keyword() {
    // use path::trait - 'trait' is a keyword
    test_code("use core::trait");
}

// ============================================================================
// REGRESSION TESTS: These should already pass
// ============================================================================

#[test]
fn test_use_simple_path_still_works() {
    // Regression: simple use statements should still work
    test_code("use std::collections::HashMap");
}

#[test]
fn test_use_wildcard_still_works() {
    // Regression: wildcard imports should still work
    test_code("use std::io::*");
}

#[test]
fn test_use_multiple_imports_still_works() {
    // Regression: multiple imports should still work
    test_code("use std::fs::{File, OpenOptions}");
}

// ============================================================================
// PROPERTY TESTS: Randomized validation (REFACTOR phase)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_use_keyword_segments(
            seg1 in "[a-z][a-z0-9_]{0,8}",
            keyword in prop::sample::select(vec!["module", "type", "fn", "const", "trait", "for", "match"])
        ) {
            let code = format!("use {seg1}::{keyword}");
            let result = std::panic::catch_unwind(|| {
                test_code(&code);
            });
            prop_assert!(result.is_ok(), "Parser panicked on use {}::{}", seg1, keyword);
        }

        #[test]
        fn prop_use_nested_keywords(
            seg1 in "[a-z][a-z0-9_]{0,5}",
            keyword1 in prop::sample::select(vec!["module", "type", "const"]),
            keyword2 in prop::sample::select(vec!["fn", "trait", "impl"])
        ) {
            let code = format!("use {seg1}::{keyword1}::{keyword2}");
            let result = std::panic::catch_unwind(|| {
                test_code(&code);
            });
            prop_assert!(result.is_ok(), "Parser panicked on nested keywords");
        }

        #[test]
        fn prop_use_keyword_start(
            keyword in prop::sample::select(vec!["module", "type", "const", "trait"])
        ) {
            let code = format!("use {keyword}");
            let result = std::panic::catch_unwind(|| {
                test_code(&code);
            });
            prop_assert!(result.is_ok(), "Parser panicked on keyword start: {}", keyword);
        }
    }
}
