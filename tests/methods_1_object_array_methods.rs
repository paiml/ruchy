#![allow(missing_docs)]
//! METHODS-1: object `.keys()/.values()/.items()/.entries()`, array
//! `.sorted()/.reversed()/.drop(n)`, and the `assert_ne` builtin.
//!
//! `examples/` calls these method forms (`records[0].keys().join(",")`,
//! `word_freq.items()`, `intermediate.keys().sorted()`), but the interpreter
//! rejected them: a plain object literal failed with "Object is missing __type
//! marker" and the array forms were "Unknown array method".
//!
//! Key order: object methods return entries in sorted key order, the same
//! order `println` uses to display an object.

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Run `ruchy -e <code>` in a temp dir and assert it prints exactly `expected`.
fn assert_eval_stdout(code: &str, expected: &str) {
    let temp = tempfile::TempDir::new().expect("create temp dir");
    ruchy_cmd()
        .current_dir(temp.path())
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .success()
        .stdout(expected.to_string());
}

/// Run `ruchy -e <code>` in a temp dir and return the assertion for exit-code checks.
fn eval_assert(code: &str) -> assert_cmd::assert::Assert {
    let temp = tempfile::TempDir::new().expect("create temp dir");
    ruchy_cmd()
        .current_dir(temp.path())
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(10))
        .assert()
}

// ---------- object methods ----------

#[test]
fn test_methods_1_object_keys_sorted_order() {
    assert_eval_stdout(
        "let o = {b: 2, a: 1}; println(o.keys())",
        "[\"a\", \"b\"]\n",
    );
}

#[test]
fn test_methods_1_object_values_in_key_order() {
    assert_eval_stdout("let o = {b: 2, a: 1}; println(o.values())", "[1, 2]\n");
}

#[test]
fn test_methods_1_object_items_pairs() {
    assert_eval_stdout(
        "let o = {b: 2, a: 1}; println(o.items())",
        "[[\"a\", 1], [\"b\", 2]]\n",
    );
}

#[test]
fn test_methods_1_object_entries_same_as_items() {
    assert_eval_stdout(
        "let o = {b: 2, a: 1}; println(o.entries())",
        "[[\"a\", 1], [\"b\", 2]]\n",
    );
}

#[test]
fn test_methods_1_object_keys_empty() {
    assert_eval_stdout("let o = {}; println(o.keys())", "[]\n");
}

#[test]
fn test_methods_1_object_keys_join_like_example_17() {
    assert_eval_stdout(
        "let rec = {name: \"x\", age: 3}; println(rec.keys().join(\",\"))",
        "age,name\n",
    );
}

#[test]
fn test_methods_1_object_keys_len_like_example_31() {
    assert_eval_stdout("let e = {x: 1, y: 2, z: 3}; println(e.keys().len())", "3\n");
}

#[test]
fn test_methods_1_object_keys_for_loop_like_example_35() {
    assert_eval_stdout(
        "let o = {b: 2, a: 1}; for k in o.keys() { println(k) }",
        "a\nb\n",
    );
}

#[test]
fn test_methods_1_mutable_object_keys() {
    assert_eval_stdout(
        "let mut o = {a: 1}; o.b = 2; println(o.keys())",
        "[\"a\", \"b\"]\n",
    );
}

#[test]
fn test_methods_1_typed_object_keeps_type_dispatch() {
    eval_assert("let o = {__type: \"Foo\"}; println(o.keys())")
        .code(1)
        .stderr(predicate::str::contains("Unknown object type: Foo"));
}

// ---------- array methods ----------

#[test]
fn test_methods_1_array_sorted_numeric_order() {
    assert_eval_stdout("println([3, 10, 1].sorted())", "[1, 3, 10]\n");
}

#[test]
fn test_methods_1_array_sorted_leaves_source_unchanged() {
    assert_eval_stdout(
        "let a = [3, 1, 2]; let s = a.sorted(); println(s); println(a)",
        "[1, 2, 3]\n[3, 1, 2]\n",
    );
}

#[test]
fn test_methods_1_array_sorted_strings_and_empty() {
    assert_eval_stdout(
        "println([\"b\", \"c\", \"a\"].sorted()); println([].sorted())",
        "[\"a\", \"b\", \"c\"]\n[]\n",
    );
}

#[test]
fn test_methods_1_array_sort_numeric_order() {
    assert_eval_stdout("println([10, 2, 1].sort())", "[1, 2, 10]\n");
}

#[test]
fn test_methods_1_object_keys_sorted_like_example_38() {
    assert_eval_stdout(
        "let m = {zeta: 1, alpha: 2}; println(m.keys().sorted())",
        "[\"alpha\", \"zeta\"]\n",
    );
}

#[test]
fn test_methods_1_array_reversed() {
    assert_eval_stdout(
        "let a = [1, 2, 3]; println(a.reversed()); println(a)",
        "[3, 2, 1]\n[1, 2, 3]\n",
    );
}

#[test]
fn test_methods_1_array_drop() {
    assert_eval_stdout("println([1, 2, 3, 4].drop(2))", "[3, 4]\n");
}

#[test]
fn test_methods_1_array_drop_past_length_is_empty() {
    assert_eval_stdout("println([1, 2].drop(10))", "[]\n");
}

#[test]
fn test_methods_1_array_drop_zero_is_whole_array() {
    assert_eval_stdout("println([1, 2].drop(0))", "[1, 2]\n");
}

// ---------- assert_ne ----------

#[test]
fn test_methods_1_assert_ne_passes_on_different_values() {
    assert_eval_stdout("assert_ne(1, 2); println(\"after\")", "after\n");
}

#[test]
fn test_methods_1_assert_ne_fails_on_equal_values() {
    eval_assert("assert_ne(1, 1); println(\"after\")")
        .code(1)
        .stdout("")
        .stderr(predicate::str::contains(
            "Assertion failed: expected values to differ, both were Integer(1)",
        ));
}

#[test]
fn test_methods_1_assert_ne_custom_message() {
    eval_assert("assert_ne(\"a\", \"a\", \"must differ\")")
        .code(1)
        // Same rendering as `assert_eq`: the message value's Display, quotes included.
        .stderr(predicate::str::contains(
            "Assertion failed: \"must differ\"",
        ));
}
