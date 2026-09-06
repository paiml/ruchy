# impl receipt — PMAT-091 · ruchy next release (5.0.0-beta.2)

Mode `paiml-implement`, unattended. Process authority: `~/.claude/skills/paiml-implement/SKILL.md`.
Deterministic body; no timestamps. Dates appear only where they are part of an artifact name.

## Identity

| Field | Value |
|---|---|
| Ticket | PMAT-091 (RELEASE-GATHER-PLAN); terminal ticket PMAT-100 (RELEASE-PUBLISH-5.0.0-beta.2) |
| Branch | `PMAT-091-plan-v3` (plan, evidence, receipts); code landed through 13 PRs listed below |
| HEAD | `59c7453bbb2d48c54be92faa7a2b715849b5a651 (release commit, tag v5.0.0-beta.2; plan branch PMAT-091-plan-v3 carries this receipt)` |
| `discover.json` sha256 | `f84c2bd1bee8d80a0b61f939e86e4dfad9fd787a9423a5f0323831c6a7679db9` (`gate_cmd_fallback=true`: `cargo test --workspace`) |
| Plan | `docs/specifications/ruchy-5.0.0-beta.2-release-plan.md` (v1 → v2 → v3 as-built, §7e) |
| Evidence | `docs/specifications/evidence/2026-09-05-release-gather/`, `…-quorum-p2/`, `…-quorum-p4-*/`, `…-dogfood/` |

## Plan, routing and trigger per phase

| Phase | Content | Routing | Trigger |
|---|---|---|---|
| P0 gather | re-verify the seed facts; measure API delta 4.2.1 → HEAD; clean-room; CI fallbacks; bin test target | direct | — |
| P1 plan | version decision (5.0.0-beta.2: 7 keywords break 4.2.1 programs, GA not earned), EV-ranked Z0–Z9, mandatory candidates accept/reject | direct | — |
| P2 quorum | 3 lanes on the plan: adversarial, clean-room, semantic preservation | `paiml-agy-delegate` → agy (teamwork/grillme) | Q2 (spec/plan artifact) |
| P3 build | Z0 PMAT-102/103/104 · Z1 PMAT-092 · Z4 PMAT-095 · Z2 PMAT-093 · Z7 PMAT-098 · Z3 PMAT-094 · R1 PMAT-099 · PMAT-119 · Z6 PMAT-097 · Z8 PMAT-101 · Z5a PMAT-112/113 · Z5 PMAT-096 | workers (`paiml-impl-worker`, disjoint `scope_paths`) for the code phases; direct for hook debt, fold-ins, rebases, merges; every PR reviewed by a 3-lane agy quorum before merge | Q1 where `|M| ≥ 3` (Z0, Z3+Z8), Phase-4 pre-PR review on every PR |
| P4 dogfood + publish | `make pre-release-gate` on `main`; Z9 bump + CHANGELOG; tag; `release.yml`; fresh-container install | direct | — |

## Dispatch ledger (measured by `transcript-gate.sh`; per-agent turn counts were not preserved across context compaction)

| Measure | Value |
|---|---|
| Claude subagent starts | 16 (`agent_calls=16`), resumes 11, workflow starts 0 |
| Peak concurrent subagents | 3 of 3 slots; denied by the hook: 0 |
| Delegate dispatches (agy) | P2 plan quorum (3 lanes); P4 pre-PR quorums: Z0 (#204), Z1+Z4 (#202/#203), Z3+Z8 (#207/#208), batch Z2/Z7/R1/Z6/PMAT-119 (#205/#206/#209/#211/#210), PMAT-112/113 (#212). Lane outputs under `docs/specifications/evidence/2026-09-05-quorum-*/`; agy conversation ids in the lane JSON |
| Worker incidents | one worker killed by an upstream 429 left a stale subagent lock (released by hand, feedback filed); one worker minted colliding ticket ids from a worktree (B31); agy lanes once ran `git checkout` in the main checkout and destroyed worktree metadata (B24, restored) |

## Verification table (worker or lane claim vs orchestrator re-run)

| PR | Ticket | Claim | Orchestrator re-run | Agreement |
|---|---|---|---|---|
| #204 | Z0 PMAT-102/103/104 | clippy `--all-targets` clean; bin tests complete | clippy clean; 956 bin unit tests pass; `build.rs` binding gate runs (20/20) | yes |
| #202 | Z1 PMAT-092 | `cargo package` allowlist only | `cargo package --list` allowlist only; 7 hygiene tests | yes |
| #203 | Z4 PMAT-095 | one identifier renamed, stdout byte-identical vs 4.2.1 | re-run of migrated example under HEAD vs original under 4.2.1: identical; 3 tests; mutation red | yes |
| #205 | Z2 PMAT-093 | publish job real | 6 workflow-lint tests; structural `continue-on-error` test red on mutation | yes |
| #206 | Z7 PMAT-098 | one roadmap | `roadmap_single_source` green; leftover references found by quorum and repointed | yes after fold-in |
| #207 | Z3 PMAT-094 | zero semantic changes | withdrawn: aprender-core 0.65 numerics differ (documented); `--locked` clean-room build green; MSRV 1.91 | corrected |
| #209 | R1 PMAT-099 | SATD 0 in src | `pmat analyze satd --path src` = 0 | yes |
| #210 | PMAT-119 | blocklists complete | measured against all 77 lexer words | yes |
| #211 | Z6 PMAT-097 | dispatch totality | 93 variants, wildcard-free; macros not wired (PMAT-116) | yes |
| #208 | Z8 PMAT-101 | dataframe ports; audit clean | first re-run found `SerReader`/`SerWriter` missing (an earlier empty `timeout command cargo` run had read as green, B36); fixed; `check --all-features --locked`, golden 4/4, `cargo audit` clean, `quick-xml` absent | corrected |
| #212 | Z5a PMAT-112/113 | lint sound | lanes' soundness findings confirmed by re-running; hardened lint found two more real cases (block comment, wasm names) | corrected |
| #213 | Z5 PMAT-096 | 8-stage gate | first run on `main`: 6 PASS, clean_room+package refused on a dirty tree (untracked local artifacts); rerun on a clean tree: see §Gate |
| #214 | Z9 PMAT-100 | version coherence | manifest test RED on the old manifests, GREEN after the bump; `cargo metadata --locked` clean | yes |
| #215 | Z5b PMAT-127 | clean-room packages from a HEAD worktree | `--only clean_room,package` PASS with an ignored proptest artifact present; full run 4: go 8/8 | yes |
| #216 | Z5c PMAT-129/130 | root crate builds for wasm32 | `RUSTFLAGS=… cargo build --lib --target wasm32-unknown-unknown` Finished; gate `--only features` PASS (`wasm32=0`); release run 33988801263 Build WASM ✓ | yes |
| — | publish | `cargo publish -p ruchy` | run 33988801263: 403 Forbidden, authentication failed (invalid `CARGO_REGISTRY_TOKEN`) — stop the line | n/a |

## Gate (dogfood receipt)

Receipt: `docs/specifications/evidence/2026-09-05-dogfood/receipt.json` (run 4; runs 1 and 3 kept as `receipt-run1-no-go-dirty-tree.json`, `receipt-run3-no-go-proptest-artifacts.json`; PMAT-127 acceptance as `receipt-pmat-127-only-clean-room-package.json`).

| Stage | Status | Detail |
|---|---|---|
| tests | PASS | {"exit":0} |
| features | PASS | {"default":0,"all":0,"minimal":0,"fmt":0,"audit":0} |
| verbs | PASS | {"total":47,"pass":27,"warn":20,"fail":0,"list":[{"verb":"repl","mode":"help_only","input":null,"exit":0,"stdout_sha256":"929905fec5353746929660df9e85 |
| differential | PASS | {"files":1287,"both_pass":1107,"check_regressions":["../ruchyruchy/validation/tests/test_property_framework.ruchy"],"run_regressions":["../ruchyruchy/ |
| transpile | PASS | {"identical":1089,"differs":["examples/01_basics.ruchy","examples/02_functions.ruchy","examples/04_collections.ruchy","examples/06_error_handling.ruch |
| clean_room | PASS | {"locked_exit":0,"unlocked_exit":0,"binary_version":"5.0.0-beta.1","unlocked_binary_version":"5.0.0-beta.1","expected_version":"5.0.0-beta.1","locked_ |
| package | PASS | {"files":1446,"bytes_compressed":3319700,"colon_paths":0,"has_lock":true} |
| satd | PASS | {"count":0,"max":0} |

Verdict: `go`; warns: 27 (verbs help-only ×20 → PMAT-123; corpus missing → PMAT-124; baseline nondeterminism ×4 → PMAT-125; compile budget → PMAT-126).

## Jidoka log (`.pmat/jidoka.jsonl`, gitignored; copied here)

| Ticket | Phase | Defect | Owner | Five whys |
|---|---|---|---|---|
| PMAT-102 | Z0 | cargo clippy --all-targets -- -D warnings red at HEAD under rustc 1.98: 24 unused_must_use (assert_cmd assert() discarde | paiml/ruchy tests+benches; paiml/.github sovereign-ci fallback masks it | pre-commit pmat verify red → clippy --all-targets red → tests discard a must_use value; bench never updated for Expr.contracts; rustfmt output drifted → rustc m |
| PMAT-104 | Z0c | paiml-impl-worker a32af0dbc875a35fe terminated by API rate limit (HTTP 429, opus session limit) during its one resume; h | claude-code harness (SubagentStop on API-error termination) | SendMessage resume returned success:false with rate_limit → lock dir entry remained → hook has no stale-lock detection by design → orchestrator released the sin |
| PMAT-091 | P4-review | agy review lanes (writes=false, --sandbox) ran git checkout of PR branches in the main checkout and created tests.disabl | paiml-implement agy lane doctrine (sandbox does not stop git writes inside the repo) | lanes were given diff commands but chose checkout to inspect both sides → --sandbox restricts the terminal, not git writes in the workspace → the delegate brief |
| PMAT-113 | Z5 stage features | minimal feature set: bin, 11 tests, 8 examples, 3 benches import runtime::repl unconditionally | tests/ + Cargo.toml | gate never built minimal with --all-targets → CI builds default features only (PMAT-111) → minimal was never defined as lib-only in the manifest → no lint named |
| PMAT-112 | Z5 stage features | 4 discarded must_use Assert values in tests/cli_contract_notebook.rs under --all-features | tests/cli_contract_notebook.rs | target only compiles with the notebook feature → CI never enables notebook (PMAT-111) → Z0 wrapped only default-feature targets → clippy runs default features l |
| PMAT-101 | ci/security | quick-xml 0.39 RUSTSEC-2026-0194/0195 via polars csv meta-feature | Cargo.toml | polars csv enables polars-lazy/csv → which pulls polars-stream → which enables polars-io cloud/http → object_store 0.13 pinned by polars-error → quick-xml is ob |
| PMAT-091 | P4 gate | untracked symlink paiml-mcp-agent-toolkit -> /home/noah/src/paiml-mcp-agent-toolkit appeared in the repo root at 18:09 l | process | gate refuses a dirty tree (correct) → lane prompts ban git state changes but not filesystem writes in the repo → the symlink target is the pmat source checkout  |
| PMAT-127 | P4 gate run 3 | cargo package -p ruchy exit 101: 24 tests/*.proptest-regressions files (gitignored) are matched by include tests/** and  | Cargo.toml include (Z1) | include allowlist has tests/** with no negation → proptest writes regression files next to failing property tests → they are gitignored so git status is clean → |
| PMAT-132 | P4 publish | cargo publish -p ruchy: 403 Forbidden authentication failed (CARGO_REGISTRY_TOKEN invalid); stop-the-line per the releas | repository secret (Noah) | the secret exists but crates.io rejects it → its validity was unverifiable before the tag push (B3) → no dry-run publish step exists in the workflow → the previ |

## Estimates

| Item | Value | basis= |
|---|---|---|
| K̂ | not preserved across context compaction | first-run[U] |
| K (actual orchestrator turns) | not measured (no status log file was kept; `status-lint.sh` NotRun) | [U] |
| Budget andon | none fired | — |

## Gaps (every NotRun lane and the artifact that closes it)

| Gap | Closes it |
|---|---|
| `pv` bindings: contracts have falsification tests but no `--implements` equation bindings, so `pmat work complete` refuses (B34) | PMAT-121 |
| `contracts/.pv/cache` tracked on `main` (B35) | PMAT-122 |
| 20 help-only verbs, missing `tooling-with-ruchy` corpus, baseline nondeterminism, compile budget (receipt warns) | PMAT-123, PMAT-124, PMAT-125, PMAT-126 |
| `status-lint.sh` (I-5) NotRun: status blocks were emitted inline, not to a log file | process; next run writes `${XDG_RUNTIME_DIR}/paiml-implement/status.log` |
| Roadmap ticket states remain `inprogress` for merged tickets | PMAT-121 then `pmat work complete` |
| Dispatched tooling left a symlink `paiml-mcp-agent-toolkit` in the repo root (removed; B38) | lane briefs gain "create no files in the repo" |

## Verdict

DONE (2026-09-06) — `ruchy` and `ruchy-wasm` 5.0.0-beta.2 published to crates.io by the operator from the tag checkout (83294baa) under the sovereign release policy (RP-001): org clean-room 10/10 at that SHA, dogfood gate go at that SHA, `cargo publish --locked` in cascade, fresh-container install verified (locked and unlocked), receipts attached to the GitHub release, all three policy gates PASS from the policy branch. The 2026-09-05 STOPPED(publish-token) verdict stands as history: its mechanism (CI held a publish credential) is closed by PMAT-134/135, not by a new token. Still open: merges of #219, #220, the plan PR and the infra PRs wait on the wedged runner fleet (B48); the tag's own tree fails `no-publish-in-ci` until #219 merges.
