//! RHL vocabularies: loading `vocab/<name>-v<N>.yaml` and refusing a term with
//! no contract template (spec RHL-001 §3.4; plan P4, D6).
//!
//! A vocabulary file lives at `<root>/vocab/<name>-v<N>.yaml`, where `<root>`
//! is the nearest ancestor of the program that contains a `vocab/` directory
//! ([`find_root`]). Every relative path inside the file — a term's `contract:`
//! and an entity's `instances_from:` — resolves against that same `<root>`.

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// RHL's own closed list of term kinds (§3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TermKind {
    /// A value with no argument, for example `ticket title`.
    Noun,
    /// A reading taken from the world, for example `disk free of`.
    Measure,
    /// Something done to the world, for example `file ticket`.
    Action,
    /// A unit of a quantity, for example `GB`.
    Unit,
    /// A named thing with declared instances, for example `host`.
    Entity,
}

/// One parameter of a term: `{name: path, type: Text}`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Param {
    /// The parameter name.
    pub name: String,
    /// The parameter type, for example `Text` or `Ticket`.
    #[serde(rename = "type")]
    pub ty: String,
}

/// One vocabulary term.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Term {
    /// The spelling, for example `disk free of`.
    pub term: String,
    /// What kind of term it is.
    pub kind: TermKind,
    /// Its parameters, in order.
    #[serde(default)]
    pub takes: Vec<Param>,
    /// The type it gives, for example `Size`.
    #[serde(default)]
    pub gives: Option<String>,
    /// Its effect, `<verb> <target>`, for example `read disk`.
    #[serde(default)]
    pub effect: Option<String>,
    /// For an entity, the glob that declares its instances.
    #[serde(default)]
    pub instances_from: Option<String>,
    /// The contract template, relative to the vocabulary root. A term that
    /// omits it deserializes with an empty path, which names no file, so the
    /// vocabulary is refused with RHL-C001 by name (§3.4) rather than failing
    /// to load as an unknown vocabulary (RHL-V004).
    #[serde(default)]
    pub contract: String,
}

impl Term {
    /// The effect split into `(verb, target)`, if the term has one.
    #[must_use]
    pub fn effect_parts(&self) -> Option<(&str, &str)> {
        self.effect.as_deref()?.split_once(' ')
    }
}

/// A loaded vocabulary file.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Vocabulary {
    /// The name, for example `fleet`.
    pub vocabulary: String,
    /// The version number, for example `1`.
    pub version: u32,
    /// Its terms, in file order.
    pub terms: Vec<Term>,
}

impl Vocabulary {
    /// How a program names this vocabulary: `fleet v1`.
    #[must_use]
    pub fn label(&self) -> String {
        format!("{} v{}", self.vocabulary, self.version)
    }
}

/// The nearest ancestor directory of `file` that contains a `vocab/` directory.
#[must_use]
pub fn find_root(file: &Path) -> Option<PathBuf> {
    file.ancestors()
        .skip(1)
        .find(|dir| dir.join("vocab").is_dir())
        .map(Path::to_path_buf)
}

/// Split the words of `use vocabulary <name> v<N>` into `(name, N)`.
#[must_use]
pub fn parse_reference(words: &[&str]) -> Option<(String, u32)> {
    match words {
        [name, version] => {
            let n = version.strip_prefix('v')?.parse().ok()?;
            Some(((*name).to_string(), n))
        }
        _ => None,
    }
}

/// Load `<root>/vocab/<name>-v<version>.yaml`.
///
/// # Errors
///
/// Returns a message when the file cannot be read or parsed, or when it
/// declares a different name or version than the one asked for.
pub fn load(root: &Path, name: &str, version: u32) -> Result<Vocabulary, String> {
    let path = root.join("vocab").join(format!("{name}-v{version}.yaml"));
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let vocab: Vocabulary = serde_yaml_ng::from_str(&text)
        .map_err(|e| format!("cannot parse {}: {e}", path.display()))?;
    if vocab.vocabulary != name || vocab.version != version {
        return Err(format!(
            "{} declares `{}`, not `{name} v{version}`",
            path.display(),
            vocab.label()
        ));
    }
    Ok(vocab)
}

/// The terms of `vocab` whose `contract:` file does not exist under `root`.
#[must_use]
pub fn missing_contracts<'v>(root: &Path, vocab: &'v Vocabulary) -> Vec<&'v Term> {
    vocab
        .terms
        .iter()
        .filter(|t| !root.join(&t.contract).is_file())
        .collect()
}

/// The instances an entity's `instances_from` glob declares (plan D6).
///
/// The glob holds exactly one `*`; an instance is what that `*` matched, so
/// `machines/*/forjar.yaml` declares `gx10` for `machines/gx10/forjar.yaml`
/// and `fleet/runners/*.yaml` declares `r1` for `fleet/runners/r1.yaml`.
/// An empty result means no source of truth exists: the entity is unverified,
/// not closed-world. A glob without exactly one `*` declares nothing.
#[must_use]
pub fn instances(root: &Path, glob: &str) -> Vec<String> {
    let parts: Vec<&str> = glob.split('/').collect();
    let starred: Vec<usize> = (0..parts.len())
        .filter(|&i| parts[i].contains('*'))
        .collect();
    let (&[at], true) = (starred.as_slice(), glob.matches('*').count() == 1) else {
        return Vec::new();
    };
    let dir = parts[..at]
        .iter()
        .fold(root.to_path_buf(), |p, c| p.join(c));
    let rest = &parts[at + 1..];
    let mut found: Vec<String> = list_names(&dir)
        .into_iter()
        .filter_map(|n| star_match(parts[at], &n).map(|m| (n, m)))
        .filter(|(n, _)| rest.iter().fold(dir.join(n), |p, c| p.join(c)).exists())
        .map(|(_, m)| m)
        .collect();
    found.sort();
    found.dedup();
    found
}

fn list_names(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default()
}

/// What `*` matched when `name` matches the one-star `pattern`, if non-empty.
fn star_match(pattern: &str, name: &str) -> Option<String> {
    let (prefix, suffix) = pattern.split_once('*')?;
    let middle = name.strip_prefix(prefix)?.strip_suffix(suffix)?;
    (!middle.is_empty()).then(|| middle.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rhl_1_vocab_loads_both_pre_registered_files() {
        let root = crate::rhl::repo_root();
        let fleet = load(&root, "fleet", 1).expect("fleet v1 loads");
        assert_eq!(fleet.label(), "fleet v1");
        assert!(fleet.terms.iter().any(|t| t.term == "disk free of"));
        let tickets = load(&root, "tickets", 1).expect("tickets v1 loads");
        assert!(missing_contracts(&root, &tickets).is_empty());
        assert!(missing_contracts(&root, &fleet).is_empty());
    }

    #[test]
    fn test_rhl_1_vocab_unknown_or_mismatched_file_is_an_error() {
        let root = crate::rhl::repo_root();
        assert!(load(&root, "fleet", 9).is_err());
        assert!(load(&root, "nosuch", 1).is_err());
    }

    #[test]
    fn test_rhl_1_vocab_reference_needs_name_and_v_number() {
        assert_eq!(parse_reference(&["fleet", "v1"]), Some(("fleet".into(), 1)));
        assert_eq!(parse_reference(&["fleet"]), None);
        assert_eq!(parse_reference(&["fleet", "1"]), None);
        assert_eq!(parse_reference(&["fleet", "v1", "x"]), None);
    }

    #[test]
    fn test_rhl_1_vocab_find_root_is_the_nearest_ancestor_with_vocab() {
        let root = crate::rhl::repo_root();
        let file = root.join("docs/rhl/breaks/valid/01-gx10-disk-watch.rhl");
        assert_eq!(find_root(&file), Some(root));
        let tmp = tempfile::tempdir().expect("tempdir");
        assert_eq!(find_root(&tmp.path().join("a.rhl")), None);
    }

    #[test]
    fn test_rhl_1_vocab_instances_are_what_the_star_matched() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join("machines/gx10")).expect("mkdir");
        std::fs::write(root.join("machines/gx10/forjar.yaml"), "x").expect("write");
        std::fs::create_dir_all(root.join("machines/empty")).expect("mkdir");
        std::fs::create_dir_all(root.join("fleet/runners")).expect("mkdir");
        std::fs::write(root.join("fleet/runners/r1.yaml"), "x").expect("write");
        std::fs::write(root.join("fleet/runners/notes.txt"), "x").expect("write");
        assert_eq!(instances(root, "machines/*/forjar.yaml"), vec!["gx10"]);
        assert_eq!(instances(root, "fleet/runners/*.yaml"), vec!["r1"]);
        assert!(instances(root, "org/repos/*.yaml").is_empty());
        assert!(instances(root, "machines/gx10/forjar.yaml").is_empty());
    }
}
