#![allow(missing_docs)]
//! G2B-S2 (RHLGA-1): no CLI verb may silently succeed without doing its job.
//!
//! The command router used to end in `_ => { eprintln!("Command not yet
//! implemented"); Ok(()) }`, so any `Commands` variant it did not route (e.g.
//! `prove`) printed that line and exited 0. These tests walk every verb that
//! `ruchy --help` lists (recursing into nested command groups): each verb's
//! `--help` must exit 0 without the stub line, and each verb that is safe to run
//! offline on a trivial file is run on one: it must either work or exit non-zero,
//! and a verb still running at the timeout fails. Servers, REPLs, network and
//! package verbs are not run for real, so the test is hermetic.

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

/// Verbs that run offline on a trivial file, without servers, network, a
/// registry or a REPL; only these are run for real.
const SAFE_OFFLINE_VERBS: [&str; 11] = [
    "parse",
    "check",
    "transpile",
    "ast",
    "lint",
    "fmt",
    "score",
    "tier",
    "provability",
    "prove",
    "explain",
];

/// Combined stdout and stderr of a finished command.
fn output_text(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// `Some(report)` when `ruchy <verb> --help` fails or prints the stub marker.
fn broken_help(verb: &[String]) -> Option<String> {
    let output = ruchy_cmd()
        .args(verb)
        .arg("--help")
        .timeout(PER_VERB_TIMEOUT)
        .output();
    let Ok(output) = output else {
        return Some(format!("`ruchy {} --help` timed out", verb.join(" ")));
    };
    let stub = output_text(&output).to_lowercase().contains(STUB_MARKER);
    (!output.status.success() || stub)
        .then(|| format!("`ruchy {} --help` failed or is a stub", verb.join(" ")))
}

/// `None` when the verb works (exit 0 without the stub marker) or fails
/// loudly (non-zero exit). `Some(report)` when it exits 0 while printing the
/// stub marker, or is still running at the timeout.
fn silent_stub(verb: &[String], dir: &Path) -> Option<String> {
    let output = ruchy_cmd()
        .current_dir(dir)
        .args(verb)
        .arg("main.ruchy")
        .write_stdin("")
        .timeout(PER_VERB_TIMEOUT)
        .output();
    let Ok(output) = output else {
        return Some(format!("`ruchy {}` timed out", verb.join(" ")));
    };
    let is_stub =
        output.status.success() && output_text(&output).to_lowercase().contains(STUB_MARKER);
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
fn test_g2b_s2_every_verb_has_a_working_help() {
    let verbs = leaf_verbs(&[]);
    assert!(
        verbs.len() > 40,
        "expected the full verb tree, got {verbs:?}"
    );
    let failures: Vec<String> = verbs.iter().filter_map(|verb| broken_help(verb)).collect();
    assert!(failures.is_empty(), "broken help:\n{}", failures.join("\n"));
}

#[test]
fn test_g2b_s2_no_safe_verb_silently_succeeds_as_a_stub() {
    let verbs = leaf_verbs(&[]);
    let safe: Vec<Vec<String>> = SAFE_OFFLINE_VERBS
        .iter()
        .map(|name| vec![(*name).to_string()])
        .collect();
    for verb in &safe {
        assert!(verbs.contains(verb), "`ruchy --help` should list {verb:?}");
    }
    let failures: Vec<String> = safe
        .iter()
        .filter_map(|verb| silent_stub(verb, scratch_dir().path()))
        .collect();
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
