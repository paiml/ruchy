//! G2BFA2 / RAWIDENT-1 (RHLGA-1), review round 3 of G2 part 2.
//!
//! 1. `&mut self` inference walks every expression kind that holds
//!    sub-expressions. The parser builds every generic macro call
//!    (`println!`, `format!`, `vec!`, `assert!`) as `MacroInvocation`; a
//!    mutation of `self` inside one must still make the receiver `&mut self`,
//!    or rustc rejects the method (E0596).
//!
//! Every program is transpiled, compiled with rustc and run.

use assert_cmd::Command;
use std::path::Path;
use std::time::Duration;

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

fn transpile(file: &Path) -> String {
    let mut cmd = ruchy();
    cmd.arg("transpile").arg(file);
    stdout_of(cmd, "ruchy transpile")
}

/// Compile `rust` with rustc inside `dir` and return the binary's stdout.
fn compiled_stdout(dir: &Path, rust: &str) -> String {
    let main_rs = dir.join("main.rs");
    let bin = dir.join("g2b_walkall_bin");
    std::fs::write(&main_rs, rust).expect("write main.rs");
    let mut rustc = Command::new("rustc");
    rustc
        .args(["--edition", "2021", "-A", "warnings", "-o"])
        .arg(&bin)
        .arg(&main_rs)
        .timeout(Duration::from_secs(120));
    stdout_of(rustc, &format!("rustc on transpiled Rust:\n{rust}\n"));
    let mut run_bin = Command::new(&bin);
    run_bin.timeout(Duration::from_secs(30));
    stdout_of(run_bin, "transpiled binary")
}

/// Whitespace-free token text, so `fn pop_print (& mut self)` matches.
fn squash(rust: &str) -> String {
    rust.chars().filter(|c| !c.is_whitespace()).collect()
}

// --------------------------------------- 1. mutation inside a macro call

const POP_PRINT: &str = r#"struct Stack {
    stack: Vec<i32>
}
impl Stack {
    fun pop_print(self) {
        println!("{}", self.stack.pop().unwrap())
    }
}
fun main() {
    let mut s = Stack { stack: vec![1, 2, 3] }
    s.pop_print()
    s.pop_print()
    println("done")
}
"#;

#[test]
fn test_g2bfa2_mutation_inside_macro_invocation_makes_receiver_mut() {
    let dir = tempfile::tempdir().expect("temp dir");
    let rust = transpile(&write_source(dir.path(), POP_PRINT));
    assert!(
        squash(&rust).contains("fnpop_print(&mutself)"),
        "`self.stack.pop()` inside println! must give `&mut self`:\n{rust}"
    );
}

#[test]
fn test_g2bfa2_mutation_inside_macro_invocation_compiles_and_runs() {
    let dir = tempfile::tempdir().expect("temp dir");
    let rust = transpile(&write_source(dir.path(), POP_PRINT));
    assert_eq!(compiled_stdout(dir.path(), &rust), "3\n2\ndone\n");
}

/// The same shape with a counter, so the interpreter can be the oracle too:
/// `ruchy run` does not persist `self.stack.pop()` on a field (it evaluates to
/// the popped array), a runtime defect outside this transpiler fix.
const BUMP_PRINT: &str = r#"struct Counter {
    n: i32
}
impl Counter {
    fun bump_print(self) {
        println!("{}", {
            self.n += 1
            self.n
        })
    }
}
fun main() {
    let mut c = Counter { n: 0 }
    c.bump_print()
    c.bump_print()
    println("done")
}
"#;

#[test]
fn test_g2bfa2_mutation_inside_macro_invocation_matches_run() {
    let dir = tempfile::tempdir().expect("temp dir");
    let file = write_source(dir.path(), BUMP_PRINT);
    let rust = transpile(&file);
    assert!(
        squash(&rust).contains("fnbump_print(&mutself)"),
        "`self.n += 1` inside println! must give `&mut self`:\n{rust}"
    );
    let binary_stdout = compiled_stdout(dir.path(), &rust);

    let mut interpreter = ruchy();
    interpreter.arg("run").arg(&file);
    let interpreter_stdout = stdout_of(interpreter, "ruchy run");

    assert_eq!(binary_stdout, "1\n2\ndone\n");
    assert_eq!(binary_stdout, interpreter_stdout);
}
