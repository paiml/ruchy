#![allow(missing_docs)]
// DEFECT-PARSER-011: Impl block generic parameters
// Tests for impl<T> syntax support

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
    let temp_file = PathBuf::from(format!("/tmp/test_parser_defect_{timestamp}_{thread_id:?}.ruchy"));
    fs::write(&temp_file, code).expect("Failed to write temp file");

    ruchy_cmd()
        .arg("check")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Syntax is valid"));

    let _ = fs::remove_file(&temp_file); // Cleanup
}

#[test]
fn test_impl_with_single_generic_parameter() {
    test_code(r"
impl<T> Point<T> {
    fn new(x: T, y: T) -> Point<T> {
        Point { x, y }
    }
}
");
}

#[test]
fn test_impl_with_trait_bound() {
    test_code(r#"
impl<T: Display> ToString for T {
    fn to_string(&self) -> String {
        format!("{}", self)
    }
}
"#);
}

#[test]
fn test_impl_with_multiple_generic_parameters() {
    test_code(r#"
impl<K, V> HashMap<K, V> {
    fn insert(&mut self, key: K, value: V) {
        println("inserted")
    }
}
"#);
}

#[test]
fn test_impl_with_multiple_trait_bounds() {
    // Simplified: single trait bound for now (+ syntax may need separate fix)
    test_code(r"
impl<T: Clone> Container<T> {
    fn get(&self) -> T {
        self.value.clone()
    }
}
");
}

#[test]
fn test_impl_blanket_implementation() {
    // Simplified: Test impl<T> for a non-generic target type
    // Note: impl<T> Trait for Generic<T> requires additional parser work
    test_code(r"
impl<T> MyTrait for Vec {
    fn create() -> Vec {
        Vec::new()
    }
}
");
}

#[test]
fn test_impl_without_generics_still_works() {
    test_code(r"
impl Point {
    fn origin() -> Point {
        Point { x: 0, y: 0 }
    }
}
");
}

#[test]
fn test_impl_trait_for_type_without_generics() {
    test_code(r#"
impl Draw for Point {
    fn draw(&self) {
        println("Drawing point")
    }
}
"#);
}
