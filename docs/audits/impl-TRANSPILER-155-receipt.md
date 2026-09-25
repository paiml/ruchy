# TRANSPILER-155 — implementation receipt (PR #249, closes #155)

## Row (docs/roadmaps/roadmap.yaml, id TRANSPILER-155)

1. A cast of a compound operand keeps its parentheses:
   `src/backend/transpiler/dispatcher_helpers/misc.rs` `transpile_type_cast`
   and `cast_operand_is_atomic`.
2. Atomic operands (identifier, literal, call, method call, field access,
   index access, cast) get no added parentheses: test 05.
3. `use std::time::Instant`, `use std::time::{Instant, Duration}` emit the
   Rust `use`: `src/backend/transpiler/std_imports.rs`
   `transpile_std_time_import_with_path`. `use std::time;` still emits the
   ruchy `time` helper module (test 08).
4. The issue's benchmark compiles and prints the correct checksum (test 09;
   the expected value is computed in Rust by the test itself).

## Evidence

- `tests/issue_155_cast_parens_std_time.rs`: 9 tests, RED before the fix
  (6 fail on main: 01, 02, 03, 06, 07, 09), all pass after.
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
