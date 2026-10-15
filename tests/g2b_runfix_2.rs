//! RHLGA-1 G2b run fixes, part 2: four interpreter defects measured with
//! `ruchy run` at a5dc8398.
//!
//! - FMTSPEC-1: the format engine substituted only `{}`/`{:?}`;
//!   `println("[{:>5}] [{:.2}] [{0}]", 7, 3.14159)` printed the spec text
//!   and appended the values. Rust prints `[    7] [3.14] [7]`.
//! - EPRINTLN-1: `eprintln`/`eprint` were "Undefined function".
//! - FORMATFN-1 (interpreter half): `format("{:?}", [1, 2])` failed with
//!   "Unknown builtin __builtin_format__".
//! - DICTIDX-1 (interpreter half): `d["k"].push(2)` on an object failed with
//!   "Array index must be an integer"; string keys now work at any link of an
//!   index/field chain for read, assignment, compound assignment and in-place
//!   methods.
//!
//! Each case states the output Rust prints for the same program. A
//! hand-written Rust reference is compiled with rustc to prove that string,
//! and `ruchy run` must print it too. `ruchy compile` parity is the
//! transpiler half, fixed in another worktree.

use assert_cmd::Command;
use std::path::Path;
use std::time::Duration;

struct Output {
    stdout: String,
    stderr: String,
}

fn output_of(mut cmd: Command, what: &str) -> Output {
    let out = cmd.output().expect("spawn command");
    assert!(
        out.status.success(),
        "{what} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    Output {
        stdout: String::from_utf8(out.stdout).expect("utf8 stdout"),
        stderr: String::from_utf8(out.stderr).expect("utf8 stderr"),
    }
}

fn ruchy_cmd(dir: &Path, src: &str) -> Command {
    let file = dir.join("prog.ruchy");
    std::fs::write(&file, src).expect("write ruchy source");
    let mut cmd = Command::cargo_bin("ruchy").expect("ruchy binary is built for tests");
    cmd.arg("run").arg(file).timeout(Duration::from_secs(10));
    cmd
}

fn rustc_run(dir: &Path, rust: &str) -> Output {
    let main_rs = dir.join("reference.rs");
    let bin = dir.join("reference_bin");
    std::fs::write(&main_rs, rust).expect("write reference.rs");
    let mut rustc = Command::new("rustc");
    rustc
        .args(["--edition", "2021", "-A", "warnings", "-o"])
        .arg(&bin)
        .arg(&main_rs)
        .timeout(Duration::from_secs(120));
    output_of(rustc, "rustc on the Rust reference");
    let mut run = Command::new(&bin);
    run.timeout(Duration::from_secs(10));
    output_of(run, "Rust reference binary")
}

/// Rust prints `expected` for `rust`, and `ruchy run` prints it for `ruchy`.
fn assert_matches_rust(ruchy: &str, rust: &str, expected: &str) {
    let dir = tempfile::tempdir().expect("tempdir");
    assert_eq!(
        rustc_run(dir.path(), rust).stdout,
        expected,
        "Rust reference"
    );
    let got = output_of(ruchy_cmd(dir.path(), ruchy), "ruchy run");
    assert_eq!(got.stdout, expected, "ruchy run");
}

// ---------------------------------------------------------------- FMTSPEC-1

#[test]
fn test_fmtspec_1_brief_case_width_precision_positional() {
    assert_matches_rust(
        r#"
fun main() {
    println("[{:>5}] [{:.2}] [{0}]", 7, 3.14159)
}
"#,
        r#"
fn main() {
    println!("[{:>5}] [{:.2}] [{0}]", 7, 3.14159);
}
"#,
        "[    7] [3.14] [7]\n",
    );
}

#[test]
fn test_fmtspec_1_inline_named_args_fill_align_sign_zero() {
    assert_matches_rust(
        r#"
fun main() {
    let name = "ab"
    let n = 42
    println("[{name:>6}] [{n:<4}] [{n:^6}] [{n:+}] [{n:05}] [{:+06}]", -7)
}
"#,
        r#"
fn main() {
    let name = "ab";
    let n = 42;
    println!("[{name:>6}] [{n:<4}] [{n:^6}] [{n:+}] [{n:05}] [{:+06}]", -7);
}
"#,
        "[    ab] [42  ] [  42  ] [+42] [00042] [-00007]\n",
    );
}

#[test]
fn test_fmtspec_1_explicit_named_args() {
    assert_matches_rust(
        r#"
fun main() {
    println("{a}-{b}-{a}", a = 1, b = "z")
}
"#,
        r#"
fn main() {
    println!("{a}-{b}-{a}", a = 1, b = "z");
}
"#,
        "1-z-1\n",
    );
}

#[test]
fn test_fmtspec_1_radix_and_alternate() {
    assert_matches_rust(
        r#"
fun main() {
    println("{:x} {:X} {:b} {:o} {:#x} {:#b} {:#o} {:08b} {:#06x}", 255, 255, 5, 8, 255, 5, 8, 5, 255)
}
"#,
        r#"
fn main() {
    println!("{:x} {:X} {:b} {:o} {:#x} {:#b} {:#o} {:08b} {:#06x}", 255, 255, 5, 8, 255, 5, 8, 5, 255);
}
"#,
        "ff FF 101 10 0xff 0b101 0o10 00000101 0x00ff\n",
    );
}

#[test]
fn test_fmtspec_1_floats_exp_width_precision() {
    assert_matches_rust(
        r#"
fun main() {
    println("{:e} [{:10.3}] [{:<10.1}] {:+.2} {:08.2} {:E}", 1234.5, 3.14159, 2.75, 1.5, -3.14159, 0.00012)
}
"#,
        r#"
fn main() {
    println!("{:e} [{:10.3}] [{:<10.1}] {:+.2} {:08.2} {:E}", 1234.5, 3.14159, 2.75, 1.5, -3.14159, 0.00012);
}
"#,
        "1.2345e3 [     3.142] [2.8       ] +1.50 -0003.14 1.2E-4\n",
    );
}

#[test]
fn test_fmtspec_1_strings_fill_and_truncate() {
    assert_matches_rust(
        r#"
fun main() {
    println("[{:>8}] [{:*^9}] [{:.3}] [{:-<6}] [{:>6.2}]", "hi", "mid", "abcdef", "x", "hello")
}
"#,
        r#"
fn main() {
    println!("[{:>8}] [{:*^9}] [{:.3}] [{:-<6}] [{:>6.2}]", "hi", "mid", "abcdef", "x", "hello");
}
"#,
        "[      hi] [***mid***] [abc] [x-----] [    he]\n",
    );
}

#[test]
fn test_fmtspec_1_escapes_debug_and_pretty_debug() {
    assert_matches_rust(
        r#"
fun main() {
    println("{{}} {{{}}} {:?} {:>5?}|", 1, "q", 3)
    println("{:#?}", [1, 2])
}
"#,
        r#"
fn main() {
    println!("{{}} {{{}}} {:?} {:>5?}|", 1, "q", 3);
    println!("{:#?}", vec![1, 2]);
}
"#,
        "{} {1} \"q\"     3|\n[\n    1,\n    2,\n]\n",
    );
}

#[test]
fn test_fmtspec_1_format_macro_and_print_share_the_engine() {
    assert_matches_rust(
        r#"
fun main() {
    let w = 3
    let s = format!("<{:>4}|{w:02}>", "a")
    print("{:<3}|", 1)
    println("{}", s)
}
"#,
        r#"
fn main() {
    let w = 3;
    let s = format!("<{:>4}|{w:02}>", "a");
    print!("{:<3}|", 1);
    println!("{}", s);
}
"#,
        "1  |<   a|03>\n",
    );
}

#[test]
fn test_fmtspec_1_width_from_argument_is_an_error_not_wrong_output() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = ruchy_cmd(
        dir.path(),
        "fun main() {\n    println(\"[{:1$}]\", 7, 5)\n}\n",
    )
    .output()
    .expect("spawn ruchy");
    assert!(!out.status.success(), "must fail, got {:?}", out);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("1$"), "error must name the spec: {stderr}");
    assert!(!String::from_utf8_lossy(&out.stdout).contains('['));
}

// ---------------------------------------------------------------- EPRINTLN-1

#[test]
fn test_eprintln_1_eprintln_and_eprint_write_stderr() {
    let ruchy = r#"
fun main() {
    eprintln("x {}", 1)
    eprint("y")
    eprint("{:>3}\n", 7)
    eprintln("{:?}", "q")
    println("out")
}
"#;
    let rust = r#"
fn main() {
    eprintln!("x {}", 1);
    eprint!("y");
    eprint!("{:>3}\n", 7);
    eprintln!("{:?}", "q");
    println!("out");
}
"#;
    let dir = tempfile::tempdir().expect("tempdir");
    let reference = rustc_run(dir.path(), rust);
    assert_eq!(reference.stdout, "out\n");
    assert_eq!(reference.stderr, "x 1\ny  7\n\"q\"\n");
    let got = output_of(ruchy_cmd(dir.path(), ruchy), "ruchy run");
    assert_eq!(got.stdout, reference.stdout, "ruchy stdout");
    assert_eq!(got.stderr, reference.stderr, "ruchy stderr");
}

// ---------------------------------------------------------------- FORMATFN-1

#[test]
fn test_formatfn_1_format_function_form_matches_macro() {
    assert_matches_rust(
        r#"
fun main() {
    let s = format("{:?}", [1, 2])
    let t = format("{:>4}|{}", 7, "z")
    println("{} {}", s, t)
}
"#,
        r#"
fn main() {
    let s = format!("{:?}", vec![1, 2]);
    let t = format!("{:>4}|{}", 7, "z");
    println!("{} {}", s, t);
}
"#,
        "[1, 2]    7|z\n",
    );
}

// ---------------------------------------------------------------- DICTIDX-1

#[test]
fn test_dictidx_1_push_through_string_key() {
    assert_matches_rust(
        r#"
fun main() {
    let mut d = {"k": [1]}
    d["k"].push(2)
    println("{:?}", d["k"])
}
"#,
        r#"
use std::collections::HashMap;
fn main() {
    let mut d: HashMap<&str, Vec<i64>> = HashMap::new();
    d.insert("k", vec![1]);
    d.get_mut("k").unwrap().push(2);
    println!("{:?}", d["k"]);
}
"#,
        "[1, 2]\n",
    );
}

#[test]
fn test_dictidx_1_assign_and_compound_assign_string_key() {
    assert_matches_rust(
        r#"
fun main() {
    let mut d = {"n": 1}
    d["n"] = 5
    d["n"] += 1
    d["m"] = 10
    println("{} {}", d["n"], d["m"])
}
"#,
        r#"
use std::collections::HashMap;
fn main() {
    let mut d: HashMap<&str, i64> = HashMap::new();
    d.insert("n", 1);
    d.insert("n", 5);
    *d.get_mut("n").unwrap() += 1;
    d.insert("m", 10);
    println!("{} {}", d["n"], d["m"]);
}
"#,
        "6 10\n",
    );
}

#[test]
fn test_dictidx_1_mixed_chains() {
    assert_matches_rust(
        r#"
fun main() {
    let mut m = {"a": {"b": [1]}}
    m["a"]["b"].push(3)
    m["a"]["b"][0] += 40
    let mut v = [{"x": 1}]
    v[0]["x"] += 10
    println("{:?} {}", m["a"]["b"], v[0]["x"])
}
"#,
        r#"
use std::collections::HashMap;
fn main() {
    let mut inner: HashMap<&str, Vec<i64>> = HashMap::new();
    inner.insert("b", vec![1]);
    let mut m: HashMap<&str, HashMap<&str, Vec<i64>>> = HashMap::new();
    m.insert("a", inner);
    m.get_mut("a").unwrap().get_mut("b").unwrap().push(3);
    m.get_mut("a").unwrap().get_mut("b").unwrap()[0] += 40;
    let mut x: HashMap<&str, i64> = HashMap::new();
    x.insert("x", 1);
    let mut v = vec![x];
    *v[0].get_mut("x").unwrap() += 10;
    println!("{:?} {}", m["a"]["b"], v[0]["x"]);
}
"#,
        "[41, 3] 11\n",
    );
}
