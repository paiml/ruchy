//! SEC-2 (#259): the differential test. `serde_yaml_ng` 0.10 (the library
//! ruchy read and wrote YAML with, here a dev-only oracle) and `yaml_io` must
//! agree on every YAML file the repo commits: both read it, into the same
//! typeless value, or both refuse it. A typed call site deserializes the same
//! events into a struct, so agreement here is agreement there; the RHL
//! surface, the one typed shape that is also WRITTEN, has its own byte-for-byte
//! differential in `rhl/yaml_tests.rs`.
//!
//! Lib tests, not `tests/`: the required CI check runs `cargo test --lib`.

use super::{from_str, line_col, to_string};
use serde_json::{json, Value};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every committed `*.yaml` / `*.yml`, from git's own index (a file that is
/// not committed is no fixture of this repo's behaviour).
fn committed_yaml() -> Vec<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["ls-files", "-z", "--", "*.yaml", "*.yml"])
        .current_dir(root())
        .output()
        .expect("SEC-2: git ls-files runs");
    assert!(out.status.success(), "SEC-2: git ls-files failed");
    out.stdout
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .map(|p| root().join(String::from_utf8_lossy(p).as_ref()))
        .filter(|p| p.is_file())
        .collect()
}

/// The oracle's reading, as the call sites made it.
fn oracle(text: &str) -> Result<Value, String> {
    serde_yaml_ng::from_str::<Value>(text).map_err(|e| e.to_string())
}

fn ours(text: &str) -> Result<Value, String> {
    from_str::<Value>(text).map_err(|e| e.to_string())
}

/// The first JSON path where two readings part, with both leaves.
fn first_diff(a: &Value, b: &Value, path: &str) -> String {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => object_diff(x, y, path),
        (Value::Array(x), Value::Array(y)) if x.len() == y.len() => {
            let at = x.iter().zip(y).position(|(p, q)| p != q);
            at.map_or_else(String::new, |i| {
                first_diff(&x[i], &y[i], &format!("{path}[{i}]"))
            })
        }
        _ => format!("{path}: oracle {a} / ours {b}"),
    }
}

fn object_diff(
    x: &serde_json::Map<String, Value>,
    y: &serde_json::Map<String, Value>,
    path: &str,
) -> String {
    let parted = x.iter().find(|(k, va)| y.get(*k) != Some(*va));
    match parted {
        Some((k, va)) => y.get(k).map_or_else(
            || format!("{path}.{k}: only the oracle has it"),
            |vb| first_diff(va, vb, &format!("{path}.{k}")),
        ),
        None => format!("{path}: ours has extra keys"),
    }
}

/// One row of disagreement, or `None` when the two agree.
fn disagreement(name: &str, text: &str) -> Option<String> {
    match (oracle(text), ours(text)) {
        (Ok(a), Ok(b)) if a == b => None,
        (Err(_), Err(_)) => None,
        (Ok(a), Ok(b)) => Some(format!(
            "{name}: both read it, differently at {}",
            first_diff(&a, &b, "$")
        )),
        (Ok(_), Err(e)) => Some(format!("{name}: oracle reads it, ours refuses: {e}")),
        (Err(e), Ok(_)) => Some(format!("{name}: ours reads it, oracle refuses: {e}")),
    }
}

#[test]
fn test_sec2_every_committed_yaml_reads_as_the_oracle_reads_it() {
    let files = committed_yaml();
    // Non-vacuous: the repo commits hundreds; a listing of a handful means the
    // listing broke, not that the repo has no YAML.
    assert!(
        files.len() > 200,
        "SEC-2: only {} YAML files listed",
        files.len()
    );
    let mut read_ok = 0usize;
    let bad: Vec<String> = files
        .iter()
        .filter_map(|p| {
            let text = std::fs::read_to_string(p).ok()?;
            if oracle(&text).is_ok() {
                read_ok += 1;
            }
            let name = p.strip_prefix(root()).unwrap_or(p).display().to_string();
            disagreement(&name, &text)
        })
        .collect();
    assert!(
        read_ok * 10 >= files.len() * 9,
        "SEC-2: the oracle reads only {read_ok} of {} files",
        files.len()
    );
    assert!(
        bad.is_empty(),
        "SEC-2: {} disagreement(s):\n{}",
        bad.len(),
        bad.join("\n")
    );
}

/// The YAML-1.1-versus-1.2 spellings, merge keys, duplicate keys, quoting and
/// number forms where two YAML libraries are most likely to part ways. Each
/// row is a document both must read the same, or both refuse.
const EDGES: &[&str] = &[
    "on: push\n",
    "a: yes\nb: no\nc: on\nd: off\ne: y\nf: n\ng: True\nh: FALSE\n",
    "a: true\nb: false\n",
    "a: ~\nb: null\nc: Null\nd:\n",
    "a: 1\nb: -1\nc: 0x1F\nd: 0o17\ne: 017\nf: 1_000\ng: +5\n",
    "a: 1.5\nb: 1e3\nc: -0.0\nd: .5\ne: 6.02e+23\n",
    "a: 2026-10-13\nb: 12:30:00\n",
    "a: '007'\nb: \"x\\ty\"\nc: 'it''s'\n",
    "a: |\n  one\n  two\nb: >\n  folded\n  text\nc: |-\n  kept\n",
    "base: &b {x: 1}\nm:\n  <<: *b\n  y: 2\n",
    "a: 1\na: 2\n",
    "[1, 2, [3, {k: v}]]\n",
    "{}\n",
    "",
    "---\na: 1\n",
    "---\na: 1\n---\nb: 2\n",
    "a: !!str 5\nb: !!int '7'\n",
    "a: [1, 2\n",
    "1: one\n2: two\n",
    "? complex\n: key\n",
    "key with spaces: v\n\"quoted key\": v\n",
    "a: \"\\u00e9\"\nb: é\n",
    // PMAT-260: libyaml scans an anchor or alias NAME as [0-9A-Za-z_-] only;
    // granit-parser takes YAML 1.2's wider ns-anchor-char. Found by
    // test_sec2_prop_damaged_yaml_reads_as_the_oracle_reads_it (`unit: &!our`).
    "a: &!our\n",
    "a: &x!y 1\n",
    "a: &x.y 1\n",
    "a: &\u{e9} 1\n",
    "a: &ok 1\nb: *ok\n",
    "a: &ok-_9 1\nb: *ok-_9\n",
    "a: &ok 1\nb: *ok!\n",
    "a: [&x 1, *x]\n",
    "a: {k: &x 1, j: *x}\n",
    // PMAT-264: libyaml applies no indentation rule to the continuation line
    // of a quoted scalar in block context; granit-parser refuses one at or
    // left of the block's indent. Found by
    // test_sec2_prop_damaged_yaml_reads_as_the_oracle_reads_it (`' v5,*\n1'`).
    "a:\n  b: 'x\ny'\n",
    "a:\n  b: 'x\n y'\n",
    "a:\n  b: 'x\n  y'\n",
    "a: 'x\ny'\n",
    "- 'x\ny'\n",
    "a:\n  - 'x\ny'\n",
    "- b: 'x\ny'\n",
    "a:\n  b: \"x\ny\"\n",
    "a:\n  b: 'x\ny\nz'\n",
    "a:\n  b: 'x\n\ny'\n",
    "a:\n  b: '\n0'\n",
    "a:\n  b: 'x\n#y'\n",
    "a:\n  b: 'x\n- y'\n",
    "a:\n  b: 'x\nc: y'\n",
    "a:\n  b: 'x\n---y'\n",
    "a:\r\n  b: 'x\r\ny'\r\n",
    "a:\n  b: 'x\ny'\nc: 1\n",
    // #261: every continuation line of the refused scalar is widened in one
    // read, so the shapes that read finds the scalar and its lines through
    "a:\n  b: 'x\ny\n z\n  w\n   v'\n",
    "a:\n  b: &k 'x\ny\nz'\n",
    "a:\n  b: !!str 'x\ny\nz'\n",
    "a:\n  b: # c\n   'x\ny\nz'\n",
    "a:\n  b: 'x\n''y\nz'\n",
    "a:\n  b: 'x\ny''\nz'\n",
    "a:\n  b: 'x\n\n\ny\n\nz'\n",
    "a:\n  b: \"x\ny\\\"\nz\"\n",
    "a:\n  b: \"x\\\ny\nz\"\n",
    "a:\n  b: 'x\ny\n'\n",
    "a:\n  b: 'x\ny'\n  c: 'x\ny'\n  d: \"x\ny\"\n",
    "- - 'x\ny\nz'\n",
    "a:\n  b: 'x\ny\n---\nz'\n",
    "a:\n  b: 'x\ny' junk\n  c: 1\n",
    // and the ones libyaml still refuses: a document indicator at column 0,
    // trailing content after the quote, an unclosed quote
    "a:\n  b: 'x\n---\ny'\n",
    "a:\n  b: 'x\n--- y'\n",
    "a:\n  b: 'x\n...\ny'\n",
    "a:\n  b: 'x\ny' junk\n",
    "a:\n  b: 'x\ny\n",
];

#[test]
fn test_sec2_edge_spellings_read_as_the_oracle_reads_them() {
    let bad: Vec<String> = EDGES
        .iter()
        .enumerate()
        .filter_map(|(i, text)| disagreement(&format!("edge #{i} {text:?}"), text))
        .collect();
    assert!(
        bad.is_empty(),
        "SEC-2: {} disagreement(s):\n{}",
        bad.len(),
        bad.join("\n")
    );
}

/// PMAT-264: a continuation line the reader re-indents to read it as libyaml
/// does keeps its own column in an error raised on that line.
#[test]
fn test_sec2_an_error_on_a_reindented_line_keeps_its_column() {
    let text = "a:\n  b: 'x\ny' junk\n";
    let e = from_str::<Value>(text).expect_err("trailing content is refused");
    let (line, col) = line_col(&e);
    assert_eq!(line, 3, "PMAT-264: {e}");
    assert!(
        (1..="y' junk".len()).contains(&col),
        "PMAT-264: column {col} is not on the line as written: {e}"
    );
}

/// PMAT-264, the HOLD on #261: one quoted scalar with K under-indented
/// continuation lines cost K re-reads of a text that grew by the widest line
/// above at each one, O(K³) (3200 lines, 6.4 KB: 32 s). The work counted is
/// the bytes handed to the parser over every pass, so this is deterministic:
/// it must stay a small multiple of the input at N and at 2N lines, and the
/// input must still read as the oracle reads it.
#[test]
fn test_sec2_under_indented_continuations_are_reread_in_linear_work() {
    let doc = |k: usize| format!("a:\n  b: 'x\n{}'\n", "y\n".repeat(k));
    for k in [256, 512] {
        let text = doc(k);
        let work = super::read::reread_bytes(&text);
        assert!(
            work <= 8 * text.len(),
            "PMAT-264: {k} under-indented lines ({} bytes) cost {work} bytes of reading",
            text.len()
        );
    }
    let text = doc(512);
    assert_eq!(disagreement("512 lines", &text), None);
    let v: Value = from_str(&text).expect("PMAT-264: libyaml reads it");
    let want = format!("x{} ", " y".repeat(512));
    assert_eq!(v["a"]["b"], json!(want));
}

/// PMAT-264: the same over many quoted scalars, one under-indented line each.
/// Each scalar is a re-read of the whole text; the passes are bounded by a
/// work budget linear in the input (past it the text is refused, as the
/// reader before PMAT-264 refused it), and 256 of them fit inside it.
#[test]
fn test_sec2_reread_work_is_bounded_over_many_quoted_scalars() {
    let body: String = (0..256).map(|i| format!("  k{i}: 'x\ny'\n")).collect();
    let text = format!("a:\n{body}");
    let work = super::read::reread_bytes(&text);
    assert!(
        work <= (16 * text.len()).max(4 << 20),
        "PMAT-264: 256 scalars ({} bytes) cost {work} bytes of reading",
        text.len()
    );
    assert_eq!(disagreement("256 scalars", &text), None);
}

/// The writer, on the typeless values the call sites write.
#[test]
fn test_sec2_writer_is_byte_identical_to_the_oracle() {
    let values = [
        json!({"rhl": 1, "decls": []}),
        json!({"a": "plain", "b": "007", "c": "true", "d": "on", "e": "", "f": "x: y", "g": "#x"}),
        json!({"multi": "one\ntwo\n", "kept": "one\ntwo", "long": "w ".repeat(70)}),
        json!({"n": [1, -2, 3.5], "nested": {"k": [{"x": null}, []], "e": {}}}),
        json!([
            "- a",
            "[b]",
            "{c}",
            "it's",
            "say \"hi\"",
            "contains(github.ref_name, '-')"
        ]),
        json!({
            "tab": "a\tb", "lead": " x", "trail": "x ", "uni": "é ü", "emoji": "🦀",
            "nul": "a\u{0}b", "bom": "\u{feff}x", "nel": "a\u{85}b", "ctl": "\u{7}",
            "two": "a\n\n", "one": "\n", "lead_nl": "\nx", "sp_nl": "a \nb", "nl_sp": "a\n b",
            "dash": "-", "dash_x": "-x", "colon": "a:b", "hash": "a #b", "q": "?",
            "dots": "...", "dashes": "---x", "num": "1e3", "neg": "-0x1F", "oct": "0o17",
            "inf": ".inf", "nan": "NaN", "us": "1_000", "plus": "+5", "yes": "yes", "tilde": "~",
            "crlf": "a\r\nb", "ls": "a\u{2028}b",
        }),
        json!({"": 1, "a b": 2, "a: b": 3, "- x": 4, "k\nl": 5, "x".repeat(129): 6, "007": 7}),
        json!([[], {}, [[1, [2]], {"a": [{"b": []}]}], {"k": {"l": {"m": "n"}}}]),
        json!({"seq": [[1, 2], ["a"]], "map_in_seq": [{"a": 1, "b": [1]}, {"c": {}}]}),
        json!([
            1.0,
            0.1,
            1e20,
            1.5e-7,
            -0.0,
            123_456_789_012_u64,
            -9_000_000_000_i64
        ]),
        json!("top"),
        json!("multi\nline\n"),
        json!("multi\nline"),
        json!(""),
        json!(null),
        json!(true),
        json!(5),
        json!([]),
        json!({}),
    ];
    for v in &values {
        let want = serde_yaml_ng::to_string(v).expect("oracle writes");
        let got = to_string(v).expect("ours writes");
        assert_eq!(got, want, "SEC-2: writer differs for {v}");
    }
}

/// Positive control: the differential must see a difference when there is
/// one, or the green above means nothing. serde-saphyr's own deserializer
/// resolves `017` and `1_000` as numbers and expands `<<`; the oracle reads
/// all three as strings / an ordinary key. Through it, the comparison must
/// report a disagreement on each row; through ours, on none.
#[test]
fn test_sec2_control_the_differential_sees_a_planted_difference() {
    for text in [
        "e: 017\n",
        "f: 1_000\n",
        "base: &b {x: 1}\nm:\n  <<: *b\n  y: 2\n",
    ] {
        let planted: Result<Value, String> =
            serde_saphyr::from_str(text).map_err(|e| e.to_string());
        assert_ne!(
            planted,
            oracle(text),
            "SEC-2 control: serde-saphyr's resolver must differ on {text:?}"
        );
        assert!(
            disagreement("control", text).is_none(),
            "ours agrees on {text:?}"
        );
    }
}

#[test]
fn test_sec2_untagged_refuses_a_local_tag_and_reads_a_core_one() {
    for text in ["a: !custom x\n", "a: !python/object x\n", "a: !m\n  b: 1\n"] {
        assert!(
            matches!(super::untagged(text), Err(super::Untagged::Tag(_))),
            "SEC-2: {text:?} passed despite a local tag"
        );
    }
    assert_eq!(super::untagged("a: !!str 5\n"), Ok(()));
    assert_eq!(
        super::untagged("a: 1\na: 2\n"),
        Err(super::Untagged::DuplicateKey)
    );
    assert_eq!(
        super::untagged("'1': x\n1: y\n"),
        Ok(()),
        "a string key is not an int key"
    );
    let r: Value = from_str("a: !!str 5\n").expect("!!str");
    assert_eq!(r, json!({"a": "5"}));
}
#[test]
fn test_sec2_a_read_error_carries_a_one_based_line_and_column() {
    let e = from_str::<Value>("a: 1\nb: [\n").expect_err("unterminated");
    let (line, col) = line_col(&e);
    assert!(line >= 2 && col >= 1, "SEC-2: ({line}, {col})");
}

proptest::proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig::with_cases(
        std::env::var("PROPTEST_CASES").ok().and_then(|v| v.parse().ok()).unwrap_or(256)
    ))]

    /// The writer, on generated strings over the characters where styles part:
    /// indicators, blanks, breaks, quotes, non-printables, BOM, non-ASCII.
    #[test]
    fn test_sec2_prop_writer_matches_the_oracle_on_any_string(
        s in r#"[ -~\t\n\r\x{85}\x{A0}é🦀\x{FEFF}\x{2028}\x{7}]{0,12}"#,
        k in r#"[ -~\n\t]{0,6}"#,
    ) {
        for v in [
            json!(s),
            json!({"k": s, "list": [s.clone(), [s.clone()]], k.clone(): {"n": [s.clone(), k.clone()]}}),
            json!([{"a": s}, [k.clone()]]),
        ] {
            let want = serde_yaml_ng::to_string(&v).expect("oracle writes");
            proptest::prop_assert_eq!(to_string(&v).expect("ours writes"), want, "{}", v);
        }
    }
}

/// PMAT-264: an empty node (`d:` with nothing after it) is the empty string
/// to libyaml and the scalar `~` to granit. Typeless the two agree (both are
/// null), so the EDGES differential cannot see it; a `String` field can, and
/// the RHL `text:` field is one. A `~` the file actually spells stays `~`.
#[test]
fn test_sec2_an_empty_node_reads_into_a_string_as_the_oracle_reads_it() {
    use std::collections::BTreeMap;
    type Typed = BTreeMap<String, String>;
    for text in ["d:\n", "d:\ne: x\n", "d: ~\n", "d: ''\n", "d:"] {
        let theirs = serde_yaml_ng::from_str::<Typed>(text).map_err(|e| e.to_string());
        let ours = from_str::<Typed>(text).map_err(|e| e.to_string());
        assert_eq!(ours.ok(), theirs.ok(), "PMAT-264: {text:?}");
    }
    let seq = serde_yaml_ng::from_str::<Vec<String>>("- \n- x\n").ok();
    assert_eq!(
        from_str::<Vec<String>>("- \n- x\n").ok(),
        seq,
        "PMAT-264: sequence"
    );
}
