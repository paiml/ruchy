# TRANSPILER-155 — implementation receipt (PR #249, closes #155)

## Row (docs/roadmaps/roadmap.yaml, id TRANSPILER-155)

1. A cast of a compound operand keeps its parentheses:
   `src/backend/transpiler/dispatcher_helpers/misc.rs` `transpile_type_cast`
   and `cast_operand_is_atomic`.
2. Atomic operands (identifier, literal, call, method call, field access,
   index access, cast) get no added parentheses: test 05 covers each of the
   seven kinds in one program and asserts the exact emitted cast. A cast is
   atomic because a cast chain `x as i64 as f64` is left-associative Rust;
   the row's acceptance line says so.
3. `use std::time::Instant`, `use std::time::{Instant, Duration}` emit the
   Rust `use`: `src/backend/transpiler/std_imports.rs`
   `transpile_std_time_import_with_path`. `use std::time;` still emits the
   ruchy `time` helper module (test 08). The other public std::time items
   (`SystemTime`, `SystemTimeError`, `TryFromFloatSecsError`, `UNIX_EPOCH`)
   were dropped the same way and are on the row too: test 10 compiles and
   runs `use std::time::{SystemTime, UNIX_EPOCH}` (it failed with E0433/E0425
   on a main build).
4. The issue's benchmark compiles and prints the correct checksum (test 09;
   the expected value is computed in Rust by the test itself).

## Evidence

- `tests/issue_155_cast_parens_std_time.rs`: 10 tests, all pass. Test 03 uses
  the row's exact expression `(idx + 1) as f64 / 2.0` (idx = 8, so the
  result is fractional: the interpreter on main prints whole floats as `4.0`).
- Hand mutations: reverting misc.rs fails 4 tests; reverting std_imports.rs
  fails 3.
- Lib suite 20831 pass; fmt clean; `clippy --all-targets -D warnings` clean.
- 78 integration targets matching cast|import|std_time|time_, run with
  `--no-fail-fast` with and without the fix: the same two pre-existing
  failures (`issue_114_usize_bench_008_end_to_end`,
  `bug_syntax_001_attribute_error_message`) on both; no new failure.

## Out of scope (recorded in the row's notes)

- The interpreter has no `std::time::Instant`, so tests 06, 07 and 09 check
  compiled output only; tests 01–04 check compiled and interpreted output.

## Quorum rounds

- Round 1 at `8bbc23c5`: gemini-3.1-pro FAIL, both flash lanes PASS. The
  findings were scope drift between the row, the tests and the code: TypeCast
  was atomic but not on the row, the extra std::time items were not on the
  row, test 05 covered one operand kind, and test 03 used `idx + 2` rather
  than the row's `idx + 1`. All four are fixed in the next commit: the row
  names cast and the extra items, test 05 covers all seven kinds, test 03
  uses the row's expression, and test 10 was added.
