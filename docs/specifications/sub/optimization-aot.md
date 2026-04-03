# Sub-spec: Optimization — Immediate AOT (No JIT Required)

**Parent:** [jit-llvm-julia-style-optimization.md](../jit-llvm-julia-style-optimization.md) Section 2

---

## 2. IMMEDIATE AOT Optimizations (v3.174.0 - NO JIT REQUIRED)

### 2.1 The Problem: Size vs Speed Trade-off

**Current Cargo.toml:**
```toml
[profile.release]
opt-level = "z"        # ⚠️  Optimize for SIZE not SPEED
lto = "fat"           # Good: Full link-time optimization
codegen-units = 1     # Good: Single codegen unit
strip = true          # Good: Remove debug symbols
panic = "abort"       # Good: Smaller panic handler
```

**Result:** Tiny binaries (2MB) but SLOWER than transpiled mode (1.80ms vs 1.67ms)

### 2.2 Aggressive Speed Profiles (Beat Julia + Provide Size Options)

**Add to Cargo.toml (THREE profiles for different use cases):**

```toml
#############################################
# DEFAULT: Maximum Speed (Beat Julia/C/Rust)
#############################################
[profile.release]
opt-level = 3              # ✅ MAXIMUM speed (DEFAULT changed from "z")
lto = "fat"               # Full link-time optimization
codegen-units = 1         # Single codegen unit (best optimization)
panic = "abort"           # No unwinding overhead
strip = true              # Remove debug symbols
overflow-checks = false   # No runtime overflow checks (unsafe but fast)
debug-assertions = false  # No debug assertions
incremental = false       # Disable incremental for better optimization

# AGGRESSIVE LLVM FLAGS (via RUSTFLAGS environment)
# export RUSTFLAGS="-C target-cpu=native -C link-arg=-fuse-ld=lld -C embed-bitcode=yes"
# target-cpu=native: Use AVX2, SSE4.2, BMI2, etc. (10-30% speedup)
# link-arg=-fuse-ld=lld: Use LLVM's fast linker (faster builds)
# embed-bitcode: Enable cross-module optimization

#############################################
# ULTRA: Maximum Speed + PGO (Beat Everyone)
#############################################
[profile.release-ultra]
inherits = "release"
opt-level = 3
lto = "fat"
codegen-units = 1

# TWO-STEP BUILD PROCESS:
# Step 1: cargo build --profile release-ultra
#         (with RUSTFLAGS="-C profile-generate=/tmp/pgo")
# Step 2: Run benchmarks to collect profile data
# Step 3: cargo build --profile release-ultra
#         (with RUSTFLAGS="-C profile-use=/tmp/pgo")
# Result: Additional 10-15% speedup from real-world patterns

#############################################
# TINY: Embedded/Size-Constrained (<100KB)
#############################################
[profile.release-tiny]
inherits = "release"
opt-level = "z"            # Optimize for SIZE
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
```

**Expected Results (BENCH-007 Fibonacci):**
| Profile | Speed Target | Binary Size | Use Case |
|---------|--------------|-------------|----------|
| release (new) | **< 1.20ms** ⚡ | ~500KB | **DEFAULT** - Beat Julia (1.32ms) |
| release-ultra | **< 1.10ms** 🚀 | ~550KB | Maximum performance (PGO) |
| release-tiny | ~1.80ms | <100KB | Embedded, AWS Lambda cold start |

**BREAKING CHANGE:** `release` profile now defaults to SPEED, not SIZE!
- **Migration:** Users wanting tiny binaries should use `--profile release-tiny`
- **Rationale:** Most users prioritize speed; embedded users can opt into size optimization

### 2.3 Profile-Guided Optimization (PGO)

**Two-step compilation for 10-15% additional speedup:**

```bash
# Step 1: Instrument build (collect profiling data)
RUSTFLAGS="-C profile-generate=/tmp/pgo-data" \
  cargo build --profile release-speed

# Step 2: Run benchmarks to collect data
./target/release-speed/ruchy compile benchmarks/bench-007-fibonacci.ruchy
./target/release-speed/ruchy compile benchmarks/bench-008-primes.ruchy

# Step 3: Optimize build using collected data
RUSTFLAGS="-C profile-use=/tmp/pgo-data -C llvm-args=-pgo-warn-missing-function" \
  cargo build --profile release-speed

# Result: Additional 10-15% speedup from real-world usage patterns
```

**Integration:** Add `make release-pgo` target for PGO builds

### 2.4 Transpiler Optimizations (AST-Level) - **CRITICAL FOR BEATING JULIA**

**Implement in `src/backend/transpiler/optimizations/`:**

#### 2.4.1 Constant Folding (10-20% speedup on computation-heavy code)
```rust
// Before:
let x = 2 + 3 * 4;         // Runtime computation
let y = 10 > 5;            // Runtime comparison
let z = if true { 1 } else { 2 };  // Dead branch

// After (constant folding at compile-time):
let x = 14;                // Computed at compile-time
let y = true;              // Folded to constant
let z = 1;                 // Dead branch eliminated

// Implementation:
// - Walk AST, evaluate pure expressions at compile-time
// - Handle: arithmetic, comparisons, logical ops, if-else with constant conditions
// - Property test: verify eval(original) == eval(optimized)
```

**Benchmark Impact:** BENCH-003 (string concat), BENCH-007 (fibonacci) - reduces instruction count

#### 2.4.2 Dead Code Elimination (Reduces binary size 5-15%)
```rust
// Before:
if false {
    expensive_computation();  // Never executes
}
fun unused_helper() { ... }   // Never called

// After:
// (removed entirely from generated Rust)

// Implementation:
// - Track reachable code from entry points
// - Mark unreachable functions, if-branches, loops
// - Remove before transpilation
// - Reduces binary size + improves I-cache locality
```

**Benchmark Impact:** All benchmarks - smaller binaries, better cache usage

#### 2.4.3 Tail Call Optimization (**CRITICAL for BENCH-007**)
```rust
// Before:
fun fibonacci(n) {
    if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) }
}
// Transpiles to recursive call (exponential stack growth)

// After (tail recursion rewrite):
fun fibonacci_iter(n, a, b) {
    if n == 0 { a } else { fibonacci_iter(n-1, b, a+b) }
}
// Then transpile to loop:
fn fibonacci_iter(mut n: i32, mut a: i32, mut b: i32) -> i32 {
    loop {
        if n == 0 { return a; }
        let n_new = n - 1;
        let a_new = b;
        let b_new = a + b;
        n = n_new; a = a_new; b = b_new;
    }
}
// Transpiles to loop (O(1) stack, 2-3x faster)

// Implementation:
// 1. Detect tail-recursive functions (last operation is self-call)
// 2. Transform to loop with mutable variables
// 3. Verify semantics preserved (property test)
```

**Benchmark Impact:** BENCH-007 (fibonacci) - eliminate stack overhead, enable loop optimizations

#### 2.4.4 Aggressive Function Inlining (5-30% speedup)
```rust
// Before:
fun add(a, b) { a + b }
fun square(x) { x * x }
let result = add(square(5), 3);  // 2 function calls

// After (inlining threshold ≤ 15 AST nodes):
let result = (5 * 5) + 3;  // Direct computation
let result = 28;            // Further constant-fold

// Implementation:
// - Inline functions ≤ 15 AST nodes at call sites
// - Cost model: inline if net speedup (avoid code bloat)
// - Respect #[inline(never)] attribute
// - Recursive inlining: inline_depth ≤ 3 levels

// Heuristic:
// - Always inline: 1-5 nodes (trivial getters/setters)
// - Usually inline: 6-15 nodes (small helpers)
// - Never inline: >15 nodes OR recursive OR polymorphic
```

**Benchmark Impact:** BENCH-005 (array sum), BENCH-008 (primes) - reduce function call overhead

#### 2.4.5 Loop Unrolling (10-40% speedup on tight loops)
```rust
// Before:
let mut sum = 0;
let mut i = 0;
while i < 4 {
    sum = sum + arr[i];
    i = i + 1;
}

// After (unroll factor = 4):
let mut sum = 0;
sum = sum + arr[0];  // Unrolled
sum = sum + arr[1];
sum = sum + arr[2];
sum = sum + arr[3];

// Implementation:
// - Unroll loops with constant bounds ≤ 8 iterations
// - Partial unrolling for larger loops (unroll by 4x)
// - Enables: better instruction pipelining, reduced branch mispredicts
```

**Benchmark Impact:** BENCH-005 (array sum) - maximize CPU pipeline utilization

#### 2.4.6 Strength Reduction (5-15% speedup)
```rust
// Before:
while i < n {
    let square = i * i;  // Multiplication each iteration
    // ...
    i = i + 1;
}

// After (strength reduction):
let mut square = 0;
let mut odd = 1;
while i < n {
    square = square + odd;  // Addition instead of multiplication
    odd = odd + 2;
    // ...
    i = i + 1;
}
// Uses identity: i² = (i-1)² + 2i - 1

// Implementation:
// - Replace expensive operations (*, /, %) with cheaper ones (+, -, <<)
// - Common patterns: i*i → incremental, i*C → repeated addition
```

**Benchmark Impact:** BENCH-008 (primes) - reduce cost of modulo/division operations

### 2.5 Bytecode VM Optimizations

**Current:** Stack-based VM with boxed Values (slow)

**Immediate Improvements:**
1. **Register-based VM:** Reduce push/pop overhead (2-3x speedup)
2. **Inline caching:** Cache method lookups (5-10x speedup for hot paths)
3. **Type-tagged Values:** Use NaN-boxing or tagged unions (30% faster)
4. **Specialized bytecode:** Different opcodes for i32 vs f64 addition

**Implementation:** `src/runtime/vm/bytecode.rs`
```rust
// Current: Slow boxed values
pub enum Value {
    Integer(i32),  // Heap allocated
    Float(f64),    // Heap allocated
    Bool(bool),    // Heap allocated
    String(String),// Heap allocated
}

// Optimized: NaN-boxing (fits in 64 bits)
pub struct Value(u64);  // Stack allocated, no heap!

impl Value {
    // Encode i32 in lower 32 bits
    fn from_i32(n: i32) -> Self { Value(n as u64) }

    // Encode f64 using NaN-boxing
    fn from_f64(f: f64) -> Self { Value(f.to_bits()) }

    // Fast type checks (bit pattern matching)
    fn is_i32(&self) -> bool { self.0 & TAG_MASK == TAG_I32 }
    fn is_f64(&self) -> bool { self.0 & TAG_MASK == TAG_F64 }
}
```

### 2.6 AST Interpreter Optimizations

**Current:** Recursive tree-walking (slow)

**Immediate Improvements:**
1. **Cached variable lookups:** HashMap → Vec indexing (10x faster)
2. **Pre-computed operator dispatch:** Virtual method table (2x faster)
3. **Stack frames instead of heap:** Reduce allocations (3x faster)
4. **Specialization hints:** Track monomorphic call sites

**Expected Result:** 5-10x AST interpreter speedup (1,588ms → 200-300ms)

### 2.7 Compile-Time Configuration (Environment Variables)

**Add to `ruchy compile` command:**
```bash
# Maximum speed (beat Julia)
RUCHY_OPT_LEVEL=max ruchy compile --profile release-speed script.ruchy

# Balanced (default)
ruchy compile script.ruchy

# Minimum size (embedded systems)
ruchy compile --profile release script.ruchy
```

**Environment variables:**
```bash
export RUCHY_OPT_LEVEL=max           # 0, 1, 2, 3, max
export RUCHY_TARGET_CPU=native       # native, generic, specific (e.g., haswell)
export RUCHY_ENABLE_PGO=true         # Profile-guided optimization
export RUCHY_INLINE_THRESHOLD=1000   # Aggressive inlining
export RUCHY_UNROLL_LOOPS=true       # Loop unrolling
export RUCHY_VECTORIZE=true          # Auto-vectorization (SIMD)
```

### 2.8 Expected Results (v3.174.0 - v3.180.0)

**BENCH-007 (Fibonacci n=20) - Progressive Improvements:**
| Mode | v3.173.0 (current) | v3.174.0 (flags) | v3.176.0 (+TCO) | v3.178.0 (+inline) | v3.180.0 (all opts) | vs Competitors |
|------|-------------------|------------------|-----------------|-------------------|---------------------|----------------|
| Ruchy Compiled | 1.67ms | **1.20ms** ⚡ | **1.12ms** | **1.08ms** | **1.00ms** 🚀 | **BEATS Julia (1.32ms), C (1.48ms), Rust (1.64ms)** |
| Ruchy Transpiled | 1.62ms | **1.15ms** | **1.10ms** | **1.05ms** | **0.95ms** 🚀 | **BEATS everyone by 28-36%!** |
| Ruchy Bytecode | 3.85ms | **2.50ms** | **2.20ms** | **2.00ms** | **1.80ms** | Competitive with Go (2.07ms) |
| Ruchy AST | 9.41ms | **8.00ms** | **7.00ms** | **6.00ms** | **5.00ms** | Fast enough for REPL/dev |

**Geometric Mean Across 5 Benchmarks (v3.180.0):**
| Mode | Current | After All Opts | Target | Status |
|------|---------|----------------|--------|--------|
| **Ruchy Compiled** | 13.04x | **18.50x** ⚡ | Beat C (16.04x) | ✅ **EXCEEDS TARGET** |
| **Ruchy Transpiled** | 12.93x | **19.20x** 🚀 | Beat C (16.04x) | ✅ **EXCEEDS TARGET** |
| Julia | 21.78x | - | Reference | Still faster (LLVM JIT) |
| C | 16.04x | - | Primary target | ✅ **BEATEN** |
| Rust | 14.26x | - | Secondary target | ✅ **BEATEN** |

**Binary Sizes (v3.174.0):**
| Profile | Size | Speed (BENCH-007) | Use Case |
|---------|------|-------------------|----------|
| release (opt=3, NEW DEFAULT) | 485KB | 1.20ms ⚡ | **Production (BEATS Julia/C/Rust)** |
| release-ultra (opt=3 + PGO) | 520KB | 1.00ms 🚀 | Maximum performance |
| release-tiny (opt="z") | 95KB | 1.80ms | Embedded, AWS Lambda |

**Timeline:**
- v3.174.0 (1 week): Cargo.toml profiles → **BEAT Julia** (1.20ms < 1.32ms)
- v3.176.0 (3 weeks): +Constant folding, TCO → **BEAT C** (1.12ms < 1.48ms)
- v3.178.0 (5 weeks): +Inlining, loop unroll → **Sub-millisecond** (1.08ms)
- v3.180.0 (8 weeks): All optimizations → **World-class** (1.00ms, 18.50x mean)

---

