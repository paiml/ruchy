//! Release manifest lint [PMAT-094] — clean-room and APR-MONO invariants.
//!
//! Contract: `contracts/clean-room-v1.yaml`.
//!
//! Four invariants, each falsifiable by a one-line manifest edit:
//! * `no_sibling_paths` — no dependency is a bare `path = "../…"` sibling, so a fresh
//!   clone (or a detached worktree) can load the manifest.
//! * `no_deprecated_facades` — the crates.io crates `trueno`, `entrenar`, `simular`,
//!   `trueno-viz` are deprecated facades; every such dependency key must be renamed
//!   onto the live `aprender-*` crate via `package = "aprender-…"`.
//! * `msrv_honest` — the declared `rust-version` is at least the maximum `rust_version`
//!   of every package in the resolved graph (`cargo metadata`).
//! * lock hygiene — the yanked `chacha20 0.10.1` is not in `Cargo.lock`.

use std::path::{Path, PathBuf};
use std::process::Command;

use toml::Value;

/// crates.io names that are deprecated facades of the aprender monorepo.
const DEPRECATED_FACADES: [&str; 4] = ["trueno", "entrenar", "simular", "trueno-viz"];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn root_manifest() -> Value {
    toml::from_str::<Value>(&read(&manifest_dir().join("Cargo.toml")))
        .expect("Cargo.toml must parse as TOML")
}

/// Collect every `*dependencies` table in the manifest, with a human-readable path.
fn dependency_tables(value: &Value, prefix: &str, out: &mut Vec<(String, Value)>) {
    let Some(table) = value.as_table() else {
        return;
    };
    for (key, child) in table {
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        if key.ends_with("dependencies") {
            out.push((path, child.clone()));
        } else {
            dependency_tables(child, &path, out);
        }
    }
}

/// `(table path, dependency key, dependency value)` for every declared dependency.
fn all_dependencies(manifest: &Value) -> Vec<(String, String, Value)> {
    let mut tables = Vec::new();
    dependency_tables(manifest, "", &mut tables);
    let mut deps = Vec::new();
    for (table_path, table) in tables {
        let Some(entries) = table.as_table() else {
            continue;
        };
        for (name, spec) in entries {
            deps.push((table_path.clone(), name.clone(), spec.clone()));
        }
    }
    deps
}

/// The crate that a dependency entry actually consumes: `package = …` if renamed.
fn effective_crate(name: &str, spec: &Value) -> String {
    spec.get("package")
        .and_then(Value::as_str)
        .unwrap_or(name)
        .to_string()
}

#[test]
fn test_pmat_094_no_sibling_paths_no_dependency_escapes_the_workspace() {
    let manifest = root_manifest();
    let offenders: Vec<String> = all_dependencies(&manifest)
        .into_iter()
        .filter_map(|(table, name, spec)| {
            let path = spec.get("path")?.as_str()?.to_string();
            path.starts_with("../")
                .then(|| format!("[{table}] {name} = {{ path = \"{path}\" }}"))
        })
        .collect();
    assert!(
        offenders.is_empty(),
        "sibling path dependencies break a fresh clone (issue #195): {offenders:?}"
    );
}

#[test]
fn test_pmat_094_no_deprecated_facades_keys_are_renamed_onto_aprender() {
    let manifest = root_manifest();
    let mut offenders = Vec::new();
    for (table, name, spec) in all_dependencies(&manifest) {
        let krate = effective_crate(&name, &spec);
        if DEPRECATED_FACADES.contains(&krate.as_str()) {
            offenders.push(format!("[{table}] {name} -> crate `{krate}`"));
        }
    }
    assert!(
        offenders.is_empty(),
        "deprecated crates.io facades must be renamed onto the live aprender-* crates \
         with `package = \"aprender-…\"`: {offenders:?}"
    );
}

#[test]
fn test_pmat_094_no_deprecated_facades_renamed_keys_point_at_aprender_crates() {
    let manifest = root_manifest();
    let mut checked = 0usize;
    for (table, name, spec) in all_dependencies(&manifest) {
        if !DEPRECATED_FACADES.contains(&name.as_str()) {
            continue;
        }
        let krate = effective_crate(&name, &spec);
        assert!(
            krate.starts_with("aprender-"),
            "[{table}] dependency key `{name}` must carry `package = \"aprender-…\"`, \
             found crate `{krate}`"
        );
        checked += 1;
    }
    assert!(
        checked > 0,
        "expected at least one aprender-renamed dependency key among {DEPRECATED_FACADES:?}"
    );
}

/// Parse a `rust-version` string into a comparable `(major, minor, patch)`.
fn parse_version(text: &str) -> (u64, u64, u64) {
    let mut parts = text.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

fn cargo_metadata() -> serde_json::Value {
    let out = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .current_dir(manifest_dir())
        .output()
        .expect("cargo metadata must run");
    assert!(
        out.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("cargo metadata emits JSON")
}

/// The highest `rust_version` declared by any package in the resolved graph.
fn graph_max_rust_version(metadata: &serde_json::Value) -> (String, (u64, u64, u64)) {
    let packages = metadata["packages"].as_array().expect("packages array");
    let mut best = (String::from("<none>"), (0, 0, 0));
    for package in packages {
        let Some(declared) = package["rust_version"].as_str() else {
            continue;
        };
        let parsed = parse_version(declared);
        if parsed > best.1 {
            let name = package["name"].as_str().unwrap_or("?");
            best = (format!("{name} {declared}"), parsed);
        }
    }
    best
}

#[test]
fn test_pmat_094_msrv_honest_declared_rust_version_covers_the_graph() {
    let manifest = root_manifest();
    let declared = manifest["workspace"]["package"]["rust-version"]
        .as_str()
        .expect("[workspace.package] rust-version must be a string")
        .to_string();
    let (who, max) = graph_max_rust_version(&cargo_metadata());
    assert!(
        parse_version(&declared) >= max,
        "declared rust-version {declared} is below the graph maximum ({who})"
    );
}

#[test]
fn test_pmat_094_lockfile_has_no_yanked_chacha20() {
    let lock = read(&manifest_dir().join("Cargo.lock"));
    // Walk each [[package]] block: the version belongs to the block that names chacha20,
    // wherever cargo places the line inside it (quorum lane 1: do not assume "next line").
    let mut seen = 0;
    for block in lock.split("[[package]]") {
        let is_chacha20 = block.lines().any(|l| l.trim() == "name = \"chacha20\"");
        if !is_chacha20 {
            continue;
        }
        seen += 1;
        let version = block
            .lines()
            .find(|l| l.trim().starts_with("version = "))
            .map(|l| l.trim().to_string())
            .expect("chacha20 block has a version line");
        assert_ne!(
            version, "version = \"0.10.1\"",
            "chacha20 0.10.1 is yanked; run `cargo update -p chacha20`"
        );
    }
    assert!(
        seen >= 1,
        "Cargo.lock has no chacha20 package block; the test would be vacuous"
    );
}

/// First `key = "value"` at column 0 (the `[package]` field), or empty.
fn top_level_string(manifest: &str, key: &str) -> String {
    manifest
        .lines()
        .filter_map(|line| line.strip_prefix(key))
        .filter_map(|rest| rest.trim_start().strip_prefix('='))
        .map(|rest| rest.trim().trim_matches('"').to_string())
        .next()
        .unwrap_or_default()
}

/// `version = "…"` inside the inline table of `dep = { … }`, or empty.
fn inline_dependency_version(manifest: &str, dep: &str) -> String {
    manifest
        .lines()
        .filter(|line| line.starts_with(&format!("{dep} = {{")))
        .filter_map(|line| line.split("version = \"").nth(1))
        .filter_map(|rest| rest.split('"').next())
        .map(str::to_string)
        .next()
        .unwrap_or_default()
}

/// PMAT-100: `ruchy-wasm` is published right after `ruchy` and must depend on
/// the version being published; the two crates carry one version number.
#[test]
fn test_pmat_100_ruchy_wasm_tracks_the_workspace_version() {
    let root = read(&manifest_dir().join("Cargo.toml"));
    let wasm = read(&manifest_dir().join("ruchy-wasm/Cargo.toml"));
    let version = top_level_string(&root, "version");
    assert!(!version.is_empty(), "root [package] version missing");
    assert_eq!(
        top_level_string(&wasm, "version"),
        version,
        "ruchy-wasm version"
    );
    assert_eq!(
        inline_dependency_version(&wasm, "ruchy"),
        version,
        "ruchy-wasm must depend on the ruchy version being published"
    );
}

#[test]
fn test_pmat_100_manifest_readers_read_package_and_inline_versions() {
    let root =
        "[package]\nname = \"x\"\nversion = \"1.2.3\"\n\n[dependencies]\nversion-sort = \"0.1\"\n";
    assert_eq!(top_level_string(root, "version"), "1.2.3");
    let wasm = "ruchy = { version = \"1.2.3\", path = \"..\", default-features = false }\n";
    assert_eq!(inline_dependency_version(wasm, "ruchy"), "1.2.3");
    assert_eq!(inline_dependency_version(wasm, "other"), "");
}

/// PMAT-129: the wasm32 dependency table must carry what the wasm build of the
/// root crate needs: getrandom 0.3 with `wasm_js` (rand 0.9 via aprender-core),
/// `serde-wasm-bindgen` (used by `src/wasm_bindings.rs`), and the web-sys
/// canvas context feature the computebrick entry imports.
#[test]
fn test_pmat_129_wasm32_dependency_table_is_complete() {
    let manifest: toml::Table = read(&manifest_dir().join("Cargo.toml"))
        .parse()
        .expect("Cargo.toml parses");
    let deps = manifest
        .get("target")
        .and_then(|t| t.get("cfg(target_arch = \"wasm32\")"))
        .and_then(|t| t.get("dependencies"))
        .and_then(|d| d.as_table())
        .expect("[target.'cfg(target_arch = \"wasm32\")'.dependencies] table");
    let getrandom03 = deps
        .values()
        .filter_map(|v| v.as_table())
        .find(|t| {
            t.get("package").and_then(|p| p.as_str()) == Some("getrandom")
                && t.get("version")
                    .and_then(|v| v.as_str())
                    .is_some_and(|v| v.starts_with("0.3"))
        })
        .expect("getrandom 0.3 declared for wasm32");
    let features = getrandom03
        .get("features")
        .and_then(|f| f.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();
    assert!(
        features.contains(&"wasm_js"),
        "getrandom 0.3 needs the wasm_js feature on wasm32; got {features:?}"
    );
    assert!(
        deps.contains_key("serde-wasm-bindgen"),
        "serde-wasm-bindgen must be a wasm32 dependency, not only a dev-dependency"
    );
    let web_sys = deps
        .get("web-sys")
        .and_then(|w| w.get("features"))
        .and_then(|f| f.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();
    assert!(
        web_sys.contains(&"CanvasRenderingContext2d"),
        "web-sys must enable CanvasRenderingContext2d (src/computebrick/wasm_entry.rs); got {web_sys:?}"
    );
}
