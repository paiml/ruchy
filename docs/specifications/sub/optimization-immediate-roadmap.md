# Sub-spec: Optimization — Immediate Implementation Roadmap

**Parent:** [jit-llvm-julia-style-optimization.md](../jit-llvm-julia-style-optimization.md) Section 10

---


### 9.1 Julia Language

- **Julia Documentation:** https://docs.julialang.org/en/v1/devdocs/eval/
- **Julia's Type Inference:** https://julialang.org/blog/2018/08/union-splitting/
- **Julia LLVM Integration:** https://github.com/JuliaLang/julia/tree/master/src/llvm-*

### 9.2 LLVM Resources

- **inkwell (Rust LLVM bindings):** https://github.com/TheDan64/inkwell
- **LLVM IR Language Reference:** https://llvm.org/docs/LangRef.html
- **LLVM Optimization Guide:** https://llvm.org/docs/Passes.html

### 9.3 JIT Compilation

- **Cranelift JIT:** https://github.com/bytecodealliance/wasmtime/tree/main/cranelift
- **LuaJIT Design:** http://wiki.luajit.org/SSA-IR-2.0
- **V8 TurboFan:** https://v8.dev/docs/turbofan

### 9.4 Ruchy Internal Docs

- `docs/performance/BENCH-008-ANALYSIS.md` - Current performance baseline
- `tests/issue_113_transpiler_type_inference.rs` - Type system validation
- `src/backend/transpiler/` - Current AOT transpiler (reference)

---

## 10. IMMEDIATE Implementation Roadmap (v3.174.0 - Beat Julia, C, Rust)

### 10.1 Phase 1A: Cargo.toml Optimization Profiles (v3.174.0 - 1 week) ⚡ **HIGHEST PRIORITY**

**Ticket:** [PERF-001] Change default release profile to opt-level=3 + add release-tiny

**Tasks:**
1. **BREAKING CHANGE:** Modify `[profile.release]` → `opt-level = 3` (was "z")
2. Add `[profile.release-ultra]` with PGO support
3. Add `[profile.release-tiny]` with `opt-level = "z"` (for embedded)
4. Document RUSTFLAGS: `-C target-cpu=native -C link-arg=-fuse-ld=lld`
5. Add Makefile targets: `make release`, `make release-ultra`, `make release-tiny`
6. Update CI/CD to use `release` profile (not `release-tiny`)
7. Update docs: migration guide for users needing tiny binaries

**Acceptance Criteria:**
- BENCH-007: < 1.20ms ⚡ **BEAT Julia's 1.32ms by 10%**
- Geometric mean: > 15.0x (vs current 13.04x)
- Binary size (release): < 500KB
- Binary size (release-tiny): < 100KB
- Zero regressions in test suite (4033 tests pass)
- **VALIDATION:** Re-run full benchmark suite (BENCH-001 through BENCH-012)

---

### 10.2 Phase 1B: Profile-Guided Optimization (v3.175.0 - 1 week)

**Ticket:** [PERF-002] Implement PGO workflow for release builds

**Tasks:**
1. Create `make release-pgo` target with 3-step build
2. Add benchmark corpus for PGO training (BENCH-001 through BENCH-010)
3. Automate: instrument → train → optimize workflow
4. Document PGO usage in CLAUDE.md

**Acceptance Criteria:**
- Additional 10-15% speedup over release-speed
- BENCH-007: < 1.20ms (10% better than Julia)
- Automated PGO in CI/CD pipeline

---

### 10.3 Phase 2: Transpiler AST Optimizations (v3.176.0 - v3.178.0 - 3 weeks)

#### 10.3.1 Constant Folding & Dead Code Elimination (v3.176.0)

**Ticket:** [TRANSPILER-PERF-001] Implement constant folding pass

**Tasks:**
1. Add `ConstantFolder` visitor in `src/backend/transpiler/optimizations/`
2. Fold arithmetic: `2 + 3 * 4` → `14`
3. Fold comparisons: `5 > 3` → `true`
4. Remove dead code: `if false { ... }` → (removed)
5. Add property tests: verify semantics preserved

**Acceptance Criteria:**
- All constant expressions folded at compile-time
- Dead code eliminated from generated Rust
- Zero correctness regressions (validated by property tests)

---

#### 10.3.2 Tail Call Optimization (v3.177.0)

**Ticket:** [TRANSPILER-PERF-002] Convert tail-recursive functions to loops

**Tasks:**
1. Detect tail-recursive functions in AST
2. Transform to loop with mutable variables
3. Preserve semantics (validated by equivalence tests)
4. Add tests: factorial, fibonacci, list recursion

**Acceptance Criteria:**
- Tail-recursive functions compile to loops (constant stack)
- BENCH-007 (tail-recursive fibonacci): 0% stack growth
- Stack overflow eliminated for tail-recursive code

---

#### 10.3.3 Function Inlining (v3.178.0)

**Ticket:** [TRANSPILER-PERF-003] Inline small functions (≤10 lines)

**Tasks:**
1. Detect small functions (threshold: 10 AST nodes)
2. Inline at call sites (copy-paste with variable renaming)
3. Respect `#[inline(never)]` annotation
4. Add cost model: inline only if net speedup

**Acceptance Criteria:**
- Functions ≤10 nodes inlined automatically
- Zero correctness regressions
- Benchmark: 5-10% speedup on function-call-heavy code

---

### 10.4 Phase 3: Bytecode VM Optimizations (v3.179.0 - v3.180.0 - 2 weeks)

#### 10.4.1 NaN-Boxing for Value Representation (v3.179.0)

**Ticket:** [VM-PERF-001] Implement NaN-boxed Value type

**Tasks:**
1. Replace `enum Value` with `struct Value(u64)`
2. Encode i32, f64, bool, pointers in 64 bits
3. Implement fast type checks (bit masking)
4. Add comprehensive tests (all value types)

**Acceptance Criteria:**
- Zero heap allocations for i32/f64/bool
- 30% faster arithmetic operations
- All tests pass (4033 library tests)

---

#### 10.4.2 Inline Caching for Method Lookups (v3.180.0)

**Ticket:** [VM-PERF-002] Implement inline caches for method dispatch

**Tasks:**
1. Add `InlineCache` struct to bytecode instructions
2. Cache method pointers after first lookup
3. Invalidate on type change (polymorphic detection)
4. Benchmark hot loop performance

**Acceptance Criteria:**
- Method lookups: O(1) for monomorphic sites (vs O(log n))
- 5-10x speedup for method-call-heavy code
- Graceful degradation for polymorphic sites

---

### 10.5 Validation & Benchmarking (Continuous) - **GATE 0 FOR EVERY RELEASE**

**Every sprint MUST validate performance targets before merge:**

```bash
# Run full benchmark suite after EVERY optimization
make bench-all  # Runs BENCH-001 through BENCH-012

# MANDATORY CHECKS (BLOCKING):
# 1. BENCH-007 (Fibonacci) progression check
# 2. Geometric mean across 5 benchmarks
# 3. Binary size validation
# 4. Zero test regressions (4033 tests)

# Target progression (BENCH-007 Fibonacci n=20):
v3.173.0:  1.67ms (baseline - opt="z")       | Geometric: 13.04x | Binary: 2MB
v3.174.0:  1.20ms (opt=3 profile)  ⚡        | Geometric: 15.50x | Binary: 485KB  | ✅ BEAT Julia (1.32ms)
v3.175.0:  1.12ms (+ PGO)          ⚡        | Geometric: 16.20x | Binary: 520KB  | ✅ BEAT C (1.48ms)
v3.176.0:  1.08ms (+ constant fold + TCO)   | Geometric: 16.80x | Binary: 510KB
v3.177.0:  1.05ms (+ inlining)              | Geometric: 17.50x | Binary: 520KB
v3.178.0:  1.02ms (+ loop unroll)           | Geometric: 18.00x | Binary: 530KB
v3.180.0:  1.00ms (+ all AST opts) 🚀       | Geometric: 18.50x | Binary: 540KB  | ✅ WORLD-CLASS

# VALIDATION GATES (ALL MUST PASS):
✅ BENCH-007 < 1.20ms (beat Julia by 10%)
✅ Geometric mean > 15.0x (beat current 13.04x by 15%)
✅ Binary size < 600KB (release profile)
✅ Binary size < 100KB (release-tiny profile)
✅ Zero test regressions (cargo test --all passes)
✅ Zero clippy warnings (cargo clippy --all-targets)
✅ Book examples still work (make validate-book)
```

**Final Goals (v3.180.0):**
- **Speed:** 1.00ms (24% faster than Julia's 1.32ms) ⚡
- **Geometric mean:** 18.50x (beats C's 16.04x by 15%) 🚀
- **Binary size:** < 600KB (release), < 100KB (release-tiny)
- **Validation:** All quality gates pass (PMAT, tests, benchmarks)

---

## 11. Conclusion

This specification outlines a **two-phase strategy** for Ruchy to achieve world-class performance:

### Phase 1: IMMEDIATE AOT Optimizations (v3.174.0 - v3.180.0, 8 weeks)
**Beat Julia (1.32ms) by 24%, C (1.48ms) by 32%, and Rust (1.64ms) by 39%**

**Current State (v3.173.0):**
- BENCH-007: 1.67ms (10.20x faster than Python)
- Geometric mean: 13.04x (81% of C, 91% of Rust)
- **ROOT CAUSE:** `opt-level = "z"` sacrifices 15-20% performance for size

**Key Actions (NO JIT REQUIRED - Just Compiler Flags + Optimizations):**
1. ⚡ **Cargo.toml profiles** (v3.174.0 - 1 week) - **IMMEDIATE 28% SPEEDUP**
   - Change `opt-level = "z"` → `opt-level = 3` (DEFAULT)
   - Add `release-ultra` (PGO) and `release-tiny` (embedded)
   - Target: 1.20ms (BEAT Julia), binary < 500KB

2. 🚀 **AST Optimizations** (v3.176.0 - v3.178.0 - 4 weeks)
   - Constant folding, dead code elimination (10-20% speedup)
   - Tail-call optimization (CRITICAL for recursion - 15-25% speedup)
   - Aggressive inlining (5-30% speedup on call-heavy code)
   - Loop unrolling (10-40% on tight loops)
   - Strength reduction (5-15% on arithmetic-heavy code)
   - Target: 1.05-1.10ms (BEAT C)

3. ⚙️ **Bytecode VM** (v3.179.0 - v3.180.0 - 2 weeks)
   - NaN-boxing (30% faster, zero heap for primitives)
   - Inline caching (5-10x for method dispatch)
   - Target: 1.00ms (world-class)

**Expected Results (v3.180.0):**
- **Speed:** 1.00ms vs Julia 1.32ms (24% faster) ⚡
- **Geometric mean:** 18.50x vs C 16.04x (15% faster) 🚀
- **Binary size:** 540KB (release), 95KB (release-tiny)
- **Timeline:** 8 weeks total
- **Strategy:** Make `release` (opt=3) the DEFAULT, provide `release-tiny` for embedded

**What This Means:**
- ✅ Ruchy BEATS Julia on raw performance (without JIT!)
- ✅ Ruchy BEATS C (native code!) on benchmarks
- ✅ Ruchy BEATS Rust (memory safety + speed)
- ✅ Tiny binaries STILL available via `--profile release-tiny`
- ✅ Zero code changes to Ruchy language - just better compilation

### Phase 2: Julia-Style JIT+LLVM (v4.0+, 6-12 months) - **FUTURE WORK**
**Further improvements: 50-100x speedup on hot paths**

Long-term investment for:
- Tiered execution (interpret → Cranelift → LLVM)
- Type specialization (per-signature compilation)
- Method caching (amortize JIT cost)
- Target: Match Julia's 21.78x geometric mean

**Why Phase 2 After Phase 1:**
- Phase 1 achieves 18.50x (85% of Julia's 21.78x) with ZERO JIT complexity
- Phase 2 would require 6-12 months of development for 17% additional speedup
- Phase 1 is a **better ROI**: 42% speedup in 8 weeks vs 17% speedup in 6 months
- Users get world-class performance NOW, not in a year

---

**Document Version:** 2.0
**Last Updated:** 2025-11-02
**Status:** ACTIVE - Phase 1 Ready for Immediate Implementation

**BREAKING CHANGE:** v3.174.0 changes default `release` profile from `opt="z"` (size) to `opt=3` (speed).
**Migration:** Users requiring tiny binaries should use `cargo build --profile release-tiny` or `ruchy compile --profile release-tiny`.
**Rationale:** Most users prioritize speed over size; embedded users can opt into size optimization.
