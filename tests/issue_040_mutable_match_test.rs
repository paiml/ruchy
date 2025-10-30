#![allow(missing_docs)]
// Issue #40: Mutable variable increment inside match expression
// Bug: Variable mutations inside match arms don't propagate to outer scope
//
// Five Whys Analysis:
// 1. Why does loop print "Character 0: a" infinitely?
//    - Because `i` is never incremented from 0
// 2. Why is `i` never incremented?
//    - Because `i = i + 1` inside match arm doesn't update outer scope's `i`
// 3. Why doesn't assignment update outer scope?
//    - Because match creates new scope with push_scope() that shadows outer variables
// 4. Why does new scope shadow instead of mutate?
//    - Because pattern bindings use env_set() which creates new bindings, not mutations
// 5. Why doesn't mutation propagate?
//    - Because pop_scope() discards all changes made in match arm scope
//
// ROOT CAUSE: Match expression creates isolated scope that doesn't propagate mutations
// to outer scope variables. Need to track which variables are mutations vs new bindings.

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// RED TEST: Issue #40 - Mutable variable in match expression (original bug report)
#[test]
fn test_issue_040_mutable_in_match_string_iteration() {
    let code = r#"
let s = "abc".to_string();
let mut i = 0;
loop {
    if i >= s.len() {
        break;
    }
    match s.chars().nth(i) {
        Some(ch) => {
            println("Character " + i.to_string() + ": " + ch.to_string());
            i = i + 1;
        }
        None => break
    }
}
println("Done");
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Character 0: a"))
        .stdout(predicate::str::contains("Character 1: b"))
        .stdout(predicate::str::contains("Character 2: c"))
        .stdout(predicate::str::contains("Done"));
}

/// RED TEST: Simple mutable increment in match
#[test]
fn test_issue_040_simple_mutable_increment() {
    let code = r"
let mut x = 0;
match 1 {
    1 => x = x + 1
    _ => {}
}
println(x)
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("1\nnil\n");
}

/// RED TEST: Multiple mutations in match
#[test]
fn test_issue_040_multiple_mutations() {
    let code = r#"
let mut x = 0;
let mut y = 0;
match 1 {
    1 => {
        x = x + 1;
        y = y + 2;
    }
    _ => {}
}
println(x.to_string() + "," + y.to_string())
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("1,2\nnil\n");
}

/// RED TEST: Mutation in nested match
#[test]
fn test_issue_040_nested_match_mutation() {
    let code = r"
let mut counter = 0;
match 1 {
    1 => {
        match 2 {
            2 => counter = counter + 1
            _ => {}
        }
    }
    _ => {}
}
println(counter)
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("1\nnil\n");
}

/// RED TEST: Mutation with pattern binding (ensure pattern vars don't interfere)
#[test]
fn test_issue_040_mutation_with_pattern_binding() {
    let code = r"
let mut sum = 0;
match Some(5) {
    Some(n) => {
        sum = sum + n;
    }
    None => {}
}
println(sum)
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("5\nnil\n");
}

/// RED TEST: Mutation in loop with match (comprehensive)
#[test]
fn test_issue_040_loop_with_match_mutation() {
    let code = r"
let mut i = 0;
let mut sum = 0;
loop {
    if i >= 5 {
        break;
    }
    match i {
        0 => sum = sum + 1
        1 => sum = sum + 2
        2 => sum = sum + 3
        3 => sum = sum + 4
        4 => sum = sum + 5
        _ => {}
    }
    i = i + 1;
}
println(sum)
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("15\nnil\n"); // 1+2+3+4+5 = 15
}

/// RED TEST: Guard condition doesn't prevent mutation
#[test]
fn test_issue_040_mutation_with_guard() {
    let code = r"
let mut x = 0;
let value = 5;
match value {
    n if n > 0 => x = x + n
    _ => {}
}
println(x)
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("5\nnil\n");
}

#[cfg(test)]
mod property_tests {
    
    use std::process::Command;

    /// Property test: Counter increments correctly N times in match
    #[test]
    #[ignore = "Run with: cargo test property_tests -- --ignored"]
    fn proptest_counter_increment_in_match() {
        for n in 0..100 {
            let code = format!(
                r"
let mut counter = 0;
let mut i = 0;
loop {{
    if i >= {n} {{
        break;
    }}
    match i {{
        _ => counter = counter + 1
    }}
    i = i + 1;
}}
println(counter)
"
            );

            let output = Command::new("ruchy")
                .arg("-e")
                .arg(&code)
                .output()
                .expect("Failed to run ruchy");

            let stdout = String::from_utf8_lossy(&output.stdout);
            let result: i32 = stdout
                .lines()
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(-1);

            assert_eq!(
                result, n,
                "Counter should increment {n} times, got {result}"
            );
        }
    }

    /// Property test: Mutation persists across match arms
    #[test]
    #[ignore = "Property test - run with --ignored"]
    fn proptest_mutation_persists_across_arms() {
        for initial in 0..50 {
            for increment in 1..20 {
                let code = format!(
                    r"
let mut x = {initial};
match {increment} {{
    n => x = x + n
}}
println(x)
"
                );

                let output = Command::new("ruchy")
                    .arg("-e")
                    .arg(&code)
                    .output()
                    .expect("Failed to run ruchy");

                let stdout = String::from_utf8_lossy(&output.stdout);
                let result: i32 = stdout
                    .lines()
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(-1);

                assert_eq!(
                    result,
                    initial + increment,
                    "Mutation failed: {} + {} should equal {}",
                    initial,
                    increment,
                    initial + increment
                );
            }
        }
    }
}
