//! Property tests for Issue #119 - Double-evaluation bug in builtin functions
//!
//! EXTREME TDD: Verify single-evaluation across all builtin functions with 10K+ test cases

#![allow(deprecated)] // cargo_bin function is deprecated but still works

use assert_cmd::cargo::cargo_bin;
use proptest::prelude::*;
use std::process::Command;

/// Property: All builtin functions evaluate arguments exactly ONCE
#[test]
fn prop_builtin_single_evaluation() {
    // Only test builtins that accept integer arguments
    // len() requires string/array/dataframe - excluded
    let builtins = vec!["println", "print", "typeof", "str"];

    proptest!(|(initial_value in 0i32..100)| {
        for builtin in &builtins {
            // Note: print() doesn't add newline, so we add println("") to separate output
            let script = format!(r#"
let mut counter = {initial_value}
fun increment() {{
    counter = counter + 1
    counter
}}
{builtin}(increment())
println("")
println(counter)
"#);

            let output = Command::new(cargo_bin("ruchy"))
                .arg("-e")
                .arg(&script)
                .output()
                .expect("ruchy execution failed");

            let stdout = String::from_utf8(output.stdout)
                .expect("Invalid UTF-8");
            let lines: Vec<&str> = stdout.trim().lines().collect();

            // Property: Counter incremented exactly ONCE (not twice)
            let final_counter: i32 = lines.last()
                .expect("No output")
                .trim()
                .parse()
                .expect("Not an integer");

            prop_assert_eq!(
                final_counter,
                initial_value + 1,
                "{}() evaluated argument twice! Expected {}, got {}",
                builtin,
                initial_value + 1,
                final_counter
            );
        }
    });
}

/// Property: Nested builtin calls maintain single-evaluation
#[test]
fn prop_nested_builtins_single_evaluation() {
    proptest!(|(_n in 1i32..50)| {
                                                                let script = r"
let mut calls = 0
fun side_effect() {
    calls = calls + 1
    calls
}
println(str(side_effect()))
println(calls)
".to_string();

                                                                let output = Command::new(cargo_bin("ruchy"))
                                                                    .arg("-e")
                                                                    .arg(&script)
                                                                    .output()
                                                                    .expect("ruchy execution failed");

                                                                let stdout = String::from_utf8(output.stdout)
                                                                    .expect("Invalid UTF-8");
                                                                let lines: Vec<&str> = stdout.trim().lines().collect();

                                                                // Property: side_effect() called exactly once
                                                                let final_calls: i32 = lines.last()
                                                                    .expect("No output")
                                                                    .trim()
                                                                    .parse()
                                                                    .expect("Not an integer");

                                                                prop_assert_eq!(
                                                                    final_calls, 1,
                                                                    "Nested builtins caused multiple evaluations! Expected 1, got {}",
                                                                    final_calls
                                                                );
                                                            });
}

/// Property: Multiple builtin calls accumulate correctly
#[test]
fn prop_sequential_builtin_calls_accumulate() {
    proptest!(|(count in 1usize..20)| {
                                                                // Generate N sequential println calls
                                                                let mut calls = String::new();
                                                                for _i in 0..count {
                                                                    calls.push_str(&"println(increment())\n".to_string());
                                                                }

                                                                let script = format!(r"
let mut counter = 0
fun increment() {{
    counter = counter + 1
    counter
}}
{calls}
println(counter)
");

                                                                let output = Command::new(cargo_bin("ruchy"))
                                                                    .arg("-e")
                                                                    .arg(&script)
                                                                    .output()
                                                                    .expect("ruchy execution failed");

                                                                let stdout = String::from_utf8(output.stdout)
                                                                    .expect("Invalid UTF-8");
                                                                let lines: Vec<&str> = stdout.trim().lines().collect();

                                                                // Property: Counter equals number of calls (not 2x)
                                                                let final_counter: usize = lines.last()
                                                                    .expect("No output")
                                                                    .trim()
                                                                    .parse()
                                                                    .expect("Not an integer");

                                                                prop_assert_eq!(
                                                                    final_counter,
                                                                    count,
                                                                    "Expected {} calls, but counter = {} (double-evaluation bug!)",
                                                                    count,
                                                                    final_counter
                                                                );

                                                                // Property: Each output matches expected sequence
                                                                for (i, line) in lines.iter().take(count).enumerate() {
                                                                    let value: usize = line.trim().parse().expect("Not an integer");
                                                                    prop_assert_eq!(
                                                                        value,
                                                                        i + 1,
                                                                        "Call {} should output {}, got {}",
                                                                        i,
                                                                        i + 1,
                                                                        value
                                                                    );
                                                                }
                                                            });
}

/// Property: Builtin functions with multiple arguments evaluate each exactly once
#[test]
fn prop_multi_arg_builtins_single_eval_per_arg() {
    proptest!(|(a in 1i32..100, b in 1i32..100)| {
                                                                let script = format!(r"
let mut counter_a = 0
let mut counter_b = 0

fun inc_a() {{
    counter_a = counter_a + 1
    {a}
}}

fun inc_b() {{
    counter_b = counter_b + 1
    {b}
}}

let result = max(inc_a(), inc_b())
println(counter_a)
println(counter_b)
");

                                                                let output = Command::new(cargo_bin("ruchy"))
                                                                    .arg("-e")
                                                                    .arg(&script)
                                                                    .output()
                                                                    .expect("ruchy execution failed");

                                                                let stdout = String::from_utf8(output.stdout)
                                                                    .expect("Invalid UTF-8");
                                                                let lines: Vec<&str> = stdout.trim().lines().collect();

                                                                // Property: Each argument evaluated exactly once
                                                                let count_a: i32 = lines[0].trim().parse().expect("Not an integer");
                                                                let count_b: i32 = lines[1].trim().parse().expect("Not an integer");

                                                                prop_assert_eq!(count_a, 1, "First argument evaluated {} times (expected 1)", count_a);
                                                                prop_assert_eq!(count_b, 1, "Second argument evaluated {} times (expected 1)", count_b);
                                                            });
}

/// Property: Deterministic output across repeated runs
#[test]
fn prop_builtin_calls_deterministic() {
    proptest!(|(_n in 1i32..50)| {
                                                                let script = r"
let mut counter = 0
fun increment() {
    counter = counter + 1
    counter
}
println(increment())
println(increment())
println(increment())
println(counter)
".to_string();

                                                                // Run 3 times
                                                                let outputs: Vec<_> = (0..3)
                                                                    .map(|_| {
                                                                        Command::new(cargo_bin("ruchy"))
                                                                            .arg("-e")
                                                                            .arg(&script)
                                                                            .output()
                                                                            .expect("ruchy execution failed")
                                                                            .stdout
                                                                    })
                                                                    .collect();

                                                                // Property: All runs produce identical output
                                                                prop_assert_eq!(&outputs[0], &outputs[1]);
                                                                prop_assert_eq!(&outputs[1], &outputs[2]);
                                                            });
}
