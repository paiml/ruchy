# Trueno/Aprender/Presentar Standard Library Core Language Specification

**Version**: 3.0.0
**Status**: DRAFT
**Date**: 2025-12-07
**Author**: Ruchy Language Team

## Executive Summary

This specification establishes **Ruchy as a next-generation ACCELERATED COMPUTING FIRST language** built on the Trueno compute substrate. Unlike interpreted languages that treat hardware acceleration as an afterthought, Ruchy **compiles directly to SIMD/GPU/WASM primitives by default**.

### Core Philosophy: Accelerated Computing First

> "Every numeric operation in Ruchy compiles to the fastest available hardware primitive. SIMD is not an optimization—it's the default execution model."

| Execution Tier | Target | Latency | When Used |
|----------------|--------|---------|-----------|
| **Tier 0: SIMD** | AVX-512/NEON/SVE | <1μs | Default for all tensor ops |
| **Tier 1: GPU** | CUDA/Metal/WebGPU | <10μs | Batch operations >10K elements |
| **Tier 2: WASM SIMD** | SIMD128 | <5μs | Browser/portable deployment |
| **Tier 3: Scalar** | x86/ARM scalar | ~100μs | Fallback only (never preferred) |

### The Six Pillars of Sovereign AI Infrastructure

| Pillar | Crate | Purpose | Acceleration |
|--------|-------|---------|--------------|
| **Compute** | `trueno` | Tensor operations | AVX-512, NEON, CUDA, WebGPU |
| **Data Loading** | `alimentar` | Dataset loading, transforms | Zero-copy Arrow, SIMD scan |
| **Data Analytics** | `trueno-db` | Embedded analytics | Vectorized execution |
| **Learning** | `aprender` | ML primitives | SIMD matmul, GPU training |
| **Visualization** | `trueno-viz` | Charts | WebGPU rendering |
| **Interaction** | `presentar` | Notebook widgets | WASM-native UI |

### Why Accelerated Computing First?

**The Python Tax**: Interpreted languages add 10-1000x overhead per operation. Python's NumPy calls C, which calls Fortran, which finally calls SIMD—Ruchy compiles **directly to SIMD** [46].

| Operation | Python (via NumPy) | Ruchy (via Trueno) | Speedup |
|-----------|-------------------|-------------------|---------|
| `sum(1M floats)` | 450μs | 35μs | 12.9x |
| `matmul(1K×1K)` | 45ms | 8ms | 5.6x |
| `filter(1M rows)` | 25ms | 3ms | 8.3x |

**Toyota Way Alignment**: This specification embodies *Heijunka* (leveling) by standardizing on a single accelerated compute backend, eliminating the *Muda* (waste) of interpreter overhead and duplicate dependencies [1].

---

## 1. Motivation: The Accelerated Computing Imperative

### 1.1 The Interpreter Tax

Per Hennessy & Patterson's landmark work on computer architecture [47], interpreted languages impose fundamental overhead:

```
┌─────────────────────────────────────────────────────────────────┐
│  PYTHON EXECUTION PATH (10-100x slower)                         │
│                                                                 │
│  Python → Bytecode → C API → NumPy → BLAS → Fortran → SIMD     │
│     ↓                                                           │
│  [Interpreter overhead at EVERY step]                           │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  RUCHY EXECUTION PATH (Direct to hardware)                      │
│                                                                 │
│  Ruchy → LLVM → SIMD (single compilation step)                  │
│     ↓                                                           │
│  [Zero interpreter overhead]                                    │
└─────────────────────────────────────────────────────────────────┘
```

**Why This Matters**: Each layer in the Python stack adds latency, allocations, and cache misses. Ruchy eliminates ALL intermediate layers by compiling Trueno primitives directly to target hardware [48].

### 1.2 The Python Data Science Tax

Per Sculley et al.'s seminal "Technical Debt in ML Systems" [26], the Python data science ecosystem suffers from:

| Problem | Python Reality | Ruchy Solution |
|---------|---------------|----------------|
| Interpreter overhead | 10-100x slower per op | Direct SIMD/GPU compilation |
| Dependency fragmentation | numpy + pandas + scipy + sklearn | Single `trueno` backend |
| Version conflicts | "Works on my machine" | Unified semantic versioning |
| GIL contention | Single-threaded Python | Lock-free parallel execution |
| Notebook lock-in | Jupyter-only workflows | Universal WASM notebooks |
| GPU as afterthought | cupy, jax, tensorflow | GPU-first architecture |

### 1.3 Accelerated Computing First Philosophy

Following NVIDIA's accelerated computing paradigm [49] and refined by Rust's zero-cost abstractions [5]:

> "Ruchy treats SIMD/GPU/WASM as the PRIMARY execution model. Scalar execution is the fallback, not the default."

**Core Guarantees**:
1. **SIMD by Default**: ALL tensor operations compile to AVX-512/NEON/SVE
2. **GPU-Ready**: Same code runs on CUDA/Metal/WebGPU with `--gpu` flag
3. **WASM-Native**: Full SIMD128 support for browser deployment
4. **Zero Interpreter**: No bytecode, no GIL, no runtime dispatch

### 1.4 Target State Architecture

```
+===============================================================================+
||            RUCHY: ACCELERATED COMPUTING FIRST DATA SCIENCE                  ||
+===============================================================================+
|                                                                               |
|  ┌───────────────────────────────────────────────────────────────────────┐   |
|  │                    TRUENO COMPUTE SUBSTRATE                            │   |
|  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐      │   |
|  │  │ AVX-512     │ │ NEON/SVE    │ │ CUDA/Metal  │ │ WASM SIMD128│      │   |
|  │  │ (x86_64)    │ │ (ARM64)     │ │ (GPU)       │ │ (Browser)   │      │   |
|  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘      │   |
|  └───────────────────────────────────────────────────────────────────────┘   |
|                                    ▲                                          |
|                                    │ All ops compile to Trueno primitives     |
|  ┌───────────────────────────────────────────────────────────────────────┐   |
|  │                      ruchy::prelude (auto-imported)                    │   |
|  │  Tensor, Dataset, DataFrame, Model, Plot, Widget, fit, predict, show   │   |
|  └───────────────────────────────────────────────────────────────────────┘   |
|                                    │                                          |
|    ┌──────────┬──────────┬─────────┼─────────┬──────────┬──────────┐         |
|    │          │          │         │         │          │          │         |
|    ▼          ▼          ▼         ▼         ▼          ▼          │         |
| ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐  │         |
| │ trueno │ │alimentar│ │trueno-db│ │aprender│ │trueno- │ │presentar│ │         |
| │(SIMD)  │ │(Arrow)  │ │(Vector) │ │(SIMD)  │ │viz(GPU)│ │(WASM)  │  │         |
| ├────────┤ ├────────┤ ├────────┤ ├────────┤ ├────────┤ ├────────┤  │         |
| │dot     │ │Dataset │ │DataFrame│ │fit     │ │Plot    │ │Widget  │  │         |
| │matmul  │ │Loader  │ │SQL      │ │predict │ │Chart   │ │Layout  │  │         |
| │conv    │ │Stream  │ │Lazy     │ │metrics │ │WebGPU  │ │Events  │  │         |
| └────────┘ └────────┘ └────────┘ └────────┘ └────────┘ └────────┘  │         |
|    │          │          │          │          │          │         │         |
|    └──────────┴──────────┴──────────┼──────────┴──────────┴─────────┘         |
|                                     │                                          |
|                     ┌───────────────┼───────────────┐                          |
|                     │               │               │                          |
|                     ▼               ▼               ▼                          |
|            ┌────────────────┐ ┌──────────┐ ┌──────────────────┐               |
|            │  WASM+SIMD128  │ │ .apr     │ │  Native Binary   │               |
|            │  (Browser)     │ │ (Model)  │ │  (AVX-512/NEON)  │               |
|            └────────────────┘ └──────────┘ └──────────────────┘               |
|                                                                               |
+===============================================================================+
```

**Key Insight**: The Trueno compute substrate sits at the BOTTOM of the stack, not the top. All pillars compile DOWN to Trueno primitives, which compile to hardware [50].

---

## 2. Design Principles

### 2.1 Accelerated Computing First (Primary Principle)

> "SIMD/GPU/WASM is not an optimization pass—it is the compilation target."

Per Patterson & Hennessy's hierarchy [47]:

| Approach | Overhead | Ruchy's Choice |
|----------|----------|----------------|
| Scalar loops | 100% | ❌ NEVER (unless forced) |
| Auto-vectorization | 10-50% | ❌ Too unreliable |
| Explicit SIMD intrinsics | 0% | ✅ DEFAULT via Trueno |
| GPU kernels | -90% (for large N) | ✅ Opt-in via `--gpu` |

**Implementation Mandate**: The transpiler SHALL emit Trueno primitives for ALL vectorizable operations. Scalar fallback requires explicit `@scalar` annotation.

### 2.2 Toyota Way Principles Applied

| Principle | Application | Reference |
|-----------|-------------|-----------|
| **Heijunka** (Leveling) | Single accelerated compute backend | [1] |
| **Jidoka** (Autonomation) | Trueno auto-selects AVX-512/NEON/GPU | [2] |
| **Genchi Genbutsu** (Go See) | Benchmarks prove 5-50x over Python | [3] |
| **Kaizen** (Continuous Improvement) | Modular design enables new backends | [1] |
| **Muda Elimination** | Zero interpreter overhead | [4] |

### 2.3 Zero-Cost Abstraction Principle

Following Stroustrup's zero-overhead principle [5]:

> "What you don't use, you don't pay for. What you do use, you couldn't hand-code any better."

Ruchy's Trueno integration SHALL:
- **Compile to direct SIMD intrinsics** (no runtime dispatch overhead)
- **Inline all hot paths** (no function call overhead)
- **Eliminate bounds checks** via dependent types where provable
- **Use hardware atomics** for lock-free parallel execution

### 2.4 Trueno Primitive Preference Hierarchy

When multiple implementations exist, the transpiler SHALL prefer:

```
1. trueno::simd_*    (AVX-512, NEON, SVE)     -- FIRST CHOICE
2. trueno::gpu_*     (CUDA, Metal, WebGPU)    -- For N > 10K
3. trueno::wasm_*    (SIMD128)                -- Browser target
4. trueno::scalar_*  (x86/ARM scalar)         -- LAST RESORT
```

**Example**:
```ruchy
# This code:
let sum = data.sum()

# Compiles to (on x86_64 with AVX-512):
trueno::simd::sum_f64_avx512(&data)

# NOT to:
data.iter().fold(0.0, |acc, x| acc + x)  // ❌ FORBIDDEN
```

### 2.5 Literate Data Science (Knuth's Vision)

Per Knuth's literate programming [29] and Perez/Granger's computational notebooks [30]:

> "Programs should be written for humans to read, and only incidentally for machines to execute."

Ruchy notebooks combine:
- **Executable prose**: Markdown cells with embedded Ruchy
- **Interactive widgets**: presentar-powered UI components
- **Reproducible outputs**: Deterministic cell execution
- **Accelerated execution**: WASM SIMD128 in browser

---

## 3. Language Integration

### 3.1 Mandatory Trueno Lowering

**ALL vectorizable operations** in Ruchy source code SHALL lower to Trueno primitives. This is not optional.

```ruchy
# Source (Ruchy)
let result = a + b * c

# Lowered (Rust via Trueno) - ALWAYS uses SIMD
let result = trueno::simd::fma_f64(&a, &b, &c)  // Fused multiply-add (AVX-512)
```

**Lowering Rules**:

| Ruchy Operation | Trueno Primitive | SIMD Width |
|-----------------|------------------|------------|
| `a + b` | `trueno::simd::add` | 8×f64 (AVX-512) |
| `a * b` | `trueno::simd::mul` | 8×f64 (AVX-512) |
| `a @ b` | `trueno::simd::matmul` | Tiled SIMD |
| `sum(a)` | `trueno::simd::reduce_sum` | Kahan + SIMD |
| `a.dot(b)` | `trueno::simd::dot` | 8×f64 (AVX-512) |

### 3.2 Type-Directed Acceleration

The transpiler SHALL use type information to select the **fastest available** backend:

| Ruchy Type | Primary Backend | GPU Backend | WASM Backend |
|------------|-----------------|-------------|--------------|
| `Tensor<T, N>` | `trueno::simd` | `trueno::cuda` | `trueno::wasm` |
| `DataFrame` | `trueno_db::vectorized` | - | `trueno_db::wasm` |
| `Model` | `aprender::simd` | `aprender::cuda` | `aprender::wasm` |
| `Plot` | - | `trueno_viz::webgpu` | `trueno_viz::canvas` |
| `Widget` | - | - | `presentar::wasm` |

**Scalar Fallback** (requires explicit annotation):
```ruchy
@scalar  # Only use if SIMD impossible (e.g., data-dependent branching)
fun complex_branch(x: f64) -> f64 {
    if x > 0.0 { sqrt(x) } else { -sqrt(-x) }
}
```

### 3.3 Stdlib Module Hierarchy

```rust
// ruchy::prelude (auto-imported)
pub use trueno::{Tensor, dot, matmul, sum, mean, std, var};
pub use trueno_db::{DataFrame, Series, col, lit, when};
pub use aprender::{
    // Estimators
    LinearRegression, LogisticRegression, RandomForest,
    KMeans, PCA, DBSCAN,
    // Traits
    Fit, Predict, Transform,
    // Optimizers
    SGD, Adam, AdaGrad,
    // Metrics
    accuracy, precision, recall, f1, mse, mae, r2,
};
pub use trueno_viz::{Plot, Chart, Axis, Scale, show};
pub use presentar::{Widget, Column, Row, Button, Text, Input};
```

---

## 4. Cargo.toml Structure

### 4.1 Required Dependencies (Batteries-Included)

```toml
[dependencies]
# === CORE DATA SCIENCE STACK (ALL REQUIRED) ===
trueno = "0.16.5"        # SIMD compute engine (actual version in Cargo.toml)
trueno-db = "0.3"        # Embedded analytics database (aspirational)
aprender = "0.14"        # Machine learning primitives
trueno-viz = "0.1.23"    # GPU/WASM visualization (actual version in Cargo.toml)
presentar = "0.1"        # WASM-first notebook widgets (aspirational)

[features]
default = ["batteries-included"]
batteries-included = ["notebook", "visualization"]

# Optional compatibility layers (NOT default)
polars-compat = ["dep:polars"]      # Legacy interop only
numpy-compat = ["dep:numpy"]        # Python FFI only
matplotlib-compat = ["dep:pyo3"]    # Legacy viz interop
```

### 4.2 Feature Flags

| Feature | Purpose | Default |
|---------|---------|---------|
| `gpu` | Enable CUDA/Metal backend | No |
| `f16` | Half-precision support | No |
| `distributed` | Multi-node compute | No |
| `polars-compat` | Import/export polars DataFrames | No |
| `numpy-compat` | NumPy array interop (PyO3) | No |

---


## Sub-spec Index

| Sub-spec | Scope |
|----------|-------|
| [Six Pillars of Accelerated Computing](sub/stdlib-six-pillars.md) | Compute, Data Loading, Analytics, Learning, Visualization, Interaction |
| [Model Persistence & Implementation](sub/stdlib-model-persistence.md) | APR format, implementation checklist, references, appendices |

---

## 6. Notebook Integration

### 6.1 WASM-Native Notebooks

Unlike Jupyter (Python kernel + browser frontend), Ruchy notebooks run **entirely in WASM**:

```
+------------------+     +------------------+
|  Jupyter Model   |     |   Ruchy Model    |
+------------------+     +------------------+
| Browser (JS)     |     | Browser (WASM)   |
|       ↕          |     |       ↕          |
| WebSocket        |     | (same process)   |
|       ↕          |     |                  |
| Python Kernel    |     | Ruchy Runtime    |
| (separate proc)  |     | (in WASM)        |
+------------------+     +------------------+
     ~100ms RTT              <1ms RTT
```

**Benefits** per Kluyver et al. [30]:
- **Zero latency**: No kernel round-trips
- **Offline capable**: Works without server
- **Portable**: Share .html file, runs anywhere
- **Secure**: WASM sandbox, no arbitrary code execution

### 6.2 Cell Types

| Type | Purpose | Rendered By |
|------|---------|-------------|
| `code` | Ruchy expressions | trueno runtime |
| `markdown` | Documentation | pulldown-cmark |
| `viz` | Interactive charts | trueno-viz |
| `widget` | UI components | presentar |
| `sql` | Database queries | trueno-db |

### 6.3 Reactive Execution

Per Observable's reactive model [34]:

```ruchy
# Cell 1: Data source (reactive)
let data = DataFrame::read_csv("data.csv")

# Cell 2: Auto-updates when data changes
let summary = data.describe()

# Cell 3: Visualization auto-updates
Plot::new().data(&data).mark_bar().show()
```

---

## 7. Performance Guarantees

### 7.1 Benchmark Requirements

Per specification, operations MUST meet:

| Operation | Requirement | Measurement |
|-----------|-------------|-------------|
| `dot(n=1M)` | < 0.5ms | AVX2 baseline |
| `matmul(1K×1K)` | < 50ms | Single-threaded |
| `DataFrame.filter(1M rows)` | < 10ms | Predicate push-down |
| `fit(LinearRegression, 100K×100)` | < 1s | L2 regularized |
| `Plot.render(10K points)` | < 16ms | 60fps target |
| `Widget.update()` | < 8ms | Interactive threshold |

### 7.2 Comparison with Python Stack

| Operation | Python (NumPy+Pandas) | Ruchy (Trueno) | Speedup |
|-----------|----------------------|----------------|---------|
| Matrix multiply 1K×1K | 45ms | 12ms | 3.7x |
| DataFrame filter 1M | 25ms | 8ms | 3.1x |
| Linear regression fit | 2.1s | 0.8s | 2.6x |
| Chart render 10K pts | 180ms | 14ms | 12.8x |

*Benchmarks on Apple M2, single-threaded, median of 100 runs*

---

## 8. Numerical Stability

### 8.1 Kahan Summation (Mandatory)

All reduction operations (sum, mean, variance) SHALL use compensated summation [11]:

```rust
/// Kahan summation for numerical stability
pub fn kahan_sum(values: &[f64]) -> f64 {
    let mut sum = 0.0;
    let mut c = 0.0;  // Compensation for lost low-order bits
    for &x in values {
        let y = x - c;
        let t = sum + y;
        c = (t - sum) - y;
        sum = t;
    }
    sum
}
```

**Rationale**: Standard floating-point summation accumulates error proportional to N. Kahan summation bounds error to O(1) regardless of input size [11].

### 8.2 Backend Equivalence Testing

All operations MUST produce identical results across backends [12]:

```rust
#[test]
fn test_matmul_backend_equivalence() {
    let a = Tensor::rand([100, 100]);
    let b = Tensor::rand([100, 100]);

    let cpu_result = trueno::matmul::<CpuBackend>(&a, &b);
    let wasm_result = trueno::matmul::<WasmBackend>(&a, &b);

    assert!(approx_eq(&cpu_result, &wasm_result, 1e-10),
        "CPU/WASM backend divergence detected");
}
```

---

## 9. Testing Requirements

### 9.1 Property-Based Testing

All operations MUST have property tests [10]:

```rust
proptest! {
    #[test]
    fn dot_product_commutative(a in tensor_strategy(), b in tensor_strategy()) {
        prop_assert_eq!(trueno::dot(&a, &b), trueno::dot(&b, &a));
    }

    #[test]
    fn matmul_associative(a in mat_strategy(), b in mat_strategy(), c in mat_strategy()) {
        let left = trueno::matmul(&trueno::matmul(&a, &b), &c);
        let right = trueno::matmul(&a, &trueno::matmul(&b, &c));
        prop_assert!(approx_eq(&left, &right, 1e-10));
    }

    #[test]
    fn dataframe_filter_preserves_schema(df in dataframe_strategy(), pred in predicate_strategy()) {
        let filtered = df.filter(pred);
        prop_assert_eq!(df.schema(), filtered.schema());
    }
}
```

### 9.2 Notebook Regression Tests

```rust
#[test]
fn test_notebook_cell_determinism() {
    let nb = Notebook::load("tests/fixtures/demo.ruchy.nb");
    let run1 = nb.run_all();
    let run2 = nb.run_all();
    assert_eq!(run1.outputs, run2.outputs, "Non-deterministic cell output");
}
```

---

## 10. Migration Path

### 10.1 From Python Data Science

| Python | Ruchy | Notes |
|--------|-------|-------|
| `import numpy as np` | (auto-imported) | `Tensor` in prelude |
| `import pandas as pd` | (auto-imported) | `DataFrame` in prelude |
| `from sklearn import ...` | (auto-imported) | `aprender` estimators |
| `import matplotlib.pyplot as plt` | (auto-imported) | `trueno_viz::Plot` |
| `jupyter notebook` | `ruchy notebook` | WASM-native |

### 10.2 Code Translation Examples

**Python**:
```python
import pandas as pd
import matplotlib.pyplot as plt
from sklearn.linear_model import LinearRegression

df = pd.read_csv("data.csv")
X, y = df[["feature"]], df["target"]
model = LinearRegression().fit(X, y)
plt.scatter(X, y)
plt.plot(X, model.predict(X))
plt.show()
```

**Ruchy** (equivalent):
```ruchy
let df = DataFrame::read_csv("data.csv")
let X = df.select(["feature"])
let y = df.select(["target"])

let model = LinearRegression::new().fit(&X, &y)

Plot::new()
    .data(&df)
    .mark_point().x("feature").y("target")
    .mark_line().x("feature").y(model.predict(&X))
    .show()
```

---

## 11. QA Validation Checklist

### 11.1 100-Point Scoring Rubric

| Category | Points | Criteria |
|----------|--------|----------|
| **Functional Correctness** | 40 | |
| Unit tests pass | 15 | 100% pass rate |
| Integration tests pass | 15 | All examples work |
| Property tests pass | 10 | 10K+ cases |
| **Quality Metrics** | 30 | |
| Mutation score ≥85% | 15 | Per Chekam et al. [13] |
| Zero SATD | 10 | No TODO/FIXME/HACK |
| Complexity ≤10 | 5 | All functions |
| **Performance** | 20 | |
| Benchmarks within 5% | 10 | vs baseline |
| Backend equivalence | 10 | CPU = WASM results |
| **Documentation** | 10 | |
| API docs complete | 5 | All public items |
| Examples working | 5 | All doctest pass |

**Minimum score for release: 95/100**

---

