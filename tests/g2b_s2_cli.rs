#![allow(missing_docs)]
//! G2B-S2 (RHLGA-1): no CLI verb may silently succeed without doing its job.
//!
//! The command router used to end in `_ => { eprintln!("Command not yet
//! implemented"); Ok(()) }`, so any `Commands` variant it did not route (e.g.
//! `prove`) printed that line and exited 0. These tests walk every verb that
//! `ruchy --help` lists (recursing into nested command groups) and run it on a
//! trivial input: it must either work or exit non-zero.

use assert_cmd::Command;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

const STUB_MARKER: &str = "not yet implemented";
const TRIVIAL_SOURCE: &str = "let x = 42\nprintln(x)\n";
const PER_VERB_TIMEOUT: Duration = Duration::from_secs(30);

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Subcommand names from the `Commands:` section of a clap help text.
fn subcommands_in_help(help: &str) -> Vec<String> {
    help.lines()
        .skip_while(|line| line.trim() != "Commands:")
        .skip(1)
        .take_while(|line| line.starts_with("  "))
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| *name != "help")
        .map(str::to_string)
        .collect()
}

fn help_of(path: &[String]) -> String {
    let output = ruchy_cmd()
        .args(path)
        .arg("--help")
        .output()
        .expect("run ruchy --help");
    assert!(output.status.success(), "`ruchy {path:?} --help` failed");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Every leaf verb path, e.g. `["prove"]`, `["infra", "destroy"]`.
fn leaf_verbs(prefix: &[String]) -> Vec<Vec<String>> {
    let children = subcommands_in_help(&help_of(prefix));
    if children.is_empty() {
        return vec![prefix.to_vec()];
    }
    children
        .into_iter()
        .flat_map(|child| {
            let mut path = prefix.to_vec();
            path.push(child);
            leaf_verbs(&path)
        })
        .collect()
}

/// `None` when the verb works (exit 0 without the stub marker), is still
/// running at the timeout (servers, REPLs), or fails loudly (non-zero exit).
/// `Some(report)` when it exits 0 while printing the stub marker.
fn silent_stub(verb: &[String], dir: &Path) -> Option<String> {
    let output = match ruchy_cmd()
        .current_dir(dir)
        .args(verb)
        .arg("main.ruchy")
        .write_stdin("")
        .timeout(PER_VERB_TIMEOUT)
        .output()
    {
        Ok(output) => output,
        Err(_) => return None,
    };
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let is_stub = output.status.success() && text.to_lowercase().contains(STUB_MARKER);
    is_stub.then(|| {
        format!(
            "`ruchy {}` exited 0 printing {STUB_MARKER:?}",
            verb.join(" ")
        )
    })
}

fn scratch_dir() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    std::fs::write(dir.path().join("main.ruchy"), TRIVIAL_SOURCE).expect("write main.ruchy");
    dir
}

#[test]
fn test_g2b_s2_help_lists_the_expected_verbs() {
    let verbs = subcommands_in_help(&help_of(&[]));
    for expected in ["prove", "infra", "serve", "run", "transpile"] {
        assert!(
            verbs.iter().any(|v| v == expected),
            "`ruchy --help` should list `{expected}`: {verbs:?}"
        );
    }
}

#[test]
fn test_g2b_s2_no_verb_silently_succeeds_as_a_stub() {
    let verbs = leaf_verbs(&[]);
    assert!(
        verbs.len() > 40,
        "expected the full verb tree, got {verbs:?}"
    );
    let failures: Vec<String> = std::thread::scope(|scope| {
        let handles: Vec<_> = verbs
            .chunks(verbs.len().div_ceil(8))
            .map(|chunk| {
                scope.spawn(move || {
                    chunk
                        .iter()
                        .filter_map(|verb| silent_stub(verb, scratch_dir().path()))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().expect("verb worker panicked"))
            .collect()
    });
    assert!(
        failures.is_empty(),
        "silent stubs:\n{}",
        failures.join("\n")
    );
}

#[test]
fn test_g2b_s2_prove_check_is_routed_to_the_prover() {
    let dir = scratch_dir();
    let output = ruchy_cmd()
        .current_dir(dir.path())
        .args(["prove", "main.ruchy", "--check"])
        .write_stdin("")
        .timeout(PER_VERB_TIMEOUT)
        .output()
        .expect("ruchy prove");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.to_lowercase().contains(STUB_MARKER),
        "prove must reach the prover, got stderr: {stderr}"
    );
    assert_eq!(output.status.code(), Some(0), "stderr: {stderr}");
}

#[test]
fn test_g2b_s2_prove_missing_file_fails() {
    let dir = TempDir::new().expect("tempdir");
    ruchy_cmd()
        .current_dir(dir.path())
        .args(["prove", "missing.ruchy", "--check"])
        .write_stdin("")
        .timeout(PER_VERB_TIMEOUT)
        .assert()
        .failure();
}
