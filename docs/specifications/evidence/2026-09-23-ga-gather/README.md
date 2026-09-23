# GA gather — 2026-09-23 (RHLGA-1, measurement only, no code changes)

Ticket: RHLGA-1. HEAD measured: `270a065c54af50a835652fc5dd19781adf3fc821` (branch
`RHLGA-ruchy-5-0-0`), version string `5.0.0-beta.2` (`Cargo.toml:170`).

This gather measures the current state of the 13 GA success criteria in
`docs/specifications/ruchy-5.0-sovereign-platform.md` §10 at this HEAD. It reuses the
methods of `docs/specifications/evidence/2026-09-05-release-gather/` where practical.

**Budget note:** this pass ran under a tight time budget with heavy resource contention —
disk was at 98-99% full (31-40GB free of 1.8TB) for the whole session, and a concurrent
`cargo test --workspace` was running in this same working directory for the duration,
holding the `target/` build-directory lock for long stretches. **Correction (orchestrator):**
that run was the RHLGA-1 orchestrator's own baseline, not another session; launched under
`timeout`, it bypassed the shell's `CARGO_TARGET_DIR` redirect and built a 55 GB `./target`
on the root disk, which is what filled it. The stray `./target` was deleted. Several long-running measurements
(`cargo check --all-features`, `cargo build --target wasm32-unknown-unknown`, the release-mode
embed startup benchmark, book-repo clones, `pmat tdg`, `cargo llvm-cov`, `cargo mutants`) were
started but did not finish before the budget ran out and are recorded as `UNMEASURED` with
reason `"not reached in budget"` per the coordinator's resume instruction, rather than
reported as failures. Every `UNMEASURED` row below names the smallest next step.

## Results

| # | Criterion | Threshold | Measured | Status | Command | Fix |
|---|-----------|-----------|----------|--------|---------|-----|
| 1 | All 9 pillar specs pass acceptance | 100% | 16/16 passed (`sovereign_nine_pillar_acceptance.rs`) | **MET** | `target/debug/deps/sovereign_nine_pillar_acceptance-8ff9bfcd28834341` | — |
| 2 | ruchy-embed startup latency | < 5ms | Only a debug-profile binary available; debug-mode pass (0.00s) is not evidence for the release-mode threshold | **UNMEASURED** | `cargo test --release -p ruchy-embed --test startup_benchmark -- --ignored` | Re-run release build once target-dir lock is free |
| 3 | Zero regressions in 4.x test suite | 0 failures | `cargo check --all-features` still compiling (no errors yet) when budget ran out; PMAT-101 (polars drift) still `inprogress` in roadmap.yaml | **UNMEASURED** | `cargo test --all-features` | Resolve/confirm PMAT-101, then run to completion in an isolated target dir |
| 4 | Downstream book repos compile | 100% on each of 7 | Not started — disk headroom risk (31GB free) deprioritized 7 fresh clones+builds | **UNMEASURED** | book validation harnesses (Appendix B) | Free disk, clone 7 `paiml/*` repos fresh, run each harness against `target/release/ruchy` |
| 5 | WASM target functional | All WASM tests pass | Build started (wasm32-unknown-unknown target confirmed installed, no wasm-pack), killed before finishing to free lock/disk | **UNMEASURED** | `cargo build -p ruchy-wasm --target wasm32-unknown-unknown --release` | Re-run in isolated target dir once lock contention clears |
| 6 | Binary size (default) | < +20% vs 4.x | HEAD 11,034,536 B vs 4.2.1 14,233,736 B = **-22.48%** (smaller) | **MET** | `stat -c%s target/release/ruchy` vs `~/.cargo/bin/ruchy` | — |
| 7 | Compile time (default) | < +30% vs 4.x | No fair clean-vs-clean comparison affordable at 98% disk; one non-comparable data point: 2m37s incremental build of HEAD against an already-warm ~50GB target dir | **UNMEASURED** | `cargo build --release --timings` (clean, both versions) | Re-run with >150GB free disk, fresh target dirs both sides |
| 8 | TDG grade | >= A- new code | Not reached in budget | **UNMEASURED** | `pmat tdg . --min-grade A- --fail-on-violation` | Scope to 9 pillar dirs (Appendix A), run |
| 9 | Test coverage | >= 80% new pillar code | Not reached in budget | **UNMEASURED** | `cargo llvm-cov` | Mold-disable per CLAUDE.md, scope to pillar modules |
| 10 | Mutation coverage | >= 75% new pillar code | Not reached in budget; prior agent-memory notes 3/3 past attempts on this repo were interrupted mid-build | **UNMEASURED** | `cargo mutants --file <pillar-file>` | Dedicated window, no lock contention |
| 11 | Zero unsafe in transpiler output | 0 occurrences | 2/2 passed (`sovereign_zero_unsafe_transpile.rs`); `pmat query --literal "unsafe {"` hits are all interpreter/runtime internals (jit/compiler.rs, runtime/arena.rs, runtime/bytecode/{opcode,vm}.rs), not transpiler output | **MET** | `target/debug/deps/sovereign_zero_unsafe_transpile-69b5ff1d965b68d5` | — |
| 12 | `migrate-4to5` handles all keyword conflicts | 100% | 4/4 passed (`sovereign_migrate_4to5_e2e.rs`) | **MET** | `target/debug/deps/sovereign_migrate_4to5_e2e-2476d61c68459953` | — |
| 13 | Provability Mandate (F1-F5, §14.5) | All 5 in-range | `ruchy tier` scorecard {f1,f2,f4,f11}=OK on examples/ and src/, but F1's 100% non-trivial is on only 6 contracted functions; F3 (pairwise oracle correlation) and F5 (contract-free-ships-to-crates.io) have **no instrument at all** in this HEAD | **UNMEASURED** | `ruchy tier examples/ --json`, `ruchy tier . --json` | Build F3/F5 instruments (net-new tooling, no ticket found) |

## Go/No-Go

Per §10, GA requires **all 13 criteria MET**. At this HEAD: 4 MET (#1, #6, #11, #12), 0 NOT_MET,
9 UNMEASURED. GA cannot be declared from this evidence — most of the remaining work is
re-running already-identified commands with more disk headroom and without lock contention,
not new implementation, except criterion #13's F3/F5 falsifiers which need new instrumentation.

## Files

- `criteria.json` — machine-readable, 13 entries, schema `{n, criterion, threshold, measured, status, cmd, exit, fix}`.
- `binary-size-comparison.txt` — raw byte sizes and pct delta for criterion #6.
- `tier-examples.json`, `tier-src.json` — raw `ruchy tier --json` output for criterion #13 (F1/F2/F4/F11).
- `unsafe-query-pmat.txt` — raw `pmat query --literal "unsafe {"` output for criterion #11 context.
- `criterion-1-nine-pillar-acceptance.log`, `criterion-11-zero-unsafe-transpile.log`,
  `criterion-12-migrate-4to5.log` — raw test-binary output for the 3 MET-by-test criteria.
- `criterion-2-startup-debug-proxy.log` — raw debug-profile run of the embed startup test (not
  a valid release-mode measurement; kept as the directional data point cited in criteria.json).
- `all-features-check-partial.log`, `criteria-1-11-12-build-partial.log`,
  `embed-startup-partial.log` — truncated tails of the long-running builds that did not finish
  in budget, kept as evidence of "no errors seen before the cutoff" rather than a pass/fail.

## Environment notes for whoever resumes this

- The built HEAD release binary landed at `./target/release/ruchy` (11,034,536 bytes), not
  `/mnt/nvme-raid0/targets/ruchy`: the shell's `cargo` function sets `CARGO_TARGET_DIR`, and
  `timeout`/`nohup` exec the real binary, bypassing it. Set `CARGO_TARGET_DIR` explicitly.
- 4.2.1 was already installed at `/home/noah/.cargo/bin/ruchy` via a prior `cargo install ruchy
  --version 4.2.1`, so no fresh install was needed for criterion #6.
- The orchestrator's own baseline `cargo test --workspace --no-fail-fast` (PID 943474) ran in this
  working directory for the whole pass, holding the shared `target/` build lock intermittently; some of its prebuilt debug test
  binaries (`sovereign_nine_pillar_acceptance`, `sovereign_zero_unsafe_transpile`,
  `sovereign_migrate_4to5_e2e`, `startup_benchmark`, all under `target/debug/deps/`, mtimes
  matching this HEAD) were reused directly here instead of rebuilding, which is how criteria
  #1, #11, #12 were measured within budget despite the lock contention.
