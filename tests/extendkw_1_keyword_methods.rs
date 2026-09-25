//! EXTENDKW-1: a keyword token after `.` is a method or field name.
//!
//! `v.extend([3])` failed with "Expected method name, tuple index, or 'await'
//! after '.'": `extend` lexes as `Token::Extend` (Ruchy's extension-block
//! keyword), and the postfix `.` parser only accepted `Token::Identifier`
//! (plus `send`/`ask`). The same held for `o.type`, `o.match(x)`,
//! `o.default()`, `o.from(x)`, and every other keyword.
//!
//! After `.` (and `?.`) a keyword is now the member name spelled as its
//! source text. `x.await` stays an await expression. Transpiled Rust escapes a
//! Rust reserved word as a raw identifier (`o.r#match(1)`).

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
        "{what} failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

fn check(src: &str) {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let mut cmd = ruchy();
    cmd.arg("check").arg(file);
    stdout_of(cmd, &format!("ruchy check on:\n{src}\n"));
}

fn transpile(dir: &Path, src: &str) -> String {
    let file = write_source(dir, src);
    let mut cmd = ruchy();
    cmd.arg("transpile").arg(file);
    stdout_of(cmd, &format!("ruchy transpile on:\n{src}\n"))
}

fn transpile_str(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    transpile(dir.path(), src)
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

/// Keywords that are Rust reserved words: transpiled as `r#kw`.
const RUST_RESERVED: &[&str] = &[
    "match", "type", "loop", "impl", "mod", "use", "as", "in", "where", "static", "const", "trait",
    "struct", "enum", "try", "yield", "final", "abstract", "override", "async", "return", "break",
    "continue", "if", "else", "for", "while", "let", "fn", "pub", "mut", "unsafe",
];

/// Ruchy-only keywords: plain Rust identifiers.
const RUCHY_ONLY: &[&str] = &[
    "extend",
    "from",
    "default",
    "module",
    "var",
    "with",
    "import",
    "export",
    "class",
    "handle",
    "handler",
    "effect",
    "property",
    "private",
    "protected",
    "sealed",
    "mixin",
    "operator",
    "interface",
    "implements",
    "receive",
    "spawn",
    "actor",
    "lazy",
    "catch",
    "finally",
    "throw",
    "signal",
    "infra",
    "requires",
    "ensures",
    "invariant",
    "decreases",
    "fun",
    "send",
    "ask",
];

fn rust_member(kw: &str) -> String {
    if RUST_RESERVED.contains(&kw) {
        format!("r#{kw}")
    } else {
        kw.to_string()
    }
}

fn all_keywords() -> impl Iterator<Item = &'static str> {
    RUST_RESERVED.iter().chain(RUCHY_ONLY.iter()).copied()
}

/// Strip whitespace so `o . r#match (1)` and `o.r#match(1)` compare equal.
fn squash(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn method_src(kw: &str) -> String {
    format!("fun probe(o: Widget) -> i32 {{\n    o.{kw}(1)\n}}\n")
}

fn field_src(kw: &str) -> String {
    format!("fun probe(o: Widget) -> i32 {{\n    o.{kw}\n}}\n")
}

#[test]
fn test_extendkw_1_01_vec_extend_checks_clean() {
    check("let mut v = [1, 2]\nv.extend([3])\nprintln(v.len())\n");
}

#[test]
fn test_extendkw_1_02_vec_extend_transpiles_to_method_call() {
    let rust = transpile_str(
        "fun main() {\n    let mut v = [1, 2]\n    v.extend([3])\n    println(v.len())\n}\n",
    );
    assert!(
        squash(&rust).contains("v.extend("),
        "expected a v.extend(..) call in:\n{rust}"
    );
}

#[test]
fn test_extendkw_1_03_vec_extend_compiles_with_rustc() {
    // A struct field typed Vec<i32> transpiles its literal to vec![..] (VECLIT-1);
    // a bare `let mut v = [1, 2]` still transpiles to a fixed-size Rust array.
    let src = "struct Bag {\n    items: Vec<i32>,\n}\n\nfun main() {\n    let mut b = Bag { items: [1, 2] }\n    b.items.extend([3])\n    println(b.items.len())\n}\n";
    let dir = tempfile::tempdir().expect("tempdir");
    let rust = transpile(dir.path(), src);
    assert!(
        squash(&rust).contains("b.items.extend([3])"),
        "expected b.items.extend([3]) in:\n{rust}"
    );
    let out = rustc_run(dir.path(), &rust);
    assert_eq!(out.trim(), "3", "rust:\n{rust}");
}

#[test]
fn test_extendkw_1_04_every_keyword_is_a_method_name() {
    for kw in all_keywords() {
        let src = method_src(kw);
        check(&src);
        let rust = squash(&transpile_str(&src));
        let want = format!("o.{}(1", rust_member(kw));
        assert!(
            rust.contains(&want),
            "`o.{kw}(1)`: expected `{want}` in:\n{rust}"
        );
    }
}

#[test]
fn test_extendkw_1_05_every_keyword_is_a_field_name() {
    // RESFIELD-1: a field named by a Rust reserved word transpiles to `r#kw`.
    for kw in all_keywords() {
        let src = field_src(kw);
        check(&src);
        let rust = squash(&transpile_str(&src));
        let want = format!("o.{}", rust_member(kw));
        assert!(
            rust.contains(&want),
            "`o.{kw}`: expected `{want}` in:\n{rust}"
        );
    }
}

#[test]
fn test_extendkw_1_05b_reserved_field_access_compiles_with_rustc() {
    // RESFIELD-1: keyword field names in a declaration, a literal and an access.
    let src = "struct Widget {\n    type: i32,\n    match: i32,\n}\n\nfun probe(o: Widget) -> i32 {\n    o.type + o.match\n}\n\nfun main() {\n    println(probe(Widget { type: 5, match: 2 }))\n}\n";
    check(src);
    let dir = tempfile::tempdir().expect("tempdir");
    let rust = transpile(dir.path(), src);
    assert_eq!(rustc_run(dir.path(), &rust).trim(), "7", "rust:\n{rust}");
}

#[test]
fn test_extendkw_1_06_keyword_method_chains() {
    let src = "fun probe(o: Widget) -> i32 {\n    o.extend(1).match(2).default().type(3)\n}\n";
    check(src);
    let rust = squash(&transpile_str(src));
    assert!(
        rust.contains("o.extend(1).r#match(2).default().r#type(3)"),
        "chain lost a member:\n{rust}"
    );
}

#[test]
fn test_extendkw_1_07_await_stays_await() {
    let src = "async fun probe(x: i32) -> i32 {\n    x.await\n}\n";
    check(src);
    let rust = squash(&transpile_str(src));
    assert!(rust.contains("x.await"), "expected `x.await` in:\n{rust}");
    assert!(!rust.contains("r#await"), "await became a member:\n{rust}");
}

#[test]
fn test_extendkw_1_08_keyword_after_optional_chain() {
    check("let o = {\"extend\": 1}\nlet a = o?.extend\nlet b = o?.type\nprintln(a)\n");
}

#[test]
fn test_extendkw_1_09_tuple_index_and_identifier_members_unchanged() {
    let src = "fun probe(t: (i32, i32), o: Widget) -> i32 {\n    t.0 + o.len() + o.size\n}\n";
    check(src);
    let rust = squash(&transpile_str(src));
    for want in ["t.0", "o.len()", "o.size"] {
        assert!(rust.contains(want), "expected `{want}` in:\n{rust}");
    }
}
