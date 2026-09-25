//! README contract (PMAT-254, `contracts/ruchy-readme-v1.yaml`).
//!
//! README.md makes claims (a version, an MSRV, sections, examples, commands,
//! links). This module is the one extractor that turns those claims into
//! `contracts/ruchy-readme-claims.json`, the entity the contract's SHACL shape
//! grades under `pv lint contracts --gate shapes`, and it resolves each claim
//! against the tree. It lives in `src/` because CI runs `cargo test --lib`
//! only; the checks that need the built binary are in
//! `tests/readme_contract.rs`.
//!
//! Regenerate the claims file after editing the README:
//! `RUCHY_README_BLESS=1 cargo test --lib readme_contract`.

use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const CLAIMS: &str = "contracts/ruchy-readme-claims.json";
const CONTRACT: &str = "contracts/ruchy-readme-v1.yaml";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// One fenced block: its info string, body, and the last non-blank line before it.
struct Fence {
    lang: String,
    body: String,
    lead: String,
}

/// Claims read from the README text, before serialization.
#[derive(Default)]
struct Claims {
    version: Vec<String>,
    msrv: Vec<String>,
    sections: Vec<String>,
    examples: Vec<String>,
    commands: Vec<String>,
    shell_commands: Vec<String>,
    links: Vec<String>,
}

fn fences(text: &str) -> Vec<Fence> {
    let mut out = Vec::new();
    let mut open: Option<Fence> = None;
    let mut lead = String::new();
    for line in text.lines() {
        match open.as_mut() {
            Some(_) if line.starts_with("```") => out.extend(open.take()),
            Some(f) => {
                f.body.push_str(line);
                f.body.push('\n');
            }
            None if line.starts_with("```") => {
                let lang = line.trim_start_matches('`').trim().to_string();
                open = Some(Fence {
                    lang,
                    body: String::new(),
                    lead: lead.clone(),
                });
            }
            None if !line.trim().is_empty() => lead = line.to_string(),
            None => {}
        }
    }
    out
}

/// Lines outside fenced blocks.
fn prose_lines(text: &str) -> Vec<&str> {
    let mut inside = false;
    let mut out = Vec::new();
    for line in text.lines() {
        if line.starts_with("```") {
            inside = !inside;
        } else if !inside {
            out.push(line);
        }
    }
    out
}

fn backticked(s: &str) -> Vec<&str> {
    s.split('`').skip(1).step_by(2).collect()
}

fn capture_all(re: &str, text: &str) -> Vec<String> {
    let re = regex::Regex::new(re).expect("static regex");
    re.captures_iter(text).map(|c| c[1].to_string()).collect()
}

fn extract_prose(prose: &[&str], c: &mut Claims) {
    let mut section = String::new();
    for line in prose {
        if let Some(h) = line.strip_prefix("## ") {
            section = h.trim().to_string();
            c.sections.push(section.clone());
        } else if section == "MSRV" {
            c.msrv.extend(capture_all(r"\*\*([0-9.]+)\*\*", line));
        } else if section == "Commands" && line.starts_with("| `ruchy ") {
            c.commands
                .extend(capture_all(r"^\| `ruchy ([a-z0-9-]+)`", line));
        }
    }
    let joined = prose.join("\n");
    c.links.extend(capture_all(r"\]\(([^)#:]+)\)", &joined));
    c.links.extend(capture_all(r##"src="([^"#:]+)""##, &joined));
}

fn extract_fence(f: &Fence, c: &mut Claims) {
    match f.lang.as_str() {
        "ruchy" => c
            .examples
            .extend(backticked(&f.lead).iter().map(|s| (*s).to_string())),
        "bash" => {
            for line in f.body.lines() {
                c.version
                    .extend(capture_all(r"^cargo install ruchy --version (\S+)$", line));
                if line.starts_with("ruchy ") {
                    c.shell_commands.push(line.to_string());
                }
            }
        }
        _ => {}
    }
}

fn extract(text: &str) -> Claims {
    let mut c = Claims::default();
    extract_prose(&prose_lines(text), &mut c);
    for f in fences(text) {
        extract_fence(&f, &mut c);
    }
    c
}

fn scalar_or_list(v: &[String]) -> Value {
    match v {
        [one] => json!(one),
        many => json!(many),
    }
}

fn to_json(c: &Claims) -> Value {
    json!({
        "version": scalar_or_list(&c.version),
        "msrv": scalar_or_list(&c.msrv),
        "sections": c.sections,
        "examples": c.examples,
        "commands": c.commands,
        "shell_commands": c.shell_commands,
        "links": c.links,
    })
}

fn values_of(claims: &Value, key: &str) -> Vec<String> {
    match claims.get(key) {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => Vec::new(),
    }
}

/// Grade one `shape.properties[]` row; returns the violations.
fn grade_property(prop: &serde_yaml::Value, claims: &Value) -> Vec<String> {
    let key = prop["path"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches("readme:");
    let vals = values_of(claims, key);
    let mut bad = Vec::new();
    if let Some(min) = prop["minCount"].as_u64() {
        if (vals.len() as u64) < min {
            bad.push(format!("{key}: {} value(s), minCount {min}", vals.len()));
        }
    }
    if let Some(max) = prop["maxCount"].as_u64() {
        if vals.len() as u64 > max {
            bad.push(format!("{key}: {} value(s), maxCount {max}", vals.len()));
        }
    }
    if let Some(p) = prop["pattern"].as_str() {
        let re = regex::Regex::new(p).expect("shape pattern compiles");
        bad.extend(
            vals.iter()
                .filter(|v| !re.is_match(v))
                .map(|v| format!("{key}: {v:?} !~ /{p}/")),
        );
    }
    if let Some(allowed) = prop["in"].as_sequence() {
        let allowed: Vec<&str> = allowed
            .iter()
            .filter_map(serde_yaml::Value::as_str)
            .collect();
        bad.extend(
            vals.iter()
                .filter(|v| !allowed.contains(&v.as_str()))
                .map(|v| format!("{key}: {v:?} not in set")),
        );
    }
    bad
}

/// The contract's shape, graded in-process so `cargo test --lib` enforces it
/// where `pv` is not installed. `pv lint --gate shapes` grades the same block.
fn grade_shape(contract: &serde_yaml::Value, claims: &Value) -> Vec<String> {
    let props = contract["shape"]["properties"]
        .as_sequence()
        .cloned()
        .unwrap_or_default();
    let declared: Vec<String> = props
        .iter()
        .filter_map(|p| p["path"].as_str())
        .map(|p| p.trim_start_matches("readme:").to_string())
        .collect();
    let mut bad: Vec<String> = props
        .iter()
        .flat_map(|p| grade_property(p, claims))
        .collect();
    let keys = claims
        .as_object()
        .map(|o| o.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    bad.extend(
        keys.into_iter()
            .filter(|k| !declared.contains(k))
            .map(|k| format!("closed: {k}")),
    );
    bad
}

fn fresh_claims() -> Value {
    to_json(&extract(&read("README.md")))
}

fn committed_claims() -> Value {
    serde_json::from_str(&read(CLAIMS)).expect("claims file is JSON")
}

#[test]
fn test_pmat_254_claims_json_is_fresh() {
    let fresh = fresh_claims();
    if std::env::var_os("RUCHY_README_BLESS").is_some() {
        let text = serde_json::to_string_pretty(&fresh).expect("serialize") + "\n";
        std::fs::write(root().join(CLAIMS), text).expect("write claims");
    }
    assert_eq!(
        committed_claims(),
        fresh,
        "FALSIFY-RUCHY-README-001: {CLAIMS} is stale; run RUCHY_README_BLESS=1 cargo test --lib readme_contract"
    );
}

#[test]
fn test_pmat_254_version_and_msrv_match_cargo() {
    let c = committed_claims();
    assert_eq!(
        values_of(&c, "version"),
        [env!("CARGO_PKG_VERSION")],
        "FALSIFY-RUCHY-README-002: version"
    );
    assert_eq!(
        values_of(&c, "msrv"),
        [env!("CARGO_PKG_RUST_VERSION")],
        "FALSIFY-RUCHY-README-002: msrv"
    );
}

fn example_files() -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(root().join("examples/readme"))
        .expect("examples/readme exists")
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| Path::new(n).extension().is_some_and(|x| x == "ruchy"))
        .map(|n| format!("examples/readme/{n}"))
        .collect();
    v.sort();
    v
}

fn evaluate(src: &str) -> Result<(), String> {
    let mut interp = crate::runtime::interpreter::Interpreter::new();
    interp
        .eval_string(src)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[test]
fn test_pmat_254_examples_match_files_and_evaluate() {
    let readme = read("README.md");
    let fenced: Vec<Fence> = fences(&readme)
        .into_iter()
        .filter(|f| f.lang == "ruchy")
        .collect();
    let mut cited: Vec<String> = values_of(&fresh_claims(), "examples");
    assert_eq!(
        cited.len(),
        fenced.len(),
        "FALSIFY-RUCHY-README-003: every ruchy fence names exactly one example file"
    );
    for (path, fence) in cited.iter().zip(&fenced) {
        assert_eq!(
            read(path),
            fence.body,
            "FALSIFY-RUCHY-README-003: README block for {path} differs from the file"
        );
        let expected = path.replace(".ruchy", ".expected");
        assert!(
            root().join(&expected).is_file(),
            "FALSIFY-RUCHY-README-003: {expected} missing"
        );
        evaluate(&fence.body)
            .unwrap_or_else(|e| panic!("FALSIFY-RUCHY-README-003: {path} does not evaluate: {e}"));
    }
    cited.sort();
    assert_eq!(
        cited,
        example_files(),
        "FALSIFY-RUCHY-README-003: examples/readme/ and the README cite different files"
    );
}

#[test]
fn test_pmat_254_links_resolve() {
    for link in values_of(&committed_claims(), "links") {
        assert!(
            root().join(&link).exists(),
            "FALSIFY-RUCHY-README-004: README links {link}, which does not exist"
        );
    }
}

#[test]
fn test_pmat_254_claims_conform_to_shape() {
    let contract: serde_yaml::Value =
        serde_yaml::from_str(&read(CONTRACT)).expect("contract is YAML");
    let props = contract["shape"]["properties"]
        .as_sequence()
        .map_or(0, Vec::len);
    assert!(
        props > 0,
        "FALSIFY-RUCHY-README-005: the shape declares no properties (vacuous)"
    );
    let bad = grade_shape(&contract, &committed_claims());
    assert!(
        bad.is_empty(),
        "FALSIFY-RUCHY-README-005: shape violations: {bad:#?}"
    );
}

#[test]
fn test_pmat_254_no_hand_typed_counts() {
    let prose = prose_lines(&read("README.md")).join("\n");
    let re = regex::Regex::new(r"(?i)\b[0-9][0-9,]*\+?\s+(tests?|passing|features|examples|commands|points?)\b|tests-[0-9]")
        .expect("static regex");
    let hits: Vec<&str> = re.find_iter(&prose).map(|m| m.as_str()).collect();
    assert!(
        hits.is_empty(),
        "FALSIFY-RUCHY-README-006: hand-typed counts in README: {hits:?}"
    );
}

#[test]
fn test_pmat_254_shape_rejects_planted_claims() {
    // Positive control: the in-process grader must refuse known-bad claims,
    // or a green run above means nothing.
    let contract: serde_yaml::Value =
        serde_yaml::from_str(&read(CONTRACT)).expect("contract is YAML");
    let mut c = committed_claims();
    c["version"] = json!("five");
    c["sections"] = json!(["Features"]);
    c["undeclared"] = json!(1);
    let bad = grade_shape(&contract, &c);
    assert!(
        bad.len() >= 4,
        "planted claims must be refused, got {bad:?}"
    );
}
