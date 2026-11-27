//! ruchy#343: a parameter used only inside a `println` f-string, with no call
//! site, is inferred as `String` and the transpiled code builds.

use assert_cmd::Command;
use std::process::Command as StdCommand;

const GREET_NO_CALL: &str = "fun greet(who) {\n    println(f\"Hello, {who}!\");\n}\n";

fn transpile(source: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("greet.ruchy");
    std::fs::write(&src, source).expect("write source");
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
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn test_ruchy_343_println_fstring_param_without_call_is_string() {
    let rust = transpile(GREET_NO_CALL);
    assert!(
        rust.contains("fn greet(who: String)"),
        "expected `who: String` for a println f-string parameter with no call site, got:\n{rust}"
    );
}

#[test]
fn test_ruchy_343_println_fstring_param_without_call_builds() {
    let rust = transpile(GREET_NO_CALL);
    let dir = tempfile::tempdir().expect("tempdir");
    let rs = dir.path().join("greet.rs");
    std::fs::write(&rs, &rust).expect("write rust");
    let build = StdCommand::new("rustc")
        .args([
            "--edition=2021",
            "--crate-type=lib",
            "--emit=metadata",
            "-o",
        ])
        .arg(dir.path().join("libgreet.rmeta"))
        .arg(&rs)
        .output()
        .expect("rustc runs");
    assert!(
        build.status.success(),
        "generated code does not build:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );
}
