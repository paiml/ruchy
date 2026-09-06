//! PMAT-136: the ruchy-dogfood skill's release gates are DECLARED, never copied.
//!
//! The dogfood protocol discovers a repo's own gates from
//! `[package.metadata.dogfood] gates` in `Cargo.toml` and runs each one with its
//! own receipt row. Two vacuity guards make discovery honest:
//!
//! * a declared script that does not exist is a *deleted* gate -> FAIL
//! * an empty (or missing) `gates` list is a clean sweep over an empty set -> FAIL
//!
//! These tests are the falsification harness for `contracts/dogfood-gates-v1.yaml`.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Gates declared here but owned by another in-flight ticket.
///
/// `scripts/release-policy.sh` is written by PMAT-135 on its own branch. Until
/// that branch merges the file is absent from this worktree, and the
/// declared-but-missing rule would fire on it for a reason that is not a defect
/// of this tree. The allowlist is deliberately narrow: one name, and
/// `test_pmat_136_pending_allowlist_is_empty_once_the_gate_lands` deletes the
/// exemption the moment the file appears.
const PENDING_FROM_OTHER_TICKETS: [&str; 1] = ["scripts/release-policy.sh"];

/// The crate root, independent of the caller's working directory.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The `gates` list from `[package.metadata.dogfood]`, parsed with the `toml` crate.
fn declared_gates() -> Vec<String> {
    let manifest = fs::read_to_string(repo_root().join("Cargo.toml")).expect("read Cargo.toml");
    let doc: toml::Value = toml::from_str(&manifest).expect("Cargo.toml is not valid TOML");
    let gates = doc
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("dogfood"))
        .and_then(|d| d.get("gates"))
        .expect("[package.metadata.dogfood] gates is missing from Cargo.toml");
    gates
        .as_array()
        .expect("[package.metadata.dogfood] gates must be an array")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("every declared gate must be a string")
                .to_string()
        })
        .collect()
}

/// True when the ticket that owns this gate has not merged yet.
fn is_pending_from_another_ticket(gate: &str) -> bool {
    PENDING_FROM_OTHER_TICKETS.contains(&gate)
}

#[test]
fn test_pmat_136_dogfood_gates_table_declares_a_non_empty_list() {
    let gates = declared_gates();
    assert!(
        !gates.is_empty(),
        "an empty gates list is a clean sweep over an empty set: declare the repo's gates"
    );
}

#[test]
fn test_pmat_136_every_declared_gate_exists_and_is_executable() {
    for gate in declared_gates() {
        if is_pending_from_another_ticket(&gate) {
            continue;
        }
        let path = repo_root().join(&gate);
        assert!(
            path.is_file(),
            "declared gate {gate} does not exist: a declared-but-missing gate is a deleted gate"
        );
        assert_executable(&path, &gate);
    }
}

#[cfg(unix)]
fn assert_executable(path: &std::path::Path, gate: &str) {
    use std::os::unix::fs::PermissionsExt;
    let mode = fs::metadata(path)
        .expect("stat declared gate")
        .permissions()
        .mode();
    assert!(
        mode & 0o111 != 0,
        "declared gate {gate} is not executable (mode {mode:o})"
    );
}

#[cfg(not(unix))]
fn assert_executable(_path: &std::path::Path, _gate: &str) {}

#[test]
fn test_pmat_136_pending_allowlist_is_empty_once_the_gate_lands() {
    for gate in PENDING_FROM_OTHER_TICKETS {
        assert!(
            !repo_root().join(gate).is_file(),
            "{gate} now exists: remove it from PENDING_FROM_OTHER_TICKETS so the \
             declared-but-missing rule covers it again (PMAT-135 has merged)"
        );
    }
}

/// The YAML frontmatter block of the skill, without its `---` fences.
fn skill_frontmatter() -> String {
    let path = repo_root().join(".claude/skills/ruchy-dogfood/SKILL.md");
    let text = fs::read_to_string(&path).expect("read .claude/skills/ruchy-dogfood/SKILL.md");
    let body = text
        .strip_prefix("---\n")
        .expect("SKILL.md must open with a --- frontmatter fence");
    let end = body
        .find("\n---\n")
        .expect("SKILL.md frontmatter must be closed by ---");
    body[..end].to_string()
}

#[test]
fn test_pmat_136_skill_frontmatter_names_the_skill_and_allows_agent() {
    let front = skill_frontmatter();
    assert!(
        front.lines().any(|l| l.trim() == "name: ruchy-dogfood"),
        "SKILL.md needs an explicit `name: ruchy-dogfood`: a directory named dogfood is \
         shadowed by the user-scope skill of the same name"
    );
    let tools = front
        .lines()
        .find(|l| l.starts_with("allowed-tools:"))
        .expect("SKILL.md frontmatter must declare allowed-tools");
    assert!(
        tools.contains("Agent"),
        "the protocol spawns three worker lanes, so Agent must be allowed: {tools}"
    );
}

#[test]
fn test_pmat_136_dogfood_gates_list_matches_the_declaration() {
    let output = Command::new("bash")
        .arg("scripts/dogfood-gates.sh")
        .arg("--list")
        .current_dir(repo_root())
        .output()
        .expect("run scripts/dogfood-gates.sh --list");
    assert!(
        output.status.success(),
        "dogfood-gates.sh --list exited {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let listed: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(std::string::ToString::to_string)
        .collect();
    assert_eq!(
        listed,
        declared_gates(),
        "--list must print exactly the gates declared in Cargo.toml"
    );
}

/// PMAT-136 (quorum on #220): a `]` or a quoted name inside a comment must neither
/// end the array early nor become a phantom gate.
#[test]
fn test_pmat_136_gate_discovery_ignores_comments_in_the_manifest() {
    let dir = std::env::temp_dir().join(format!("pmat136-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let manifest = dir.join("Cargo.toml");
    std::fs::write(
        &manifest,
        "[package]\nname = \"x\"\n[package.metadata.dogfood]\n# gates = [\"phantom\"] ]\ngates = [\n    \"scripts/a.sh\", # first ] not the end\n    \"scripts/b.sh\",\n]\n",
    )
    .expect("write manifest");
    let output = Command::new("bash")
        .arg("scripts/dogfood-gates.sh")
        .arg("--list")
        .arg("--manifest")
        .arg(&manifest)
        .output()
        .expect("run scripts/dogfood-gates.sh --list --manifest");
    let _ = std::fs::remove_dir_all(&dir);
    let listed = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        listed.trim(),
        "scripts/a.sh\nscripts/b.sh",
        "listed: {listed:?}"
    );
}
