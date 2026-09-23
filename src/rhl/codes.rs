//! The RHL diagnostic code catalogue (spec RHL-001 §3.5; §9 rule 6: codes are
//! stable forever).
//!
//! This table is the single source. `docs/rhl/diagnostic-codes.md` is rendered
//! from it and a test holds the two equal, so a code cannot be documented
//! without existing or exist without being documented. A code is added here
//! only when a checker emits it and a test observes it (plan D3).

/// One catalogue entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeInfo {
    /// The stable code, `RHL-<family><nnn>`.
    pub code: &'static str,
    /// What the code means, in a few words.
    pub title: &'static str,
    /// The §3.5 refusal this code is an instance of. A refusal exits 2 rather
    /// than 1, because the compiler declines to guess (§3.5 "never a best guess").
    pub refusal: Option<&'static str>,
}

/// `end` of input where the parser needs `end` (pre-registered by RHL-0).
pub const P001: &str = "RHL-P001";
/// A token the grammar does not allow at that point.
pub const P002: &str = "RHL-P002";
/// Text that starts no token of the grammar.
pub const P003: &str = "RHL-P003";
/// A `.rhl.yaml` text that does not describe an intent tree (RHL-3).
pub const P004: &str = "RHL-P004";
/// Unknown word with no candidate.
pub const V001: &str = "RHL-V001";
/// Unknown word with exactly one candidate; its fix is safe (pre-registered).
pub const V002: &str = "RHL-V002";
/// Unknown word with several candidates; no fix is safe (pre-registered).
pub const V003: &str = "RHL-V003";
/// `use vocabulary` names no loadable vocabulary.
pub const V004: &str = "RHL-V004";
/// Quantities of different dimensions (pre-registered).
pub const T001: &str = "RHL-T001";
/// A value of the wrong type (pre-registered).
pub const T002: &str = "RHL-T002";
/// A bare number where a unit-bearing quantity is expected (§3.1 principle 4).
pub const T003: &str = "RHL-T003";
/// A term's effect is not declared by its unit (pre-registered).
pub const E001: &str = "RHL-E001";
/// `for each` over something that is not a finite collection (§3.1 principle 5).
pub const B001: &str = "RHL-B001";
/// A vocabulary term with no contract template; the vocabulary is refused (§3.4).
pub const C001: &str = "RHL-C001";
/// `given` or `then` outside an `example` block.
pub const X001: &str = "RHL-X001";
/// An `example` whose `given` lines leave a fact the job reads unset, so its
/// test would read a value the example never stated (RHL-5).
pub const X002: &str = "RHL-X002";
/// A construct of a checked program that RHL-4's lowering to ruchy does not
/// cover in v0; lowering declines rather than guesses (RHL-4).
pub const L001: &str = "RHL-L001";
/// A job that `ruchy compile` or `ruchy run` must build reads or performs a
/// term whose `lowers_to` is `[U]`: there is nothing to run it with (RHL-4).
pub const L002: &str = "RHL-L002";
/// `ruchy compile` could not emit or write the job's unit contract; a unit
/// without its contract is not built (§9.3, RHL-5b).
pub const C002: &str = "RHL-C002";

/// Every code, in family order. Append only; never renumber or reuse.
pub const CATALOGUE: &[CodeInfo] = &[
    info(P001, "missing `end`", None),
    info(P002, "unexpected token", None),
    info(P003, "invalid token", None),
    info(V001, "unknown word, no candidate", Some("OutOfVocabulary")),
    info(V002, "unknown word, one candidate", Some("OutOfVocabulary")),
    info(V003, "unknown word, several candidates", Some("Ambiguous")),
    info(V004, "unknown vocabulary", Some("OutOfVocabulary")),
    info(T001, "wrong unit", None),
    info(T002, "wrong type", None),
    info(T003, "bare number", None),
    info(E001, "undeclared effect", Some("UndeclaredEffect")),
    info(B001, "not a finite collection", Some("Unbounded")),
    info(C001, "no contract template", Some("NoContractTemplate")),
    info(X001, "`given`/`then` outside `example`", None),
    info(P004, "YAML is not an intent tree", None),
    info(L001, "construct not lowerable in v0", None),
    info(
        L002,
        "term has no runtime binding",
        Some("EngineUnavailable"),
    ),
    info(X002, "example leaves a fact unset", None),
    info(
        C002,
        "unit contract could not be emitted",
        Some("NoContractTemplate"),
    ),
];

const fn info(code: &'static str, title: &'static str, refusal: Option<&'static str>) -> CodeInfo {
    CodeInfo {
        code,
        title,
        refusal,
    }
}

/// The catalogue entry for `code`, if the code exists.
#[must_use]
pub fn lookup(code: &str) -> Option<&'static CodeInfo> {
    CATALOGUE.iter().find(|c| c.code == code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use std::collections::HashSet;

    const DOC: &str = "docs/rhl/diagnostic-codes.md";

    #[test]
    fn test_rhl_1_codes_match_the_stable_family_pattern_and_are_unique() {
        let re = Regex::new(r"^RHL-[PVTEBCXL][0-9]{3}$").expect("static regex");
        let mut seen = HashSet::new();
        for c in CATALOGUE {
            assert!(re.is_match(c.code), "{} is not RHL-<family><nnn>", c.code);
            assert!(seen.insert(c.code), "{} is listed twice", c.code);
        }
    }

    #[test]
    fn test_rhl_1_codes_keep_the_six_pre_registered_meanings() {
        let pre = [
            (P001, "missing `end`"),
            (V002, "unknown word, one candidate"),
            (V003, "unknown word, several candidates"),
            (E001, "undeclared effect"),
            (T001, "wrong unit"),
            (T002, "wrong type"),
        ];
        for (code, title) in pre {
            assert_eq!(lookup(code).map(|c| c.title), Some(title), "{code}");
        }
    }

    #[test]
    fn test_rhl_1_codes_doc_lists_exactly_the_catalogue() {
        let path = crate::rhl::repo_root().join(DOC);
        let doc = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("RHL-1: cannot read {}: {e}", path.display()));
        let rows: Vec<String> = doc
            .lines()
            .filter(|l| l.starts_with("| `RHL-"))
            .map(str::to_string)
            .collect();
        let want: Vec<String> = CATALOGUE.iter().map(doc_row).collect();
        assert_eq!(rows, want, "{DOC} has drifted from src/rhl/codes.rs");
    }

    fn doc_row(c: &CodeInfo) -> String {
        format!(
            "| `{}` | {} | {} |",
            c.code,
            c.title,
            c.refusal.map_or("—".to_string(), |r| format!("`{r}`"))
        )
    }
}
