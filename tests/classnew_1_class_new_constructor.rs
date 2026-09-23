//! CLASSNEW-1: a class `new(...)` constructor (and a named `new name(...)`
//! constructor) binds `self` to a fresh instance, runs the body, and returns
//! the instance; `Class::new(args)` calls it. The book form
//! (ruchy-book ch19 "Classes") used to fail with
//! `Runtime error: Undefined variable: self` because the interpreter read
//! `self` back after it had already popped the constructor's scope.

use assert_cmd::Command;
use std::path::Path;
use std::time::Duration;

const BOOK_CALCULATOR: &str = r"class Calculator {
    value: i32
    new(initial: i32) {
        self.value = initial
    }
    pub fun add(&mut self, amount: i32) { self.value = self.value + amount }
    pub fun get(&self) -> i32 { self.value }
}
let mut calc = Calculator::new(10)
calc.add(5)
println(calc.get())
";

fn ruchy() -> Command {
    let mut cmd = Command::cargo_bin("ruchy").expect("ruchy binary is built for tests");
    cmd.timeout(Duration::from_secs(30));
    cmd
}

fn stdout_of(mut cmd: Command, what: &str) -> String {
    let out = cmd.output().expect("spawn command");
    assert!(
        out.status.success(),
        "{what} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

fn write_source(dir: &Path, src: &str) -> std::path::PathBuf {
    let file = dir.join("prog.ruchy");
    std::fs::write(&file, src).expect("write ruchy source");
    file
}

fn run_file(src: &str) -> String {
    let dir = tempfile::tempdir().expect("temp dir");
    let file = write_source(dir.path(), src);
    let mut cmd = ruchy();
    cmd.arg("run").arg(file);
    stdout_of(cmd, "ruchy run")
}

fn eval_inline(src: &str) -> String {
    let mut cmd = ruchy();
    cmd.arg("-e").arg(src);
    stdout_of(cmd, "ruchy -e")
}

#[test]
fn test_classnew_1_01_book_calculator_run_prints_15() {
    assert_eq!(run_file(BOOK_CALCULATOR), "15\n");
}

#[test]
fn test_classnew_1_02_book_calculator_eval_prints_15() {
    assert_eq!(eval_inline(BOOK_CALCULATOR), "15\n");
}

#[test]
fn test_classnew_1_03_constructor_sets_two_fields() {
    let src = r"class Point {
    x: i32
    y: i32
    new(x: i32, y: i32) {
        self.x = x
        self.y = y
    }
}
let p = Point::new(3, 4)
println(p.x)
println(p.y)
";
    assert_eq!(run_file(src), "3\n4\n");
}

#[test]
fn test_classnew_1_04_declared_field_defaults_survive_constructor() {
    let src = r"class Counter {
    count: i32 = 7
    step: i32 = 2
    new(step: i32) {
        self.step = step
    }
    fun bump(&mut self) { self.count = self.count + self.step }
}
let mut c = Counter::new(3)
println(c.count)
c.bump()
println(c.count)
";
    assert_eq!(run_file(src), "7\n10\n");
}

#[test]
fn test_classnew_1_05_named_constructor() {
    let src = r"class Rect {
    w: i32
    h: i32
    new(w: i32, h: i32) {
        self.w = w
        self.h = h
    }
    new square(size: i32) {
        self.w = size
        self.h = size
    }
    fun area(&self) -> i32 { self.w * self.h }
}
let s = Rect::square(4)
let r = Rect::new(2, 5)
println(s.area())
println(r.area())
";
    assert_eq!(run_file(src), "16\n10\n");
}

#[test]
fn test_classnew_1_06_init_form_still_works() {
    let src = r"class Point {
    x: i32
    y: i32
    init(x: i32, y: i32) {
        self.x = x
        self.y = y
    }
}
let p = Point(10, 20)
println(p.x)
println(p.y)
";
    assert_eq!(run_file(src), "10\n20\n");
}

#[test]
fn test_classnew_1_07_mutating_method_updates_instance() {
    let src = r"class Tally {
    n: i32
    new() {
        self.n = 0
    }
    fun inc(&mut self) { self.n = self.n + 1 }
}
let mut t = Tally::new()
t.inc()
t.inc()
t.inc()
println(t.n)
";
    assert_eq!(run_file(src), "3\n");
}

#[test]
fn test_classnew_1_08_constructor_body_reads_self() {
    let src = r"class Acc {
    total: i32
    new(a: i32, b: i32) {
        self.total = a
        self.total = self.total + b
    }
}
println(Acc::new(4, 5).total)
";
    assert_eq!(run_file(src), "9\n");
}

#[test]
fn test_classnew_1_09_transpiled_book_calculator_prints_15() {
    let dir = tempfile::tempdir().expect("temp dir");
    let file = write_source(dir.path(), BOOK_CALCULATOR);
    let mut transpile = ruchy();
    transpile.arg("transpile").arg(&file);
    let rust = stdout_of(transpile, "ruchy transpile");
    let main_rs = dir.path().join("main.rs");
    let bin = dir.path().join("prog_bin");
    std::fs::write(&main_rs, &rust).expect("write main.rs");
    let mut rustc = Command::new("rustc");
    rustc
        .args(["--edition", "2021", "-A", "warnings", "-o"])
        .arg(&bin)
        .arg(&main_rs)
        .timeout(Duration::from_secs(120));
    stdout_of(rustc, &format!("rustc on transpiled Rust:\n{rust}\n"));
    let mut run = Command::new(&bin);
    run.timeout(Duration::from_secs(30));
    assert_eq!(stdout_of(run, "transpiled binary"), "15\n");
}

#[test]
fn test_classnew_1_10_new_instance_has_reference_semantics() {
    let src = r"class Cell {
    v: i32
    new(v: i32) { self.v = v }
    fun set(&mut self, n: i32) { self.v = n }
}
let a = Cell::new(1)
let b = a
b.set(9)
println(a.v)
";
    assert_eq!(run_file(src), "9\n");
}

#[test]
fn test_classnew_1_11_constructor_error_is_reported() {
    let dir = tempfile::tempdir().expect("temp dir");
    let file = write_source(
        dir.path(),
        "class E {\n    v: i32\n    new(v: i32) { self.v = missing_name }\n}\nlet e = E::new(1)\n",
    );
    let mut cmd = ruchy();
    cmd.arg("run").arg(file);
    let out = cmd.output().expect("spawn command");
    assert!(!out.status.success(), "constructor error must fail the run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Undefined variable: missing_name"),
        "stderr: {stderr}"
    );
}
