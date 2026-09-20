//! RHL-0 corpus gate — holds `docs/rhl/corpus/` to the shape RHL-001 §8
//! pre-registers: exactly 40 task directories, exactly 10 of them marked
//! `ambiguous: true`, and every task's `tests.rs` a real (if not yet
//! compilable — see `HARNESS.md`) Rust file that names no arm-specific
//! symbol. See `docs/rhl/corpus/HARNESS.md` for the fixed interface
//! contract these tests are written against, and falsifier **F8**
//! (`docs/specifications/ruchy-high-level-language-interface.md` §7) for
//! why the contract must be fixed before either arm of the A/B exists.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::repo_root;

const EXPECTED_TASK_COUNT: usize = 40;
const EXPECTED_AMBIGUOUS_COUNT: usize = 10;
const FORBIDDEN_IDENTIFIERS: [&str; 4] = ["Transpiler", "Parser", "rhl", "ruchy"];

#[derive(Debug, Deserialize)]
struct TaskYaml {
    #[serde(default)]
    #[allow(dead_code)]
    id: String,
    #[serde(default)]
    #[allow(dead_code)]
    title: String,
    ambiguous: bool,
    entrypoint: String,
    #[serde(default)]
    ambiguity: Option<String>,
}

fn corpus_dir() -> PathBuf {
    repo_root().join("docs/rhl/corpus")
}

/// Every `NN-slug` task directory directly under `docs/rhl/corpus/`, sorted
/// lexically (which is numeric order, since `NN` is always two digits).
fn task_dirs() -> Vec<PathBuf> {
    let corpus = corpus_dir();
    let mut dirs: Vec<PathBuf> = fs::read_dir(&corpus)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", corpus.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();
    dirs
}

fn read_required(dir: &Path, name: &str) -> String {
    let path = dir.join(name);
    let content =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("missing {}: {e}", path.display()));
    assert!(!content.trim().is_empty(), "{} is empty", path.display());
    content
}

fn parse_task_yaml(dir: &Path) -> TaskYaml {
    let content = read_required(dir, "task.yaml");
    serde_yaml::from_str(&content)
        .unwrap_or_else(|e| panic!("{}/task.yaml does not parse: {e}", dir.display()))
}

#[test]
fn test_rhl_corpus_0_has_exactly_40_numbered_directories_no_gaps_no_dupes() {
    let dirs = task_dirs();
    assert_eq!(
        dirs.len(),
        EXPECTED_TASK_COUNT,
        "expected {EXPECTED_TASK_COUNT} task directories, found {}",
        dirs.len()
    );

    let mut names = Vec::with_capacity(dirs.len());
    for (index, dir) in dirs.iter().enumerate() {
        let expected_prefix = format!("{:02}-", index + 1);
        let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        assert!(
            name.starts_with(&expected_prefix),
            "directory {name} should start with {expected_prefix} \
             (task directories must be numbered 01..{EXPECTED_TASK_COUNT} with no gaps or duplicates)"
        );
        names.push(name.to_string());
    }
    let mut unique = names.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        names.len(),
        "duplicate task directory numbers found among {names:?}"
    );
}

#[test]
fn test_rhl_corpus_0_has_exactly_10_ambiguous_tasks_with_reasons() {
    let mut ambiguous_count = 0;
    for dir in task_dirs() {
        let task = parse_task_yaml(&dir);
        assert_eq!(
            task.entrypoint,
            "run",
            "{}/task.yaml must declare entrypoint: run",
            dir.display()
        );
        if task.ambiguous {
            ambiguous_count += 1;
            let ambiguity = task.ambiguity.unwrap_or_default();
            assert!(
                !ambiguity.trim().is_empty(),
                "{}/task.yaml is ambiguous: true but has no non-empty ambiguity: explanation",
                dir.display()
            );
        }
    }
    assert_eq!(
        ambiguous_count, EXPECTED_AMBIGUOUS_COUNT,
        "expected exactly {EXPECTED_AMBIGUOUS_COUNT} ambiguous tasks, found {ambiguous_count}"
    );
}

#[test]
fn test_rhl_corpus_0_every_task_has_three_nonempty_files() {
    for dir in task_dirs() {
        read_required(&dir, "task.yaml");
        read_required(&dir, "intent.en");
        read_required(&dir, "tests.rs");
    }
}

#[test]
fn test_rhl_corpus_0_every_tests_rs_parses_as_real_rust_with_a_test_fn() {
    for dir in task_dirs() {
        let content = read_required(&dir, "tests.rs");
        let parsed = syn::parse_file(&content)
            .unwrap_or_else(|e| panic!("{}/tests.rs does not parse as Rust: {e}", dir.display()));
        let has_test_fn = parsed.items.iter().any(|item| {
            let syn::Item::Fn(item_fn) = item else {
                return false;
            };
            item_fn
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("test"))
        });
        assert!(
            has_test_fn,
            "{}/tests.rs parses but has no #[test] function",
            dir.display()
        );
    }
}

#[test]
fn test_rhl_corpus_0_tests_rs_names_no_arm_specific_symbol() {
    for dir in task_dirs() {
        let content = read_required(&dir, "tests.rs");
        for forbidden in FORBIDDEN_IDENTIFIERS {
            assert!(
                !content.contains(forbidden),
                "{}/tests.rs names forbidden arm-specific symbol {forbidden:?}; \
                 a task's tests.rs may reference only run, TaskCtx, Report and \
                 TaskError (see HARNESS.md)",
                dir.display()
            );
        }
    }
}

#[test]
fn test_rhl_corpus_0_harness_md_declares_the_contract_types() {
    let path = corpus_dir().join("HARNESS.md");
    let content =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("missing {}: {e}", path.display()));
    assert!(!content.trim().is_empty(), "{} is empty", path.display());
    for name in ["TaskCtx", "Report", "TaskError"] {
        assert!(
            content.contains(name),
            "HARNESS.md must name the harness type {name}"
        );
    }
}

/// A task's `tests.rs` must actually drive the entrypoint. A file that parses as
/// Rust and holds a `#[test]` fn but never calls `run` asserts nothing about
/// either arm, and the structural checks above would not notice.
#[test]
fn test_rhl_corpus_0_every_tests_rs_actually_calls_the_entrypoint() {
    let mut silent = Vec::new();
    for dir in task_dirs() {
        let src =
            std::fs::read_to_string(dir.join("tests.rs")).expect("RHL-0: unreadable tests.rs");
        if !src.contains("run(") {
            silent.push(dir.file_name().unwrap().to_string_lossy().to_string());
        }
    }
    assert!(
        silent.is_empty(),
        "RHL-0: {} corpus task(s) have tests that never call `run`: {silent:?}. \
         A test that does not drive the entrypoint scores neither arm of F8's A/B.",
        silent.len()
    );
}

/// Every host and repo an `intent.en` names must be set up in that task's
/// `tests.rs`. Found by pre-PR review on task 01, whose intent said "on
/// runner-01" while its `TaskCtx` registered no host at all: a path-A solution
/// that checks the host exists would refuse and fail, while a path-B solution
/// that ignored the host would pass. That is an arm advantage hidden in a
/// fixture, and F8 would have measured it as a win for RHL.
#[test]
fn test_rhl_corpus_0_every_named_host_and_repo_is_set_up_in_the_tests() {
    let host = regex_lite_hosts;
    let mut missing = Vec::new();
    for dir in task_dirs() {
        let intent =
            std::fs::read_to_string(dir.join("intent.en")).expect("RHL-0: unreadable intent.en");
        let tests =
            std::fs::read_to_string(dir.join("tests.rs")).expect("RHL-0: unreadable tests.rs");
        for name in host(&intent) {
            if !tests.contains(&name) {
                missing.push(format!(
                    "{}: intent names `{name}`, tests.rs never mentions it",
                    dir.file_name().unwrap().to_string_lossy()
                ));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "RHL-0: {} task(s) name an entity the tests never set up: {missing:#?}. \
         An arm that checks for it fails while an arm that ignores it passes, so \
         F8 would measure the fixture instead of the language.",
        missing.len()
    );
}

/// Hosts (`runner-01`) and repos (`paiml/fleet-ops`) as they appear in prose.
/// Deliberately conservative: it recognises only these two shapes, so it cannot
/// flag ordinary English.
fn regex_lite_hosts(intent: &str) -> Vec<String> {
    intent
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '/' || c == '_'))
        .filter(|t| is_host(t) || is_repo(t))
        .map(|t| t.to_string())
        .collect()
}

fn is_host(t: &str) -> bool {
    let Some((head, tail)) = t.rsplit_once('-') else {
        return false;
    };
    is_slug(head) && !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit())
}

fn is_repo(t: &str) -> bool {
    let Some((owner, name)) = t.split_once('/') else {
        return false;
    };
    is_slug(owner) && is_slug(name)
}

/// A lowercase-and-hyphen slug, as fleet hosts and repo names are written.
fn is_slug(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase() || c == '-')
}
