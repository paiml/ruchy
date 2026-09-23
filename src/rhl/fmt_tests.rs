//! RHL-1 `fmt` tests (spec RHL-001 §3.1 principle 2, §7 F2; plan D7). Lib
//! tests, not `tests/`: the required CI check runs `cargo test --lib` only.
//!
//! Five groups: the corpus is already in normal form (byte-for-byte); F2's
//! floor, idempotence and tree preservation over every parseable corpus
//! program; F2's positive control (a whitespace-mangled program formats to the
//! corpus bytes) and a negative control; property tests over trees the grammar
//! can produce; and the D7 blank-line rule's edge cases.

use super::{format, format_source};
use crate::rhl::parse;
use crate::rhl::tree::{
    Action, App, Atom, CompOp, Cond, CondKind, Decl, Effect, EffectVerb, Int, Phrase, Program,
    Quantity, Span, Stmt, StmtKind, Text, Unit, UnitKind, UseDecl, Word,
};
use proptest::prelude::*;
use std::path::{Path, PathBuf};

const SPEC: &str = "docs/specifications/ruchy-high-level-language-interface.md";
const BREAKS: &str = "docs/rhl/breaks";

fn root() -> PathBuf {
    crate::rhl::repo_root()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("RHL-1: cannot read {}: {e}", path.display()))
}

/// The 12 valid programs, sorted.
fn valid_files() -> Vec<PathBuf> {
    let dir = root().join(BREAKS).join("valid");
    let mut v: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("RHL-1: cannot list {}: {e}", dir.display()))
        .map(|e| e.expect("RHL-1: readable dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "rhl"))
        .collect();
    v.sort();
    v
}

/// `planted/<class>/*/broken.rhl` for every class except `missing-end`.
fn parseable_planted() -> Vec<PathBuf> {
    let base = root().join(BREAKS).join("planted");
    let mut out = Vec::new();
    for class in std::fs::read_dir(&base)
        .expect("RHL-1: planted dir")
        .flatten()
    {
        if class.file_name() == "missing-end" {
            continue;
        }
        for case in std::fs::read_dir(class.path())
            .expect("class dir")
            .flatten()
        {
            out.push(case.path().join("broken.rhl"));
        }
    }
    out.sort();
    out
}

/// The fenced code block under `### 3.2 Example` in the spec.
fn spec_example() -> String {
    let spec = read(&root().join(SPEC));
    let after = spec.split_once("### 3.2 Example").expect("§3.2").1;
    let body = after.split_once("```").expect("§3.2 fence").1;
    let body = body.split_once('\n').expect("fence line ends").1;
    body.split_once("```").expect("fence closes").0.to_string()
}

fn fmt_ok(src: &str, what: &str) -> String {
    format_source(src).unwrap_or_else(|e| panic!("{what}: does not parse: {e:?}"))
}

fn meaning(src: &str, what: &str) -> serde_json::Value {
    let program = parse(src).unwrap_or_else(|e| panic!("{what}: does not parse: {e:?}"));
    serde_json::to_value(program).expect("tree serializes")
}

// ───────────────────────── a. corpus bytes ─────────────────────────

#[test]
fn test_rhl_1_fmt_corpus_valid_programs_are_normal_form() {
    let files = valid_files();
    assert_eq!(files.len(), 12, "valid programs: {files:?}");
    for p in &files {
        let src = read(p);
        let what = p.display().to_string();
        assert_eq!(fmt_ok(&src, &what), src, "{what} is not in normal form");
    }
}

#[test]
fn test_rhl_1_fmt_corpus_spec_example_is_normal_form() {
    let src = spec_example();
    assert!(
        src.contains("job \"gx10 disk watch\""),
        "wrong block: {src}"
    );
    assert_eq!(fmt_ok(&src, "§3.2"), src);
}

// ───────────────────────── b. F2 floor ─────────────────────────

/// Every parseable corpus program: 12 valid, 60 planted, the §3.2 example.
fn parseable_corpus() -> Vec<(String, String)> {
    let mut all: Vec<(String, String)> = valid_files()
        .into_iter()
        .chain(parseable_planted())
        .map(|p| (p.display().to_string(), read(&p)))
        .collect();
    all.push(("§3.2 example".to_string(), spec_example()));
    all
}

fn assert_idempotent_and_tree_preserving(what: &str, src: &str) {
    let f1 = fmt_ok(src, what);
    assert_eq!(fmt_ok(&f1, what), f1, "{what}: fmt is not idempotent");
    assert_eq!(
        meaning(&f1, what),
        meaning(src, what),
        "{what}: tree changed"
    );
}

#[test]
fn test_rhl_1_fmt_f2_idempotent_and_tree_preserving_over_corpus() {
    let corpus = parseable_corpus();
    assert_eq!(corpus.len(), 73, "12 valid + 60 planted + §3.2");
    for (what, src) in &corpus {
        assert_idempotent_and_tree_preserving(what, src);
    }
}

// ───────────────────────── c. positive / negative control ─────────────────────────

/// Keywords the lexer reads as ONE token with exactly one inner space.
const MULTI_WORD: &[&str] = &[
    "use vocabulary",
    "for each",
    "repeat at most",
    "wait up to",
    "runs on",
    "may read",
    "may write",
    "may call",
    "give back",
    "stop with",
    "is not",
    "is below",
    "is above",
    "is at least",
    "is at most",
    "is one of",
];

/// A tiny deterministic generator (an LCG), so every run mangles alike.
struct Lcg(u64);

impl Lcg {
    fn pick<'a>(&mut self, options: &[&'a str]) -> &'a str {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let i = usize::try_from((self.0 >> 33) % options.len() as u64).unwrap_or(0);
        options[i]
    }
}

/// Split one line into tokens: string literals whole, other text on spaces.
fn line_tokens(line: &str) -> Vec<String> {
    let mut toks = Vec::new();
    let mut rest = line.trim();
    while !rest.is_empty() {
        let end = if rest.starts_with('"') {
            rest[1..].find('"').map_or(rest.len(), |i| i + 2)
        } else {
            rest.find(char::is_whitespace).unwrap_or(rest.len())
        };
        toks.push(rest[..end].to_string());
        rest = rest[end..].trim_start();
    }
    toks
}

/// How many leading tokens form a multi-word keyword (1 if none does).
fn keyword_len(toks: &[String]) -> usize {
    MULTI_WORD
        .iter()
        .map(|k| k.split(' ').collect::<Vec<_>>())
        .filter(|ws| ws.len() <= toks.len() && ws.iter().zip(toks).all(|(a, b)| a == b))
        .map(|ws| ws.len())
        .max()
        .unwrap_or(1)
}

/// Tokens with every multi-word keyword merged back into one token.
fn merged_tokens(line: &str) -> Vec<String> {
    let toks = line_tokens(line);
    let mut out = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        let n = keyword_len(&toks[i..]);
        out.push(toks[i..i + n].join(" "));
        i += n;
    }
    out
}

/// One non-blank line, re-spaced: indentation, gaps and trailing blanks vary.
fn mangle_line(line: &str, rng: &mut Lcg) -> String {
    let mut out = rng.pick(&["", " ", "\t", "      ", " \t  "]).to_string();
    for (i, tok) in merged_tokens(line).iter().enumerate() {
        if i > 0 {
            out.push_str(rng.pick(&[" ", "  ", "\t", " \t ", "    "]));
        }
        out.push_str(tok);
    }
    out.push_str(rng.pick(&["", "", " ", "\t  "]));
    out
}

/// A deterministic whitespace-mangled variant of `src`.
fn mangle(src: &str, seed: u64) -> String {
    let mut rng = Lcg(seed);
    let mut out = rng.pick(&["", "\n", "\n \t\n", "   \n\n"]).to_string();
    for line in src.lines() {
        if line.trim().is_empty() {
            out.push_str(rng.pick(&["", "\n", " \t\n", "\n\n"]));
            continue;
        }
        out.push_str(&mangle_line(line, &mut rng));
        out.push('\n');
        out.push_str(rng.pick(&["", "", "", "\n", "\t\n  \n"]));
    }
    out.push_str(rng.pick(&["", "\n", "  \n\t"]));
    out
}

#[test]
fn test_rhl_1_fmt_positive_control_mangled_programs_format_to_corpus_bytes() {
    let files = valid_files();
    assert_eq!(files.len(), 12);
    for (seed, p) in (1u64..).zip(&files) {
        let src = read(p);
        let mangled = mangle(&src, seed);
        let what = format!("{} (mangled, seed {seed})", p.display());
        assert_ne!(mangled, src, "{what}: mangling changed nothing");
        assert_eq!(fmt_ok(&mangled, &what), src, "{what}:\n{mangled}");
    }
}

#[test]
fn test_rhl_1_fmt_positive_control_mangler_is_not_vacuous() {
    let src = read(&valid_files()[0]);
    let mangled = mangle(&src, 7);
    assert!(mangled.contains('\t'), "no tab introduced");
    assert!(mangled
        .lines()
        .any(|l| l.ends_with(' ') || l.ends_with('\t')));
    assert!(mangled.contains("runs on"), "multi-word keyword split");
    assert!(mangled.contains("\"gx10 disk watch\""), "string re-spaced");
}

#[test]
fn test_rhl_1_fmt_negative_control_one_token_change_formats_differently() {
    let src = read(&valid_files()[0]);
    assert!(src.contains("100 GB"), "fixture lacks `100 GB`");
    let changed = src.replacen("100 GB", "101 GB", 1);
    let out = fmt_ok(&changed, "changed");
    assert_ne!(out, src);
    assert_eq!(out, changed, "the change is itself in normal form");
}

#[test]
fn test_rhl_1_fmt_leading_zeros_print_canonical_digits() {
    let src = "job \"j\"\n  every 007 hour\nend\n";
    assert_eq!(fmt_ok(src, "zeros"), "job \"j\"\n  every 7 hour\nend\n");
}

// ───────────────────────── d. property tests ─────────────────────────

fn sp() -> Span {
    Span::default()
}

fn word(text: &str) -> Word {
    Word {
        text: text.to_string(),
        span: sp(),
    }
}

/// Non-keyword lowercase words; none is a keyword or a keyword part.
fn arb_word() -> impl Strategy<Value = Word> {
    prop::sample::select(vec![
        "alpha", "beta", "gamma", "disk", "free", "host", "gx10", "load", "size", "label", "title",
        "repo_x",
    ])
    .prop_map(word)
}

fn arb_phrase() -> impl Strategy<Value = Phrase> {
    prop::collection::vec(arb_word(), 1..=3).prop_map(|words| Phrase { words, span: sp() })
}

fn arb_int() -> impl Strategy<Value = Int> {
    "0|[1-9][0-9]{0,24}".prop_map(|digits| Int { digits, span: sp() })
}

fn arb_unit() -> impl Strategy<Value = Option<Word>> {
    prop::option::of(
        prop::sample::select(vec!["GB", "MB", "Kb", "hour", "day", "%"]).prop_map(word),
    )
}

fn arb_quantity() -> impl Strategy<Value = Quantity> {
    (arb_int(), arb_unit()).prop_map(|(value, unit)| Quantity {
        value,
        unit,
        span: sp(),
    })
}

fn arb_text() -> impl Strategy<Value = Text> {
    "[a-z0-9 /._-]{0,12}".prop_map(|value| Text { value, span: sp() })
}

fn arb_atom() -> impl Strategy<Value = Atom> {
    prop_oneof![
        arb_text().prop_map(Atom::Text),
        arb_quantity().prop_map(Atom::Quantity)
    ]
}

fn arb_app() -> impl Strategy<Value = App> {
    prop_oneof![
        (arb_phrase(), prop::collection::vec(arb_atom(), 0..3))
            .prop_map(|(phrase, args)| App::Call { phrase, args }),
        arb_quantity().prop_map(App::Quantity),
        arb_text().prop_map(App::Text),
    ]
}

fn arb_op() -> impl Strategy<Value = CompOp> {
    prop::sample::select(vec![
        CompOp::Is,
        CompOp::IsNot,
        CompOp::IsBelow,
        CompOp::IsAbove,
        CompOp::IsAtLeast,
        CompOp::IsAtMost,
        CompOp::IsOneOf,
        CompOp::Contains,
    ])
}

fn cond(kind: CondKind) -> Cond {
    Cond { kind, span: sp() }
}

/// Grammar `Cmp` without `not`.
fn arb_cmp_leaf() -> BoxedStrategy<Cond> {
    prop_oneof![
        arb_app().prop_map(|a| cond(CondKind::App(a))),
        (arb_app(), arb_app()).prop_map(|(left, right)| cond(CondKind::In { left, right })),
        (arb_app(), arb_op(), arb_app()).prop_map(|(left, op, right)| cond(CondKind::Compare {
            left,
            op,
            right
        })),
    ]
    .boxed()
}

/// Grammar `Cmp`: `not` applies to a `Cmp`.
fn arb_cmp(depth: u32) -> BoxedStrategy<Cond> {
    if depth == 0 {
        return arb_cmp_leaf();
    }
    let not = arb_cmp(depth - 1).prop_map(|c| cond(CondKind::Not(Box::new(c))));
    prop_oneof![arb_cmp_leaf(), not].boxed()
}

/// Grammar `Conj`: left-associative, right child at `Cmp` level.
fn arb_conj(depth: u32) -> BoxedStrategy<Cond> {
    if depth == 0 {
        return arb_cmp(1);
    }
    let and = (arb_conj(depth - 1), arb_cmp(1))
        .prop_map(|(a, b)| cond(CondKind::And(Box::new(a), Box::new(b))));
    prop_oneof![arb_cmp(1), and].boxed()
}

/// Grammar `Disj`: left-associative, right child at `Conj` level.
fn arb_cond() -> BoxedStrategy<Cond> {
    let or =
        (arb_conj(1), arb_conj(1)).prop_map(|(a, b)| cond(CondKind::Or(Box::new(a), Box::new(b))));
    let or_or =
        (or.clone(), arb_conj(0)).prop_map(|(a, b)| cond(CondKind::Or(Box::new(a), Box::new(b))));
    prop_oneof![arb_conj(1), or, or_or].boxed()
}

fn stmt(kind: StmtKind) -> Stmt {
    Stmt { kind, span: sp() }
}

fn arb_verb() -> impl Strategy<Value = EffectVerb> {
    prop::sample::select(vec![EffectVerb::Read, EffectVerb::Write, EffectVerb::Call])
}

/// The header and binding statements.
fn arb_header_or_binding() -> BoxedStrategy<StmtKind> {
    prop_oneof![
        arb_app().prop_map(StmtKind::RunsOn),
        arb_quantity().prop_map(StmtKind::Every),
        (arb_verb(), arb_phrase())
            .prop_map(|(verb, target)| StmtKind::May(Effect { verb, target })),
        arb_quantity().prop_map(StmtKind::WaitUpTo),
        (arb_phrase(), arb_cond()).prop_map(|(name, value)| StmtKind::Let { name, value }),
        (arb_phrase(), arb_cond()).prop_map(|(name, value)| StmtKind::Set { name, value }),
    ]
    .boxed()
}

/// `expect`, `give back`, `stop with`, `given`, `then`.
fn arb_cond_stmt() -> BoxedStrategy<StmtKind> {
    prop_oneof![
        arb_cond().prop_map(StmtKind::Expect),
        arb_cond().prop_map(StmtKind::GiveBack),
        arb_cond().prop_map(StmtKind::StopWith),
        arb_cond().prop_map(StmtKind::Given),
        arb_cond().prop_map(StmtKind::Then),
    ]
    .boxed()
}

/// An action; `with` bodies only when `depth > 0`.
fn arb_action(depth: u32) -> BoxedStrategy<StmtKind> {
    let with = if depth == 0 {
        Just(None).boxed()
    } else {
        prop::option::of(arb_body(depth - 1)).boxed()
    };
    (arb_app(), prop::option::of(arb_app()), with)
        .prop_map(|(head, target, with)| StmtKind::Action(Action { head, target, with }))
        .boxed()
}

fn arb_simple() -> BoxedStrategy<StmtKind> {
    prop_oneof![arb_header_or_binding(), arb_cond_stmt(), arb_action(0)].boxed()
}

/// The block statements, bodies one level shallower.
fn arb_block(depth: u32) -> BoxedStrategy<StmtKind> {
    let body = arb_body(depth - 1);
    prop_oneof![
        (arb_cond(), body.clone(), prop::option::of(body.clone())).prop_map(
            |(cond, then_body, otherwise)| StmtKind::When {
                cond,
                then_body,
                otherwise
            }
        ),
        (arb_phrase(), arb_cond(), body.clone()).prop_map(|(name, collection, body)| {
            StmtKind::ForEach {
                name,
                collection,
                body,
            }
        }),
        (arb_int(), prop::option::of(arb_cond()), body.clone())
            .prop_map(|(times, until, body)| StmtKind::Repeat { times, until, body }),
        (arb_text(), body).prop_map(|(name, body)| StmtKind::Example { name, body }),
        arb_action(depth),
    ]
    .boxed()
}

fn arb_stmt(depth: u32) -> BoxedStrategy<Stmt> {
    if depth == 0 {
        return arb_simple().prop_map(stmt).boxed();
    }
    prop_oneof![3 => arb_simple(), 1 => arb_block(depth)]
        .prop_map(stmt)
        .boxed()
}

fn arb_body(depth: u32) -> BoxedStrategy<Vec<Stmt>> {
    prop::collection::vec(arb_stmt(depth), 1..4).boxed()
}

fn arb_kind() -> impl Strategy<Value = UnitKind> {
    prop::sample::select(vec![
        UnitKind::Job,
        UnitKind::Command,
        UnitKind::Check,
        UnitKind::Pipeline,
        UnitKind::Shape,
    ])
}

fn arb_decl() -> BoxedStrategy<Decl> {
    let unit = (arb_kind(), arb_text(), arb_body(2)).prop_map(|(kind, name, body)| {
        Decl::Unit(Unit {
            kind,
            name,
            body,
            span: sp(),
        })
    });
    let use_decl = arb_phrase().prop_map(|vocabulary| {
        Decl::Use(UseDecl {
            vocabulary,
            span: sp(),
        })
    });
    prop_oneof![1 => use_decl, 2 => unit].boxed()
}

pub(crate) fn arb_program() -> impl Strategy<Value = Program> {
    prop::collection::vec(arb_decl(), 1..4).prop_map(|decls| Program { decls })
}

/// `PROPTEST_CASES` if set, else 100.
fn cases() -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(cases()))]

    #[test]
    fn test_rhl_1_fmt_prop_format_reparses_to_the_same_tree(t in arb_program()) {
        let text = format(&t);
        let back = parse(&text).map_err(|e| TestCaseError::fail(format!("{e:?}\n{text}")))?;
        prop_assert_eq!(
            serde_json::to_value(&back).expect("tree serializes"),
            serde_json::to_value(&t).expect("tree serializes"),
            "{}", text
        );
    }

    #[test]
    fn test_rhl_1_fmt_prop_format_is_idempotent(t in arb_program()) {
        let text = format(&t);
        let back = parse(&text).map_err(|e| TestCaseError::fail(format!("{e:?}\n{text}")))?;
        prop_assert_eq!(format(&back), text);
    }
}

// ───────────────────────── e. blank-line rule ─────────────────────────

fn body_of(src: &str) -> String {
    fmt_ok(src, "edge case")
}

#[test]
fn test_rhl_1_fmt_blank_no_blank_before_unit_end() {
    let src = "job \"j\"\n  when a is b\n    expect c\n  end\nend\n";
    assert_eq!(body_of(src), src);
}

#[test]
fn test_rhl_1_fmt_blank_two_expects_have_none() {
    let src = "job \"j\"\n  expect a\n\n  expect b\nend\n";
    assert_eq!(body_of(src), "job \"j\"\n  expect a\n  expect b\nend\n");
}

#[test]
fn test_rhl_1_fmt_blank_header_then_let_has_one() {
    let src = "job \"j\"\n  every 1 hour\n  let a be b\nend\n";
    assert_eq!(
        body_of(src),
        "job \"j\"\n  every 1 hour\n\n  let a be b\nend\n"
    );
}

#[test]
fn test_rhl_1_fmt_blank_let_then_when_has_one() {
    let src = "job \"j\"\n  let a be b\n  when a\n    expect c\n  end\nend\n";
    let want = "job \"j\"\n  let a be b\n\n  when a\n    expect c\n  end\nend\n";
    assert_eq!(body_of(src), want);
}

#[test]
fn test_rhl_1_fmt_blank_nested_when_body_has_none() {
    let src = "job \"j\"\n  when a\n    let x be y\n\n    expect c\n    file t with\n      title \"x\"\n    end\n  end\nend\n";
    let want = "job \"j\"\n  when a\n    let x be y\n    expect c\n    file t with\n      title \"x\"\n    end\n  end\nend\n";
    assert_eq!(body_of(src), want);
}

#[test]
fn test_rhl_1_fmt_blank_units_separated_by_one_blank_line() {
    let src = "use vocabulary a v1\nuse vocabulary b v1\njob \"j\"\n  expect a\nend\n\n\n\ncheck \"k\"\n  expect b\nend";
    let want = "use vocabulary a v1\nuse vocabulary b v1\n\njob \"j\"\n  expect a\nend\n\ncheck \"k\"\n  expect b\nend\n";
    assert_eq!(body_of(src), want);
}

#[test]
fn test_rhl_1_fmt_otherwise_and_repeat_and_for_each_shapes() {
    let src = "job \"j\"\n  when a\n    expect b\n  otherwise\n    stop with c\n  end\n  repeat at most 3 times until d\n    wait up to 5 %\n  end\n  for each x in y and not z or w\n    give back x\n  end\nend\n";
    let want = "job \"j\"\n  when a\n    expect b\n  otherwise\n    stop with c\n  end\n\n  repeat at most 3 times until d\n    wait up to 5 %\n  end\n\n  for each x in y and not z or w\n    give back x\n  end\nend\n";
    assert_eq!(body_of(src), want);
}

#[test]
fn test_rhl_1_fmt_format_is_total_on_trees_the_parser_cannot_produce() {
    let or = cond(CondKind::Or(
        Box::new(cond(CondKind::App(App::Text(arb_free_text("a"))))),
        Box::new(cond(CondKind::App(App::Text(arb_free_text("b"))))),
    ));
    let and = cond(CondKind::And(Box::new(or.clone()), Box::new(or)));
    let empty_unit = Decl::Unit(Unit {
        kind: UnitKind::Job,
        name: arb_free_text("j"),
        body: vec![stmt(StmtKind::Expect(and))],
        span: sp(),
    });
    let out = format(&Program {
        decls: vec![empty_unit],
    });
    assert_eq!(
        out,
        "job \"j\"\n  expect \"a\" or \"b\" and \"a\" or \"b\"\nend\n"
    );
    assert_eq!(format(&Program { decls: vec![] }), "");
}

fn arb_free_text(value: &str) -> Text {
    Text {
        value: value.to_string(),
        span: sp(),
    }
}
