# RHL-1 plan — parser, intent tree, `fmt`, diagnostics JSON, `check`

Spec: `docs/specifications/ruchy-high-level-language-interface.md` (RHL-001) §3–§5, §7 F1/F2/F4/F9.
Row: `RHL-1` in `docs/roadmaps/roadmap.yaml` (admitted by RHL-0's merge, `1c2caf5d`).
Branch: `RHL-1-parser-fmt-check` off `main` at `e41774b4`.
Budget: K̂ 150 `[A]` (spec §8) · K 190 · andon at 152.

This plan names the design decisions RHL-1 needs that the spec does not make. Each has a
proposal and the reason for it. The decisions are put to a quorum. None is recorded as
the operator's.

---

## 1. Measured before planning `[V]`

| # | Fact | Command |
|---|---|---|
| M1 | 84 `.rhl` files exist: 12 valid programs, 72 planted breaks (6 classes × 12). The task corpus has none; it is English plus Rust tests. | `find docs/rhl -name '*.rhl' \| wc -l` |
| M2 | The pre-registered break expectations are: `missing end`→`RHL-P001`, `typo'd term`→`RHL-V002` (`safe_fix: true`), `two-candidate typo`→`RHL-V003` (candidates `[host, hour]`), `undeclared effect`→`RHL-E001`, `wrong type`→`RHL-T002`, `wrong unit`→`RHL-T001`. No other code is allocated anywhere in the repo. | `docs/rhl/breaks/planted/*/*/break.yaml` |
| M3 | **The 12 valid programs are not valid under the pre-registered vocabulary.** All 12 use phrases that are no vocabulary term: `ticket count` (in `expect`/`then`), and `title`/`label` (the attribute lines of the `with` block). §3.3 says everything else in a program is a term, a literal or a `let` name. | `grep -h 'ticket count\|title\|label' docs/rhl/breaks/valid/*.rhl` vs `vocab/*.yaml` |
| M4 | **6 of 12 valid programs are mistyped or undeclared-effect under the vocabulary's own types and effects.** Every program compares `free` with a `GB` quantity and declares only `read disk`/`write tickets`/`call tickets`. But `runner load of` gives `Percent` with effect `read runner` (03, 08), `pin status of` gives `Text` with effect `read host` (04, 09), and `tickets filed in` gives `Count` with effect `read tickets` (05, 10). | `vocab/fleet-v1.yaml`, `vocab/tickets-v1.yaml`, `grep 'let free\|may ' docs/rhl/breaks/valid/*.rhl` |
| M5 | Because of M4, three planted breaks cannot be told from their base by code alone. For 03/08 `wrong unit`, the base already has a Percent-vs-Size mismatch on the same line. For 04/09 `wrong type` (`free is below "low"`), the operands are Text vs Text. | as M4 |
| M6 | **The spec's §3.2 example conflicts with the break corpus on `may call`.** §3.2 says "`may read disk` and `may call tickets` are the entire effect surface", so `call tickets` must cover `file ticket` (effect `write tickets`). But every `undeclared effect` break removes only `may write tickets`, keeps `may call tickets`, and expects `RHL-E001`. | spec §3.2; `docs/rhl/breaks/planted/undeclared-effect/*` |
| M7 | Entity instances are closed-world (`instances_from: machines/*/forjar.yaml`, `fleet/runners/*.yaml`, `org/repos/*.yaml`). None of those paths exists in this repository. | `ls machines fleet org` |
| M8 | `ci / gate` runs `cargo test --lib` on the root crate only (binding B4). Every RHL-1 gate must be a lib test. | `docs/rhl/phase0-bindings.md` B4 |
| M9 | `lalrpop 0.23` is already a dev-dependency (RHL-0), so it is in `Cargo.lock`. `ruchy check` has no `--format`; `ruchy fmt` takes one `file`. | `Cargo.toml:424`, `src/bin/ruchy.rs:163,416` |

---

## 2. Design decisions (the spec does not make these)

### D1 — How the production parser relates to the pre-registered grammar

**Proposal.** Generate the parser at build time with LALRPOP from `src/rhl/grammar.lalrpop`.
That file has the **same productions** as `grammar/rhl.lalrpop`; only the action code
differs (it builds the tree instead of returning `()`). A lib-test gate strips action code,
bindings and comments from both files and asserts the production skeletons are
**identical**. So F1's "conflict-free by the generator's report" applies to the parser that
runs, not to a copy that could drift. `lalrpop` moves to `[build-dependencies]`, and
`lalrpop-util` (feature `lexer`) goes into `[dependencies]`. `grammar/rhl.lalrpop` stays
byte-frozen: it is manifest-hashed.

*Rejected: a hand-written recursive-descent parser.* Nothing would tie it to the proven
grammar. "Exactly one parse" would then be a claim about a different grammar.
*Rejected: committing the generated parser.* It is thousands of generated lines under
`src/`, it would need a freshness gate anyway, and the generator would still have to run.

### D2 — What RHL-1 does about M3–M6 (the corpus is not valid under its own vocabulary)

**Proposal.** Implement the checks **as the spec states them**, strictly. Do not tune them
to make the valid corpus pass. Then measure the valid corpus and **report** every
diagnostic it produces. Do **not** gate "valid programs check clean" in RHL-1, because it
is false at HEAD for reasons in pre-registered, hash-frozen files. File one follow-up row
(RHL-16) for the corpus/vocabulary repair: v2 files, never an in-place edit (§3.4 freeze).
It goes to the operator, because it changes pre-registered data.

What RHL-1 **does** gate on the corpus:
- **F1:** all 12 valid programs and all 60 non-`missing end` breaks parse. All 12
  `missing end` breaks are refused with `RHL-P001`.
- **F2:** `fmt` is idempotent and preserves the tree on every parseable corpus program.
- **Breaks:** every planted break yields its `expected_code` **on the mutated line**. For
  `RHL-V002` there is exactly one candidate, and it carries a `safe` fix. For `RHL-V003`
  the candidates are exactly the `break.yaml` list, with no safe fix.
- **The differential is reported, not gated:** a break's code must be *new* relative to
  its base. The M5 breaks fail this, and the receipt names each one.

*Rejected: a lenient checker*, where unknown phrases, `may call` and quantity dimensions
pass so the valid corpus comes out clean. That tunes the compiler to the corpus, and it
breaks §3.3 and F4.

### D3 — The diagnostic code catalogue (stable forever, §3.5 / §9.6)

**Proposal.** Allocate only the codes RHL-1 emits and tests. Keep the six pre-registered
codes and their meaning exactly. Record the catalogue once, in `src/rhl/codes.rs`
(the single source), rendered to `docs/rhl/diagnostic-codes.md`.

| Code | Meaning | Emitted when |
|---|---|---|
| `RHL-P001` | missing `end` | end of input where the parser expects `end` (pre-registered) |
| `RHL-P002` | unexpected token | any other unrecognized token; `expected` = the parser's expected set |
| `RHL-P003` | invalid token | a character sequence that starts no token (e.g. `!`, `\r`, an unterminated string) |
| `RHL-V001` | unknown word, no candidate | a phrase that resolves to no term, no `let` name and no instance, with no candidate within the threshold |
| `RHL-V002` | unknown word, one candidate | as V001, with exactly one candidate; the fix is `safe` (pre-registered) |
| `RHL-V003` | unknown word, several candidates | as V001, with ≥2 candidates; no fix is `safe` (pre-registered) |
| `RHL-V004` | unknown vocabulary | `use vocabulary <name> v<N>` names no loadable file |
| `RHL-T001` | wrong unit | both operands are quantities of different dimensions (pre-registered) |
| `RHL-T002` | wrong type | an operand type does not fit (e.g. Text where a Size is needed; an ordering comparison on Text) (pre-registered) |
| `RHL-T003` | bare number | an unannotated integer where a unit-bearing quantity is expected (§3.1 principle 4) |
| `RHL-E001` | undeclared effect | a term's `effect` is not covered by a `may` line of its unit (pre-registered) |
| `RHL-X001` | `given`/`then` outside `example` | the grammar leaves this check to RHL-1 (grammar "NOTE ON SCOPE") |
| `RHL-B001` | not a finite collection | `for each … in <x>` where `<x>` is not a finite collection. No v1 term gives a collection, so every `for each` is refused today (§3.1 principle 5). *Added by round 1, lane 1.* |
| `RHL-C001` | no contract template | a vocabulary term whose `contract:` file does not exist; the **vocabulary** is refused by name, exit 2 (§3.4, §3.5 `NoContractTemplate`, §9.3, F6's positive control). *Added by round 1, lanes 1 and 2.* |

The V001/V002/V003 numbering follows the pre-registered pair: V002 means one candidate
and V003 means several, so V001 means none. An unknown **effect target** (`may read dsk`)
and an unknown **unit** (`100 gib`) reuse V001–V003, with candidates drawn from their own
namespace. The code says "unknown word"; `expected.kind` says which namespace.

*Round 1 (lane 2): D3 and D4 read as contradicting each other.* They are reconciled like
this: the **syntactic position** picks the namespace, and the expected **semantic** kind
never does. The word after `may <verb>` comes from effect targets. The word after an
integer in a quantity comes from unit terms. Every generic `App` position (the operand of
`runs on`, `let … be`, `when`, a comparison) draws from all terms plus the `let` names in
scope, because the grammar leaves that position open.

### D4 — Candidate rule for unknown words (an invented threshold would be a §10 STOP)

**Proposal.** The distance is Levenshtein over characters, spaces included. The threshold
is **rustc's `find_best_match_for_name` rule**, `max(len/3, 1)`. It is borrowed and cited,
not invented. The search runs over the unresolved phrase's word-prefixes, **longest
first**. The first prefix length that yields any candidate within the threshold wins, and
its candidates are every term (or in-scope `let` name) at that prefix's **minimum**
distance. In a generic `App` position, candidates are **not** filtered by expected kind (see the
D3 reconciliation: the position picks the namespace). The pre-registered
`hosr → [host, hour]` requires this, because `hour` is a unit where an entity is expected.

Checked by hand against M2:
- `disk fre of` → `disk free of` (1, whole phrase).
- `runer load of` → `runner load of` (1, whole phrase). The 1-word prefix `runer → runner`
  never competes, because the longer span matched first.
- `hosr gx10` → no 2-word match within 3, so the 1-word prefix gives `host` (1) and
  `hour` (1): V003 ✓.
- `pin staus of`, `ticket filed in` and `disk usge of` each give one candidate: V002 ✓.

### D5 — Effect coverage, and the §3.2-vs-breaks conflict (M6)

**Proposal.** A term effect `<verb> <target>` is covered **iff** its unit declares
`may <verb> <target>`. `may call X` covers only effects whose verb is `call`, and no v1 term
has one. This follows the **pre-registered breaks** over the §3.2 prose, because the breaks
are the pre-registered falsifier (F7/F9) and the prose is an illustration. The
consequence, stated rather than hidden: §3.2's example as written yields `RHL-E001` on its
`file ticket` line. An amendment note (A3) goes into the spec in this PR to say so.

### D6 — Closed-world entity instances with no source in the repo (M7)

**Proposal (amended by round 1, lane 2).** Relative paths in a vocabulary file resolve
against the directory that contains `vocab/`. The same rule already locates its
`contract:` files, so `instances_from` needs no new flag. The instance name is whatever
the glob's `*` matched: `machines/gx10/forjar.yaml` declares `gx10`.

- **The glob matches at least one file:** the entity is closed-world. An unknown instance
  is `RHL-V001`–`V003`, with candidates drawn from the declared instances (`host gx11` →
  `gx10`, §3.4).
- **The glob matches nothing:** no source of truth exists, so by §3.4's own qualifier
  ("closed-world *where a source of truth exists*") the entity is not closed-world and
  nothing is refused. The JSON output says so in a top-level `unverified` list, e.g.
  `host gx10 — no file matches machines/*/forjar.yaml`. That list carries no diagnostic
  code, does not change the exit code, and says which instances were unverified.

*Dropped from the first draft:* the `--world` flag, the code `RHL-V005`, and exit 2 on an
absent source. Lane 2: the flag and the code were unruled inventions, and exit 2 would
refuse every program in this repository.

### D7 — The `fmt` normal form

**Proposal.** The tree carries no layout. `fmt` prints:
- one statement per line, two-space indentation per block depth;
- single spaces between tokens;
- a trailing newline;
- one blank line between the `use` block and the first unit, and between units;
- inside a **unit body only**, between two **consecutive sibling** statements: a blank line
  when either is a block statement (`when`, `for each`, `repeat`, `example`, an action with
  a `with` block), or when both are simple statements of **different groups** — header
  (`runs on`, `every`, `may …`, `wait up to`), binding (`let`, `set`), assertion
  (`expect`), other. Never a blank line before the first statement or after the last, so
  the unit's closing `end` follows its last statement directly. *Reworded by round 1
  (lanes 1 and 2): "before and after every block" put a blank line before the unit's
  `end`.*
- no blank lines inside nested bodies.

Checked by hand: this reproduces all 12 valid programs and the §3.2 example
byte-for-byte. That is F2's positive control ("a whitespace-mangled program formats to
the corpus bytes"), which presupposes the corpus is in normal form.

### D8 — CLI surface in RHL-1

**Proposal.** Dispatch on the `.rhl` extension inside the existing verbs (§1, rule 1):
- `ruchy check <f.rhl> [--format text|json]`. Exit codes: 0 pass,
  1 error diagnostics, 2 refusal.
- `ruchy fmt <f.rhl> [--check|--stdout]`.

*Round 1 (lane 2):* today the binary exits 1 on every handler `Err`
(`src/bin/ruchy.rs`, the final `process::exit(1)` in `main`). P5 therefore returns a
typed exit status from the `.rhl` path of `check`, and `main` maps it to 0/1/2 instead of
sending it through the generic `Err` → 1 path.

**JSON output shape (P2, stated for round 2).** `--format json` prints one object,
`{"file", "verdict", "diagnostics": [<§3.5 objects>], "unverified": [<D6 entries>]}`,
where `verdict` is `pass`, `fail` or `refused`. §3.5 says the command "emits the list".
Here the list is the `diagnostics` array. It is wrapped in an object only so that D6's
`unverified` list has somewhere to live without becoming a diagnostic code. Each
diagnostic also carries `refusal` (for example `OutOfVocabulary`) when its code is an
instance of a §3.5 refusal. **Exit rule:** 2 if any diagnostic carries a refusal, else 1
if there is any error, else 0. The catalogue in `src/rhl/codes.rs` maps codes to refusals.

Vocabulary search path: `vocab/` beside the file's nearest ancestor that contains one,
else `--vocab <dir>`. All logic lives in the lib (`src/rhl/`); the bin is thin wiring, so
the logic is gated by `cargo test --lib` (M8).

---

## 3. Phases and acceptance commands

| # | Phase | Scope | Route | A_i |
|---|---|---|---|---|
| P1 | tree + generated parser + skeleton-identity gate + F1 over the corpus | `src/rhl/{mod,tree,parse}.rs`, `src/rhl/grammar.lalrpop`, `build.rs`, `Cargo.toml` | self | `cargo test --lib rhl::parse` |
| P2 | diagnostics model, P-codes, JSON render, code catalogue | `src/rhl/{diag,codes}.rs` | self | `cargo test --lib rhl::diag` |
| P3 | `fmt` normal form: F2 idempotence and tree preservation over the corpus plus proptest; mangled→corpus positive control | `src/rhl/fmt.rs` | worker (disjoint scope) | `cargo test --lib rhl::fmt` |
| P4 | vocabulary loader (C001 contract-template refusal), resolver, V/T/E/X/B checks, candidates and fixes, planted-break gate, valid-corpus measurement | `src/rhl/{vocab,check}.rs`, `src/rhl/check/**` | self | `cargo test --lib rhl::check` |
| P5 | CLI wiring by extension; exit codes | `src/bin/ruchy.rs`, `src/bin/handlers/check_handler.rs`, the fmt handler | self | `cargo test --lib rhl::cli` (the process exit code is also exercised by `tests/rhl_1_cli_exit_codes.rs`, which is outside the required check, B4) |
| P6 | spec amendment notes (A3: §3.2 effect, M3–M6), code catalogue doc, roadmap (RHL-16 filed), CHANGELOG, receipt; diff quorum; PR | `docs/**`, `CHANGELOG.md` | self + delegate | `pmat work validate` + quorum PASS |

A mutation must be observed RED for the DoD. `cargo mutants --file src/rhl/check.rs` runs
on the resolver and the type/effect rules.

## 4. STOP conditions that could fire

- `grammar-conflict`: only if D1's generated parser needs a production the frozen grammar
  lacks. Report it; do not add the production.
- Any threshold not covered by D4's borrowed rule.
- A quorum split on D2 or D5 after one round.

## 5. Quorum rounds

**Round 1 (`grillme`, width 3; gemini-3.1-pro-high, gemini-3.8-flash-high,
gemini-3.7-flash-high).** Verdicts: FAIL, FAIL, PASS, so no agreement. All three lanes
re-measured M3–M7 as true, and D1, D2 and D5 were accepted by every lane that took a
position. The amendments taken, each written into its decision above:

- D3 gains `RHL-B001` and `RHL-C001` (lanes 1 and 2).
- D3 and D4 are reconciled: the syntactic position picks the namespace (lane 2).
- D6 drops `--world`, `RHL-V005` and exit 2 on an absent source (lane 2).
- D7's blank-line rule is reworded (lanes 1 and 2).
- D8 names the exit-status plumbing (lane 2).

Lane 2's charge that "not filtering by kind" tunes the checker to one fixture is answered,
not adopted. The `[host, hour]` expectation is pre-registered F7 data, and the
reconciliation filters by position, not by semantic kind. Round 2 grills the amended
plan. A second split is §10's `quorum-split` STOP.

**Round 2 (`grillme`, width 3; same three models).** Verdicts: FAIL, FAIL, PASS, the
same split as round 1. That is §10's `quorum-split` STOP, so the disputed decisions
went to the operator.

- Both FAIL lanes blocked on D6 (an absent instance source still exits 0) and on D8
  (an object where §3.5 says "emits the list").
- Both also noted that `EngineUnavailable` has no code.
- Lane 1 objected to a `cargo run` step in the P5 gate.
- Lane 2 found a leftover `[--world <dir>]` in D8.
- All three lanes accepted D7. The fmt tests later confirmed it byte-for-byte.

**Operator rulings (2026-09-21).** These are the options the operator selected,
quoted verbatim:

- D6: **"Pass, list as unverified (Recommended)"**.
- D8: **"Object, amend §3.5 (Recommended)"**.
- The two planted breaks that contradict the vocabulary (below):
  **"Named exception + RHL-16 (Recommended)"**.

Taken without a ruling, because they are corrections rather than choices:

- The `--world` leftover and the "`Unknown`" wording are removed from D8.
- The P5 acceptance command is a lib test, and the process exit code is tested
  outside the required check.
- `EngineUnavailable` is recorded as deferred to the first row that calls an engine
  (spec Amendment A3 item 4).

**Measured after the rulings.** These are what the checker reports, not tuned to:

- **M3 and M4 hold.**
- **M5 undercounted.** Six planted breaks repeat a code their base already has on
  the mutated line, not four: wrong-type 04/09 and wrong-unit 03/05/08/10.
- **Two breaks contradict the vocabulary.** `wrong-unit/04` and `/09` expect
  `RHL-T001`, but their base binds `free` to Text, so the checker reports
  `RHL-T002`. They are the named exception, and RHL-16 repairs the corpus as v2
  files.
