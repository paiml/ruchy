#![allow(missing_docs)]
// DEFECT-PARSER-015: pub keyword on modules
// Tests for pub mod syntax

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
    let temp_file = PathBuf::from(format!("/tmp/test_pub_mod_{timestamp}_{thread_id:?}.ruchy"));
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
fn test_pub_mod_basic() {
    // Basic pub mod
    test_code(r"
pub mod utils {
    fn helper() {}
}
");
}

#[test]
fn test_pub_mod_nested() {
    // Nested pub mod (example 26)
    test_code(r"
mod graphics {
    pub mod shapes {
        pub fn draw_circle() {}
    }
}
");
}

#[test]
fn test_pub_mod_empty() {
    // Empty pub mod
    test_code(r"
pub mod empty {}
");
}

#[test]
fn test_pub_mod_with_functions() {
    // pub mod with multiple functions
    test_code(r"
pub mod network {
    fn connect() {}
    pub fn public_function() {}
}
");
}

#[test]
fn test_regular_mod_still_works() {
    // Regression: regular mod without pub should still work
    test_code(r"
mod private {
    fn helper() {}
}
");
}
