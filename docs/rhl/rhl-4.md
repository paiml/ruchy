# RHL-4a — lowering a `job` to ruchy, and `.rhl` through `ruchy transpile`

Row RHL-4 (ticket RHLGA-1, phase P4). Ruling Q4 of `docs/rhl/rhlga-1-plan.md` §5;
spec RHL-001 §4 (pipeline, determinism), §5 (CLI), §9.1 (no model in the compile path);
binding B1 of `docs/rhl/phase0-bindings.md`: RHL emits ruchy **source text**, which
ruchy's public `Parser` and `Transpiler` turn into Rust. Lowering never builds a ruchy
`Expr` and never touches the transpiler. Code: `src/rhl/lower.rs`; tests:
`src/rhl/lower_tests.rs` (`rhl4_`), `tests/rhl_lower_cli.rs`.

## Pipeline

```
.rhl / .rhl.yaml ─► check (must pass, exit 0) ─► lower ─► ruchy source ─► ruchy Parser + Transpiler ─► Rust
```

A `.rhl.yaml` file is checked as YAML and lowered from its RHL normal form, so both
surfaces lower to the same bytes (test `rhl4_yaml_surface_lowers_to_the_same_ruchy`).
Lowering runs only on a program that checks clean; otherwise the check's report is the
answer and its exit code is the process's.

## CLI

| Command | Behaviour |
|---|---|
| `ruchy transpile x.rhl --emit ruchy [-o out]` | prints (or writes) the lowered ruchy source |
| `ruchy transpile x.rhl [--emit rust] [-o out]` | lowers, then the same calls `ruchy transpile x.ruchy` makes (`transpile_to_program`, `prettyplease`); a test holds the two outputs byte-equal |
| `ruchy transpile x.rhl.yaml …` | the same, from the YAML surface |
| `ruchy compile x.rhl` | declined, exit 2: the observe/apply bindings land in RHL-4b |

Exit codes: 0 success; the check's code (1 or 2) when the program does not check clean;
2 for `RHL-L001` or an unknown `--emit`; 1 when a file cannot be read or written.
Diagnostics go to standard error; standard output is only code. `--emit` on a non-RHL
file is an error.

## Lowering rules (construct → ruchy)

| RHL | ruchy |
|---|---|
| a measure with an effect, applied to literal arguments | a field of `struct Facts`, one per distinct (term, arguments) pair, named `<term>_<n>` in order of first use, with a `// Facts.<field>: <term> <args> (<type>)` comment; read as `facts.<field>` (`.clone()` for Text) |
| an action term | a struct variant of `enum Action` named in PascalCase (`file ticket` → `FileTicket`); fields = the term's `takes`, then its `attributes` |
| action statement | `plan.push(Action::V { … })`; the target after `in` fills the parameter named after its entity; a `with` line sets an attribute; an unset attribute is the type's zero value (`String::new()`, `0`, `false`) |
| `let x be c` | `let v_x: T = …`; `let mut` when some `set` names `x` |
| `set x to c` | `v_x = …` |
| `when c … otherwise … end` | `if … { … } else { … }`; an empty body is `()` (an empty `{ }` is an object literal in ruchy) |
| `repeat at most N times [until c] … end` | `let mut repeat_<n>: i64 = 0` / `while repeat_<n> < N { [if c { break }] … repeat_<n> = repeat_<n> + 1 }`; `until` is tested before each round |
| `is` · `is not` · `is below` · `is above` · `is at least` · `is at most` | `==` · `!=` · `<` · `>` · `>=` · `<=` |
| `a contains b` | `a.contains(b)` (`&b` for a non-literal) |
| `and` · `or` | `&&` · `||` (same precedence order; no parentheses needed) |
| `not c` | pushed into `c`: a comparison is inverted (`not x is below y` → `x >= y`), `not not c` → `c`, a Bool name → `!v_x`, `not a contains b` → `!a.contains(b)` |
| `runs on` · `every` · `may` (top level) | the leading comment block |
| `expect` · `example` | not lowered into `decide` (RHL-5 owns them); the header says so |

Names: a `let` name becomes `v_<words>`; a field name that is a ruchy or Rust keyword
gets `_` appended. Strings are escaped (`\\`, `\n`, `\r`, `\t`; RHL strings hold no `"`).
No closures, no maps, no `unsafe`.

## Types and units

| RHL type | ruchy | base unit |
|---|---|---|
| Size | `i64` | bytes: B 1, KB 10^3, MB 10^6, GB 10^9, TB 10^12, KiB 2^10, MiB 2^20, GiB 2^30, TiB 2^40 |
| Duration | `i64` | seconds: second(s) 1, minute(s) 60, hour(s) 3600, day(s) 86400 |
| Percent | `i64` | hundredths of a percent: `90 %` = 9000 |
| Count | `i64` | a bare number, as written |
| Text | `String` | — |
| Bool | `bool` | — |

A vocabulary names a unit term and the type it gives; the factor comes from the built-in
table above (the v2 vocabularies declare only `GB` and `hour`). A unit that is not in the
table for its type, or a quantity that does not fit in `i64`, is `RHL-L001`.

## Refusals — `RHL-L001` (construct not lowerable in v0)

One new code, appended to `src/rhl/codes.rs` and `docs/rhl/diagnostic-codes.md` in a
new family `RHL-L` (lowering). Lowering declines rather than guesses:

- a unit other than `job` (RHL-12 owns them), a file with no unit or more than one;
- `give back` (a job gives back its plan, not a value), `stop with` (`decide` has no early
  stop with a reason), `wait up to` (waiting belongs to running a job, RHL-4b);
- `for each` and `is one of` (no v2 term gives a collection or list; the checker already
  refuses both, lowering refuses them again on an unchecked tree);
- a noun, or a measure with no effect such as `ticket count` (read from the run's own
  plan, not the world, so not a fact);
- a measure applied to a non-literal (facts are read before `decide` runs);
- a `let` used outside the block that binds it (v0 keeps ruchy block scope);
- a parameter or attribute type with no v0 form (`Ticket`, an entity);
- `runs on` / `every` / `may` inside a nested block.

## Determinism (RHL-001 F5)

The output depends only on the tree and the vocabularies, in source order: no maps are
iterated, nothing is timed or read from the host, and the file name is not in the output.
`rhl4_determinism_two_lowerings_are_byte_identical` lowers and transpiles every v2 valid
program twice and compares bytes and sha256; the positive control
`rhl4_determinism_positive_control_one_literal_changes_the_sha` changes `100 GB` to
`101 GB` and requires a different sha of the Rust.

## The §3.2 example, lowered

`docs/rhl/breaks/v2/valid/01-gx10-disk-watch.rhl`, `ruchy transpile --emit ruchy`:

```
// Lowered from RHL by ruchy (RHL-4). Regenerate it from the RHL source; do not edit.
// job "gx10 disk watch"
// use vocabulary fleet v2
// use vocabulary tickets v2
// runs on host gx10
// every 1 hour
// may read disk
// may write tickets
// Not lowered into decide: expect and example (RHL-5).

// Facts.disk_free_of_0: disk free of "/" (Size, bytes)
struct Facts {
    disk_free_of_0: i64,
}

enum Action {
    FileTicket { repo: String, title: String, label: String },
}

fun decide(facts: Facts) -> Vec<Action> {
    let mut plan: Vec<Action> = Vec::new()
    let v_free: i64 = facts.disk_free_of_0
    if v_free < 100000000000 {
        plan.push(Action::FileTicket { repo: "paiml/infra".to_string(), title: "gx10 disk watch".to_string(), label: "fleet".to_string() })
    }
    plan
}
```

`rhl4_gx10_examples_mean_one_ticket_then_none` appends a `main` that calls `decide` with
`Facts { disk_free_of_0: 90 GB }` and `Facts { disk_free_of_0: 400 GB }`, compiles it with
`rustc`, and reads `1 0` — the meaning of §3.2's two examples (RHL-5 will generate these).

## Transpiler limitations met (reported, not patched — RHL-001 §10)

- `!(a == b)` transpiles to `!a == b`, and `!(b && x)` to `!b && x`: the parentheses of a
  unary `!` are dropped, which changes the meaning or fails to compile. Lowering therefore
  never emits `!( … )` (see the `not` rule).
- `(a < 1) == false` transpiles to a chained comparison that `syn` rejects.
- An empty block `{ }` is an empty object literal (a `BTreeMap`), so `if c { }` without
  `else` does not type-check; lowering emits `()`.
