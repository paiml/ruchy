//! README contract, binary half (PMAT-254, `contracts/ruchy-readme-v1.yaml`).
//!
//! `src/readme_contract.rs` extracts README.md into
//! `contracts/ruchy-readme-claims.json` and proves that file fresh. This target
//! reads the claims and runs them against the `ruchy` binary cargo built from
//! the same commit: every example prints its `.expected` output, every Quick
//! start command exits 0, and every Commands-table entry is a real subcommand.

use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn claims() -> Value {
    let text = std::fs::read_to_string(root().join("contracts/ruchy-readme-claims.json"))
        .expect("contracts/ruchy-readme-claims.json exists");
    serde_json::from_str(&text).expect("claims file is JSON")
}

fn list(key: &str) -> Vec<String> {
    claims()[key]
        .as_array()
        .unwrap_or_else(|| panic!("claims.{key} is a list"))
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect()
}

fn ruchy(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ruchy").expect("ruchy binary is built");
    cmd.current_dir(dir)
        .timeout(std::time::Duration::from_secs(120));
    cmd
}

/// Split a README command line into argv; double quotes group words.
fn argv(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, part) in line.split('"').enumerate() {
        if i % 2 == 1 {
            out.push(part.to_string());
        } else {
            out.extend(part.split_whitespace().map(String::from));
        }
    }
    out
}

#[test]
fn test_pmat_254_examples_stdout_matches_expected() {
    let examples = list("examples");
    assert!(
        !examples.is_empty(),
        "FALSIFY-RUCHY-README-007: no examples claimed (vacuous)"
    );
    for path in examples {
        let expected = std::fs::read_to_string(root().join(path.replace(".ruchy", ".expected")))
            .unwrap_or_else(|e| panic!("{path}: .expected: {e}"));
        let out = ruchy(&root()).arg(&path).output().expect("spawn ruchy");
        assert!(
            out.status.success(),
            "FALSIFY-RUCHY-README-007: `ruchy {path}` failed: {out:?}"
        );
        assert_eq!(
            String::from_utf8_lossy(&out.stdout),
            expected,
            "FALSIFY-RUCHY-README-007: {path} stdout"
        );
    }
}

/// After `ruchy compile <src> -o <bin>`, the binary must print `<src>`'s expected output.
fn check_compiled(dir: &Path, args: &[String]) {
    let (Some(src), Some(bin)) = (args.get(1), args.iter().skip_while(|a| *a != "-o").nth(1))
    else {
        panic!("FALSIFY-RUCHY-README-008: compile line needs <src> -o <bin>: {args:?}");
    };
    let expected = std::fs::read_to_string(root().join(src.replace(".ruchy", ".expected")))
        .expect(".expected");
    let out = std::process::Command::new(dir.join(bin))
        .output()
        .expect("run compiled binary");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        expected,
        "FALSIFY-RUCHY-README-008: compiled {src}"
    );
}

#[test]
fn test_pmat_254_quick_start_and_commands_run() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dst = dir.path().join("examples/readme");
    std::fs::create_dir_all(&dst).expect("mkdir");
    for e in std::fs::read_dir(root().join("examples/readme")).expect("examples/readme") {
        let e = e.expect("entry");
        std::fs::copy(e.path(), dst.join(e.file_name())).expect("copy example");
    }
    for line in list("shell_commands") {
        let args: Vec<String> = argv(&line).into_iter().skip(1).collect();
        let out = ruchy(dir.path()).args(&args).output().expect("spawn ruchy");
        assert!(
            out.status.success(),
            "FALSIFY-RUCHY-README-008: `{line}` failed: {out:?}"
        );
        if args.first().map(String::as_str) == Some("compile") {
            check_compiled(dir.path(), &args);
        }
    }
    let help = ruchy(dir.path())
        .arg("--help")
        .output()
        .expect("ruchy --help");
    let help = String::from_utf8_lossy(&help.stdout).to_string();
    for cmd in list("commands") {
        let listed = help
            .lines()
            .any(|l| l.split_whitespace().next() == Some(cmd.as_str()));
        assert!(
            listed,
            "FALSIFY-RUCHY-README-008: `ruchy {cmd}` is not in `ruchy --help`"
        );
        ruchy(dir.path())
            .args([cmd.as_str(), "--help"])
            .assert()
            .success();
    }
}

#[test]
fn test_pmat_254_bare_ruchy_starts_repl() {
    let readme = std::fs::read_to_string(root().join("README.md")).expect("README.md");
    assert!(
        readme.contains("Running `ruchy` with no arguments starts the REPL."),
        "README REPL sentence moved"
    );
    let out = ruchy(&root())
        .write_stdin("1 + 1\n")
        .output()
        .expect("spawn ruchy");
    assert!(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .any(|l| l.trim() == "2"),
        "REPL did not evaluate 1 + 1: {out:?}"
    );
}
