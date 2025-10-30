#![allow(missing_docs)]
// DEFECT-PARSER-016: pub(in path) visibility modifier
//
// ROOT CAUSE: Parser accepts pub(crate) and pub(super) but rejects pub(in path)
// Error message: "Expected 'crate' or 'super' after 'pub('"
//
// TEST STRATEGY:
// 1. Regression tests for working pub(crate) and pub(super)
// 2. RED tests for broken pub(in path) variants
//
// From: ruchy-book/test/extracted-examples/appendix-b-syntax-reference_example_27.ruchy

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
        "/tmp/test_pub_visibility_{timestamp}_{thread_id:?}.ruchy"
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
// REGRESSION TESTS: These should already pass
// ============================================================================

#[test]
fn test_pub_crate_already_works() {
    // pub(crate) visibility - should already work
    test_code("pub(crate) fn crate_visible() {}");
}

#[test]
fn test_pub_super_already_works() {
    // pub(super) visibility - should already work
    test_code("pub(super) fn parent_visible() {}");
}

// ============================================================================
// RED TESTS: These should fail initially (the actual defect)
// ============================================================================

#[test]
fn test_pub_in_crate_path() {
    // pub(in crate::utils) - specific path visibility
    // This is the primary syntax from example 27
    test_code("pub(in crate::utils) fn limited() {}");
}

#[test]
fn test_pub_in_super_path() {
    // pub(in super::utils) - parent module path
    test_code("pub(in super::utils) fn helper() {}");
}

#[test]
fn test_pub_in_self_path() {
    // pub(in self::utils) - current module path
    test_code("pub(in self::module) fn local() {}");
}

#[test]
fn test_pub_in_nested_path() {
    // pub(in crate::a::b::c) - deeply nested path
    test_code("pub(in crate::graphics::shapes::circle) fn draw() {}");
}

#[test]
fn test_pub_in_absolute_path() {
    // pub(in ::top_level) - absolute path from root
    test_code("pub(in ::utils) fn global_helper() {}");
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
        fn prop_pub_in_single_segment(
            segment in "[a-z][a-z0-9_]{0,10}"
        ) {
            let code = format!("pub(in crate::{segment}) fn test() {{}}");
            let result = std::panic::catch_unwind(|| {
                test_code(&code);
            });
            prop_assert!(result.is_ok(), "Parser panicked on pub(in crate::{})", segment);
        }

        #[test]
        fn prop_pub_in_nested_segments(
            seg1 in "[a-z][a-z0-9_]{0,5}",
            seg2 in "[a-z][a-z0-9_]{0,5}",
            seg3 in "[a-z][a-z0-9_]{0,5}"
        ) {
            let code = format!("pub(in crate::{seg1}::{seg2}::{seg3}) fn test() {{}}");
            let result = std::panic::catch_unwind(|| {
                test_code(&code);
            });
            prop_assert!(result.is_ok(), "Parser panicked on nested path");
        }

        #[test]
        fn prop_pub_in_super_path(
            segment in "[a-z][a-z0-9_]{0,10}"
        ) {
            let code = format!("pub(in super::{segment}) fn test() {{}}");
            let result = std::panic::catch_unwind(|| {
                test_code(&code);
            });
            prop_assert!(result.is_ok(), "Parser panicked on pub(in super::{})", segment);
        }

        #[test]
        fn prop_pub_in_self_path(
            segment in "[a-z][a-z0-9_]{0,10}"
        ) {
            let code = format!("pub(in self::{segment}) fn test() {{}}");
            let result = std::panic::catch_unwind(|| {
                test_code(&code);
            });
            prop_assert!(result.is_ok(), "Parser panicked on pub(in self::{})", segment);
        }

        #[test]
        fn prop_pub_in_absolute_path(
            segment in "[a-z][a-z0-9_]{0,10}"
        ) {
            let code = format!("pub(in ::{segment}) fn test() {{}}");
            let result = std::panic::catch_unwind(|| {
                test_code(&code);
            });
            prop_assert!(result.is_ok(), "Parser panicked on pub(in ::{})", segment);
        }
    }
}
