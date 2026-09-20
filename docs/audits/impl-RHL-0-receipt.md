# RHL-0 — implementation receipt

## Identity

| | |
|---|---|
| ticket | `RHL-0` (kind=code, `pmat work`) |
| spec | `docs/specifications/ruchy-high-level-language-interface.md` (RHL-001), row RHL-0 |
| branch | `RHL-0-spec-grammar-corpus`, off `main` at `be26174c` |
| HEAD | `bcf16e26699514d1341de1d01c4846d5087f734d` |
| discover.json sha256 | `e9d8312404c381bd92bee6b65c7783d6db77243d9acd5b4f03ba4d401efc2106` |
| required check | `gate` (source: rulesets) |
| gate_cmd | `cargo test --workspace` — **`gate_cmd_fallback=true`**: not discovered from repo config, so it is a fallback, not the repo's declared gate. The check that actually gates is `ci / gate`, measured in binding B4 as `cargo test --lib` + clippy + fmt. |
| orchestrator model | opus-5, `model-gate.sh` decision=admit basis=file |

## Verdict

**PARTIAL(blocked)** — the row is implemented and every gate it owns is green.
The **merge** is blocked by `SEC-1`, which RHL-0 did not cause and cannot fix.

### The blocker, measured

`ci / security` fails with `error: 1 vulnerability found! Crate: rustls, ID:
RUSTSEC-2026-0285`. That job feeds `ci / gate`, which re-exports as the required
context `gate`, so the PR sits at `mergeStateStatus=BLOCKED`.

It is not this branch's doing:

- the `rustls 0.23.43` entry in `Cargo.lock` is **byte-identical** between `main`
  and this branch — `git diff main...HEAD -- Cargo.lock` shows no rustls line;
- `main`'s last CI run was green, and `cargo-audit` fetches the advisory database
  fresh on every run, so a newly published advisory turned an already-merged
  dependency red;
- **there is no semver-compatible fix**: `cargo update -p rustls` reports
  *"Locking 0 packages to latest compatible versions"*, and the only newer
  release is `0.24.0-dev.1`, a pre-release.

Ignoring the advisory, or moving off `rustls`, is an operator decision about the
whole repository's dependency set, not something to bury in a 312-file
pre-registration PR. Filed as `SEC-1` (priority high) with the three options.

### CI on PR #225

| Check | Result |
|---|---|
| `ci / test` — **runs the 34 RHL-0 gates** | **pass**, 2m39s |
| `ci / lint` (clippy + fmt) | pass, 3m25s |
| `ci / coverage` | pass, 3m4s |
| `ci / provenance` | pass |
| `ci / security` | **fail** — `SEC-1`, see above |
| `ci / gate` → `gate` (required) | fail, cascaded from `ci / security` |
| `Baseline Comparison` (not required) | fail — `CIBASE-1`, pre-existing on every branch |

`ci / test` passing is the load-bearing result here: it is `cargo test --lib`, so
binding B4's seam is confirmed in CI, not just locally — the gates really do run
inside the required check.

Everything in *Gaps* below is additional to this, and named because RHL-0 does
not close it and does not claim to.

## Plan and routing

| Phase | What | Route | Trigger | Acceptance `A_i` | My re-run |
|---|---|---|---|---|---|
| 1 | spec, Phase-0 bindings, RHL-1..12 DAG, TESTDEBT-1 | `route=self` | orchestration | `pmat work validate` + doc exists | PASS |
| 1.grill | plan grill, 3 agy lanes | `route=agy-plan w=1.00` | Q2 (spec/plan artifact) | delegate receipt | 3/3 do-not-implement-as-written — findings adjudicated below |
| 2 | grammar, fixture, conflict report, gate | `route=agy-goal w=1.00` → **fell back to self** | R-4 | `cargo test --lib -p ruchy rhl_pre_registration::grammar_gate` | PASS 5/5 |
| 3 | vocabularies + 14 term contracts + gate | `route=agy-goal` → `sonnet-worker` | R-4 fallback | `… ::vocab_gate` | PASS 8/8 |
| 4 | 40-task corpus + HARNESS.md + gate | `route=agy-goal` → `sonnet-worker` | R-4 fallback | `… ::corpus_gate` | PASS 8/8 |
| 5 | 12×6 break corpus + gate | `route=agy-goal` → `sonnet-worker` | R-4 fallback | `… ::breaks_gate` | PASS 11/11 |
| 6 | manifest, pv contract, amendments, receipt, PR | `route=self` | orchestration | `pv validate` + full gate | PASS 34/34 |
| 6.review | pre-PR review, 3 agy lanes | `route=agy-quorum w=1.00` | Phase-4 pre-PR | delegate receipt | 3/3 implement-with-changes — acted on below |

`route=` lines are `route.sh`'s output copied verbatim. Every `impl` phase
printed `route=agy-goal w=1.00 basis=absent note=fable-binding effort=1[U]
bucket_collision=true`.

### R-4 deviation, and why

R-4 sends single-module implementation to an agy goal lane before any Claude
subagent. It was dispatched for phase 2 and **came back BLIND (exit 4)**: agy
waits 5 seconds for background tasks at exit, the lane had started `cargo test`
in the background in a fresh worktree with a cold 6.6 GB target dir, and agy
terminated it. `--print-timeout 25m` does not cover background tasks, so
re-dispatching would repeat it. Its verdict is void and was not counted; its
worktree output was **not** cherry-picked. Phases 3–5 therefore took R-4's named
fallback, `sonnet-worker`, which shares the warm target dir in the checkout.
Phase 2 I did myself, because the grammar needed iteration against the
generator's conflict report.

## Dispatch ledger

| Agent | Mode | Turns | maxTurns hit | Resumed | Lanes / conversations |
|---|---|---|---|---|---|
| `paiml-agy-delegate` ph1.grill | quorum/grillme, w=3, writes=false | 23 | no | no | f5905d95…, bf081dd4…, adde8977… (children=3) |
| `paiml-agy-delegate` ph2.grammar | goal, w=1, writes=true | 31 | no | no | bf6f7b98… (children=1) — **BLIND, exit 4** |
| `paiml-impl-worker` ph3 vocab | sonnet | 40 | no | no | — |
| `paiml-impl-worker` ph4 corpus | sonnet | 45 | **yes** | **once** | — |
| `paiml-impl-worker` ph5 breaks | sonnet | 48 | **yes** | **once** | — |
| `paiml-agy-delegate` ph6.review | quorum/grillme, w=3, writes=false | 33 | no | no | d6da1978…, 600176c5…, 8f6c0e37… (children=3) |

Lane models were measured, never assumed: `gemini-3.1-pro-high`,
`gemini-3.8-flash-high`, `gemini-3.7-flash-high` on both quorums — three
distinct models, none in the author's `claude` family.

**I-3 (transcript-gate.sh):** `attempted=7 denied=0 stalled=0 running_peak=2
slots=3 segments=219 files=5 (agent_calls=5 resumes=2 workflow_started=0)` →
**PASS**. Peak concurrency never exceeded slots, and no hook denial occurred.

## Verification table — claimed vs my own re-run

| Claim | Claimed by | My re-run | Agree |
|---|---|---|---|
| `rhl_vocab` acceptance exit 0 | worker ph3 | 8 passed, 0 failed | yes |
| `rhl_corpus` acceptance exit 0 | worker ph4 | 8 passed, 0 failed | yes |
| `rhl_breaks` acceptance exit 0 | worker ph5 | 11 passed, 0 failed | yes |
| grammar conflict-free | me | `lalrpop … process_file → Ok` | — |
| `cargo test --lib -p ruchy rhl_pre_registration` | — | **34 passed, 0 failed, 1 ignored** | — |
| `cargo clippy --lib -p ruchy --tests -- -D warnings` | worker ph3/ph5 | exit 0 | yes |
| `cargo fmt --all -- --check` | worker ph3/ph5 | exit 0 | yes |
| `pv validate contracts/rhl-*.yaml` | me | 15/15 valid | — |

**A worker claim I did not take at face value.** Worker ph3 reported that the
brief's test naming produced a *silent vacuous pass*: `cargo test --lib -p ruchy
rhl_vocab` is a literal substring filter over the full path
`rhl_pre_registration::vocab_gate::<name>`, and `rhl_vocab` never appears in
`test_rhl_0_vocab_…`, so the filter matched nothing and the run reported "0
tests, ok". I re-ran every acceptance with the unambiguous module-path form
`rhl_pre_registration::<module>` and confirmed non-zero test counts each time.
A green run reporting zero tests is a failure, not a pass.

## Anti-vacuity — every gate proven able to FAIL

No gate here is trusted because it is green. Each was broken deliberately and
watched go red, then restored byte-identical.

| Probe | Which test went RED | Restored |
|---|---|---|
| append a line to `grammar/rhl.lalrpop` | `…committed_conflict_report_matches_the_grammar` (only that one) | byte-identical |
| add `#[precedence]` to the grammar | 3 RED: precedence check, report sha, **and LALRPOP itself rejected it** | byte-identical |
| tune `corpus/01…/intent.en` | `…manifest_0_every_recorded_hash_still_matches`, naming the file | byte-identical |
| add an unlisted artifact under `docs/rhl/corpus/` | `…manifest_0_covers_every_pre_registered_file` | removed |
| set a term's `kind` to `verb` | `…vocab_0_kind_is_closed_list` (worker ph3) | byte-identical |
| syntax error in a task's `tests.rs` | `…corpus_0_every_tests_rs_parses_as_real_rust…` (worker ph4) | byte-identical |
| flip a task to `ambiguous: true` (count 11) | `…corpus_0_has_exactly_10_ambiguous_tasks…` (worker ph4) | byte-identical |
| make a `broken.rhl` identical to its base | `…breaks_0_every_broken_program_differs_from_its_base` (worker ph5) | byte-identical |
| flip a two-candidate break to `safe_fix: true` | `…breaks_0_f7_positive_control_safe_fix_matches_class` (worker ph5) | byte-identical |

## Findings this row produced

1. **The spec's own §3.2 syntax is ambiguous.** An action opens its attribute
   block with no marker; LALRPOP reports a local ambiguity after `Body App` on a
   `NEWLINE`, and resolving it needs unbounded lookahead. **Amendment A1** adds
   `with` to §3.3's closed keyword set; the unmarked form is kept verbatim as
   F1's positive control.
2. **§3.4's Σ sentence is a category error.** Σ (ONT-001 v4.9 §3.1) classifies
   *what artifact is under contract*, not what sort of word a term is.
   **Amendment A2**. §10's `vocabulary-unbindable` does not fire.
3. **`pv` does not gate this repo** (binding B3): `sovereign-ci` runs
   `pv codegen … || true` and exits 0 when `pv` is absent. §9 hard rule 3 is
   enforced by RHL's own gate instead.
4. **An F8 fixture would have advantaged path B.** Task 01 named `runner-01`
   and registered no host. Generalised into a gate, which then caught three more
   (tasks 24, 25, 29). All four fixed.
5. **The 14 term contracts declared phantom Kani harnesses** — the practice this
   PR files `KANIDEBT-1` against. Caught by the review quorum; fixed.
6. **The packaged crate would not have built its own gates**: `Cargo.toml`'s
   `include` omitted `grammar/**`, `vocab/**`, `docs/rhl/**`. Fixed.
7. **`.claude/worktrees/` is untracked and not git-ignored**, and agy lanes leave clones
   there: six accumulated in this one session and totalled **34 GB**. Every
   `writes=false` review lane wrote scratch files into its clone, which makes
   `agy-lane.sh` KEEP it. Filed as `WORKTREE-1`.

## Tickets filed, not fixed

| id | why |
|---|---|
| `TESTDEBT-1` | The three disabled-test directories RHL-001 §1 cites were deleted in `67d2e16e`; the spec's `[C]` claim is stale. Two `.disabled` files of the same class do remain. |
| `KANIDEBT-1` | All 28 contracts in this repo that declare `kani_harnesses` name functions that exist nowhere in the tree. `pv validate` requires the block, which is the incentive. |
| `SEC-1` | `RUSTSEC-2026-0285` in `rustls 0.23.43` fails `ci / security` and so the required `gate`, on every PR. Not introduced by any branch; no semver-compatible fix exists. Priority high. |
| `CIBASE-1` | `Baseline Comparison` has failed on every branch PR and passes only on `main`: its `Checkout baseline` step runs `git checkout $base_ref -- benches/`, but the base ref is never fetched. It has never compared anything on a PR. |
| `WORKTREE-1` | `.claude/worktrees/` is not git-ignored, so a `git add -A` can sweep a whole lane clone into a commit. |
| `RHL-1`..`RHL-12` | The §8 DAG, labelled `unadmitted`: the spec admits no implementation row until RHL-0 merges. pmat's lifecycle refuses Planned→Blocked, so the label carries it, not the status. |

## Second-family review (operator ruling S5, 2026-09-20)

The three ph6 lanes were all Gemini, and an earlier lane had gone BLIND — one
reviewer, sampled twice, one sample void. A **Claude-family** lane was dispatched
on the diff with a standing instruction to examine `grammar_gate.rs`, the F1
instrument no lane had read.

It did something no lane had done: **it generated a parser from
`grammar/rhl.lalrpop` and ran the committed corpus through it.** That found
defects that no amount of reading would have.

| # | Finding | Status |
|---|---|---|
| M1 | The gate's stated reason for discarding LALRPOP's error was **false**. Measured: a conflict renders as `invalid data`, a typo as ``no definition found for `X` ``. The message *does* discriminate. | fixed — the control now asserts the error equals the conflict string, and keeps the differential as a second leg |
| M2 | The differential alone accepts a fixture whose marked line is a typo rather than an ambiguity. Two committed sentences said it could not. | fixed; the false sentences corrected in `conflict-report.txt` and `ambiguous.lalrpop` |
| M3 | **Moving `pub` off `Program` makes the whole gate green on a grammar with A1's ambiguity restored.** Four characters, plausible in an honest refactor, and F1's floor is gone. Re-measured here: `Ok`. | fixed — `…program_is_the_only_entry_point` |
| M4 | The gate placed **no floor on what the grammar describes**. A grammar keeping the header and the whole `match` block, whose only production is `Phrase = WORD+`, passed every check while parsing no RHL at all. | fixed — `…productions_still_cover_the_keyword_set` |
| M5 | `contains("#[precedence")` missed `#[ precedence`. | hardened to an attribute scan covering `precedence` and `assoc`. Note: re-measured, LALRPOP itself **rejects** `#[ precedence` as an unrecognized attribute, so this was never an actual silencing path — the lane's `Ok` did not reproduce |
| M6 | Three lines of §3.2 do not parse (`to`, `per`, `are`), and the obvious repair reintroduces an LR conflict. | **not fixed** — recorded as spec **open question O1**, because it is a design question and guessing at it is what §10 forbids |
| M7 | **2 of 12 "valid" break programs did not parse**, and 10 planted breaks built on them failed for that reason instead of their planted defect. `tickets filed in` contains the reserved word `in`. | fixed — `Cmp` now admits `App "in" App`; re-measured 72/72 |
| M8 | `vocab` declares the unit `GB`; every corpus program wrote `gb`; and `GB` **lexed as nothing at all**, so §3.1 principle 4's own example was an `InvalidToken`. | fixed — `UNIT` terminal added, corpus reconciled to `GB` |
| M10 | Amendment A1's claim to be "the only" repair was false; a keyword-free repair exists. | corrected in the spec |
| M11/M12 | Nothing connects grammar↔corpus↔vocabulary, and the manifest excludes the gate modules. | named in *Gaps*; M4's floor is a partial answer |

Both new gates were proven able to fail, restoring the grammar byte-identical:
the `pub`-relocation probe turned `…program_is_the_only_entry_point` **and**
`…productions_still_cover_the_keyword_set` RED (previously the same grammar was
fully green); the gutted-grammar probe turned the coverage test RED.

Gate count after the review: **36 passed, 0 failed, 1 ignored.**

**The honest summary of M3/M4:** before this review, the F1 instrument could be
switched off by a four-character edit, and would certify the empty language. It
was not lying about today's bytes — it never was — but it was weaker than the row
claimed, and the row's own documents asserted guarantees it did not provide.
Those sentences are now corrected rather than defended.

## Gaps — what this row does NOT close

| Gap | What would close it |
|---|---|
| **F1's second half is unmeasured.** The grammar is proven conflict-free, but "every corpus program has exactly 1 parse" needs a parser. | RHL-1 |
| **No lane reviewed `grammar_gate.rs`.** The ph6 delegate says so itself: lanes named evasions for four gates and none for the positive-control gate. | a targeted review, or RHL-1's first use of it |
| **Grammar↔corpus↔vocabulary are still unconnected.** No gate parses a committed program; `…productions_still_cover_the_keyword_set` is a floor, not a proof. The one grammar-shaped check on corpus bytes is a string heuristic, which CLAUDE.md rule 2 forbids. | RHL-1, once a parser exists |
| **§3.2 open question O1** — three lines still do not parse, and the obvious repair reintroduces a conflict. | RHL-1 must amend §3.2 or redesign `Args`, with evidence |
| **Only 4 of 40 corpus tasks were audited for arm fairness** — task 01 by a lane, tasks 24/25/29 by the gate that fix produced. The other 36 pass the mechanical gate but no one has read them against their intents. | RHL-8 must audit all 40 before running the A/B |
| **`pv_lane` = enforced locally only.** `pv validate` passes on all 15 contracts here, but binding B3 shows CI does not run it, so it is not a gate. | a repo-level decision to arm `pv` in `ci / gate` |
| `cargo test --workspace` (the fallback `gate_cmd`) was not run to completion — it exceeds the tool's command timeout on this tree. The check that gates, `cargo test --lib` + clippy + fmt, was run in full. | CI on the PR |

## Estimates

| | |
|---|---|
| K̂ | 80 (spec RHL-001 §11, `[A]` operator estimate) |
| K | 100, andon at 80 |
| basis | `first-run[U]` |

`estimate.sh ruchy` exits 2 with an ENV finding: *"23 measured rows for
repo=ruchy and none enters a total — the writer and the reader disagree."* Every
`repo:"ruchy"` row in `docs/audits/impl-estimates.jsonl` carries
`est:null, actual:null` and a prose `phase`, so none pools. Recorded, not worked
around.

## Machine-readable block

orch_model: opus-5   orch_class: opus   orch_decision: admit
fable_binding: true   quota_age_h: absent   quota_mark: ?   k_measured_at_set: 118

`quota.sh binding` prints `fable_binding=true stale=false age_h=absent
account_mismatch=true`; `quota.json` is absent, so `route.sh` reported
`basis=absent` on every phase and the status blocks carried `q=?`.

**k gap, recorded as the skill requires.** `k_measured` is 118 distinct
assistant message ids in this session's transcript; my own orchestrator-turn
count is ~72. The two count different things — `statusline.sh` counts every
assistant message, including the intra-turn tool-call messages, while K̂=80 from
RHL-001 §11 is budgeted in orchestrator turns. The gap is a unit mismatch, not a
budget overrun; `andon at 80` was measured against the turn count and was not
reached.

routes:
  ph1  class=orchestration  route=self        w=100.00  basis=absent
  ph1.grill  class=plan     route=agy-plan    w=1.00    basis=absent
  ph2  class=impl           route=agy-goal    w=1.00    basis=absent
  ph3  class=impl           route=agy-goal    w=1.00    basis=absent
  ph4  class=impl           route=agy-goal    w=1.00    basis=absent
  ph5  class=impl           route=agy-goal    w=1.00    basis=absent
  ph6  class=orchestration  route=self        w=100.00  basis=absent
  ph6.review  class=review  route=agy-quorum  w=1.00    basis=absent

Phases 2–5 printed `route=agy-goal`; phase 2's lane returned BLIND (exit 4) and
phases 3–5 took R-4's named fallback `sonnet-worker`. Both are recorded above as
printed and explained in *R-4 deviation*.

verification:
  cmd=cargo test --lib -p ruchy rhl_pre_registration  claimed_exit=0  rerun_exit=0  log_path=docs/audits/rhl-0-logs/gate-rhl-pre-registration.log  sha256=7c597254cb8d4c0915856536c92cd561015f0c929d5d9b1eeda33be6b389bbcc
  cmd=cargo clippy --lib -p ruchy --tests -- -D warnings  claimed_exit=0  rerun_exit=0  log_path=docs/audits/rhl-0-logs/gate-clippy.log  sha256=0bac832435ba6fc37cb60260ee97f76708e1c0b6e0f9644664bd43cb574f3c8b
  cmd=cargo fmt --all -- --check  claimed_exit=0  rerun_exit=0  log_path=docs/audits/rhl-0-logs/gate-fmt.log  sha256=1d7695848349005237a697294e56759a89a4491d4d0be8698a326efbd0df4264
  cmd=pv validate contracts/rhl-*.yaml  claimed_exit=0  rerun_exit=0  log_path=docs/audits/rhl-0-logs/gate-pv.log  sha256=d8d23f09d51fef6d96f05546394a1f7aef9bb23f2c346eff7105a0eaca410d52
