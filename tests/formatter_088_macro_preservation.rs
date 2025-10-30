// FORMATTER-088: Issue #72 - Macro Call Preservation Tests
#![allow(clippy::ignore_without_reason)] // Property tests run with --ignored flag
#![allow(missing_docs)]

// GitHub: https://github.com/paiml/ruchy/issues/72
//
// BUG: Formatter transforms macro CALLS to macro DEFINITIONS
// Input:  vec![1, 2, 3]
// Output: macro vec(1, 2, 3) { }  (WRONG!)
//
// ROOT CAUSE (Five Whys): Untested code path - formatter doesn't preserve `!` for macro calls

use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: Create temp file with Ruchy code
fn create_temp_file(dir: &TempDir, filename: &str, content: &str) -> PathBuf {
    let path = dir.path().join(filename);
    fs::write(&path, content).unwrap();
    path
}

/// Helper: Run ruchy fmt and return formatted output
fn format_code(input: &str) -> String {
    let temp_dir = TempDir::new().unwrap();
    let input_file = create_temp_file(&temp_dir, "test.ruchy", input);

    // Run formatter with --stdout to get output without modifying file
    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("fmt")
        .arg("--stdout")
        .arg(&input_file)
        .output()
        .expect("failed to execute ruchy fmt");

    String::from_utf8(output.stdout).unwrap()
}

// ==================== RED PHASE TESTS (Expected to FAIL) ====================

#[test]
fn test_formatter_088_01_vec_macro_preserved() {
    // CRITICAL: vec! macro must remain a macro CALL, not become a macro DEFINITION
    let input = "let x = vec![1, 2, 3]";
    let formatted = format_code(input);

    // MUST preserve the `!` suffix - this is a macro CALL
    // Note: Ruchy uses parentheses for macro args, both `vec![...]` and `vec!(...)` are valid
    assert!(formatted.contains("vec!(1, 2, 3)") || formatted.contains("vec![1, 2, 3]"),
        "vec! macro call must be preserved, got: {formatted}");

    // MUST NOT contain "macro vec(" - that's a definition, not a call
    assert!(!formatted.contains("macro vec("),
        "Formatter must NOT transform macro calls into macro definitions");
}

#[test]
fn test_formatter_088_02_println_macro_preserved() {
    // CRITICAL: println! macro must remain a macro CALL
    let input = "println!(\"Hello\")";
    let formatted = format_code(input);

    // MUST preserve the `!` suffix
    assert_eq!(formatted, "println!(\"Hello\")",
        "println! macro call must be preserved");

    // MUST NOT contain "macro println(" - that's a definition
    assert!(!formatted.contains("macro println("),
        "Formatter must NOT transform println! into macro definition");
}

#[test]
fn test_formatter_088_03_assert_macro_preserved() {
    // CRITICAL: assert! macro must remain a macro CALL
    let input = "assert!(x > 0)";
    let formatted = format_code(input);

    // MUST preserve the `!` suffix
    assert_eq!(formatted, "assert!(x > 0)",
        "assert! macro call must be preserved");

    // MUST NOT contain "macro assert(" - that's a definition
    assert!(!formatted.contains("macro assert("),
        "Formatter must NOT transform assert! into macro definition");
}

#[test]
fn test_formatter_088_04_custom_macro_preserved() {
    // CRITICAL: Custom macros must remain macro CALLS
    let input = "my_macro!(args)";
    let formatted = format_code(input);

    // MUST preserve the `!` suffix
    assert_eq!(formatted, "my_macro!(args)",
        "Custom macro calls must be preserved");

    // MUST NOT contain "macro my_macro(" - that's a definition
    assert!(!formatted.contains("macro my_macro("),
        "Formatter must NOT transform custom macros into macro definitions");
}

#[test]
fn test_formatter_088_05_macro_in_function() {
    // CRITICAL: Macros inside functions must be preserved
    let input = r#"fun test_vec_macro() {
    let numbers = vec![1, 2, 3]
    println!("Numbers: {:?}", numbers)
}"#;

    let formatted = format_code(input);

    // MUST preserve vec! and println! as macro CALLS
    assert!(formatted.contains("vec!(1, 2, 3)") || formatted.contains("vec![1, 2, 3]"),
        "vec! macro must be preserved in function body, got: {formatted}");
    assert!(formatted.contains("println!("),
        "println! macro must be preserved in function body");

    // MUST NOT transform to macro definitions
    assert!(!formatted.contains("macro vec("),
        "Formatter must NOT transform vec! to macro definition");
    assert!(!formatted.contains("macro println("),
        "Formatter must NOT transform println! to macro definition");
}

#[test]
fn test_formatter_088_06_nested_macro_calls() {
    // CRITICAL: Nested macro calls must all be preserved
    let input = "vec![assert!(x > 0), println!(\"test\")]";
    let formatted = format_code(input);

    // MUST preserve all three macros as CALLS
    assert!(formatted.contains("vec!(") || formatted.contains("vec!["),
        "vec! must be preserved, got: {formatted}");
    assert!(formatted.contains("assert!("),
        "assert! must be preserved");
    assert!(formatted.contains("println!("),
        "println! must be preserved");

    // MUST NOT transform any to definitions
    assert!(!formatted.contains("macro vec("),
        "vec! must not become definition");
    assert!(!formatted.contains("macro assert("),
        "assert! must not become definition");
    assert!(!formatted.contains("macro println("),
        "println! must not become definition");
}

// ==================== PROPERTY TESTS ====================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Formatter NEVER transforms macro calls to macro definitions
        /// Invariant: If input contains `name!(`, output must also contain `name!(`
        #[test]
        #[ignore = "Run with: cargo test --test formatter_088_macro_preservation property_tests -- --ignored"]
        fn prop_macro_calls_never_become_definitions(
            macro_name in "[a-z_][a-z0-9_]{0,20}",
            arg in 0..1000i32
        ) {
            let input = format!("{macro_name}!({arg})");
            let formatted = format_code(&input);

            // INVARIANT: Macro call syntax must be preserved
            prop_assert!(
                formatted.contains(&format!("{macro_name}!(")),
                "Macro call {macro_name}!(...) must be preserved, not transformed to definition"
            );

            // INVARIANT: Must NOT become a macro definition
            prop_assert!(
                !formatted.contains(&format!("macro {macro_name}(")),
                "Formatter must NEVER transform macro calls into definitions"
            );
        }

        /// Property: vec! macro with any integer list is preserved
        #[test]
        #[ignore]
        fn prop_vec_macro_always_preserved(values: Vec<i32>) {
            let values_str = values.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", ");
            let input = format!("vec![{values_str}]");
            let formatted = format_code(&input);

            // INVARIANT: vec! must remain vec!, not become "macro vec("
            // Note: Ruchy formats as vec!() with parentheses, both [] and () are valid
            prop_assert!(
                formatted.contains("vec!(") || formatted.contains("vec!["),
                "vec! macro must always be preserved as macro call, got: {}", formatted
            );
            prop_assert!(
                !formatted.contains("macro vec("),
                "vec! must NEVER become macro definition"
            );
        }
    }
}
