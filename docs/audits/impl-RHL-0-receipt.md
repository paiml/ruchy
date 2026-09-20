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

**DONE** — with three things named in *Gaps* below that RHL-0 does not close and does not claim to.

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
| `WORKTREE-1` | `.claude/worktrees/` is not git-ignored, so a `git add -A` can sweep a whole lane clone into a commit. |
| `RHL-1`..`RHL-12` | The §8 DAG, labelled `unadmitted`: the spec admits no implementation row until RHL-0 merges. pmat's lifecycle refuses Planned→Blocked, so the label carries it, not the status. |

## Gaps — what this row does NOT close

| Gap | What would close it |
|---|---|
| **F1's second half is unmeasured.** The grammar is proven conflict-free, but "every corpus program has exactly 1 parse" needs a parser. | RHL-1 |
| **No lane reviewed `grammar_gate.rs`.** The ph6 delegate says so itself: lanes named evasions for four gates and none for the positive-control gate. | a targeted review, or RHL-1's first use of it |
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
