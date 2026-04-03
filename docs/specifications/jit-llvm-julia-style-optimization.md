# Julia-Style JIT+LLVM Optimization for Ruchy

**Version:** 2.0
**Date:** 2025-11-02
**Status:** ACTIVE - Immediate + Long-term Roadmap
**Authors:** Ruchy Core Team
**References:** BENCH-007 Results (Fibonacci), Julia Language Design

---

## Executive Summary

This specification outlines a **two-phase optimization strategy** for Ruchy:

### Phase 1: IMMEDIATE AOT Optimizations (v3.174.0 - v3.180.0)
**Goal:** Beat Julia (1.35ms) with aggressive compiler flags - NO JIT needed yet

**Current Benchmark Results (ruchy-book BENCH-007 Fibonacci n=20):**
```
🥇 Julia:            1.32ms  (12.90x faster) ⚡ PRIMARY TARGET
🥈 C:                1.48ms  (11.51x faster) 🎯 SECONDARY TARGET
🥉 Ruchy Transpiled: 1.62ms  (10.51x faster) ✅ BEATS RUST (91% of C!)
   Rust:             1.64ms  (10.38x faster)
   Ruchy Compiled:   1.67ms  (10.20x faster) ⚠️  SLOWER due to opt="z"
   Go:               2.07ms  ( 8.22x faster)
```

**Geometric Mean Across 5 Benchmarks:**
```
🥇 Julia:            21.78x ⚡ TARGET TO BEAT
🥈 C:                16.04x (native baseline)
🥉 Rust:             14.26x (safety + speed)
   Ruchy Compiled:   13.04x (81% of C, 91% of Rust) ✅ BEATS GO
   Ruchy Transpiled: 12.93x (81% of C, 91% of Rust)
   Go:               12.16x
```

**Root Cause:** Current `opt-level = "z"` (size) sacrifices 15-20% performance vs `opt-level = 3`

**Immediate Actions (v3.174.0 - 2 weeks):**
1. Add `[profile.release-speed]` with maximum performance flags (NO CODE CHANGES!)
2. Enable LTO, PGO (Profile-Guided Optimization), and target-cpu=native
3. Pre-configure aggressive transpiler optimizations (tail-call, constant folding, dead code elimination)
4. Add `[profile.release-tiny]` for embedded systems (<100KB binary, reasonable speed)
5. **PRIMARY TARGET:** < 1.20ms (BEAT Julia's 1.32ms by 10%)
6. **STRETCH TARGET:** < 1.10ms (BEAT C's 1.48ms) with <500KB binary

**Strategy:** Make `release-speed` the DEFAULT profile, provide `release-tiny` for size-constrained use cases

### Phase 2: Julia-Style JIT+LLVM (v4.0+ - 6-12 months)
- **JIT compilation** for adaptive optimization
- **LLVM backend** for multi-platform support
- **Type specialization** based on runtime observations
- **Target:** 50-100x improvement for hot paths + <100ms REPL startup

---

## 1. Current State Analysis

### 1.1 Performance Baseline (v3.171.0)

From BENCH-008 (Prime Generation - 10,000 primes):

| Mode | Time (ms) | vs Rust Native | Use Case |
|------|----------:|---------------:|----------|
| Rust (native) | 5 | 1.0x | Reference |
| Python | 90 | 18x | Scripting |
| **Ruchy AST** | **1,588** | **318x** | Dev/REPL |
| **Ruchy Transpile** | **~5** | **1.0x** | Production |

### 1.2 Current Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Ruchy v3.171.0                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Interpreter Mode (AST Walking):                            │
│  Source → Parse → AST → Walk → Execute                      │
│  └──────────── 1,588ms ────────────┘                        │
│                                                             │
│  Transpile Mode (AOT via Rust):                             │
│  Source → Transpile → Rust → rustc → LLVM → Native          │
│  └──────────────────── ~5ms ───────────────────┘            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 1.3 Bottlenecks Identified

1. **Function Call Overhead:** 2x penalty for main() wrapper vs inline
2. **Variable Lookup:** Scope chain traversal on every access
3. **Arithmetic Operations:** Interpreted multiplication, modulo
4. **Type Checks:** Runtime type dispatch for every operation
5. **Memory Allocation:** Heap allocation for every Value

---


## Sub-spec Index

| Sub-spec | Scope |
|----------|-------|
| [AOT Optimizations](sub/optimization-aot.md) | Immediate AOT optimizations — beat Julia/C with compiler flags |
| [JIT Architecture & LLVM](sub/optimization-jit-llvm.md) | Julia-style JIT design + LLVM/inkwell integration |
| [Immediate Implementation Roadmap](sub/optimization-immediate-roadmap.md) | v3.174.0 action items to beat Julia |

---

## 6. Performance Targets

### 6.1 Expected Performance (BENCH-008)

| Tier | Mode | Expected Time | vs Current | Implementation |
|------|------|--------------|------------|----------------|
| 0 | AST Interpreter | 1,588ms | 1.0x | ✅ Current |
| 1 | Cranelift JIT | ~300ms | 5x faster | 🔧 Medium effort |
| 2 | LLVM Optimized | ~10ms | 150x faster | 🔧 High effort |
| - | Transpile (reference) | ~5ms | 300x faster | ✅ Current |

### 6.2 Tiered Execution Benefits

```
Example: BENCH-008 with tiered execution

Call Pattern:
- is_prime() called 104,729 times total
- First 10 calls: Tier 0 (interpret) → ~0.15ms each
- Calls 11-100: Tier 1 (Cranelift) → ~0.03ms each
- Calls 101+: Tier 2 (LLVM) → ~0.00005ms each

Total Time Breakdown:
- Tier 0: 10 calls × 0.15ms = 1.5ms
- Tier 1: 90 calls × 0.03ms = 2.7ms
- Tier 2: 104,629 calls × 0.00005ms = 5.2ms
- Compilation overhead: ~2ms
───────────────────────────────────────────
Total: ~11.4ms (vs 1,588ms current = 139x faster!)

Compared to:
- Pure interpretation: 1,588ms
- Pure LLVM (cold start): ~50ms compile + 5ms execute = 55ms
- Julia-style: 11.4ms (best of both!)
```

---

## 7. Implementation Roadmap (JIT - Long-term)

### 6.1 Phase 1: Foundation (v3.180.0) - 2 months

**Goal:** Set up infrastructure for tiered execution

**Tasks:**
1. ✅ Add `inkwell` and `cranelift-jit` dependencies
2. ✅ Implement `RuntimeProfiler` for call counting
3. ✅ Design `MethodCache` structure
4. ✅ Create `TypeSignature` system
5. ✅ Implement type inference from runtime values
6. ✅ Write integration tests

**Deliverables:**
- Profiling infrastructure working
- Type observation collecting runtime data
- Design doc validated with prototypes

**Success Criteria:**
- Can track hot functions (>100 calls)
- Can infer type signatures from Values
- Zero performance regression on current code

---

### 6.2 Phase 2: Tier 1 JIT (v3.200.0) - 3 months

**Goal:** Implement Cranelift-based quick JIT

**Tasks:**
1. ✅ Integrate `cranelift-jit` crate
2. ✅ Implement simple codegen for basic operations
   - Arithmetic: +, -, *, /, %
   - Comparisons: <, >, <=, >=, ==, !=
   - Control flow: if, while, return
3. ✅ Implement variable storage (stack allocation)
4. ✅ Implement function calls (calling convention)
5. ✅ Add Tier 0 → Tier 1 promotion logic
6. ✅ Benchmark and validate

**Deliverables:**
- Working Tier 1 JIT for subset of Ruchy
- 5-10x speedup on BENCH-008
- Comprehensive test suite

**Success Criteria:**
- BENCH-008: <400ms (vs 1,588ms baseline)
- Zero correctness regressions
- All existing tests pass

---

### 6.3 Phase 3: LLVM Backend (v4.0.0) - 4 months

**Goal:** Implement LLVM-based optimizing JIT

**Tasks:**
1. ✅ Integrate `inkwell` crate
2. ✅ Implement LLVM IR generation for:
   - Functions with type signatures
   - Control flow (if, while, for, match)
   - Operations (arithmetic, logical, comparison)
   - Variable bindings (let, mut)
   - Function calls (specialized per type sig)
3. ✅ Implement type specialization
4. ✅ Add Tier 1 → Tier 2 promotion logic
5. ✅ Enable LLVM optimizations (O3)
6. ✅ Benchmark end-to-end

**Deliverables:**
- Full LLVM codegen for Ruchy core
- 100-200x speedup on BENCH-008
- Multi-platform support (x86_64, ARM)

**Success Criteria:**
- BENCH-008: <15ms (vs 1,588ms baseline)
- Within 2-3x of transpile mode (~5ms)
- WebAssembly backend working

---

### 6.4 Phase 4: Optimization & Production (v4.5.0) - 3 months

**Goal:** Production-ready JIT+LLVM system

**Tasks:**
1. ✅ Advanced optimizations:
   - Inlining hot functions
   - Loop unrolling
   - SIMD vectorization
   - Escape analysis for stack allocation
2. ✅ Deoptimization support
   - Handle type instability
   - Fallback to interpreter when needed
3. ✅ Memory management:
   - Code cache eviction (LRU)
   - Compilation memory limits
4. ✅ Debugging support:
   - Source maps for JIT code
   - Profiler integration
5. ✅ Production hardening:
   - Stress testing
   - Memory leak detection
   - Crash reporting

**Deliverables:**
- Production-grade JIT+LLVM system
- Comprehensive benchmarks
- Documentation and tutorials

**Success Criteria:**
- BENCH-008: <10ms (match transpile mode)
- Stable in production workloads
- 99.9% compatibility with existing code

---

## 7. Technical Challenges & Mitigations

### 7.1 Challenge: Compilation Latency

**Problem:** LLVM compilation can take 10-100ms for complex functions

**Solution:** Tiered execution
- Tier 0: Interpret immediately (0ms compile)
- Tier 1: Quick compile with Cranelift (1-5ms compile)
- Tier 2: LLVM optimize only for proven hot code (10-50ms compile)

**Result:** Most code never pays LLVM cost, hot code gets full optimization

---

### 7.2 Challenge: Type Instability

**Problem:** If a function is called with different types, need multiple compiled versions

**Example:**
```ruchy
fun add(a, b) { a + b }

add(1, 2)        // i32 + i32 → Compile add_i32_i32
add(1.5, 2.3)    // f64 + f64 → Compile add_f64_f64
add("hi", "bye") // String + String → Compile add_string_string
```

**Solution:** Method cache per type signature
- Track observed type signatures
- Compile most common signatures (top 3)
- Fall back to interpreter for rare signatures
- Limit max compiled versions per function (prevent explosion)

---

### 7.3 Challenge: Memory Overhead

**Problem:** Storing compiled code consumes memory

**Solution:** LRU cache with limits
```rust
pub struct MethodCache {
    max_methods: usize,  // Default: 10,000
    lru: LRUList,
    cache: HashMap<MethodKey, CompiledMethod>,
}

impl MethodCache {
    fn insert(&mut self, key: MethodKey, method: CompiledMethod) {
        if self.cache.len() >= self.max_methods {
            // Evict least recently used
            let evicted = self.lru.pop_back();
            self.cache.remove(&evicted);
        }
        self.cache.insert(key, method);
        self.lru.push_front(key);
    }
}
```

**Monitoring:**
- Track cache hit rate (target: >95%)
- Alert if cache thrashing (eviction rate >10%)
- Adaptive sizing based on workload

---

### 7.4 Challenge: Debugging JIT Code

**Problem:** Stack traces point to JIT code addresses, not source lines

**Solution:** Source maps
```rust
pub struct SourceMap {
    mappings: HashMap<*const (), SourceLocation>,
}

struct SourceLocation {
    file: String,
    line: usize,
    column: usize,
}

// When JIT compiling:
fn emit_debug_info(&mut self, native_addr: *const (), ast_span: Span) {
    self.source_map.add_mapping(native_addr, SourceLocation {
        file: ast_span.file.clone(),
        line: ast_span.line,
        column: ast_span.column,
    });
}

// When printing stack trace:
fn format_stack_frame(addr: *const ()) -> String {
    if let Some(loc) = source_map.lookup(addr) {
        format!("{}:{}:{}", loc.file, loc.line, loc.column)
    } else {
        format!("<JIT code at {:p}>", addr)
    }
}
```

---

## 8. Validation & Testing

### 8.1 Performance Benchmarks

**Micro-benchmarks:**
```ruchy
// bench-micro-001: Arithmetic hot loop
fun arithmetic_loop() {
    let mut sum = 0
    let mut i = 0
    while i < 1000000 {
        sum = sum + i
        i = i + 1
    }
    sum
}
// Target: <10ms (vs 500ms interpreted)

// bench-micro-002: Function call overhead
fun call_intensive(n) {
    if n <= 0 { return 1 }
    call_intensive(n - 1) + call_intensive(n - 1)
}
// Target: Fibonacci(20) in <50ms (vs 2000ms interpreted)

// bench-micro-003: Type-specialized operations
fun type_specialized(a, b) { a + a * b - b / a }
// Target: 1M ops in <20ms per type (i32, f64, etc.)
```

**Macro-benchmarks:**
- BENCH-008 (Prime generation): <10ms (vs 1,588ms)
- BENCH-007 (Fibonacci): <50ms
- BENCH-003 (String concat): <100ms
- Real-world: Reaper process analysis <1s

### 8.2 Correctness Testing

**Property-based tests:**
```rust
#[proptest]
fn jit_matches_interpreter(#[strategy(arbitrary_expr())] expr: Expr) {
    let interpreted = interpreter.eval(&expr)?;
    let jit_compiled = jit_engine.execute(&expr)?;
    prop_assert_eq!(interpreted, jit_compiled);
}

#[proptest]
fn llvm_matches_interpreter(
    #[strategy(arbitrary_expr())] expr: Expr,
    #[strategy(arbitrary_types())] types: TypeSignature,
) {
    let interpreted = interpreter.eval(&expr)?;
    let llvm_compiled = llvm_engine.execute(&expr, &types)?;
    prop_assert_eq!(interpreted, llvm_compiled);
}
```

### 8.3 Regression Testing

**All existing tests must pass:**
- 4,000+ library tests
- Language compatibility suite (41/41 features)
- Integration tests (examples/, tests/)
- Property tests (14,000+ cases)

---

## 9. References
