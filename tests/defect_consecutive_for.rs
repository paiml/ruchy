// DEFECT-CONSECUTIVE-FOR: Parser fails on consecutive for loops
// ROOT CAUSE: Parser doesn't properly handle multiple for loops in sequence
// FIX: Parser must allow consecutive loop statements
// EXTREME TDD: RED→GREEN→REFACTOR

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper to get ruchy binary command
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// RED: This test MUST fail before the fix
#[test]
fn test_defect_consecutive_for_loops() {
    let temp_file = std::env::temp_dir().join("defect_consecutive_for.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn main() {
    for i in 0..3 {
        println(i)
    }

    for n in 5..7 {
        println(n)
    }
}
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("5"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_defect_three_consecutive_for_loops() {
    let temp_file = std::env::temp_dir().join("defect_three_for.ruchy");
    std::fs::write(
        &temp_file,
        r#"
fn main() {
    for i in 0..2 {
        println(i)
    }

    for j in 10..12 {
        println(j)
    }

    for k in 20..22 {
        println(k)
    }
}
"#,
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("10"))
        .stdout(predicate::str::contains("20"));

    std::fs::remove_file(&temp_file).ok();
}
