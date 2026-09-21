//! RHL-1 parser tests (spec RHL-001, plan D1). Lib tests, not `tests/`: the
//! required CI check runs `cargo test --lib` only (`docs/rhl/phase0-bindings.md` B4).
//!
//! Four groups: the skeleton-identity gate that ties `src/rhl/grammar.lalrpop`
//! to the pre-registered `grammar/rhl.lalrpop` (with its positive controls);
//! F1's second half, that the committed corpus parses and the `missing-end`
//! breaks do not; tree shape; and the mapping of LALRPOP errors.

use super::parse::{parse, ParseFailure};
use super::tree::{App, Atom, CompOp, Cond, CondKind, Decl, Program, Stmt, StmtKind, UnitKind};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const PRE_REGISTERED: &str = "grammar/rhl.lalrpop";
const PRODUCTION: &str = "src/rhl/grammar.lalrpop";
const SPEC: &str = "docs/specifications/ruchy-high-level-language-interface.md";
const BREAKS: &str = "docs/rhl/breaks";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("RHL-1: cannot read {}: {e}", path.display()))
}

// ───────────────────────── .lalrpop tokenizer ─────────────────────────

/// Split `.lalrpop` text into tokens: identifiers, string and raw-string
/// literals (kept verbatim), `@L`/`@R`, `=>`, `::` and single punctuation.
/// Whitespace and `//` comments are dropped.
fn tokenize(src: &str) -> Vec<String> {
    let b = src.as_bytes();
    let (mut i, mut out) = (0, Vec::new());
    while i < b.len() {
        match trivia_end(b, i) {
            Some(end) => i = end,
            None => {
                let end = token_end(b, i);
                out.push(String::from_utf8_lossy(&b[i..end]).into_owned());
                i = end;
            }
        }
    }
    out
}

fn trivia_end(b: &[u8], i: usize) -> Option<usize> {
    if b[i].is_ascii_whitespace() {
        return Some(i + 1);
    }
    if b[i] == b'/' && b.get(i + 1) == Some(&b'/') {
        let rest = b[i..].iter().position(|&c| c == b'\n');
        return Some(rest.map_or(b.len(), |n| i + n));
    }
    None
}

fn is_ident(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

fn ident_end(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && is_ident(b[i]) {
        i += 1;
    }
    i
}

fn token_end(b: &[u8], i: usize) -> usize {
    literal_end(b, i).unwrap_or_else(|| word_or_punct_end(b, i))
}

/// End of a string, raw-string or char literal starting at `i`, if one does.
fn literal_end(b: &[u8], i: usize) -> Option<usize> {
    match b[i] {
        b'r' if matches!(b.get(i + 1), Some(b'"' | b'#')) => Some(raw_end(b, i)),
        b'"' => Some(str_end(b, i)),
        b'\'' if b.get(i + 2) == Some(&b'\'') => Some(i + 3),
        _ => None,
    }
}

/// End of an identifier, `@L`/`@R`, a lifetime, `=>`, `::` or one punctuation byte.
fn word_or_punct_end(b: &[u8], i: usize) -> usize {
    let rest = &b[i..];
    match b[i] {
        b'@' | b'\'' => ident_end(b, i + 1),
        c if is_ident(c) => ident_end(b, i),
        _ if rest.starts_with(b"=>") || rest.starts_with(b"::") => i + 2,
        _ => i + 1,
    }
}

/// End of `r"…"` / `r#"…"#` starting at `i`.
fn raw_end(b: &[u8], i: usize) -> usize {
    let hashes = b[i + 1..].iter().take_while(|&&c| c == b'#').count();
    let mut j = i + 2 + hashes;
    let closes = |j: usize| b[j] == b'"' && b[j + 1..].iter().take(hashes).all(|&c| c == b'#');
    while j + hashes < b.len() && !closes(j) {
        j += 1;
    }
    (j + 1 + hashes).min(b.len())
}

/// End of `"…"` starting at `i`, honouring backslash escapes.
fn str_end(b: &[u8], i: usize) -> usize {
    let mut j = i + 1;
    while j < b.len() && b[j] != b'"' {
        j += if b[j] == b'\\' { 2 } else { 1 };
    }
    (j + 1).min(b.len())
}

// ───────────────────────── skeleton extraction ─────────────────────────

/// Alternatives of one nonterminal, each an ordered list of grammar symbols.
type Alternatives = Vec<Vec<String>>;

/// What must be identical between the two grammars: the lexer `match` block
/// (as tokens) and every nonterminal's ordered alternatives.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Skeleton {
    matcher: Vec<String>,
    rules: BTreeMap<String, Alternatives>,
}

struct Cursor<'a> {
    toks: &'a [String],
    i: usize,
}

impl Cursor<'_> {
    fn peek(&self) -> Option<&str> {
        self.toks.get(self.i).map(String::as_str)
    }

    fn at(&self, tok: &str) -> bool {
        self.peek() == Some(tok)
    }

    fn bump(&mut self) -> Result<String, String> {
        let t = self.peek().ok_or("unexpected end of grammar")?.to_string();
        self.i += 1;
        Ok(t)
    }

    fn expect(&mut self, tok: &str) -> Result<(), String> {
        let got = self.bump()?;
        if got == tok {
            Ok(())
        } else {
            Err(format!(
                "expected `{tok}`, found `{got}` at token {}",
                self.i - 1
            ))
        }
    }

    /// Consume one bracketed group (`(`, `[` or `{` … its match), returning its tokens.
    fn group(&mut self) -> Result<Vec<String>, String> {
        let start = self.i;
        let mut depth = 0i32;
        loop {
            depth += bracket_delta(&self.bump()?);
            if depth == 0 {
                return Ok(self.toks[start..self.i].to_vec());
            }
        }
    }
}

fn bracket_delta(tok: &str) -> i32 {
    match tok {
        "(" | "[" | "{" => 1,
        ")" | "]" | "}" => -1,
        _ => 0,
    }
}

/// Normalize a `.lalrpop` file to its [`Skeleton`].
fn skeleton(src: &str) -> Result<Skeleton, String> {
    let toks = tokenize(src);
    let mut c = Cursor { toks: &toks, i: 0 };
    let mut matcher = None;
    let mut rules = BTreeMap::new();
    while let Some(tok) = c.peek() {
        match tok {
            "use" | "grammar" => skip_past_semicolon(&mut c)?,
            "match" => matcher = Some(match_block(&mut c)?),
            "extern" => return Err("an `extern` block changes the lexer".into()),
            _ => insert_rule(&mut c, &mut rules)?,
        }
    }
    let matcher = matcher.ok_or("no `match` block")?;
    Ok(Skeleton { matcher, rules })
}

/// Read one rule and add it, refusing a nonterminal defined twice.
fn insert_rule(
    c: &mut Cursor<'_>,
    rules: &mut BTreeMap<String, Alternatives>,
) -> Result<(), String> {
    let (name, alts) = rule(c)?;
    if rules.insert(name.clone(), alts).is_some() {
        return Err(format!("nonterminal `{name}` defined twice"));
    }
    Ok(())
}

fn skip_past_semicolon(c: &mut Cursor<'_>) -> Result<(), String> {
    while c.bump()? != ";" {}
    Ok(())
}

fn match_block(c: &mut Cursor<'_>) -> Result<Vec<String>, String> {
    let mut out = vec![c.bump()?];
    out.extend(c.group()?);
    while c.at("else") {
        out.push(c.bump()?);
        out.extend(c.group()?);
    }
    Ok(out)
}

/// `[pub] Name [: Type] = { alt, … };`
fn rule(c: &mut Cursor<'_>) -> Result<(String, Alternatives), String> {
    if c.at("pub") {
        c.bump()?;
    }
    let name = c.bump()?;
    if c.at(":") {
        skip_type(c)?;
    }
    c.expect("=")?;
    c.expect("{")?;
    let mut alts = Vec::new();
    while !c.at("}") {
        alts.push(alternative(c)?);
    }
    c.expect("}")?;
    c.expect(";")?;
    Ok((name, alts))
}

/// Skip `: Type` up to the `=` that opens the rule body.
fn skip_type(c: &mut Cursor<'_>) -> Result<(), String> {
    let mut depth = 0i32;
    while !(depth == 0 && c.at("=")) {
        let t = c.bump()?;
        depth += bracket_delta(&t) + i32::from(t == "<") - i32::from(t == ">");
    }
    Ok(())
}

/// One alternative's symbols; its action and trailing `,` are consumed.
fn alternative(c: &mut Cursor<'_>) -> Result<Vec<String>, String> {
    let mut symbols = Vec::new();
    while !(c.at("=>") || c.at(",") || c.at("}")) {
        symbols.extend(symbol(c)?);
    }
    if c.at("=>") {
        skip_action(c)?;
    }
    if c.at(",") {
        c.bump()?;
    }
    Ok(symbols)
}

/// The next grammar symbol, or `None` for binding brackets and `@L`/`@R`.
fn symbol(c: &mut Cursor<'_>) -> Result<Option<String>, String> {
    let t = c.bump()?;
    if t == "<" {
        skip_binding_name(c);
        return Ok(None);
    }
    Ok((!matches!(t.as_str(), ">" | "@L" | "@R")).then_some(t))
}

/// After `<`: drop `name:` or `mut name:` if present.
fn skip_binding_name(c: &mut Cursor<'_>) {
    let off = usize::from(c.at("mut"));
    if c.toks.get(c.i + off + 1).map(String::as_str) == Some(":") {
        c.i += off + 2;
    }
}

/// Skip `=> code` up to the `,` or `}` that ends the alternative.
fn skip_action(c: &mut Cursor<'_>) -> Result<(), String> {
    let mut depth = 0i32;
    while !(depth == 0 && (c.at(",") || c.at("}"))) {
        depth += bracket_delta(&c.bump()?);
    }
    Ok(())
}

/// `Ok` when the skeletons are identical, else every difference found.
fn compare(pre: &Skeleton, prod: &Skeleton) -> Result<(), String> {
    let mut diffs = header_diffs(pre, prod);
    diffs.extend(rule_diffs(pre, prod));
    if diffs.is_empty() {
        Ok(())
    } else {
        Err(diffs.join("\n"))
    }
}

/// Differences in the lexer block or in the set of nonterminal names.
fn header_diffs(pre: &Skeleton, prod: &Skeleton) -> Vec<String> {
    let mut diffs = Vec::new();
    if pre.matcher != prod.matcher {
        diffs.push("the lexer `match` blocks differ".to_string());
    }
    let names = |s: &Skeleton| s.rules.keys().cloned().collect::<Vec<_>>();
    if names(pre) != names(prod) {
        diffs.push(format!(
            "nonterminals differ: {:?} vs {:?}",
            names(pre),
            names(prod)
        ));
    }
    diffs
}

/// Nonterminals present in both whose alternatives differ.
fn rule_diffs(pre: &Skeleton, prod: &Skeleton) -> Vec<String> {
    pre.rules
        .iter()
        .filter_map(|(name, alts)| {
            let other = prod.rules.get(name).filter(|o| *o != alts)?;
            Some(format!("`{name}` differs: {alts:?} vs {other:?}"))
        })
        .collect()
}

fn skeleton_of(rel: &str) -> Skeleton {
    skeleton(&read(&root().join(rel)))
        .unwrap_or_else(|e| panic!("RHL-1: cannot normalize {rel}: {e}"))
}

#[test]
fn test_rhl_1_skeleton_production_grammar_is_identical_to_pre_registered() {
    let (pre, prod) = (skeleton_of(PRE_REGISTERED), skeleton_of(PRODUCTION));
    if let Err(diff) = compare(&pre, &prod) {
        panic!(
            "F1 BROKEN: {PRODUCTION} no longer has the skeleton of {PRE_REGISTERED}, so \
             LALRPOP's conflict-free report is about a different grammar than the parser \
             that runs. Change only action code, bindings, @L/@R and result types.\n{diff}"
        );
    }
}

#[test]
fn test_rhl_1_skeleton_normalizer_is_not_vacuous() {
    let pre = skeleton_of(PRE_REGISTERED);
    assert_eq!(pre.rules.len(), 20, "nonterminals: {:?}", pre.rules.keys());
    assert_eq!(pre.rules["Stmt"].len(), 15);
    assert_eq!(pre.rules["UnitKind"].len(), 5);
    assert_eq!(
        pre.rules["Decl"][0],
        ["\"use vocabulary\"", "Phrase", "NEWLINE"]
    );
    assert_eq!(pre.rules["Quantity"][3], ["INT", "\"%\""]);
    assert_eq!(pre.matcher.iter().filter(|t| *t == "else").count(), 2);
    assert!(pre.matcher.contains(&r####"r#""[^"]*""#"####.to_string()));
}

#[test]
fn test_rhl_1_skeleton_gate_goes_red_on_a_dropped_symbol() {
    let pre = skeleton_of(PRE_REGISTERED);
    let mut mutated = skeleton_of(PRODUCTION);
    let dropped = mutated.rules.get_mut("Decl").expect("Decl exists")[0].pop();
    assert_eq!(dropped.as_deref(), Some("NEWLINE"));
    assert!(
        compare(&pre, &mutated).is_err(),
        "POSITIVE CONTROL BROKEN: dropping a symbol from one alternative was not detected"
    );
}

#[test]
fn test_rhl_1_skeleton_gate_goes_red_on_a_reordered_lexer_tier() {
    let pre = skeleton_of(PRE_REGISTERED);
    let mut mutated = skeleton_of(PRODUCTION);
    let is = mutated
        .matcher
        .iter()
        .position(|t| t == "\"is\"")
        .expect("`is` is a keyword");
    let is_not = mutated
        .matcher
        .iter()
        .position(|t| t == "\"is not\"")
        .expect("`is not`");
    mutated.matcher.swap(is, is_not);
    assert!(
        compare(&pre, &mutated).is_err(),
        "a reordered `match` tier was not detected"
    );
}

#[test]
fn test_rhl_1_production_grammar_has_no_fallible_action() {
    let toks = tokenize(&read(&root().join(PRODUCTION)));
    let fallible = toks.windows(2).any(|w| w[0] == "=>" && w[1] == "?");
    assert!(
        !fallible,
        "{PRODUCTION} has a `=>?` action; parse.rs maps ParseError::User as unreachable"
    );
}

// ───────────────────────── F1 second half: corpus ─────────────────────────

fn rhl_files(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("RHL-1: cannot list {}: {e}", dir.display()))
        .map(|e| e.expect("RHL-1: readable dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "rhl"))
        .collect();
    v.sort();
    v
}

/// `planted/<class>/*/broken.rhl`, for the classes `keep` selects.
fn planted(keep: impl Fn(&str) -> bool) -> Vec<PathBuf> {
    let base = root().join(BREAKS).join("planted");
    let mut out = Vec::new();
    for class in std::fs::read_dir(&base)
        .expect("RHL-1: planted dir")
        .flatten()
    {
        if !keep(&class.file_name().to_string_lossy()) {
            continue;
        }
        for case in std::fs::read_dir(class.path())
            .expect("RHL-1: class dir")
            .flatten()
        {
            out.push(case.path().join("broken.rhl"));
        }
    }
    out.sort();
    out
}

fn failures(files: &[PathBuf]) -> Vec<String> {
    files
        .iter()
        .filter_map(|p| {
            parse(&read(p))
                .err()
                .map(|e| format!("{}: {e:?}", p.display()))
        })
        .collect()
}

#[test]
fn test_rhl_1_corpus_every_valid_program_parses() {
    let files = rhl_files(&root().join(BREAKS).join("valid"));
    assert_eq!(files.len(), 12, "valid programs: {files:?}");
    let bad = failures(&files);
    assert!(
        bad.is_empty(),
        "valid programs that do not parse:\n{}",
        bad.join("\n")
    );
}

#[test]
fn test_rhl_1_corpus_non_syntax_planted_breaks_parse() {
    let files = planted(|class| class != "missing-end");
    assert_eq!(files.len(), 60, "five classes x 12: {files:?}");
    let bad = failures(&files);
    assert!(
        bad.is_empty(),
        "type/vocabulary breaks rejected by the parser:\n{}",
        bad.join("\n")
    );
}

#[test]
fn test_rhl_1_corpus_missing_end_breaks_fail_at_eof_expecting_end() {
    let files = planted(|class| class == "missing-end");
    assert_eq!(files.len(), 12);
    for p in &files {
        match parse(&read(p)) {
            Err(ParseFailure::UnrecognizedEof { expected, .. }) => assert!(
                expected.iter().any(|t| t == "\"end\""),
                "{}: expected set lacks `end`: {expected:?}",
                p.display()
            ),
            other => panic!("{}: expected UnrecognizedEof, got {other:?}", p.display()),
        }
    }
}

/// The fenced code block under `### 3.2 Example` in the spec.
fn spec_example() -> String {
    let spec = read(&root().join(SPEC));
    let after = spec
        .split_once("### 3.2 Example")
        .expect("spec has §3.2 Example")
        .1;
    let body = after.split_once("```").expect("§3.2 has a code fence").1;
    let body = body.split_once('\n').expect("fence line ends").1;
    body.split_once("```")
        .expect("§3.2 fence closes")
        .0
        .to_string()
}

#[test]
fn test_rhl_1_corpus_spec_section_3_2_example_parses() {
    let src = spec_example();
    assert!(
        src.contains("job \"gx10 disk watch\""),
        "wrong block extracted: {src}"
    );
    let program = parse(&src).unwrap_or_else(|e| panic!("§3.2 example does not parse: {e:?}"));
    assert_eq!(program.decls.len(), 3);
}

// ───────────────────────── tree shape ─────────────────────────

fn words(p: &super::tree::Phrase) -> Vec<&str> {
    p.words.iter().map(|w| w.text.as_str()).collect()
}

fn gx10() -> (String, Program) {
    let src = read(&root().join(BREAKS).join("valid/01-gx10-disk-watch.rhl"));
    let program = parse(&src).expect("01-gx10-disk-watch.rhl parses");
    (src, program)
}

fn gx10_body(program: &Program) -> &[Stmt] {
    match &program.decls[2] {
        Decl::Unit(u) => &u.body,
        other => panic!("third decl is not a unit: {other:?}"),
    }
}

#[test]
fn test_rhl_1_tree_gx10_decls() {
    let (_, program) = gx10();
    assert_eq!(program.decls.len(), 3);
    let uses: Vec<String> = program.decls[..2]
        .iter()
        .map(|d| match d {
            Decl::Use(u) => u.vocabulary.text(),
            other => panic!("expected a use decl, got {other:?}"),
        })
        .collect();
    assert_eq!(uses, ["fleet v1", "tickets v1"]);
    let Decl::Unit(unit) = &program.decls[2] else {
        panic!("third decl is not a unit")
    };
    assert_eq!(unit.kind, UnitKind::Job);
    assert_eq!(unit.name.value, "gx10 disk watch");
}

#[test]
fn test_rhl_1_tree_gx10_let_statement_and_phrase_span() {
    let (src, program) = gx10();
    let StmtKind::Let { name, value } = &gx10_body(&program)[5].kind else {
        panic!("sixth statement is not `let`")
    };
    assert_eq!(words(name), ["free"]);
    let CondKind::App(App::Call { phrase, args }) = &value.kind else {
        panic!("let value is not an application: {value:?}")
    };
    assert_eq!(words(phrase), ["disk", "free", "of"]);
    assert!(matches!(args.as_slice(), [Atom::Text(t)] if t.value == "/"));
    assert_eq!(&src[phrase.span.start..phrase.span.end], "disk free of");
}

/// The `when free is below 100 GB … end` statement: its condition and body.
fn gx10_when(program: &Program) -> (Cond, Vec<Stmt>) {
    match &gx10_body(program)[6].kind {
        StmtKind::When {
            cond,
            then_body,
            otherwise: None,
        } => (cond.clone(), then_body.clone()),
        other => panic!("seventh statement is not a plain `when`: {other:?}"),
    }
}

#[test]
fn test_rhl_1_tree_gx10_quantity_keeps_unit_spelling() {
    let (cond, _) = gx10_when(&gx10().1);
    let CondKind::Compare {
        op: CompOp::IsBelow,
        right: App::Quantity(q),
        ..
    } = &cond.kind
    else {
        panic!("when condition is not `… is below <quantity>`: {cond:?}")
    };
    assert_eq!(q.value.digits, "100");
    assert_eq!(q.unit.as_ref().map(|w| w.text.as_str()), Some("GB"));
}

#[test]
fn test_rhl_1_tree_gx10_action_head_target_and_with_block() {
    let (_, body) = gx10_when(&gx10().1);
    let StmtKind::Action(action) = &body[0].kind else {
        panic!("not an action: {:?}", body[0])
    };
    let App::Call { phrase, args } = &action.head else {
        panic!("head is not a call")
    };
    assert_eq!((words(phrase), args.len()), (vec!["file", "ticket"], 0));
    let Some(App::Call { phrase, args }) = &action.target else {
        panic!("no `in` target")
    };
    assert_eq!(words(phrase), ["repo"]);
    assert!(matches!(args.as_slice(), [Atom::Text(t)] if t.value == "paiml/infra"));
    assert_eq!(action.with.as_ref().map(Vec::len), Some(2));
}

/// The value of `let v be <expr>` inside a one-statement job.
fn let_value(expr: &str) -> Cond {
    let src = format!("job \"x\"\n  let v be {expr}\nend\n");
    let program = parse(&src).unwrap_or_else(|e| panic!("`{expr}` does not parse: {e:?}"));
    let Decl::Unit(unit) = &program.decls[0] else {
        panic!("not a unit")
    };
    match &unit.body[0].kind {
        StmtKind::Let { value, .. } => value.clone(),
        other => panic!("not a let: {other:?}"),
    }
}

/// `(or a (and b c))` rendering of a condition over bare words.
fn show(c: &Cond) -> String {
    match &c.kind {
        CondKind::Or(a, b) => format!("(or {} {})", show(a), show(b)),
        CondKind::And(a, b) => format!("(and {} {})", show(a), show(b)),
        CondKind::Not(a) => format!("(not {})", show(a)),
        CondKind::App(App::Call { phrase, .. }) => phrase.text(),
        other => format!("{other:?}"),
    }
}

#[test]
fn test_rhl_1_tree_int_drops_leading_zeros() {
    for (written, digits) in [("007", "7"), ("0", "0"), ("000", "0"), ("100", "100")] {
        let CondKind::App(App::Quantity(q)) = let_value(written).kind else {
            panic!("`{written}` is not a quantity")
        };
        assert_eq!(q.value.digits, digits, "for `{written}`");
        assert_eq!(q.unit, None);
    }
}

#[test]
fn test_rhl_1_tree_and_binds_tighter_than_or() {
    assert_eq!(show(&let_value("a or b and c")), "(or a (and b c))");
    assert_eq!(show(&let_value("a and b or c")), "(or (and a b) c)");
}

#[test]
fn test_rhl_1_tree_or_and_are_left_associative() {
    assert_eq!(show(&let_value("a or b or c")), "(or (or a b) c)");
    assert_eq!(show(&let_value("a and b and c")), "(and (and a b) c)");
}

#[test]
fn test_rhl_1_tree_units_keep_their_spelling() {
    for (written, unit) in [("5 %", "%"), ("1 hour", "hour"), ("100 GB", "GB")] {
        let CondKind::App(App::Quantity(q)) = let_value(written).kind else {
            panic!("`{written}` is not a quantity")
        };
        assert_eq!(q.unit.map(|w| w.text), Some(unit.to_string()));
    }
}

// ───────────────────────── error mapping ─────────────────────────

#[test]
fn test_rhl_1_error_missing_end_is_unrecognized_eof() {
    let err = parse("job \"x\"\n  every 1 hour\n").expect_err("no `end`");
    let ParseFailure::UnrecognizedEof { expected, .. } = err else {
        panic!("got {err:?}")
    };
    assert!(expected.iter().any(|t| t == "\"end\""), "{expected:?}");
}

#[test]
fn test_rhl_1_error_unknown_character_is_invalid_token() {
    let src = "job \"x\"\n  every 1 hour !\nend";
    let err = parse(src).expect_err("`!` is no terminal");
    assert_eq!(
        err,
        ParseFailure::InvalidToken {
            at: src.find('!').expect("has !")
        }
    );
}

#[test]
fn test_rhl_1_error_misplaced_token_is_unrecognized_token() {
    let src = "job \"x\"\n  every hour\nend";
    let err = parse(src).expect_err("`every` needs a quantity");
    let ParseFailure::UnrecognizedToken {
        span,
        found,
        expected,
    } = err
    else {
        panic!("got {err:?}")
    };
    assert_eq!(found, "hour");
    assert_eq!(&src[span.start..span.end], "hour");
    assert!(expected.iter().any(|t| t == "INT"), "{expected:?}");
}

#[test]
fn test_rhl_1_error_non_ascii_inside_string_parses_outside_is_invalid() {
    let program = parse("job \"gx10 über\"\n  every 1 hour\nend\n").expect("non-ASCII in a string");
    let Decl::Unit(unit) = &program.decls[0] else {
        panic!("not a unit")
    };
    assert_eq!(unit.name.value, "gx10 über");
    let src = "job \"x\"\n  let v be é\nend\n";
    assert_eq!(
        parse(src),
        Err(ParseFailure::InvalidToken {
            at: src.find('é').expect("é")
        })
    );
}

#[test]
fn test_rhl_1_crlf_line_endings_parse_to_the_lf_tree_with_the_same_spans() {
    let lf = read(&root().join("docs/rhl/breaks/valid/01-gx10-disk-watch.rhl"));
    let crlf = lf.replace('\n', "\r\n");
    let (a, b) = (
        parse(&lf).expect("LF parses"),
        parse(&crlf).expect("CRLF parses"),
    );
    assert_eq!(
        serde_json::to_value(&a).expect("tree"),
        serde_json::to_value(&b).expect("tree")
    );
    let phrase_at = |p: &crate::rhl::tree::Program, src: &str| {
        let span = first_let_phrase_span(p);
        src[span.start..span.end].to_string()
    };
    assert_eq!(phrase_at(&b, &crlf), "disk free of");
}

#[test]
fn test_rhl_1_crlf_inside_a_string_is_kept() {
    let src = "job \"a\r\nb\"\r\n  every 1 hour\r\nend\r\n";
    let p = parse(src).expect("parses");
    let crate::rhl::tree::Decl::Unit(u) = &p.decls[0] else {
        panic!("a unit")
    };
    assert_eq!(u.name.value, "a\r\nb");
}

#[test]
fn test_rhl_1_lone_carriage_return_is_still_an_invalid_token() {
    let err = parse("job \"a\"\r  every 1 hour\nend\n").expect_err("lone CR");
    assert!(matches!(err, ParseFailure::InvalidToken { .. }), "{err:?}");
}

/// The span of the phrase in the first `let` statement's value.
fn first_let_phrase_span(p: &crate::rhl::tree::Program) -> crate::rhl::tree::Span {
    use crate::rhl::tree::{App, CondKind, Decl, StmtKind};
    let unit = p
        .decls
        .iter()
        .find_map(|d| match d {
            Decl::Unit(u) => Some(u),
            Decl::Use(_) => None,
        })
        .expect("a unit");
    let value = unit
        .body
        .iter()
        .find_map(|s| match &s.kind {
            StmtKind::Let { value, .. } => Some(value),
            _ => None,
        })
        .expect("a let");
    match &value.kind {
        CondKind::App(App::Call { phrase, .. }) => phrase.span,
        other => panic!("unexpected let value {other:?}"),
    }
}
