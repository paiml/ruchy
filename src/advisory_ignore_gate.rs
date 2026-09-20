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
    /// A PERSON, not a ticket. An entry that named only a ticket, or carried a
    /// bare `needs_review: true`, was an unowned obligation with a date on it —
    /// which expires into exactly the red nobody consumes that this ledger
    /// exists to drain. There is no second date field: the review date IS
    /// `removed_by`.
    ///
    /// Defaulted so that an entry MISSING it fails the owner test by name,
    /// rather than aborting YAML parsing and failing all four with a
    /// deserialization error that says nothing about what is wrong.
    #[serde(default)]
    owner: String,
    removed_by: String,
    /// Which mechanism exempts this advisory — `audit`, `deny`, or both.
    #[serde(default)]
    source: String,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel)).unwrap_or_default()
}

/// Every advisory id a file exempts, deduplicated — with the SAME reach as CI's
/// scanner, deliberately.
///
/// The sovereign-ci `security` job extracts ignores with
/// `sed -n 's/.*\(RUSTSEC-[0-9]*-[0-9]*\).*/\1/p'`. That matches ANYWHERE inside a
/// token and stops each digit run at the first non-digit, so CI turns
/// `"RUSTSEC-2026-0285-rustls-pending-pmcp"` into a real `--ignore
/// RUSTSEC-2026-0285`.
///
/// An earlier revision here split the text into whole `[A-Za-z0-9-]` runs and
/// required the WHOLE run to be a well-formed id. MEASURED (second-family review,
/// 2026-09-20): the two disagree in the dangerous direction on at least four token
/// shapes — `xRUSTSEC-2024-0384`, `0RUSTSEC-2024-0384`, `RUSTSEC-2024-0384x` and
/// `RUSTSEC-2024-0384-instant`. CI exempts the advisory; the gate saw nothing; the
/// ledger never dated it. The hyphen-suffix form is what someone writes by
/// accident, the others are what someone writes on purpose.
///
/// So this now scans the way the sed does, and takes EVERY match on a line rather
/// than the last one — being over-strict is safe, being under-strict is the hole.
fn advisory_ids(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, _) in text.match_indices("RUSTSEC-") {
        let rest = &text[i + "RUSTSEC-".len()..];
        let year: String = rest.chars().take_while(char::is_ascii_digit).collect();
        let after = &rest[year.len()..];
        let Some(tail) = after.strip_prefix('-') else {
            continue;
        };
        let num: String = tail.chars().take_while(char::is_ascii_digit).collect();
        out.push(format!("RUSTSEC-{year}-{num}"));
    }
    out.sort();
    out.dedup();
    out
}

fn ledger() -> Ledger {
    let text = read(LEDGER);
    assert!(!text.is_empty(), "SEC-1: {LEDGER} is missing or empty");
    serde_yaml::from_str(&text).unwrap_or_else(|e| panic!("SEC-1: {LEDGER} is not valid YAML: {e}"))
}

/// `deny.toml`, parsed. CLAUDE.md rule 2: no simple heuristics.
///
/// An earlier revision scanned this file as TEXT with `#`-comments stripped
/// mid-line. TOML does not treat `#` inside a string as a comment, so
/// `{ reason = "tracked in #1234", id = "RUSTSEC-2024-0384" }` hid a LIVE
/// exemption from the gate — measured end to end, with the previous revision as
/// a control that caught it. Parsing removes that whole class rather than
/// patching one spelling of it.
fn deny_doc() -> toml::Value {
    let text = read(DENY_TOML);
    toml::from_str(&text).unwrap_or_else(|e| panic!("SEC-1: {DENY_TOML} is not valid TOML: {e}"))
}

/// The advisory ids `deny.toml` exempts, read from the parsed document rather
/// than grepped: an entry is either a bare string or a table with an `id`.
fn deny_ignored_ids(doc: &toml::Value) -> Vec<String> {
    doc.get("advisories")
        .and_then(|a| a.get("ignore"))
        .and_then(toml::Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|r| r.as_str().or_else(|| r.get("id")?.as_str()))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Every advisory this repository exempts, wherever it says so.
///
/// Not just the config files: `cargo audit --ignore RUSTSEC-…` written inline in
/// a workflow or a Makefile recipe is a fully effective, undated exemption.
/// sovereign-ci states that norm ("Advisory exemptions go in .cargo/audit.toml,
/// not in CI skip logic") in a comment, which is not a check. This is.
///
/// `.cargo/audit.toml`, the Makefile and the workflows are scanned WHOLE,
/// comments included — CI's sed is whole-file, `#` inside a quoted shell string
/// is not a comment to bash or to make, and being over-strict here is the safe
/// direction. `deny.toml` alone is read structurally, because it alone is parsed
/// rather than grepped by its consumer.
fn ignored_everywhere() -> Vec<String> {
    let mut ids = advisory_ids(&read(AUDIT_TOML));
    ids.extend(advisory_ids(&read("Makefile")));
    ids.extend(deny_ignored_ids(&deny_doc()));
    for (_, text) in workflow_files() {
        ids.extend(advisory_ids(&text));
    }
    ids.sort();
    ids.dedup();
    ids
}

/// Every workflow file and its text. Panics rather than returning nothing: a
/// missing or unreadable directory used to be swallowed by `flatten()`, so a
/// renamed `.github/workflows` — or one file at mode 000 — made the gate scan
/// nothing and pass. Measured, both cases.
fn workflow_files() -> Vec<(PathBuf, String)> {
    let dir = repo_root().join(WORKFLOWS);
    let entries = std::fs::read_dir(&dir).unwrap_or_else(|e| {
        panic!(
            "SEC-1: cannot read {WORKFLOWS}: {e}. An unreadable \
             workflow directory must not read as 'no exemptions here'."
        )
    });
    let mut out = Vec::new();
    for e in entries {
        let p = e
            .expect("SEC-1: unreadable entry in .github/workflows")
            .path();
        let is_yaml = p
            .extension()
            .and_then(|x| x.to_str())
            .is_some_and(|x| x.eq_ignore_ascii_case("yml") || x.eq_ignore_ascii_case("yaml"));
        if is_yaml {
            let text = std::fs::read_to_string(&p)
                .unwrap_or_else(|e| panic!("SEC-1: cannot read {}: {e}", p.display()));
            out.push((p, text));
        }
    }
    assert!(
        !out.is_empty(),
        "SEC-1: no workflow files found under {WORKFLOWS}. This gate scans them for \
         inline `cargo audit --ignore`; finding none means the path moved, not that \
         the repository has no workflows."
    );
    out
}

const WORKFLOWS: &str = ".github/workflows";

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
        .filter(|e| {
            e.reason.trim().len() < 20
                || e.owner_ticket.trim().is_empty()
                || e.owner.trim().is_empty()
        })
        .map(|e| {
            format!(
                "{} (owner {:?}, ticket {:?})",
                e.id, e.owner, e.owner_ticket
            )
        })
        .collect();
    assert!(
        thin.is_empty(),
        "SEC-1: {} ledger entr(ies) carry no usable reason, no named owner, or no \
         owner ticket: {thin:#?}. An exemption nobody owns is an exemption nobody \
         removes — the date then expires into a red nobody consumes, which is the \
         failure this ledger exists to prevent. `owner` is a person; `owner_ticket` \
         is where the work is tracked. Both are required.",
        thin.len()
    );
}

#[test]
fn test_sec_1_no_advisory_exemption_is_past_its_removed_by() {
    // UTC, not Local: MEASURED that the same ledger and the same instant give
    // GREEN in Berlin and RED in Pacific/Kiritimati. One calendar day either
    // way, failing unsafe westward. A gate's verdict must not depend on the
    // runner's timezone.
    let today = chrono::Utc::now().date_naive();
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

/// Suppression spellings that carry no advisory id at all.
///
/// MEASURED with cargo-deny 0.19.0 against this manifest: with `[advisories]
/// ignore = []`, `cargo deny check advisories` exits 1 naming RUSTSEC-2025-0141
/// and RUSTSEC-2024-0384 — but adding EITHER `unmaintained = "none"` OR
/// `[graph] exclude = ["instant","bincode"]` takes it to exit 0. Neither line
/// contains a `RUSTSEC-` token, so no id-scanning gate can ever see them.
///
/// `unmaintained = "none"` is the worse of the two: it is a blanket FORWARD
/// exemption, silencing the whole class the ledger's entries cover and every
/// future advisory in it, with one line that carries no id to date.
#[test]
fn test_sec_1_no_untraceable_blanket_suppression() {
    let doc = deny_doc();
    let mut found = suppressing_levels(&doc);
    found.extend(narrowing_graph_keys(&doc));
    assert!(
        found.is_empty(),
        "SEC-1: {DENY_TOML} suppresses advisories in a way that carries NO advisory \
         id, so nothing can date it and this ledger can never see it: {found:#?}.\n\n\
         An exemption that names the advisory can be owned and expired. One that \
         silences a whole class, or removes the crate from the graph, cannot — and \
         it silences every FUTURE advisory in that class too. Exempt by id, or say \
         in {DENY_TOML} why the class is genuinely not applicable.",
    );
}

/// Advisory lint classes set to anything that stops them being reported.
///
/// WHAT THIS DOES NOT COVER, stated because the test's name reads as a
/// universal and is not one: it inspects the five lint classes named below and
/// `[graph] exclude`. A future cargo-deny key that suppresses advisories under
/// some other name would evade it until added here. Allow-listing the REPORTING
/// values rather than deny-listing the suppressing ones is what keeps that gap
/// to new KEYS rather than new VALUES.
///
/// Allow-listing the reporting values, rather than deny-listing the suppressing
/// ones, is deliberate. A review measured four evasions of the old deny-list —
/// `'none'` in single quotes, a quoted key, and cargo-deny's own
/// `unmaintained = "workspace"`, which reads as ordinary configuration and
/// silences exactly the class every current ledger entry lives in. Any value this
/// gate does not recognise as REPORTING is treated as suppressing.
///
/// The same reasoning does NOT protect the `[graph]` keys, which are a
/// hand-maintained list — see `narrowing_graph_keys`, and the contract's
/// `scope_limit`, which had to be corrected once already.
fn suppressing_levels(doc: &toml::Value) -> Vec<String> {
    const CLASSES: &[&str] = &[
        "vulnerability",
        "unmaintained",
        "unsound",
        "notice",
        "yanked",
    ];
    const REPORTING: &[&str] = &["deny", "warn", "all"];
    let Some(adv) = doc.get("advisories").and_then(toml::Value::as_table) else {
        return Vec::new();
    };
    adv.iter()
        .filter(|(k, _)| CLASSES.contains(&k.as_str()))
        .filter_map(|(k, v)| v.as_str().map(|s| (k, s)))
        .filter(|(_, v)| !REPORTING.contains(v))
        .map(|(k, v)| format!("advisories.{k} = \"{v}\"  (silences the `{k}` class outright)"))
        .collect()
}

/// `[graph]` keys that narrow the graph so an advisory stops being reported.
///
/// `exclude` drops named crates outright. `exclude-dev` and `exclude-unpublished`
/// drop whole SUBTREES, which is worse: they carry no crate name either, so
/// nothing in them can be owned or dated.
///
/// MEASURED 2026-09-20, cargo-deny 0.19.0 on this manifest: with the
/// `RUSTSEC-2024-0384` ignore removed, `cargo deny check advisories` exits 1
/// naming `instant`; adding the single line `exclude-dev = true` inside the
/// existing `[graph]` table takes it to exit 0. A live advisory silenced with no
/// id, no owner and no date. Found by the Claude lane of the third review round;
/// my own first probe of it was malformed (a second `[graph]` table, which
/// cargo-deny rejects as a redefinition) and wrongly read as "not an evasion".
fn narrowing_graph_keys(doc: &toml::Value) -> Vec<String> {
    const NARROWING: &[&str] = &["exclude", "exclude-dev", "exclude-unpublished"];
    let Some(graph) = doc.get("graph").and_then(toml::Value::as_table) else {
        return Vec::new();
    };
    graph
        .iter()
        .filter(|(k, _)| NARROWING.contains(&k.as_str()))
        .filter(|(_, v)| narrows(v))
        .map(|(k, v)| format!("graph.{k} = {v}  (removes crates from the graph, so their advisories are never reported)"))
        .collect()
}

/// A narrowing key is live when it is `true`, or a non-empty array.
fn narrows(v: &toml::Value) -> bool {
    match v {
        toml::Value::Boolean(b) => *b,
        toml::Value::Array(a) => !a.is_empty(),
        _ => false,
    }
}

/// The audited files must EXIST, or the gate reads an empty string and passes.
///
/// `read()` returns "" for a missing file, which is right for `.cargo/audit.toml`
/// (this repository has none, by design — the rustls advisory was fixed rather
/// than exempted) but wrong for `deny.toml`, which is tracked and mandatory. The
/// exposure is path drift: both names are defined in ANOTHER repository's
/// workflow, and if one moves upstream this gate's failure mode is green.
#[test]
fn test_sec_1_the_audited_files_are_where_this_gate_looks() {
    assert!(
        repo_root().join("Makefile").is_file(),
        "SEC-1: the Makefile is missing, and it is one of the files scanned for an \
         inline `cargo audit --ignore`. `read()` returns an empty string for a file \
         that is not there, so a renamed Makefile reads as 'no exemptions here' — \
         the same fail-open shape this gate closed for the workflows."
    );
    assert!(
        repo_root().join(DENY_TOML).is_file(),
        "SEC-1: {DENY_TOML} is missing. It is tracked and mandatory, and this gate \
         reads it by a hardcoded name whose authoritative definition lives in \
         paiml/.github's workflow. If the path moved, this gate silently passes."
    );
    let declares_audit_source = ledger()
        .ignores
        .iter()
        .any(|e| matches!(e.source.trim(), "audit" | "both"));
    assert!(
        !declares_audit_source || repo_root().join(AUDIT_TOML).is_file(),
        "SEC-1: a ledger entry declares `source: audit`, but {AUDIT_TOML} does not \
         exist. Either the file was removed and its entry was not, or the entry is \
         wrong — both leave an exemption recorded that nothing enforces."
    );
}

/// A date is not a window. MEASURED: `removed_by: "9999-12-31"` satisfied every
/// other check here. "Extend the date" is the failure the expiry test warns
/// about, and without a cap it is unbounded.
#[test]
fn test_sec_1_no_exemption_is_dated_beyond_the_maximum_window() {
    let today = chrono::Utc::now().date_naive();
    let cap = today + chrono::Duration::days(MAX_WINDOW_DAYS);
    let far: Vec<String> = ledger()
        .ignores
        .iter()
        .filter_map(|e| {
            chrono::NaiveDate::parse_from_str(e.removed_by.trim(), "%Y-%m-%d")
                .ok()
                .map(|d| (e, d))
        })
        .filter(|(_, d)| *d > cap)
        .map(|(e, d)| format!("{} dated {d}, {} days out", e.id, (d - today).num_days()))
        .collect();
    assert!(
        far.is_empty(),
        "SEC-1: {} exemption(s) are dated more than {MAX_WINDOW_DAYS} days out: \
         {far:#?}. A promise far enough away is not a promise. If an exemption \
         genuinely cannot be removed inside that window, that is a decision worth \
         re-taking on a schedule, not one to write down once.",
        far.len()
    );
}

/// The longest an exemption may be dated into the future. 90 days is three
/// re-reviews a year; it is a cap, not a target.
const MAX_WINDOW_DAYS: i64 = 90;
