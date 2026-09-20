//! RHL-0 manifest gate — tamper-evidence for the pre-registration itself.
//!
//! RHL-0 exists so that F1–F9 cannot be tuned to fit an implementation: the
//! corpus, the breaks, the vocabularies and the grammar are committed *before*
//! any compiler code. Three independent reviewers, unanimously, pointed out that
//! the other four gates assert only STRUCTURE — 40 directories, 6 mutation
//! classes, a contract per term — so the CONTENT could still be quietly edited
//! during RHL-1..RHL-12 until the falsifiers passed, and every structural gate
//! would stay green.
//!
//! This gate closes that. `docs/rhl/PREREGISTRATION.sha256` records the sha256 of
//! every pre-registered file. The point is not to make editing impossible — it is
//! to make editing IMPOSSIBLE TO DO QUIETLY. A row that legitimately revises an
//! artifact must revise this manifest in the same commit, and that line of diff
//! is the audit trail.
//!
//! Regenerate after a deliberate change:
//!
//! ```text
//! RHL0_WRITE_MANIFEST=1 cargo test --lib -p ruchy rhl_manifest_0_regenerate -- --ignored
//! ```

use super::repo_root;
use sha2::{Digest, Sha256};
use std::path::Path;

const MANIFEST: &str = "docs/rhl/PREREGISTRATION.sha256";

/// Every root whose bytes RHL-0 pre-registers. A file under one of these that
/// the manifest does not mention is a finding, not an oversight.
const ROOTS: &[&str] = &[
    "grammar/rhl.lalrpop",
    "grammar/rhl.conflict-report.txt",
    "grammar/fixtures",
    "vocab",
    "docs/rhl/corpus",
    "docs/rhl/breaks",
    // Governance, not just artifacts: the spec states the falsifiers, and the
    // bindings state what was measured at HEAD. Quietly editing either redefines
    // what "success" means, which is the same failure as tuning the corpus.
    // Amending them is fine and expected -- doing it without a manifest diff is not.
    "docs/specifications/ruchy-high-level-language-interface.md",
    "docs/rhl/phase0-bindings.md",
];

/// Term contracts are pre-registered too, but `contracts/` also holds this
/// repository's pre-existing process contracts, which RHL-0 does not own.
const CONTRACT_PREFIX: &str = "contracts/rhl-";

fn sha256_of_file(p: &Path) -> String {
    let bytes =
        std::fs::read(p).unwrap_or_else(|e| panic!("RHL-0: cannot read {}: {e}", p.display()));
    let mut h = Sha256::new();
    h.update(&bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Every pre-registered file, repo-relative, sorted — so the manifest is
/// byte-identical on any host and a diff shows only real changes.
fn pre_registered_files() -> Vec<String> {
    let root = repo_root();
    let mut out = Vec::new();
    for r in ROOTS {
        for e in walkdir::WalkDir::new(root.join(r)).sort_by_file_name() {
            let e = e.unwrap_or_else(|err| panic!("RHL-0: cannot walk {r}: {err}"));
            if e.file_type().is_file() {
                out.push(rel(&root, e.path()));
            }
        }
    }
    for e in std::fs::read_dir(root.join("contracts")).expect("RHL-0: contracts/ is missing") {
        let p = e.expect("RHL-0: unreadable contracts/ entry").path();
        let r = rel(&root, &p);
        if r.starts_with(CONTRACT_PREFIX) {
            out.push(r);
        }
    }
    out.sort();
    out
}

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .unwrap_or_else(|_| panic!("RHL-0: {} escaped the repo root", p.display()))
        .to_string_lossy()
        .replace('\\', "/")
}

fn render_manifest(files: &[String]) -> String {
    let root = repo_root();
    let mut s = String::from(
        "# RHL-0 pre-registration manifest (spec RHL-001, row RHL-0).\n\
         #\n\
         # sha256 of every artifact RHL-0 pre-registered, so that a later row cannot\n\
         # quietly edit the corpus, the breaks, the grammar or a vocabulary until the\n\
         # falsifiers pass. Editing is allowed; editing UNNOTICED is not. A row that\n\
         # revises an artifact revises this file in the same commit, and that diff is\n\
         # the audit trail.\n\
         #\n\
         # Regenerate: RHL0_WRITE_MANIFEST=1 cargo test --lib -p ruchy \\\n\
         #             rhl_manifest_0_regenerate -- --ignored\n\
         #\n",
    );
    s.push_str(&format!("# files: {}\n\n", files.len()));
    for f in files {
        s.push_str(&format!("{}  {f}\n", sha256_of_file(&root.join(f))));
    }
    s
}

/// Parse the manifest into (path, sha) pairs, ignoring comments and blanks.
fn parse_manifest(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter(|l| !l.trim_start().starts_with('#') && !l.trim().is_empty())
        .filter_map(|l| l.split_once("  "))
        .map(|(sha, path)| (path.trim().to_string(), sha.trim().to_string()))
        .collect()
}

#[test]
fn test_rhl_manifest_0_covers_every_pre_registered_file() {
    let text = std::fs::read_to_string(repo_root().join(MANIFEST))
        .unwrap_or_else(|e| panic!("RHL-0: cannot read {MANIFEST}: {e}"));
    let listed: Vec<String> = parse_manifest(&text).into_iter().map(|(p, _)| p).collect();
    let actual = pre_registered_files();

    let unlisted: Vec<&String> = actual.iter().filter(|f| !listed.contains(f)).collect();
    assert!(
        unlisted.is_empty(),
        "RHL-0: {} pre-registered file(s) are not in {MANIFEST}, so their bytes are \
         unprotected: {unlisted:?}. Regenerate the manifest in the same commit that \
         added them.",
        unlisted.len()
    );

    let missing: Vec<&String> = listed.iter().filter(|f| !actual.contains(f)).collect();
    assert!(
        missing.is_empty(),
        "RHL-0: {MANIFEST} lists {} file(s) that no longer exist: {missing:?}. A \
         pre-registered artifact was deleted; say so in the commit that deleted it.",
        missing.len()
    );
}

#[test]
fn test_rhl_manifest_0_every_recorded_hash_still_matches() {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(MANIFEST))
        .unwrap_or_else(|e| panic!("RHL-0: cannot read {MANIFEST}: {e}"));
    let drifted: Vec<String> = parse_manifest(&text)
        .into_iter()
        .filter(|(p, sha)| root.join(p).is_file() && &sha256_of_file(&root.join(p)) != sha)
        .map(|(p, _)| p)
        .collect();
    assert!(
        drifted.is_empty(),
        "RHL-0 PRE-REGISTRATION BROKEN: {} artifact(s) changed since RHL-0 committed \
         them, without {MANIFEST} being regenerated: {drifted:?}.\n\n\
         This is the exact failure the manifest exists to catch. Either the edit was \
         deliberate — in which case regenerate the manifest IN THE SAME COMMIT, so the \
         change is on the record — or an artifact was tuned to make a falsifier pass, \
         which is the one thing pre-registration forbids.",
        drifted.len()
    );
}

/// Not a gate: the regenerator, kept beside the gate so there is exactly one
/// definition of the manifest's format. `--ignored` plus an explicit env var,
/// so it can never rewrite the manifest during an ordinary test run.
#[test]
#[ignore = "writes docs/rhl/PREREGISTRATION.sha256; run deliberately"]
fn test_rhl_manifest_0_regenerate() {
    assert!(
        std::env::var("RHL0_WRITE_MANIFEST").is_ok(),
        "refusing to rewrite {MANIFEST} without RHL0_WRITE_MANIFEST=1"
    );
    let files = pre_registered_files();
    std::fs::write(repo_root().join(MANIFEST), render_manifest(&files))
        .expect("RHL-0: cannot write the manifest");
    println!("wrote {MANIFEST} covering {} files", files.len());
}
