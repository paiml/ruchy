//! Advisory-exemption expiry gate (`SEC-1`).
//!
//! `cargo audit` and `cargo deny` both let a repository ignore an advisory. Neither
//! lets it say *until when*. That is how a standing red becomes permanent: the
//! exemption outlives the reason for it, nobody re-reads it, and the repository
//! goes on reporting green about a thing it stopped looking at.
//!
//! This gate makes an exemption a dated promise. `docs/audits/advisory-ignores.yaml`
//! carries a `removed_by` for every advisory ignored anywhere in the tree, and the
//! day one of those dates passes, this test goes RED — the same rule refusals get.
//!
//! It lives in `src/` and is `#[cfg(test)]` because the required status check
//! `gate` runs `cargo test --lib` on the root crate only (`sovereign-ci.yml` sets
//! `TEST_SCOPE=--lib`; this repository passes no `test_workspace` input). A gate
//! under `tests/` would never run in the required check, and a gate that does not
//! run is not a gate.

use serde::Deserialize;
use std::path::PathBuf;

const LEDGER: &str = "docs/audits/advisory-ignores.yaml";
const AUDIT_TOML: &str = ".cargo/audit.toml";
const DENY_TOML: &str = "deny.toml";

#[derive(Debug, Deserialize)]
struct Ledger {
    ignores: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Entry {
    id: String,
    reason: String,
    owner_ticket: String,
    removed_by: String,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel)).unwrap_or_default()
}

/// Every `RUSTSEC-yyyy-nnnn` token in a file, deduplicated.
///
/// Deliberately the same crude scan the sovereign-ci `security` job performs on
/// `.cargo/audit.toml` — it greps every such token and turns each into an
/// `--ignore` flag. Matching its behaviour exactly is the point: a token written
/// in a comment there IS an exemption, and this gate must see the same set CI
/// does, not a tidier one.
fn advisory_ids(text: &str) -> Vec<String> {
    let mut out: Vec<String> = text
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .filter_map(well_formed_id)
        .collect();
    out.sort();
    out.dedup();
    out
}

/// `RUSTSEC-<digits>-<digits>` and nothing else, matching the shape CI's
/// `sed -n 's/.*\(RUSTSEC-[0-9]*-[0-9]*\).*/\1/p'` accepts. An earlier revision
/// accepted any token starting `RUSTSEC-` and trimmed trailing non-digits, which
/// turned the placeholder `RUSTSEC-` + `yyyy-nnnn` in a comment into an EMPTY id
/// and failed the ledger check on a phantom. Caught by this gate on its first run.
fn well_formed_id(token: &str) -> Option<String> {
    let rest = token.strip_prefix("RUSTSEC-")?;
    let (year, num) = rest.split_once('-')?;
    let ok = year.len() == 4
        && year.chars().all(|c| c.is_ascii_digit())
        && !num.is_empty()
        && num.chars().all(|c| c.is_ascii_digit());
    ok.then(|| token.to_string())
}

fn ledger() -> Ledger {
    let text = read(LEDGER);
    assert!(!text.is_empty(), "SEC-1: {LEDGER} is missing or empty");
    serde_yaml::from_str(&text).unwrap_or_else(|e| panic!("SEC-1: {LEDGER} is not valid YAML: {e}"))
}

/// Every advisory this repository exempts, from both ignore mechanisms.
fn ignored_everywhere() -> Vec<String> {
    let mut ids = advisory_ids(&read(AUDIT_TOML));
    ids.extend(advisory_ids(&read(DENY_TOML)));
    ids.sort();
    ids.dedup();
    ids
}

#[test]
fn test_sec_1_every_ignored_advisory_is_in_the_ledger() {
    let listed: Vec<String> = ledger().ignores.iter().map(|e| e.id.clone()).collect();
    let unlisted: Vec<String> = ignored_everywhere()
        .into_iter()
        .filter(|id| !listed.contains(id))
        .collect();
    assert!(
        unlisted.is_empty(),
        "SEC-1: {} advisory exemption(s) have no entry in {LEDGER}: {unlisted:?}. \
         An exemption with no recorded expiry never expires, which is the failure \
         this gate exists to prevent. Add a reason, an owner ticket and a \
         `removed_by` date.",
        unlisted.len()
    );
}

#[test]
fn test_sec_1_no_ledger_entry_is_orphaned() {
    let ignored = ignored_everywhere();
    let orphans: Vec<String> = ledger()
        .ignores
        .iter()
        .filter(|e| !ignored.contains(&e.id))
        .map(|e| format!("{} (owner {})", e.id, e.owner_ticket))
        .collect();
    assert!(
        orphans.is_empty(),
        "SEC-1: {} ledger entr(ies) name an advisory that nothing ignores any more: \
         {orphans:?}. The exemption was dropped and the ledger was not. Remove them, \
         so the ledger keeps meaning what it says.",
        orphans.len()
    );
}

#[test]
fn test_sec_1_every_entry_carries_a_reason_and_an_owner() {
    let thin: Vec<String> = ledger()
        .ignores
        .iter()
        .filter(|e| e.reason.trim().len() < 20 || e.owner_ticket.trim().is_empty())
        .map(|e| e.id.clone())
        .collect();
    assert!(
        thin.is_empty(),
        "SEC-1: {} ledger entr(ies) carry no usable reason or no owner ticket: \
         {thin:?}. An exemption nobody owns is an exemption nobody removes.",
        thin.len()
    );
}

#[test]
fn test_sec_1_no_advisory_exemption_is_past_its_removed_by() {
    let today = chrono::Local::now().date_naive();
    let expired: Vec<String> = ledger()
        .ignores
        .iter()
        .filter_map(|e| expired_on(e, today))
        .collect();
    assert!(
        expired.is_empty(),
        "SEC-1 EXEMPTION EXPIRED: {} advisory exemption(s) are past their \
         `removed_by` date: {expired:#?}.\n\n\
         This is not a flaky gate — it is the promise coming due. Either the \
         advisory was fixed, in which case drop the ignore and the ledger entry, \
         or it was not, in which case re-review it and move the date DELIBERATELY, \
         naming what changed. Extending a date without re-reading the advisory is \
         how a standing red becomes permanent.",
        expired.len()
    );
}

/// `Some(description)` when this entry is past its date, `None` otherwise.
/// An unparseable date counts as expired: a promise nobody can read is not a
/// promise, and the safe reading of an unreadable date is "overdue".
fn expired_on(e: &Entry, today: chrono::NaiveDate) -> Option<String> {
    match chrono::NaiveDate::parse_from_str(e.removed_by.trim(), "%Y-%m-%d") {
        Ok(d) if d >= today => None,
        Ok(d) => Some(format!(
            "{} expired {} ({} day(s) ago), owner {}",
            e.id,
            d,
            (today - d).num_days(),
            e.owner_ticket
        )),
        Err(err) => Some(format!(
            "{} has an unreadable `removed_by` {:?}: {err}",
            e.id, e.removed_by
        )),
    }
}
