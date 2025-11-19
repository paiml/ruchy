#![allow(missing_docs)]
// DEFECT-PARSER-013: ColonColon (::) operator in import statements
// Tests for Rust-style module path syntax

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
        "/tmp/test_coloncolon_{timestamp}_{thread_id:?}.ruchy"
    ));
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
fn test_simple_coloncolon_import() {
    // Original bug: import std::fs
    test_code(
        r"
import std::fs
",
    );
}

#[test]
fn test_nested_coloncolon_import() {
    // Test: import std::collections::HashMap
    test_code(
        r"
import std::collections::HashMap
",
    );
}

#[test]
fn test_deeply_nested_coloncolon_import() {
    // Test: import std::io::fs::File
    test_code(
        r"
import std::io::fs::File
",
    );
}

#[test]
fn test_coloncolon_import_with_usage() {
    // Test: import then use the module
    test_code(
        r#"
import std::fs

fn main() {
    fs::write("test.txt", "content")
}
"#,
    );
}

#[test]
fn test_from_coloncolon_import() {
    // Test: from std::io import println
    test_code(
        r"
from std::io import println
",
    );
}

#[test]
fn test_from_nested_coloncolon_import() {
    // Test: from std::collections::map import HashMap
    test_code(
        r"
from std::collections::map import HashMap
",
    );
}

#[test]
fn test_mixed_dot_and_coloncolon() {
    // Test: Mixing . and :: should both work
    test_code(
        r"
import std.fs
import std::io
",
    );
}

#[test]
fn test_coloncolon_function_call() {
    // Test: Direct :: function calls (already working)
    test_code(
        r#"
fn main() {
    std::println("test")
}
"#,
    );
}

#[test]
fn test_coloncolon_with_alias() {
    // Test: import std::fs as filesystem
    test_code(
        r"
import std::fs as filesystem
",
    );
}

#[test]
fn test_multiple_coloncolon_imports() {
    // Test: Multiple imports with ::
    test_code(
        r#"
import std::fs
import std::io
import std::collections::HashMap

fn main() {
    fs::write("test.txt", "data")
    println("done")
}
"#,
    );
}

#[test]
fn test_coloncolon_in_from_import_multiple() {
    // Test: from std::io import println, eprintln
    test_code(
        r"
from std::io import println, eprintln
",
    );
}

#[test]
fn test_coloncolon_wildcard_import() {
    // Test: from std::fs import *
    test_code(
        r"
from std::fs import *
",
    );
}
