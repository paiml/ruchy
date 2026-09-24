//! OPTMETHODS-1: `Option`/`Result` methods in the interpreter.
//!
//! `Some(3).unwrap()` failed with "Method 'unwrap' not found for type
//! enum_variant" and `{"a": 1}.get("a")` with "Object is missing __type
//! marker" (the Option methods were lost in SPRINT-65).
//!
//! The interpreter keeps nil-for-absent library results (`pop`, `first`,
//! `get` return the element or nil), so these methods accept both forms:
//! `Some(v)`/`Ok(v)` and a plain non-nil value are present; `None`, `Err(e)`
//! and nil are absent. A plain value only reaches these methods for names
//! its own type does not already handle.

use assert_cmd::Command;
use std::path::Path;
use std::time::Duration;

fn ruchy() -> Command {
    let mut cmd = Command::cargo_bin("ruchy").expect("ruchy binary is built for tests");
    cmd.timeout(Duration::from_secs(30));
    cmd
}

fn write_source(dir: &Path, src: &str) -> std::path::PathBuf {
    let file = dir.join("prog.ruchy");
    std::fs::write(&file, src).expect("write ruchy source");
    file
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

fn interpret(dir: &Path, src: &str) -> String {
    let file = write_source(dir, src);
    let mut cmd = ruchy();
    cmd.arg("run").arg(file);
    stdout_of(cmd, "ruchy run")
}

fn run_file(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    interpret(dir.path(), src)
}

/// Run a program that must fail; return its stdout + stderr.
fn run_failing(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let mut cmd = ruchy();
    let out = cmd.arg("run").arg(file).output().expect("spawn ruchy");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.status.success(), "expected failure, got:\n{text}");
    text
}

fn main_of(body: &str) -> String {
    format!("fun main() {{\n{body}\n}}\n")
}

fn lines(values: &[&str]) -> String {
    values.iter().map(|v| format!("{v}\n")).collect()
}

fn transpile(dir: &Path, src: &str) -> String {
    let file = write_source(dir, src);
    let mut cmd = ruchy();
    cmd.arg("transpile").arg(file);
    stdout_of(cmd, "ruchy transpile")
}

fn rustc_run(dir: &Path, rust: &str) -> String {
    let main_rs = dir.join("main.rs");
    let bin = dir.join("prog_bin");
    std::fs::write(&main_rs, rust).expect("write main.rs");
    let mut rustc = Command::new("rustc");
    rustc
        .args(["--edition", "2021", "-A", "warnings", "-o"])
        .arg(&bin)
        .arg(&main_rs)
        .timeout(Duration::from_secs(120));
    stdout_of(rustc, &format!("rustc on transpiled Rust:\n{rust}\n"));
    let mut run = Command::new(&bin);
    run.timeout(Duration::from_secs(30));
    stdout_of(run, "transpiled binary")
}

/// Transpile, compile, run; assert interpreter == rustc output. Returns the output.
fn differential(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let rust = transpile(dir.path(), src);
    let expected = interpret(dir.path(), src);
    let actual = rustc_run(dir.path(), &rust);
    assert_eq!(
        actual, expected,
        "transpiled Rust disagrees with the interpreter\nsource:\n{src}\nrust:\n{rust}"
    );
    expected
}

#[test]
fn test_optmethods_1_01_some_methods() {
    let src = main_of(
        r#"    let s = Some(3)
    println(s.unwrap())
    println(s.expect("need a value"))
    println(s.unwrap_or(0))
    println(s.unwrap_or_else(|| 9))
    println(s.unwrap_or_default())
    println(s.is_some())
    println(s.is_none())
    println(s.map(|x| x * 2).unwrap())
    println(s.and_then(|x| Some(x + 1)).unwrap())
    println(s.or(Some(7)).unwrap())
    println(s.filter(|x| x > 1).is_some())
    println(s.filter(|x| x > 5).is_none())
    println(s.ok_or("missing").unwrap())"#,
    );
    assert_eq!(
        run_file(&src),
        lines(&["3", "3", "3", "3", "3", "true", "false", "6", "4", "3", "true", "true", "3"])
    );
}

#[test]
fn test_optmethods_1_02_none_methods() {
    let src = main_of(
        r#"    let n = None
    println(n.unwrap_or(5))
    println(n.unwrap_or_else(|| 9))
    println(n.is_some())
    println(n.is_none())
    println(n.map(|x| x * 2).is_none())
    println(n.and_then(|x| Some(x)).is_none())
    println(n.or(Some(7)).unwrap())
    println(n.filter(|x| x > 1).is_none())
    println(n.ok_or("missing").is_err())"#,
    );
    assert_eq!(
        run_file(&src),
        lines(&["5", "9", "false", "true", "true", "true", "7", "true", "true"])
    );
}

#[test]
fn test_optmethods_1_03_ok_methods() {
    let src = main_of(
        r#"    let r = Ok(2)
    println(r.unwrap())
    println(r.expect("need ok"))
    println(r.unwrap_or(0))
    println(r.unwrap_or_else(|e| 0))
    println(r.is_ok())
    println(r.is_err())
    println(r.map(|x| x + 1).unwrap())
    println(r.and_then(|x| Ok(x * 10)).unwrap())
    println(r.ok().unwrap())
    println(r.or(Ok(9)).unwrap())"#,
    );
    assert_eq!(
        run_file(&src),
        lines(&["2", "2", "2", "2", "true", "false", "3", "20", "2", "2"])
    );
}

#[test]
fn test_optmethods_1_04_err_methods() {
    let src = main_of(
        r#"    let r = Err("bad")
    println(r.unwrap_or(0))
    println(r.unwrap_or_else(|e| 1))
    println(r.is_ok())
    println(r.is_err())
    println(r.map(|x| x + 1).is_err())
    println(r.and_then(|x| Ok(x)).is_err())
    println(r.ok().is_none())
    println(r.or(Ok(9)).unwrap())"#,
    );
    assert_eq!(
        run_file(&src),
        lines(&["0", "1", "false", "true", "true", "true", "true", "9"])
    );
}

#[test]
fn test_optmethods_1_05_plain_value_is_present() {
    let src = main_of(
        r#"    let x = 5
    println(x.unwrap())
    println(x.expect("need x"))
    println(x.unwrap_or(0))
    println(x.is_some())
    println(x.is_none())
    println(x.and_then(|v| v + 1))
    println("hi".unwrap())
    let v = [1, 2, 3]
    println(v.first().unwrap())"#,
    );
    assert_eq!(
        run_file(&src),
        lines(&["5", "5", "5", "true", "false", "6", "hi", "1"])
    );
}

#[test]
fn test_optmethods_1_06_plain_value_methods_not_shadowed() {
    let src = main_of(
        r#"    let v = [1, 2, 3]
    println(v.map(|x| x * 2))
    println(v.filter(|x| x > 1))
    println("abc".len())
    println(7.abs())
    let m = {"a": 1}
    println(m.keys())"#,
    );
    assert_eq!(
        run_file(&src),
        lines(&["[2, 4, 6]", "[2, 3]", "3", "7", "[\"a\"]"])
    );
}

#[test]
fn test_optmethods_1_07_nil_is_absent() {
    let src = main_of(
        r#"    let mut e = [1]
    e.pop()
    let p = e.pop()
    println(p.is_none())
    println(p.is_some())
    println(p.unwrap_or(4))
    println(p.unwrap_or_else(|| 8))
    println(p.map(|x| x + 1).is_none())
    println(p.or(Some(2)).unwrap())"#,
    );
    assert_eq!(
        run_file(&src),
        lines(&["true", "false", "4", "8", "true", "2"])
    );
}

#[test]
fn test_optmethods_1_08_unwrap_on_absent_is_an_error_naming_the_value() {
    let none = run_failing(&main_of("    let n = None\n    println(n.unwrap())"));
    assert!(none.contains("unwrap") && none.contains("None"), "{none}");
    let err = run_failing(&main_of(
        "    let r = Err(\"boom\")\n    println(r.unwrap())",
    ));
    assert!(err.contains("unwrap") && err.contains("boom"), "{err}");
    let nil = run_failing(&main_of(
        "    let mut e = []\n    let p = e.pop()\n    println(p.unwrap())",
    ));
    assert!(nil.contains("unwrap") && nil.contains("nil"), "{nil}");
}

#[test]
fn test_optmethods_1_09_expect_on_absent_includes_the_message() {
    let out = run_failing(&main_of(
        "    let n = None\n    println(n.expect(\"value was required\"))",
    ));
    assert!(out.contains("value was required"), "{out}");
    let err = run_failing(&main_of(
        "    let r = Err(\"boom\")\n    println(r.expect(\"result was required\"))",
    ));
    assert!(
        err.contains("result was required") && err.contains("boom"),
        "{err}"
    );
}

#[test]
fn test_optmethods_1_10_unwrap_or_default_on_absent_is_an_error_not_nil() {
    let out = run_failing(&main_of(
        "    let n = None\n    println(n.unwrap_or_default())",
    ));
    assert!(
        out.contains("unwrap_or_default") && out.contains("None") && !out.contains("not found"),
        "{out}"
    );
}

const STACK: &str = "struct Stack { items: Vec<i32> }
impl Stack {
    fun pop_one(&mut self) -> i32 {
        self.items.pop().unwrap()
    }
}
";

#[test]
fn test_optmethods_1_11_field_pop_unwrap_in_method() {
    let src = format!(
        "{STACK}fun main() {{
    let mut s = Stack {{ items: vec![1, 2, 3] }}
    println(s.pop_one())
    println(s.pop_one())
}}
"
    );
    assert_eq!(run_file(&src), "3\n2\n");
}

#[test]
fn test_optmethods_1_12_map_get_on_plain_object() {
    let src = main_of(
        r#"    let m = {"a": 1, "b": 2}
    println(m.get("a"))
    println(m.get("a").unwrap())
    println(m.get("z").is_none())
    println(m.get("z").unwrap_or(0))
    println(m.values())"#,
    );
    assert_eq!(run_file(&src), lines(&["1", "1", "true", "0", "[1, 2]"]));
}

#[test]
fn test_optmethods_1_13_differential_pop_unwrap_through_rustc() {
    let src = format!(
        "{STACK}fun main() {{
    let mut s = Stack {{ items: vec![1, 2, 3] }}
    println(s.pop_one())
    println(s.pop_one())
    println(s.items.pop().unwrap_or(0))
    println(s.items.pop().unwrap_or(0))
    println(s.items.pop().is_none())
}}
"
    );
    assert_eq!(differential(&src), "3\n2\n1\n0\ntrue\n");
}
