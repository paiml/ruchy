// DEFECT-PARSER-014: Impl blocks with generic target types
// Tests for impl<T> Trait for Type<T> syntax

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

fn test_code(code: &str) {
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::thread;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let thread_id = thread::current().id();
    let temp_file = PathBuf::from(format!("/tmp/test_impl_generic_target_{}_{:?}.ruchy", timestamp, thread_id));
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
    test_code(r#"
impl<T> Default for Point<T> {
    fn default() -> Point<T> {
        Point { x: 0, y: 0 }
    }
}
"#);
}

#[test]
fn test_impl_trait_bound_for_generic_type() {
    // Test: impl<T: Clone> Clone for Box<T>
    test_code(r#"
impl<T: Clone> Clone for Box<T> {
    fn clone(&self) -> Box<T> {
        Box::new((**self).clone())
    }
}
"#);
}

#[test]
fn test_impl_display_for_generic_wrapper() {
    // Test: impl<T: Display> Display for Wrapper<T>
    test_code(r#"
impl<T: Display> Display for Wrapper<T> {
    fn fmt(&self) -> String {
        format!("{}", self.inner)
    }
}
"#);
}

#[test]
fn test_impl_trait_for_multiple_generic_params() {
    // Test: impl<K, V> Map for HashMap<K, V>
    test_code(r#"
impl<K, V> Map for HashMap<K, V> {
    fn insert(&mut self, key: K, value: V) {
        self.data.insert(key, value)
    }
}
"#);
}

#[test]
fn test_impl_trait_for_nested_generics() {
    // Test: impl<T> Iterator for Vec<Vec<T> > (space required due to >> lexing)
    test_code(r#"
impl<T> Iterator for Vec<Vec<T> > {
    fn next(&mut self) -> Option<Vec<T> > {
        self.items.pop()
    }
}
"#);
}

#[test]
fn test_impl_from_for_generic() {
    // Test: impl<T> From<T> for Option<T>
    test_code(r#"
impl<T> From<T> for Option<T> {
    fn from(value: T) -> Option<T> {
        Some(value)
    }
}
"#);
}

#[test]
fn test_impl_trait_no_generics_still_works() {
    // Regression: impl Trait for Type without generics
    test_code(r#"
impl Default for Point {
    fn default() -> Point {
        Point { x: 0, y: 0 }
    }
}
"#);
}

#[test]
fn test_impl_generic_type_without_trait() {
    // Regression: impl<T> Type<T> without trait
    test_code(r#"
impl<T> Point<T> {
    fn new(x: T, y: T) -> Point<T> {
        Point { x, y }
    }
}
"#);
}
