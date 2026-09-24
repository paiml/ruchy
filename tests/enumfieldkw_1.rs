//! ENUMFIELDKW-1: a keyword names a field of an enum struct variant, as it
//! names a struct field (RESFIELD-1) and a member after `.` (EXTENDKW-1).
//!
//! Before the fix `enum Test { Variant { enum: int } }` failed to parse with
//! "Expected RightBrace, found Enum" (found by the parser_defect_018
//! proptest `prop_enum_variant_with_single_field`).
//!
//! Rust reserved words are emitted as raw identifiers (`r#enum`).

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

fn stdout_of(mut cmd: Command, what: &str) -> String {
    let out = cmd.output().expect("spawn command");
    assert!(
        out.status.success(),
        "{what} failed: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

fn run_ruchy(sub: &str, src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let mut cmd = ruchy();
    cmd.arg(sub).arg(file);
    stdout_of(cmd, &format!("ruchy {sub} of:\n{src}\n"))
}

/// `ruchy compile` (rustc) the source, then run the binary.
fn compile_and_run(src: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_source(dir.path(), src);
    let bin = dir.path().join("prog_bin");
    let mut cmd = ruchy();
    cmd.arg("compile")
        .arg(&file)
        .arg("-o")
        .arg(&bin)
        .timeout(Duration::from_secs(60));
    stdout_of(cmd, &format!("ruchy compile of:\n{src}\n"));
    let mut run = Command::new(&bin);
    run.timeout(Duration::from_secs(30));
    stdout_of(run, "compiled binary")
}

fn squash(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
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

fn all_keywords() -> impl Iterator<Item = &'static str> {
    RUST_RESERVED.iter().chain(RUCHY_ONLY.iter()).copied()
}

fn single_field_enum(kw: &str) -> String {
    format!("enum Test {{\n    Variant {{ {kw}: i32 }}\n}}\n")
}

#[test]
fn test_enumfieldkw_1_every_keyword_parses_as_variant_field() {
    for kw in all_keywords() {
        run_ruchy("check", &single_field_enum(kw));
    }
}

#[test]
fn test_enumfieldkw_1_reserved_keyword_field_is_raw_ident() {
    for kw in RUST_RESERVED {
        let rust = squash(&run_ruchy("transpile", &single_field_enum(kw)));
        assert!(
            rust.contains(&format!("Variant{{r#{kw}:i32}}")),
            "`{kw}` must be emitted as r#{kw}: {rust}"
        );
    }
}

#[test]
fn test_enumfieldkw_1_ruchy_only_keyword_field_is_plain_ident() {
    for kw in RUCHY_ONLY {
        let rust = squash(&run_ruchy("transpile", &single_field_enum(kw)));
        assert!(
            rust.contains(&format!("Variant{{{kw}:i32}}")),
            "`{kw}` must be emitted as a plain identifier: {rust}"
        );
    }
}

#[test]
fn test_enumfieldkw_1_proptest_shape_with_literal() {
    let src = "fn main() {\n    enum Test {\n        Variant { enum: int }\n    }\n    \
               let x = Test::Variant { enum: 5 }\n    println(x)\n}\n";
    run_ruchy("check", src);
}

#[test]
fn test_enumfieldkw_1_all_keyword_fields_compile() {
    let fields: Vec<String> = all_keywords().map(|kw| format!("{kw}: i32")).collect();
    let values: Vec<String> = all_keywords().map(|kw| format!("{kw}: 1")).collect();
    let src = format!(
        "enum Test {{\n    Variant {{ {} }},\n    Empty\n}}\n\
         fun main() {{\n    let _t = Test::Variant {{ {} }}\n    println!(\"ok\")\n}}\n",
        fields.join(", "),
        values.join(", ")
    );
    assert_eq!(compile_and_run(&src).trim(), "ok");
}
