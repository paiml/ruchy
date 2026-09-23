//! RHL-4: `ruchy transpile` and `ruchy compile` on `.rhl` / `.rhl.yaml`
//! files, through the built binary (spec RHL-001 §5: extend, do not collide).
//!
//! The library decides (src/rhl/cli.rs, src/rhl/lower.rs, `cargo test --lib`);
//! this file shows what only a process can: the verb dispatch, the exit
//! codes, and that the Rust of an RHL file is byte for byte what
//! `ruchy transpile` makes of its lowered ruchy.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

const VALID: &str = "docs/rhl/breaks/v2/valid/01-gx10-disk-watch.rhl";
const TYPO: &str = "docs/rhl/breaks/planted/typo-term/01-gx10-disk-watch/broken.rhl";

fn stdout_of(args: &[&str]) -> String {
    let out = ruchy_cmd()
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(out).expect("utf-8")
}

/// A scratch root holding copies of `vocab/` and `contracts/` (files only).
fn scratch_root() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    for sub in ["vocab", "contracts"] {
        std::fs::create_dir_all(dir.path().join(sub)).expect("mkdir");
        for e in std::fs::read_dir(sub)
            .expect("read dir")
            .filter_map(Result::ok)
        {
            if e.path().is_file() {
                std::fs::copy(e.path(), dir.path().join(sub).join(e.file_name())).expect("copy");
            }
        }
    }
    dir
}

#[test]
fn test_rhl4_cli_transpile_emit_ruchy_prints_the_lowering() {
    ruchy_cmd()
        .args(["transpile", VALID, "--emit", "ruchy"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "fun decide(facts: Facts) -> Vec<Action>",
        ))
        .stdout(predicate::str::contains("struct Facts {"));
}

#[test]
fn test_rhl4_cli_transpile_default_is_the_rust_of_the_lowered_ruchy() {
    let rust = stdout_of(&["transpile", VALID]);
    assert!(
        rust.contains("fn decide(facts: Facts) -> Vec<Action>"),
        "{rust}"
    );
    let dir = tempfile::tempdir().expect("tempdir");
    let ruchy = dir.path().join("j.ruchy");
    std::fs::write(&ruchy, stdout_of(&["transpile", VALID, "--emit", "ruchy"])).expect("write");
    let via_ruchy = stdout_of(&["transpile", ruchy.to_str().expect("utf-8 path")]);
    assert_eq!(
        rust, via_ruchy,
        "the .rhl path must be `ruchy transpile` of its lowering"
    );
}

#[test]
fn test_rhl4_cli_transpile_writes_output_and_yaml_matches_text() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("j.rs");
    ruchy_cmd()
        .args(["transpile", VALID, "-o", out.to_str().expect("path")])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
    let written = std::fs::read_to_string(&out).expect("written");
    let root = scratch_root();
    let yaml = root.path().join("j.rhl.yaml");
    ruchy_cmd()
        .args(["convert", VALID, "-o", yaml.to_str().expect("path")])
        .assert()
        .success();
    assert_eq!(
        stdout_of(&["transpile", yaml.to_str().expect("path")]),
        written
    );
}

#[test]
fn test_rhl4_cli_transpile_declines_a_typo_and_an_unlowerable_construct() {
    ruchy_cmd()
        .args(["transpile", TYPO])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("RHL-V002"));
    let root = scratch_root();
    let file = root.path().join("g.rhl");
    std::fs::write(
        &file,
        "use vocabulary fleet v2\n\njob \"g\"\n  give back 1\nend\n",
    )
    .expect("write");
    ruchy_cmd()
        .args(["transpile", file.to_str().expect("path")])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("RHL-L001"));
}

#[test]
fn test_rhl4_cli_emit_is_for_rhl_and_compile_waits_for_rhl_4b() {
    ruchy_cmd()
        .args(["transpile", VALID, "--emit", "c"])
        .assert()
        .code(2);
    let dir = tempfile::tempdir().expect("tempdir");
    let ruchy = dir.path().join("x.ruchy");
    std::fs::write(&ruchy, "fun f() -> i64 { 1 }\n").expect("write");
    ruchy_cmd()
        .args([
            "transpile",
            ruchy.to_str().expect("path"),
            "--emit",
            "ruchy",
        ])
        .assert()
        .failure();
    ruchy_cmd()
        .args(["compile", VALID])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("RHL-4b"));
    assert!(Path::new(VALID).is_file());
}
