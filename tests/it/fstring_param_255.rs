//! ruchy#255 item 5: a parameter used only inside an f-string is inferred as
//! `String`, and the transpiled program builds and prints the interpolation.

use assert_cmd::Command;
use std::process::Command as StdCommand;

const GREET: &str = "fun greet(who) {\n    println(f\"Hello, {who}!\");\n}\n\ngreet(\"Ruchy\");\n";

fn transpile_greet() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("greet.ruchy");
    std::fs::write(&src, GREET).expect("write source");
    let out = Command::cargo_bin("ruchy")
        .expect("ruchy binary is built")
        .arg("transpile")
        .arg(&src)
        .output()
        .expect("ruchy runs");
    assert!(
        out.status.success(),
        "transpile failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    (dir, String::from_utf8_lossy(&out.stdout).into_owned())
}

#[test]
fn test_ruchy_255_item5_fstring_param_emits_string() {
    let (_dir, rust) = transpile_greet();
    assert!(
        rust.contains("fn greet(who: String)"),
        "expected `who: String` for an f-string-only parameter, got:\n{rust}"
    );
}

#[test]
fn test_ruchy_255_item5_fstring_param_program_builds_and_prints() {
    let (dir, rust) = transpile_greet();
    let rs = dir.path().join("greet.rs");
    let bin = dir.path().join("greet_bin");
    std::fs::write(&rs, &rust).expect("write rust");
    let build = StdCommand::new("rustc")
        .args(["--edition=2021", "--crate-type=bin", "-o"])
        .arg(&bin)
        .arg(&rs)
        .output()
        .expect("rustc runs");
    assert!(
        build.status.success(),
        "generated program does not build:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let run = StdCommand::new(&bin).output().expect("binary runs");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "Hello, Ruchy!\n");
}
