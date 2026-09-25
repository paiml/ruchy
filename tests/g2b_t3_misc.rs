//! G2B-T10/T12 (RHLGA-1): range aggregations and Rust-compatible `{:?}`.
//!
//! T10: `count`/`sum`/`min`/`max` on a range receiver (`range(a, b)` or
//! `a..b`) are called on the range itself; a range has no `.iter()`.
//! T12: the interpreter renders `{:?}` exactly as rustc renders the
//! transpiled value, in `println!`, `format!` and the `println(..)` function.
//! Each program is transpiled, compiled with rustc and run; its output is
//! compared with `ruchy run` where the interpreter accepts the program.

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::{Parser, Transpiler};
use std::process::Command;

/// A scratch directory for one program, removed when the returned guard drops.
fn scratch_dir(name: &str) -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix(&format!("g2b_t3_{name}_"))
        .tempdir()
        .expect("scratch dir")
}

fn transpile(code: &str) -> String {
    let ast = Parser::new(code).parse().expect("parses");
    Transpiler::new()
        .transpile_to_program(&ast)
        .expect("transpiles")
        .to_string()
}

fn compiled_output(code: &str, name: &str) -> String {
    let rust = transpile(code);
    let dir = scratch_dir(name);
    let src = dir.path().join("main.rs");
    let bin = dir.path().join("main");
    std::fs::write(&src, &rust).expect("write rust");
    let out = Command::new("rustc")
        .args(["--edition", "2021", "-A", "warnings", "-o"])
        .arg(&bin)
        .arg(&src)
        .output()
        .expect("rustc runs");
    assert!(
        out.status.success(),
        "rustc rejected the transpiled program:\n{}\n--- rust ---\n{rust}",
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&bin).output().expect("binary runs");
    assert!(run.status.success(), "binary failed: {run:?}");
    String::from_utf8_lossy(&run.stdout).into_owned()
}

fn interpreted_output(code: &str, name: &str) -> String {
    let dir = scratch_dir(name);
    let file = dir.path().join("prog.ruchy");
    std::fs::write(&file, code).expect("write ruchy");
    let out = Command::new(env!("CARGO_BIN_EXE_ruchy"))
        .arg("run")
        .arg(&file)
        .output()
        .expect("ruchy runs");
    assert!(out.status.success(), "ruchy run failed: {out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn assert_same_output(code: &str, name: &str, expected: &str) {
    let compiled = compiled_output(code, name);
    let interpreted = interpreted_output(code, name);
    assert_eq!(compiled, expected, "rustc output for:\n{code}");
    assert_eq!(interpreted, compiled, "interpreter vs rustc for:\n{code}");
}

// ---------------------------------------------------------------- T10 ranges

#[test]
fn test_g2bt10_range_call_count_compiles_and_counts() {
    let code = "fun main() {\n    let count = range(0, 10).count()\n    println(count)\n}\n";
    assert_eq!(compiled_output(code, "t10_count"), "10\n");
}

#[test]
fn test_g2bt10_range_call_sum_matches_interpreter() {
    let code = "fun main() {\n    let total = range(0, 10).sum()\n    println(total)\n}\n";
    assert_same_output(code, "t10_sum", "45\n");
}

#[test]
fn test_g2bt10_range_literal_count_and_sum_compile() {
    let code = "fun main() {\n    let c = (0..10).count()\n    let s = (1..=4).sum()\n    println(c)\n    println(s)\n}\n";
    assert_eq!(compiled_output(code, "t10_literal"), "10\n10\n");
}

#[test]
fn test_g2bt10_range_min_max_compile() {
    let code = "fun main() {\n    let lo = range(3, 9).min()\n    let hi = range(3, 9).max()\n    println(lo)\n    println(hi)\n}\n";
    assert_eq!(compiled_output(code, "t10_minmax"), "Some(3)\nSome(8)\n");
}

#[test]
fn test_g2bt10_array_receiver_keeps_iter() {
    let code = "fun main() {\n    let total = [1, 2, 3].sum()\n    println(total)\n}\n";
    assert!(transpile(code).contains(". iter () . sum"));
    assert_same_output(code, "t10_array", "6\n");
}

// ------------------------------------------------------------- T12 {:?} Debug

const DEBUG_TABLE: &str = r#"fun main() {
    println!("{:?}", [1, 2])
    println!("{:?}", "hi")
    println!("{:?}", "a\"b\n")
    println!("{:?}", (1, true))
    println!("{:?}", 2.0)
    println!("{:?}", 1.5)
    println!("{:?}", -3)
    println!("{:?}", [[1, 0], [2, 3]])
    println!("{:?}", Some(3))
    println!("{:?}", true)
    println!("{:?}", [(1, "x")])
    println!("n={:?} s={:?} d={}", 7, "q", "r")
    let v: Option<i32> = None
    println!("{:?}", v)
    let r: Result<i32, String> = Ok(1)
    println!("{:?}", r)
    println!("{:?}", 0..3)
}
"#;

const DEBUG_EXPECTED: &str = r#"[1, 2]
"hi"
"a\"b\n"
(1, true)
2.0
1.5
-3
[[1, 0], [2, 3]]
Some(3)
true
[(1, "x")]
n=7 s="q" d=r
None
Ok(1)
0..3
"#;

#[test]
fn test_g2bt12_println_macro_debug_matches_rustc() {
    assert_same_output(DEBUG_TABLE, "t12_table", DEBUG_EXPECTED);
}

#[test]
fn test_g2bt12_format_macro_matches_rustc() {
    let code = "fun main() {\n    let s = format!(\"{:?}|{}-{}\", [\"a\", \"b\"], \"x\", 1)\n    println!(\"{}\", s)\n}\n";
    assert_same_output(code, "t12_format", "[\"a\", \"b\"]|x-1\n");
}

#[test]
fn test_g2bt12_println_function_substitutes_like_macro() {
    let code = "fun main() {\n    println(\"{:?}\", [1, 2])\n    println(\"{} {:?} {}\", \"a\", \"b\", 3.5)\n    println(\"{}+{}\", 1, 2)\n}\n";
    let expected = "[1, 2]\na \"b\" 3.5\n1+2\n";
    assert_eq!(interpreted_output(code, "t12_fn"), expected);
}
