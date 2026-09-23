# RHL-16 — vocabulary and break corpus v2

Roadmap row RHL-16; plan `docs/rhl/rhlga-1-plan.md` §3 P1. Spec RHL-001 §3.4 freezes v1:
`vocab/*-v1.yaml`, `contracts/rhl-*-v1.yaml`, `docs/rhl/breaks/{valid,planted}/` and
`docs/rhl/corpus/` are byte-identical before and after this row. v2 lives beside them.

## Why v2

RHL-1's checker measured three defects in the pre-registered data (Amendment A3; RHL-1 plan M3–M5):

1. All 12 valid programs used `ticket count`, `title` and `label`. None is a v1 term: RHL-V001 three times per program.
2. Programs 03/08 bound `free` to `runner load of` (Percent, `read runner`), 04/09 to `pin status of` (Text, `read host`) and 05/10 to `tickets filed in` (Count, `read tickets`). All six compared it with a GB quantity and declared only `read disk`, `write tickets` and `call tickets`.
3. So wrong-unit/04 and /09 gave RHL-T002 where RHL-T001 was expected. Those two are the v1 `KNOWN_CORPUS_DEFECTS`. Six more breaks repeated a code their base already had on the mutated line.

## What changed in the vocabularies

| File | Change |
|---|---|
| `vocab/fleet-v2.yaml` | Every v1 term, unchanged: same kind, `takes`, `gives`, `effect` and `instances_from`. Each term points to its own `contracts/rhl-fleet-<term>-v2.yaml`. |
| `vocab/tickets-v2.yaml` | Every v1 term, unchanged, each with a v2 contract. **`file ticket`** gains `attributes: [{name: title, type: Text}, {name: label, type: Text}]`. **`ticket count`** is new: a measure that gives Count and has **no effect**. |

`lowers_to: "[U]"` stays on every term (RHL-4 binds it), and so does `sigma_entity: "[U]"`.
The v2 contracts copy the v1 template's shape: version `2.0.0`, IDs suffixed `-V2`. Each
cites the `test_rhl16_*` tests that check v2.

### The attribute rule (new)

For an action that declares `attributes:`, each action-shaped line of its `with … end` block
assigns an attribute. The line is `<attribute> <value>`:

- The name must be one of the action's declared attributes. If it is not, the code depends on how close the name is to a declared one, as for any unknown word (plan D4). With one near candidate it is RHL-V002, whose fix is safe (`titl` → `title`). With several it is RHL-V003. With none it is **RHL-V001**. Because the attribute namespace is closed and small, that V001 lists every declared attribute as a candidate, with its distance, and offers no fix.
- The value must be exactly one literal of the attribute's type. Otherwise the checker reports RHL-T002: `title 5`, `title 5 GB`, `title "a" "b"` or a bare `title`.
- An attribute line takes no `in` target and no nested `with` block. Either one gives RHL-T002.
- Any other statement in the block (`let`, `when`, …) is checked as an ordinary statement.

An action without `attributes:` keeps the v1 behaviour: its block is checked as ordinary
statements. That covers every v1 action and v2's `label ticket`. The v1 harness in
`src/rhl/check_tests.rs` still gets exactly the codes RHL-1 recorded.

### The no-effect measure rule (new)

`ticket count` is the number of tickets the run files. RHL-4 reads it from the run's own
plan, not from the world. So it has no `effect:`, and using it needs no `may` line. The
checker already recorded effects only for terms that declare one, so this needed no code
change. `test_rhl16_measure_without_effect_needs_no_may_line` covers it.

No diagnostic code was added. `docs/rhl/diagnostic-codes.md` is unchanged.

## The v2 corpus — `docs/rhl/breaks/v2/`

Every valid program starts `use vocabulary fleet v2` / `use vocabulary tickets v2`. It
declares exactly the effects it uses: one `read` for its measure, plus `write tickets` for
`file ticket`. v1's unused `may call tickets` is dropped. Each program keeps its job name,
host, schedule, repo, ticket title and label. The `given` line sets the `let` name and the
`then` line asserts `ticket count is 1`.

| # | Measure | v1 comparison | v2 comparison | v2 `given` |
|---|---|---|---|---|
| 01, 06, 11 | `disk free of` (Size) | `free is below N GB` | unchanged | unchanged |
| 02, 07, 12 | `disk usage of` (Size) | `free is below N GB` | `used is above N GB` (high usage files the ticket) | `used is 95 / 80 / 65 GB` |
| 03, 08 | `runner load of` (Percent) | `free is below 90 / 85 GB` | `load is above 90 / 85 %` | `load is 95 / 90 %` |
| 04, 09 | `pin status of` (Text) | `free is below 1 GB` | `status is not "pinned"` | `status is "unpinned"` |
| 05, 10 | `tickets filed in` (Count) | `free is below 5 / 3 GB` | `filed is above 5 / 3` | `filed is 7 / 4` |

The `let` name now says what it holds (`free`, `used`, `load`, `status`, `filed`). Every
v2 valid program checks with no diagnostic. Its only notes are the unverified instances
of `host` and `repo` (Amendment A3), because this repository has no `machines/` or
`org/repos/`. Every program is also in `ruchy fmt` normal form.

### Planted breaks

There are 6 classes × 12 programs. Each `break.yaml` keeps v1's schema (`base`, `class`,
`expected_code`, `safe_fix`, plus `candidates` for V003), and its base is the v2 valid
program.

| Class | Mutation | Code |
|---|---|---|
| missing-end | delete the `end` of the `when` block | RHL-P001 |
| two-candidate-typo | `runs on host` → `runs on hosr` (candidates `host`, `hour`) | RHL-V003 |
| typo-term | the measure on the `let` line (`disk fre of`, `disk usge of`, `runer load of`, `pin staus of`, `ticket filed in`) | RHL-V002, safe fix |
| undeclared-effect | delete `may write tickets` | RHL-E001 |
| wrong-type | the `when` right-hand side becomes `"low"`, or `1` for the Text comparisons of 04/09 | RHL-T002 |
| wrong-unit | the `when` threshold's unit becomes `hour`. For 04/09, whose `when` compares Text, the mutation is `every 1 hour` → `every 1 GB` | RHL-T001 |

The substitution classes yield their code on the mutated line. Every code is new relative
to its base, because the base checks clean. The deletion classes are held as in v1: the
code occurs more often than in the base. v2 has **no** known-defect exceptions:
`KNOWN_CORPUS_DEFECTS_V2` is empty and a test asserts it. The v1 list of two stays as it was.

## Manifest

`docs/rhl/PREREGISTRATION.sha256` was regenerated with the command in its header. The diff
only adds lines: 173 new hashes (156 corpus files, 2 vocabularies, 15 contracts) and the
`# files:` count. No v1 hash changed.
