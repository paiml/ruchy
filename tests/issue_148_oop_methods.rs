//! Regression tests for Issue #148: OOP Method Syntax
//!
//! Verifies struct methods in body syntax works correctly.

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
fn test_issue_148_01_struct_methods_in_body() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
struct Calculator {
    value: i32,

    pub fun new() -> Calculator {
        Calculator { value: 0 }
    }

    pub fun add(&mut self, amount: i32) {
        self.value = self.value + amount
    }

    pub fun get(&self) -> i32 {
        self.value
    }
}

fun main() {
    let mut c = Calculator::new();
    c.add(5);
    println!("{}", c.get())
}
"#,
    )
    .unwrap();

    // Should transpile and compile successfully
    ruchy_cmd()
        .arg("compile")
        .arg(&file_path)
        .assert()
        .success();
}

#[test]
fn test_issue_148_02_struct_methods_transpile_to_impl() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
struct Point {
    x: i32,
    y: i32,

    pub fun new(x: i32, y: i32) -> Point {
        Point { x: x, y: y }
    }
}

fun main() {
    let p = Point::new(3, 4);
    println!("{} {}", p.x, p.y)
}
"#,
    )
    .unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // Methods should be in impl block
    assert!(
        output.contains("impl Point"),
        "Expected impl Point block: {output}"
    );
    assert!(
        output.contains("pub fn new"),
        "Expected pub fn new in impl: {output}"
    );
}

#[test]
fn test_issue_148_03_multiple_methods() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
struct Counter {
    count: i32,

    pub fun new() -> Counter {
        Counter { count: 0 }
    }

    pub fun increment(&mut self) {
        self.count = self.count + 1
    }

    pub fun decrement(&mut self) {
        self.count = self.count - 1
    }

    pub fun reset(&mut self) {
        self.count = 0
    }
}

fun main() {
    let mut c = Counter::new();
    c.increment();
    c.increment();
    c.decrement();
    println!("{}", c.count)
}
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("compile")
        .arg(&file_path)
        .assert()
        .success();
}
