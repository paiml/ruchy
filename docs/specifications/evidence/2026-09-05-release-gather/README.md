# Release gather — 2026-09-05 (P0, genchi genbutsu at HEAD)

Ticket: PMAT-091. HEAD measured: `3a665f36edc170694159746aa1f9bffec4f4a007` (origin/main, 2026-08-11).
Local checkout was 4 commits behind (`e6503890`) and was fast-forwarded after stashing a
pre-existing dirty tree (see `dirty-tree-at-start.diff`, stash `stash@{0}`).

Every file here is raw command output. Nothing is summarised from memory.

## Seed facts — re-verified

| Seed fact (prompt) | Status | Evidence file |
|---|---|---|
| version `5.0.0-beta.1` | VERIFIED (`Cargo.toml:170`) | `cargo-toml-deps.txt`, `discover.json` |
| crates.io `4.2.1` (2026-02-10), ~7-month lag | VERIFIED | `crates-io-latest.txt`, `crates-io-version-history.txt` |
| `rust-version = "1.75"` | VERIFIED as declared; **FALSIFIED as a claim** — resolved graph max is 1.89 (`trueno`, `aprender`, `trueno-gemm-codegen`) | `msrv-and-claims.txt`, `msrv-claim-test.txt` |
| pins trueno 0.16, aprender 0.27, entrenar 0.7, alimentar 0.2.5, presentar 0.3.1, simular 0.3, trueno-viz 0.1.23 | VERIFIED; resolve to 0.16.5 / 0.27.8 / 0.7.13 / 0.2.8 / 0.3.4 / 0.3.1 / 0.1.27 | `cargo-toml-deps.txt`, `cargo-tree-siblings.txt` |
| crates.io now aprender 0.65.2 (2026-09-04); trueno 0.19.1 / entrenar 0.8.1 frozen | VERIFIED latest = 0.65.2 / 0.19.1 / 0.8.1; trueno, entrenar, simular, trueno-viz are DEPRECATED facades | `crates-io-latest.txt`, `aprender-monorepo-facades.txt` |
| post APR-MONO the sovereign crates live in the aprender workspace | VERIFIED: `aprender-compute`, `-train`, `-viz`, `-simulate`, `-contracts` all at 0.65.2 on crates.io | `aprender-monorepo-facades.txt` |
| quarantined tests 277 + 24 + 501 = 802 vs 456 in `tests/` | VERIFIED as tracked-file counts (sprint7 dir: 501 tracked, 458 `.rs`); 265 basenames duplicated between the two big dirs; none referenced by Cargo/Makefile/CI | `quarantine-counts.txt`, `quarantine-detail.txt` |
| release.yml publishes `ruchy-cli` under `continue-on-error`, no such member | VERIFIED. **Additionally**: it reads `secrets.CRATES_TOKEN`, which does not exist (repo secrets: `CARGO_REGISTRY_TOKEN`, `CARGO_TOKEN`) | `workflows-grep.txt`, `workflows-snapshot.txt`, `auth-and-pv-surface.txt` |
| `contracts/`: 2 YAML | VERIFIED (`checkpoint-v1`, `dispatch-v1`); 3 more live out-of-repo in `../provable-contracts/contracts/ruchy/` | `contracts-inventory.txt` |
| 472 kLOC `src/` | VERIFIED (472,301 lines) | `contracts-inventory.txt` (bottom) |
| HEAD #200 (2026-08-11) "clean checkout could not build" | VERIFIED; local was STALE (4 behind), now fast-forwarded | `head.txt`, `git-log-since-2026-02-10.txt` |
| `.pmat-work/ledger.jsonl`: 1 receipt, 2026-04-04 | VERIFIED | `pmat-work-ledger.jsonl` |
| ci.yml → `sovereign-ci.yml@main`, `gate` job; `Makefile:918`; both scripts exist | VERIFIED; required check `gate` comes from an org ruleset, not legacy branch protection | `workflows-snapshot.txt`, `gh-ci-status.txt`, `tooling-capacity.txt` |

## New facts measured here (not in the seed)

| Fact | Evidence file |
|---|---|
| **Clean-room is GREEN at HEAD**: `cargo package -p ruchy` verify exit 0; extracted crate builds `--release --locked` in an empty `CARGO_HOME`, binary prints `ruchy 5.0.0-beta.1` | `cargo-package-verify.log`, `cleanroom-empty-cargo-home-build.log` |
| Package contents: 2882 files, 38.9 MiB raw / 7.8 MiB compressed (limit 10 MiB); includes 802 quarantine files, 574 `docs/` files, 10 `.pmat-work/` files, **7 paths containing `:`** | `cargo-package-precheck.txt`, `package-contents-precheck.txt` |
| Windows nightly fails daily on a tracked path with `:` (`.pmat-work/Apex Hunt: PDCA Cycle 20 …`) | `gh-ci-runs-detail.txt` |
| `provable-contracts` is a **dev-dependency** by path with no version: stripped from the package (not a publish blocker) but a clone without `../provable-contracts` cannot load the workspace manifest | `path-dep-and-build-rs.txt`, `apr-mono-spike-default-features.txt` |
| 7/7 reserved keywords (`requires ensures invariant decreases infra signal yield`) parse as identifiers in 4.2.1 and are rejected at HEAD | `keyword-break-and-verbs.txt` |
| Corpus differential (1286 files, 7 corpora): 1104 pass on both, **2 regress** (both keyword collisions, one in-repo `examples/24_math_science.ruchy`), 0 fixed, 180 fail on both | `differential-check-4.2.1-vs-head.csv`, `differential-check-summary.txt`, `differential-regressions-detail.txt` |
| CLI verb surface: 4.2.1 has 37 verbs, HEAD 47 (+10, −0) | `keyword-break-and-verbs.txt`, `baseline-4.2.1-verbs.txt` |
| Transpile output on `examples/` (both parse): 138 identical, 13 differ, 3 fail on both | `transpile-diff-examples.txt` |
| `make pre-release-gate` at HEAD: **43/100 BLOCKED, exit 2**; coverage stage silently reads 0%, 15 SATD markers, doc coverage 45% | `pre-release-gate-run.log`, `pre-release-gate-result.txt`, `pre-release-gate.sh.snapshot` |
| `pmat comply check`: exit 1, 10 failures (CB-040, CB-081, CB-200, CB-400, CB-1308, CB-1700, CB-1701, CB-2100, version currency, evidence group) | `pmat-comply.txt` |
| CI on main green at HEAD (2026-08-11); nightly bench fails on `cargo fmt --check` drift in `ruchy-embed/src/lib.rs`, `command_router.rs`, `migrate.rs` | `gh-ci-runs-detail.txt`, `dispatch-and-nightly-detail.txt` |
| Two roadmaps: `docs/roadmaps/roadmap.yaml` (126 tickets, written by `pmat work`, read by discover/target-guard, touched 2026-04-04) vs `docs/execution/roadmap.yaml` (11,241 lines, last touched 2025-12-09, referenced only by CLAUDE.md) | `roadmaps-compare.txt`, `roadmap-and-hooks.txt` |
| Dispatch hot paths: `transpile_expr` (`src/backend/transpiler/expr_dispatcher.rs:86-175`, TDG B+), `eval_expr_kind` (`src/runtime/interpreter.rs:219-254`, B+; `src/runtime/eval_expr.rs:16-46`, A), CLI `handle_complex_command` (`src/bin/handlers/command_router.rs:60-74`, A+) | `pmat-query-dispatch.txt`, `dispatch-and-nightly-detail.txt` |
| Sibling-crate usage is thin: trueno in 1 bridge file, aprender in 4, the rest 1–2 each | `sibling-crate-usage.txt`, `sibling-bridge-imports.txt` |
| aprender-0.65 spike (default build, `trueno`→`aprender-compute`, `aprender`→0.65): 2 unresolved-import errors | `apr-mono-spike-default-features.txt` |
| `cargo semver-checks` vs 4.2.1 with all features: baseline 4.2.1 **cannot be built today** (`entrenar` 0.7 fails to compile) | `semver-checks-vs-4.2.1.log` |
| 49 open issues (30 are automated "Web Quality Alert"), 0 open PRs; #195 and #196 are addressed by this plan / already on main | `gh-issues-open.txt`, `gh-prs-open.txt` |
| Tooling: pmat 3.36.0 (no `hooks install --strict`, no commit-msg hook), agy 1.1.27, pv 0.63.0, cargo 1.98, docker present, `ruchydbg` absent | `tooling-capacity.txt`, `auth-and-pv-surface.txt` |

## Files

- `discover.json` — paiml-implement Phase 0 output (sha256 in the receipt).
- `dirty-tree-at-start.diff` — what was stashed before fast-forward.
- `baseline-4.2.1-install.log`, `baseline-4.2.1-verbs.txt` — the 4.2.1 binary built from crates.io (`--locked`).
- `head-release-build.log` — HEAD release binary used for the differential.
- `head-all-features-check.log`, `semver-checks-vs-4.2.1-default-features.log` — follow-up measurements launched after the first pass.

## Added after P2 (quorum) and the first jidoka

| Fact | Evidence file |
|---|---|
| P2 quorum: 3 agy lanes (adversarial / clean-room / semantic), 0 PASS; raw lane outputs archived | `../2026-09-05-quorum-p2/lane-{1,2,3}.json` |
| Lane-2 claim re-measured: `aprender`, `aprender-core`, `aprender-compute` 0.65.2 declare `rust-version = 1.91` (not 1.89); `Cargo.lock` is already in the package list (auto-included for binary packages) | `quorum-p2-remeasure.txt` |
| Jidoka PMAT-102: `cargo clippy --all-targets -- -D warnings` fails at HEAD under rustc 1.98 (7 × `unused_must_use` on `assert_cmd::Command::assert` in `tests/cli_contract_fuzz.rs`); the pre-commit hook and `ci / lint` run the same lint, so every commit and every PR is blocked until fixed | this README; the fix is PR PMAT-102 |
| Baseline of plain `main` under CI's exact toolchain (rustc 1.97.1, installed locally): `cargo fmt --all -- --check` DIRTY (same 28 files) and `cargo clippy --all-targets -- -D warnings` red with the same 24 `unused_must_use` + 4 bench errors — so CI's `gate` was green only because of the lib-only fallback (paiml/.github#64), not because of a toolchain difference | `main-under-ci-toolchain-1.97.1.log`, `sovereign-ci-lint-job.txt` |
| The pre-commit hook checks `cargo fmt --all -- --check` on the WHOLE tree and complexity per staged file (30/25 per function); with 28 drifted files at `main`, no commit on any branch can pass until the tree is formatted, and four of those files hold six over-limit functions — hence Z0b (PMAT-103) in the same commit | `z0-pmat-102-verification.txt`; plan §6 B17–B20 |
| P4 pre-PR quorum on #204 (Z0): FAIL/PASS/FAIL; all lanes confirm control-flow preservation of the six decompositions and the prover EOF fix; folded: commit count in PR text, `build.rs` warns when the binding file is absent (measured in a sibling-less worktree), hermetic replacements for the two removed dispatch-routing tests, un-glued `#[rustfmt::skip]` lines | `../2026-09-05-quorum-p4-z0/lane-{1,2,3}.json`, `z0-pmat-102-verification.txt` |
| `ci / lint` on #204 failed on a NEW advisory: RUSTSEC-2026-0258 (`h2` 0.4.15, unbounded empty DATA frames); `cargo update -p h2` → 0.4.19, `cargo deny check advisories` ok (PMAT-105) | this README; PR #204 |
| `cargo test --workspace` on the Z0 branch (gate_cmd) stops at `tests/bug_032_range_function_not_transpiled.rs::test_bug_032_red_range_in_expression`, which fails identically on `main` — a pre-existing RED-phase integration test; full `--no-fail-fast` inventory under PMAT-106 | `z0-workspace-gate.log`, `z0-workspace-nofailfast.log` |
| `build.rs` AllImplemented binding gate: resolved the sibling manifest two directories up and never found it (0 `CONTRACT_*` env vars, silent skip); after the path fix `cargo check -p ruchy --lib` prints `AllImplemented: 20/20 implemented, 0 gaps` and exports 20 env vars (PMAT-107, Z0 PR) | this README; PR #204 commit "fix(build): the AllImplemented binding check…" |
| `cargo test --workspace --no-fail-fast` on the Z0 branch: per-target inventory of pre-existing failing integration tests (RED-phase, parser-defect, environment-bound); the same test files run on plain `main` for attribution (PMAT-106) | `z0-workspace-nofailfast.log`, `main-integration-failures-triage.log` |
