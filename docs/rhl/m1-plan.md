# M1 — measuring F7 and F8 before ruchy 5.0.0 (RHL-7, RHL-8)

Ticket RHLGA-1, phase M1 of `docs/rhl/rhlga-1-plan.md`; epic #238. Ruling Q2 (plan §5):
RHL-7 and RHL-8 are run **before** the release and published **whatever they say**; RHL
ships labelled experimental either way; lane 1's residual ("if F8 fails, RHL must not
ship in 5.0.0") is read against the F8 result before R1.

Provenance tags as in RHL-001: `[V]` measured · `[C]` cited · `[A]` directive/estimate ·
`[U]` unknown.

## §1 Ground truth at `765d0cfb` `[V]`

| Fact | Evidence |
|---|---|
| F7: "an agent given only the diagnostics JSON and the MCP tools repairs ≥ X % within ≤ k rounds; `ruchy fix --safe` alone repairs every single-candidate break; a two-candidate break must not be auto-fixed". X and k are `[U]`, set from the first run. | RHL-001 §7 |
| `ruchy fix --safe` already repairs 12/12 v2 typo-term breaks to a clean check and leaves 12/12 two-candidate breaks byte-unchanged | RHL-2 acceptance, #235 |
| The v2 planted-break corpus has 72 breaks in 6 classes; `ruchy mcp` serves `rhl_check`, `rhl_fix`, `rhl_vocabulary`, `rhl_grammar`, … | #235, #243 |
| F8: the pre-registered A/B — path A English → model → Rust, path B English → model → RHL → compiler; scored by the same pre-written tests; claim holds only if B > A; ambiguous tasks refused 100 % on path B | RHL-001 §7, `docs/rhl/corpus/HARNESS.md` |
| The 40 tasks target `run(&mut TaskCtx) -> Result<Report, TaskError>`; `TaskCtx`, `Report`, `TaskError` are declared in HARNESS.md and implemented nowhere yet | HARNESS.md "Nothing here compiles yet" |
| RHL-8's row: all 40 tasks must be audited for arm fairness before the A/B; a fix makes the corpus v1 with a public diff and manifest re-hash, never an in-place edit; a seeded shuffle, seed recorded | roadmap RHL-8 notes |
| RHL v0 lowering refuses `for each`, `give back`, `stop with` (RHL-L001), and the v2 vocabularies have no terms for hosts, repos, CPU, PR counts or the clock — the world the tasks read | `docs/rhl/rhl-4.md`, `vocab/*-v2.yaml` |
| Grammar-constrained decoding (the spec's path B) does not exist: it is RHL-10, blocked on aprender | roadmap RHL-10 |

## §2 Decisions for the quorum

**D1 — The model.** Both arms use one model, pinned by id. Proposal: a non-Claude model through
agy (the author of RHL is Claude; a Claude model may carry the author's framing of the
language). The id, the date, and every prompt are recorded in the receipt.

**D2 — No grammar-constrained decoding.** RHL-10 does not exist, so path B is unconstrained
generation of RHL text followed by `ruchy check`. This handicaps B relative to the spec's
design (a constrained decoder cannot emit a parse error). Recorded as a deviation in the
report; the report states that F8 as measured is a **lower bound** for path B.

**D3 — Rounds.** Primary measurement: **single shot**, both arms (A: rustc errors fail the
task; B: `ruchy check`/lowering errors fail the task). Secondary, pre-registered now: **up to
3 repair rounds** in both arms, each arm seeing only its own compiler's output (rustc for A;
the §3.5 diagnostics JSON for B). F8's verdict is the primary; the secondary is reported
beside it.

**D4 — Refusal channel.** Both arms receive the HARNESS.md sentence verbatim. Path A refuses
by writing a `run` that returns `Err(TaskError::Ambiguous { .. })`. Path B refuses by
answering with a single line `AMBIGUOUS: <what is underdetermined>` instead of a program; the
harness turns that into a `run` returning `Err(TaskError::Ambiguous { reason })`. Both arms
may refuse; neither is told which tasks are ambiguous.

**D5 — Path B's machinery (built before the run, frozen before the run).**
- A **harness form** of the lowering: `observe()` reads `TaskCtx` through its accessors; actions
  become `Report.tickets`; `give back` becomes `Report.values`; the generated module exposes
  `run(&mut TaskCtx) -> Result<Report, TaskError>`.
- **Language completeness for the keywords §3.3 already lists**: lowering of `for each` over a
  finite collection, `give back`, and `stop with` (→ `TaskError::Refused`). These are gaps in
  RHL v0, not new language.
- **A vocabulary `fleet v3` / `tickets v3`** covering exactly the `TaskCtx` world (disk free by
  path, hosts with cpu/online, repos with open PRs/default branch, now) — one term per accessor,
  no task-specific terms. It is written from HARNESS.md alone, before any task is attempted.
- The `TaskCtx`/`Report`/`TaskError` implementation both arms link against, written from
  HARNESS.md alone.

**D6 — Corpus fairness audit (before the run).** A 3-lane review of all 40 tasks (intent vs
tests vs the harness contract). Any task a lane finds unfair to either arm, or whose tests do
not follow from its English, is fixed in **corpus v1** (new directory, public diff, manifest
re-hash); v0 stays byte-identical. The audit report is committed before the first model call.

**D7 — F7 protocol.** The agent (same model as D1) receives, per break: the broken program, the
`ruchy check --format json` report, and access to the MCP tools (`rhl_check`, `rhl_fix`,
`rhl_vocabulary`, `rhl_grammar`, `rhl_explain`). It may not see the base program. A round is one
edited program; after each round `ruchy check` is run; success = the program checks clean **and**
its tree equals the base program's tree (a repair that changes the meaning is not a repair).
Rounds ≤ 5 `[A]`; X is reported, not set in advance (§7: "set from the first measured run").
Controls: `ruchy fix --safe` alone on all 72 (expected: every single-candidate break repaired,
no two-candidate break changed).

**D8 — Publication.** `docs/rhl/reports/f7.json`, `f8.json` and a one-page `docs/rhl/reports/F8.md`,
committed whatever they say, with: model id, seed, prompts, per-task outcomes both arms, the
deviation list (D2), and the verdict (`B > A`, `B ≤ A`, or `Unknown{reason}`).

## §3 Phases

| # | Phase | Acceptance |
|---|---|---|
| M1a | D5 machinery: harness form, `for each`/`give back`/`stop with` lowering, vocabulary v3, `TaskCtx` crate | every construct has an rhl-to-rustc test; a hand-written RHL solution to task 01 passes task 01's tests through the harness |
| M1b | D6 audit; corpus v1 if needed | audit report committed; manifest verifies |
| M1c | F7 run | `test -s docs/rhl/reports/f7.json` with per-break outcomes and the fix --safe control |
| M1d | F8 run (seeded) | `test -s docs/rhl/reports/f8.json` with 40 × 2 outcomes, seed, model id |

## §4 STOP conditions
A quorum split on D1–D8 after one amendment round → operator. The harness cannot give both
arms the same `TaskCtx` → operator. Any measurement tuned after seeing results → void, redo
with a new seed and say so.

## §5 Rulings — grill quorum, 2026-09-24 `[V]`

Three lanes (gemini-3.1-pro-high, gemini-3.8-flash-high, gemini-3.7-flash-high; author Claude),
all exit 0 with tree witnesses: **3/3 PASS with amendments**. Adopted:

| D | Ruling |
|---|---|
| D1 | ACCEPT, plus: pin the model id and **temperature 0**, and log every prompt and full response. |
| D2 | ACCEPT (3/3): F8 runs before RHL-10, as spec §8 orders; path B without constrained decoding is a lower bound. Plus: if B ≤ A **and** path B's failures are parse errors, the verdict is `Unknown{"unconstrained decoding"}`, not a falsification. |
| D3 | ACCEPT (3/3). |
| D4 | AMEND (3/3): the refusal channel must be symmetric. Both arms get the HARNESS.md sentence verbatim and refuse the same way, by answering `Err(TaskError::Ambiguous { reason: "…" })`; for path B the harness turns that answer into a `run` returning it. No `AMBIGUOUS:` line. |
| D5 | AMEND (2/3): v3 terms map **1:1 onto primitive** `TaskCtx` accessors and `Report` fields — no compound terms. M1a's acceptance uses a **synthetic fixture outside `docs/rhl/corpus/`**, never a hand-written solution to a corpus task (HARNESS.md: "Do not write a solution here"). |
| D6 | AMEND (3/3): the fairness audit spans **model families** (spec: one agy, one Claude, one apr; where a family is unavailable the report says which and why), checks every `tests.rs` against the HARNESS.md signatures, and is committed before the first model call. Unfair tasks go to corpus v1 (public diff, manifest re-hash); v0 stays byte-identical. |
| D7 | ACCEPT, plus: success = clean check **and** the program's examples pass **and** its tree equals the base tree under canonical declaration order. |
| D8 | ACCEPT (3/3). |

**Corpus defects already found (verified by the orchestrator, for the D6 audit):**
task 01 `tests.rs:5` calls `with_host("runner-01")` while HARNESS.md:51 declares
`with_host(HostRecord)`, so it cannot compile against the contract; task 12 `tests.rs:13`
asserts a `report.values` key `total_free_bytes` that its `intent.en` never names, so neither
arm can know it (lanes report the same gap in 13 and 14).
