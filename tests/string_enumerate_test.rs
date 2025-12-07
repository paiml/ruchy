#![allow(missing_docs)]
//! RED TEST for Issue #40: Missing .`enumerate()` causes O(n²) code
//!
//! ROOT CAUSE: No way to iterate with index, forcing users to use
//! inefficient pattern: loop { chars().nth(i); i += 1 }
//!
//! SOLUTION: Implement .`enumerate()` that returns (index, value) tuples

use assert_cmd::Command;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
#[ignore = "enumerate not implemented"]
fn test_string_chars_enumerate_basic() {
    let code = r#"
let s = "hello".to_string();
let mut count = 0;

for item in s.chars().enumerate() {
    count = count + 1;
}

println(count)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("5\nnil\n");
}

#[test]
#[ignore = "enumerate not implemented"]
fn test_array_enumerate() {
    let code = r"
let arr = [10, 20, 30];
for item in arr.enumerate() {
    println(item)
}
";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("(0, 10)\n(1, 20)\n(2, 30)\nnil\n");
}

#[test]
#[ignore = "enumerate not implemented"]
fn test_enumerate_replaces_on_squared_pattern() {
    // This is the CORRECT way to iterate with index - O(n) not O(n²)
    let code = r#"
fun count_chars_efficient(input: String) -> i32 {
    let mut count = 0;

    for item in input.chars().enumerate() {
        count = count + 1;
    }

    count
}

let result = count_chars_efficient("hello".to_string());
println(result)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout("5\nnil\n");
}
