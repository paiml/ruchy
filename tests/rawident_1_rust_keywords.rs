//! RAWIDENT-1: a Ruchy identifier that is a Rust reserved word but not a
//! Ruchy keyword (`do`, `box`, `typeof`, ...) used as a function or method
//! NAME must be emitted as a raw identifier (`r#do`) at both the definition
//! and the call site, so rustc accepts the transpiled code. Before the fix,
//! `Transpiler::safe_ident` did not exist and `fn do(&self)` / `obj.do()`
//! were emitted verbatim, which rustc rejects (E0379/E0533-class syntax
//! errors: `do` etc. are Rust keywords even though Ruchy allows them as
//! plain identifiers).
//!
//! This test transpiles a class with methods named `do` and `box` plus a
//! free function named `typeof`, compiles the result with rustc, runs the
//! binary, and checks its stdout matches `ruchy run` on the same source.

use assert_cmd::Command;
use std::path::Path;
use std::time::Duration;

/// A struct with two reserved-but-not-Ruchy-keyword method names, plus a
/// free function with a reserved-but-not-Ruchy-keyword name, called from
/// `main`.
const RAWIDENT_SOURCE: &str = r"class Thing {
    n: i32
    new(n: i32) { self.n = n }
    fun do(&self) -> i32 { self.n }
    fun box(&self) -> i32 { self.n * 2 }
}
fun typeof(x: i32) -> i32 { x + 100 }
fun main() {
    let t = Thing::new(5)
    println(t.do())
    println(t.box())
    println(typeof(t.n))
}
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

/// Test 1: `ruchy run` (interpreter) already prints the expected values;
/// this is the oracle the transpiled binary must match.
#[test]
fn test_rawident_1_01_interpreter_runs_reserved_word_names() {
    let dir = tempfile::tempdir().expect("temp dir");
    let file = write_source(dir.path(), RAWIDENT_SOURCE);
    let mut cmd = ruchy();
    cmd.arg("run").arg(&file);
    assert_eq!(stdout_of(cmd, "ruchy run"), "5\n10\n105\n");
}

/// Test 2: the transpiler must emit raw identifiers (`r#do`, `r#box`,
/// `r#typeof`) at both the definition and the call site.
#[test]
fn test_rawident_1_02_transpile_emits_raw_identifiers() {
    let dir = tempfile::tempdir().expect("temp dir");
    let file = write_source(dir.path(), RAWIDENT_SOURCE);
    let mut cmd = ruchy();
    cmd.arg("transpile").arg(&file);
    let rust = stdout_of(cmd, "ruchy transpile");

    // Definition sites
    assert!(
        rust.contains("fn r#do"),
        "method definition `do` must be raw:\n{rust}"
    );
    assert!(
        rust.contains("fn r#box"),
        "method definition `box` must be raw:\n{rust}"
    );
    assert!(
        rust.contains("fn r#typeof"),
        "function definition `typeof` must be raw:\n{rust}"
    );

    // Call sites
    assert!(
        rust.contains(".r#do()"),
        "method call `.do()` must be raw:\n{rust}"
    );
    assert!(
        rust.contains(".r#box()"),
        "method call `.box()` must be raw:\n{rust}"
    );
    assert!(
        rust.contains("r#typeof(t.n)") || rust.contains("r#typeof (t . n)"),
        "function call `typeof(..)` must be raw:\n{rust}"
    );
}

/// Test 3: the transpiled program compiles with rustc and, once run, prints
/// exactly what `ruchy run` prints for the same source.
#[test]
fn test_rawident_1_03_transpiled_binary_matches_interpreter() {
    let dir = tempfile::tempdir().expect("temp dir");
    let file = write_source(dir.path(), RAWIDENT_SOURCE);

    let mut transpile = ruchy();
    transpile.arg("transpile").arg(&file);
    let rust = stdout_of(transpile, "ruchy transpile");

    let main_rs = dir.path().join("main.rs");
    let bin = dir.path().join("rawident_bin");
    std::fs::write(&main_rs, &rust).expect("write main.rs");

    let mut rustc = Command::new("rustc");
    rustc
        .args(["--edition", "2021", "-A", "warnings", "-o"])
        .arg(&bin)
        .arg(&main_rs)
        .timeout(Duration::from_secs(120));
    stdout_of(rustc, &format!("rustc on transpiled Rust:\n{rust}\n"));

    let mut run_bin = Command::new(&bin);
    run_bin.timeout(Duration::from_secs(30));
    let binary_stdout = stdout_of(run_bin, "transpiled binary");

    let mut interpreter = ruchy();
    interpreter.arg("run").arg(&file);
    let interpreter_stdout = stdout_of(interpreter, "ruchy run");

    assert_eq!(binary_stdout, interpreter_stdout);
    assert_eq!(binary_stdout, "5\n10\n105\n");
}

/// Test 4: a reserved word reached through a `::` path (`m::r#do()`) is raw too,
/// so the module path call compiles (review round 2 of G2 part 2).
#[test]
fn test_rawident_1_04_module_path_call_is_raw() {
    let dir = tempfile::tempdir().expect("temp dir");
    let file = write_source(
        dir.path(),
        "mod m {\n    pub fun do() -> i32 {\n        7\n    }\n}\n\nfun main() {\n    println(m::do())\n}\n",
    );
    let mut transpile = ruchy();
    transpile.arg("transpile").arg(&file);
    let rust = stdout_of(transpile, "ruchy transpile");
    assert!(rust.contains("r#do"), "expected r#do in:\n{rust}");

    let main_rs = dir.path().join("main.rs");
    let bin = dir.path().join("rawident_path_bin");
    std::fs::write(&main_rs, &rust).expect("write main.rs");
    let mut rustc = Command::new("rustc");
    rustc
        .args(["--edition", "2021", "-A", "warnings", "-o"])
        .arg(&bin)
        .arg(&main_rs)
        .timeout(Duration::from_secs(120));
    stdout_of(rustc, &format!("rustc on transpiled Rust:\n{rust}\n"));
    let mut run_bin = Command::new(&bin);
    run_bin.timeout(Duration::from_secs(30));
    assert_eq!(stdout_of(run_bin, "transpiled binary"), "7\n");
}
