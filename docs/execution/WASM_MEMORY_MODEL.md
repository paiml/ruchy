# WASM Memory Model Design

**Status**: DESIGN PHASE
**Created**: 2025-10-08
**Ticket**: [WASM-005]

## Purpose

Design and implement a memory model for Ruchy WASM compilation to enable actual data structure storage (tuples, structs, arrays) instead of placeholders.

## Critical Principle

**START SIMPLE, ITERATE INCREMENTALLY**. This is an MVP memory model, not a production-ready system. Follow Toyota Way: build working foundation, then improve.

---

## MVP Design Decisions

### 1. Memory Allocation Strategy: **Bump Allocator**

**Decision**: Use simple bump allocator (no free, just allocate forward)

**Rationale**:
- Simplest possible allocator
- O(1) allocation time
- No fragmentation concerns
- No free() needed for MVP
- Easy to implement and understand

**Implementation**:
```wat
;; Global: current heap pointer (starts at 0)
(global $heap_ptr (mut i32) (i32.const 0))

;; Allocate n bytes, return address
(func $malloc (param $size i32) (result i32)
  (local $addr i32)
  (local.set $addr (global.get $heap_ptr))
  (global.set $heap_ptr (i32.add (global.get $heap_ptr) (local.get $size)))
  (local.get $addr)
)
```

### 2. Garbage Collection: **NONE (MVP)**

**Decision**: No GC, no free() for MVP

**Rationale**:
- Simplest approach for MVP
- Avoids complex GC implementation
- Memory "leaks" acceptable for short-running programs
- Can add GC in future iteration

**Limitation**: Programs with many allocations will exhaust memory
**Acceptable**: MVP targets simple examples, not long-running production code

### 3. Heap Size: **Fixed (1 page = 64KB)**

**Decision**: Single WASM memory page (64KB)

**Rationale**:
- WASM minimum is 1 page (64KB)
- Sufficient for LANG-COMP test examples
- Simple memory declaration: `(memory 1)`
- Can grow later if needed

**Calculation**:
- 64KB = 65,536 bytes
- Typical tuple (2 i32): 8 bytes
- Can fit ~8,000 tuples in 64KB
- More than enough for MVP testing

### 4. Type Layout: **Simple Sequential**

**Decision**: Store values sequentially, no padding, no tags

**Rationale**:
- Simplest memory layout
- No alignment concerns for MVP
- Direct offset calculation
- Easy to debug

**Examples**:
```
Tuple (1, 2):
  Address: 0
  [0-3]: i32 = 1
  [4-7]: i32 = 2

Struct Point { x: 1, y: 2 }:
  Address: 8
  [8-11]: i32 = 1  (x field)
  [12-15]: i32 = 2 (y field)

Array [10, 20, 30]:
  Address: 16
  [16-19]: i32 = 10
  [20-23]: i32 = 20
  [24-27]: i32 = 30
```

### 5. Type Representation: **i32 Address**

**Decision**: All composite types return i32 memory address

**Rationale**:
- Compatible with existing placeholder (I32Const(0))
- Simple value representation
- No type tagging needed for MVP
- Direct memory access via address

**Impact**:
- `let pair = (1, 2)` returns address (e.g., 0)
- `pair.0` becomes `i32.load (address + 0)`
- `pair.1` becomes `i32.load (address + 4)`

---

## Implementation Plan (EXTREME TDD)

### Phase 1: Basic Tuple Storage (CRITICAL PATH)

**Goal**: Make `let pair = (1, 2); println(pair.0)` print actual value, not 0

**RED**:
```ruchy
fn main() {
    let pair = (3, 4)
    println(pair.0)  // Should print 3, not 0
    println(pair.1)  // Should print 4, not 0
}
```

**GREEN**:
1. Add memory section: `(memory 1)` - 64KB
2. Add global `$heap_ptr`: `(global $heap_ptr (mut i32) (i32.const 0))`
3. Implement `lower_tuple()` to allocate and store:
   - Call `$malloc(8)` for 2x i32
   - Store each element using i32.store
   - Return address instead of I32Const(0)
4. Implement `lower_field_access()` to load:
   - Calculate offset (index * 4 bytes)
   - Load using i32.load at (address + offset)
   - Return actual value

**VALIDATE**:
- Test prints "3" and "4", not "0" and "0"
- Verify WASM module validates
- Check memory usage stays within 64KB

### Phase 2: Tuple Destructuring (UNBLOCK WASM-002)

**Goal**: Make `let (x, y) = (3, 4); println(x)` work correctly

**Implementation**:
- `lower_let_pattern()`: Load from tuple memory into locals
- Use i32.load to read tuple elements
- Store into pattern variable locals

### Phase 3: Struct Field Mutation (UNBLOCK WASM-003)

**Goal**: Make `p.x = 5` actually modify p.x

**Implementation**:
- `lower_assign()` for FieldAccess: Calculate field offset, use i32.store
- `lower_field_access()`: Calculate field offset, use i32.load

### Phase 4: Array Element Access (UNBLOCK WASM-004)

**Goal**: Make `arr[0] = 10; println(arr[0])` work correctly

**Implementation**:
- `lower_index_access()`: Calculate element offset, use i32.load
- `lower_assign()` for IndexAccess: Calculate offset, use i32.store

---

## Type Sizes (i32 only for MVP)

For MVP, all values are i32 (4 bytes):
- Integers: 4 bytes
- Booleans: 4 bytes (0 = false, 1 = true)
- Chars: 4 bytes (UTF-32 code points)
- Tuples: 4 bytes per element
- Struct fields: 4 bytes per field
- Array elements: 4 bytes per element

**Strings**: Not in MVP scope (requires length + data)
**Floats**: Not in MVP scope (would be f32 = 4 bytes)

---

## Memory Safety (MVP)

**Not Implemented**:
- Bounds checking (will trap on out-of-bounds access)
- Type safety (can read wrong type from memory)
- Use-after-free detection (no free() exists)
- Double-free detection (no free() exists)

**Acceptable**: MVP focuses on correctness for simple programs, not security

---

## Testing Strategy

### Unit Tests:
- Tuple allocation and retrieval
- Struct field access and mutation
- Array element access and mutation
- Multiple allocations (no collision)

### Property Tests:
- Any tuple stores and retrieves correct values
- Field offsets calculated correctly
- Memory allocations don't overlap

### Integration Tests:
- All LANG-COMP-013 (tuples) tests pass with real values
- All LANG-COMP-014 (structs) tests pass with real values
- Complex nested structures work correctly

---

## Future Work (POST-MVP)

### Short Term:
- String support (length + data pointer)
- Float support (f32/f64 types)
- Mixed-type tuples (i32, f32, bool)

### Medium Term:
- Garbage collection (mark-and-sweep or reference counting)
- Memory compaction (defragmentation)
- Growing heap (multiple pages)

### Long Term:
- Optimized allocators (free lists, size classes)
- Memory safety validation (bounds checking, type checking)
- WASM SIMD for bulk memory operations

---

## Acceptance Criteria

[WASM-005] is complete when:

1. ✅ Basic tuple creation stores actual values in memory
2. ✅ Tuple field access returns actual values, not placeholders
3. ✅ Struct field access returns actual values
4. ✅ Struct field mutation actually modifies memory
5. ✅ Array element access returns actual values
6. ✅ Array element mutation actually modifies memory
7. ✅ All LANG-COMP tests pass with real values
8. ✅ Memory allocations stay within 64KB limit
9. ✅ WASM module validates correctly
10. ✅ Complexity <10 for all new functions

---

## References

- [WASM Memory Spec](https://webassembly.github.io/spec/core/syntax/instructions.html#memory-instructions)
- [WASM Linear Memory](https://developer.mozilla.org/en-US/docs/WebAssembly/Understanding_the_text_format#linear_memory)
- Toyota Way: Simple solutions, incremental improvement, quality built-in

---

**Remember**: This is MVP. Start simple, make it work, then improve. No premature optimization. Quality is built in through TDD, not bolted on.
