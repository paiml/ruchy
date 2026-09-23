# RHL-5a — examples lower to tests, `expect` to a guard, `ruchy test` on `.rhl`

Row RHL-5 (ticket RHLGA-1, phase P5). Spec RHL-001 §3.1 principle 7 ("Examples are tests;
expectations are contracts"), §4, §5 (`ruchy test` row), §9.4 (the plan is a draft).
Code: `src/rhl/examples.rs` (a child of `src/rhl/lower.rs`), `src/rhl/cli.rs` (`test_file`);
tests: `src/rhl/examples_tests.rs` (`rhl5_`), `tests/rhl_test_cli.rs` (`rhl5_cli_`).
The pv contract emission of `expect` is RHL-5b and is not done here.

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
