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

## §2 Questions for the quorum (each needs one ruling)

**Q1 — Name and extension (RHL-001 §12.1).** The header says RHL / `.rhl` are placeholders,
"rename before the first release, never after." 5.0.0 would be RHL's first release.
Options: (a) keep `RHL` / `.rhl` and record the ruling; (b) rename now (name?); (c) ship
the RHL verbs in 5.0.0 marked *experimental* with the extension explicitly unstable, and
rule the name before 5.1. Orchestrator recommendation: **(a)** — every artifact, diagnostic
code prefix (`RHL-P…`, stable forever per §9.6) and corpus already uses it; renaming after
codes are published is the costly direction, and the codes are published by this release.

**Q2 — Which RHL rows ship in 5.0.0.** Proposal:
- IN: RHL-2 (`fix --safe`, `vocab`, vocabulary shape), RHL-3 (YAML, `convert`), RHL-4
  (lowering of `job`), RHL-5 (`example`→tests, `expect`→contract, receipt), RHL-6 (MCP
  tools), RHL-9 (`explain`).
- OUT, tracked, not release-blocking: RHL-7 and RHL-8 are *measurements* (agent repair
  rate; the English-vs-RHL A/B) — `kind:measurement` is not implementable by this skill and
  both need model runs whose thresholds are `[U]`; RHL-10/RHL-11 depend on work outside this
  repo (constrained decoding in aprender; the whisper.apr decision); RHL-12 (other unit
  kinds) — spec §8 says one unit kind end to end first.
- RHL-16 (corpus + vocabulary v2): IN, as the first phase — without it no valid corpus
  program checks clean, so RHL-4/5 cannot be exercised on the pre-registered corpus. §3.4
  forbids in-place edits, so this is `v2` files plus a manifest re-hash; the v1 files stay.
  RHL-16's note says pre-registered data changes are an operator decision — the directive
  above routes that decision here.

**Q3 — GA criteria (5.0 spec §10: "ALL 13 … any single failure blocks").** Proposal:
measure every criterion to completion (phase G1); fix every NOT_MET that is fixable in this
repo; for a criterion that cannot be measured as written (#13 F3/F5 have no instrument;
#4 `tooling-with-ruchy` has no corpus), amend §10 in the same PR with the reason and the
replacement measurement — never mark it MET. Book repos (#4): "100% on each book" is
measured as each repo's own harness; a failure caused by the book (not a 4.x→5.0
regression) is ticketed in that repo rather than fixed here. The alternative the quorum may
rule instead: GA waits until #4/#13 are MET as written.

**Q4 — Lowering design for `job` (RHL-4).** Proposal: **observe → decide → apply.**
The job lowers to a *pure* ruchy function `decide(facts) -> plan`: every `measure` term
reads its value from `facts` (keyed by term and literal args), every `action` term appends
a record to the returned plan instead of acting. A generated `observe()` fills `facts`
through each term's `lowers_to` binding, and `apply(plan)` performs the actions, and only
with an explicit `--apply` flag (default prints the plan: §9.4, output is a draft).
Consequences: determinism (F5) is a property of `decide`; `example given … then …` (RHL-5)
lowers to a test that builds `facts` from `given` lines and asserts on the plan, so tests
need no host; `expect` clauses become `ensures` on the plan (count bounds, no write to an
entity). `lowers_to` values are bound in the **v2** vocabulary (v1 is frozen).

**Q5 — Safe-fix scope (carried on RHL-2).** Local (demote a V002 fix only on an RHL-T code
on an edited line — RHL-1's rule) or global (demote on any new RHL-T anywhere). Proposal:
**local for `check`'s `safe` flag, and `fix --safe` re-checks the whole file after applying
and reverts any fix that raised the file's error count** — both readings' failure modes are
closed: a fix never makes a file worse, and the v1 typo breaks keep their pre-registered
safe fixes.

**Q6 — Publishing.** RHL-001 §9.9: "no workflow runs `cargo publish`"; §10 lists "a publish"
as a STOP. The operator directive asks for crates.io and GitHub. Proposal: as beta.2 did —
the publish runs from the tag checkout by this session, after the clean-room gate, the
dogfood gate and a green `gate` on `main`; no workflow publishes. `ruchy` first, then
`ruchy-wasm` after the index resolves.

## §3 Phases (sequential PRs; one ruchy PR in CI at a time, RHL-001 §10)

| # | Phase | PR | Acceptance `A_i` (command) |
|---|---|---|---|
| P0 | GA gather (partial, done) + this plan + quorum rulings | 1 (docs) | `jq -e 'length==13' docs/specifications/evidence/2026-09-23-ga-gather/criteria.json` |
| P1 | RHL-16: `vocab/*-v2.yaml`, v2 valid + planted corpus, manifest re-hash, `KNOWN_CORPUS_DEFECTS` empty for v2 | 2 | `cargo test --lib rhl::` |
| P2 | RHL-2: `ruchy fix --safe`, `ruchy vocab list/show/validate [--format json]`, vocabulary shape contract | 3 | `cargo test --lib rhl:: && cargo test --test rhl_cli` |
| P3 | RHL-3: YAML surface (`.rhl.yaml`), `ruchy convert`, round-trip property test (F3) | 4 | same |
| P4 | RHL-4: lowering of `job` (Q4), `ruchy transpile x.rhl [--emit ruchy]`, `ruchy compile x.rhl`, determinism test (F5) | 5 | same + corpus transpile→rustc |
| P5 | RHL-5: `example`→tests, `expect`→`pv` contract, receipt; `ruchy test x.rhl` | 6 | same |
| P6 | RHL-6 + RHL-9: MCP tools (§6) and `ruchy explain` | 7 | same + `cargo test --features mcp` |
| G1 | GA criteria measured to completion; fixes / §10 amendments per Q3 | 8+ | `jq` over a re-gathered `criteria.json`: no UNMEASURED without an amendment |
| R1 | 5.0.0: version bump (`ruchy`, `ruchy-wasm`), CHANGELOG, clean-room, dogfood, tag `v5.0.0`, GitHub release, `cargo publish` | 9 | `cargo install ruchy --version 5.0.0` in a fresh `CARGO_HOME` runs `ruchy --version` |

Each implementation phase: RED tests first, then code; ≤10 complexity per function; the
pre-PR review is a 3-lane quorum over the diff; merge only on green `gate` + quorum PASS.

## §4 STOP conditions (inherited, plus)
RHL-001 §10 · quorum split after one amendment round → operator · `gate` red after three
five-whys loops · clean-room red · publish token refused (as beta.2's 403).
