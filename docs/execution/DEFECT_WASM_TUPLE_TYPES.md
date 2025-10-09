# DEFECT-WASM-TUPLE-TYPES: WASM tuple compilation fails with mixed types

**Ticket**: DEFECT-WASM-TUPLE-TYPES
**Created**: 2025-10-09
**Severity**: P0 - Advertised feature completely broken
**Status**: 🔴 OPEN

---

## Problem Statement

**What**: WASM compilation fails for tuples containing mixed types (int + float)
**Where**: `src/backend/wasm/mod.rs` - `lower_tuple()` and `lower_field_access()`
**Impact**: Tuples with mixed types don't compile to WASM (interpreter works fine)

---

## Reproduction

### Failing Example
```ruchy
let x = (1, 3.0)
println(x.0)  // int
println(x.1)  // float
```

**Error**: `✗ WASM validation failed: type mismatch: expected i32, found f32`

### Working Example (Interpreter)
```bash
$ cargo run --bin ruchy -- run /tmp/test.ruchy
(1, 3.0)
1
3.0
```

---

## Root Cause Analysis (Five Whys)

1. **Why does WASM compilation fail?**
   → Type mismatch: expected i32, found f32 on stack

2. **Why is there a type mismatch?**
   → `lower_tuple()` always uses `I32Store` for all elements

3. **Why does it always use I32Store?**
   → Code at line 1128 hard-codes `Instruction::I32Store` regardless of element type

4. **Why wasn't this caught earlier?**
   → Tests only used uniform-type tuples like `(1, 2)` not mixed types like `(1, 3.0)`

5. **Why does the interpreter work?**
   → Interpreter is dynamically typed, WASM is statically typed

**ROOT CAUSE**: `lower_tuple()` stores all elements as i32, `lower_field_access()` loads all elements as i32. Mixed types require type-specific store/load instructions.

---

## Investigation Findings

### Current Code (src/backend/wasm/mod.rs:1128-1132)
```rust
// Store at address + offset
instructions.push(Instruction::I32Store(wasm_encoder::MemArg {
    offset: offset as u64,
    align: 2,
    memory_index: 0,
}));
```

**Problem**: Always uses `I32Store`, even for floats

### Required Fix
- Detect element type (i32, f32, f64)
- Use appropriate store instruction: `I32Store`, `F32Store`, `F64Store`
- Use appropriate load instruction when accessing fields
- Handle addresses (strings, arrays) correctly

---

## Test Plan (EXTREME TDD)

### Unit Tests (RED → GREEN → REFACTOR)

1. **test_wasm_tuple_int_float** - Tuple with (int, float)
2. **test_wasm_tuple_float_int** - Tuple with (float, int)
3. **test_wasm_tuple_all_floats** - Tuple with all floats
4. **test_wasm_tuple_string_int** - Tuple with (string, int)
5. **test_wasm_tuple_nested** - Nested tuple with mixed types

### Property Tests (10,000 cases)

1. **prop_wasm_tuple_type_preservation** - All type combinations preserve values
2. **prop_wasm_tuple_access_correct** - Field access returns correct types

---

## Fix Strategy

### Step 1: Write Failing Tests (RED)
Create test file: `tests/defect_wasm_tuple_types.rs`

### Step 2: Implement Type Detection Helper
Add function to determine WASM type from AST expression

### Step 3: Fix lower_tuple (GREEN)
- Detect each element's type
- Use correct store instruction per type
- Store type metadata for tuple (for later access)

### Step 4: Fix lower_field_access (GREEN)
- Look up tuple element type
- Use correct load instruction per type

### Step 5: Refactor (REFACTOR)
- Ensure complexity ≤10
- Run PMAT quality gates

---

## Success Criteria

- ✅ All 5 unit tests pass
- ✅ Property tests pass (10K cases each)
- ✅ PMAT TDG score ≥ A- (85+)
- ✅ Cyclomatic complexity ≤10
- ✅ P0 tests 15/15 passing (zero regressions)
- ✅ `examples/lang_comp/06-data-structures/03_tuples.ruchy` compiles to WASM

---

**Generated**: 2025-10-09
**Ticket**: DEFECT-WASM-TUPLE-TYPES
**Priority**: P0
**Estimate**: 3-4 hours (type system + WASM lowering + tests)
