//! RHL-5a tests: `given` → `Facts`, `then` → an assertion over the plan,
//! `expect` → a guard on the decided plan, the refusals, determinism of the
//! generated test program, and the test program built and run with `rustc`.

use super::super::{lower_source_as, to_rust, Form, LowerFailure};
use super::{example_names, PLAN_MEASURES};
use crate::rhl::codes;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------- helpers

fn root() -> PathBuf {
    crate::rhl::repo_root()
}

const GX10: &str = "docs/rhl/breaks/v2/valid/01-gx10-disk-watch.rhl";

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn v2_valid() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(root().join("docs/rhl/breaks/v2/valid"))
        .expect("RHL-5: v2 valid corpus exists")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "rhl"))
        .collect();
    files.sort();
    files
}

/// A job over both v2 vocabularies that declares every effect; `body` lines
/// are indented by two spaces.
fn job(body: &str) -> String {
    format!(
        "use vocabulary fleet v2\nuse vocabulary tickets v2\n\njob \"t\"\n  runs on host gx10\n  \
         every 1 hour\n  may read disk\n  may read runner\n  may read host\n  \
         may read tickets\n  may write tickets\n\n{body}end\n"
    )
}

fn tests_of(source: &str) -> Result<String, LowerFailure> {
    lower_source_as(Form::Tests, "t.rhl", source, Some(&root()))
}

fn tests_body(body: &str) -> String {
    tests_of(&job(body)).unwrap_or_else(|f| panic!("RHL-5: expected a test program, got {f:?}"))
}

/// The code and message of the refusal `body` must produce.
fn refused(body: &str) -> (String, String) {
    match tests_of(&job(body)) {
        Err(LowerFailure::Refused(d)) => (d.code, d.message),
        other => panic!("RHL-5: expected a refusal, got {other:?}"),
    }
}

fn gx10_with(from: &str, to: &str) -> String {
    let src = read(&root().join(GX10));
    assert!(src.contains(from), "GX10 has `{from}`");
    src.replace(from, to)
}

fn sha(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

// ---------------------------------------------------------------- given

#[test]
fn test_rhl5_given_a_let_name_sets_the_field_of_its_measure() {
    let t = tests_of(&read(&root().join(GX10))).expect("GX10 lowers to a test program");
    assert!(
        t.contains("let plan: Vec<Action> = decide(Facts { disk_free_of_0: 90000000000 })"),
        "{t}"
    );
}

#[test]
fn test_rhl5_given_a_measure_with_literal_arguments_sets_that_field() {
    let t = tests_body(
        "  when disk free of \"/\" is below 100 GB\n  file ticket in repo \"paiml/infra\"\n  end\n\n  \
         example \"e\"\n    given disk free of \"/\" is 90 GB\n    then ticket count is 1\n  end\n",
    );
    assert!(
        t.contains("decide(Facts { disk_free_of_0: 90000000000 })"),
        "{t}"
    );
}

#[test]
fn test_rhl5_given_values_use_the_lowering_unit_scale_and_text() {
    let t = tests_body(
        "  let load be runner load of \"r\"\n  let status be pin status of \"h\"\n  \
         let age be disk usage of \"/\"\n\n  example \"e\"\n    given load is 95 %\n    \
         given status is \"unpinned\"\n    given age is 2 GB\n  end\n",
    );
    assert!(
        t.contains(
            "Facts { runner_load_of_0: 9500, pin_status_of_1: \"unpinned\".to_string(), \
             disk_usage_of_2: 2000000000 }"
        ),
        "{t}"
    );
}

#[test]
fn test_rhl5_examples_are_lowered_in_source_order_with_their_names() {
    let body = "  let free be disk free of \"/\"\n\n  example \"low\"\n    given free is 90 GB\n  end\n\n  \
                example \"high\"\n    given free is 400 GB\n  end\n";
    let t = tests_body(body);
    let (a, b) = (
        t.find("fun example_0() -> String {").expect("example_0"),
        t.find("fun example_1() -> String {").expect("example_1"),
    );
    assert!(a < b);
    assert!(t.contains("// example_0: \"low\"") && t.contains("// example_1: \"high\""));
    let program = crate::rhl::parse(&job(body)).expect("parses");
    assert_eq!(example_names(&program), ["low", "high"]);
}

// ---------------------------------------------------------------- then, expect

#[test]
fn test_rhl5_then_asserts_a_plan_measure_over_the_decided_plan() {
    let t = tests_of(&read(&root().join(GX10))).expect("lowers");
    for want in [
        "if plan_ticket_count(&plan) == 1 {",
        "failed = \"then ticket count is 1\".to_string()",
        "fun plan_ticket_count(plan: &Vec<Action>) -> i64 {",
        "Action::FileTicket { repo, title, label } => true,",
    ] {
        assert!(t.contains(want), "missing `{want}`:\n{t}");
    }
}

#[test]
fn test_rhl5_expect_is_a_named_guard_and_examples_check_it() {
    let t = tests_of(&read(&root().join(GX10))).expect("lowers");
    for want in [
        "// expect_0: expect ticket count is at most 1",
        "fun expect_0(plan: &Vec<Action>) -> bool {\n    plan_ticket_count(plan) <= 1\n}",
        "fun failed_expectation(plan: &Vec<Action>) -> String {",
        "failed = \"expect ticket count is at most 1\".to_string()",
        "failed = failed_expectation(&plan)",
    ] {
        assert!(t.contains(want), "missing `{want}`:\n{t}");
    }
}

#[test]
fn test_rhl5_program_main_checks_expectations_before_it_can_apply() {
    let src = read(&root().join(GX10));
    let p = lower_source_as(Form::Program, GX10, &src, Some(&root())).expect("program");
    let check = p
        .find("let failed: String = failed_expectation(&plan)")
        .expect("main checks the expectations");
    let exit = p[check..].find("std::process::exit(1)").expect("exits");
    let apply = p.find("performed = apply(plan)").expect("apply");
    assert!(check + exit < apply, "the guard comes before apply:\n{p}");
    assert!(
        p.contains("fun expect_0(plan: &Vec<Action>) -> bool {"),
        "{p}"
    );
    assert!(
        !p.contains("fun example_0("),
        "the program form has no examples"
    );
}

#[test]
fn test_rhl5_plan_measure_table_is_ticket_count_over_file_ticket() {
    assert_eq!(PLAN_MEASURES, [("ticket count", "file ticket")]);
    // With no `file ticket` in the job, the count is 0 and names no variant.
    let t = tests_body("  expect ticket count is at most 1\n");
    assert!(
        t.contains("fun plan_ticket_count(plan: &Vec<Action>) -> i64 {\n    0\n}"),
        "{t}"
    );
}

#[test]
fn test_rhl5_tests_form_has_no_observe_apply_or_runtime_binding() {
    // 03 reads `runner load of`, bound to `[U]`: it cannot be run, but its
    // examples can, because they never read the world.
    let rel = "docs/rhl/breaks/v2/valid/03-gx12-runner-load-watch.rhl";
    let t = tests_of(&read(&root().join(rel))).expect("03 lowers to a test program");
    for absent in [
        "fun observe(",
        "fun apply(",
        "fun file_ticket(",
        "Command::new",
    ] {
        assert!(
            !t.contains(absent),
            "`{absent}` in a hermetic test program:\n{t}"
        );
    }
    assert!(t.contains("runner_load_of_0: 9500"), "{t}");
}

// ---------------------------------------------------------------- refusals

#[test]
fn test_rhl5_refuses_a_given_over_anything_but_a_fact_or_a_direct_let() {
    let base = "  let free be disk free of \"/\"\n  let limit be 5 GB\n  \
                let copy be free\n";
    for (given, name) in [
        ("given limit is 1 GB", "`limit`"),
        ("given copy is 5 GB", "`copy`"),
        ("given ticket count is 1", "`ticket count`"),
        (
            "given disk free of \"/data\" is 5 GB",
            "`disk free of \"/data\"`",
        ),
    ] {
        let body = format!("{base}\n  example \"e\"\n    {given}\n  end\n");
        let (code, message) = refused(&body);
        assert_eq!(code, codes::L001, "{given}: {message}");
        assert!(message.contains(name), "{given}: {message}");
    }
}

#[test]
fn test_rhl5_refuses_a_given_set_twice_or_a_let_changed_by_set() {
    let twice = "  let free be disk free of \"/\"\n\n  example \"e\"\n    given free is 1 GB\n    \
                 given disk free of \"/\" is 2 GB\n  end\n";
    let (code, message) = refused(twice);
    assert_eq!(code, codes::L001);
    assert!(message.contains("twice"), "{message}");
    let set = "  let free be disk free of \"/\"\n  set free to 5 GB\n\n  example \"e\"\n    \
               given free is 1 GB\n  end\n";
    let (code, message) = refused(set);
    assert_eq!(code, codes::L001);
    assert!(
        message.contains("`free`") && message.contains("set"),
        "{message}"
    );
}

#[test]
fn test_rhl5_an_example_that_leaves_a_fact_unset_is_x002_naming_it() {
    let body = "  let free be disk free of \"/\"\n  let used be disk usage of \"/home\"\n\n  \
                example \"half\"\n    given free is 90 GB\n  end\n";
    let (code, message) = refused(body);
    assert_eq!(code, codes::X002);
    assert!(message.contains("`disk usage of \"/home\"`"), "{message}");
    assert!(message.contains("\"half\""), "{message}");
    assert!(codes::lookup(codes::X002).is_some());
}

#[test]
fn test_rhl5_refuses_then_and_expect_over_anything_but_a_plan_measure() {
    let base = "  let free be disk free of \"/\"\n";
    for (line, name) in [
        ("  expect free is above 1 GB\n", "`free`"),
        (
            "  expect disk free of \"/\" is above 1 GB\n",
            "`disk free of`",
        ),
        (
            "  example \"e\"\n    given free is 1 GB\n    then free is 1 GB\n  end\n",
            "`free`",
        ),
    ] {
        let (code, message) = refused(&format!("{base}{line}"));
        assert_eq!(code, codes::L001, "{line}: {message}");
        assert!(message.contains(name), "{line}: {message}");
        assert!(message.contains("plan measure"), "{line}: {message}");
    }
}

// ---------------------------------------------------------------- determinism

#[test]
fn test_rhl5_determinism_test_programs_are_byte_identical_and_transpile() {
    for path in v2_valid() {
        let file = path.display().to_string();
        let src = read(&path);
        let a = lower_source_as(Form::Tests, &file, &src, Some(&root()))
            .unwrap_or_else(|f| panic!("{file}: {f:?}"));
        let b = lower_source_as(Form::Tests, &file, &src, Some(&root())).expect("twice");
        assert_eq!(sha(&a), sha(&b), "{file}");
        let (ra, rb) = (to_rust(&a), to_rust(&b));
        let ra = ra.unwrap_or_else(|e| panic!("{file}: {e}"));
        assert_eq!(Ok(ra), rb, "{file}");
    }
    // Positive control: one `then` literal changes the program.
    let a = tests_of(&read(&root().join(GX10))).expect("lowers");
    let b = tests_of(&gx10_with(
        "then ticket count is 1",
        "then ticket count is 0",
    ))
    .expect("0");
    assert_ne!(sha(&a), sha(&b));
}

// ---------------------------------------------------------------- built and run

fn rustc_available() -> bool {
    let ok = Command::new("rustc")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    if !ok {
        println!("RHL-5: skipped: no `rustc` on PATH to build the test program");
    }
    ok
}

/// Build the test program of `source` with `rustc` and run it: its stdout
/// and exit code.
fn run_tests(source: &str) -> (String, i32) {
    let ruchy = tests_of(source).unwrap_or_else(|f| panic!("RHL-5: {f:?}"));
    let rust = to_rust(&ruchy).expect("transpiles");
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("j.rs"), &rust).expect("write");
    let built = Command::new("rustc")
        .args(["--edition", "2021", "-o", "j", "j.rs"])
        .current_dir(dir.path())
        .output()
        .expect("rustc runs");
    assert!(
        built.status.success(),
        "{}\n{rust}",
        String::from_utf8_lossy(&built.stderr)
    );
    let out = Command::new(dir.path().join("j"))
        .env_clear()
        .output()
        .expect("runs");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    (stdout, out.status.code().unwrap_or(-1))
}

#[test]
fn test_rhl5_run_gx10_example_holds_and_a_wrong_then_fails_naming_it() {
    if !rustc_available() {
        return;
    }
    assert_eq!(
        run_tests(&read(&root().join(GX10))),
        ("rhl-example 0 ok\n".to_string(), 0)
    );
    let zero = gx10_with("then ticket count is 1", "then ticket count is 0");
    assert_eq!(
        run_tests(&zero),
        (
            "rhl-example 0 FAILED then ticket count is 0\n".to_string(),
            1
        )
    );
    // 400 GB free: no ticket, so `then ticket count is 1` fails.
    let healthy = gx10_with("given free is 90 GB", "given free is 400 GB");
    assert_eq!(run_tests(&healthy).1, 1);
}

#[test]
fn test_rhl5_run_an_example_whose_plan_breaks_an_expect_fails_naming_the_expect() {
    if !rustc_available() {
        return;
    }
    let two = gx10_with(
        "      label \"fleet\"\n    end\n",
        "      label \"fleet\"\n    end\n    file ticket in repo \"paiml/infra\" with\n      \
         title \"again\"\n    end\n",
    );
    let two = two.replace("then ticket count is 1", "then ticket count is 2");
    assert_eq!(
        run_tests(&two),
        (
            "rhl-example 0 FAILED expect ticket count is at most 1\n".to_string(),
            1
        )
    );
}

#[test]
fn test_rhl5_refuses_an_expect_inside_a_block() {
    let body = "  let free be disk free of \"/\"\n  when free is below 1 GB\n    \
                expect ticket count is at most 1\n  end\n";
    let (code, message) = refused(body);
    assert_eq!(code, codes::L001);
    assert!(message.contains("inside a block"), "{message}");
}
