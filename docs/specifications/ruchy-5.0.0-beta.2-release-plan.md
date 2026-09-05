# Ruchy 5.0.0-beta.2 — release plan

| Field | Value |
|---|---|
| Plan version | **v2** (post-quorum; v1 was pre-quorum) → v3 as-built |
| Ticket | PMAT-091 (`docs/roadmaps/roadmap.yaml`) |
| HEAD measured | `3a665f36edc170694159746aa1f9bffec4f4a007` (origin/main, 2026-08-11) |
| Evidence (EV) | `docs/specifications/evidence/2026-09-05-release-gather/` — every number below cites a file there, or carries `[U]` (unmeasured). Quorum lane outputs: `docs/specifications/evidence/2026-09-05-quorum-p2/` |
| Process | `paiml-implement`, unattended; stop-the-line only for clean-room red, `ci / gate` red after 3 five-whys loops, publish-token failure |
| Hard gates, in order | clean-room publish → `ci / gate` on `main` → `make pre-release-gate` go → `cargo publish` |

## §1 Version and scope

**Version: `5.0.0-beta.2`** — a crates.io pre-release.

**Why not `4.3.0`.** The language delta is breaking. All seven reserved words
(`requires ensures invariant decreases infra signal yield`) parse as identifiers under the
4.2.1 binary and are rejected at HEAD, both as `let` bindings and as parameters
(EV `keyword-break-and-verbs.txt`). Two real corpus files regress for exactly this reason,
one of them in-repo (EV `differential-regressions-detail.txt`). Semver 2.0 forbids shipping
that as a minor bump.

**Why not `5.0.0` GA.** The spec's own go/no-go
(`docs/specifications/ruchy-5.0-sovereign-platform.md` §10) requires all 13 criteria.
Criterion #4 is PARTIAL (cookbook 77%, `tooling-with-ruchy` empty) and the RC.1 gate
("all seven book repos PASS or ticketed") is not met; #1 and #13 have no in-repo
measurement (EV `spec-5.0-status-and-gate.txt`). GA is not earned. Saying so is part of
the deliverable.

**Why `beta.2` rather than `rc.1`.** CHANGELOG `[5.0.0-beta.1]` says rc.1 waits on the
book-repo integration gate; nothing since 2026-04-04 has closed it (EV `changelog-head.txt`).

### Measured API delta, 4.2.1 → HEAD

| Surface | Delta | EV |
|---|---|---|
| Language | 7 keywords reserved — **breaking** | `keyword-break-and-verbs.txt` |
| Corpus, 1286 `.ruchy` files across 7 corpora | 1104 pass on both; **2 regressions** (keyword collisions); 0 fixed; 180 fail on both (pre-existing) | `differential-check-summary.txt`, `differential-check-4.2.1-vs-head.csv` |
| CLI verbs | 37 → 47 (+10: `apr contracts infra migrate-4to5 model purify sim suggest-contracts tier widget`; −0) | `keyword-break-and-verbs.txt` |
| Transpile output, `examples/` that parse on both | 138 identical, 13 differ (`.to_int()` → `.parse::<i64>()` class), 3 fail on both | `transpile-diff-examples.txt` |
| Library API (`cargo semver-checks`, default features) | "no semver update required" but **0 of 254 lints ran (all skipped)** — inconclusive `[U]`; with `--all-features` the 4.2.1 baseline no longer compiles (`entrenar` 0.7) | `semver-checks-vs-4.2.1-default-features.log`, `semver-checks-vs-4.2.1.log` |
| `cargo check --all-features` at HEAD | **FAILS** (exit 101, 10 errors) — all in `src/stdlib/dataframe.rs` and `src/backend/arrow_integration.rs` against the locked `polars-core 0.55.2`; the `dataframe`/`polars-compat` features are already broken, unrelated to the sovereign crates | `head-all-features-check.log`, `head-all-features-errors.txt` |
| `cargo clippy --all-targets -- -D warnings` at HEAD, rustc 1.98 | **FAILS** — 7 × `unused_must_use` in `tests/cli_contract_fuzz.rs`; CI's `lint` uses `dtolnay/rust-toolchain@stable`, so the next PR hits it | `README.md` (jidoka PMAT-102) |
| Clean-room | **GREEN at HEAD**: packaged crate builds `--release --locked` in an empty `CARGO_HOME` and runs | `cleanroom-empty-cargo-home-build.log`, `cargo-package-verify.log` |

### Scope

The release is HEAD plus §2 zeros Z0–Z8, then Z9 (publish). Ratchet R1 is taken only
after every zero is merged. **§1 scope is met when Z9's acceptance command exits 0.**

Publish-lag closure: beta.2 closes the 7-month lag for `cargo install ruchy --version 5.0.0-beta.2`.
A bare `cargo install ruchy` keeps resolving 4.2.1 until a stable 5.0.0 exists; a 4.2.2
security backport is rejected in §3 with its reason.

### APR-MONO dependency decision (a decision, not a ticket)

Measured inputs:

- The default build pulls `trueno 0.16.5` and `aprender 0.27.8` from crates.io, both
  non-optional, both declaring `rust-version = 1.89` (EV `cargo-tree-siblings.txt`,
  `msrv-and-claims.txt`). `trueno`, `entrenar`, `simular`, `trueno-viz` on crates.io are
  DEPRECATED facades; the live crates are `aprender-compute`, `aprender-train`,
  `aprender-viz`, `aprender-simulate`, all 0.65.2 (EV `aprender-monorepo-facades.txt`).
- Usage is thin: `trueno` in one bridge file (`Vector`, `Matrix`), `aprender` in four
  (`format::{quantize,gguf}`, `serialization`, `preprocessing::PCA`, `online::{drift,corpus}`),
  `entrenar` in two (EV `sibling-bridge-imports.txt`).
- Spike, default build, `trueno`→`aprender-compute 0.65` + `aprender`→`0.65`: **2 errors**,
  both `unresolved import … found an item that was configured out` — a feature flag, not
  code (EV `apr-mono-spike-default-features.txt`).
- Spike v2 (every sovereign dep on its facade — `aprender-compute`, `aprender-train`,
  `aprender-viz`, `aprender-simulate`, `alimentar 0.4`, `presentar 0.3.4`, `forjar 1.25`,
  `bashrs 7.0` — with `--all-features`): **13 errors**, of which **10 are the pre-existing
  polars drift** (identical at HEAD without any migration) and **3 are feature flags**
  (`entrenar::citl` → `aprender-train` feature `citl`; `aprender::format::quantize` and the
  signing items are `cfg`'d behind `aprender-core` features `format-quantize` /
  `format-signing`, which the `aprender` facade does not re-export). Zero code-level API
  errors in the sovereign bridges (EV `apr-mono-spike-v2-all-features.txt`,
  `aprender-monorepo-facades.txt`).
- **MSRV after migration is 1.91**, not 1.89: `aprender`, `aprender-core`, `aprender-compute`
  0.65.2 all declare `rust-version = 1.91` (quorum lane 2, measured; re-measured in EV
  `quorum-p2-remeasure.txt`).

Decision:

1. **Default build consumes the aprender monorepo from crates.io**: the dependency named
   `trueno` becomes `{ package = "aprender-compute", version = "0.65", default-features = false }`
   (so `trueno::` paths keep compiling); `aprender = { version = "0.65", default-features = false }`
   plus `aprender-core = { version = "0.65", features = ["format-quantize", "format-signing"] }`.
   Landed in Z3. Mutation that turns the gate red: any dependency whose crate name is
   `trueno` (Z3's manifest-lint test).
2. **Optional stack** (`sovereign-stack`, `infra`, `simulation`, `shell-target`) **migrates
   in Z3** — the measured cost is three feature flags, under the ≤ 20-error threshold set
   before the spike ran. `entrenar` → `{ package = "aprender-train", features = ["citl"] }`,
   `trueno-viz` → `aprender-viz`, `simular` → `aprender-simulate`, `alimentar 0.4`,
   `presentar 0.3.4`, `forjar 1.25`, `bashrs 7.0`. The deprecated crates are never consumed.
   Platform-heavy features (`cuda`, `wgpu`, `gpu`) stay off; Z3 asserts the default
   feature set of every facade pulls none of them (`cargo tree -e features`).
3. `sovereign-stack` is **not** dropped: dropping it alone would leave the non-optional
   `trueno`/`aprender` pins in place and satisfy nothing.
4. The polars breakage is its own zero (Z8, PMAT-101): it is what makes `--all-features`
   red today and would be attributed to the migration if left in place.
5. **Runtime semantics are covered by the differential, not by type-checking**: the
   interpreter's bridges change implementation while transpiled text stays identical
   (quorum lane 3), so §4 compares `ruchy run` output unconditionally (see Z5).

## §2 EV-ranked tickets (zeros before ratchets)

Each ticket: `pmat work` id · what changes · `pv` contract (same PR) · acceptance command
`A_i` (re-run by the orchestrator, never trusted from a worker) · **DoD = the exact mutation
that turns its gate red** · routing.

### Z0 · PMAT-102 · CLIPPY-1.98-MUST-USE (jidoka — found while committing v1)

- **Why first**: `cargo clippy --all-targets -- -D warnings` fails at HEAD under rustc 1.98
  with 7 × `unused_must_use` on discarded `assert_cmd::Command::assert()` results in
  `tests/cli_contract_fuzz.rs`. The pre-commit hook (`pmat verify`) and CI's `lint` run
  the same lint, so **no commit and no PR can land until this merges**. Five whys: hook red
  → clippy red → 7 discarded must-use values → rustc moved to 1.98 (2026-08-18) after the
  last green CI (2026-08-11) and CI tracks `stable` → nothing was committed in between →
  root: the tests discard the value they claim to check ("just verify the command accepts
  the parameter").
- **Changes** (as measured, the defect was wider than the hook's first report): one shared
  helper `tests/support/mod.rs::assert_args_accepted` asserting the exit code is not clap's
  usage-error code 2 (the stated intent) at all **24** sites in four test files; no lint
  silencing. `benches/transpiler.rs` (4 `Expr` literals missing `contracts`) uses `Expr::new`.
  **`cargo fmt --all -- --check` is red at `main` under both the local 1.98 and CI's 1.97.1**
  (28 files; EV `main-under-ci-toolchain-1.97.1.log`) — the shared CI's fmt step lost its
  suppression after 2026-08-11, so the next CI run fails without this. Z0 formats the 24
  files whose functions are within the pre-commit complexity limits.
- **Z0b · PMAT-103** (same PR, second commit, subagent:opus): the remaining four files
  (`migrate.rs`, `provability.rs`, `sovereign.rs`, `interpreter.rs`) hold six functions the
  hook refuses to let a fmt-only touch pass (cognitive 34–64, cyclomatic 35). Decompose
  exactly those six by extract-method with no behaviour change, then format. A_0b:
  `cargo +1.97.1 fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo test --lib`
  plus the `examples/` run-output differential unchanged before/after (the corpus harness
  from P0). DoD mutation: re-inline one extracted helper → the hook's complexity gate is
  red again on that file.
- **A_0**: `cargo clippy --all-targets -- -D warnings && cargo test --test cli_contract_fuzz`
  (default features — exactly what CI's `lint` runs; `--all-features` is red at HEAD for the
  unrelated polars reason, Z8, and is the feature-matrix stage of Z5)
- **DoD mutation**: rename `--iterations` in one site to `--iteration` → the test is red
  (clap exit 2), where before the fix it was silently green.
- **Routing**: direct (blocking; 7 mechanical sites). Pre-PR review: 3-lane `grillme` quorum
  runs in parallel with CI; merge needs both.

### Z1 · PMAT-092 · RELEASE-PKG-HYGIENE

- **Why**: the daily Windows nightly fails on a tracked path containing `:`
  (EV `gh-ci-runs-detail.txt`); the same 7 paths ship inside the `.crate`, so
  `cargo install` on Windows would fail to extract; the crate is 7.8 MiB compressed of a
  10 MiB limit with 802 quarantined test files, 574 `docs/` files and `.pmat-work/`
  receipts inside (EV `cargo-package-precheck.txt`).
- **Changes**: `[package] include = [...]` allowlist (src, benches, examples, tests, build.rs,
  Cargo.toml, README.md, CHANGELOG.md, LICENSE*, contracts, `static/notebook.html`,
  `golden_traces/`, `notebooks/`, plus **only those `include_str!` targets that exist on
  disk** — quorum lane 2 measured that two paths in EV `package-allowlist-inputs.txt` are
  ghosts extracted from prose). `Cargo.lock` is auto-included for binary packages
  (present in today's list, EV `quorum-p2-remeasure.txt`) and the test asserts it stays.
  Quarantine triage, one verdict per dir, never a fourth dir:
  - `tests.disabled/` (277 files, last touched 2025-10-04) → **delete**;
  - `tests_disabled_for_mutation/` (24 files, `.disabled`/`.NEEDS_REWRITE`, 2025-10-13) → **delete**;
  - `tests_temp_disabled_for_sprint7_mutation/` (501 files, 265 basenames duplicated with
    `tests.disabled/`, 2026-01-09) → **delete**.
  Reasons: none is referenced by Cargo.toml, Makefile or any workflow (EV `quarantine-detail.txt`);
  8 basenames were already restored into `tests/`; git history retains every file
  (`git show <sha>:<path>`), so restoration is a per-test ticket on demand.
  Untrack the 3 `.pmat-work/Apex Hunt: …` directories (`git rm -r --cached`) and ignore
  `.pmat-work/*:*`; file an upstream pmat ticket for ticket-id sanitisation.
- **pv contract**: `contracts/release-hygiene-v1.yaml` —
  `∀ p ∈ package_files(HEAD): ':' ∉ p ∧ top(p) ∈ include_allowlist ∧ Cargo.lock ∈ package_files`.
- **A_1**: `test "$(cargo package --list -p ruchy | grep -cE ':|^tests\.disabled|^tests_disabled|^tests_temp|^\.pmat-work|^docs/')" = 0 && cargo package --list -p ruchy | grep -qx Cargo.lock && test "$(git ls-files | grep -c ':')" = 0 && cargo test --test release_hygiene`
- **DoD mutation**: `git mv tests/lang_comp_tests.rs 'tests/a:b.rs'` → `release_hygiene` red;
  `mkdir tests.disabled && git add` → red.
- **Routing**: subagent:sonnet · |M|=1 · trigger −.

### Z2 · PMAT-093 · RELEASE-YML-THEATER

- **Why**: the publish job cannot succeed today — it publishes a phantom `ruchy-cli`
  member, masks both publishes with `continue-on-error: true`, and reads
  `secrets.CRATES_TOKEN`, which does not exist (repo secrets are `CARGO_REGISTRY_TOKEN`,
  `CARGO_TOKEN`; EV `workflows-snapshot.txt`, `auth-and-pv-surface.txt`).
- **Changes** (`.github/workflows/release.yml`): drop the `ruchy-cli` step; drop both
  `continue-on-error`; `publish-crates` `needs: [create-release, build-binaries, build-wasm]`
  so a broken build never publishes (quorum lane 1: the working-tree builds on three OSes
  and `cargo publish`'s own package-verify build together cover "the packaged crate builds");
  `env: CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}`; publish `ruchy`, poll
  the index until it resolves (≤ 10 min), then publish `ruchy-wasm`;
  `prerelease: ${{ contains(github.ref_name, '-') }}`. Add
  `.github/required-status-checks.txt` containing `gate` (closes pmat CB-2100/CB-1701, EV
  `pmat-comply.txt`).
- **pv contract**: `contracts/release-workflow-v1.yaml` —
  `∀ step ∈ publish-crates: continue_on_error(step) = false ∧ ∀ publish(pkg): pkg ∈ workspace.members ∧ needs(publish-crates) ⊇ {build-binaries, build-wasm}`.
- **A_2**: `cargo test --test release_workflow_lint` (parses `release.yml` and `Cargo.toml`
  members: no `continue-on-error`, no `ruchy-cli`, every `cargo publish` package is a
  member, publish job needs the build jobs, only secret names that exist are referenced).
- **DoD mutation**: insert `continue-on-error: true` under any publish step → red.
- **Routing**: direct (workers may not edit `.github/workflows`, §6.1); token has `workflow`
  scope (EV `auth-and-pv-surface.txt`).

### Z3 · PMAT-094 · RELEASE-DEPS-CLEANROOM

- **Why**: a clone without `../provable-contracts` cannot even load the workspace manifest —
  the dev-dependency is a bare path (issue #195; EV `apr-mono-spike-default-features.txt`
  first run, `path-dep-and-build-rs.txt`); `rust-version = "1.75"` is false (graph max 1.89
  today, 1.91 after migration, EV `msrv-and-claims.txt`, `quorum-p2-remeasure.txt`);
  `chacha20 0.10.1` in the lock is yanked (EV `cargo-package-precheck.txt`); APR-MONO
  decision (§1).
- **Changes**: `[dev-dependencies] provable-contracts = "0.3.1"` (crates.io; the sibling's
  manifest is also 0.3.1, EV `contracts-inventory.txt`); **`rust-version = "1.91"`** +
  README MSRV section; `cargo update -p chacha20`; APR-MONO per §1 including
  `aprender-core` with `format-quantize`/`format-signing`. The `ruchy-wasm` dependency on
  `ruchy` is **not** touched here: `^5.0.0-alpha.1` already matches every `5.0.0-*`
  pre-release, and pinning it to `beta.2` before the workspace bump would be a lie in
  the interim (quorum lane 2); it is set in Z9 with the version bump.
- **Differential coverage for the migration** (quorum lane 3): the corpus run-output
  differential of Z5 must be green on the Z3 branch before merge, and Z3 adds
  `examples/35_simd_arrays_pca.ruchy` exercising `Vector`/`Matrix` lowering and `PCA` so the
  migrated bridges are on the differential's path.
- **pv contract**: `contracts/clean-room-v1.yaml` —
  `build(extract(package(HEAD)), CARGO_HOME = ∅, --locked) = 0 ∧ build(…, unlocked) = 0 ∧ rust_version(ruchy) ≥ max_{d ∈ deps} rust_version(d) ∧ ∄ d: name(d) = trueno`.
- **A_3**: `cargo test --test release_manifest_lint && W=$(mktemp -d)/wt && git worktree add --detach "$W" HEAD && (cd "$W" && cargo metadata --format-version 1 >/dev/null) ; rc=$?; git worktree remove --force "$W"; test $rc = 0 && cargo test --no-run -p ruchy && cargo check -p ruchy --lib --all-features`
  (the worktree lives where no `../provable-contracts` exists — the fresh-clone shape; the
  packaged-crate build itself is Z5 stage 5, not A_3).
- **DoD mutation**: reintroduce `path = "../provable-contracts/…"` → red; set
  `rust-version = "1.89"` → red (the test computes the graph max via `cargo metadata`).
- **Routing**: subagent:opus · |M|≥2 (manifests, README, bridges) · trigger Q1 → delegate plan
  review (`grillme`, 3 lanes) before implementation.

### Z4 · PMAT-095 · EXAMPLES-MIGRATE-4TO5

- **Why**: the repo's own `examples/24_math_science.ruchy` no longer parses at HEAD
  (keyword collision, EV `differential-regressions-detail.txt`); criterion #12 says
  `migrate-4to5` handles every keyword conflict — dogfood it.
- **Changes**: `ruchy migrate-4to5 examples/`; `tests/examples_must_parse.rs` with a
  checked-in manifest of every example that parses on both binaries plus the migrated file
  (155 files, EV `differential-check-4.2.1-vs-head.csv`). **Semantic check** (quorum lane 3):
  for every migrated file that runs under 4.2.1, compare the 4.2.1 `run` stdout of the
  *original* file with the HEAD `run` stdout of the *migrated* file — never the same file
  on both binaries.
- **pv contract**: `contracts/examples-parse-v1.yaml` — `∀ f ∈ manifest: parse(f) ≠ ⊥`.
- **A_4**: `timeout 10 target/release/ruchy check examples/24_math_science.ruchy && cargo test --test examples_must_parse`
- **DoD mutation**: `git checkout HEAD~1 -- examples/24_math_science.ruchy` → red.
- **Routing**: subagent:sonnet · |M|=1. Downstream collision in
  `../ruchyruchy/validation/tests/test_property_framework.ruchy` → issue on `paiml/ruchyruchy`
  (`blocker=`, other repo).

### Z5 · PMAT-096 · PRE-RELEASE-GATE-V2

- **Why**: P4 mandates the gate is rewritten before it is run. The current gate scores
  43/100 BLOCKED with a coverage stage that reads 0% without measuring and a mutation
  stage that scores 15 points for producing nothing (EV `pre-release-gate-result.txt`,
  `pre-release-gate.sh.snapshot`). Quorum lanes 1 and 3 both found v1's differential
  sampled one point (transpile text) and would miss an interpreter-only change.
- **Changes** (`scripts/pre-release-gate.sh`, bashrs-clean, + `make pre-release-gate`), every
  stage PASS/WARN/FAIL, nothing silently 0:
  1. `cargo test --lib`;
  2. **feature matrix**: `cargo clippy --all-targets -- -D warnings` for each of default,
     `--all-features`, `--no-default-features --features minimal`, plus `cargo fmt --all --check`
     (the nightly-bench fmt drift, EV `dispatch-and-nightly-detail.txt`);
  3. **verb surface derived from the built binary** (`ruchy --help`), each verb run on a
     golden input under `tests/golden/` with `timeout 10`, comparing **exit code and stdout**
     against a checked-in expectation; verbs that need a server, network or a TTY
     (`repl serve notebook mcp …`) get a `--help` smoke and are listed as `warn`;
  4. **three-way differential** against the 4.2.1 binary (`RUCHY_BASELINE_BIN`, else
     `cargo install ruchy --version 4.2.1 --locked --root <cache>`) over `examples/` and every
     sibling book corpus present: for **every** file, (a) `check` exit, (b) `transpile` text,
     (c) `run` exit + stdout — (c) is compared **unconditionally**, not only when (b) differs.
     A file that passes (a) or (c) on 4.2.1 and fails at HEAD is FAIL unless listed in
     `scripts/release-known-breaks.txt` with a ticket; a (b) diff alone is informational;
     a (c) stdout diff is FAIL unless listed in `scripts/release-known-fixes.txt` with a ticket.
     For `examples/` that `compile` under both, also compare the compiled binaries' stdout
     (time-boxed, `warn` on budget exhaustion with the count of files skipped — no silent cap);
  5. **clean-room**: `cargo package -p ruchy` → extract → `CARGO_HOME=$(mktemp -d) cargo build --release --locked`
     **and** a second build without `--locked` (fresh resolution) → `--version` equals the
     crate version in both;
  6. package hygiene: 0 colon paths, ≤ 10 MiB compressed, `Cargo.lock` present;
  7. SATD via `pmat analyze satd --fail-on-violation` (not grep); threshold 0 → FAIL once
     R1 has merged (`scripts/release-gate.toml` `satd_max = 0`), WARN before;
  8. receipt JSON → `docs/specifications/evidence/<date>-dogfood/receipt.json`, verdict
     `go`/`no-go`, `warns[]`; exit 1 on `no-go`.
  The coverage/mutation/doc-% point scores are removed: coverage is CI's `coverage` job
  (EV `workflows-snapshot.txt`), mutation is Tier 3 (`Makefile`).
- **pv contract**: `contracts/dogfood-receipt-v1.yaml` — `verdict = go ⟺ ∀ stage: status ≠ FAIL`.
- **A_5**: `make pre-release-gate && jq -e '.verdict=="go"' docs/specifications/evidence/*-dogfood/receipt.json && cargo test --test dogfood_receipt_schema && bashrs lint scripts/pre-release-gate.sh`
- **DoD mutation**: delete the `clean_room` stage → `dogfood_receipt_schema` red (required
  key missing); `RUCHY_BIN=/bin/false make pre-release-gate` → `no-go`; a fixture file whose
  4.2.1 `run` stdout differs from HEAD's and is not in the known-fixes list → `no-go`.
- **Routing**: subagent:opus · |M|=3 (script, Makefile, tests) · trigger Q1 → delegate plan review.

### Z6 · PMAT-097 · CONTRACTS-DISPATCH

- **Why**: `contracts/` holds 2 contracts against 472 kLOC and neither covers the two
  dispatch hot paths — `transpile_expr` (`src/backend/transpiler/expr_dispatcher.rs:86-175`,
  TDG B+) and `eval_expr_kind` (`src/runtime/interpreter.rs:219-254`, B+) (EV
  `pmat-query-dispatch.txt`).
- **Changes**: `contracts/transpile-dispatch-v1.yaml` and `contracts/eval-dispatch-v1.yaml`
  with equations `dispatch_totality: ∀ e: f(e) ∈ Ok ∪ Err` (never a panic) and
  `dispatch_determinism: f(e) = f(e)`; `pv codegen contracts/ -o src/generated_contracts.rs`;
  `contract_pre_/post_` macros invoked in both functions (they expand to `debug_assert!`,
  so release output is unchanged — quorum lane 3 checked this); `tests/dispatch_contracts.rs`
  drives **every `ExprKind` variant** through both under `catch_unwind`: an exhaustive
  `match` over `ExprKind` in the test generator (so a new variant fails to compile until
  covered) plus property-generated payloads (quorum lane 1: a one-arm mutation is not
  enough evidence of totality).
- **A_6**: `pv lint contracts/ && cargo test --test dispatch_contracts`
- **DoD mutation**: replace **any** `transpile_expr` arm body with `unreachable!()` → red;
  the test's exhaustive match makes every arm a mutation target.
- **Routing**: subagent:opus · |M|=2 (transpiler, runtime) · disjoint from Z5, so both
  worker slots run together.

### Z7 · PMAT-098 · ROADMAP-CONSOLIDATE

- **Decision**: `docs/roadmaps/roadmap.yaml` is the single source of truth. It is the file
  `pmat work` writes (PMAT-091…102 landed there today), the file `discover.sh` and
  `target-guard.sh` read, the only one touched since 2026-02-10 (4 commits vs 0).
  `docs/execution/roadmap.yaml` is an 11,241-line narrative frozen 2025-12-09 and referenced
  only by CLAUDE.md (5 lines) (EV `roadmaps-compare.txt`, `roadmap-and-hooks.txt`).
- **Changes**: `git mv docs/execution/roadmap.yaml docs/archive/roadmap-execution-frozen-2025-12-09.yaml`;
  CLAUDE.md lines 11, 262, 307, 523, 552 → `docs/roadmaps/roadmap.yaml`; `pmat roadmap sync`
  (CB-1655, EV `pmat-comply.txt`).
- **pv contract**: `contracts/roadmap-sot-v1.yaml` —
  `|{p ∈ tracked : basename(p) = roadmap.yaml ∧ p ∉ docs/archive}| = 1`.
- **A_7**: `cargo test --test roadmap_single_source && ! grep -q docs/execution/roadmap.yaml CLAUDE.md`
- **DoD mutation**: `git mv` the archive back → red.
- **Routing**: subagent:haiku · |M|=1 · mechanical. (All three quorum lanes accepted Z7 unmodified.)

### Z8 · PMAT-101 · DATAFRAME-POLARS-DRIFT

- **Why**: `cargo check -p ruchy --lib --all-features` at HEAD exits 101 with 10 errors,
  all in `src/stdlib/dataframe.rs` and `src/backend/arrow_integration.rs`
  (`DataFrame::new` arity, `CsvReadOptions`/`CsvWriter` gone, `ChunkedArray` iteration)
  against the locked `polars-core 0.55.2` (EV `head-all-features-errors.txt`). Spec
  criterion #3 is measured with `cargo test --all-features`; CI runs default features
  only, so this shipped silently.
- **Changes**: port the two files to the locked polars API (or pin `polars` to the last
  compiling minor if the port exceeds the two files — say which in the PR); add a
  feature-gated golden `examples/dataframe/36_dataframe_basics.ruchy` and a
  `#[cfg(feature = "dataframe")]` test so the port is exercised (quorum lane 3: no
  dataframe example exists in any corpus today); Z5 stage 2's feature matrix keeps it
  red-on-regression.
- **pv contract**: `contracts/feature-matrix-v1.yaml` — `∀ F ∈ {default, all, minimal}: check(ruchy, F) = 0`.
- **A_8**: `cargo check -p ruchy --lib --all-features && cargo check -p ruchy --lib --no-default-features --features minimal && cargo test --features dataframe --test dataframe_golden`
- **DoD mutation**: reintroduce the one-argument `DataFrame::new(cols)` call → red.
- **Routing**: subagent:opus · |M|=2 (stdlib, backend) · disjoint from Z3's manifests, so it
  can share the slot pair with Z7/R1.

### R1 · PMAT-099 · SATD-ZERO (ratchet, after all zeros)

- 15 markers (EV `satd-markers.txt`): 7 are test-string literals in
  `src/bin/handlers/commands_tests.rs` (false positives of the grep-based stage; Z5's
  pmat-based stage or `concat!` splitting), 7 `PARSER-XXX` placeholders → the real ticket id
  from `git log -S`, 1 `TODO` in `src/transpiler/canonical_ast.rs:323` → ticket + marker removed.
- **A_R1**: `pmat analyze satd --path src --fail-on-violation` exit 0, and Z5's
  `scripts/release-gate.toml` flips `satd_max = 0` in the same PR so the stage becomes FAIL.
- **DoD mutation**: add `// TODO: x` in `src/` → `make pre-release-gate` is `no-go`
  (quorum lane 2: with the stage at WARN the mutation could not redden the gate; the flip
  is part of this ticket).
- **Routing**: subagent:haiku.

### Z9 · PMAT-100 · RELEASE-PUBLISH-5.0.0-beta.2 (terminal)

- **Changes**: `[workspace.package] version = "5.0.0-beta.2"`; `ruchy-wasm/Cargo.toml`
  `ruchy = { version = "5.0.0-beta.2", path = ".." … }` (moved here from Z3);
  CHANGELOG `[5.0.0-beta.2]` from merged PRs (#196, #199, #200, Z0–Z8, R1); tag
  `v5.0.0-beta.2` → fixed `release.yml`.
- **A_9**: `curl -s https://crates.io/api/v1/crates/ruchy/versions | jq -e '.versions[]|select(.num=="5.0.0-beta.2")' && docker run --rm rust:1.91-slim sh -c 'apt-get update -qq && apt-get install -y -qq pkg-config libssl-dev >/dev/null; cargo install ruchy --version 5.0.0-beta.2 --locked && cargo install ruchy --version 5.0.0-beta.2 --force && ruchy --version'`
  (both `--locked` and fresh resolution, quorum lane 1; the container tag matches the
  measured MSRV, quorum lane 2).
- **DoD**: A_9 exit 0; receipt filed; a ticket per `warn` in the dogfood receipt.
- **Routing**: direct (tag, push, PR are the orchestrator's only).

### Order and slots

Z0 (direct, blocking) → Z1 ‖ Z4 (disjoint, slots B+C) → Z2 (direct) → Z3 (opus, after
delegate plan review) → Z5 ‖ Z6 (disjoint) → Z8 ‖ Z7 → R1 (haiku) → Z9 (direct). Slot A is
the agy delegate throughout (P2 lanes, Z3/Z5 plan reviews, pre-PR review quorum — lane
defaults are now verified, see §7).

### Mandatory candidates — verdicts

| Candidate | Verdict |
|---|---|
| release.yml theater fix | **ACCEPT** → Z2 (plus the missing-secret finding) |
| APR-MONO dependency decision | **DECIDED** in §1 (default build and optional stack both migrate; MSRV 1.91) |
| quarantine triage | **ACCEPT** → Z1, delete × 3 dirs, reasons above |
| publish-lag closure | **ACCEPT** → Z9 as a pre-release; default-install lag remains (§3) |
| contract coverage on the dispatch path | **ACCEPT** → Z6 |
| two-roadmap consolidation | **ACCEPT** → Z7 |

## §3 Do-not-do (one line each)

- **5.0.0 GA** — the spec's 13-criterion gate is unmet (§1); not this release.
- **4.2.2 security backport** of the RUSTSEC clearance (#199) onto the v4.2.1 line — a separate maintenance branch with its own dependency resolution; ticket filed, not this release.
- **Restore any quarantined test** — nothing references them; delete is the triage verdict (Z1); per-test restoration from git history on demand.
- **CB-200 (827 functions below A), CB-040 file health, CB-081 (64 direct deps > 50), doc coverage 45% → 80%** — unbounded ratchets with no release-blocking effect.
- **bashrs errors in the other ~20 scripts (CB-400)** — only the gate script is rewritten (Z5).
- **The 28 in-repo examples and 152 downstream files that fail on both 4.2.1 and HEAD** — pre-existing; ticket EXAMPLES-TRIAGE for rc.1.
- **DATAFRAMES-001 and the cookbook `Expected RightBrace, found Let` parser issue** — pre-existing, not beta.2 regressions (EV `spec-5.0-status-and-gate.txt`).
- **`stash@{0}` (issue #196 KEYWORDS-const refactor)** — the fix already landed as 70576494; the remainder is a test-only refactor; left for Noah, not discarded.
- **Nightly-bench rustfmt drift** — nightly rustfmt formats three files differently from stable; a CI-toolchain pin, ticket filed.
- **`pmat comply migrate` (pmat is 29 versions behind)** — environment, not repo.
- **CB-1653 ladder drift on 5 pre-April tickets** — historical ticket metadata.
- **30 automated "Web Quality Alert" issues** — not code; not touched.
- **Publishing `ruchy-embed`** — 0.1.0, path dep without version, never published; out of scope.
- **Migrating downstream repos** (ruchyruchy keyword collision) — other repo; issue filed as `blocker=`.
- **Making the 7 keywords contextual to earn `4.3.0`** — parser work that the language spec explicitly chose not to do.
- **Dropping `--locked` from the install check** (quorum lane 1) — `Cargo.lock` ships in the crate, so `--locked` is what users get by default; both modes are checked instead.

## §4 Dogfood contract

Receipt (`docs/specifications/evidence/<date>-dogfood/receipt.json`), produced only by
`make pre-release-gate` (Z5):

```json
{"schema_version": 1, "version": "5.0.0-beta.2", "head": "<sha>", "baseline": "4.2.1",
 "stages": {
   "tests":        {"status": "PASS|FAIL", "exit": 0},
   "features":     {"status": "…", "default": 0, "all": 0, "minimal": 0, "fmt": 0},
   "verbs":        {"status": "…", "total": 47, "pass": 0, "warn": 0, "fail": 0, "list": []},
   "differential": {"status": "…", "files": 0, "both_pass": 0,
                    "check_regressions": [], "run_regressions": [], "run_stdout_diffs": [],
                    "known_breaks": [], "known_fixes": [], "fixed": 0, "both_fail": 0,
                    "compiled_compared": 0, "compiled_skipped_budget": 0},
   "transpile":    {"status": "…", "identical": 0, "differs": []},
   "clean_room":   {"status": "…", "locked_exit": 0, "unlocked_exit": 0, "binary_version": ""},
   "package":      {"status": "…", "files": 0, "bytes_compressed": 0, "colon_paths": 0, "has_lock": true},
   "satd":         {"status": "…", "count": 0, "max": 0}
 },
 "warns": [], "verdict": "go|no-go"}
```

Thresholds (each cites the P0 measurement that motivated it): `tests` exit 0; `features`
all three exits 0 and `fmt` 0 (HEAD is red on `all` today, Z8); `verbs.fail = 0` with exit
code and stdout compared, every `warn` becomes a ticket (P4); `differential.check_regressions ⊆ known_breaks`
(today: 1 in-repo file, fixed by Z4, so the list is empty at release);
`run_regressions ⊆ known_breaks`; `run_stdout_diffs ⊆ known_fixes` (each entry carries a ticket
id — the `.to_int()` class today, EV `transpile-diff-examples.txt`); a `transpile.differs`
entry alone is informational; `clean_room.locked_exit = unlocked_exit = 0` and
`binary_version = version`; `package.colon_paths = 0`, `bytes_compressed ≤ 10 MiB`,
`has_lock`; `satd.count ≤ satd.max` (`max` flips to 0 with R1). `verdict = go` iff no stage
is `FAIL`. Any bounded stage (compiled comparison budget) reports what it skipped — no
silent cap.

## §5 Publish runbook

1. Every Z ticket merged; `main` green on the `gate` check.
2. On `main`: `make pre-release-gate` → `receipt.json` with `verdict = go` (else stop; the
   failing stage is a five-whys ticket).
3. Z9 PR: version bump + `ruchy-wasm` dep + CHANGELOG → merge.
4. `git tag -a v5.0.0-beta.2 -m "5.0.0-beta.2" && git push origin v5.0.0-beta.2` →
   `release.yml`: create-release (prerelease) → build-binaries (4 targets) → build-wasm →
   publish `ruchy` → poll index → publish `ruchy-wasm`.
5. `gh run watch`; an auth failure in `publish-crates` is the publish-token stop-the-line.
6. Verify with A_9 (crates.io API + fresh `rust:1.91-slim` container, locked and unlocked).
7. Write plan v3 (as-built), `docs/audits/impl-PMAT-091-receipt.md`, and a ticket per receipt `warn`.

## §6 Blockers (andons — appended, never resolved silently)

| # | Andon | Handling |
|---|---|---|
| B1 | Dirty tree at start (issue #196 WIP + `Cargo.lock` re-resolve) | stashed as `stash@{0}`, diff in EV `dirty-tree-at-start.diff`; not discarded |
| B2 | pmat 3.36.0 has no `hooks install --strict` and installs no commit-msg hook | `Pmat-Ticket: <id>` trailers written by hand on every commit |
| B3 | `secrets.CRATES_TOKEN` does not exist | Z2 switches to `CARGO_REGISTRY_TOKEN`; validity is unverifiable before the tag push |
| B4 | `pmat comply check` exit 1, 10 failures (EV `pmat-comply.txt`) | Z2 closes CB-2100/CB-1701, Z7 closes CB-1655; the rest are §3 |
| B5 | gitignored `.cargo/config.toml` ("temporary coverage config") redirects `target-dir` | every measurement sets `CARGO_TARGET_DIR` explicitly; environment noise |
| B6 | `cargo semver-checks` all-features baseline unbuildable (`entrenar` 0.7); default-features run skipped all 254 lints | library API delta is `[U]`; not used as a semver basis |
| B7 | `ruchydbg` not installed (CLAUDE.md's mandatory parser-debug tool) | no parser change is in scope; unused |
| B8 | `tooling-with-ruchy` absent locally | differential covers 6 of 7 book repos |
| B9 | Turn estimate `K̂ = 9`, `basis = first-run[U]` (no `docs/audits/impl-estimates.jsonl` yet) | the prompt's three stop conditions govern; estimates logged as they land |
| B10 | `gate_cmd_fallback = true` in `discover.json` (`cargo test --workspace`) | named here and in the receipt |
| B11 | Unmerged branches from other sessions exist (`fix/emit-org-required-gate-context`, `fix/add-workflow-dispatch`, `fix/clippy-warnings`, `fix/rustsec-backlog`, `ci/self-hosted-runners`) and two sibling worktrees (`../ruchy-gate`, `../ruchy-sec`) | left untouched; Z2 does not depend on them (`gate` already reports on `main`) |
| B12 | HEAD `--all-features` is red (polars drift) | Z8 (PMAT-101) filed from measurement, not from the seed |
| B13 | **Jidoka**: `cargo clippy --all-targets -- -D warnings` is red at HEAD under rustc 1.98; the `pmat verify` pre-commit hook refused the v1 commit and would refuse every worker commit | Z0 (PMAT-102) filed with five whys; fixed at root; cherry-picked into every open branch |
| B14 | P2 `teamwork` lanes did not fan out (one brain entry per lane): three single-agent reviews, not three teams; lane 2 needed two reruns (Bash 10-min cap, `/teamwork-preview` confirmation gate) | lane defaults verified for `grillme`/`plan` quorum modes, which P3 reviews use; `teamwork` is not relied on again |
| B15 | Lane 1's nine findings are all `grounding=asserted` with unverified line numbers | each was re-derived against the plan text before folding; two were rejected (§7) |
| B16 | Both first workers (Z1, Z4) hit the 40-turn `maxTurns` before committing | resumed once each after Z0 lands (the hook blocked their RED commits) |
| B17 | **`ci / gate` green does not mean `--all-targets` clippy or the integration tests pass**: the shared `paiml/.github` `sovereign-ci.yml` lint step runs `cargo clippy $CLIPPY_ARGS … \|\| cargo clippy -p "$REPO_NAME"` and the test step `cargo test … \|\| cargo test --lib -p "$REPO_NAME"` — a silent fallback to lib-only (EV `sovereign-ci-lint-job.txt`). That is how HEAD's broken bench and 24 discarded `assert()`s stayed green since April | other repo: issue filed on `paiml/.github` (`blocker=`); Z0 fixes the ruchy side; Z5 stage 2 runs the full clippy/test matrix locally with **no fallback**, so the local gate is the real one |
| B18 | Z0 grew from 7 sites to 24 sites + 1 bench. `cargo fmt --all --check` also drifts in 28 files under the local rustc 1.98, and 4 property-test files (`.rustfmt.toml`'s nightly-only `ignore` list) are non-idempotent there — but CI's lint runs in a container pinned by digest at **rustc 1.97.1** (EV `sovereign-ci-lint-job.txt`, run 31494372313), where the tree is fmt-clean | Z0 ships only the code fixes; the pre-commit complexity gate (per-function 30/25) refuses fmt-only touches of files holding six pre-existing over-limit functions, and a 1.98-formatted tree would fail CI's 1.97.1 fmt check anyway. The 1.97.1 toolchain is installed locally; Z0 and Z5 verify fmt/clippy under `cargo +1.97.1`, the CI toolchain, and record it in `scripts/release-gate.toml` as `ci_toolchain` |
| B19 | **Escalation (design decision not given)**: local `stable` (1.98) ≠ CI image (1.97.1). Either pin `rust-toolchain.toml` to the image's version and bump both together, or accept local/CI fmt and clippy drift as a standing WARN. Not decided here; Z5 mirrors CI by toolchain selection, which needs no policy change | recorded for Noah; `PARTIAL(escalate)` only if Z5 cannot be built without the decision — it can |
| B20 | Six functions exceeded the hook's per-function limits (cognitive 34–68): `is_identifier_usage` (migrate.rs), `handle_provability_command` (provability.rs), `handle_suggest_contracts` (sovereign.rs), `call_function`, `eval_type_cast`, `resolve_qualified_name` (interpreter.rs). Because the hook's fmt check is whole-tree, they blocked every commit on every branch | Z0b (PMAT-103) decomposed all six by extract-method (measured before→after: 18/34→3/6, 35/64→3/2, 9/36→5/4, 15/37→8/8, 8/36→3/4, 13/68→4/11); `examples/` run-output differential 153 identical / 1 pre-existing HashMap-order nondeterminism (`09_async_await.ruchy`, differs run-to-run on the unmodified binary too); commit `34a53fa6` |
| B21 | **`cargo test --workspace` (the discovered `gate_cmd`) is red at `main`**: `src/bin/handlers/mcp_handler.rs` tests call `handle_mcp_command`, whose `#[cfg(not(feature = "mcp"))]` body ends in `std::process::exit(1)`, so the test binary dies ("exited abnormally"). CI's test step falls back to `cargo test --lib -p ruchy`, which is how it stayed green | Z0c (PMAT-104): the stub returns `Err` instead of exiting, the tests assert the error; third commit of the Z0 PR, since every ticket's gate re-run needs it |
| B22 | **The `ruchy` bin unit-test target (952 tests) has never run to completion**: `test_handle_watch_mode_setup` spawned a thread that called `std::process::exit(0)` after 10 ms — cargo saw exit 0 and reported success after ~640–666 of 952 tests every time. With it removed, the target exposes: 2 tests asserting `/nonexistent` is absent (it exists on this host), 1 case-sensitive assertion bug (`notebook feature`), 1 wrong expectation (`parse_source("")` is an error in 4.2.1 and HEAD alike), 2 further failures (`test_handle_test_dispatch_with_filter`, `test_run_cargo_build_no_project`), and **18 tests that block forever** (11 `prove_handler`, 4 watch-mode, 3 dispatch in `ruchy.rs`) | all under PMAT-104, one commit on the Z0 PR; the fix list is in the commit body. Measured on the side: `ruchy prove < /dev/null` **hangs on 4.2.1 (exit 124) and exits 0 at HEAD** — a user-facing fix already on `main`, to be named in the beta.2 CHANGELOG |

## §7 Quorum log (P2) — folded into v2

Three agy lanes, `writes=false`, schema `quorum-schema.json`; raw outputs in
`docs/specifications/evidence/2026-09-05-quorum-p2/lane-{1,2,3}.json`.

| Lane | Lens | Verdict | Folded |
|---|---|---|---|
| 1 | adversarial / CF-4 | FAIL | Z5 compares run exit+stdout and both locked/unlocked clean-room; Z6 exhaustive variants; Z8/Z5 feature matrix incl. `minimal`; Z2 publish needs build jobs |
| 2 | clean-room | do-not-implement-as-written (Z1, Z3, R1 as written) | Z1 includes only existing paths and asserts `Cargo.lock`; Z3 MSRV 1.91 and `ruchy-wasm` bump moved to Z9; R1 flips the SATD stage to FAIL |
| 3 | semantic preservation | do-not-implement-as-written (Z5 as written) | Z5 run-output comparison unconditional; Z3 adds a SIMD/PCA example to the differential; Z4 compares original-on-4.2.1 vs migrated-on-HEAD; Z8 adds a dataframe golden; Z6 cleared |

Rejections, each with reason:

- Lane 1/2 "Z1's allowlist omits `Cargo.lock`, breaking `--locked` installs" — **rejected as a
  defect, accepted as an assertion**: cargo auto-includes `Cargo.lock` for packages with
  binaries regardless of `include` (it is in today's list, EV `quorum-p2-remeasure.txt`);
  Z1's test asserts it stays.
- Lane 1 "drop `--locked` from A_9 because it masks fresh resolution" — **rejected**:
  `--locked` is what users get by default because the lock ships; both modes are checked.
- Lane 2 "bumping `ruchy-wasm`'s dependency to beta.2 breaks workspace resolution mid-CI" —
  **rejected as a break** (`^5.0.0-alpha.1` matches every `5.0.0-*` pre-release), **accepted
  as sequencing**: the bump moves to Z9.
- Lane 1 "Z3's A_3 worktree retains excluded files, so it is not a clean-room proof" —
  **rejected as scoped**: A_3 proves sibling-path independence only; the packaged-crate
  build is Z5 stage 5 and the hard gate.
- Lane 1 line numbers (plan v1 lines 108–286) — **not relied on**; findings re-derived from text.

Open questions carried: lane 2 could not cover the Windows `:` paths, the `chacha20` yank or
the 14.19 MB binary-size budget — Z1, Z3 and Z5 stage 6 cover them from the P0 measurements.
