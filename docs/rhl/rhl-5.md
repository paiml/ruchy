# RHL-5a — examples lower to tests, `expect` to a guard, `ruchy test` on `.rhl`

Row RHL-5 (ticket RHLGA-1, phase P5). Spec RHL-001 §3.1 principle 7 ("Examples are tests;
expectations are contracts"), §4, §5 (`ruchy test` row), §9.4 (the plan is a draft).
Code: `src/rhl/examples.rs` (a child of `src/rhl/lower.rs`), `src/rhl/cli.rs` (`test_file`);
tests: `src/rhl/examples_tests.rs` (`rhl5_`), `tests/rhl_test_cli.rs` (`rhl5_cli_`).
The pv contract emission of `expect` and the compile receipt are RHL-5b (below).

## `given` — an example states its facts

`given X is V` sets one field of `struct Facts`:

| `X` | sets |
|---|---|
| a measure applied to literal arguments the job reads (`given disk free of "/" is 90 GB`) | that fact's field |
| a `let` name bound directly to one measure application (`let free be disk free of "/"`, then `given free is 90 GB`) | the field of that measure |

`V` must be a literal of the fact's type. It is scaled by the same unit table as lowering
(`docs/rhl/rhl-4.md`): `90 GB` = `90000000000`, `95 %` = `9500`.

Every fact the job reads must be set by the example's `given` lines. The test builds
`Facts { … }` from them alone, so no example reads the host.

## `then` and `expect` read the plan

`then C` and `expect C` are conditions over the plan `decide` returns. Their names resolve
only to **plan measures**:

| plan measure | value | ruchy |
|---|---|---|
| `ticket count` | the number of `file ticket` actions in the plan | `plan_ticket_count(plan) -> i64` (0 when the job files no ticket) |

The table is `PLAN_MEASURES` in `src/rhl/examples.rs`. Comparisons lower as in `decide`
(`is at most` → `<=`, and so on).

- `expect C` (at the top level of the job) becomes `fun expect_<n>(plan: &Vec<Action>) -> bool`.
  `fun failed_expectation(plan: &Vec<Action>) -> String` gives the text of the first
  failing expect, or `""` when all hold.
- The built job (`ruchy compile` / `ruchy run`): `main` prints the plan and then checks the
  expects. If one fails, it prints `expectation failed: <expect>; nothing applied` to standard
  error and exits 1 **without applying**, even with `--apply` (§9.4).
- An example fails at its first `then` that does not hold. If every `then` holds, it fails
  at the first expect its plan breaks.

## `ruchy test x.rhl` (`.rhl.yaml` too)

`ruchy test` checks the job and lowers it to its test form (`Form::Tests`). That form holds
`decide`, the plan measures, the expects, one `fun example_<n>() -> String` per example,
and a runner `main`. It builds that form the way `ruchy compile` does and runs it with an
empty environment. Output, one line per example in source order:

```
example "gx10 disk watch example" … ok
j.rhl: 1 example, 1 passed, 0 failed
```

A failing example prints `example "<name>" … FAILED (<then or expect>)`. `--format json`
prints `{"file", "examples": [{"name", "ok", "failed"?}], "passed", "failed", "exit"}`.
Exit codes: 0 when every example holds; 1 when one fails, or the file cannot be read or
built; the check's own code when the file does not check clean; 2 for a refusal
(`RHL-L001`, `RHL-X002`).

**A job with no examples** exits 0 and reports `0 examples, 0 passed, 0 failed`. Having no
examples is reported, not treated as a failure. Its expects are still lowered, so a
refusal there is still an error.

**Hermeticity.** The test form has no `observe`, no `apply` and no runtime binding, and it
calls nothing in `vocab/runtime`. So examples of a job whose terms are bound to `[U]` (for
example `runner load of`, `pin status of`) still run. The test program runs with
`env_clear()`. It never reads the host or changes it.

## Refusals

| construct | code |
|---|---|
| `given` with an operator other than `is`, or a non-literal value, or a value of another type | `RHL-L001` |
| `given` over a `let` not bound directly to one measure application, bound more than once, or changed by `set` | `RHL-L001` |
| `given` over a term that is not a fact the job reads (`ticket count`, `disk free of "/data"` when the job reads only `"/"`) | `RHL-L001` |
| the same fact given twice in one example | `RHL-L001` |
| a line other than `given`/`then` in an example | `RHL-L001` |
| `then` or `expect` over anything but a plan measure (a `let`, a fact, a measure) | `RHL-L001` |
| `expect` or `example` inside a block | `RHL-L001` |
| an example that leaves a fact the job reads unset | `RHL-X002` (new, appended to the X family) |

The message names the construct, the fact, or the example.

## Determinism

The test form depends only on the tree and the vocabularies, in source order.
`rhl5_determinism_test_programs_are_byte_identical_and_transpile` lowers every v2 valid
program twice and compares the bytes and the Rust. Its positive control changes one `then`
literal and gets a different sha. All 12 v2 valid programs pass `ruchy test`.

## Known limit

A job that reads no fact has an empty `struct Facts`. ruchy transpiles an empty struct
literal `Facts {}` to `Facts::default()`, which rustc rejects. So an example of such a
job does not build yet. No v2 program has one.

# RHL-5b — the unit contract and the compile receipt

Code: `src/rhl/contract.rs` (+ `contract_render.rs`), `src/rhl/receipt.rs`, `src/rhl/cli.rs`
(`transpile_file`, `compile_file`); tests: `src/rhl/contract_tests.rs`,
`src/rhl/receipt_tests.rs` (`rhl5b_`), `tests/rhl_contract_cli.rs` (`rhl5b_cli_`).
Spec RHL-001 §3.1 principle 7, §4, §7 F6, §9.3.

## `ruchy transpile x.rhl --emit contract [-o out.yaml]`

Checks the job and lowers it as `ruchy test` does (`Form::Tests`), so a refusal there is a
refusal here, then writes one `pv` contract for the job. Every string is a double-quoted
scalar. There is no timestamp: the bytes depend only on the tree (its `fmt` normal form, so
`.rhl` and `.rhl.yaml` give the same contract), the vocabulary files and the source bytes.

| block | content |
|---|---|
| `metadata.unit` | `kind`, `name`, `source_sha256` (of the input file), `runs_on`, `every`, `effects` (the `may` lines), `examples` (names) |
| `metadata.vocabularies` | `name`, `version`, `sha256` of each `use vocabulary` file |
| `metadata.terms` | each vocabulary term the job uses (longest leading run of words, as the checker resolves it; units too): `term`, `vocabulary`, `contract` (its term contract path) |
| `equations.effect_surface` | `effects(apply(plan)) ⊆ {…}`; `invariants`: one `may …` per effect line |
| `equations.expect_<n>` | one per top-level `expect`, `n` as in the generated guard `expect_<n>`: `formula` over the plan-measure table (`ticket count is at most 1` → `plan_ticket_count(plan) ≤ 1`, the generated function's name) and the same formula as its `postconditions` (ensures) |
| `proof_obligations` | a `postcondition` per expect; an `invariant` for the closed effect surface (always present); one per `may` line; one per term (its contract) |
| `falsification_tests` | one per obligation (the guard `expect_<n>`, `ruchy check` RHL-E001, RHL-C001), plus one per example (`example_<n> "<name>"`) |
| `kani_harnesses` | one entry marked NOT APPLICABLE (`pv validate` requires the block) |
| `qa_gate` | `required_tests`: `example_<n>`, `expect_<n>`, `ruchy check` |

A job with no `expect` and no `example` still gets a contract: the effect surface and its
terms. It is never skipped.

**`shapes_n`.** F6 asks for "a `pv`-valid contract with `shapes_n > 0`". The field named
`shapes_n` in `pv extract` counts SHACL node shapes declared by ontology `shapes:` blocks;
it is 0 for every kernel contract (all 144 contracts of provable-contracts, and every
`contracts/rhl-*` term contract). The count used here is the proof obligations `pv status`
reports (`Proof obligations: N`), which is at least 1 for every job because the effect
surface obligation is always emitted. `rhl5b_contract_every_v2_valid_program_validates_with_pv`
asserts `pv validate` exits 0 and N > 0 for all 12 v2 valid programs.

## `ruchy compile x.rhl -o out`

1. lowers the job (`Form::Program`);
2. emits the unit contract and writes `out.contract.yaml` **before** building. If it cannot
   be written, the command refuses with `RHL-C002` (refusal `NoContractTemplate`), builds
   nothing and exits 2 (§9.3);
3. builds `out`;
4. runs the job's examples with `ruchy test`'s runner and writes `out.receipt.json`.

`ruchy run` builds to a temporary directory and writes neither file.

## Receipt schema

```json
{
  "inputs": {
    "source_sha256": "…",
    "vocabularies": [{"name": "fleet", "version": 2, "sha256": "…"}],
    "runtime_files": [{"path": "vocab/runtime/fleet.ruchy", "sha256": "…"}]
  },
  "ruchy_version": "…",
  "rust_sha256": "…",
  "contract_sha256": "…",
  "verdict": "Pass"
}
```

`runtime_files` are the `vocab/runtime/<module>.ruchy` files that the used vocabularies'
`lowers_to` bindings name, sorted. `rust_sha256` is the sha of the Rust the lowered program
transpiles to. No field holds a path to the source or a wall-clock value, so the same inputs
give byte-identical receipts.

## Verdict rules

| examples | verdict |
|---|---|
| every example holds | `"Pass"` |
| one or more fails | `"Fail"` |
| the job has no example | `{"Unknown": "no examples"}` |
| the test program could not be built or run | `{"Unknown": "the examples could not be built or run: <first line>"}` |

There is no `Pass` without an example that ran and held.
