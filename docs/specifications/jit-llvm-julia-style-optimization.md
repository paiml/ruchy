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

**Current Benchmark Results (BENCH-007 Fibonacci n=20):**
```
🥇 Julia:            1.35ms  (13.05x faster) ⚡ TARGET TO BEAT
🥈 Ruchy Transpiled: 1.67ms  (10.55x faster) 🦀 BEATS RUST
🥉 Rust:             1.70ms  (10.36x faster)
   Ruchy Compiled:   1.80ms  ( 9.79x faster) ⚠️  SLOWER THAN TRANSPILED!
```

**Root Cause:** Current `opt-level = "z"` (size) instead of `opt-level = 3` (speed)

**Immediate Actions (v3.174.0 - 2 weeks):**
1. Add `[profile.release-speed]` with maximum performance flags
2. Enable LTO, PGO (Profile-Guided Optimization), and target-cpu=native
3. Pre-configure aggressive transpiler optimizations (tail-call, constant folding, dead code elimination)
4. **Target:** < 1.30ms (BEAT Julia by 5%) with <500KB binary size

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

### 2.2 Aggressive Speed Profile (Beat Julia by 5%)

**Add to Cargo.toml:**
```toml
[profile.release-speed]
inherits = "release"
opt-level = 3              # ✅ MAXIMUM speed (not size)
lto = "fat"               # Full link-time optimization
codegen-units = 1         # Better optimization (slower compile)
panic = "abort"           # No unwinding overhead
strip = true              # Remove symbols
overflow-checks = false   # No runtime overflow checks (unsafe but fast)
debug-assertions = false  # No debug assertions

# AGGRESSIVE LLVM FLAGS (via RUSTFLAGS environment)
# RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C embed-bitcode=yes"
# target-cpu=native: Use CPU-specific instructions (AVX2, SSE4.2, etc.)
# embed-bitcode: Enable cross-module optimization
```

**Expected Result:**
- Speed: < 1.30ms (BEAT Julia's 1.35ms by 5%)
- Binary size: ~500KB (larger than "z" but still tiny)
- **Use case:** `ruchy compile --release-speed script.ruchy`

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

### 2.4 Transpiler Optimizations (AST-Level)

**Implement in `src/backend/transpiler/`:**

#### 2.4.1 Constant Folding
```rust
// Before:
let x = 2 + 3 * 4;  // Transpiles to: let x = 2 + 3 * 4;

// After (constant folding):
let x = 14;  // Transpiles to: let x = 14;
```

#### 2.4.2 Dead Code Elimination
```rust
// Before:
if false {
    expensive_computation();  // Dead code
}

// After:
// (removed entirely)
```

#### 2.4.3 Tail Call Optimization
```rust
// Before:
fun factorial(n, acc) {
    if n == 0 { acc } else { factorial(n - 1, n * acc) }
}
// Transpiles to recursive call (stack overflow risk)

// After (tail call optimization):
fn factorial(mut n: i32, mut acc: i32) -> i32 {
    loop {
        if n == 0 { return acc; }
        let n_new = n - 1;
        let acc_new = n * acc;
        n = n_new;
        acc = acc_new;
    }
}
// Transpiles to loop (constant stack usage)
```

#### 2.4.4 Inline Small Functions
```rust
// Before:
fun add(a, b) { a + b }
let result = add(5, 3);  // Function call overhead

// After (inlining):
let result = 5 + 3;  // Direct computation
```

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

### 2.8 Expected Results (v3.174.0)

**BENCH-007 (Fibonacci n=20):**
| Mode | Current | After Optimizations | Speedup | vs Julia |
|------|---------|---------------------|---------|----------|
| Ruchy Compiled | 1.80ms | **1.25ms** ⚡ | 1.44x | **BEATS Julia (1.35ms)** |
| Ruchy Transpiled | 1.67ms | **1.20ms** | 1.39x | BEATS Julia |
| Ruchy Bytecode | 3.76ms | **1.50ms** | 2.5x | Competitive |
| Ruchy AST | 140ms | **20ms** | 7x | Usable for dev |

**Binary Sizes:**
| Profile | Size | Use Case |
|---------|------|----------|
| release (opt="z") | 2MB | Embedded, minimal footprint |
| release-speed (opt=3) | 500KB | Production performance |
| release-speed + PGO | 450KB | Maximum performance + small |

**Timeline:** v3.174.0 release (2 weeks)

---

## 3. Julia-Style JIT Architecture (Long-term)

### 3.1 How Julia Achieves Near-Native Performance

```
Julia Execution Flow:
─────────────────────

1. Parse & Lower to IR (one-time)
   source.jl → AST → Typed IR

2. Type Inference (runtime profiling)
   function add(a, b)      # Called with (5, 3)
   └→ Inferred: add(Int, Int) → Int

3. LLVM Code Generation (specialized)
   define i64 @add_Int_Int(i64 %a, i64 %b) {
     %result = add i64 %a, %b
     ret i64 %result
   }

4. LLVM Optimization & JIT Compile
   LLVM IR → Optimized IR → Native x86_64 assembly

5. Cache & Execute (subsequent calls)
   add(5, 3) → Lookup cache → Execute native code
```

### 3.2 Key Principles

1. **Lazy Compilation:** Only compile what's executed
2. **Type Specialization:** Generate different native code for different type combinations
3. **Method Cache:** Store compiled versions indexed by type signature
4. **Tiered Execution:**
   - Tier 0: Interpret (cold code, <10 calls)
   - Tier 1: Quick compile (warm code, 10-100 calls)
   - Tier 2: LLVM full optimization (hot code, 100+ calls)

---

## 4. Ruchy JIT+LLVM Design

### 4.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Ruchy v4.0 (Julia-Style)                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Tier 0: AST Interpreter (Cold Path)                          │  │
│  │ - First execution: Parse → AST → Interpret                   │  │
│  │ - Profile: Track call counts, type observations              │  │
│  │ - Decision: If hotness > threshold → promote to Tier 1       │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                              ↓                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Tier 1: Quick JIT (Warm Path)                                │  │
│  │ - Simple codegen: Direct x86_64 assembly (via Cranelift)     │  │
│  │ - No optimization: Fast compile, decent performance          │  │
│  │ - Continue profiling: Track types, inline candidates         │  │
│  │ - Decision: If hotness > threshold → promote to Tier 2       │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                              ↓                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Tier 2: LLVM Full Optimization (Hot Path)                    │  │
│  │ - Type specialization: Generate per-type-signature versions  │  │
│  │ - LLVM IR generation: From typed AST                         │  │
│  │ - Full optimization: -O3, inlining, vectorization, etc.      │  │
│  │ - Cache: Store in method table indexed by type signature     │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Method Cache (Global)                                         │  │
│  │ HashMap<(FunctionName, TypeSignature), CompiledCode>         │  │
│  │                                                               │  │
│  │ Example:                                                      │  │
│  │ ("is_prime", [i32]) → 0x7f8a4c0012a0 (Tier 2, native code)  │  │
│  │ ("add", [i32, i32]) → 0x7f8a4c001500 (Tier 2, native code)  │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.2 Core Components

#### 4.2.1 Execution Engine

```rust
pub struct RuchyExecutionEngine {
    /// AST interpreter for cold code
    interpreter: ASTInterpreter,

    /// Quick JIT compiler (Cranelift)
    quick_jit: CraneliftJIT,

    /// LLVM JIT compiler (inkwell)
    llvm_jit: LLVMJITEngine,

    /// Method cache: (function, type_sig) → compiled code
    method_cache: MethodCache,

    /// Profiler: Tracks hotness and type observations
    profiler: RuntimeProfiler,

    /// Configuration
    config: JITConfig,
}

pub struct JITConfig {
    /// Promote to Tier 1 after N calls
    tier1_threshold: usize,  // Default: 10

    /// Promote to Tier 2 after N calls
    tier2_threshold: usize,  // Default: 100

    /// Enable LLVM optimizations
    llvm_opt_level: OptLevel,  // Default: Aggressive

    /// Maximum cached methods
    max_cached_methods: usize,  // Default: 10000
}
```

#### 4.2.2 Type Specialization

```rust
/// Type signature for method specialization
#[derive(Hash, Eq, PartialEq, Clone)]
pub struct TypeSignature {
    params: Vec<ConcreteType>,
    return_type: ConcreteType,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum ConcreteType {
    Int32,
    Int64,
    Float64,
    Bool,
    String,
    Vec(Box<ConcreteType>),
    Function(Vec<ConcreteType>, Box<ConcreteType>),
}

/// Example specialization:
/// Ruchy function: fun add(a, b) { a + b }
///
/// Compiled versions:
/// - add(i32, i32) -> i32  (one LLVM function)
/// - add(f64, f64) -> f64  (different LLVM function)
/// - add(String, String) -> String  (different LLVM function)
```

#### 4.2.3 Method Cache

```rust
pub struct MethodCache {
    /// Cache of compiled methods
    cache: HashMap<MethodKey, CompiledMethod>,

    /// LRU eviction policy
    lru: LRUList,
}

#[derive(Hash, Eq, PartialEq)]
struct MethodKey {
    function_name: String,
    type_signature: TypeSignature,
}

struct CompiledMethod {
    /// Tier level (1 = Cranelift, 2 = LLVM)
    tier: u8,

    /// Function pointer to native code
    native_fn: *const (),

    /// Metadata for debugging
    metadata: MethodMetadata,
}
```

#### 4.2.4 Runtime Profiler

```rust
pub struct RuntimeProfiler {
    /// Call counts per function
    call_counts: HashMap<String, usize>,

    /// Observed type signatures per function
    type_observations: HashMap<String, Vec<TypeSignature>>,

    /// Execution time tracking
    execution_times: HashMap<MethodKey, Duration>,
}

impl RuntimeProfiler {
    /// Record a function call with observed types
    pub fn record_call(&mut self, func: &str, args: &[Value]) {
        // Increment call count
        *self.call_counts.entry(func.to_string()).or_insert(0) += 1;

        // Record type signature
        let sig = TypeSignature::from_values(args);
        self.type_observations
            .entry(func.to_string())
            .or_default()
            .push(sig);
    }

    /// Check if function should be promoted to next tier
    pub fn should_promote(&self, func: &str, current_tier: u8) -> bool {
        let count = self.call_counts.get(func).copied().unwrap_or(0);
        match current_tier {
            0 => count >= self.config.tier1_threshold,
            1 => count >= self.config.tier2_threshold,
            _ => false,
        }
    }
}
```

---

## 5. LLVM Integration (inkwell)

### 5.1 LLVM IR Generation

```rust
use inkwell::*;
use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;

pub struct LLVMCodegen<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> LLVMCodegen<'ctx> {
    /// Generate LLVM IR for a Ruchy function
    pub fn codegen_function(
        &self,
        ast: &Expr,
        type_sig: &TypeSignature,
    ) -> Result<FunctionValue<'ctx>> {
        match &ast.kind {
            ExprKind::Function { name, params, body, .. } => {
                // Create function signature
                let param_types: Vec<_> = type_sig.params.iter()
                    .map(|t| self.llvm_type(t))
                    .collect();
                let return_type = self.llvm_type(&type_sig.return_type);

                let fn_type = return_type.fn_type(&param_types, false);
                let function = self.module.add_function(name, fn_type, None);

                // Create entry basic block
                let entry = self.context.append_basic_block(function, "entry");
                self.builder.position_at_end(entry);

                // Generate IR for function body
                let return_value = self.codegen_expr(body, &type_sig)?;
                self.builder.build_return(Some(&return_value));

                Ok(function)
            }
            _ => bail!("Expected function expression"),
        }
    }

    /// Generate LLVM IR for an expression
    fn codegen_expr(
        &self,
        expr: &Expr,
        type_sig: &TypeSignature,
    ) -> Result<BasicValueEnum<'ctx>> {
        match &expr.kind {
            // Integer literal
            ExprKind::Literal(Literal::Integer(n)) => {
                Ok(self.context.i32_type().const_int(*n as u64, false).into())
            }

            // Binary operation (specialized!)
            ExprKind::Binary { op, left, right } => {
                let lhs = self.codegen_expr(left, type_sig)?;
                let rhs = self.codegen_expr(right, type_sig)?;

                match op {
                    BinaryOp::Add => {
                        // Type-specialized addition
                        match &type_sig.return_type {
                            ConcreteType::Int32 => {
                                let result = self.builder.build_int_add(
                                    lhs.into_int_value(),
                                    rhs.into_int_value(),
                                    "add"
                                );
                                Ok(result.into())
                            }
                            ConcreteType::Float64 => {
                                let result = self.builder.build_float_add(
                                    lhs.into_float_value(),
                                    rhs.into_float_value(),
                                    "fadd"
                                );
                                Ok(result.into())
                            }
                            _ => bail!("Unsupported add type"),
                        }
                    }
                    BinaryOp::Multiply => {
                        let result = self.builder.build_int_mul(
                            lhs.into_int_value(),
                            rhs.into_int_value(),
                            "mul"
                        );
                        Ok(result.into())
                    }
                    BinaryOp::Less => {
                        let result = self.builder.build_int_compare(
                            IntPredicate::SLT,
                            lhs.into_int_value(),
                            rhs.into_int_value(),
                            "lt"
                        );
                        Ok(result.into())
                    }
                    // ... other operators
                    _ => bail!("Unsupported operator: {:?}", op),
                }
            }

            // Variable reference
            ExprKind::Identifier(name) => {
                // Look up in local variables (would need proper scope tracking)
                self.lookup_variable(name)
            }

            // Function call
            ExprKind::Call { func, args } => {
                self.codegen_call(func, args, type_sig)
            }

            // While loop
            ExprKind::While { condition, body, .. } => {
                self.codegen_while_loop(condition, body, type_sig)
            }

            // If expression
            ExprKind::If { condition, then_branch, else_branch } => {
                self.codegen_if(condition, then_branch, else_branch.as_deref(), type_sig)
            }

            _ => bail!("Unsupported expression: {:?}", expr.kind),
        }
    }

    /// Generate optimized while loop
    fn codegen_while_loop(
        &self,
        condition: &Expr,
        body: &Expr,
        type_sig: &TypeSignature,
    ) -> Result<BasicValueEnum<'ctx>> {
        let current_fn = self.builder.get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        // Create basic blocks
        let loop_header = self.context.append_basic_block(current_fn, "loop");
        let loop_body = self.context.append_basic_block(current_fn, "loop.body");
        let loop_exit = self.context.append_basic_block(current_fn, "loop.exit");

        // Jump to loop header
        self.builder.build_unconditional_branch(loop_header);

        // Loop header: evaluate condition
        self.builder.position_at_end(loop_header);
        let cond_value = self.codegen_expr(condition, type_sig)?
            .into_int_value();
        self.builder.build_conditional_branch(cond_value, loop_body, loop_exit);

        // Loop body
        self.builder.position_at_end(loop_body);
        self.codegen_expr(body, type_sig)?;
        self.builder.build_unconditional_branch(loop_header);

        // Loop exit
        self.builder.position_at_end(loop_exit);

        // Return unit
        Ok(self.context.i32_type().const_int(0, false).into())
    }

    /// Map Ruchy type to LLVM type
    fn llvm_type(&self, ty: &ConcreteType) -> BasicTypeEnum<'ctx> {
        match ty {
            ConcreteType::Int32 => self.context.i32_type().into(),
            ConcreteType::Int64 => self.context.i64_type().into(),
            ConcreteType::Float64 => self.context.f64_type().into(),
            ConcreteType::Bool => self.context.bool_type().into(),
            ConcreteType::String => {
                // String as i8* (pointer to char array)
                self.context.i8_type().ptr_type(AddressSpace::default()).into()
            }
            ConcreteType::Vec(elem_ty) => {
                // Vec as struct { ptr: *T, len: i64, capacity: i64 }
                let elem_type = self.llvm_type(elem_ty);
                let ptr_type = elem_type.ptr_type(AddressSpace::default());
                let len_type = self.context.i64_type();
                self.context.struct_type(
                    &[ptr_type.into(), len_type.into(), len_type.into()],
                    false
                ).into()
            }
            _ => panic!("Unsupported type: {:?}", ty),
        }
    }
}
```

### 5.2 Optimized BENCH-008 Example

```rust
// Ruchy source
fun is_prime(n) {
    if n < 2 { return false }
    if n == 2 { return true }
    if n % 2 == 0 { return false }
    let mut i = 3
    while i * i <= n {
        if n % i == 0 { return false }
        i = i + 2
    }
    true
}

// After type specialization: is_prime(i32) -> bool
// Generated LLVM IR (simplified):

define i1 @is_prime_i32(i32 %n) {
entry:
  %cmp1 = icmp slt i32 %n, 2
  br i1 %cmp1, label %return_false, label %check2

check2:
  %cmp2 = icmp eq i32 %n, 2
  br i1 %cmp2, label %return_true, label %check_even

check_even:
  %rem = srem i32 %n, 2
  %is_even = icmp eq i32 %rem, 0
  br i1 %is_even, label %return_false, label %loop_init

loop_init:
  br label %loop

loop:
  %i = phi i32 [ 3, %loop_init ], [ %i_next, %loop_body ]
  %i_squared = mul i32 %i, %i
  %continue = icmp sle i32 %i_squared, %n
  br i1 %continue, label %loop_body, label %return_true

loop_body:
  %rem2 = srem i32 %n, %i
  %divides = icmp eq i32 %rem2, 0
  br i1 %divides, label %return_false, label %loop_continue

loop_continue:
  %i_next = add i32 %i, 2
  br label %loop

return_false:
  ret i1 false

return_true:
  ret i1 true
}

// After LLVM optimization (O3):
// - Loop unrolling for small i values
// - Strength reduction (i*i → incremental)
// - Dead code elimination
// - Register allocation
// Result: Near-native performance (~5ms vs 1,588ms)
```

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

## 10. IMMEDIATE Implementation Roadmap (v3.174.0 - Beat Julia NOW)

### 10.1 Phase 1A: Cargo.toml Optimization Profiles (v3.174.0 - 3 days)

**Ticket:** [PERF-001] Add release-speed profile with aggressive compiler flags

**Tasks:**
1. Add `[profile.release-speed]` to Cargo.toml
2. Configure: `opt-level=3, lto="fat", codegen-units=1, overflow-checks=false`
3. Document RUSTFLAGS: `-C target-cpu=native -C embed-bitcode=yes`
4. Add `make release-speed` target
5. Update `ruchy compile` to accept `--profile release-speed` flag

**Acceptance Criteria:**
- BENCH-007: < 1.30ms (vs Julia's 1.35ms)
- Binary size: < 600KB
- Zero regressions in test suite (4033 tests pass)

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

### 10.5 Validation & Benchmarking (Continuous)

**Every sprint validates performance targets:**

```bash
# Run BENCH-007 after each optimization
make bench-fibonacci

# Target progression:
v3.173.0:  1.80ms (baseline - size-optimized)
v3.174.0:  1.25ms (release-speed profile)  ✅ BEAT Julia
v3.175.0:  1.15ms (PGO)                    ✅ 15% better than Julia
v3.176.0:  1.10ms (+ constant folding)
v3.177.0:  1.05ms (+ tail-call opt)
v3.178.0:  1.00ms (+ inlining)             ✅ 35% better than Julia
v3.179.0:  0.95ms (+ NaN-boxing)
v3.180.0:  0.90ms (+ inline caching)       ✅ 50% better than Julia!
```

**Final Goal:** < 0.90ms (50% faster than Julia's 1.35ms) with <500KB binaries

---

## 11. Conclusion

This specification outlines a **two-phase strategy** for Ruchy performance:

### Phase 1: IMMEDIATE AOT Optimizations (v3.174.0 - v3.180.0, 8 weeks)
**Beat Julia (1.35ms) by 50% using aggressive compiler flags and AST optimizations**

**Key Actions:**
1. ✅ **Cargo.toml profiles** - Switch from size (`opt="z"`) to speed (`opt=3`)
2. ✅ **PGO** - Profile-guided optimization for 10-15% additional speedup
3. ✅ **Transpiler** - Constant folding, tail-call opt, inlining
4. ✅ **Bytecode VM** - NaN-boxing, inline caching for 3-10x improvements
5. ✅ **AST interpreter** - Cached lookups, stack frames for 5-10x speedup

**Expected Results:**
- **Speed:** 0.90ms (50% faster than Julia's 1.35ms)
- **Binary size:** <500KB (still tiny!)
- **Timeline:** 8 weeks (v3.174.0 - v3.180.0)
- **No JIT required:** Pure AOT compilation wins

### Phase 2: Julia-Style JIT+LLVM (v4.0+, 6-12 months)
**Adaptive optimization for 100-200x improvements on hot paths**

Long-term investment for:
- Tiered execution (interpret → Cranelift → LLVM)
- Type specialization (per-signature compilation)
- Method caching (amortize JIT cost)
- Near-native performance (within 2x of handwritten Rust)

---

**Document Version:** 2.0
**Last Updated:** 2025-11-02
**Status:** ACTIVE - Phase 1 Ready for Implementation
