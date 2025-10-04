# Value Type Migration - Remaining Manual Fixes

## Status: PARTIALLY COMPLETE

**Date**: 2025-10-04
**Sprint**: 7 - Phase 4 (Mutation Testing)
**Blocking**: cargo-mutants baseline compilation

## Overview

Automated migration script successfully migrated 25 test files from old Value API to new API.
**8 test files** remain with compilation errors requiring manual fixes.

## Migration Summary

### Completed (Automated)
- ✅ **25 files migrated** via `scripts/migrate_value_types.sh`
- ✅ **3,383 lib/bin tests passing** (no regressions)
- ✅ **Simple one-line patterns** successfully converted:
  - `Value::Array(Rc::new(vec![...]))` → `Value::Array(vec![...].into())`
  - `Value::String(Rc::new("str".to_string()))` → `Value::String(Rc::from("str"))`
  - `Value::Tuple(Rc::new(vec![...]))` → `Value::Tuple(vec![...].into())`

### Remaining (Manual)
- ⚠️ **8 files** with compilation errors
- ⚠️ **Multiline vec![]** patterns not handled by perl regex
- ⚠️ **cargo-mutants** blocked until all tests compile

## Files Requiring Manual Fixes

Run the following command to identify all remaining errors:

```bash
cargo test --no-run 2>&1 | grep "error\[E0308\]" | grep -A 2 "mismatched types"
```

### Common Pattern Needing Manual Fix

**Before** (causes compilation error):
```rust
let val = Value::Array(Rc::new(vec![
    Value::Integer(1),
    Value::Integer(2),
    Value::Integer(3),
]));
```

**After** (correct - add `.into()`):
```rust
let val = Value::Array(vec![
    Value::Integer(1),
    Value::Integer(2),
    Value::Integer(3),
].into());
```

### String Pattern Needing Manual Fix

**Before**:
```rust
let val = Value::String(Rc::new("hello".to_string()));
```

**After**:
```rust
let val = Value::String(Rc::from("hello"));
```

### Tuple Pattern Needing Manual Fix

**Before**:
```rust
let val = Value::Tuple(Rc::new(vec![
    Value::Integer(42),
    Value::Bool(true),
]));
```

**After**:
```rust
let val = Value::Tuple(vec![
    Value::Integer(42),
    Value::Bool(true),
].into());
```

## Manual Fix Protocol

### Step 1: Identify Failing Files
```bash
cargo test --no-run 2>&1 | grep "^   -->" | cut -d: -f1 | sort -u
```

### Step 2: Fix Each File
For each file identified:

1. Open the file in editor
2. Search for patterns: `Value::Array(Rc::new(vec!`
3. Replace `Rc::new(vec![...])` with `vec![...].into()`
4. Search for patterns: `Value::String(Rc::new(`
5. Replace with `Rc::from("string")`
6. Search for patterns: `Value::Tuple(Rc::new(vec!`
7. Replace `Rc::new(vec![...])` with `vec![...].into()`

### Step 3: Verify After Each Fix
```bash
cargo test --no-run --quiet
```

### Step 4: Verify All Tests Pass
```bash
cargo test --lib --bins
```

## Why This Matters

**cargo-mutants** requires a successful baseline build before mutation testing:
```bash
cargo mutants --file src/frontend/parser/expressions.rs
```

This command runs `cargo test --no-run` internally, which compiles **ALL** tests (including integration tests).

**The `additional_cargo_test_args` config only affects test execution, not compilation.**

Therefore, ALL test files must compile successfully before mutation testing can proceed.

## Verification Commands

### Check Compilation Status
```bash
cargo test --no-run 2>&1 | grep -c "error\[E0308\]"
```
- **Expected output**: `0` (when all fixes complete)
- **Current output**: `8` (8 remaining errors)

### Check Test Suite Health
```bash
cargo test --lib --bins --quiet
```
- **Expected**: All 3,383 tests pass
- **Current**: ✅ PASSING

### Verify cargo-mutants Baseline
```bash
cargo mutants --file src/frontend/parser/expressions.rs --check-only
```
- **Expected**: "Baseline build succeeded"
- **Current**: ❌ FAILED (8 compilation errors)

## Technical Background

### Why the Migration Was Needed

**Old API** (memory inefficient):
```rust
pub enum Value {
    Array(Rc<Vec<Value>>),    // Extra indirection
    String(Rc<String>),       // Extra heap allocation
    Tuple(Rc<Vec<Value>>),    // Extra indirection
}
```

**New API** (memory efficient):
```rust
pub enum Value {
    Array(Rc<[Value]>),       // Direct slice
    String(Rc<str>),          // Direct str
    Tuple(Rc<[Value]>),       // Direct slice
}
```

**Memory savings**:
- `Rc<Vec<T>>`: 16 bytes (Rc) + 24 bytes (Vec) = 40 bytes overhead
- `Rc<[T]>`: 16 bytes (Rc) = 16 bytes overhead
- **Savings**: 24 bytes per array/tuple instance

### Why Automated Migration Had Limitations

**Perl regex** used in migration script:
```bash
perl -i -pe 's/Value::Array\(Rc::new\(vec!\[(.*?)\]\)\)/Value::Array(vec![$1].into())/g'
```

**Limitation**: The `.*?` regex is **non-greedy** and only matches within a single line.

**Multiline patterns** like:
```rust
vec![
    item1,
    item2,
]
```

Are **not matched** by single-line regex patterns.

**Solution**: Manual fixes required for multiline patterns.

## Next Steps

1. ✅ **Automated migration complete** (25 files)
2. ⏳ **Manual fixes needed** (8 files) ← **YOU ARE HERE**
3. ⏳ **Verify baseline**: `cargo test --no-run` succeeds
4. ⏳ **Resume mutation testing**: `cargo mutants` on parser/transpiler modules
5. ⏳ **Achieve ≥90% mutation kill rate** (Sprint 7 Phase 4 goal)

## Time Estimate

- **Per-file fix time**: 2-5 minutes
- **Total estimated time**: 15-40 minutes
- **Verification time**: 5 minutes
- **Total**: ~20-45 minutes to unblock mutation testing

## Success Criteria

- [ ] All 8 files fixed
- [ ] `cargo test --no-run` succeeds (no compilation errors)
- [ ] `cargo test --lib --bins` still passes (3,383 tests)
- [ ] `cargo mutants --file <any-file> --check-only` shows "Baseline build succeeded"
- [ ] Mutation testing can proceed on parser/transpiler modules

## References

- Migration script: `scripts/migrate_value_types.sh`
- cargo-mutants config: `.cargo/mutants.toml`
- Sprint 7 roadmap: `docs/execution/roadmap.md` (WASM-015)
- Value type definition: `src/runtime/value.rs`
