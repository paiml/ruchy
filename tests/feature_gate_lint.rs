#![allow(missing_docs)]
//! Feature-gate lint for the integration test tree (PMAT-112, PMAT-113).
//!
//! The gate is `cargo clippy --all-targets --no-default-features --features minimal`
//! together with `cargo clippy --all-targets --all-features -- -D warnings`.
//! These tests are the fast discriminator: they name the offending file and
//! line in milliseconds instead of after a full feature-matrix build.
//!
//! Detection works on source with comments and string literals stripped, and
//! on identifier tokens that are declared under `src/runtime/repl`, so a glob
//! or multi-line `use` does not evade it, a mention in a comment does not
//! trigger it, and unrelated names such as `wasm::ReplOutput` do not count.

use std::fs;
use std::path::{Path, PathBuf};

const REPL_GATE: &str = "#[cfg(feature = \"repl\")]";

fn crate_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn sources_in(dir: &str) -> Vec<(PathBuf, String)> {
    let mut files: Vec<PathBuf> = fs::read_dir(crate_path(dir))
        .expect("source directory is readable")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name != "feature_gate_lint.rs")
        })
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let src = fs::read_to_string(&path).expect("source is readable");
            (path, src)
        })
        .collect()
}

fn stem_of(path: &Path) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// One line of code with its `//` comment and the contents of its string
/// literals removed (escaped quotes inside a literal are honoured).
fn code_only(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    let mut in_string = false;
    while let Some(c) = chars.next() {
        match (in_string, c) {
            (true, '\\') => {
                chars.next();
            }
            (true, '"') => in_string = false,
            (true, _) => {}
            (false, '"') => in_string = true,
            (false, '/') if chars.peek() == Some(&'/') => break,
            (false, _) => out.push(c),
        }
    }
    out
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The source with every `/* ... */` block comment blanked out; newlines are
/// kept so line numbers still refer to the original file.
fn without_block_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after.find("*/").map_or(after.len(), |n| n + 2);
        out.extend(after[..end].chars().filter(|c| *c == '\n'));
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

/// `pub struct|enum|trait|type Repl…` on one line, if any.
fn declared_repl_type(line: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix("pub ")?;
    let rest = ["struct ", "enum ", "trait ", "type "]
        .iter()
        .find_map(|keyword| rest.strip_prefix(keyword))?;
    let name: String = rest.chars().take_while(|c| is_ident_char(*c)).collect();
    name.starts_with("Repl").then_some(name)
}

fn collect_repl_types(dir: &Path, names: &mut Vec<String>) {
    for entry in fs::read_dir(dir)
        .expect("src/runtime/repl is readable")
        .flatten()
    {
        let path = entry.path();
        if path.is_dir() {
            collect_repl_types(&path, names);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let src = fs::read_to_string(&path).expect("module source is readable");
            names.extend(src.lines().filter_map(declared_repl_type));
        }
    }
}

/// Public `Repl…` types declared under `src/runtime/repl`, the feature-gated
/// module. Names from elsewhere (`wasm::ReplOutput`, `WasmRepl`) are absent.
fn repl_type_names() -> Vec<String> {
    let mut names = Vec::new();
    collect_repl_types(&crate_path("src/runtime/repl"), &mut names);
    names.sort();
    names.dedup();
    names
}

/// The identifier that starts at byte `idx` of `code`, if `idx` starts one.
fn identifier_at(code: &str, idx: usize) -> Option<&str> {
    let before = code[..idx].chars().next_back();
    if before.is_some_and(is_ident_char) {
        return None;
    }
    let end = code[idx..]
        .find(|c: char| !is_ident_char(c))
        .map_or(code.len(), |n| idx + n);
    Some(&code[idx..end])
}

/// A line that pulls the REPL into the target: a `runtime::repl` path or a
/// whole-word use of a type declared in that module.
fn line_references_repl(line: &str, names: &[String]) -> bool {
    let code = code_only(line);
    code.contains("runtime::repl")
        || code
            .match_indices("Repl")
            .filter_map(|(idx, _)| identifier_at(&code, idx))
            .any(|word| names.iter().any(|name| name == word))
}

/// A `cfg` attribute, plain or inside `all(...)`, that names the `repl` feature.
fn names_repl_feature(attribute: &str) -> bool {
    attribute.starts_with('#')
        && attribute.contains("cfg(")
        && attribute.contains("feature = \"repl\"")
}

/// A crate-level `#![cfg(feature = "...")]`, or `#![cfg(all(..., feature = "..."))]`,
/// keeps the whole target out of the `minimal` build, whichever feature it names.
fn has_file_level_feature_gate(src: &str) -> bool {
    src.lines()
        .any(|line| line.starts_with("#![cfg(") && line.contains("feature = \""))
}

/// Index of the column-0 `fn` header that encloses line `idx`, if any.
fn enclosing_fn_header(lines: &[&str], idx: usize) -> Option<usize> {
    (0..=idx).rev().find(|&i| {
        let line = lines[i];
        !line.starts_with(char::is_whitespace) && (line.starts_with("fn ") || line.contains(" fn "))
    })
}

/// The outer attribute block directly above `header` carries the REPL gate.
fn attributes_gate(lines: &[&str], header: usize) -> bool {
    (0..header)
        .rev()
        .map(|i| lines[i].trim())
        .take_while(|line| line.starts_with("#["))
        .any(names_repl_feature)
}

/// An item-level `#[cfg(feature = "repl")]`: either directly above the line
/// (a gated `use` item) or on the function whose body contains it.
fn item_gated_above(lines: &[&str], idx: usize) -> bool {
    let directly_above = idx > 0 && names_repl_feature(lines[idx - 1].trim());
    directly_above
        || enclosing_fn_header(lines, idx).is_some_and(|header| attributes_gate(lines, header))
}

fn ungated_repl_uses(path: &Path, src: &str, names: &[String]) -> Vec<String> {
    if has_file_level_feature_gate(src) {
        return Vec::new();
    }
    let src = without_block_comments(src);
    let lines: Vec<&str> = src.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line_references_repl(line, names))
        .filter(|(idx, _)| !item_gated_above(&lines, *idx))
        .map(|(idx, line)| format!("{}:{}: {}", path.display(), idx + 1, line.trim()))
        .collect()
}

/// Index of the first line of the statement that ends at `idx`: the line after
/// the nearest earlier line that closes a statement or block, or is blank.
fn statement_start(lines: &[&str], idx: usize) -> usize {
    (0..idx)
        .rev()
        .find(|&i| {
            let code = code_only(lines[i]);
            let code = code.trim();
            code.is_empty() || code.ends_with(';') || code.ends_with('{') || code.ends_with('}')
        })
        .map_or(0, |i| i + 1)
}

/// A statement that ends in `.assert();` and neither binds, returns nor
/// otherwise consumes the `assert_cmd::assert::Assert` it produces discards a
/// `#[must_use]` value: a clippy error under `-D warnings` once the target's
/// feature is on. Only `assert_cmd` chains count (mockito's `Mock::assert`
/// returns unit).
fn is_bare_assert_statement(lines: &[&str], idx: usize) -> bool {
    let compact: String = code_only(lines[idx]).split_whitespace().collect();
    if !compact.ends_with(".assert();") {
        return false;
    }
    let statement: String = lines[statement_start(lines, idx)..=idx]
        .iter()
        .map(|line| code_only(line))
        .collect::<Vec<_>>()
        .join("\n");
    let from_assert_cmd = statement.contains("ruchy_cmd(")
        || statement.contains("cargo_bin")
        || statement.contains("Command::");
    let consumed = statement.contains("let ")
        || statement.contains(" = ")
        || statement.trim_start().starts_with("return ");
    from_assert_cmd && !consumed
}

fn bare_assert_statements(path: &Path, src: &str) -> Vec<String> {
    let src = without_block_comments(src);
    let lines: Vec<&str> = src.lines().collect();
    (0..lines.len())
        .filter(|&idx| is_bare_assert_statement(&lines, idx))
        .map(|idx| format!("{}:{}: {}", path.display(), idx + 1, lines[idx].trim()))
        .collect()
}

fn manifest() -> String {
    fs::read_to_string(crate_path("Cargo.toml")).expect("Cargo.toml is readable")
}

/// Value of `key = ...` inside one manifest table, or empty.
fn toml_field(block: &str, key: &str) -> String {
    block
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix(key))
        .filter_map(|rest| rest.trim_start().strip_prefix('='))
        .map(|value| value.trim().to_string())
        .next()
        .unwrap_or_default()
}

/// `[[example]]` / `[[bench]]` / `[[test]]` / `[[bin]]` tables as
/// (name, required-features) pairs.
fn manifest_targets(manifest: &str, kind: &str) -> Vec<(String, String)> {
    manifest
        .split(&format!("[[{kind}]]"))
        .skip(1)
        .map(|block| block.split("\n[").next().unwrap_or_default())
        .map(|block| {
            (
                toml_field(block, "name"),
                toml_field(block, "required-features"),
            )
        })
        .collect()
}

fn target_requires_repl(manifest: &str, kind: &str, stem: &str) -> bool {
    manifest_targets(manifest, kind)
        .iter()
        .any(|(name, required)| name.trim_matches('"') == stem && required.contains("\"repl\""))
}

fn repl_targets_without_required_features(
    manifest: &str,
    names: &[String],
    dir: &str,
    kind: &str,
) -> Vec<String> {
    sources_in(dir)
        .iter()
        .filter(|(_, src)| {
            without_block_comments(src)
                .lines()
                .any(|line| line_references_repl(line, names))
        })
        .map(|(path, _)| stem_of(path))
        .filter(|stem| !target_requires_repl(manifest, kind, stem))
        .map(|stem| {
            format!("{dir}/{stem}.rs: add [[{kind}]] name = \"{stem}\" with required-features = [\"repl\"]")
        })
        .collect()
}

/// A test target is gated either in source (`cfg`) or in the manifest
/// (`[[test]]` with `required-features`).
#[test]
fn test_pmat_113_repl_tests_are_feature_gated() {
    let names = repl_type_names();
    let manifest = manifest();
    let violations: Vec<String> = sources_in("tests")
        .iter()
        .filter(|(path, _)| !target_requires_repl(&manifest, "test", &stem_of(path)))
        .flat_map(|(path, src)| ungated_repl_uses(path, src, &names))
        .collect();
    assert!(
        violations.is_empty(),
        "REPL-dependent tests must carry `{REPL_GATE}` (crate-level `#!` form or on the item) \
         or a [[test]] required-features entry, so that `--no-default-features --features minimal` \
         compiles:\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_pmat_112_no_bare_assert_statements_in_tests() {
    let violations: Vec<String> = sources_in("tests")
        .iter()
        .flat_map(|(path, src)| bare_assert_statements(path, src))
        .collect();
    assert!(
        violations.is_empty(),
        "bare `.assert();` statements discard a must_use value; wrap them in \
         `support::assert_args_accepted(..)` or assert on the result:\n{}",
        violations.join("\n")
    );
}

/// Examples and benches cannot be cfg-gated (an empty crate has no `main`),
/// so a REPL-dependent one must be declared with `required-features` and is
/// then skipped, not broken, under `--features minimal`.
#[test]
fn test_pmat_113_repl_examples_and_benches_declare_required_features() {
    let manifest = manifest();
    let names = repl_type_names();
    let mut violations =
        repl_targets_without_required_features(&manifest, &names, "examples", "example");
    violations.extend(repl_targets_without_required_features(
        &manifest, &names, "benches", "bench",
    ));
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

/// `minimal` is the core library only; the binary is a REPL-driven CLI, so it
/// must say so instead of failing to compile under `--features minimal`.
#[test]
fn test_pmat_113_ruchy_binary_requires_the_repl_feature() {
    assert!(
        target_requires_repl(&manifest(), "bin", "ruchy"),
        "[[bin]] ruchy must declare required-features = [\"repl\"]"
    );
}

/// And the default build must still produce the binary.
#[test]
fn test_pmat_113_default_features_enable_repl() {
    let manifest = manifest();
    let batteries = manifest
        .lines()
        .find(|line| line.starts_with("batteries-included ="))
        .expect("batteries-included feature");
    let default = manifest
        .lines()
        .find(|line| line.starts_with("default ="))
        .expect("default feature");
    assert!(default.contains("\"batteries-included\""), "{default}");
    assert!(batteries.contains("\"repl\""), "{batteries}");
}

#[test]
fn test_pmat_113_repl_type_names_come_from_the_gated_module() {
    let names = repl_type_names();
    for expected in ["Repl", "ReplState", "ReplMode"] {
        assert!(
            names.iter().any(|n| n == expected),
            "{expected} missing from {names:?}"
        );
    }
    assert!(
        !names.iter().any(|n| n == "ReplOutput"),
        "wasm::ReplOutput must not count"
    );
}

#[test]
fn test_pmat_113_lint_discriminates_gated_from_ungated_repl_uses() {
    let path = Path::new("x.rs");
    let names = vec!["Repl".to_string(), "ReplState".to_string()];
    let ungated = "use ruchy::runtime::Repl;\n#[test]\nfn t() {}\n";
    assert_eq!(ungated_repl_uses(path, ungated, &names).len(), 1);
    let file_gated = format!("#![cfg(feature = \"repl\")]\n{ungated}");
    assert!(ungated_repl_uses(path, &file_gated, &names).is_empty());
    let all_form = "#![cfg(all(not(target_arch = \"wasm32\"), feature = \"repl\"))]\nuse ruchy::runtime::Repl;\n";
    assert!(ungated_repl_uses(path, all_form, &names).is_empty());
    let body_use = "#[cfg(feature = \"repl\")]\n#[test]\nfn t() {\n    let a = 1;\n    let b = 2;\n    let c = 3;\n    let s = ReplState::new();\n}\n";
    assert!(ungated_repl_uses(path, body_use, &names).is_empty());
    let item_all_form =
        "#[cfg(all(unix, feature = \"repl\"))]\n#[test]\nfn t() {\n    let r = Repl::new(d);\n}\n";
    assert!(ungated_repl_uses(path, item_all_form, &names).is_empty());
    let next_fn = "#[cfg(feature = \"repl\")]\n#[test]\nfn gated() {}\n\n#[test]\nfn open() {\n    let s = ReplState::new();\n}\n";
    assert_eq!(ungated_repl_uses(path, next_fn, &names).len(), 1);
    let top_level = "#[test]\nfn t() {}\n\nuse ruchy::runtime::Repl;\n";
    assert_eq!(ungated_repl_uses(path, top_level, &names).len(), 1);
}

#[test]
fn test_pmat_113_lint_sees_through_import_spellings_and_ignores_prose() {
    let path = Path::new("x.rs");
    let names = vec!["Repl".to_string(), "ReplState".to_string()];
    let multi_line = "use ruchy::runtime::{\n    Repl,\n    Value,\n};\n";
    assert_eq!(ungated_repl_uses(path, multi_line, &names).len(), 1);
    let glob_then_use =
        "use ruchy::runtime::*;\nfn t() {\n    let r = Repl::new(std::env::temp_dir());\n}\n";
    assert_eq!(ungated_repl_uses(path, glob_then_use, &names).len(), 1);
    let self_import =
        "use ruchy::runtime::{self};\nfn t() {\n    runtime::repl::Repl::new(d);\n}\n";
    assert_eq!(ungated_repl_uses(path, self_import, &names).len(), 1);
    let prose = "// the runtime::Repl is not used here\nlet s = \"use ruchy::runtime::Repl;\";\nlet x = Replay::new();\nlet o: ReplOutput = WasmRepl::new();\n";
    assert!(ungated_repl_uses(path, prose, &names).is_empty());
    let block_comment =
        "fn t() {\n    /*\n    let repl = Repl::new(d);\n    */\n    let live = Repl::new(d);\n}\n";
    let hits = ungated_repl_uses(path, block_comment, &names);
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(hits[0].starts_with("x.rs:5:"), "{hits:?}");
}

#[test]
fn test_pmat_112_lint_discriminates_bare_from_consumed_asserts() {
    let bare = [
        "fn t() {",
        "    ruchy_cmd()",
        "        .arg(\"x\")",
        "        .assert(); // times out",
        "}",
    ];
    assert!(is_bare_assert_statement(&bare, 3));
    let spaced = ["fn t() {", "    ruchy_cmd().arg(\"x\").assert() ;", "}"];
    assert!(is_bare_assert_statement(&spaced, 1));
    let bound = [
        "fn t() {",
        "    let out = ruchy_cmd()",
        "        .arg(\"x\")",
        "        .assert();",
        "}",
    ];
    assert!(!is_bare_assert_statement(&bound, 3));
    let returned = [
        "fn t() -> Assert {",
        "    return ruchy_cmd()",
        "        .assert();",
        "}",
    ];
    assert!(!is_bare_assert_statement(&returned, 2));
    let wrapped = [
        "    assert_args_accepted(",
        "        ruchy_cmd()",
        "            .assert(),",
        "    );",
    ];
    assert!(!is_bare_assert_statement(&wrapped, 2));
    let mockito = [
        "    let mock = server.mock(\"GET\", \"/\");",
        "    mock.assert();",
    ];
    assert!(!is_bare_assert_statement(&mockito, 1));
    let in_string = ["    let s = \"cmd.assert();\";"];
    assert!(!is_bare_assert_statement(&in_string, 0));
}

#[test]
fn test_pmat_113_manifest_target_parser_reads_name_and_required_features() {
    let manifest = "[[test]]\nname = \"a\"\nrequired-features = [\"repl\"]\n\n[[test]]\nname = \"b\"\n\n[dependencies]\nx = \"1\"\n";
    assert!(target_requires_repl(manifest, "test", "a"));
    assert!(!target_requires_repl(manifest, "test", "b"));
    assert!(!target_requires_repl(manifest, "bench", "a"));
}
