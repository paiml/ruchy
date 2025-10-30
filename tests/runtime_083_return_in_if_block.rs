// RUNTIME-083: Test for Issue #66 - return statements in if blocks must terminate function
//
// ROOT CAUSE: Return statements inside if/else blocks don't properly terminate enclosing function
// SYMPTOM: Execution continues after return, causing incorrect behavior
// IMPACT: Blocks RuchyRuchy classification functions (4/8 tests failing)
//
// Five Whys:
// 1. Why do classification tests fail?
//    → Functions return wrong values despite explicit early returns
// 2. Why do they return wrong values?
//    → Execution continues after return statement in if block
// 3. Why does execution continue?
//    → Return statement doesn't properly terminate function scope
// 4. Why doesn't it terminate?
//    → Runtime evaluator doesn't handle early returns from nested blocks
// 5. ROOT CAUSE: Missing control flow termination for returns in conditional blocks
//
// NOTE: Tests use i32 return types due to PARSER-084 (missing &'static str support)

use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_runtime_083_return_in_if_terminates_function() {
    // Simple case: return in if block should exit function immediately
    let code = r"fun check_positive(x: i32) -> i32 {
    if x > 0 {
        return 1;
    }
    0
}

fun main() {
    println(check_positive(5));
}
";

    let temp_file = PathBuf::from("/tmp/test_runtime_083_return_if.ruchy");
    fs::write(&temp_file, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout("1\n");  // Should print 1, NOT 0

    let _ = fs::remove_file(&temp_file);
}

#[test]
fn test_runtime_083_return_in_else_terminates_function() {
    // Return in else block should also exit function immediately
    let code = r"fun check_negative(x: i32) -> i32 {
    if x > 0 {
        1
    } else {
        return -1;
    }
}

fun main() {
    println(check_negative(-5));
}
";

    let temp_file = PathBuf::from("/tmp/test_runtime_083_return_else.ruchy");
    fs::write(&temp_file, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout("-1\n");

    let _ = fs::remove_file(&temp_file);
}

#[test]
fn test_runtime_083_early_return_prevents_subsequent_code() {
    // Code after if block with return should NOT execute
    let code = r"fun classify(x: i32) -> i32 {
    if x < 0 {
        return -1;
    }
    if x == 0 {
        return 0;
    }
    1
}

fun main() {
    println(classify(-5));
    println(classify(0));
    println(classify(5));
}
";

    let temp_file = PathBuf::from("/tmp/test_runtime_083_early_return.ruchy");
    fs::write(&temp_file, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout("-1\n0\n1\n");

    let _ = fs::remove_file(&temp_file);
}

#[test]
fn test_runtime_083_nested_if_return() {
    // Return in nested if block should exit outer function
    let code = r"fun check_range(x: i32) -> i32 {
    if x > 0 {
        if x < 10 {
            return 1;
        }
        return 2;
    }
    0
}

fun main() {
    println(check_range(5));
    println(check_range(15));
    println(check_range(-5));
}
";

    let temp_file = PathBuf::from("/tmp/test_runtime_083_nested_if.ruchy");
    fs::write(&temp_file, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout("1\n2\n0\n");

    let _ = fs::remove_file(&temp_file);
}

#[test]
fn test_runtime_083_return_value_not_overwritten() {
    // Return value should be preserved, not overwritten by later code
    let code = r"fun get_status(x: i32) -> i32 {
    if x > 100 {
        return 1;
    }
    0
}

fun main() {
    let result = get_status(150);
    println(result);
}
";

    let temp_file = PathBuf::from("/tmp/test_runtime_083_value_preserved.ruchy");
    fs::write(&temp_file, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout("1\n");  // Should be 1, not 0

    let _ = fs::remove_file(&temp_file);
}

#[test]
fn test_runtime_083_multiple_returns_first_wins() {
    // First matching return should execute, subsequent returns ignored
    let code = r"fun priority_check(x: i32) -> i32 {
    if x < 0 {
        return 1;
    }
    if x < 10 {
        return 2;
    }
    if x < 100 {
        return 3;
    }
    4
}

fun main() {
    println(priority_check(-5));
    println(priority_check(5));
    println(priority_check(50));
    println(priority_check(150));
}
";

    let temp_file = PathBuf::from("/tmp/test_runtime_083_multiple_returns.ruchy");
    fs::write(&temp_file, code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout("1\n2\n3\n4\n");

    let _ = fs::remove_file(&temp_file);
}
