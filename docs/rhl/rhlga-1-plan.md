# RHLGA-1 — the admissible RHL-001 rows, then ruchy 5.0.0

Ticket: RHLGA-1 · branch `RHLGA-ruchy-5-0-0` off `main` `270a065c` · spec
`docs/specifications/ruchy-high-level-language-interface.md` (RHL-001, amended A1–A3,
O1-R) · process `paiml-implement`, unattended.

**Operator directive (verbatim, 2026-09-23):** "implment paiml-implement:
'/home/noah/Downloads/ruchy-high-level-language-interface.md' autonomously (quorum for
questions), goal is this is the "non-beta" version to crates.io and github."

So: the open questions below are ruled by a quorum, not escalated. The Downloads copy is
the pre-amendment v0.1 of the in-repo spec; the in-repo spec governs.

Provenance tags as in RHL-001: `[V]` measured · `[C]` cited · `[A]` directive/estimate ·
`[U]` unknown.

---

## §1 Ground truth at HEAD `270a065c` `[V]`

| Fact | Evidence |
|---|---|
| Version is `5.0.0-beta.2`; last tags `v5.0.0-beta.2`, `v4.2.1` | `Cargo.toml:170`, `git tag` |
| RHL-0 (pre-registration) and RHL-1 (parser, tree, `fmt`, JSON diagnostics, `check`) merged | `1c2caf5d`, `d56cf407` |
| RHL-2..RHL-12 and RHL-16 are `planned` | `docs/roadmaps/roadmap.yaml` |
| RHL-1's tree is `serde`-serializable, spans skipped — the YAML surface (RHL-3) is a serializer over it | `src/rhl/tree.rs` |
| Lowering seam: RHL emits ruchy **source text** into the existing parser and transpiler | `docs/rhl/phase0-bindings.md` B1 |
| `ruchy mcp` exists (`src/mcp.rs`, feature `mcp`) | tree |
| 5.0 GA go/no-go: all 13 criteria of `ruchy-5.0-sovereign-platform.md` §10 | spec §10 |
| GA gather at this HEAD: 4 MET (#1, #6, #11, #12), 0 NOT_MET, 9 UNMEASURED | `docs/specifications/evidence/2026-09-23-ga-gather/` |
| Every `vocab/*.yaml` term has `lowers_to: "[U]"` | `vocab/fleet-v1.yaml`, `vocab/tickets-v1.yaml` |
| The 12 valid break-corpus programs do not check clean under v1 vocabulary (RHL-16) | roadmap RHL-16 |

The GA gather's "concurrent unrelated session" was this orchestrator's own background
`cargo test --workspace`; it is not a second session. Recorded, and corrected in the
evidence README before commit.

## §1b Measured after plan v1 `[V]`

| Fact | Evidence |
|---|---|
| `cargo test --workspace --no-fail-fast` at `main`: **27,573 pass, 285 fail (27 binaries), 2,773 ignored**, exit 101. Largest: `probar_wasm_tests` 194, `lang_comp_suite` 25, `cli_interactive_validation` 13, `http_server_cli` 11. CI's `gate` runs `cargo test --lib` + clippy + fmt only, so none of these is gated. This is GA criterion #3's measurement. | `evidence/2026-09-23-ga-gather/criterion-3-workspace-tests.txt` |
| `tests/cli_contract_doc.rs` ran `ruchy doc` with the repository as cwd, so every workspace test run rewrote the tracked `docs/simple.html`, `docs/test.html`, `docs/test.md`. That write voided two plan-v1 quorum lanes (isolation exit 3). Fixed on this branch: the tests run in their temp dir, a regression test pins it, the three generated files are removed. | this branch |
| Q4 probe: a hand-written `decide(facts: Facts) -> Vec<RhlAction>` using a `struct`, an `enum` with a struct variant, `let mut plan: Vec<…> = Vec::new()` and `plan.push(…)` (no closures, no map) transpiles with `ruchy transpile`, compiles with `rustc --edition 2021`, and prints `1 0` for the two §3.2 examples. | `evidence/2026-09-23-ga-gather/q4-lowering-probe/` |
| Defect found by the same probe, present in 4.2.1 and HEAD: `ruchy run` does not call `main()` when the file defines any other function — `fun d(a: i64) -> i64 { a }` + `fun main() { println("y") }` prints nothing, exit 0. `ruchy transpile`+`rustc` runs it. Filed as RUNMAIN-1 and fixed in this program (it is on the RHL path: `ruchy run` of lowered RHL). | `q4-lowering-probe/m5.ruchy` |

## §2 Questions for the quorum — round 2 (plan v2)

Round 1 (3 lanes, gemini family): lane 3 PASS (counted); lanes 1 and 2 FAIL but void
(isolation exit 3, caused by the test defect above, not by the lanes). Their findings are
adopted here anyway. The operator's directive is quoted verbatim at the top of this file;
this round rules only what it does not already decide.

**Q1 — Name and extension.** Round 1: lane 1 "(a) keep `RHL`/`.rhl`, record the ruling"; lane
3 no objection. **Proposed ruling: (a).** Diagnostic codes (`RHL-P001` …) are stable forever
(RHL-001 §9.6) and this release publishes them; renaming afterwards is the costly direction.

**Q2 — Rows in 5.0.0.** Round 1 lane 1: F8 is the headline falsifier ("F8 can fail. If it
does, the honest conclusion is that RHL is not worth building"), so shipping RHL without it
ships an unmeasured claim. **Proposed ruling:** IN, in order: RHL-16, RHL-2, RHL-3, RHL-4,
RHL-5, RHL-6, RHL-9, then **RHL-7 and RHL-8 as measurements run before the release**, their
reports published in the release whatever they say. RHL ships in 5.0.0 labelled
**experimental** in `--help`, README and CHANGELOG (the language, codes and v1/v2 vocabularies
are frozen; the verbs may change in 5.x). If F8 FAILS, the release says so and RHL stays
experimental; ruchy 5.0.0 (the language) does not hinge on RHL. OUT: RHL-10, RHL-11 (blocked
outside this repo), RHL-12 (spec §8: one unit kind end to end first).

**Q3 — GA criteria.** Round 1 lanes 1 and 2: a quorum may not amend §10 ("Release 5.0.0
requires ALL 13 criteria met"). **Adopted: no amendment of §10 by this program.** What
remains is *interpretation*, which this round rules:
- #13 "Provability Mandate (F1–F5 in §14.5) — all 5 metrics in-range". §14.5 defines each
  metric by its *falsifier* ("we're wrong if…"), and §14.6 sets the gate level per release
  (5.0.0: "Silver opt-in. Bronze warned but allowed everywhere"; the Platinum oracle quorum
  arrives in 5.3.0). **Proposed reading:** "in-range" = not in the falsified range, measured
  at the 5.0.0 gate level: F1 ≥ 50 % of contracted `fun` defs non-trivial; F2 ≤ 5 exempt/KLoC;
  F3 has no oracle set to correlate before 5.3.0 — the instrument is built and reports
  `not-applicable: 0 oracles` rather than a number; F4 is counted and reported (the 0 target
  binds "after 5.2"); F5 is measured over what `cargo package` would ship.
  Any metric in its falsified range blocks GA; the program then STOPs for the operator.
- #4 "Downstream book repos compile — 100 % on each book". **Proposed reading:** each repo's own
  harness on a fresh clone of its default branch; a failure is fixed where the defect lives
  (a ruchy bug here, a book bug by a PR to that repo); `tooling-with-ruchy` has no corpus
  (Appendix B's own status table says SKIPPED) and is recorded as vacuous, not as a pass.
- #3 "Zero regressions in 4.x test suite: `cargo test --all-features`". **Proposed reading:**
  the whole suite at 0 failures; an `#[ignore]` added by this program needs a named reason
  (environment: needs a browser, network, or TTY) — never "fails". The 285 failures above are
  work items, not exemptions.
If a criterion cannot be met under these readings, the program stops at `PARTIAL(escalate)`
and asks the operator; it never ships with a criterion unmet.

**Q4 — Lowering.** Round 1 lane 2: no closures (mutable capture), no `HashMap::get` (runtime
defect), generated structs and typed enum variants. **Proposed ruling, now measured (§1b):**
per job, a generated `struct Facts` with one field per distinct (measure term, literal
arguments) pair; a generated `enum Action` with one struct variant per action term used;
`fun decide(facts: Facts) -> Vec<Action>` using `let mut plan … plan.push(…)`, `if/else`, and
`for`/`while` with a counter for `repeat at most N times`; `fun observe() -> Facts` from the
v2 `lowers_to` bindings; `fun apply(plan)` gated behind an explicit flag. No closures, no
maps. The determinism test (F5 of RHL-001) compares two lowerings byte for byte.

**Q5 — Safe-fix scope.** Round 1 lane 1: accept the hybrid. **Proposed ruling:** `check`
keeps RHL-1's local `safe` rule; `fix --safe` applies every safe fix, re-checks the whole
file, and reverts any fix after which the file's error count rose.

**Q6 — Publishing.** Round 1 lane 1: RHL-001 §10 lists "a publish" as a STOP and the directive
"does not waive it". The directive's words are "goal is this is the "non-beta" version to
crates.io and github". **Proposed ruling:** the directive is the operator's explicit request
to publish, which is what the STOP exists to wait for; RHL-001 §9.9 ("no workflow runs `cargo
publish`") still binds, so the publish runs from the tag checkout in this session after every
gate in §3 R1 is green, exactly as beta.2 was published. If `cargo publish` is refused (token),
STOP.

## §3 Phases — plan v2

One ruchy PR in CI at a time (RHL-001 §10). Branches may be prepared while another PR is in
CI; merges are serial. Every acceptance command below is executable and fails on an empty
selection (`--exact` names or a `grep -c` floor); `N_*` floors are recorded in the phase's
receipt at RED time.

| # | Phase | Acceptance `A_i` |
|---|---|---|
| P0 | This plan ruled (round 2 quorum PASS) · doc-test cwd fix · evidence | `cargo test --test cli_contract_doc` exits 0 **and** `git status --porcelain docs/` is empty after it |
| G1 | Full GA measurement: #2, #3 (all-features), #4 (7 books), #5, #7, #8–#10 on RHL + pillar code, #13 per the Q3 reading. Writes `criteria.json` v2 | `jq -e 'length==13 and all(.[]; .status\|IN("MET","NOT_MET"))' criteria.json` (UNMEASURED is not allowed out of G1) |
| G2 | RUNMAIN-1 fix + the 285 workspace failures, by binary, root-caused | `cargo test --workspace --no-fail-fast` exit 0, with the `#[ignore]` count not above 2,773 + named-environment ignores listed in the receipt |
| P1 | RHL-16 v2 vocabulary + corpus | `test "$(cargo test --lib rhl16_ -- --list 2>/dev/null \| grep -c ': test$')" -ge 1 && cargo test --lib rhl::`, and `KNOWN_CORPUS_DEFECTS` empty for v2 (a test asserts it) |
| P2 | RHL-2 `fix --safe`, `vocab`, vocabulary shape | `cargo test --lib rhl::fix rhl::vocab` count ≥ recorded floor · `ruchy fix --safe` on every v2 typo break yields a file that `ruchy check` passes |
| P3 | RHL-3 YAML, `convert` | round-trip property test over every v2 corpus program (count asserted) |
| P4 | RHL-4 lowering | `set -- docs/rhl/breaks/v2/valid/*.rhl; test -e "$1" && for f in docs/rhl/breaks/v2/valid/*.rhl; do ruchy transpile "$f" -o /tmp/x.rs && rustc --edition 2021 --crate-type lib /tmp/x.rs; done` all exit 0, count = number of v2 valid programs; determinism test |
| P5 | RHL-5 examples/expect/receipt, `ruchy test x.rhl` | every v2 valid program's examples pass; a planted wrong `then` fails |
| P6 | RHL-6 MCP + RHL-9 `explain` | `cargo test --features mcp rhl_mcp` count ≥ 8 (one per §6 tool) |
| M1 | RHL-7 (F7) and RHL-8 (F8) measurements, reports committed | `test -s docs/rhl/reports/f7.json && test -s docs/rhl/reports/f8.json` and each carries its verdict and seed |
| G3 | Re-measure GA after all merges | `jq -e 'length==13 and all(.[]; .status=="MET")' criteria.json` |
| R1 | 5.0.0: bump `ruchy`, `ruchy-wasm`; CHANGELOG; clean-room; dogfood; tag; GitHub release; `cargo publish` | fresh `CARGO_HOME`: `cargo install ruchy --version 5.0.0 && ruchy --version` prints `ruchy 5.0.0` |

## §4 STOP conditions
RHL-001 §10 (publish waived by the directive, Q6) · quorum split after this amendment round
→ operator · a GA criterion unmeetable under the Q3 readings → operator · `gate` red after
three five-whys loops · clean-room red · publish refused.

## §5 Rulings — round 2 quorum, 2026-09-23 `[V]`

Three lanes (gemini-3.1-pro-high, gemini-3.8-flash-high, gemini-3.7-flash-high; author
Claude), all exit 0, tree witness verified, no isolation violation: **3/3 PASS.**
Receipt: delegate `ph0r2`, conversations `0e2e2fd4…`, `d184dc97…`, `7a1fbc78…`.

| Q | Ruling | Votes | Residual |
|---|---|---|---|
| Q1 | keep `RHL` / `.rhl` | 2 explicit ACCEPT, 1 implicit | — |
| Q2 | as proposed: RHL ships experimental; F7/F8 measured and published before release | 2 ACCEPT, 1 AMEND | lane 1: "If F8 FAILS, RHL must not ship in 5.0.0" (grounding asserted). Recorded; M1's report is read against it before R1. |
| Q3 | #13/#4/#3 readings are interpretation, not amendment | 2 ACCEPT, 1 AMEND on #13 | lane 1: F3 must correlate the existing oracles (rustc, Kani, SMT). Its citation is the §14.2 tier table, not an oracle quorum. G1 therefore also reports, where more than one discharge engine exists for the same function, their verdict agreement — as data, beside the `not-applicable` F3 reading. |
| Q4 | observe → decide → apply, structs/enums/Vec, no closures or maps | 3 ACCEPT | — |
| Q5 | local `safe`; `fix --safe` reverts a fix that raises the error count | 2 ACCEPT, 1 implicit | — |
| Q6 | the directive authorises the publish; it runs from the tag checkout, no workflow publishes | 3 ACCEPT | — |

Acceptance amendments adopted (lanes 1+2): G3 asserts `length==13`; P1 counts `rhl16_`
tests before running; P4 fails on an empty glob.
