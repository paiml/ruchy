# RHL-1 — implementation receipt

## Identity

| | |
|---|---|
| ticket | `RHL-1` (kind=code, `pmat work`): parser, intent tree, `fmt`, diagnostics JSON, `check` |
| spec | `docs/specifications/ruchy-high-level-language-interface.md` (RHL-001) §3–§5, §7 F1/F2/F4/F9; Amendment A3 added |
| plan | `docs/rhl/rhl-1-plan.md` (D1–D8, two quorum rounds, operator rulings) |
| branch | `RHL-1-parser-fmt-check`, off `main` at `e41774b4` |
| HEAD described | `82c24b999b5c7bfc36fbf86a30851e8438618c86` (the last code commit; this receipt is committed after it) |
| discover.json sha256 | `f2dfa224bb54d16daa5f4fc9a5e66b3f28172f932c6eed8c3bd2aa6c31681c42` |
| required check | `gate` (source: rulesets) |
| gate_cmd | `cargo test --workspace`, but **`gate_cmd_fallback=true`**: it is a fallback, not the repo's declared gate. What `ci / gate` actually runs is measured in binding B4: `cargo test --lib` plus clippy and fmt. The verification below uses that. |
| status-line join `[U]` | Not measured in this run: statusLine `session_id` = hook `session_id`, `tasks[].id` = hook `agent_id`, and `transcript_path` on the subagentStatusLine stdin. |

## Verdict

**DONE (pending CI and merge).** Every gate this row owns is green locally. On
the way, the row hit §10's `quorum-split` STOP: two plan-grill rounds each split
FAIL/FAIL/PASS. The STOP was resolved by three operator rulings, taken through
the session's question prompt, and quoted verbatim below. The pre-PR review did
not reach unanimity either (round 1: 2 FAIL / 1 PASS; round 2: 1 FAIL / 2 PASS).
Every blocking finding was reproduced and fixed, except one, whose disposition is
recorded under *Adjudication*.

## Operator rulings (verbatim option labels, 2026-09-21)

| Question | Selected |
|---|---|
| An entity instance with no source of truth in the repo | **"Pass, list as unverified (Recommended)"** |
| JSON output shape vs §3.5 "emits the list" | **"Object, amend §3.5 (Recommended)"** |
| wrong-unit/04 and /09 expect T001 but the vocabulary makes them T002 | **"Named exception + RHL-16 (Recommended)"** |

## Plan and routing

```
orch_model: opus-5   orch_class: opus   orch_decision: admit
fable_binding: true   quota_age_h: absent   quota_mark: ?   k_measured_at_set: 0
```

`quota.sh binding` printed `fable_binding=true stale=false age_h=absent
account_mismatch=true`. `k_measured_at_set` is 0 because `goal.sh set` ran
before any subagent existed. The ticket was not freshly opened: `RHL-1` was
already on the roadmap, admitted by RHL-0's merge.

routes:
  ph1  class=plan           route=agy-plan    w=1.00  basis=absent   (plan grill round 1, delegate, Q2)
  ph1.parser  class=impl    route=agy-goal    w=1.00  basis=absent   -> opus-worker (see note)
  ph2  class=plan           route=agy-quorum  w=1.00  basis=absent   (plan grill round 2, delegate)
  ph2.diag  class=impl      route=self        w=100.00 basis=absent  (diagnostics model; orchestrator)
  ph3  class=impl           route=agy-goal    w=1.00  basis=absent   -> opus-worker (fmt)
  ph4  class=impl           route=agy-goal    w=1.00  basis=absent   -> opus-worker (check)
  ph5  class=impl           route=self        w=100.00 basis=absent  (CLI wiring; orchestrator)
  ph6  class=review         route=agy-quorum  w=1.00  basis=absent   (pre-PR review, 2 rounds)
  ph6.orch  class=orchestration route=self    w=100.00 basis=absent  (amendments, receipt, PR)

Every `impl` phase printed `route=agy-goal` (R-4). R-4 was overridden to Claude
workers, and the override is recorded rather than hidden. The reasons:

- **The delegate slot was busy.** Each time a worker ran, slot A held the plan
  grill.
- **The one-writer rule.** Adding an agy writing lane beside a read-only grill
  would have needed a second delegate or a mixed brief.
- **Precedent.** RHL-0's `agy-goal` lane went BLIND (exit 4) and fell back the
  same way.

## Dispatch ledger

| # | description | type / model | agent id | tools | outcome |
|---|---|---|---|---|---|
| 1 | `RHL-1/ph1.delegate` plan grill r1 | paiml-agy-delegate / opus | `aedbb68e16e78ee1f` | 13 | agy lanes `f6edddd9…`, `d5a0efd2…`, `af05e601…` (gemini-3.1-pro / 3.8-flash / 3.7-flash): FAIL, FAIL, PASS |
| 2 | `RHL-1/ph1.parser` worker B | paiml-impl-worker / opus | `a32325a47c3a4b8ee` | 41 | parser + tree + skeleton gate; `partial=false` |
| 3 | `RHL-1/ph2.delegate` plan grill r2 | paiml-agy-delegate / opus | `a2d3b2e198da382e6` | 14 | lanes `19dffe3e…`, `082c7cd6…`, `52c7f1a5…`: FAIL, FAIL, PASS → §10 STOP → operator |
| 4 | `RHL-1/ph3.fmt` worker B | paiml-impl-worker / opus | `a7c2386f1ca43d9ff` | 25 | fmt + F2; own tests 17/17; `gate.ok=false` only from worker C's in-flight file |
| 5 | `RHL-1/ph4.check` worker C | paiml-impl-worker / opus | `a6e9347c8ef4cdadf` | 40 | checker; `partial=true`: 70/72, the two T001/T002 conflicts escalated, not bent |
| 6 | `RHL-1/ph6.delegate` review r1 | paiml-agy-delegate / opus | `acb63dcb1479b2995` | 10 | lanes `4ad377e5…`, `8727d5fb…`, `29bca839…`: FAIL, FAIL, PASS |
| 7 | `RHL-1/ph6.review2` review r2 | paiml-agy-delegate / opus | `a25907e9d3fc0ec39` | 13 | lanes `8a68c7ea…`, `3f566d92…`, `c42875b6…`: FAIL, PASS, PASS |

- **Resumes and caps:** none resumed, none hit `maxTurns`.
- **Lane models:** every lane was gemini-family, measured from its log; the
  author was claude.
- **Kept clones:** writes=false lanes that wrote scratch into their own clones
  kept them. Each delegate removed the clone after confirming its pid was dead.
  No lane touched the shared checkout, and there were no exit 3s and no BLIND
  lanes.

## Slots and I-3

```
PASS transcript-gate: attempted=6 denied=0 stalled=0 running_peak=3 slots=3 segments=150 files=6 (agent_calls=6 resumes=0 workflow_started=0; denied from hook log)
```

The gate ran before dispatch 7, which is why it counts 6. Dispatches 4, 5 and 3
ran in one message: the delegate and two workers with disjoint `scope_paths`,
three slots. The delegate declared `--concurrent-scope src/rhl Cargo.toml
Cargo.lock` for them, and its lanes echoed `agy-lane: --concurrent-scope src/rhl
Cargo.toml Cargo.lock — those paths are NOT asserted against this lane
(PMAT-061); everything else is`.

## Verification (claimed vs rerun)

verification:
  cmd=cargo test --lib  claimed_exit=-  rerun_exit=0  log_path=docs/audits/rhl-1-logs/gate-cargo-test-lib.log  sha256=1779d0dd7919059e1ab59a6cfa146a33c2ad3be1388efeed4507c57fdf670e98
  cmd=cargo test --lib rhl  claimed_exit=0  rerun_exit=0  log_path=docs/audits/rhl-1-logs/gate-rhl-lib-tests.log  sha256=3baf3258ae83787117d2277b9c72c65370381aa01b6e27e5c5912857323edea0
  cmd=cargo clippy --lib --tests --all-features -- -D warnings  claimed_exit=0  rerun_exit=0  log_path=docs/audits/rhl-1-logs/gate-clippy.log  sha256=37af26fb8943cd38951fddacb48a7470489ed4cdc5938d40ad6622693e2c8c29
  cmd=cargo fmt --all -- --check  claimed_exit=0  rerun_exit=0  log_path=docs/audits/rhl-1-logs/gate-fmt.log  sha256=19eaf43821a7660ec323a87c8457bf74823beb296c39f5e01aa8a683aa50f061
  cmd=cargo check --lib --target wasm32-unknown-unknown --no-default-features  claimed_exit=0  rerun_exit=0  log_path=docs/audits/rhl-1-logs/gate-wasm32-check.log  sha256=41d0e62b01769ece33348591bdd9f065a6223d59714141c4660f74e7072266be
  cmd=cargo deny check advisories  claimed_exit=0  rerun_exit=0  log_path=docs/audits/rhl-1-logs/gate-deny-advisories.log  sha256=911c18c0eb56e83ee781e6709c2a01b999137b6f84e35b007ea055a97e890dab
  cmd=cargo test --test rhl_1_cli_exit_codes  claimed_exit=-  rerun_exit=0  log_path=docs/audits/rhl-1-logs/gate-cli-exit-codes.log  sha256=453eee31a1d3dfec16e6f5f8a1a43b9ea853e17962b4c2209f7704a23a1d40bf
  cmd=hand mutations M1-M4 (cargo test --lib rhl per mutation)  claimed_exit=-  rerun_exit=101x4  log_path=docs/audits/rhl-1-logs/hand-mutations.log  sha256=bf99ad7a33582fc790b59ce449c67fecd25f46da767ddefa26cd21452b26f673

Where a worker's claim and my rerun disagreed:

- **P4 check worker.** Its acceptance command pipes through `tail`, so the
  shell exit was 0; the worker itself reported cargo's 101, and my rerun agreed
  (70/72). After the operator ruling it is 72 of 72 accounted for: 70 gated,
  plus the exact two-entry exception list.
- **P3 fmt worker.** Its `gate.ok=false` came only from worker C's in-flight
  test. My rerun after both finished: 117/117.

What the gates establish:

- **F1 second half.** 12/12 valid programs, 60/60 non-`missing end` breaks and
  the §3.2 example parse. The 12 `missing end` breaks fail with `RHL-P001`. The
  production grammar's skeleton is gated identical to the pre-registered one,
  with two positive controls.
- **F2.** The 12 valid programs and §3.2 format byte-exact. On all 73 parseable
  programs and in proptest, `fmt` is idempotent and tree-preserving. The mangled
  positive control and the one-token negative control both hold.
- **Checker.** 70/72 planted breaks yield their pre-registered code on the
  mutated line. For V002 there is one safe fix, and it restores the base line.
  For V003 the candidates are exactly the pre-registered ones, with no safe fix.
  The two remaining breaks are the named exception list, exact in both
  directions. A clean program checks with exit 0, so the checker does not refuse
  everything.
- **Mutation.** 4/4 hand mutations were caught (threshold rule, the type demotion
  of safe fixes, E001, and the fmt block rule). cargo-mutants did not complete
  (see *Jidoka*).

## Adjudication of review findings

| Finding (lanes) | Reproduced? | Disposition |
|---|---|---|
| A V002 fix was marked safe without a type check (r1: 1, 2) | yes, `runs on hou gx10`, `10 hou` | fixed (`63be25e1`): apply, re-check, demote on RHL-T on the edited line |
| CRLF line endings failed with P003 (r1: 1) | yes | fixed: a same-length space for a line-ending `\r` outside strings; frozen grammar untouched |
| A term with no `contract:` gave V004, not C001 (r1: 2) | yes | fixed: `#[serde(default)]`; C001 names the term |
| A file of only `use` lines passes (r1: 2) | yes | **kept**: the spec requires no unit and has no code for one; r2 3/3 agree it is defensible |
| A type error on a **different** line leaves the fix safe (r2: 1 FAIL, 2 concedes and passes) | lane 1's own input no (already demoted); the variant `let t be 10 hou` yes | **kept at the edit site.** The global reading also demotes the pre-registered safe fixes of typo-term 03/04/05/08/09/10, whose bases are mistyped (RHL-16). §3.5's `fix --safe` re-checks the whole program. The question is recorded on **RHL-2**, which builds `fix --safe`. |
| (found while measuring r2) a `let` name plus extra words gave a V002 with a distance-0, no-op safe fix | yes | fixed (`82c24b99`): RHL-T002, no fix |

## Jidoka

- **ph1 complexity refused a commit.** The pre-commit gate refused cognitive
  13 > 10 in the worker's `skeleton()`. Split, then committed.
- **cargo-mutants was interrupted three times** during a build phase:
  - two copy-mode runs, both at 109 s;
  - one `--in-place` shard at 260 s.

  One mutant build of this crate measures about 150 s, so a single file
  (29 mutants) is over an hour. The signal's source was not found (no `pkill`
  in the hooks or scripts).

  Replaced by four hand mutations, all RED. **Not root-caused; not filed.**
- **Stale build after a hand mutation.** Restoring a mutated file with `mv`
  kept its old mtime, so cargo reused the mutated build, and the next full run
  showed a phantom `fmt` failure. Touching the file fixed it (133/133). The
  script now touches restored files. The mutation results stand, because each
  mutation's build recompiled the crate from the current files.
- **The pre-registered corpus is invalid under its vocabulary.** Filed as
  **RHL-16**. Spec A3 and plan M3–M5 carry the details; M5 was corrected from 4
  to 6.

## Estimates

| K̂ | K | andon | k (measured) | basis |
|---|---|---|---|---|
| 150 | 190 | 152 | 138 | spec RHL-001 §8 `[A]`; goal basis `docs/audits/impl-estimates.jsonl:L26` (RHL-0) |

`k` is `statusline.sh`'s measurement: distinct assistant message ids in the
transcript, sidechains excluded. It was never counted by hand.

## Gaps

- **`ci / gate` on the PR has not run yet.** The DoD requires it to be merged
  green.
- **cargo-mutants did not complete.** 4 hand mutations stand in for it. There
  is no ≥75% kill-ratio figure.
- **`pv` lane: NotRun.** RHL-1 emits no contract. `expect` → `pv` is RHL-5, and
  binding B3 measured that `pv` runs in no required check here.
- **The valid corpus does not check clean.** RHL-16.
- **Local vs global safe-fix type preservation.** RHL-2.
- **The status-line join table was not measured.** Marked `[U]` above.
- **The estimate row is not in `docs/audits/impl-estimates.jsonl`.** The
  gated writer `pmat work estimate record --repo ruchy` refused with *"repo
  "ruchy" would split the ledger, which is keyed infra,ruchy"*. The ledger
  already carries two `infra` rows (PMAT-134 and PMAT-138), so no new row can
  be written until those move to infra's own ledger. The row wanted is
  `{repo: ruchy, ticket: RHL-1, phase: all, est: 150, actual: 138, unit: turn}`.
