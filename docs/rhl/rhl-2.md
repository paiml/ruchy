# RHL-2 — `ruchy fix --safe`, `ruchy vocab`, and the vocabulary shape

Roadmap row RHL-2; spec RHL-001 §3.4, §3.5 and §5; plan `docs/rhl/rhlga-1-plan.md` ruling Q5.
The code is `src/rhl/fix.rs`, `src/rhl/vocab_cli.rs` and `src/rhl/vocab.rs`. The binary
only prints the result and exits with its code (`src/bin/handlers/rhl_handler.rs`).

## `ruchy fix --safe <file.rhl>...`

`ruchy check` marks a fix `safe` when it is the only candidate and the fixed program has
no type or unit error on the line it edits. That is RHL-1's local rule, and `check` keeps
it (ruling Q5). `fix --safe` adds the global guard:

1. Check the file. The number of safe fixes it proposes, N, bounds the rounds.
2. In each round, try every safe fix of the current text, last position first. After each
   fix, check the whole file again. If the file's error count rose, revert that fix and do
   not try it again. Otherwise keep it.
3. Stop when a round keeps no fix, or after N rounds.
4. Write the file in place if it changed. Print one `fixed[CODE]` line for each fix kept
   and one `reverted[CODE]` line for each fix reverted, then the check of the result.
   `--format json` prints the check report object with three more fields: `applied`,
   `reverted` and `rounds`.

A fix is never applied when its diagnostic has two or more candidates, even if it is
marked safe. The checker never marks such a fix safe (RHL-V003), and `fix` checks again.
Running `fix --safe` a second time changes nothing: every fix left is either unsafe or
already reverted once, and a reverted fix is reverted again.

The exit codes are the same as for `ruchy check` on the result: 0 clean, 1 errors
remain, 2 a refusal remains. So a file whose only error is a two-candidate typo exits 2
unchanged. `ruchy fix` without `--safe` is refused (exit 2): v0 has only safe mode. A
file that is not `.rhl` gives exit 1.

Measured on the v2 break corpus: each of the 12 `typo-term` breaks becomes, byte for
byte, its valid base program, and that program checks clean. None of the 12
`two-candidate-typo` breaks is changed. The case that needs the global guard is
`let t be 10 hou` compared with a Size twice. `hou` → `hour` is safe at the edit site,
but it turns one error into two RHL-T001s, so `fix --safe` reverts it.

## `ruchy vocab`

The vocabulary root is `--root`, or else the nearest directory, starting at the current
one, that holds `vocab/`. This is the same rule `ruchy check` uses (`find_root`).

| Command | Prints | Exit |
|---|---|---|
| `vocab list [--format json]` | every `vocab/<name>-v<N>.yaml`: name, version, term count, file | 0, or 1 if a file does not load |
| `vocab show <name> <version> [--format json]` | every term: kind, `takes`, `gives`, effect, attributes, `instances_from`, contract | 0, or 2 with RHL-V004 |
| `vocab validate <file.yaml> \| <name> <version> [--format json]` | the report `{file, vocabulary, verdict, diagnostics}` | 0 pass, 1 error, 2 refusal |

The version can be written `v2` or `2`.

## The vocabulary shape

`contracts/rhl-vocabulary-shape-v1.yaml` states the shape. There is one implementation,
the loader in `src/rhl/vocab.rs`, and both `ruchy check` and `ruchy vocab` use it:

| Rule | Where | Code |
|---|---|---|
| `vocabulary`, `version`, `terms`; each term has `term` and `kind` | serde (`parse_file`) | RHL-V004 |
| `kind` is one of noun, measure, action, unit, entity | serde | RHL-V004 |
| every `takes` parameter and every attribute has a name and a type | serde + `shape_violations` | RHL-V004 |
| an entity has `instances_from` | `shape_violations` | RHL-V004 |
| a unit has `gives` | `shape_violations` | RHL-V004 |
| every term's `contract` exists under the root | `missing_contracts` | RHL-C001 |

A file that breaks the shape cannot be loaded, so a program that uses it gets RHL-V004
from `ruchy check`. `vocab validate` reports each violation as its own finding.
No new diagnostic code was added.

## Ruling on RHL-16's open question

An unknown attribute that is close to no declared attribute is RHL-V001, and the catalogue
defines V001 as "no candidate". So the diagnostic has no candidates. The declared
attribute names go in `expected` instead:
`{"kind": "attribute", "of": "file ticket", "names": ["title", "label"]}`. An attribute near
miss (V002/V003) keeps its candidates and gets the same `expected`.
