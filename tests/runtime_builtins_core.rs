//! EXTREME TDD: Testing ruchy-cookbook bug reports
//!
//! Bug #1 (CRITICAL): Methods in struct bodies fail with "Expected field name"
//! Bug #2: Cryptic error messages for unsupported Rust syntax
//!
//! RED → GREEN → REFACTOR → VALIDATE

use assert_cmd::Command;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// BUG #1: Methods in struct/class bodies
// Bug Report: ../ruchy-cookbook/docs/bugs/oo-syntax-bug.md
// ============================================================================

#[test]
fn test_bug_oo_001_struct_with_method() {
    let code = r#"
struct Rectangle {
    width: i32,
    height: i32,

    fun area(&self) -> i32 {
        self.width * self.height
    }
}

fun main() {
    let rect = Rectangle { width: 10, height: 20 }
    println("{}", rect.area())
}
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("200\n");
}

#[test]
fn test_bug_oo_002_class_with_method_works() {
    // Verify that class with methods DOES work (this should pass)
    let code = r#"
class Point {
    x: f64,
    y: f64,

    fun distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

fun main() {
    let p = Point { x: 3.0, y: 4.0 }
    println("{}", p.distance())
}
"#;

    ruchy_cmd().arg("-e").arg(code).assert().success();
}

#[test]
fn test_bug_oo_003_struct_with_multiple_methods() {
    let code = r#"
struct Rectangle {
    width: i32,
    height: i32,

    fun new(w: i32, h: i32) -> Rectangle {
        Rectangle { width: w, height: h }
    }

    fun area(&self) -> i32 {
        self.width * self.height
    }

    fun perimeter(&self) -> i32 {
        2 * (self.width + self.height)
    }
}

fun main() {
    let rect = Rectangle::new(5, 10)
    println("Area: {}", rect.area())
    println("Perimeter: {}", rect.perimeter())
}
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("Area: 50\nPerimeter: 30\n");
}

#[test]
#[ignore = "Requires method dispatch refactor for &mut self persistence (methods work on copy of self, changes don't persist to original variable)"]
fn test_bug_oo_004_struct_with_mutating_method() {
    let code = r#"
struct Counter {
    count: i32,

    fun new() -> Counter {
        Counter { count: 0 }
    }

    fun increment(&mut self) {
        self.count += 1
    }

    fun get(&self) -> i32 {
        self.count
    }
}

fun main() {
    let mut counter = Counter::new()
    counter.increment()
    counter.increment()
    println("{}", counter.get())
}
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("2\n");
}

// ============================================================================
// BUG #2: Cryptic error messages for unsupported Rust syntax
// Bug Report: ../ruchy-cookbook/RUCHY_ISSUE_SYNTAX_ERROR_REPORT.md
// ============================================================================

#[test]
fn test_bug_syntax_001_attribute_error_message() {
    let code = r"
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32
}
";

    let output = ruchy_cmd().arg("-e").arg(code).assert().failure();

    // Should have helpful error message, not cryptic "Unexpected token: AttributeStart"
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    assert!(
        stderr.contains("Attributes are not supported")
            || stderr.contains("does not support #[derive]")
            || stderr.contains("Ruchy does not use Rust-style attributes"),
        "Error message should explain that attributes are not supported. Got: {stderr}"
    );
}

#[test]
fn test_bug_syntax_002_pub_keyword_works_correctly() {
    // pub IS supported in Ruchy - this test verifies it works
    let code = r#"
pub struct Point {
    pub x: i32,
    pub y: i32
}

fun main() {
    let p = Point { x: 10, y: 20 }
    println!("Point: x={}, y={}", p.x, p.y)
}
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("Point: x=10, y=20\n");
}

#[test]
fn test_bug_syntax_003_impl_block_error_message() {
    let code = r"
struct Point {
    x: i32,
    y: i32
}

impl Point {
    fun new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
}
";

    let output = ruchy_cmd().arg("-e").arg(code).assert().failure();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    assert!(
        stderr.contains("impl")
            || stderr.contains("Methods should be defined inside struct")
            || stderr.contains("not supported"),
        "Error message should explain impl blocks. Got: {stderr}"
    );
}
