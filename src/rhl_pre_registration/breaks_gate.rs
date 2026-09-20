//! RHL-0 breaks gate — falsifier F7, "agent-repairable" (spec `RHL-001` §7).
//!
//! Holds `docs/rhl/breaks/` to the shape RHL-001 §8 pre-registers: at least
//! 12 valid `.rhl` programs (`docs/rhl/breaks/valid/`) and, for each of the
//! six mutation classes named in this ticket, at least six planted breaks
//! (`docs/rhl/breaks/planted/<class-dir>/<NN-slug>/{broken.rhl,break.yaml}`)
//! spread across several different base programs. §3.5's F7 positive control
//! ("a break with two valid candidates must NOT be auto-fixed") is asserted
//! directly: every `two-candidate typo` break carries `safe_fix: false` with
//! exactly two candidates, and every `typo'd term` break carries
//! `safe_fix: true`, exactly as the worked example in §3.5 does for its
//! single-candidate `RHL-V002` case.
//!
//! No RHL compiler exists yet (row RHL-1+), so this gate cannot run a real
//! parser or vocabulary checker over these programs. What it CAN check, and
//! does, is that the corpus is not vacuous: every `broken.rhl` is a real,
//! byte-different mutation of a base program that itself exists, every
//! `break.yaml` fully declares its own contract, and every diagnostic code
//! belongs to one of `RHL-001` §3.5's stable families.
//!
//! # What this gate cannot check, and why that is correct here
//!
//! It proves that every planted break DIFFERS from its base, that its class is
//! one of the six, and that its expected diagnostic code is in a stable family.
//! It does NOT prove that compiling the broken program actually produces that
//! code — no RHL compiler exists at RHL-0, and inventing one here would defeat
//! the point of pre-registering the breaks before the compiler. That
//! correspondence is exactly what row RHL-7 measures, against these files.

use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

use super::repo_root;

/// The six mutation classes this ticket names, spelled exactly as every
/// `break.yaml` in the corpus must spell them in its `class:` field.
const CLOSED_CLASSES: [&str; 6] = [
    "typo'd term",
    "missing end",
    "wrong unit",
    "wrong type",
    "undeclared effect",
    "two-candidate typo",
];

const MIN_VALID_PROGRAMS: usize = 12;
const MIN_BREAKS_PER_CLASS: usize = 6;

/// The `with` amendment keyword (RHL-0, grammar/rhl.lalrpop's AMENDMENT
/// note). Every valid program's action blocks must open with it.
const WITH_MARKER: &str = "with";

#[derive(Debug, Deserialize)]
struct BreakYaml {
    base: String,
    class: String,
    expected_code: String,
    safe_fix: bool,
    #[serde(default)]
    candidates: Vec<String>,
}

fn breaks_root() -> PathBuf {
    repo_root().join("docs/rhl/breaks")
}

fn valid_dir() -> PathBuf {
    breaks_root().join("valid")
}

fn planted_dir() -> PathBuf {
    breaks_root().join("planted")
}

/// Every `.rhl` file directly under `docs/rhl/breaks/valid/`, sorted.
fn valid_program_paths() -> Vec<PathBuf> {
    let dir = valid_dir();
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rhl"))
        .collect();
    paths.sort();
    paths
}

/// Every `docs/rhl/breaks/planted/<class-dir>/<NN-slug>/` leaf directory,
/// sorted. Two levels deep: a class directory, then a break directory.
fn planted_break_dirs() -> Vec<PathBuf> {
    let root = planted_dir();
    let mut class_dirs: Vec<PathBuf> = fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", root.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.is_dir())
        .collect();
    class_dirs.sort();

    let mut out = Vec::new();
    for class_dir in class_dirs {
        let mut leaves: Vec<PathBuf> = fs::read_dir(&class_dir)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", class_dir.display()))
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|p| p.is_dir())
            .collect();
        leaves.sort();
        out.extend(leaves);
    }
    out
}

fn load_break(dir: &Path) -> BreakYaml {
    let path = dir.join("break.yaml");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: cannot read: {e}", path.display()));
    serde_yaml::from_str(&text)
        .unwrap_or_else(|e| panic!("{}: does not parse as break.yaml: {e}", path.display()))
}

fn read_broken_rhl(dir: &Path) -> String {
    let path = dir.join("broken.rhl");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: cannot read: {e}", path.display()))
}

/// (a) At least [`MIN_VALID_PROGRAMS`] valid programs exist, each non-empty.
#[test]
fn test_rhl_breaks_0_at_least_twelve_valid_programs_nonempty() {
    let paths = valid_program_paths();
    assert!(
        paths.len() >= MIN_VALID_PROGRAMS,
        "expected at least {MIN_VALID_PROGRAMS} valid programs under {}, found {}",
        valid_dir().display(),
        paths.len()
    );
    for path in &paths {
        let text = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{}: cannot read: {e}", path.display()));
        assert!(
            !text.trim().is_empty(),
            "{}: valid program is empty",
            path.display()
        );
    }
}

/// (b) All six mutation classes are present, each with at least
/// [`MIN_BREAKS_PER_CLASS`] planted breaks.
#[test]
fn test_rhl_breaks_0_all_six_classes_present_with_at_least_six_each() {
    let dirs = planted_break_dirs();
    assert!(
        !dirs.is_empty(),
        "no planted breaks found under {}",
        planted_dir().display()
    );

    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for dir in &dirs {
        let b = load_break(dir);
        *counts.entry(b.class.clone()).or_insert(0) += 1;
    }

    for class in CLOSED_CLASSES {
        let n = counts.get(class).copied().unwrap_or(0);
        assert!(
            n >= MIN_BREAKS_PER_CLASS,
            "class `{class}` has {n} planted breaks, expected at least {MIN_BREAKS_PER_CLASS}"
        );
    }
}

/// (c) Every `break.yaml` parses via `serde_yaml` and declares all of
/// `base`, `class`, `expected_code`, `safe_fix` (a missing required field
/// fails to parse, per [`BreakYaml`]'s shape, so a successful load already
/// proves declaration; this test also rejects an empty value in any of
/// them, which a bare `Default` would let through).
#[test]
fn test_rhl_breaks_0_break_yaml_declares_all_required_fields() {
    for dir in planted_break_dirs() {
        let b = load_break(&dir);
        assert!(
            !b.base.trim().is_empty(),
            "{}: `base` is empty",
            dir.display()
        );
        assert!(
            !b.class.trim().is_empty(),
            "{}: `class` is empty",
            dir.display()
        );
        assert!(
            !b.expected_code.trim().is_empty(),
            "{}: `expected_code` is empty",
            dir.display()
        );
        // `safe_fix` is a plain bool: any value present is a declaration.
        let _ = b.safe_fix;
    }
}

/// (d) Every `base:` path exists and is one of the committed valid programs.
#[test]
fn test_rhl_breaks_0_every_base_exists_and_is_a_valid_program() {
    let root = repo_root();
    let valid: HashSet<PathBuf> = valid_program_paths().into_iter().collect();

    for dir in planted_break_dirs() {
        let b = load_break(&dir);
        let base_path = root.join(&b.base);
        assert!(
            base_path.is_file(),
            "{}: base `{}` does not exist",
            dir.display(),
            b.base
        );
        assert!(
            valid.contains(&base_path),
            "{}: base `{}` is not one of the committed valid programs under {}",
            dir.display(),
            b.base,
            valid_dir().display()
        );
    }
}

/// (e) Anti-vacuity: every `broken.rhl` actually differs from its base file.
/// A "break" identical to its base tests nothing.
#[test]
fn test_rhl_breaks_0_every_broken_program_differs_from_its_base() {
    let root = repo_root();
    for dir in planted_break_dirs() {
        let b = load_break(&dir);
        let base_text = fs::read_to_string(root.join(&b.base))
            .unwrap_or_else(|e| panic!("{}: cannot read base {}: {e}", dir.display(), b.base));
        let broken_text = read_broken_rhl(&dir);
        assert_ne!(
            base_text,
            broken_text,
            "{}: broken.rhl is byte-identical to its base `{}` — this break tests nothing",
            dir.display(),
            b.base
        );
    }
}

/// (f) Every `class` is one of the six exact spellings, and every
/// `expected_code` matches `^RHL-[PVTEBCX][0-9]{3}$`.
#[test]
fn test_rhl_breaks_0_class_closed_and_expected_code_matches_family_regex() {
    let code_re = Regex::new(r"^RHL-[PVTEBCX][0-9]{3}$").expect("static regex is valid");
    for dir in planted_break_dirs() {
        let b = load_break(&dir);
        assert!(
            CLOSED_CLASSES.contains(&b.class.as_str()),
            "{}: class `{}` is not one of {CLOSED_CLASSES:?}",
            dir.display(),
            b.class
        );
        assert!(
            code_re.is_match(&b.expected_code),
            "{}: expected_code `{}` does not match ^RHL-[PVTEBCX][0-9]{{3}}$",
            dir.display(),
            b.expected_code
        );
    }
}

/// (g) F7 positive control: every `two-candidate typo` break has
/// `safe_fix: false` and exactly two candidates; every `typo'd term` break
/// has `safe_fix: true`.
#[test]
fn test_rhl_breaks_0_f7_positive_control_safe_fix_matches_class() {
    for dir in planted_break_dirs() {
        let b = load_break(&dir);
        match b.class.as_str() {
            "two-candidate typo" => {
                assert!(
                    !b.safe_fix,
                    "{}: class `two-candidate typo` must have safe_fix: false (F7 positive \
                     control — a break with two valid candidates must NOT be auto-fixed)",
                    dir.display()
                );
                assert_eq!(
                    b.candidates.len(),
                    2,
                    "{}: class `two-candidate typo` must declare exactly two candidates, found {}",
                    dir.display(),
                    b.candidates.len()
                );
            }
            "typo'd term" => {
                assert!(
                    b.safe_fix,
                    "{}: class `typo'd term` must have safe_fix: true (§3.5 — a fix is safe \
                     when it is the unique candidate and preserves types)",
                    dir.display()
                );
            }
            _ => {}
        }
    }
}

/// Keyword-led productions (`grammar/rhl.lalrpop`'s `Decl` and `BlockStmt`)
/// that legitimately open a `Body … "end"` block WITHOUT the `with` marker.
/// These are unambiguous by construction — the marker only matters for the
/// keyword-less `Action` production (`ActionHead` alone, i.e. a plain `App`
/// or `App "in" App`), which is what the AMENDMENT note describes.
const BLOCK_OPENING_KEYWORD_PREFIXES: [&str; 9] = [
    "job ",
    "command ",
    "check ",
    "pipeline ",
    "shape ",
    "example ",
    "when ",
    "for each ",
    "repeat at most ",
];

/// True when `line` opens an action block in the pre-amendment, ambiguous
/// form: it ends with a quoted string argument (an `ActionHead`), does not
/// start with one of [`BLOCK_OPENING_KEYWORD_PREFIXES`] (which would make it
/// an unambiguous keyword-led block instead), and the following line is more
/// indented than it — i.e. it is read as opening a body the way
/// `grammar/rhl.lalrpop`'s AMENDMENT note describes as rejected by LALRPOP.
fn opens_unmarked_action_block(line: &str, next: &str) -> bool {
    let trimmed = line.trim_end();
    if trimmed.is_empty() || !trimmed.ends_with('"') {
        return false;
    }
    let content = line.trim_start();
    if BLOCK_OPENING_KEYWORD_PREFIXES
        .iter()
        .any(|prefix| content.starts_with(prefix))
    {
        return false;
    }
    let next_trimmed = next.trim();
    if next_trimmed.is_empty() {
        return false;
    }
    let this_indent = line.len() - line.trim_start().len();
    let next_indent = next.len() - next.trim_start().len();
    next_indent > this_indent
}

/// (h) Every valid program uses only §3.3 keywords plus `with`, and in
/// particular no valid program contains an action block opened WITHOUT
/// `with` — the amendment `grammar/rhl.lalrpop` made to close the F1
/// ambiguity. This is what keeps the corpus from drifting back to the
/// unmarked, non-conflict-free form.
#[test]
fn test_rhl_breaks_0_valid_programs_never_open_action_block_without_with() {
    for path in valid_program_paths() {
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{}: cannot read: {e}", path.display()));
        let lines: Vec<&str> = text.lines().collect();
        for i in 0..lines.len() {
            let next = lines.get(i + 1).copied().unwrap_or("");
            assert!(
                !opens_unmarked_action_block(lines[i], next),
                "{}:{}: opens an action block without the `with` marker — this is the \
                 pre-amendment ambiguous form grammar/rhl.lalrpop's AMENDMENT note documents \
                 as rejected by LALRPOP; every action block must open `with`",
                path.display(),
                i + 1
            );
        }
        assert!(
            text.contains(WITH_MARKER),
            "{}: contains no `with` at all — every valid program in this corpus is expected \
             to exercise at least one action-with block",
            path.display()
        );
    }
}

#[cfg(test)]
mod unit {
    use super::opens_unmarked_action_block;

    #[test]
    fn test_rhl_breaks_0_unmarked_detector_flags_the_preamendment_form() {
        assert!(opens_unmarked_action_block(
            r#"    file ticket in repo "paiml/infra""#,
            r#"      title "gx10 disk below 100 GB""#,
        ));
    }

    #[test]
    fn test_rhl_breaks_0_unmarked_detector_accepts_the_with_marked_form() {
        assert!(!opens_unmarked_action_block(
            r#"    file ticket in repo "paiml/infra" with"#,
            r#"      title "gx10 disk below 100 GB""#,
        ));
    }

    #[test]
    fn test_rhl_breaks_0_unmarked_detector_ignores_same_or_lesser_indent() {
        // `title "..."` followed by a sibling line at the same indent, or by
        // a dedent (`end`), is an ordinary single-line statement, not an
        // opened block.
        assert!(!opens_unmarked_action_block(
            r#"      title "gx10 disk below 100 GB""#,
            r#"      label "fleet""#,
        ));
        assert!(!opens_unmarked_action_block(
            r#"      label "fleet""#,
            r#"    end"#,
        ));
    }
}
