//! TRANSPILER-155 (GitHub #155): the issue's matrix benchmark failed to compile.
//!
//! - A cast of a compound operand lost its parentheses: `(idx % 100) as f64`
//!   was emitted as `idx % 100 as f64`, which casts only `100`.
//! - `use std::time::Instant` was replaced by the ruchy time helper module,
//!   so `Instant` was not in scope.
//!
//! Where the interpreter can run the program, compiled output must match it.

use assert_cmd::Command;
use std::path::Path;
use std::time::Duration;

fn ruchy() -> Command {
    let mut cmd = Command::cargo_bin("ruchy").expect("ruchy binary is built for tests");
    cmd.timeout(Duration::from_secs(10));
    cmd
}

fn write_source(dir: &Path, src: &str) -> std::path::PathBuf {
    let file = dir.join("prog.ruchy");
    std::fs::write(&file, src).expect("write ruchy source");
    file
}

/// stdout of a command that must succeed.
fn stdout_of(mut cmd: Command, what: &str) -> String {
    let out = cmd.output().expect("spawn command");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    assert!(out.status.success(), "{what} failed:\n{stdout}{stderr}");
    stdout
}

fn transpile(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let mut cmd = ruchy();
    cmd.arg("transpile").arg(file);
    stdout_of(cmd, &format!("ruchy transpile of:\n{src}\n"))
}

fn compile_and_run(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let bin = dir.path().join("prog_bin");
    let mut cmd = ruchy();
    cmd.arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(&bin)
        .timeout(Duration::from_secs(90));
    stdout_of(cmd, &format!("ruchy compile of:\n{src}\n"));
    let mut run = Command::new(&bin);
    run.timeout(Duration::from_secs(30));
    stdout_of(run, "compiled binary")
}

fn interpret(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let mut cmd = ruchy();
    cmd.arg("run").arg(file);
    stdout_of(cmd, &format!("ruchy run of:\n{src}\n"))
}

fn main_with(body: &str) -> String {
    format!("fun main() {{\n{body}\n}}\n")
}

/// Compiled and interpreted output agree, and equal `expected`.
fn assert_agree(src: &str, expected: &str) {
    let compiled = compile_and_run(src);
    assert_eq!(compiled.trim(), expected, "compiled output of:\n{src}");
    assert_eq!(
        interpret(src).trim(),
        expected,
        "interpreted output of:\n{src}"
    );
}

#[test]
fn test_transpiler_155_01_cast_of_remainder_keeps_parens() {
    let src = main_with("  let idx = 7\n  let a = (idx % 5) as f64 / 4.0\n  println!(\"{}\", a)");
    assert_agree(&src, "0.5");
}

#[test]
fn test_transpiler_155_02_cast_of_nested_binary_keeps_parens() {
    let src =
        main_with("  let idx = 7\n  let b = ((idx * 2) % 5) as f64 / 8.0\n  println!(\"{}\", b)");
    assert_agree(&src, "0.5");
}

#[test]
fn test_transpiler_155_03_cast_then_divide() {
    let src = main_with("  let idx = 8\n  let c = (idx + 1) as f64 / 2.0\n  println!(\"{}\", c)");
    assert_agree(&src, "4.5");
}

#[test]
fn test_transpiler_155_04_cast_of_method_call() {
    let src =
        main_with("  let v = [1, 2, 3]\n  let n = v.len() as f64 / 2.0\n  println!(\"{}\", n)");
    assert_agree(&src, "1.5");
}

const ATOMIC_OPERANDS_PROGRAM: &str = r#"struct P { x: i32 }
fun f() -> i32 { 3 }
fun main() {
  let x = 3
  let v = [1, 2, 3]
  let p = P { x: 4 }
  let a = x as f64
  let b = 5 as f64
  let c = f() as f64
  let d = v.len() as f64
  let e = p.x as f64
  let g = v[0] as f64
  let h = x as i64 as f64
  println!("{} {} {} {} {} {} {}", a, b, c, d, e, g, h)
}
"#;

#[test]
fn test_transpiler_155_05_atomic_operands_get_no_added_parens() {
    let rust = transpile(ATOMIC_OPERANDS_PROGRAM);
    // One pair of parentheses wraps each whole cast; none is added around
    // the operand (identifier, literal, call, method call, field, index, cast).
    for cast in [
        "(x as f64)",
        "(5 as f64)",
        "(f() as f64)",
        "(v.len() as f64)",
        "(p.x as f64)",
        "(v[0 as usize].clone() as f64)",
        "((x as i64) as f64)",
    ] {
        assert!(rust.contains(cast), "expected `{cast}` in:\n{rust}");
    }
    for wrapped in ["((x) as f64)", "((5) as f64)", "((f()) as f64)", "((((x"] {
        assert!(
            !rust.contains(wrapped),
            "operand wrapped: `{wrapped}` in:\n{rust}"
        );
    }
    assert_eq!(
        compile_and_run(ATOMIC_OPERANDS_PROGRAM).trim(),
        "3 5 3 3 4 1 3"
    );
}

#[test]
fn test_transpiler_155_06_use_std_time_instant_is_emitted() {
    let src = "use std::time::Instant;\nfun main() {\n  let t0 = Instant::now()\n  println!(\"{}\", t0.elapsed().as_nanos() >= 0)\n}\n";
    let rust = transpile(src);
    assert!(
        rust.contains("use std::time::Instant;"),
        "use dropped in:\n{rust}"
    );
    assert_eq!(compile_and_run(src).trim(), "true");
}

#[test]
fn test_transpiler_155_07_use_std_time_group_is_emitted() {
    let src = "use std::time::{Instant, Duration};\nfun main() {\n  let _t = Instant::now()\n  let d = Duration::from_millis(5)\n  println!(\"{}\", d.as_millis())\n}\n";
    assert_eq!(compile_and_run(src).trim(), "5");
}

#[test]
fn test_transpiler_155_08_ruchy_time_module_import_keeps_helpers() {
    let rust = transpile("use std::time;\nfun main() {\n  println!(\"{}\", 1)\n}\n");
    assert!(
        rust.contains("fn now_millis"),
        "time helper module dropped in:\n{rust}"
    );
}

#[test]
fn test_transpiler_155_10_use_std_time_system_time_is_emitted() {
    let src = "use std::time::{SystemTime, UNIX_EPOCH};\nfun main() {\n  let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()\n  println!(\"{}\", secs > 1000000000)\n}\n";
    assert_eq!(compile_and_run(src).trim(), "true");
}

/// The naive checksum the issue's benchmark prints for n = 128.
fn issue_155_expected_checksum() -> i64 {
    let n = 128usize;
    let cell = |i: usize, j: usize, k: usize| ((i * n + j) * k % 100) as f64;
    let mut sum = 0.0f64;
    for i in 0..n {
        for j in 0..n {
            let mut acc = 0.0f64;
            for k in 0..n {
                acc += cell(i, k, 1) * cell(k, j, 2);
            }
            sum += acc;
        }
    }
    sum as i64
}

const ISSUE_155_PROGRAM: &str = r#"use std::time::Instant;

fun matmul(a: Vec<Vec<f64>>, b: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let n = a.len();
    let mut c = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            let mut sum = 0.0;
            for k in 0..n {
                sum += a[i][k] * b[k][j];
            }
            c[i][j] = sum;
        }
    }
    c
}

fun main() {
    let t0 = Instant::now();
    let size = 128;
    let mut a = vec![vec![0.0; size]; size];
    let mut b = vec![vec![0.0; size]; size];
    for i in 0..size {
        for j in 0..size {
            let idx = i * size + j;
            a[i][j] = (idx % 100) as f64;
            b[i][j] = ((idx * 2) % 100) as f64;
        }
    }
    let c = matmul(a, b);
    let elapsed = t0.elapsed();
    let mut sum = 0.0;
    for i in 0..size {
        for j in 0..size {
            sum += c[i][j];
        }
    }
    println!("TIMED: {}", elapsed.as_nanos() > 0);
    println!("RESULT: {}", sum as i64);
}
"#;

#[test]
fn test_transpiler_155_09_issue_matrix_benchmark_compiles_and_is_correct() {
    let out = compile_and_run(ISSUE_155_PROGRAM);
    let expected = format!("TIMED: true\nRESULT: {}", issue_155_expected_checksum());
    assert_eq!(out.trim(), expected);
}
