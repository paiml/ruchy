//! G2BFB (RHLGA-1): review fixes on the G2 part 2 branch.
//!
//! Item 5: a nested module keeps the visibility the parser recorded
//! (`pub(crate)`, `pub(super)`), instead of widening every `pub(..)` to `pub`.
//! The program is transpiled, compiled with rustc and run.

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::{Parser, Transpiler};
use std::process::Command;

fn transpile(code: &str) -> String {
    let ast = Parser::new(code).parse().expect("parses");
    Transpiler::new()
        .transpile_to_program(&ast)
        .expect("transpiles")
        .to_string()
}

fn compile_and_run(rust: &str) -> String {
    let dir = tempfile::tempdir().expect("scratch dir");
    let src = dir.path().join("main.rs");
    let bin = dir.path().join("main");
    std::fs::write(&src, rust).expect("write rust");
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

const NESTED_VISIBILITY: &str = r#"
mod outer {
    pub(crate) mod inner {
        pub fun value() -> i32 { 7 }
    }
    pub(super) mod up {
        pub fun value() -> i32 { 5 }
    }
    pub mod open {
        pub fun value() -> i32 { 1 }
    }
}
fun main() {
    println!("{}", outer::inner::value() + outer::up::value() + outer::open::value())
}
"#;

#[test]
fn test_g2bfb5_01_nested_pub_crate_module_keeps_its_visibility() {
    let rust = transpile(NESTED_VISIBILITY);
    assert!(rust.contains("pub (crate) mod inner"), "{rust}");
}

#[test]
fn test_g2bfb5_02_nested_pub_super_module_keeps_its_visibility() {
    let rust = transpile(NESTED_VISIBILITY);
    assert!(rust.contains("pub (super) mod up"), "{rust}");
}

#[test]
fn test_g2bfb5_03_nested_plain_pub_module_stays_pub() {
    let rust = transpile(NESTED_VISIBILITY);
    assert!(rust.contains("pub mod open"), "{rust}");
}

#[test]
fn test_g2bfb5_04_nested_module_visibilities_compile_and_run() {
    let rust = transpile(NESTED_VISIBILITY);
    assert_eq!(compile_and_run(&rust).trim(), "13", "{rust}");
}
