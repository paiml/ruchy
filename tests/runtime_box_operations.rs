// RUNTIME-BOX: Box<T> Runtime Operations
// EXTREME TDD: RED → GREEN → REFACTOR → FAST

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ========================================
// RED PHASE: Tests that MUST fail initially
// ========================================

#[test]
fn test_red_box_new_simple() {
    // RED: Box::new() should work but currently hangs
    let code = r#"
fn main() {
    let x = 42;
    let boxed = Box::new(x);
    println(1);
}
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
}

#[test]
fn test_red_box_new_string() {
    // RED: Box::new() with String
    let code = r#"
fn main() {
    let s = "hello";
    let boxed = Box::new(s);
    println(2);
}
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
}

#[test]
fn test_red_box_deref() {
    // RED: Dereferencing Box should work
    let code = r#"
fn main() {
    let boxed = Box::new(42);
    let value = *boxed;
    println(value);
}
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_red_box_in_enum_variant() {
    // RED: Using Box in enum variant construction
    let code = r#"
enum Tree {
    Leaf(i32),
    Node(i32, Box<Tree>, Box<Tree>)
}

fn main() {
    let leaf1 = Tree::Leaf(1);
    let leaf2 = Tree::Leaf(2);
    let node = Tree::Node(3, Box::new(leaf1), Box::new(leaf2));
    println(3);
}
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_red_box_pattern_match() {
    // RED: Pattern matching on Box in enum
    let code = r#"
enum Tree {
    Leaf(i32),
    Node(i32, Box<Tree>, Box<Tree>)
}

fn main() {
    let leaf1 = Tree::Leaf(10);
    let leaf2 = Tree::Leaf(20);
    let node = Tree::Node(30, Box::new(leaf1), Box::new(leaf2));

    match node {
        Tree::Leaf(val) => println(val),
        Tree::Node(val, left, right) => println(val)
    }
}
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("30"));
}

// ========================================
// BASELINE: Tests that MUST pass (verify enum runtime works)
// ========================================

#[test]
fn test_baseline_enum_without_box_runtime() {
    // BASELINE: Enums work without Box
    let code = r#"
enum Simple {
    A(i32),
    B(String)
}

fn main() {
    let x = Simple::A(42);
    match x {
        Simple::A(val) => println(val),
        Simple::B(s) => println(s)
    }
}
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}
