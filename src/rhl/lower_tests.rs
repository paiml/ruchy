//! RHL-4 lowering tests: one per construct of the emitted ruchy, the v2 valid
//! corpus through ruchy's own parser and transpiler (and `rustc`), the
//! determinism falsifier (RHL-001 F5) with its positive control, and the
//! meaning of §3.2's two examples.

use super::{lower, lower_source, lower_yaml, to_rust, LowerFailure};
use crate::rhl::codes;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------- helpers

fn root() -> PathBuf {
    crate::rhl::repo_root()
}

const V2_VALID: &str = "docs/rhl/breaks/v2/valid";

/// A job over both v2 vocabularies that declares every effect, so a body can
/// use any measure; `body` lines are indented by two spaces.
fn job(body: &str) -> String {
    format!(
        "use vocabulary fleet v2\nuse vocabulary tickets v2\n\njob \"t\"\n  runs on host gx10\n  \
         every 1 hour\n  may read disk\n  may read runner\n  may read host\n  \
         may read tickets\n  may write tickets\n\n{body}end\n"
    )
}

fn try_lower(body: &str) -> Result<String, LowerFailure> {
    lower_source("t.rhl", &job(body), Some(&root()))
}

fn lowered(body: &str) -> String {
    try_lower(body).unwrap_or_else(|f| panic!("RHL-4: expected a lowering, got {f:?}"))
}

/// The code of the refusal `body` must produce.
fn refused(body: &str) -> String {
    match try_lower(body) {
        Err(LowerFailure::Refused(d)) => {
            assert_eq!(d.code, codes::L001, "{d:?}");
            d.message
        }
        other => panic!("RHL-4: expected RHL-L001, got {other:?}"),
    }
}

/// Lower a parsed program directly, bypassing the check (for constructs the
/// checker already refuses, so that lowering's own refusal is observed).
fn lower_unchecked(source: &str) -> Result<String, crate::rhl::diag::Diagnostic> {
    let program = crate::rhl::parse(source).expect("RHL-4: test program parses");
    let lex = crate::rhl::check::load_lexicon("t.rhl", source, &program, Some(&root()));
    lower("t.rhl", source, &program, &lex)
}

fn v2_valid() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(root().join(V2_VALID))
        .expect("RHL-4: v2 valid corpus exists")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "rhl"))
        .collect();
    files.sort();
    files
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn lower_file(path: &Path) -> String {
    let file = path.display().to_string();
    lower_source(&file, &read(path), Some(&root()))
        .unwrap_or_else(|f| panic!("RHL-4: {file} does not lower: {f:?}"))
}

fn sha(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

/// `rustc`, if this host has one; the reason to skip otherwise.
fn rustc_available() -> bool {
    let ok = Command::new("rustc")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    if !ok {
        println!("RHL-4: skipped: no `rustc` on PATH to compile the transpiled Rust");
    }
    ok
}

fn rustc(dir: &Path, rs: &str, args: &[&str]) {
    let src = dir.join("j.rs");
    std::fs::write(&src, rs).expect("write j.rs");
    let out = Command::new("rustc")
        .args(["--edition", "2021"])
        .args(args)
        .arg(&src)
        .current_dir(dir)
        .output()
        .expect("rustc runs");
    assert!(
        out.status.success(),
        "RHL-4: rustc rejected the transpiled Rust:\n{}\n---\n{rs}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// ---------------------------------------------------------------- Facts

#[test]
fn test_rhl4_facts_one_field_per_distinct_measure_and_arguments() {
    let r = lowered(
        "  let a be disk free of \"/\"\n  let b be disk free of \"/\"\n  \
         let c be disk free of \"/data\"\n  let d be disk usage of \"/\"\n",
    );
    assert!(r.contains("struct Facts {\n    disk_free_of_0: i64,\n    disk_free_of_1: i64,\n    disk_usage_of_2: i64,\n}\n"), "{r}");
    assert!(r.contains("let v_a: i64 = facts.disk_free_of_0\n"), "{r}");
    assert!(r.contains("let v_b: i64 = facts.disk_free_of_0\n"), "{r}");
    assert!(r.contains("let v_c: i64 = facts.disk_free_of_1\n"), "{r}");
    assert!(r.contains("let v_d: i64 = facts.disk_usage_of_2\n"), "{r}");
    assert!(
        r.contains("// Facts.disk_free_of_0: disk free of \"/\" (Size, bytes)\n"),
        "{r}"
    );
    assert!(
        r.contains("// Facts.disk_free_of_1: disk free of \"/data\" (Size, bytes)\n"),
        "{r}"
    );
}

#[test]
fn test_rhl4_facts_types_text_percent_count_and_the_in_form() {
    let r = lowered(
        "  let s be pin status of \"gx13\"\n  let l be runner load of \"runner-01\"\n  \
         let n be tickets filed in \"paiml/infra\"\n  let m be tickets filed in repo \"paiml/infra\"\n",
    );
    assert!(r.contains("    pin_status_of_0: String,\n"), "{r}");
    assert!(r.contains("    runner_load_of_1: i64,\n"), "{r}");
    assert!(r.contains("    tickets_filed_in_2: i64,\n"), "{r}");
    assert!(
        !r.contains("tickets_filed_in_3"),
        "an entity argument is its instance: {r}"
    );
    assert!(
        r.contains("let v_s: String = facts.pin_status_of_0.clone()\n"),
        "{r}"
    );
    assert!(
        r.contains("let v_m: i64 = facts.tickets_filed_in_2\n"),
        "{r}"
    );
    assert!(r.contains("(Text)"), "{r}");
    assert!(r.contains("(Percent, hundredths of a percent)"), "{r}");
    assert!(r.contains("(Count)"), "{r}");
}

#[test]
fn test_rhl4_quantities_scale_to_the_base_unit() {
    let r = lowered("  let a be 100 GB\n  let d be 2 hour\n  let p be 90 %\n  let n be 3\n");
    assert!(r.contains("let v_a: i64 = 100000000000\n"), "{r}");
    assert!(r.contains("let v_d: i64 = 7200\n"), "{r}");
    assert!(r.contains("let v_p: i64 = 9000\n"), "{r}");
    assert!(r.contains("let v_n: i64 = 3\n"), "{r}");
}

// ---------------------------------------------------------------- conditions

const ORDERINGS: [(&str, &str, &str); 6] = [
    ("is", "==", "!="),
    ("is not", "!=", "=="),
    ("is below", "<", ">="),
    ("is above", ">", "<="),
    ("is at least", ">=", "<"),
    ("is at most", "<=", ">"),
];

fn when_free(cond: &str) -> String {
    lowered(&format!(
        "  let free be disk free of \"/\"\n  when {cond}\n    file ticket in repo \"a/b\"\n  end\n"
    ))
}

#[test]
fn test_rhl4_comparison_operators_map_one_to_one() {
    for (op, rust, _) in ORDERINGS {
        let r = when_free(&format!("free {op} 5 GB"));
        let want = format!("    if v_free {rust} 5000000000 {{\n");
        assert!(r.contains(&want), "`{op}` must lower to `{rust}`:\n{r}");
    }
}

#[test]
fn test_rhl4_not_inverts_the_comparison_it_applies_to() {
    for (op, _, inverse) in ORDERINGS {
        let r = when_free(&format!("not free {op} 5 GB"));
        let want = format!("    if v_free {inverse} 5000000000 {{\n");
        assert!(
            r.contains(&want),
            "`not … {op}` must lower to `{inverse}`:\n{r}"
        );
    }
    let r = when_free("not not free is below 5 GB");
    assert!(r.contains("    if v_free < 5000000000 {\n"), "{r}");
    let r = lowered(
        "  let free be disk free of \"/\"\n  let low be free is below 5 GB\n  \
         when not low\n    file ticket in repo \"a/b\"\n  end\n",
    );
    assert!(r.contains("let v_low: bool = v_free < 5000000000\n"), "{r}");
    assert!(r.contains("    if !v_low {\n"), "{r}");
}

#[test]
fn test_rhl4_and_or_contains_keep_rhl_precedence() {
    let r = lowered(
        "  let free be disk free of \"/\"\n  let s be pin status of \"gx13\"\n  \
         let t be \"pin\"\n  let n be tickets filed in \"paiml/infra\"\n  \
         when free is below 5 GB and s is \"x\" or n is above 3\n    file ticket in repo \"a/b\"\n  end\n  \
         when s contains \"pin\" and not s contains t\n    file ticket in repo \"a/b\"\n  end\n",
    );
    assert!(
        r.contains("    if v_free < 5000000000 && v_s == \"x\" || v_n > 3 {\n"),
        "{r}"
    );
    assert!(
        r.contains("    if v_s.contains(\"pin\") && !v_s.contains(&v_t) {\n"),
        "{r}"
    );
    assert!(r.contains("let v_t: String = \"pin\".to_string()\n"), "{r}");
}

// ---------------------------------------------------------------- statements

#[test]
fn test_rhl4_action_variant_has_takes_then_attributes_and_missing_is_empty() {
    let r = lowered(
        "  let free be disk free of \"/\"\n  when free is below 5 GB\n    \
         file ticket in repo \"paiml/infra\" with\n      title \"low\"\n    end\n  end\n",
    );
    assert!(
        r.contains(
            "enum Action {\n    FileTicket { repo: String, title: String, label: String },\n}\n"
        ),
        "{r}"
    );
    assert!(r.contains(
        "        plan.push(Action::FileTicket { repo: \"paiml/infra\".to_string(), title: \"low\".to_string(), label: String::new() })\n"
    ), "{r}");
}

#[test]
fn test_rhl4_let_is_mut_only_when_set_later_and_set_reassigns() {
    let r = lowered(
        "  let free be disk free of \"/\"\n  let limit be 5 GB\n  when free is below 5 GB\n    \
         set limit to 1 GB\n  otherwise\n    set limit to 2 GB\n  end\n",
    );
    assert!(
        r.contains("    let v_free: i64 = facts.disk_free_of_0\n"),
        "{r}"
    );
    assert!(r.contains("    let mut v_limit: i64 = 5000000000\n"), "{r}");
    assert!(r.contains(
        "    if v_free < 5000000000 {\n        v_limit = 1000000000\n    } else {\n        v_limit = 2000000000\n    }\n"
    ), "{r}");
}

#[test]
fn test_rhl4_empty_block_is_unit_not_an_empty_object() {
    let r = lowered(
        "  let free be disk free of \"/\"\n  when free is below 5 GB\n    \
         expect ticket count is at most 1\n  end\n",
    );
    assert!(
        r.contains("    if v_free < 5000000000 {\n        ()\n    }\n"),
        "{r}"
    );
}

#[test]
fn test_rhl4_repeat_is_a_counted_while_with_an_until_break() {
    let r = lowered(
        "  let free be disk free of \"/\"\n  let limit be 5 GB\n  \
         repeat at most 3 times until free is above 5 GB\n    set limit to 1 GB\n  end\n  \
         repeat at most 2 times\n    set limit to 2 GB\n  end\n",
    );
    assert!(r.contains(
        "    let mut repeat_0: i64 = 0\n    while repeat_0 < 3 {\n        if v_free > 5000000000 {\n            break\n        }\n        v_limit = 1000000000\n        repeat_0 = repeat_0 + 1\n    }\n"
    ), "{r}");
    assert!(r.contains(
        "    let mut repeat_1: i64 = 0\n    while repeat_1 < 2 {\n        v_limit = 2000000000\n        repeat_1 = repeat_1 + 1\n    }\n"
    ), "{r}");
}

#[test]
fn test_rhl4_header_comments_and_no_expect_or_example_in_decide() {
    let r = lower_file(&root().join(V2_VALID).join("01-gx10-disk-watch.rhl"));
    for line in [
        "// job \"gx10 disk watch\"\n",
        "// use vocabulary fleet v2\n",
        "// runs on host gx10\n",
        "// every 1 hour\n",
        "// may read disk\n",
        "// may write tickets\n",
    ] {
        assert!(r.contains(line), "missing {line:?}:\n{r}");
    }
    assert!(r.starts_with("// Lowered from RHL"), "{r}");
    assert!(
        !r.contains("ticket_count"),
        "expect/example are RHL-5's:\n{r}"
    );
    assert!(r.contains("fun decide(facts: Facts) -> Vec<Action> {\n    let mut plan: Vec<Action> = Vec::new()\n"), "{r}");
    assert!(r.trim_end().ends_with("    plan\n}"), "{r}");
    assert!(
        !r.contains("unsafe") && !r.contains("HashMap") && !r.contains("=>"),
        "{r}"
    );
}

#[test]
fn test_rhl4_text_literals_are_escaped_for_ruchy() {
    let r = lowered("  let s be \"a\\b{c}\"\n");
    assert!(
        r.contains("let v_s: String = \"a\\\\b{c}\".to_string()\n"),
        "{r}"
    );
}

// ---------------------------------------------------------------- refusals

#[test]
fn test_rhl4_refuses_give_back_stop_with_and_wait_up_to() {
    assert!(refused("  give back 1\n").contains("give back"));
    assert!(refused("  stop with \"done\"\n").contains("stop with"));
    assert!(refused("  wait up to 1 hour\n").contains("wait up to"));
}

#[test]
fn test_rhl4_refuses_nouns_plan_measures_and_non_literal_arguments() {
    assert!(refused("  let t be ticket title\n").contains("ticket title"));
    assert!(refused("  let c be ticket count\n").contains("ticket count"));
    let m = refused("  let p be \"paiml/infra\"\n  let n be tickets filed in p\n");
    assert!(m.contains("tickets filed in"), "{m}");
}

#[test]
fn test_rhl4_refuses_a_let_used_outside_its_block() {
    let m = refused(
        "  let free be disk free of \"/\"\n  when free is below 5 GB\n    let x be 1 GB\n  end\n  let y be x\n",
    );
    assert!(m.contains('x'), "{m}");
}

#[test]
fn test_rhl4_refuses_other_unit_kinds_and_several_units() {
    let src = "use vocabulary fleet v2\n\ncommand \"c\"\n  let x be 1\nend\n";
    match lower_source("c.rhl", src, Some(&root())) {
        Err(LowerFailure::Refused(d)) => {
            assert_eq!(d.code, codes::L001);
            assert!(d.message.contains("command"), "{}", d.message);
        }
        other => panic!("RHL-4: expected RHL-L001, got {other:?}"),
    }
    let two = format!("{}\njob \"u\"\n  let y be 1\nend\n", job("  let x be 1\n"));
    assert!(matches!(
        lower_source("t.rhl", &two, Some(&root())),
        Err(LowerFailure::Refused(_))
    ));
}

#[test]
fn test_rhl4_refuses_for_each_and_is_one_of_even_unchecked() {
    let for_each =
        job("  let free be disk free of \"/\"\n  for each h in free\n    let x be 1\n  end\n");
    let d = lower_unchecked(&for_each).expect_err("for each is not lowerable");
    assert_eq!(d.code, codes::L001);
    let one_of =
        job("  let s be pin status of \"gx13\"\n  when s is one of \"a\"\n    let x be 1\n  end\n");
    let d = lower_unchecked(&one_of).expect_err("is one of is not lowerable");
    assert!(d.message.contains("is one of"), "{}", d.message);
}

#[test]
fn test_rhl4_a_program_that_does_not_check_clean_is_not_lowered() {
    match try_lower("  let free be disk fre of \"/\"\n") {
        Err(LowerFailure::Check(report)) => {
            assert!(report.diagnostics.iter().any(|d| d.code == codes::V002));
            assert_eq!(report.exit_code(), 2);
        }
        other => panic!("RHL-4: expected the check's report, got {other:?}"),
    }
}

// ---------------------------------------------------------------- corpus

#[test]
fn test_rhl4_every_v2_valid_program_lowers_parses_and_transpiles() {
    let files = v2_valid();
    assert_eq!(files.len(), 12, "the v2 valid corpus has 12 programs");
    for path in &files {
        let r = lower_file(path);
        let ast = crate::Parser::new(&r).parse().unwrap_or_else(|e| {
            panic!(
                "{}: ruchy::Parser rejects the lowering: {e}\n{r}",
                path.display()
            )
        });
        crate::Transpiler::new()
            .transpile_to_program(&ast)
            .unwrap_or_else(|e| {
                panic!("{}: ruchy::Transpiler rejects it: {e}\n{r}", path.display())
            });
        let rust = to_rust(&r).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert!(
            rust.contains("fn decide(facts: Facts) -> Vec<Action>"),
            "{rust}"
        );
    }
}

#[test]
fn test_rhl4_every_v2_valid_program_compiles_as_a_lib_with_rustc() {
    if !rustc_available() {
        return;
    }
    let files = v2_valid();
    assert_eq!(files.len(), 12);
    for path in &files {
        let rust = to_rust(&lower_file(path)).expect("transpiles");
        let dir = tempfile::tempdir().expect("tempdir");
        rustc(
            dir.path(),
            &rust,
            &["--crate-type", "lib", "-o", "libj.rlib"],
        );
    }
}

#[test]
fn test_rhl4_every_construct_compiles_with_rustc() {
    if !rustc_available() {
        return;
    }
    let r = lowered(
        "  let free be disk free of \"/\"\n  let s be pin status of \"gx13\"\n  let t be \"pin\"\n  \
         let low be free is below 5 GB\n  let limit be 5 GB\n  let n be tickets filed in repo \"a/b\"\n  \
         when not low and s contains t or n is at least 2\n    set limit to 1 GB\n    \
         file ticket in repo \"a/b\" with\n      title \"x\"\n    end\n  otherwise\n    \
         set s to \"q\"\n  end\n  repeat at most 3 times until not free is at most 1 GB\n    \
         set t to s\n  end\n  when s is not t\n    expect ticket count is at most 1\n  end\n",
    );
    let rust = to_rust(&r).expect("transpiles");
    let dir = tempfile::tempdir().expect("tempdir");
    rustc(
        dir.path(),
        &rust,
        &["--crate-type", "lib", "-o", "libj.rlib"],
    );
}

#[test]
fn test_rhl4_yaml_surface_lowers_to_the_same_ruchy() {
    for path in v2_valid() {
        let src = read(&path);
        let program = crate::rhl::parse(&src).expect("parses");
        let yaml = crate::rhl::yaml::to_yaml(&program);
        let from_yaml = lower_yaml("j.rhl.yaml", &yaml, Some(&root())).expect("YAML lowers");
        assert_eq!(from_yaml, lower_file(&path), "{}", path.display());
    }
}

// ---------------------------------------------------------------- determinism (F5)

#[test]
fn test_rhl4_determinism_two_lowerings_are_byte_identical() {
    for path in v2_valid() {
        let (a, b) = (lower_file(&path), lower_file(&path));
        assert_eq!(
            a,
            b,
            "{}: ruchy differs between two lowerings",
            path.display()
        );
        let (ra, rb) = (to_rust(&a).expect("rust"), to_rust(&b).expect("rust"));
        assert_eq!(sha(&ra), sha(&rb), "{}: Rust differs", path.display());
    }
}

#[test]
fn test_rhl4_determinism_positive_control_one_literal_changes_the_sha() {
    let path = root().join(V2_VALID).join("01-gx10-disk-watch.rhl");
    let src = read(&path);
    let changed = src.replace("below 100 GB", "below 101 GB");
    assert_ne!(src, changed, "the control edits the program");
    let lower_rust = |s: &str| {
        let r = lower_source("j.rhl", s, Some(&root())).expect("lowers");
        to_rust(&r).expect("transpiles")
    };
    assert_ne!(sha(&lower_rust(&src)), sha(&lower_rust(&changed)));
}

// ---------------------------------------------------------------- meaning (§3.2)

#[test]
fn test_rhl4_gx10_examples_mean_one_ticket_then_none() {
    if !rustc_available() {
        return;
    }
    let r = lower_file(&root().join(V2_VALID).join("01-gx10-disk-watch.rhl"));
    let main = "\nfun main() {\n    let low = decide(Facts { disk_free_of_0: 90000000000 })\n    \
                let ok = decide(Facts { disk_free_of_0: 400000000000 })\n    \
                println!(\"{} {}\", low.len(), ok.len())\n}\n";
    let rust = to_rust(&format!("{r}{main}")).expect("transpiles");
    let dir = tempfile::tempdir().expect("tempdir");
    rustc(dir.path(), &rust, &["-o", "j"]);
    let out = Command::new(dir.path().join("j")).output().expect("runs");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "1 0");
}

// ---------------------------------------------------------------- RHL-4b: one lookup

/// A scratch root holding copies of `vocab/` and `contracts/` (files only),
/// with `disk free of` renamed `disk space of` in fleet v2.
fn renamed_term_root() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    for sub in ["vocab", "contracts"] {
        std::fs::create_dir_all(dir.path().join(sub)).expect("mkdir");
        for e in std::fs::read_dir(root().join(sub))
            .expect("read dir")
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file())
        {
            std::fs::copy(e.path(), dir.path().join(sub).join(e.file_name())).expect("copy");
        }
    }
    let fleet = dir.path().join("vocab/fleet-v2.yaml");
    let text = read(&fleet).replace("term: disk free of", "term: disk space of");
    std::fs::write(&fleet, text).expect("write renamed vocabulary");
    dir
}

#[test]
fn test_rhl4b_lower_uses_the_checkers_lookup_so_a_vocabulary_change_is_seen_by_both() {
    let scratch = renamed_term_root();
    let src = job(
        "  let free be disk space of \"/\"\n  when free is below 1 GB\n    \
                   file ticket in repo \"paiml/infra\"\n  end\n",
    );
    // The repository's vocabulary has no `disk space of`: check refuses, so
    // nothing is lowered.
    assert!(matches!(
        lower_source("t.rhl", &src, Some(&root())),
        Err(LowerFailure::Check(_))
    ));
    // The scratch vocabulary has it: check passes and lower resolves it.
    let program = crate::rhl::parse(&src).expect("parses");
    let lex = crate::rhl::check::load_lexicon("t.rhl", &src, &program, Some(scratch.path()));
    assert!(lex.get("disk space of").is_some() && lex.get("disk free of").is_none());
    let direct = lower("t.rhl", &src, &program, &lex).expect("lowers with the checker's lexicon");
    let via_check = lower_source("t.rhl", &src, Some(scratch.path())).expect("checks and lowers");
    assert_eq!(direct, via_check);
    assert!(direct.contains("disk_space_of_0: i64"), "{direct}");
    // One loader: lowering never reads a vocabulary file itself.
    assert!(!include_str!("lower.rs").contains("vocab::load("));
}
