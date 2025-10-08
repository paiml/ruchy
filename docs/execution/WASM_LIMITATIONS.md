# WASM Compilation Limitations Tracker

**Last Updated**: 2025-10-08
**Version**: v3.70.0
**Status**: MEMORY MODEL WORKING - Real data structures in WASM!

## Purpose

This document tracks known WASM compilation limitations to ensure TRANSPARENCY and systematic resolution. Following Toyota Way principles: **STOP THE LINE** when limitations are discovered, document them, then fix them systematically.

## Critical Principle

**NEVER HIDE BUGS OR LIMITATIONS**. Every limitation here represents work to be done. Each limitation blocks real user workflows and must be fixed with EXTREME TDD.

---

## Current Limitations (Active Issues)

### 1. Tuple Destructuring Patterns [WASM-002]

**Status**: ✅ 80% COMPLETE - Real memory loading works! Match patterns not supported
**Priority**: MEDIUM (core functionality working)
**Completed**: v3.70.0 (Phases 1-3 of WASM-005)
**Blocking Tests**:
- `test_langcomp_013_02_basic_destructuring` ✅ WORKS WITH REAL VALUES
- `test_langcomp_013_02_underscore_destructuring` ✅ WORKS WITH REAL VALUES
- `test_langcomp_013_04_destructuring_nested` ✅ WORKS WITH REAL VALUES
- `test_langcomp_013_02_tuple_destructuring_example_file` ⚠️ (blocked by match patterns only)

**Current Behavior**:
```rust
// ✅ WORKS: Basic let destructuring WITH REAL MEMORY LOADS
let (x, y) = (3, 4)
println(x)  // Prints: 3 (loaded from memory address 0)
println(y)  // Prints: 4 (loaded from memory address 4)

// ✅ WORKS: Nested destructuring WITH REAL MEMORY
let ((a, b), c) = ((1, 2), 3)
println(a)  // Prints: 1 (loaded from inner tuple memory)
println(b)  // Prints: 2
println(c)  // Prints: 3 (loaded from outer tuple memory)

// ✅ WORKS: Underscore patterns
let (x, _, z) = (1, 2, 3)
println(x)  // Prints: 1 (real value)
println(z)  // Prints: 3 (real value)

// ❌ ONLY LIMITATION: Pattern variables in match arms
match point {
    (x, y) => println(x)  // Not supported (requires scoped locals)
}
```

**Root Cause**:
- ✅ `lower_let_pattern()` loads actual values from tuple memory (v3.70.0)
- ✅ `store_pattern_values()` uses i32.load to extract tuple elements
- ✅ Bump allocator allocates real memory for tuples
- ✅ Nested tuples work correctly
- ❌ Match arm pattern variables require scoped locals (WASM architectural limitation)

**Implementation Complete (v3.70.0)**:
- Handle Pattern::Tuple in let bindings ✅
- Allocate memory for tuples using bump allocator ✅
- Load actual values from tuple memory (NOT placeholders!) ✅
- Support nested destructuring ✅
- Support underscore patterns ✅
- Match arm patterns: Intentionally not supported (requires scoped locals)

**Memory Model (WASM-005 Phases 1-3)**:
```
Example: let (x, y) = (3, 4)

1. Allocate tuple at address 0 (8 bytes)
   Memory[0-3]: 3
   Memory[4-7]: 4

2. Destructure with i32.load
   x = i32.load(address + 0) = 3
   y = i32.load(address + 4) = 4
```

**Remaining Work**:
- Match pattern bindings: Requires scoped locals architecture (future work, low priority)

**Test Files**:
- ✅ test_destructure_real.ruchy - PASSES
- ✅ test_nested_destructure.ruchy - PASSES
- ⚠️ examples/lang_comp/13-tuples/02_tuple_destructuring.ruchy (match patterns only)

---

### 2. Struct Field Mutation [WASM-003]

**Status**: ✅ COMPLETE - Real struct field mutation working! (v3.70.0 Phase 4)
**Priority**: COMPLETE
**Completed**: v3.70.0 (Phase 4 - Struct Registry & Field Mutation)

**Current Behavior**:
```rust
// ✅ WORKS: Struct definition and instantiation
struct Point { x: i32, y: i32 }
let mut p = Point { x: 3, y: 4 }
println(p.x)  // Prints: 3 (loaded from memory offset 0)
println(p.y)  // Prints: 4 (loaded from memory offset 4)

// ✅ WORKS: Field mutation stores to memory
p.x = 10
println(p.x)  // Prints: 10 (real mutation!)

// ✅ WORKS: Field access after mutation
println(p.y)  // Prints: 4 (unchanged)
```

**Implementation Complete (v3.70.0 Phase 4)**:
- ✅ Struct registry: Maps struct_name → field_names for offset calculation
- ✅ collect_struct_definitions(): Traverses AST to collect struct definitions
- ✅ lower_struct_literal(): Allocates memory with bump allocator, returns address
- ✅ lower_field_access(): Looks up field offset from registry, uses i32.load
- ✅ lower_assign() for FieldAccess: Uses i32.store at calculated offset
- ✅ Five Whys root cause analysis: Fixed missing struct registry architecture

**Memory Model**:
```
struct Point { x: i32, y: i32 }
let p = Point { x: 3, y: 4 }

1. Allocate 8 bytes at address 0 (2 fields × 4 bytes)
   Memory[0-3]: 3  (field x at offset 0)
   Memory[4-7]: 4  (field y at offset 4)

2. Field access: p.x
   i32.load(address + 0) = 3

3. Field mutation: p.x = 10
   i32.store(address + 0, 10)
```

**Limitations**:
- ⚠️ Nested field mutation `obj.field.subfield = val` requires chained address calculation (future work)
- ⚠️ Field name collisions across structs use heuristic (first match wins)

**Test Files**:
- test_struct_mutation.ruchy ✅ PASSING (compiles to valid 146-byte WASM module)

---

### 3. Complex Assignment Targets [WASM-004]

**Status**: ✅ COMPLETE - All assignment targets working! (v3.70.0 Phases 4-5)
**Priority**: COMPLETE
**Completed**: v3.70.0 (Phase 4: Fields, Phase 5: Arrays/Tuples)

**Current Behavior**:
```rust
// ✅ WORKS: Array element assignment
let mut arr = [10, 20, 30]
arr[0] = 100
println(arr[0])    // Prints: 100 (real mutation!)

// ✅ WORKS: Field assignment
struct Point { x: i32, y: i32 }
let mut p = Point { x: 3, y: 4 }
p.x = 10
println(p.x)       // Prints: 10 (real mutation!)

// ✅ WORKS: Tuple element assignment (using field syntax)
let mut tup = (1, 2, 3)
tup.0 = 100
println(tup.0)     // Prints: 100 (real mutation!)
```

**Implementation Complete (v3.70.0)**:
- ✅ Field assignment (Phase 4): `obj.field = val` with i32.store
- ✅ Array element assignment (Phase 5): `arr[i] = val` with dynamic offset
- ✅ Tuple element assignment (Phase 5): `tup.0 = val` works via field access
- ✅ Dynamic indexing: Runtime offset computation with i32.mul
- ✅ Memory stores: All mutations persist in memory

**Limitations**:
- ⚠️ Nested complex assignments `arr[i].field = val` require chained address calculation (future work)

**Test Files**:
- test_array_mutation.ruchy ✅ PASSING (204-byte WASM module)
- test_struct_mutation.ruchy ✅ PASSING (146-byte WASM module)

---

### 4. Full Memory Model [WASM-005]

**Status**: ✅ PHASES 1-5 COMPLETE - Full memory model working! (v3.70.0)
**Priority**: COMPLETE
**Completed**: Bump allocator, tuples, structs, arrays with full mutation support

**Current Behavior**:
```rust
// ✅ WORKING: Tuples use real memory
let pair = (3, 4)           // Allocates 8 bytes, returns address 0
let first = pair.0          // i32.load(0) = 3 (REAL value!)
let (x, y) = pair           // Loads x=3, y=4 from memory

// ✅ WORKING: Structs use real memory
let s = Point { x: 1, y: 2 } // Allocates 8 bytes, returns address
s.x = 10                    // i32.store with field offset

// ✅ WORKING: Arrays use real memory
let arr = [1, 2, 3]         // Allocates 12 bytes, returns address
arr[0] = 100                // i32.store with dynamic offset
```

**Completed Implementation (v3.70.0)**:
- ✅ **Phase 1**: Memory foundation (64KB heap, global $heap_ptr)
- ✅ **Phase 2**: Tuple storage (allocate, store, return address)
- ✅ **Phase 3**: Tuple destructuring (i32.load elements into locals)
- ✅ **Phase 4**: Struct field mutation (struct registry, field offset calculation)
- ✅ **Phase 5**: Array element access (dynamic indexing with i32.mul)

**Design Decisions (Implemented)**:
- ✅ Bump allocator: O(1) allocation, no GC/free (acceptable for MVP)
- ✅ Fixed 64KB heap: Sufficient for testing
- ✅ Sequential layout: No padding, no type tags
- ✅ i32 only: All values are 4 bytes
- ✅ Dynamic offsets: Runtime index * 4 for arrays

**Future Work (Low Priority)**:
- Garbage collection (not needed for short-running programs)
- Growing heap (64KB sufficient for current use cases)
- Nested complex assignments: `arr[i].field = val`

**Memory Model Architecture**:
```wat
;; Global heap pointer (mutable)
(global $heap_ptr (mut i32) (i32.const 0))

;; Inline bump allocator in lower_tuple()
global.get $heap_ptr          ;; Get current address
local.set $temp               ;; Save it
global.get $heap_ptr
i32.const 8                   ;; Size needed
i32.add
global.set $heap_ptr          ;; Update heap pointer

;; Store elements
local.get $temp
i32.const 3
i32.store offset=0            ;; Store first element
local.get $temp
i32.const 4
i32.store offset=4            ;; Store second element

local.get $temp               ;; Return address
```

---

## Completed Features

### ✅ Memory Model Foundation [WASM-005 Phase 1] (v3.70.0)
- **Implemented**: Commit 9a4a67ae
- **Status**: FULLY WORKING - 64KB heap with global heap pointer
- **Features**:
  - Memory section: 1 page (64KB), max=1
  - Global section: `$heap_ptr` (mutable i32, init=0)
  - Design document: docs/execution/WASM_MEMORY_MODEL.md
- **Test**: Memory and global sections present in all tuple code

### ✅ Tuple Memory Storage [WASM-005 Phase 2] (v3.70.0)
- **Implemented**: Commit f7fdb1de
- **Status**: FULLY WORKING - Real memory allocation and storage
- **Features**:
  - Inline bump allocator in `lower_tuple()`
  - Allocates memory: GlobalGet($heap_ptr) → save → update → store
  - Returns memory address instead of placeholder
  - Field access loads from memory with i32.load
- **Test**: `let pair = (3, 4); println(pair.0)` prints 3 (real value!)

### ✅ Tuple Destructuring [WASM-005 Phase 3] (v3.70.0)
- **Implemented**: Commit 30089fc6
- **Status**: FULLY WORKING - Loads real values from memory
- **Features**:
  - `store_pattern_values()` loads from tuple memory
  - Uses i32.load at address + offset for each element
  - Stores loaded values into pattern variable locals
  - Nested destructuring works correctly
- **Test**: `let (x, y) = (3, 4); println(x)` prints 3 (real value!)

### ✅ Struct Field Mutation [WASM-005 Phase 4] (v3.70.0)
- **Implemented**: Commit 4a42b76a
- **Status**: FULLY WORKING - Real struct field mutation with memory stores
- **Features**:
  - Struct registry: Maps struct_name → field_names for offset calculation
  - `collect_struct_definitions()`: Traverses AST to collect struct definitions
  - `lower_struct_literal()`: Allocates memory with bump allocator
  - `lower_field_access()`: Looks up field offset from registry, uses i32.load
  - `lower_assign()` for FieldAccess: Uses i32.store at calculated offset
  - Five Whys root cause analysis: Fixed missing struct registry architecture
- **Test**: `p.x = 10` stores to memory, `println(p.x)` prints 10 (real mutation!)

### ✅ Array Element Access [WASM-005 Phase 5] (v3.70.0)
- **Implemented**: Commit 27bb8474
- **Status**: FULLY WORKING - Dynamic array indexing with memory loads/stores
- **Features**:
  - `lower_list()`: Allocates memory for arrays with bump allocator
  - `lower_index_access()`: Dynamic offset computation (index * 4) with i32.mul
  - `lower_assign()` for IndexAccess: Dynamic i32.store at runtime-computed address
  - Runtime indexing: Supports variable indices, not just constants
  - Works for both arrays and tuples with numeric indices
- **Test**: `arr[0] = 100; println(arr[0])` prints 100 (real dynamic mutation!)

### ✅ Basic Tuple Creation [WASM-001] (v3.69.0 → v3.70.0)
- **Implemented**: Commit d43197f2 → Upgraded f7fdb1de
- **Status**: FULLY WORKING WITH REAL MEMORY (v3.70.0)
- **Test**: `test_langcomp_013_01_basic_tuples_example_file` ✅ PASSES

### ✅ Tuple Field Access [WASM-001] (v3.69.0 → v3.70.0)
- **Implemented**: Commit d43197f2 → Upgraded f7fdb1de
- **Status**: FULLY WORKING WITH REAL MEMORY (v3.70.0)
- **Test**: Loads actual values from memory with i32.load

### ✅ Simple Assignment [WASM-001] (v3.69.0)
- **Implemented**: Commit d43197f2
- **Status**: WORKING
- **Test**: `coords = (5, 10)` compiles successfully

### ✅ Unit Literal [WASM-001] (v3.69.0)
- **Implemented**: Commit d43197f2
- **Status**: WORKING
- **Test**: `let unit = ()` compiles successfully

### ✅ Character Literals [WASM-001] (v3.69.0)
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

3. ✅ **[WASM-004]** Complex Assignment Targets MVP (Current session)
   - Array element assignment: `arr[0] = 10` ✅ COMPILES
   - Tuple element assignment: `tup.0 = 3` ✅ COMPILES
   - Added lower_index_access() for reading array/tuple elements
   - All assignment targets now supported (placeholders)
   - Complexity maintained <10 for all functions

### Current Priorities

1. **CRITICAL BLOCKER**: Design [WASM-005] Memory Model
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
