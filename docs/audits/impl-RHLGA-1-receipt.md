# RHLGA-1 G2c — implementation receipt (PR #248)

## Scope

RHLGA-1 is an epic delivered in gates (plan: `docs/rhl/rhlga-1-plan.md`).
This PR is gate **G2c** only: the workspace test suite passes on any host.
It does NOT bump the version or release 5.0.0; that is a later gate of the
same epic. The PR is stacked on #246 (G2b); the diff under review is the
four commits after `1e82f210` (the #246 head).

## Rows this diff implements (docs/roadmaps/roadmap.yaml)

- **HOSTDEP-1** — acceptance, verbatim:
  - "matrix_001..004_native, lang_comp_suite and issue_040 spawn env!(\"CARGO_BIN_EXE_ruchy\"), not Command::new(\"ruchy\")"
  - "Named-environment ignores (G2 acceptance): tooling_002 test 01 (renacer on PATH), opt_global_001 test 03 (llvm-profdata plus the profiles ignored test 02 collects)"
  - "cli_unify_003 determinism block runs 1K cases like the other spawn-heavy blocks"
  - "opt_global_001 test 03 merges the collected /tmp/pgo-test-*.profraw files and asserts success"
  The two `#[ignore = "named environment: …"]` attributes are therefore
  required by the ticket, not a weakened gate: each names the host tool it
  needs, and the tests still run in nightly/tier3.
- **RAWIDENT-2**, **GLOBALSHADOW-1**, **RETUNIT-1**, **BLOCKFOR-1** —
  transpiler/parser fixes found by the G2 suite; tests in
  `tests/g2b_transfix_7.rs` (21, RED first, compiled vs interpreted output).

## Evidence

- At `ba96b2dd` on intel: `cargo nextest run --workspace --no-fail-fast`,
  28138 passed, 2492 skipped, exit 0. `feb1d2c5` changes only test 03 of
  `opt_global_001`, which is ignored by default.
- CI at `feb1d2c5`: `ci / gate`, lint, test, security, provenance pass.
- Prior round at `ba96b2dd`: sonnet-5 PASS, haiku-4-5 PASS, gemini-3.1-pro
  FAIL (PGO test 03 passed a literal glob and asserted only spawn) — fixed
  in `feb1d2c5`.
- Round 2 at `feb1d2c5`, base `main` (so #246's G2b diff was judged too):
  gemini-3.1-pro FAIL — release not done (out of scope, see *Scope*) and
  the two HOSTDEP-1 named-environment ignores (required, see above).
  Both flash lanes PASS.
- Round 3 at `feb1d2c5`, base #246: gemini-3.1-pro FAIL —
  `lifetime_helpers.rs:162` still used the name-only heuristic, so the
  lifetime path (a `#[test]` fn with 2+ reference params and a reference
  return) missed RETUNIT-1. Reproduced RED by
  `test_retunit_1_lifetime_path_numeric_name_ending_in_assign_is_unit`
  (`expected unit, got -> i32`); fixed in the next commit. Lib 20892 pass,
  `g2b_transfix_7` 21 pass, fmt clean, clippy `-D warnings` clean.
