# RHL diagnostic codes

Spec RHL-001 §3.5. Codes are **stable forever** (§9 rule 6): a code is never
renumbered, reused or given a new meaning. New codes are appended.

This table is rendered from `src/rhl/codes.rs`, which is the single source. The lib test
`test_rhl_1_codes_doc_lists_exactly_the_catalogue` fails if the two differ. A code enters
the catalogue only when a checker emits it and a test observes it (RHL-1 plan, decision
D3).

The **refusal** column names the §3.5 refusal a code is an instance of. When any
diagnostic carries a refusal, `ruchy check` exits **2**: the compiler declines to guess.
When there are errors but no refusal, it exits **1**. With no errors it exits **0**.

| Code | Meaning | Refusal |
|---|---|---|
| `RHL-P001` | missing `end` | — |
| `RHL-P002` | unexpected token | — |
| `RHL-P003` | invalid token | — |
| `RHL-V001` | unknown word, no candidate | `OutOfVocabulary` |
| `RHL-V002` | unknown word, one candidate | `OutOfVocabulary` |
| `RHL-V003` | unknown word, several candidates | `Ambiguous` |
| `RHL-V004` | unknown vocabulary | `OutOfVocabulary` |
| `RHL-T001` | wrong unit | — |
| `RHL-T002` | wrong type | — |
| `RHL-T003` | bare number | — |
| `RHL-E001` | undeclared effect | `UndeclaredEffect` |
| `RHL-B001` | not a finite collection | `Unbounded` |
| `RHL-C001` | no contract template | `NoContractTemplate` |
| `RHL-X001` | `given`/`then` outside `example` | — |

Six codes were pre-registered by RHL-0 in `docs/rhl/breaks/planted/*/*/break.yaml`
before any checker existed. Their meaning is fixed by that data: `RHL-P001`,
`RHL-V002`, `RHL-V003`, `RHL-T001`, `RHL-T002` and `RHL-E001`.

## The `expected` set of an unknown attribute

An unknown attribute in the `with` block of an action that declares `attributes:`
(RHL-16) has `expected` = `{"kind": "attribute", "of": "<action>", "names": [<declared
attributes>]}`. When no declared attribute is near, the code is `RHL-V001`, which has no
candidates. The declared names are listed in `expected.names` and not as candidates
(RHL-2).

## Diagnostics of the YAML surface `.rhl.yaml`

RHL-3 adds no code. A `.rhl.yaml` text that is not a program tree (not YAML, not the
tree's shape, an unknown key, a local YAML tag, no `rhl: 1`, or a tree the grammar could not produce) is
reported as `RHL-P001`, the parse family, at the YAML's line and column, or at line 0
when the problem has no one place. `ruchy convert` exits 2 on it; `ruchy check` gives
the verdict `fail` and exits 1, as for a `.rhl` parse failure. Every other diagnostic of
a `.rhl.yaml` file is the checker's, with the code it has for the `.rhl` form, a span of
line 0, column 0 (unknown), and no fixes. See `docs/rhl/rhl-3.md`.
