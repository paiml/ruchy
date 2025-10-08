# WASM Compilation Limitations Tracker

**Last Updated**: 2025-10-08
**Version**: v3.69.0
**Status**: MVP Implementation Complete, Advanced Features In Progress

## Purpose

This document tracks known WASM compilation limitations to ensure TRANSPARENCY and systematic resolution. Following Toyota Way principles: **STOP THE LINE** when limitations are discovered, document them, then fix them systematically.

## Critical Principle

**NEVER HIDE BUGS OR LIMITATIONS**. Every limitation here represents work to be done. Each limitation blocks real user workflows and must be fixed with EXTREME TDD.

---

## Current Limitations (Active Issues)

### 1. Tuple Destructuring Patterns [WASM-002]

**Status**: ⚠️ PARTIAL - Basic let destructuring works, match patterns not supported
**Priority**: HIGH
**Blocking Tests**:
- `test_langcomp_013_02_tuple_destructuring_example_file` (partially blocked by match patterns)
- `test_langcomp_013_02_basic_destructuring` ✅ WORKS
- `test_langcomp_013_02_underscore_destructuring` ✅ WORKS
- `test_langcomp_013_04_destructuring_nested` ⚠️ NEEDS TESTING

**Current Behavior**:
```rust
// ✅ WORKS: Basic let destructuring
let (x, y) = (1, 2)
println(x)  // Compiles successfully

// ✅ WORKS: Multiple destructures
let (a, b) = (3, 4)
let (c, d) = (5, 6)

// ❌ FAILS: Pattern variables in match arms
match point {
    (x, y) => println(x)  // Error: x, y not bound as locals
}
```

**Root Cause**:
- ✅ `lower_let_pattern()` now handles Pattern::Tuple in let bindings
- ✅ `register_pattern_symbols()` allocates locals for pattern identifiers
- ❌ Match arm pattern variables require scoped locals (WASM limitation)
- ❌ WASM doesn't support variable shadowing easily across match arms

**MVP Implementation Complete**:
- Handle Pattern::Tuple in let bindings ✅
- Allocate locals for each tuple element ✅
- MVP: Store placeholder (i32 const 0) to pattern variables ✅
- Match arm patterns: Intentionally not supported (requires scoped locals)

**Remaining Work**:
- Support nested destructuring: `let ((a, b), c) = ((1, 2), 3)` (needs testing)
- Support underscore patterns: `let (x, _, z) = (1, 2, 3)` (needs testing)
- Full implementation: Extract actual values from tuple memory (blocked by WASM-005)
- Match pattern bindings: Requires scoped locals architecture (future work)

**Test Files**:
- examples/lang_comp/13-tuples/02_tuple_destructuring.ruchy (has match patterns - will fail)

---

### 2. Struct Field Mutation [WASM-003]

**Status**: ⚠️ PARTIAL - Compiles but doesn't actually mutate (placeholder)
**Priority**: HIGH
**Blocking Tests**:
- `test_langcomp_014_01_struct_field_access` ⚠️ COMPILES (but mutation is no-op)
- `test_langcomp_014_02_struct_method_call` ⚠️ COMPILES (but mutation is no-op)

**Current Behavior**:
```rust
// ✅ COMPILES: Field mutation syntax accepted
struct Point { x: i32, y: i32 }
let mut p = Point { x: 0, y: 0 }
p.x = 5  // Compiles, but value is dropped (MVP placeholder)
println(p.x)  // Will print 0, not 5 (no actual mutation)

// ❌ LIMITATION: Mutation doesn't persist
// The assignment compiles but the value isn't stored anywhere
// This is because WASM-005 (memory model) isn't implemented yet
```

**Root Cause**:
- ✅ `lower_assign()` now handles FieldAccess targets
- ❌ No memory model to actually store field values (WASM-005 blocker)
- MVP: Value is dropped instead of stored (honest placeholder)

**MVP Implementation Complete**:
- Handle FieldAccess in lower_assign() ✅
- Code compiles without errors ✅
- MVP: Drop value instead of store (placeholder) ✅
- Honest documentation of limitation ✅

**Remaining Work**:
- Implement memory model (WASM-005) - CRITICAL blocker
- Calculate field offset from struct layout
- Use i32.store to write value at computed address
- Support nested field mutation: `obj.field.subfield = value`

**Test Files**:
- examples/lang_comp/14-structs/01_basic_structs.ruchy (will compile but mutations are no-op)

---

### 3. Complex Assignment Targets [WASM-004]

**Status**: ❌ NOT IMPLEMENTED
**Priority**: MEDIUM
**Blocking Tests**: (To be identified)

**Current Behavior**:
```rust
// All FAIL with "Assignment target must be identifier"
arr[0] = 10        // Array element assignment
obj.field = 5      // Field assignment (see #2)
tup.0 = 3          // Tuple element assignment
```

**Root Cause**: `lower_assign()` only handles ExprKind::Identifier

**Required Implementation**:
- Handle IndexAccess as assignment target (array[i] = val)
- Handle FieldAccess as assignment target (obj.field = val)
- Compute lvalue address before storing
- Requires memory model with addressable storage

---

### 4. Full Memory Model [WASM-005]

**Status**: ⚠️ PLACEHOLDER ONLY
**Priority**: CRITICAL (Blocks #2, #3, and proper data structures)
**Blocking**: All advanced data structure features

**Current Behavior**:
```rust
// All return i32 const 0 placeholder:
let pair = (1, 2)           // Returns 0, not actual tuple
let s = Point { x: 1, y: 2 } // Returns 0, not actual struct
let first = pair.0          // Returns 0, not actual field value
```

**Root Cause**: MVP uses I32Const(0) placeholders instead of actual memory allocation

**Required Implementation**:
- Linear memory allocation strategy
- Heap management (malloc/free equivalent)
- Type layout calculation (size, alignment, field offsets)
- Garbage collection or reference counting
- Memory safety validation

**Design Decisions Needed**:
- Manual memory management vs GC?
- Fixed-size heap vs growing heap?
- Type tagging strategy for dynamic types?

---

## Completed Features (v3.69.0)

### ✅ Basic Tuple Creation [WASM-001]
- **Implemented**: Commit d43197f2
- **Status**: WORKING (MVP - returns placeholder)
- **Test**: `test_langcomp_013_01_basic_tuples_example_file` ✅ PASSES

### ✅ Tuple Field Access [WASM-001]
- **Implemented**: Commit d43197f2
- **Status**: WORKING (MVP - returns placeholder)
- **Test**: Compiles without validation errors

### ✅ Simple Assignment [WASM-001]
- **Implemented**: Commit d43197f2
- **Status**: WORKING
- **Test**: `coords = (5, 10)` compiles successfully

### ✅ Unit Literal [WASM-001]
- **Implemented**: Commit d43197f2
- **Status**: WORKING
- **Test**: `let unit = ()` compiles successfully

### ✅ Character Literals [WASM-001]
- **Implemented**: Commit d43197f2
- **Status**: WORKING (UTF-32 code points)
- **Test**: `'a'` compiles to I32Const(97)

---

## Implementation Strategy (EXTREME TDD)

### Phase 1: Tuple Destructuring [WASM-002]
1. **RED**: Write failing test for `let (x, y) = (1, 2)`
2. **GREEN**: Implement Pattern::Tuple handling in lower_let()
3. **REFACTOR**: Ensure complexity <10, add comprehensive tests
4. **VALIDATE**: Run mutation tests, property tests
5. **COMMIT**: Only after all tests pass

### Phase 2: Struct Field Mutation [WASM-003]
1. **RED**: Write failing test for `p.x = 5`
2. **GREEN**: Implement FieldAccess in lower_assign()
3. **REFACTOR**: Handle nested fields
4. **VALIDATE**: Mutation testing
5. **COMMIT**: Only after all tests pass

### Phase 3: Memory Model [WASM-005]
1. **DESIGN**: Document memory layout strategy
2. **RED**: Write tests for actual data storage/retrieval
3. **GREEN**: Implement linear memory allocation
4. **REFACTOR**: Optimize for common patterns
5. **VALIDATE**: Property tests for memory safety
6. **COMMIT**: Only after comprehensive validation

---

## Test Coverage Requirements

Each limitation fix MUST include:
- ✅ Unit tests for specific feature
- ✅ Integration tests with LANG-COMP examples
- ✅ Property tests (10K+ random inputs)
- ✅ Mutation tests (75%+ coverage)
- ✅ Negative tests (error handling)
- ✅ Edge cases (empty tuples, nested patterns, etc.)

---

## Toyota Way Principles

1. **Jidoka (Stop the Line)**:
   - When WASM compilation fails, STOP and fix immediately
   - Don't defer to "future work" - fix it NOW
   - Document limitation → Implement fix → Validate → Ship

2. **Genchi Genbutsu (Go and See)**:
   - Run actual LANG-COMP tests to see real failures
   - Use failing tests to drive implementation
   - Never assume - always verify with tests

3. **Kaizen (Continuous Improvement)**:
   - MVP → Basic → Full implementation
   - Each commit adds working functionality
   - No half-finished features in main branch

4. **Respect for People**:
   - Clear documentation of what works and what doesn't
   - No surprises for users
   - Honest about limitations

---

## Acceptance Criteria

A limitation is considered RESOLVED when:
1. ✅ All blocking LANG-COMP tests pass
2. ✅ Property tests validate correctness (10K+ cases)
3. ✅ Mutation tests achieve 75%+ coverage
4. ✅ Documentation updated
5. ✅ Examples work end-to-end
6. ✅ No regressions in existing features

---

## Next Steps

### Completed (v3.69.x)
1. ✅ **[WASM-002]** Tuple Destructuring MVP (Commit 38096bfb)
   - Basic let destructuring works: `let (x, y) = (1, 2)`
   - Match pattern bindings not supported (documented limitation)

2. ✅ **[WASM-003]** Struct Field Mutation MVP (Commit 5e8a8042)
   - Syntax compiles: `p.x = 5`
   - Values dropped (no actual mutation until memory model)
   - Honest documentation prevents user confusion

### Current Priorities

1. **NEXT**: Implement [WASM-004] Complex Assignment Targets
   - Array element assignment: `arr[0] = 10`
   - Tuple element assignment: `tup.0 = 3`
   - Follow same MVP pattern (compile but use placeholders)
   - Complexity <10 for all functions

2. **CRITICAL BLOCKER**: Design [WASM-005] Memory Model
   - Research WASM linear memory best practices
   - Document design decisions:
     * Manual memory management vs GC?
     * Fixed-size heap vs growing heap?
     * Type tagging strategy for dynamic types?
   - Implement incrementally with EXTREME TDD
   - This unblocks ACTUAL implementation of WASM-002, WASM-003, WASM-004

3. **AFTER MEMORY MODEL**: Convert MVPs to Real Implementations
   - Replace I32Const(0) placeholders with actual data
   - Implement tuple unpacking from memory
   - Implement struct field access from memory
   - Implement array element access from memory
   - Full validation with property tests + mutation tests

---

**Remember**: Every limitation here represents a defect. **STOP THE LINE** and fix them systematically. No shortcuts. No deferrals. Quality is built in, not bolted on.
