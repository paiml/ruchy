#![allow(missing_docs)]
// DEFECT-PARSER-014: Impl blocks with generic target types
// Tests for impl<T> Trait for Type<T> syntax

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

fn test_code(code: &str) {
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let thread_id = thread::current().id();
    let temp_file = PathBuf::from(format!(
        "/tmp/test_impl_generic_target_{timestamp}_{thread_id:?}.ruchy"
    ));
    fs::write(&temp_file, code).expect("Failed to write temp file");

    ruchy_cmd()
        .arg("check")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Syntax is valid"));

    let _ = fs::remove_file(&temp_file);
}

#[test]
fn test_impl_trait_for_generic_type() {
    // Original bug: impl<T> Trait for Type<T>
    test_code(
        r"
impl<T> Default for Point<T> {
    fn default() -> Point<T> {
        Point { x: 0, y: 0 }
    }
}
",
    );
}

#[test]
fn test_impl_trait_bound_for_generic_type() {
    // Test: impl<T: Clone> Clone for Box<T>
    test_code(
        r"
impl<T: Clone> Clone for Box<T> {
    fn clone(&self) -> Box<T> {
        Box::new((**self).clone())
    }
}
",
    );
}

#[test]
fn test_impl_display_for_generic_wrapper() {
    // Test: impl<T: Display> Display for Wrapper<T>
    test_code(
        r#"
impl<T: Display> Display for Wrapper<T> {
    fn fmt(&self) -> String {
        format!("{}", self.inner)
    }
}
"#,
    );
}

#[test]
fn test_impl_trait_for_multiple_generic_params() {
    // Test: impl<K, V> Map for HashMap<K, V>
    test_code(
        r"
impl<K, V> Map for HashMap<K, V> {
    fn insert(&mut self, key: K, value: V) {
        self.data.insert(key, value)
    }
}
",
    );
}

#[test]
fn test_impl_trait_for_nested_generics() {
    // Test: impl<T> Iterator for Vec<Vec<T> > (space required due to >> lexing)
    test_code(
        r"
impl<T> Iterator for Vec<Vec<T> > {
    fn next(&mut self) -> Option<Vec<T> > {
        self.items.pop()
    }
}
",
    );
}

#[test]
fn test_impl_from_for_generic() {
    // Test: impl<T> From<T> for Option<T>
    test_code(
        r"
impl<T> From<T> for Option<T> {
    fn from(value: T) -> Option<T> {
        Some(value)
    }
}
",
    );
}

#[test]
fn test_impl_trait_no_generics_still_works() {
    // Regression: impl Trait for Type without generics
    test_code(
        r"
impl Default for Point {
    fn default() -> Point {
        Point { x: 0, y: 0 }
    }
}
",
    );
}

#[test]
fn test_impl_generic_type_without_trait() {
    // Regression: impl<T> Type<T> without trait
    test_code(
        r"
impl<T> Point<T> {
    fn new(x: T, y: T) -> Point<T> {
        Point { x, y }
    }
}
",
    );
}

// Property Tests for DEFECT-014
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Property: All valid type names should parse in impl blocks
    proptest! {
        #[test]
        fn prop_impl_with_arbitrary_type_names(
            type_name in "[A-Z][a-zA-Z0-9]{0,10}",
            trait_name in "[A-Z][a-zA-Z0-9]{0,10}"
        ) {
            let code = format!("impl {trait_name} for {type_name} {{}}");
            let result = std::panic::catch_unwind(|| {
                test_code(&code);
            });
            // Should not panic - either parse successfully or fail gracefully
            prop_assert!(result.is_ok(), "Parser panicked on valid syntax");
        }
    }

    proptest! {
        #[test]
        fn prop_impl_with_generic_type_names(
            type_name in "[A-Z][a-zA-Z0-9]{0,10}",
            trait_name in "[A-Z][a-zA-Z0-9]{0,10}",
            generic_param in "[A-Z]"
        ) {
            let code = format!("impl<{generic_param}> {trait_name} for {type_name}<{generic_param}> {{}}");
            let result = std::panic::catch_unwind(|| {
                test_code(&code);
            });
            prop_assert!(result.is_ok(), "Parser panicked on valid generic syntax");
        }
    }

    proptest! {
        #[test]
        fn prop_keyword_types_as_impl_targets(
            keyword in prop::sample::select(vec!["Option", "Result", "Some", "None", "Ok", "Err"])
        ) {
            let code = format!("impl Display for {keyword} {{}}");
            let result = std::panic::catch_unwind(|| {
                test_code(&code);
            });
            prop_assert!(result.is_ok(), "Parser should accept keyword type names");
        }
    }

    proptest! {
        #[test]
        fn prop_keyword_traits_in_impl(
            keyword in prop::sample::select(vec!["From", "Default"])
        ) {
            let code = format!("impl {keyword} for MyType {{}}");
            let result = std::panic::catch_unwind(|| {
                test_code(&code);
            });
            prop_assert!(result.is_ok(), "Parser should accept keyword trait names");
        }
    }

    proptest! {
        #[test]
        fn prop_keyword_method_names(
            keyword in prop::sample::select(vec!["from", "default"])
        ) {
            let code = format!("impl MyTrait for MyType {{ fn {keyword}() {{}} }}");
            let result = std::panic::catch_unwind(|| {
                test_code(&code);
            });
            prop_assert!(result.is_ok(), "Parser should accept keyword method names");
        }
    }

    proptest! {
        #[test]
        fn prop_multiple_generic_params(
            num_params in 1usize..4usize
        ) {
            let params: Vec<String> = (0..num_params)
                .map(|i| ((b'A' + u8::try_from(i).unwrap_or(0)) as char).to_string())
                .collect();
            let generic_list = params.join(", ");
            let type_params = params.join(", ");

            let code = format!(
                "impl<{generic_list}> MyTrait for MyType<{type_params}> {{}}"
            );
            let result = std::panic::catch_unwind(|| {
                test_code(&code);
            });
            prop_assert!(result.is_ok(), "Parser should handle multiple generic params");
        }
    }
}
