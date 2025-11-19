#![allow(missing_docs)]
// PARSER-061 & PARSER-080: Box<T> and Vec<T> Generics in Enum Variants
//
// Test Strategy:
// 1. Verify Box<T> works in enum variants (recursive data structures)
// 2. Verify Vec<T> works in enum variants (collection-based structures)
// 3. Test parser acceptance (ruchy check)
// 4. Test transpiler correctness (ruchy transpile)
// 5. Test interpreter correctness (ruchy -e)
// 6. Test nested/complex scenarios
//
// Requirements from docs/execution/roadmap.yaml:
// - PARSER-061: Box<T> support (v3.96.0 - already implemented)
// - PARSER-080: Vec<T> support (v3.96.0 - already implemented)
// - Tests document that both features work correctly

use assert_cmd::Command as AssertCommand;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Test Suite 1: Box<T> in Enum Variants (8 tests)
// ============================================================================

#[test]
fn test_parser_061_01_box_simple_enum_check() {
    // Parser accepts Box<T> syntax in enum variants
    let code = r"
enum Expr {
    Literal(i32),
    Binary(String, Box<Expr>, Box<Expr>)
}
";

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.ruchy");
    fs::write(&file_path, code).unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("check")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Syntax is valid"));
}

#[test]
fn test_parser_061_02_box_transpile_correctness() {
    // Transpiler generates correct Rust code with Box<T>
    let code = r"
enum Expr {
    Num(i32),
    Add(Box<Expr>, Box<Expr>)
}
let x = Expr::Num(42)
";

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.ruchy");
    fs::write(&file_path, code).unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Box<Expr>"))
        .stdout(predicate::str::contains("enum Expr"));
}

#[test]
fn test_parser_061_03_box_instantiation_literal() {
    // Can instantiate enum with Box<T> - simple case
    let code = r"
enum Expr {
    Lit(i32),
    Add(Box<Expr>, Box<Expr>)
}
let x = Expr::Lit(42)
println(x)
";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Lit(42)"));
}

#[test]
fn test_parser_061_04_box_instantiation_recursive() {
    // Can instantiate enum with Box<T> - recursive case
    let code = r"
enum Expr {
    Num(i32),
    Add(Box<Expr>, Box<Expr>)
}
let left = Expr::Num(1)
let right = Expr::Num(2)
let add = Expr::Add(Box::new(left), Box::new(right))
println(add)
";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Add"));
}

#[test]
fn test_parser_061_05_box_nested_three_levels() {
    // Box<T> works with deep nesting (3 levels)
    let code = r"
enum Tree {
    Leaf(i32),
    Node(Box<Tree>, Box<Tree>)
}
let a = Tree::Leaf(1)
let b = Tree::Leaf(2)
let c = Tree::Node(Box::new(a), Box::new(b))
let d = Tree::Leaf(3)
let root = Tree::Node(Box::new(c), Box::new(d))
println(root)
";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Node"));
}

#[test]
fn test_parser_061_06_box_multiple_type_params() {
    // Box<T> with different type parameters in same enum
    let code = r"
enum Value {
    Int(Box<i32>),
    Str(Box<String>),
    Bool(Box<bool>)
}
let v = Value::Int(Box::new(42))
println(v)
";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Int"));
}

#[test]
fn test_parser_061_07_box_with_unary_operator() {
    // Box<T> with unary operator enum (from ruchyruchy BOOTSTRAP-006)
    let code = r#"
enum Expr {
    Number(String),
    Unary(UnOp, Box<Expr>)
}

enum UnOp {
    Neg,
    Not
}

let num = Expr::Number("42")
let neg = Expr::Unary(UnOp::Neg, Box::new(num))
println(neg)
"#;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Unary"));
}

#[test]
fn test_parser_061_08_box_ast_from_bootstrap() {
    // Full recursive AST from ruchyruchy BOOTSTRAP-006
    let code = r#"
enum Expr {
    Number(String),
    Identifier(String),
    BoolTrue,
    BoolFalse,
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Group(Box<Expr>)
}

enum BinOp {
    Add, Sub, Mul, Div
}

enum UnOp {
    Neg, Not
}

let left = Expr::Number("1")
let right = Expr::Number("2")
let add = Expr::Binary(BinOp::Add, Box::new(left), Box::new(right))
println(add)
"#;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Binary"));
}

// ============================================================================
// Test Suite 2: Vec<T> in Enum Variants (7 tests)
// ============================================================================

#[test]
fn test_parser_080_01_vec_simple_enum_check() {
    // Parser accepts Vec<T> syntax in enum variants
    let code = r"
enum Statement {
    Block(Vec<Statement>),
    Expr(i32)
}
";

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.ruchy");
    fs::write(&file_path, code).unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("check")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Syntax is valid"));
}

#[test]
fn test_parser_080_02_vec_transpile_correctness() {
    // Transpiler generates correct Rust code with Vec<T>
    let code = r"
enum Stmt {
    Block(Vec<Stmt>),
    Val(i32)
}
let x = Stmt::Val(42)
";

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.ruchy");
    fs::write(&file_path, code).unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Vec<Stmt>"))
        .stdout(predicate::str::contains("enum Stmt"));
}

#[test]
fn test_parser_080_03_vec_empty_instantiation() {
    // Can instantiate enum with Vec<T> - empty case (using Vec::new())
    let code = r"
enum Stmt {
    Block(Vec<Stmt>),
    Val(i32)
}
let v: Vec<Stmt> = Vec::new()
let empty = Stmt::Block(v)
println(empty)
";

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.ruchy");
    fs::write(&file_path, code).unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("run")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Block"));
}

#[test]
fn test_parser_080_04_vec_with_elements() {
    // Can instantiate enum with Vec<T> - with elements (using push)
    let code = r"
enum Stmt {
    Block(Vec<Stmt>),
    Expr(i32)
}
let s1 = Stmt::Expr(1)
let s2 = Stmt::Expr(2)
let mut v: Vec<Stmt> = Vec::new()
v.push(s1)
v.push(s2)
let block = Stmt::Block(v)
println(block)
";

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.ruchy");
    fs::write(&file_path, code).unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("run")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Block"));
}

#[test]
fn test_parser_080_05_vec_nested_blocks() {
    // Vec<T> with nested blocks (2 levels)
    let code = r"
enum Stmt {
    Block(Vec<Stmt>),
    Val(i32)
}
let mut inner_v: Vec<Stmt> = Vec::new()
inner_v.push(Stmt::Val(1))
inner_v.push(Stmt::Val(2))
let inner = Stmt::Block(inner_v)
let mut outer_v: Vec<Stmt> = Vec::new()
outer_v.push(inner)
outer_v.push(Stmt::Val(3))
let outer = Stmt::Block(outer_v)
println(outer)
";

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.ruchy");
    fs::write(&file_path, code).unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("run")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Block"));
}

#[test]
fn test_parser_080_06_vec_with_string_type() {
    // Vec<T> with different type parameter
    let code = r#"
enum Data {
    List(Vec<String>),
    Single(String)
}
let mut v: Vec<String> = Vec::new()
v.push("hello")
v.push("world")
let d = Data::List(v)
println(d)
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.ruchy");
    fs::write(&file_path, code).unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("run")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("List"));
}

#[test]
fn test_parser_080_07_vec_function_params() {
    // Vec<T> for function parameter lists (bootstrap use case)
    let code = r#"
enum Param {
    Named(String, String)
}

enum Function {
    Def(String, Vec<Param>)
}

let mut params: Vec<Param> = Vec::new()
params.push(Param::Named("x", "i32"))
params.push(Param::Named("y", "i32"))
let func = Function::Def("add", params)
println(func)
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.ruchy");
    fs::write(&file_path, code).unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("run")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Def"));
}

// ============================================================================
// Test Suite 3: Combined Box<T> and Vec<T> (3 tests)
// ============================================================================

#[test]
fn test_parser_061_080_01_box_and_vec_same_enum() {
    // Enum with both Box<T> and Vec<T> variants
    let code = r"
enum Node {
    Single(Box<Node>),
    Multiple(Vec<Node>),
    Leaf(i32)
}
let leaf = Node::Leaf(42)
let single = Node::Single(Box::new(leaf))
println(single)
";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Single"));
}

#[test]
fn test_parser_061_080_02_vec_of_box() {
    // Vec<Box<T>> - combination of both
    let code = r#"
enum Expr {
    Call(String, Vec<Box<Expr>>),
    Lit(i32)
}
let arg1 = Box::new(Expr::Lit(1))
let arg2 = Box::new(Expr::Lit(2))
let mut args: Vec<Box<Expr>> = Vec::new()
args.push(arg1)
args.push(arg2)
let call = Expr::Call("add", args)
println(call)
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.ruchy");
    fs::write(&file_path, code).unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("run")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Call"));
}

#[test]
fn test_parser_061_080_03_complex_ast() {
    // Complex AST with Box<T>, Vec<T>, and multiple enums
    let code = r#"
enum Type {
    Int,
    Str,
    Func(Vec<Box<Type>>, Box<Type>)
}

enum Expr {
    Var(String),
    Lambda(Vec<String>, Box<Expr>),
    App(Box<Expr>, Vec<Box<Expr>>)
}

let int_type = Type::Int
let mut param_types: Vec<Box<Type>> = Vec::new()
param_types.push(Box::new(Type::Int))
let func_type = Type::Func(param_types, Box::new(Type::Str))
let var = Expr::Var("x")
println(func_type)
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.ruchy");
    fs::write(&file_path, code).unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("run")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Func"));
}

// Total: 18 comprehensive tests
// - Suite 1: 8 Box<T> tests (simple → complex → bootstrap AST)
// - Suite 2: 7 Vec<T> tests (empty → nested → function params)
// - Suite 3: 3 combined tests (Box + Vec in same enum, Vec<Box<T>>, complex AST)
//
// All tests verify:
// - Parser acceptance (check)
// - Transpiler correctness (transpile)
// - Interpreter correctness (-e)
// - Matches ruchyruchy BOOTSTRAP-006/007 use cases
