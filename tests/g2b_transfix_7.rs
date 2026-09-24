//! RHLGA-1 transpiler fixes (seventh batch), each checked end to end:
//! `ruchy compile` (rustc), run the binary, compare exact stdout, and require
//! `ruchy run` to print the same.
//!
//! - RAWIDENT-2: a Ruchy identifier that is a Rust reserved word (`box`, `do`,
//!   `typeof`, ...) in a binding position (mutable global, function or closure
//!   parameter, `for` variable, pattern) is emitted as a raw identifier.

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

/// (stdout, stderr) of a command that must succeed.
fn output_of(mut cmd: Command, what: &str) -> (String, String) {
    let out = cmd.output().expect("spawn command");
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    let stderr = String::from_utf8(out.stderr).expect("utf8 stderr");
    assert!(out.status.success(), "{what} failed:\n{stdout}{stderr}");
    (stdout, stderr)
}

fn transpile(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let mut cmd = ruchy();
    cmd.arg("transpile").arg(file);
    output_of(cmd, &format!("ruchy transpile of:\n{src}\n")).0
}

/// `ruchy compile` the source, run the binary, return (stdout, stderr).
fn compile_and_run(src: &str) -> (String, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let bin = dir.path().join("prog_bin");
    let mut cmd = ruchy();
    cmd.arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(&bin)
        .timeout(Duration::from_secs(90));
    output_of(cmd, &format!("ruchy compile of:\n{src}\n"));
    let mut run = Command::new(&bin);
    run.timeout(Duration::from_secs(30));
    output_of(run, "compiled binary")
}

fn interpret(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let mut cmd = ruchy();
    cmd.arg("run").arg(file);
    output_of(cmd, &format!("ruchy run of:\n{src}\n")).0
}

fn main_with(body: &str) -> String {
    format!("fun main() {{\n{body}\n}}\n")
}

/// Compiled stdout is `expected`, and `ruchy run` agrees.
fn check(src: &str, expected: &str) {
    let (stdout, _) = compile_and_run(src);
    assert_eq!(stdout, expected, "compiled stdout of:\n{src}");
    assert_eq!(interpret(src), expected, "ruchy run stdout of:\n{src}");
}

#[test]
fn test_rawident_2_mutable_global_named_box() {
    check(
        "let mut box = 0\nfun update() {\n    box = box + 1\n}\nupdate()\nprintln(box)\n",
        "1\n",
    );
}

#[test]
fn test_rawident_2_function_parameter_named_box() {
    check(
        "fun f(box: i32) -> i32 { box + 1 }\nfun main() { println(f(1)) }\n",
        "2\n",
    );
}

#[test]
fn test_rawident_2_mutable_parameter_named_do() {
    check(
        "fun f(mut do: i32) -> i32 {\n    do = do * 2\n    do\n}\nfun main() { println(f(4)) }\n",
        "8\n",
    );
}

#[test]
fn test_rawident_2_for_variable_named_box() {
    check(
        &main_with("    for box in [1, 2] { println(box) }"),
        "1\n2\n",
    );
}

#[test]
fn test_rawident_2_closure_parameter_named_box() {
    check(
        &main_with("    let f = |box| box + 1\n    println(f(1))"),
        "2\n",
    );
}

#[test]
fn test_rawident_2_typed_closure_parameter_named_typeof() {
    check(
        &main_with("    let f = |typeof: i32| typeof * 3\n    println(f(2))"),
        "6\n",
    );
}

#[test]
fn test_rawident_2_tuple_pattern_binds_box() {
    check(
        &main_with("    let (box, n) = (3, 4)\n    println(box + n)"),
        "7\n",
    );
}

#[test]
fn test_rawident_2_emits_raw_identifier() {
    let rust = transpile("fun f(box: i32) -> i32 { box + 1 }\nfun main() { println(f(1)) }\n");
    assert!(rust.contains("r#box"), "expected r#box in:\n{rust}");
}

/// Every Rust reserved word that Ruchy lexes as a plain identifier transpiles
/// (syn parses the output) as a mutable global, and as a parameter, a `for`
/// variable and a closure parameter.
#[test]
fn test_rawident_2_every_reserved_word_ruchy_accepts() {
    const RUST_RESERVED: &[&str] = &[
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
        "while", "async", "await", "dyn", "final", "try", "abstract", "become", "box", "do",
        "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "gen",
    ];
    let idents: Vec<&str> = RUST_RESERVED
        .iter()
        .copied()
        .filter(|w| {
            let mut tokens = ruchy::TokenStream::new(w);
            matches!(tokens.next(), Some((ruchy::Token::Identifier(_), _)))
                && tokens.next().is_none()
        })
        .collect();
    assert!(
        idents.contains(&"box"),
        "box lexes as an identifier: {idents:?}"
    );
    for w in idents {
        transpile(&format!(
            "let mut {w} = 0\nfun update() {{\n    {w} = {w} + 1\n}}\nupdate()\nprintln({w})\n"
        ));
        transpile(&format!(
            "fun bump({w}: i32) -> i32 {{ {w} + 1 }}\nfun main() {{\n    for {w} in [1] {{ println(bump({w})) }}\n    let f = |{w}| {w} + 1\n    println(f(2))\n}}\n"
        ));
    }
}
