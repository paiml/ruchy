//! IDXASSIGN-1: an expression in place (lvalue) position is emitted as a
//! place, with no `.clone()` anywhere in its chain. Place positions are the
//! target of `=` and of a compound assignment (`+=` etc.), the receiver chain
//! of a mutating method call (`o.rows[0].vals.push(x)`), and the operand of
//! `&mut`. The transpiler used to emit `o.items[0 as usize].clone().z = 5`,
//! which assigns to a temporary: the compiled binary silently printed the old
//! value.
//!
//! Every program is transpiled, compiled with rustc and run; the binary must
//! print the expected output. Where the interpreter runs the program, it must
//! print the same output.

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

fn transpile(dir: &Path, src: &str) -> String {
    let file = write_source(dir, src);
    let mut cmd = ruchy();
    cmd.arg("transpile").arg(file);
    stdout_of(cmd, "ruchy transpile")
}

fn interpret(dir: &Path, src: &str) -> String {
    let file = write_source(dir, src);
    let mut cmd = ruchy();
    cmd.arg("run").arg(file);
    stdout_of(cmd, "ruchy run")
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

/// Transpile + rustc + run must print `expected`; returns the Rust source.
fn assert_compiled(src: &str, expected: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let rust = transpile(dir.path(), src);
    let actual = rustc_run(dir.path(), &rust);
    assert_eq!(actual, expected, "compiled output\nRust:\n{rust}");
    rust
}

/// Interpreter and compiled binary must both print `expected`.
fn assert_both(src: &str, expected: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    assert_eq!(
        interpret(dir.path(), src),
        expected,
        "interpreter output\nsource:\n{src}"
    );
    assert_compiled(src, expected)
}

const ITEMS: &str = "struct P { z: i32 }\nstruct O { items: Vec<P> }\n";

#[test]
fn test_idxassign_1_01_field_of_indexed_element_assign() {
    let src = format!(
        "{ITEMS}fun main() {{\n    let mut o = O {{ items: vec![P {{ z: 1 }}, P {{ z: 2 }}] }}\n    \
         o.items[0].z = 5\n    println!(\"{{}} {{}}\", o.items[0].z, o.items[1].z)\n}}\n"
    );
    let rust = assert_both(&src, "5 2\n");
    assert!(
        !rust.contains(".clone().z ="),
        "assignment target must be a place:\n{rust}"
    );
}

#[test]
fn test_idxassign_1_02_field_of_indexed_element_compound_assign() {
    let src = format!(
        "{ITEMS}fun main() {{\n    let mut o = O {{ items: vec![P {{ z: 1 }}] }}\n    \
         o.items[0].z += 3\n    o.items[0].z *= 2\n    println!(\"{{}}\", o.items[0].z)\n}}\n"
    );
    assert_both(&src, "8\n");
}

/// Compiled only: the interpreter does not write `push` through an indexed
/// field chain back to `t` (a runtime defect reported with this ticket).
#[test]
fn test_idxassign_1_03_mutating_method_on_indexed_chain() {
    let src = "struct Row { vals: Vec<i32> }\nstruct T { rows: Vec<Row> }\nfun main() {\n    \
               let mut t = T { rows: vec![Row { vals: vec![1] }] }\n    \
               t.rows[0].vals.push(9)\n    t.rows[0].vals.push(4)\n    \
               println!(\"{} {}\", t.rows[0].vals.len(), t.rows[0].vals[1])\n}\n";
    assert_compiled(src, "3 9\n");
}

#[test]
fn test_idxassign_1_04_mut_ref_argument_of_indexed_element() {
    let src = format!(
        "{ITEMS}fun bump(p: &mut P) {{\n    p.z = p.z + 10\n}}\nfun main() {{\n    \
         let mut o = O {{ items: vec![P {{ z: 1 }}] }}\n    bump(&mut o.items[0])\n    \
         println!(\"{{}}\", o.items[0].z)\n}}\n"
    );
    assert_compiled(&src, "11\n");
}

#[test]
fn test_idxassign_1_05_nested_index_field_chain() {
    let src = "struct D { d: i32 }\nstruct C { c: Vec<D> }\nstruct B { b: Vec<C> }\n\
               fun main() {\n    let mut a = B { b: vec![C { c: vec![D { d: 0 }, D { d: 0 }] }] }\n    \
               let i = 0\n    let j = 1\n    a.b[i].c[j].d = 7\n    a.b[i].c[j].d += 1\n    \
               println!(\"{} {}\", a.b[0].c[0].d, a.b[0].c[1].d)\n}\n";
    assert_both(src, "0 8\n");
}

#[test]
fn test_idxassign_1_06_reads_still_compile() {
    let src = "struct N { name: String, z: i32 }\nstruct O { items: Vec<N> }\nfun main() {\n    \
               let o = O { items: vec![N { name: \"a\".to_string(), z: 3 }] }\n    \
               let y = o.items[0].z\n    let s = o.items[0].name\n    \
               println!(\"{} {} {}\", y, s, o.items[0].name)\n}\n";
    assert_both(src, "3 a a\n");
}

/// Compiled only: the interpreter rejects `v[1] += 5` with "Invalid compound
/// assignment target" (a runtime defect reported with this ticket).
#[test]
fn test_idxassign_1_07_plain_index_assign_unchanged() {
    let src = "fun main() {\n    let mut v = vec![1, 2, 3]\n    v[0] = 9\n    v[1] += 5\n    \
               println!(\"{} {} {}\", v[0], v[1], v[2])\n}\n";
    assert_compiled(src, "9 7 3\n");
}

#[test]
fn test_idxassign_1_08_whole_element_replace_in_field_vec() {
    let src = format!(
        "{ITEMS}fun main() {{\n    let mut o = O {{ items: vec![P {{ z: 1 }}, P {{ z: 2 }}] }}\n    \
         o.items[1] = P {{ z: 7 }}\n    println!(\"{{}} {{}}\", o.items[0].z, o.items[1].z)\n}}\n"
    );
    assert_both(&src, "1 7\n");
}
