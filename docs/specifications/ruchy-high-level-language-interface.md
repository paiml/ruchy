# RHL-001 — Ruchy High-Level Language Interface

**Repo:** `paiml/ruchy` · **Drop at:** `docs/specifications/ruchy-high-level-language-interface.md`
**Status:** DRAFT v0.1 for quorum (one agy · one claude · one apr) · 2026-09-20 · owner: Noah Gift
**Working name:** RHL ("Ruchy High Level"). File extension `.rhl`. Both are `[A]` placeholders — rename before the first release, never after.
**This month:** this spec ruled + the pre-registered corpus (row RHL-0). **No implementation rows are admitted until RHL-0 is merged** — the readiness rule: a row is admitted only when its plan is ruled and its falsifier is proven satisfiable.

Provenance: `[V]` verified by command · `[C]` cited (README / repo tree at `paiml/ruchy` `main`, read 2026-09-20) · `[A]` operator directive or estimate · `[U]` unknown — bind at Phase 0 by reading HEAD.

---

## §0 Purpose

**A layer between human language and code.** Today the path is *English → agent → Rust*. English is ambiguous, so the agent guesses, and a wrong guess looks exactly like a right one until it runs. RHL inserts a small language that is **code, not prose**:

```
human intent ──► RHL (terse, closed vocabulary, one meaning) ──► ruchy ──► Rust ──► pv contract + tests
   voice · MCP · YAML · typed by hand or by an agent
```

Design target, in one sentence: **a language an agent can repair from the compiler's error output alone** — more robust than English because it cannot be ambiguous, and more robust than Rust because its failure messages are data with the fix attached.

It is **ontology-heavy**: every noun and verb is a declared term in a versioned vocabulary validated by `pv` shapes. An unknown word is a compile error with candidates, never a guess.

It is for the operator and his agents first. It is not a beginner's teaching language and not a replacement for ruchy or Rust.

### Non-goals
- Not a general-purpose language. No generics, lifetimes, traits, macros, unsafe, or user-defined syntax.
- Not a second compiler to Rust: RHL lowers to **ruchy**, and ruchy's existing transpiler produces the Rust.
- Not a shell generator (bashrs owns shell) and not a provisioning language (forjar owns hosts). RHL may *call* their verbs through vocabulary; it does not re-implement them.
- No model in the compile path. A model may help *write* RHL; from `.rhl` downward everything is deterministic.

---

## §1 Ground truth — ruchy today `[C]`

| Fact | Source |
|---|---|
| ruchy transpiles to Rust and compiles to native binaries; generated Rust is 100 % safe | README |
| Latest release v4.2.1 (2026-02-10); `cargo install ruchy`; MIT; MSRV 1.75 | README, releases |
| Syntax: `let`, `fun`, `match`, f-strings, closures, `async fun`, labeled loops | README |
| Existing CLI: `ruchy` (REPL) · `ruchy <file>` · `-e` · `compile` · `transpile` · `check` · `lint` · `fmt` · `test` · `wasm compile/run` · `mcp` (feature `mcp`) | README |
| An MCP server already exists (`ruchy mcp`) | README |
| Repo already has `grammar/`, `contracts/`, `stdlib/`, `ruchy-embed/`, `ruchy-wasm/`, `notebooks/`, `docs/SPECIFICATION.md`, `docs/execution/roadmap.yaml` | repo tree |
| Three disabled-test directories exist at the root (`tests.disabled`, `tests_disabled_for_mutation`, `tests_temp_disabled_for_sprint7_mutation`) | repo tree — **a standing debt; see RHL-0** |
| ruchy's internal AST and the right seam for a new front end | `[U]` — bind at Phase 0 with `pmat query` |
| What `contracts/` holds and whether `pv lint` runs in ruchy's required check | `[U]` |

**Consequences for this design**
1. **No verb collisions.** `check`, `fmt`, `lint`, `test`, `transpile`, `compile`, `mcp` already exist. RHL **extends them by file extension** (`ruchy check job.rhl`) and adds only the verbs that do not exist: `fix`, `explain`, `convert`, `vocab`, and later `say` / `ask`.
2. **Small compiler.** RHL is a front end that emits ruchy. Everything from ruchy down is already built and tested.
3. **MCP is an extension, not a new server.** New tools are added to `ruchy mcp`.

---

## §2 The four inputs, one tree

All four surfaces produce the **same AST** (the *intent tree*). There is one checker and one compiler.

```
             ┌─ RHL text (.rhl) ───────── hand or agent ──┐
             ├─ YAML (.rhl.yaml) ──────── tools, forjar-style authors ─┤
inputs  ─────┤                                                        ├──► intent tree ──► check ──► ruchy ──► Rust
             ├─ MCP tool calls ────────── agents (arbiter, RAH, Claude) ─┤
             └─ voice ─► transcript ─► apr (grammar-constrained) ─► RHL text ─┘
```

| Surface | What it is | Rule |
|---|---|---|
| **RHL text** | The readable syntax (§3) | The canonical human form; what gets reviewed in a PR |
| **YAML** | A lossless serialization of the same tree | `ruchy convert` round-trips RHL ⇄ YAML byte-identically after `fmt`. One schema family with forjar and contract YAML — not a third dialect |
| **MCP** | The same operations as tools (§6) | An agent never needs to shell out or scrape text |
| **Voice** | Intent capture only — never editing | whisper.apr (local) → transcript shown back → `apr` decodes **under the RHL grammar** → `.rhl` draft. Voice is an untrusted channel: it produces a draft or a PR, never an action |

The only non-deterministic step in the whole system is *voice/English → RHL*, and its output is a file a human or agent reads before anything compiles.

---

## §3 The language

### 3.1 Principles (each is a testable property, §7)
1. **One meaning.** The grammar is unambiguous (LL(1) or proven conflict-free). Every valid program has exactly one parse.
2. **One spelling.** `ruchy fmt` is total and idempotent; formatted source is a normal form. Two programs with the same tree format to the same bytes.
3. **Closed vocabulary.** Nouns and verbs come from `use vocabulary` declarations. An unknown term never compiles.
4. **Typed literals with units.** `100 GB`, `1 hour`, `5 %`, `"text"`, `host gx10`. A bare number where a quantity is expected is an error.
5. **Bounded by construction.** `for each` over finite collections; `repeat` requires `at most N times`; `wait` requires `up to <duration>`. No unbounded loop is expressible.
6. **Effects are declared.** A unit states what it may touch (`may read`, `may write`, `may call`). Anything else is a compile error, and the declaration becomes part of the contract.
7. **Examples are tests; expectations are contracts.** `example … given … then …` lowers to a test. `expect …` lowers to `pv` `requires` / `ensures` / `invariants`.
8. **Line-oriented, block-closed.** One statement per line; blocks end with `end`. No significant indentation, no optional punctuation — an agent can repair a truncated file by counting `end`s.
9. **Small.** The keyword list fits on one screen (§3.3) and only grows by spec amendment.

### 3.2 Example

```
use vocabulary fleet v1
use vocabulary tickets v1

job "gx10 disk watch"
  runs on host gx10
  every 1 hour
  may read disk
  may call tickets

  let free be disk free of "/"

  when free is below 100 GB
    file ticket in repo "paiml/infra" with
      title "gx10 disk below 100 GB"
      label "fleet"
    end
  end

  expect no change to host
  expect at most 1 ticket per run

  example "low disk files one ticket"
    given disk free of "/" is 90 GB
    then 1 ticket is filed
  end

  example "healthy disk files nothing"
    given disk free of "/" is 400 GB
    then 0 tickets are filed
  end
end
```

Read aloud it is nearly English. Parsed, it has exactly one meaning: `disk free of`, `file ticket`, `host`, `GB` are vocabulary terms with types; `below` is a comparison on a `Size`; both examples become tests; both `expect` lines become contract clauses; `may read disk` and `may call tickets` are the entire effect surface.

### 3.3 Keyword set v0 (closed)

`use vocabulary` · unit kinds: `job` `command` `check` `pipeline` `shape` · `let … be` · `set … to` · `when / otherwise / end` · `for each … in … / end` · `repeat at most N times / until / end` · `wait up to` · `may read | write | call` · `runs on` · `every` · `expect` · `example / given / then / end` · `give back` · `stop with` · `with` · comparisons: `is` `is not` `is below` `is above` `is at least` `is at most` `is one of` `contains` · logic: `and` `or` `not` · `end`

Everything else in a program is a **vocabulary term**, a **literal**, or a **name** introduced by `let`.

> **Amendment A1 — `with` (RHL-0, 2026-09-20) `[V]`.**
> The keyword `with` is new. §3.2 originally opened an action's attribute block
> with no marker at all, and that syntax is not conflict-free. Measured with
> LALRPOP 0.23 on the transcribed production, the generator's own words:
>
> > *Local ambiguity detected. The problem arises after having observed the
> > following symbols in the input: `Body App`. At that point, if the next token
> > is a `NEWLINE`, then the parser can proceed in two different ways.*
>
> A parser cannot tell a plain statement from a block opener until it has
> counted every `end` to the end of the file, so the unmarked form is not LR(k)
> for any fixed k. It contradicts §3.1 principle 1 — "every valid program has
> exactly one parse" — which is the principle F1 exists to protect, so the
> example had to give way rather than the principle.
>
> `with` is the minimal repair **that keeps §3.2's nested attribute block**: one
> keyword, no §3.1 principle weakened, the keyword list still fits on one screen.
> §3.3 says the set grows only by spec amendment; this is that amendment, and
> RHL-0 is the row that rules the spec.
>
> *Correction, same PR.* This paragraph first said `with` was "the only" repair
> keeping every §3.1 principle intact. A second-family review falsified that: it
> measured a conflict-free grammar in which the nested block is removed
> altogether and attributes become sibling statements, adding no keyword at all
> — arguably better service to principle 9 ("small"). `with` is still preferred,
> because an action's attributes reading as siblings of the statements around
> them loses the nesting §3.2 uses to say which attributes belong to which
> action. But "the only one" was not measured and was not true.
> **Open question O1 — three lines of §3.2 do not parse (RHL-0, 2026-09-20) `[V]`.**
> A parser was generated from `grammar/rhl.lalrpop` and fed §3.2 verbatim. After
> Amendment A1 these three lines still fail:
>
> | line | failure | why |
> |---|---|---|
> | `expect no change to host` | `UnrecognizedToken "to"` | `to` is reserved by `set … to`, so no phrase may contain the word |
> | `expect at most 1 ticket per run` | `UnrecognizedToken "per"` | `Args` admits only literals and quantities, so a bare word cannot follow an argument |
> | `then 0 tickets are filed` | `UnrecognizedToken "are"` | `are` is not a comparison operator |
>
> Two neighbouring failures were REPAIRED rather than recorded, because they were
> plainly defects: `100 GB` did not lex at all (`WORD` is lowercase-only, so the
> unit in §3.1 principle 4's own example matched no rule), and the vocabulary term
> `tickets filed in` could not appear in a condition (`in` was reserved and
> `App "in" App` existed only under an action). With both fixed, all 72
> non-`missing-end` corpus programs parse.
>
> These three are a design question, so both candidate repairs were MEASURED and
> are put to the operator rather than guessed at.
>
> **Candidate O1-A — reserve `per` and `are`, and let `to` and `per` join a
> comparison the way `in` already does.** Two new keywords, three new productions.
> Committed as `grammar/fixtures/o1-a-candidate.lalrpop`.
>
> | measurement | result |
> |---|---|
> | LALRPOP 0.23.1 | **`Ok` — no conflict, no local ambiguity** |
> | the three failing §3.2 lines, through a generated parser | **all three parse** |
> | the 72 non-`missing-end` corpus programs | **72/72, 0 failures** |
>
> Cost: §3.3 grows by two keywords, and `per` and `are` become unusable inside any
> vocabulary term — the same trade §3.3 already makes for `of`, `in` and `to`. It
> costs §3.1 principle 8 (block-closed, count the `end`s) **nothing**: no
> production here touches block structure.
>
> **Candidate O1-B — admit a bare `WORD` as an argument. No new keyword.**
> Committed as `grammar/fixtures/o1-b-candidate.lalrpop`.
>
> | measurement | result |
> |---|---|
> | LALRPOP 0.23.1 | **`Err` — "Local ambiguity detected"** |
>
> In the generator's own words: *"and looking at a token `UNIT` we can reduce to a
> `Quantity` but we can also shift"*, and the same for `"%"`. Once a bare word can
> be an argument, `100 GB` has two readings — a quantity, or an integer followed by
> a word argument. It costs principle 8 nothing and principle 1 everything, and
> principle 1 is the one F1 exists to protect.
>
> **The ruling wanted, in one line:** adopt O1-A and add `per` and `are` to §3.3,
> or reword these three lines of §3.2 so the language does not need them. O1-B is
> not available — it is not conflict-free.
>
> The unmarked form is kept verbatim as `grammar/fixtures/ambiguous.lalrpop`,
> where it serves as F1's positive control. Evidence:
> `grammar/rhl.conflict-report.txt`; gate:
> `src/rhl_pre_registration/grammar_gate.rs`.

### 3.4 Vocabulary — where the ontology lives

A vocabulary is a versioned YAML file of **terms**, itself a contract validated by a `pv` shape (one schema family with `contracts/*.yaml`):

```yaml
vocabulary: fleet
version: 1
terms:
  - term: disk free of
    kind: measure            # noun | measure | action | unit | entity
    takes: [{name: path, type: Text}]
    gives: Size
    effect: read disk
    lowers_to: ruchy stdlib call        # bound at RHL-3
    contract: contracts/rhl-fleet-disk-free-v1.yaml
  - term: host
    kind: entity
    instances_from: machines/*/forjar.yaml    # closed world: only declared hosts exist
```

Rules:
- **No term without a contract template.** The compiler refuses the vocabulary, by name.
- **Entities are closed-world** where a source of truth exists (hosts come from forjar declarations; repos from the org). `host gx11` is a compile error with `gx10` as the candidate.
- Vocabulary terms map to the ontology's entity types (`paiml-ontology.md` Σ) — a term is an instance of a Σ class, so shapes inherit. Exact binding is `[U]` until Phase 0 reads Σ at HEAD.
- Vocabularies are versioned and **never edited in place**: `v1` is frozen when first released; changes ship as `v2`.

> **Amendment A2 — Σ and term kinds (RHL-0, 2026-09-20) `[V]`.**
> The bullet above about Σ says "a term is an instance of a Σ class, so shapes
> inherit". Σ was read at HEAD as this section instructs — ONT-001 v4.9 §3.1,
> `infra/docs/specifications/paiml-ontology.md` — and its `entity_types`
> classify **what artifact is under contract** (`code`, `documents`, `data
> files`, `web`, `media`, `pv-contract`). They do not classify what sort of
> *word* a term is. Σ and `noun | measure | action | unit | entity` are
> different sorts, and inheriting one from the other is a category error.
>
> RHL-0 therefore pre-registers vocabularies in which the **file** carries
> `sigma_entity: pv-contract` and a term carries no `sigma_class` at all.
>
> This is why §10's STOP condition `vocabulary-unbindable` — "Σ at HEAD cannot
> express term kinds" — **does not fire**: Σ expresses what is genuinely under
> contract, which is the vocabulary document. What could not be bound was the
> sentence, not the ontology. Rewriting the bullet itself is left to RHL-2, when
> the vocabulary shape enters `pv` and the wording can be fixed against a working
> shape rather than guessed at. See `docs/rhl/phase0-bindings.md` binding B5.

### 3.5 Diagnostics are data — the agent-repair contract

Every diagnostic is a JSON object with a **stable code**, a **span**, the **expected set**, **candidates**, and **fix edits**. This is the interface agents program against; the human-readable message is rendered from it.

```json
{
  "code": "RHL-V002",
  "severity": "error",
  "file": "gx10-disk.rhl",
  "span": {"line": 8, "col": 15, "len": 12},
  "message": "unknown term `disk fre of`",
  "expected": {"kind": "measure", "gives": "Size"},
  "candidates": [{"term": "disk free of", "vocabulary": "fleet v1", "distance": 1}],
  "fixes": [
    {"title": "replace with `disk free of`",
     "safe": true,
     "edits": [{"span": {"line": 8, "col": 15, "len": 12}, "text": "disk free of"}]}
  ]
}
```

- `ruchy check --format json` emits the list. `ruchy fix --safe` applies every fix marked `safe: true` — deterministic, idempotent, and re-checks.
- A fix is `safe` only when it is the **unique** candidate and preserves types. Two candidates ⇒ no safe fix; the agent (or human) chooses.
- Codes are stable forever and grouped: `RHL-P` parse · `RHL-V` vocabulary · `RHL-T` type/unit · `RHL-E` effect · `RHL-B` bound · `RHL-C` contract · `RHL-X` example.
- **Refusals are diagnostics too**, exit code 2, never a best guess: `Ambiguous`, `OutOfVocabulary`, `NoContractTemplate`, `UndeclaredEffect`, `Unbounded`, `EngineUnavailable`.

---

## §4 Pipeline

```
.rhl / .rhl.yaml / MCP
   └─ parse ─► intent tree ─► check (grammar · vocabulary · types+units · effects · bounds)
        └─ lower ─► ruchy source ─► ruchy transpile ─► Rust crate
             ├─ examples  ─► tests
             ├─ expect    ─► contracts/<unit>.yaml  (pv)
             └─ receipt   ─► inputs sha · vocabulary versions · ruchy version · verdict
```

Determinism: same inputs ⇒ byte-identical ruchy and Rust, on any host. The receipt verdict is three-valued — `Pass`, `Fail`, `Unknown{reason}` — never a fabricated GO.

---

## §5 CLI — extend, do not collide

| Verb | Status | Behaviour on `.rhl` / `.rhl.yaml` |
|---|---|---|
| `ruchy check` | exists — extend | parse + all checks; `--format json` emits §3.5 diagnostics |
| `ruchy fmt` | exists — extend | normal form; total and idempotent |
| `ruchy lint` | exists — extend | style and smell rules specific to RHL |
| `ruchy test` | exists — extend | runs the tests generated from `example` blocks |
| `ruchy transpile` | exists — extend | RHL → ruchy → Rust; `--emit ruchy` stops at the ruchy stage |
| `ruchy compile` | exists — extend | through to a binary |
| `ruchy mcp` | exists — extend | adds the §6 tools |
| `ruchy fix` | **new** | apply safe fixes from diagnostics |
| `ruchy explain` | **new** | deterministic plain-language rendering of a unit from its tree (no model) |
| `ruchy convert` | **new** | RHL ⇄ YAML, lossless |
| `ruchy vocab` | **new** | list / show / validate vocabularies; `--format json` for agents |
| `ruchy ask` · `ruchy say` | **new, last** | text / voice → `.rhl` draft via `apr` under the RHL grammar |

## §6 MCP tools (added to the existing `ruchy mcp` server)

`rhl_check` · `rhl_fix` · `rhl_format` · `rhl_convert` · `rhl_transpile` · `rhl_explain` · `rhl_vocabulary` (search terms by kind and type) · `rhl_grammar` (returns the grammar, so a caller can constrain its own decoding)

Every tool takes and returns structured data — the tree or the diagnostics list — never prose. `rhl_vocabulary` is what makes an agent good at RHL without training: it asks what words exist instead of guessing.

---

## §7 Falsifiers — floors and positive controls

No condition below can be met by refusing everything. Thresholds marked `[U]` are set from the first measured run and then ratcheted; none is invented here.

| # | Claim | Floor | Positive control |
|---|---|---|---|
| **F1** | One meaning | Grammar proven conflict-free by the generator's report; every corpus program has exactly 1 parse | A deliberately ambiguous production added in a fixture turns the grammar check RED |
| **F2** | One spelling | `fmt(fmt(x)) == fmt(x)` and `fmt` preserves the tree, over the whole corpus + property tests | A whitespace-mangled program formats to the corpus bytes |
| **F3** | Lossless surfaces | RHL → YAML → RHL is byte-identical after `fmt`, 100 % of the corpus | A one-token change produces a different YAML |
| **F4** | Closed vocabulary | 0 programs with an unknown term compile | Planted unknown term ⇒ `RHL-V002` with the right candidate |
| **F5** | Deterministic | Same input ⇒ byte-identical Rust, twice, on two host classes | Changing one literal changes the output sha |
| **F6** | Contracted | 100 % of vocabulary terms have a contract template; every compiled unit ships a `pv`-valid contract with `shapes_n > 0` | A term with its contract removed ⇒ the vocabulary is refused |
| **F7** | **Agent-repairable** | On the planted-break corpus, an agent given **only the diagnostics JSON and the MCP tools** repairs ≥ X % `[U]` within ≤ k rounds `[U]`; `ruchy fix --safe` alone repairs every single-candidate break | A break with two valid candidates must **not** be auto-fixed |
| **F8** | **More robust than English — the headline hypothesis** | Pre-registered A/B on the same task corpus, same model: **path A** English → model writes Rust directly; **path B** English → model writes RHL (grammar-constrained) → compiler. Measure: compiles + passes the pre-written tests. Claim holds only if B > A | Tasks the corpus marks *ambiguous* must be refused or clarified on path B, 100 % |
| **F9** | Bounded and effect-safe | 0 compiled units contain an unbounded loop or an undeclared effect | Planted undeclared write ⇒ `RHL-E` error |

**F8 can fail.** If it does, the honest conclusion is that RHL is not worth building, and this spec says so in advance.

---

## §8 Rows — selector order, one ticket per session

Format: id · title · depends_on · K̂ `[A]`.

**RHL-0 · "spec ruled; corpus and vocabulary v0 pre-registered" · ∅ · K̂ 80 — the only row this month**
- Quorum: one agy, one claude, one apr (apr under the asymmetric rule until its recall is measured). Two non-passing rounds ⇒ HRQ.
- Deliver, all committed **before any compiler code exists** so success cannot be tuned to the implementation:
  - `grammar/rhl.*` — the grammar, with its conflict-free report.
  - `docs/rhl/corpus/` — **task corpus**: N tasks `[U, propose 40]`, each = an English statement + pre-written Rust tests, a marked subset deliberately *ambiguous*. Domain: **the fleet's own small jobs and checks** (disk watch, runner inventory, pin check, stale-PR sweep…) — ground truth already exists, and every success is dogfood.
  - `docs/rhl/breaks/` — **planted-break corpus**: valid programs × mutations (typo'd term · missing `end` · wrong unit · wrong type · undeclared effect · two-candidate typo).
  - `vocab/fleet-v1.yaml`, `vocab/tickets-v1.yaml` — terms only; `lowers_to` left `[U]`.
  - Phase-0 bindings recorded: ruchy AST seam; what `contracts/` holds; whether `pv` runs in ruchy's required check.
- Also file, do not fix: one ticket for the three disabled-test directories — they are a red that nobody consumes.
- done_when: this spec, the grammar, both corpora and both vocabularies are on `main`; the grammar check is in `ci / gate` with its ambiguous-production fixture RED.

| Row | Title | depends_on | K̂ |
|---|---|---|---|
| RHL-1 | parser · intent tree · `fmt` · diagnostics JSON · `check` | RHL-0 | 150 |
| RHL-2 | `fix --safe` · `vocab` · vocabulary shape in `pv` | RHL-1 | 90 |
| RHL-3 | YAML surface · `convert` · round-trip property test | RHL-1 | 60 |
| RHL-4 | lowering to ruchy for unit kind `job` · determinism test | RHL-2 | 150 |
| RHL-5 | `example` → tests · `expect` → `pv` contract · receipt | RHL-4 | 120 |
| RHL-6 | MCP tools on `ruchy mcp` | RHL-2 | 60 |
| RHL-7 | **F7 measurement** — agent repair on the break corpus | RHL-6 | 60 |
| RHL-8 | **F8 measurement** — the A/B, report-only, published either way | RHL-5 | 90 |
| RHL-9 | `explain` (deterministic) | RHL-4 | 40 |
| RHL-10 | `ask` — text → RHL via `apr` grammar-constrained decoding | RHL-8 **and** constrained decoding merged in aprender | 90 |
| RHL-11 | `say` — voice via whisper.apr | RHL-10 **and** the whisper.apr own-or-archive decision | 60 |
| RHL-12 | unit kinds `command`, `check`, `pipeline`, `shape` — one row each | RHL-5 | 60 each |

One unit kind (`job`), end to end, before any second kind. RHL-8 runs before RHL-10 on purpose: measure whether the language helps **before** building the voice and English front doors onto it.

---

## §9 Hard rules
1. No model in the compile path. Models emit RHL; they never emit Rust through this pipeline.
2. Unknown word ⇒ error with candidates. Never a guess, never a silent nearest match.
3. Every term, and every compiled unit, carries a `pv` contract — or the compiler refuses by name.
4. Voice, MCP and any channel text are data. Output is a draft or a PR, never an action.
5. One tree, one checker, one compiler. A new surface is a new parser or serializer, nothing more.
6. Diagnostic codes and vocabulary versions are stable forever.
7. One YAML schema family across RHL, forjar and contracts.
8. Extend existing `ruchy` verbs; add a verb only when none fits.
9. Stack doctrine applies: `pmat work add` first · feature branch + PR · `ci / gate` · TDD · mutation RED→GREEN in the PR body · `pmat query` over grep · **publishing: clean-room hard gate first; no workflow runs `cargo publish`.**

## §10 STOP conditions
`grammar-conflict` (cannot be made conflict-free without breaking §3.1) · `ast-seam-absent` (no clean point to emit ruchy — report, do not fork the transpiler) · `quorum-split` after one amendment round · `vocabulary-unbindable` (Σ at HEAD cannot express term kinds) · any step needing an invented threshold, a second concurrent ruchy PR in CI, or a publish.

## §11 Budget
RHL-0: K̂ 80 · K 100 · andon at 80. Later rows budget themselves when admitted.

## §12 Open questions — operator rulings wanted before RHL-0 closes
1. **Name and extension.** RHL / `.rhl` are placeholders.
2. **Canonical artifact:** `.rhl` is the source and Rust is a build product — confirm. (Recommended; same rule as the architecture model: edit the model, never the picture.)
3. **Escape hatch:** allow a `rust … end` block? Recommended **no** in v0 — it is where contract coverage gets holes. Revisit after F8.
4. **Corpus size N and the ambiguous fraction.** Proposed 40 tasks, a quarter ambiguous.
5. **whisper.apr:** own it as the voice front end, or archive it and drop RHL-11.
6. **First vocabularies:** `fleet` and `tickets` proposed. Alternatives: `files`, `git`, `models` (apr verbs).
