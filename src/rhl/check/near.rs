//! Candidates for an unknown word (plan D4).
//!
//! Distance is Levenshtein over characters, spaces included. The threshold is
//! borrowed, not invented: rustc's `find_best_match_for_name`
//! (`compiler/rustc_span/src/edit_distance.rs`) accepts a candidate within
//! `max(lookup.len(), 3) / 3`, which is `max(len / 3, 1)`.

use crate::rhl::diag::Candidate;

/// Levenshtein distance between `a` and `b`, counted in characters.
pub(crate) fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut row = Vec::with_capacity(b.len() + 1);
        row.push(i + 1);
        for (j, cb) in b.iter().enumerate() {
            let substitute = prev[j] + usize::from(ca != *cb);
            row.push(substitute.min(prev[j + 1] + 1).min(row[j] + 1));
        }
        prev = row;
    }
    prev[b.len()]
}

/// rustc's threshold for a lookup of `text`: `max(chars / 3, 1)`.
pub(crate) fn threshold(text: &str) -> usize {
    (text.chars().count() / 3).max(1)
}

/// Every `(spelling, vocabulary)` of `pool` at the minimum distance from
/// `text`, if that minimum is within the threshold. Spellings are unique: the
/// first vocabulary that offers a spelling names it.
pub(crate) fn nearest(text: &str, pool: &[(String, String)]) -> Vec<Candidate> {
    let limit = threshold(text);
    let scored: Vec<Candidate> = pool
        .iter()
        .map(|(term, vocabulary)| Candidate {
            term: term.clone(),
            vocabulary: vocabulary.clone(),
            distance: distance(text, term),
        })
        .filter(|c| c.distance <= limit)
        .collect();
    let Some(best) = scored.iter().map(|c| c.distance).min() else {
        return Vec::new();
    };
    let mut out: Vec<Candidate> = Vec::new();
    for c in scored.into_iter().filter(|c| c.distance == best) {
        if !out.iter().any(|o| o.term == c.term) {
            out.push(c);
        }
    }
    out
}

/// Search the word-prefixes of `words`, longest first. The first prefix
/// length `k` with any candidate wins: `(k, its candidates)`.
pub(crate) fn search_prefixes(
    words: &[String],
    pool: &[(String, String)],
) -> Option<(usize, Vec<Candidate>)> {
    (1..=words.len()).rev().find_map(|k| {
        let found = nearest(&words[..k].join(" "), pool);
        (!found.is_empty()).then_some((k, found))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool(words: &[&str]) -> Vec<(String, String)> {
        words
            .iter()
            .map(|w| ((*w).to_string(), "t v1".to_string()))
            .collect()
    }

    #[test]
    fn test_rhl_1_check_near_distance_is_levenshtein_over_chars() {
        assert_eq!(distance("disk fre of", "disk free of"), 1);
        assert_eq!(distance("", "abc"), 3);
        assert_eq!(distance("kitten", "sitting"), 3);
        assert_eq!(distance("é", "e"), 1);
    }

    #[test]
    fn test_rhl_1_check_near_threshold_is_rustc_rule() {
        assert_eq!(threshold("ab"), 1);
        assert_eq!(threshold("hosr"), 1);
        assert_eq!(threshold("disk fre of"), 3);
    }

    #[test]
    fn test_rhl_1_check_near_longest_prefix_wins_with_all_minimum_candidates() {
        let p = pool(&["host", "hour", "runner", "runner load of"]);
        let words: Vec<String> = ["hosr", "gx10"].map(String::from).to_vec();
        let (k, c) = search_prefixes(&words, &p).expect("candidates");
        assert_eq!(k, 1);
        let terms: Vec<&str> = c.iter().map(|c| c.term.as_str()).collect();
        assert_eq!(terms, vec!["host", "hour"]);
        let words: Vec<String> = ["runer", "load", "of"].map(String::from).to_vec();
        let (k, c) = search_prefixes(&words, &p).expect("candidates");
        assert_eq!((k, c.len(), c[0].term.as_str()), (3, 1, "runner load of"));
    }
}
