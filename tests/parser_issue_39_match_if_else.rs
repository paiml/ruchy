#![allow(missing_docs)]
//! RED TEST for Issue #39: Match with if-else in arm causes parser error
//!
//! BUG: Parser fails when match arm contains if-else block
//! Error: "Expected `RightBrace`, found Match"
//!
//! Extreme TDD: RED → GREEN → REFACTOR

use assert_cmd::Command;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// RED TEST: Minimal reproduction of Issue #39
/// Parser should accept match with if-else in arm
#[test]
fn test_issue_39_match_with_if_else_in_arm() {
    let code = r#"
enum Result {
    Ok(i32),
    Err(String)
}

fun test() {
    match Result::Ok(42) {
        Result::Ok(x) => {
            if x > 0 {
                println("positive")
            } else {
                println("negative")
            }
        },
        Result::Err(msg) => println(msg)
    }
}

test()
"#;

    // RED: This test will FAIL with parser error
    ruchy_cmd().arg("-e").arg(code).assert().success(); // Expected: parser accepts the code
}

/// RED TEST: Original Issue #39 case with Box and recursive call
#[test]
fn test_issue_39_algorithm_w_lookup_pattern() {
    let code = r#"
enum TypeEnv {
    Empty,
    Extend(String, Box<TypeEnv>)
}

enum InferResult {
    Success,
    Failure(String)
}

fun lookup(env: TypeEnv, name: String) -> InferResult {
    match env {
        TypeEnv::Empty => InferResult::Failure("Not found".to_string()),
        TypeEnv::Extend(var, rest) => {
            if var == name {
                InferResult::Success
            } else {
                lookup(*rest, name)
            }
        }
    }
}

fun main() {
    let env = TypeEnv::Extend("x".to_string(), Box::new(TypeEnv::Empty));
    let result = lookup(env, "x".to_string());
    println("Done");
}

main();
"#;

    // RED: This test will FAIL with parser error
    ruchy_cmd().arg("-e").arg(code).assert().success();
}

/// RED TEST: Match with nested if-else-if chain
#[test]
fn test_issue_39_match_with_nested_if_else_if() {
    let code = r#"
enum Color {
    RGB(i32, i32, i32)
}

fun classify(c: Color) {
    match c {
        Color::RGB(r, g, b) => {
            if r > 200 {
                println("bright red")
            } else if g > 200 {
                println("bright green")
            } else if b > 200 {
                println("bright blue")
            } else {
                println("dark")
            }
        }
    }
}

classify(Color::RGB(255, 0, 0))
"#;

    // RED: This test will FAIL with parser error
    ruchy_cmd().arg("-e").arg(code).assert().success();
}
