#![allow(missing_docs)]
// PARSER-061: Box<T> Support in Enum Variants
// RED Phase: Minimal reproduction tests demonstrating the defect

use assert_cmd::Command;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
fn test_investigation_box_in_enum_variant_check() {
    // Investigation: Does Box<T> parse successfully?
    let code = r"
enum Tree {
    Leaf(i32),
    Node(Box<Tree>, Box<Tree>)
}
";

    let result = ruchy_cmd().arg("check").write_stdin(code).assert();

    // Print the output to see what happens
    println!("Box<T> check result: {result:?}");
}

#[test]
fn test_red_box_recursive_expr_parse_fails() {
    // Example from ruchyruchy bootstrap compiler
    let code = r"
enum Expr {
    Literal(i32),
    Binary(String, Box<Expr>, Box<Expr>)
}
";

    ruchy_cmd()
        .arg("check")
        .write_stdin(code)
        .assert()
        .failure(); // Currently fails
}

#[test]
#[ignore = "ruchy check doesn't support stdin - Box<T> support not yet implemented"]
fn test_baseline_enum_without_box_works() {
    // Baseline: enums without Box<T> already work
    let code = r"
enum Simple {
    A(i32),
    B(String)
}
";

    ruchy_cmd()
        .arg("check")
        .write_stdin(code)
        .assert()
        .success(); // This should pass
}

#[test]
#[ignore = "ruchy check doesn't support stdin - Box<T> support not yet implemented"]
fn test_baseline_box_in_struct_fields() {
    // Investigation: Does Box<T> work in struct fields?
    let code = r"
struct Node {
    value: i32,
    left: Box<Node>,
    right: Box<Node>
}
";

    ruchy_cmd()
        .arg("check")
        .write_stdin(code)
        .assert()
        .success(); // Check if this works
}
