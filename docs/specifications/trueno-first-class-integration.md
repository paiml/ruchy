# Sub-spec: Trueno First-Class Compute Substrate Integration

**Parent:** [trueno-aprender-stdlib-core-language-spec.md](trueno-aprender-stdlib-core-language-spec.md)
**Version:** 1.0.0
**Status:** DRAFT
**Date:** 2026-04-03

---

## 0. Prerequisites

1. **Current dependency**: trueno 0.15 — upgrade to 0.16.5 required for ComputeBrick, tiling, quantization, and hardware-aware dispatch.
2. **Current bridge scope**: `src/stdlib/trueno_bridge.rs` exposes 11 functions (kahan_sum, kahan_sum_f32, dot_f32, dot, mean, variance, std_dev, best_backend, add_f32, mul_f32, scale_f32), not 8 as previously documented.
3. **Trueno 0.16.5 API verification**: API surface referenced in this spec (Vector64, ComputeBrick, BrickProfiler, BrickTuner, RooflineParams, HardwareCapability, TiledQ4KMatvec, select_backend_for_operation) is based on source inspection of `../trueno` — verify against published crate docs before implementation.
4. **Array element type inference**: Ruchy's transpiler must be extended with array element type inference before type-directed lowering can work (see Section 2 prerequisite note).
5. **Decorator grammar**: `@verified`, `@tuned`, `@gpu` decorators require parser extension — see `provable-contracts-language-integration.md` for the decorator grammar.

## 1. Overview

### 1.1 Current State

The trueno bridge (`src/stdlib/trueno_bridge.rs`) exposes 11 functions covering approximately 15% of the trueno public API:

| Function | Type | SIMD-Accelerated |
|----------|------|-------------------|
| `kahan_sum` | f64 reduction | No (pure Rust) |
| `kahan_sum_f32` | f32 reduction | No (pure Rust) |
| `dot_f32` | f32 reduction | Yes |
| `dot` | f64 reduction (via f32) | Yes (with precision loss) |
| `mean` | f64 reduction | No (Kahan wrapper) |
| `variance` | f64 reduction | No (Kahan wrapper) |
| `std_dev` | f64 reduction | No (Kahan wrapper) |
| `add_f32` | f32 element-wise | Yes |
| `mul_f32` | f32 element-wise | Yes |
| `scale_f32` | f32 element-wise | Yes |
| `best_backend` | runtime query | N/A |

The current bridge is a thin stdlib wrapper. Array arithmetic in Ruchy source code is not lowered to trueno ops by the transpiler; users must call bridge functions explicitly.

### 1.2 Target State

The transpiler emits `trueno::Vector` and `trueno::Matrix` operations directly when it detects array-typed operands. The dependency upgrades from `0.15` to `0.16.5`, unlocking ComputeBrick, hardware-aware dispatch, tiling, and quantized inference APIs (see Prerequisites for API verification caveat).

### 1.3 Success Criteria

1. Arithmetic on `f32[]`/`f64[]` arrays transpiles to trueno SIMD calls with zero user annotation.
2. Coverage of trueno API rises from ~15% to >60% (42+ operations).
3. All backends (Scalar, SSE2, AVX2, AVX-512, NEON, WASM SIMD128) produce equivalent results within IEEE 754 tolerance.
4. Benchmark suite demonstrates measurable speedup over scalar baseline.

## 2. Transpiler-Level Integration

### 2.1 Type-Directed Lowering

The transpiler's type inference pass already tracks numeric types. When both operands of a binary expression resolve to `f32[]` or `f64[]`, the backend emits trueno vector calls instead of scalar loops.

> **Prerequisite**: Ruchy's transpiler must be extended with array element type inference (currently tracks annotations but not inferred array element types).

| Ruchy Source | Inferred Types | Transpiled Rust |
|-------------|----------------|-----------------|
| `a + b` | `f32[], f32[]` | `trueno::Vector::from_slice(&a).add(&trueno::Vector::from_slice(&b))?` |
| `a - b` | `f32[], f32[]` | `trueno::Vector::from_slice(&a).sub(&trueno::Vector::from_slice(&b))?` |
| `a * b` | `f32[], f32[]` | `trueno::Vector::from_slice(&a).mul(&trueno::Vector::from_slice(&b))?` |
| `a / b` | `f32[], f32[]` | `trueno::Vector::from_slice(&a).div(&trueno::Vector::from_slice(&b))?` |
| `a.dot(b)` | `f32[], f32[]` | `trueno::Vector::from_slice(&a).dot(&trueno::Vector::from_slice(&b))?` |
| `a @ b` | `Matrix, Matrix` | `trueno::Matrix::matmul(&a_mat, &b_mat)?` |

### 2.2 Auto-Dispatch via Backend Selection

The transpiler injects a backend selection preamble into the generated `main()`:

```rust
let _backend = trueno::select_backend_for_operation(
    trueno::OperationHint::VectorArithmetic,
    data_len,
);
```

This allows trueno to choose the optimal SIMD width at runtime (e.g., AVX-512 for large vectors, scalar for vectors shorter than the SIMD lane width).

### 2.3 f64 Native Support

The current bridge converts f64 to f32 for SIMD, losing precision. With trueno 0.16.5, native f64 SIMD paths are available:

```rust
// Before (0.15): precision loss
let a_f32: Vec<f32> = a.iter().map(|&x| x as f32).collect();

// After (0.16.5): native f64 SIMD
let va = trueno::Vector64::from_slice(&a);
let vb = trueno::Vector64::from_slice(&b);
let result = va.dot(&vb)?;
```

### 2.4 Transpiler Implementation Points

Modifications required in the transpiler backend:

1. `src/backend/transpiler/statements.rs` -- detect array-typed binary ops, emit trueno calls.
2. `src/backend/transpiler/effects.rs` -- handle `Result` unwrapping from fallible trueno ops.
3. `src/backend/transpiler/program_transpiler.rs` -- inject `use trueno::...` imports and backend preamble.
4. `src/backend/transpiler/mod.rs` -- register trueno type mappings in the type environment.

## 3. Extended Operation Coverage

### 3.1 Arithmetic Operations

| Ruchy Syntax | trueno API | Category | Notes |
|-------------|-----------|----------|-------|
| `a + b` | `Vector::add()` | element-wise | existing |
| `a - b` | `Vector::sub()` | element-wise | new |
| `a * b` | `Vector::mul()` | element-wise | existing |
| `a / b` | `Vector::div()` | element-wise | new |
| `a * scalar` | `Vector::scale()` | broadcast | existing |
| `fma(a, b, c)` | `Vector::fma()` | fused multiply-add | new |

### 3.2 Reduction Operations

| Ruchy Syntax | trueno API | Category | Notes |
|-------------|-----------|----------|-------|
| `a.sum()` | `Vector::sum()` | reduction | new (replaces kahan_sum wrapper) |
| `a.dot(b)` | `Vector::dot()` | reduction | existing |
| `a.max()` | `Vector::max()` | reduction | new |
| `a.min()` | `Vector::min()` | reduction | new |
| `a.argmax()` | `Vector::argmax()` | reduction | new |
| `a.argmin()` | `Vector::argmin()` | reduction | new |
| `a.mean()` | `Vector::mean()` | statistical | new (replaces mean wrapper) |
| `a.variance()` | `Vector::variance()` | statistical | new (replaces variance wrapper) |
| `a.stddev()` | `Vector::stddev()` | statistical | new (replaces std_dev wrapper) |

### 3.3 Transform Operations

| Ruchy Syntax | trueno API | Category | Notes |
|-------------|-----------|----------|-------|
| `abs(a)` | `Vector::abs()` | unary | new |
| `clamp(a, lo, hi)` | `Vector::clamp()` | ternary | new |
| `lerp(a, b, t)` | `Vector::lerp()` | interpolation | new |
| `sqrt(a)` | `Vector::sqrt()` | unary | new |
| `recip(a)` | `Vector::recip()` | unary | new |
| `pow(a, n)` | `Vector::pow()` | binary | new |

### 3.4 Activation Functions

| Ruchy Syntax | trueno API | Category | Notes |
|-------------|-----------|----------|-------|
| `relu(a)` | `Vector::relu()` | activation | new |
| `sigmoid(a)` | `Vector::sigmoid()` | activation | new |
| `gelu(a)` | `Vector::gelu()` | activation | new |
| `silu(a)` | `Vector::silu()` | activation | new |
| `tanh(a)` | `Vector::tanh()` | activation | new |

### 3.5 Normalization Operations

| Ruchy Syntax | trueno API | Category | Notes |
|-------------|-----------|----------|-------|
| `zscore(a)` | `Vector::zscore()` | normalization | new |
| `layer_norm(a)` | `Vector::layer_norm()` | normalization | new |
| `minmax_normalize(a)` | `Vector::minmax_normalize()` | normalization | new |

### 3.6 Matrix Operations

| Ruchy Syntax | trueno API | Category | Notes |
|-------------|-----------|----------|-------|
| `a.T` or `transpose(a)` | `Matrix::transpose()` | layout | new |
| `a @ b` | `Matrix::matmul()` | multiply | new |
| `matvec(a, v)` | `Matrix::matvec()` | multiply | new |
| `conv1d(x, kernel)` | `Matrix::convolution()` | signal | new |
| `embedding(ids, table)` | `Matrix::embedding()` | lookup | new |
| `pool(x, size)` | `Matrix::pooling()` | downsample | new |

### 3.7 Coverage Summary

| Category | Count | Status |
|----------|-------|--------|
| Arithmetic | 6 | 3 existing, 3 new |
| Reductions | 9 | 1 existing, 8 new |
| Transforms | 6 | all new |
| Activations | 5 | all new |
| Normalization | 3 | all new |
| Matrix | 6 | all new |
| Runtime/Utility | 1 | existing (best_backend) |
| **Total** | **36** | 5 existing, 31 new |

Combined with the 6 bridge-only functions retained (`kahan_sum`, `kahan_sum_f32`, `dot`, `mean`, `variance`, `std_dev`), total coverage reaches 42 operations.

## 4. ComputeBrick Integration

### 4.1 Self-Verifying Compute Units

A ComputeBrick is a self-verifying unit of computation. When a Ruchy function is decorated with `@verified` (see Prerequisites #5 for parser requirements), the transpiler wraps it in a `ComputeBrick` that carries correctness assertions and a compute budget.

```ruchy
@verified(tolerance=1e-6, budget_us=100)
fun normalize(data: f32[]) -> f32[]:
    let mu = data.mean()
    let sigma = data.stddev()
    return (data - mu) / sigma
```

Transpiles to:

```rust
fn normalize_brick() -> trueno::ComputeBrick<Vec<f32>, Vec<f32>> {
    trueno::ComputeBrick::new("normalize")
        .with_tolerance(1e-6)
        .with_budget_us(100)
        .with_compute(|data: &[f32]| {
            let v = trueno::Vector::from_slice(data);
            let mu = v.mean()?;
            let sigma = v.stddev()?;
            let centered = v.add_scalar(-mu)?;
            centered.scale(1.0 / sigma).map(|r| r.to_vec())
        })
        .with_assert_equiv(|input, output| {
            let out_mean = trueno::Vector::from_slice(output).mean()?;
            Ok(out_mean.abs() < 1e-6) // normalized mean near zero
        })
        .build()
}
```

### 4.2 BrickProfiler

Execution telemetry is available via `BrickProfiler`:

```ruchy
@verified(profile=true)
fun softmax(logits: f32[]) -> f32[]:
    let mx = logits.max()
    let shifted = logits - mx
    let exps = exp(shifted)
    return exps / exps.sum()
```

The profiler records wall time, SIMD utilization, cache miss ratio, and backend used. Results are queryable at runtime via `brick.profile()`.

### 4.3 Execution Graphs

Multiple bricks compose into a `BrickGraph` for pipeline optimization:

```ruchy
@pipeline
fun inference(tokens: i32[]) -> f32[]:
    let emb = embedding(tokens, WEIGHTS)
    let normed = layer_norm(emb)
    let out = matvec(PROJ, normed)
    return softmax(out)
```

The transpiler fuses adjacent bricks when safe (no side effects, matching types) and allocates shared buffers to eliminate intermediate copies.

## 5. Hardware-Aware Dispatch

### 5.1 HardwareCapability Detection

The transpiler emits a one-time capability probe at program startup:

```rust
let hw = trueno::HardwareCapability::detect();
// hw.simd_width: 128 | 256 | 512
// hw.has_fma: bool
// hw.l1_cache_bytes: usize
// hw.num_cores: usize
```

### 5.2 Automatic GPU Dispatch

> **Safety note**: GPU interop relies on trueno's internal unsafe abstraction — Ruchy-generated code remains safe per zero-unsafe policy (Issue #132).

When the `gpu` feature is enabled, vectors exceeding a size threshold are dispatched to GPU:

```rust
if data.len() > 10_000 && hw.has_gpu() {
    trueno::gpu::matmul(&a, &b) // WebGPU/CUDA/Metal
} else {
    trueno::Matrix::matmul(&a, &b) // CPU SIMD
}
```

The threshold of 10,000 elements is the default. Users override it with:

```ruchy
@gpu(threshold=5000)
fun large_matmul(a: f32[][], b: f32[][]) -> f32[][]:
    return a @ b
```

### 5.3 RooflineParams

Roofline model parameters inform the dispatch decision:

```rust
let roofline = trueno::RooflineParams {
    peak_flops: hw.peak_gflops() * 1e9,
    peak_bandwidth: hw.memory_bandwidth_gbps() * 1e9,
    operational_intensity: flops_per_byte(op),
};
let is_compute_bound = roofline.is_compute_bound();
```

This prevents dispatching memory-bound operations to GPU where the PCIe transfer cost exceeds the compute benefit.

### 5.4 Feature Flags

| Feature | Gate | Default | Description |
|---------|------|---------|-------------|
| CPU SIMD | (always on) | yes | AVX2/AVX-512/NEON auto-detected |
| `parallel` | `trueno/parallel` | no | Rayon-based multi-core parallelism |
| `gpu` | `trueno/gpu` | no | WebGPU/CUDA/Metal backend |
| `ml-tuner` | `trueno/ml-tuner` | no | Adaptive kernel selection |
| `tiling` | `trueno/tiling` | no | Cache-aware tiled operations |

## 6. Tiling and Quantization

### 6.1 TiledQ4KMatvec

For LLM inference workloads, trueno 0.16.5 provides cache-aware tiled matrix-vector multiply with 4-bit quantization:

```ruchy
@quantized(bits=4, tiled=true)
fun llm_forward(weights: q4k[][], input: f32[]) -> f32[]:
    return matvec(weights, input)
```

Transpiles to:

```rust
let result = trueno::tiling::TiledQ4KMatvec::new(tile_m, tile_n)
    .with_cache_line(hw.l1_cache_bytes)
    .execute(&weights, &input)?;
```

### 6.2 Higher-Precision Quantization

| Format | Bits | Use Case | trueno API |
|--------|------|----------|-----------|
| Q4K | 4 | Aggressive compression, mobile | `TiledQ4KMatvec` |
| Q5K | 5 | Balanced quality/size | `q5k_dot_product()` |
| Q6K | 6 | High-quality inference | `q6k_dot_product()` |
| f16 | 16 | Training, fine-tuning | `Vector::from_f16()` |

### 6.3 BrickTuner (ML-Tuner)

The adaptive kernel selector profiles multiple implementations and selects the fastest for the current hardware:

```rust
let tuner = trueno::BrickTuner::new()
    .add_candidate("naive", naive_matmul)
    .add_candidate("tiled_4k", tiled_q4k_matmul)
    .add_candidate("tiled_6k", tiled_q6k_matmul)
    .with_warmup(10)
    .with_trials(100);

let best = tuner.select(&sample_input)?;
// best.name: "tiled_4k"
// best.median_ns: 4200
```

In Ruchy, this is exposed via the `@tuned` decorator:

```ruchy
@tuned(warmup=10, trials=100)
fun inference_kernel(w: f32[][], x: f32[]) -> f32[]:
    return matvec(w, x)
```

## 7. Cargo.toml Changes

### 7.1 Dependency Upgrade

```toml
# Before
trueno = { version = "0.15", default-features = false }

# After
trueno = { version = "0.16.5", default-features = false }
```

### 7.2 Feature Gates

```toml
[features]
default = ["simd"]
simd = []  # CPU SIMD always available via trueno

# Extended trueno features (opt-in)
gpu = ["trueno/gpu"]
parallel = ["trueno/parallel"]
ml-tuner = ["trueno/ml-tuner"]
tiling = ["trueno/tiling"]
sovereign-stack = [
    "dep:alimentar", "dep:entrenar", "dep:trueno-viz", "dep:presentar",
    "gpu", "parallel", "tiling"
]
```

### 7.3 Migration Checklist

1. Update `trueno` version in `Cargo.toml` from `0.15` to `0.16.5`.
2. Run `cargo update trueno` to resolve transitive dependencies.
3. Update `src/stdlib/trueno_bridge.rs` to use new API surface (e.g., `Vector64`).
4. Add `use trueno::...` imports to transpiler output preamble.
5. Run full test suite: `cargo test --all-features`.
6. Verify WASM target still compiles: `cargo build --target wasm32-unknown-unknown --no-default-features`.

## 8. Testing Requirements

### 8.1 Backend Equivalence Tests

Every operation in Section 3 must have a backend equivalence test verifying that the trueno SIMD result matches a scalar baseline within IEEE 754 tolerance.

Pattern (extending existing `backend_equivalence_tests` module):

```rust
#[test]
fn test_sub_matches_scalar_baseline() {
    let a: Vec<f32> = (0..256).map(|i| i as f32 * 0.1).collect();
    let b: Vec<f32> = (0..256).map(|i| i as f32 * 0.05).collect();
    let simd = sub_f32(&a, &b).unwrap();
    let scalar: Vec<f32> = a.iter().zip(b.iter()).map(|(x, y)| x - y).collect();
    for (i, (s, r)) in simd.iter().zip(scalar.iter()).enumerate() {
        assert!((s - r).abs() < 1e-6, "sub mismatch at {i}");
    }
}
```

### 8.2 Property Tests for Numerical Stability

Extend the existing `proptest` suite with properties that exercise Kahan-compensated paths:

| Property | Assertion |
|----------|-----------|
| Commutativity | `add(a, b) == add(b, a)` element-wise |
| Associativity (approximate) | `\|sum(a++b) - (sum(a) + sum(b))\| < eps` |
| Scale identity | `scale(a, 1.0) == a` |
| Dot self non-negative | `dot(a, a) >= 0.0` |
| Normalization mean | `\|mean(zscore(a))\| < 1e-6` for non-constant `a` |
| Activation bounds | `0 <= sigmoid(x) <= 1` for all `x` |
| ReLU non-negative | `relu(x) >= 0` for all `x` |

Minimum case count: 10,000 per property (matching existing `proptest_config`).

### 8.3 Benchmark Suite

Create `benches/trueno_simd.rs` using `criterion`, benchmarking each operation at sizes [256, 1K, 4K, 16K, 64K, 256K, 1M] against a scalar baseline.

Benchmark categories:

| Category | Operations | Input Sizes |
|----------|-----------|-------------|
| Vector arithmetic | add, sub, mul, div, scale, fma | 256 to 1M |
| Reductions | sum, dot, max, min, mean | 256 to 1M |
| Transforms | abs, clamp, sqrt, relu, sigmoid | 256 to 1M |
| Matrix | matmul, matvec, transpose | 64x64 to 1024x1024 |
| Quantized | Q4K matvec, Q5K dot, Q6K dot | 256 to 64K |

Acceptance: trueno SIMD path must be at least 2x faster than scalar baseline for vectors of 1024+ elements.

### 8.4 Transpiler Integration Tests

End-to-end tests verifying the transpile-compile-execute pipeline:

```rust
#[test]
fn test_transpiler_emits_trueno_vector_add() {
    let source = r#"
        let a = [1.0, 2.0, 3.0, 4.0]
        let b = [5.0, 6.0, 7.0, 8.0]
        let c = a + b
        print(c)
    "#;
    let rust_code = transpile(source).unwrap();
    assert!(rust_code.contains("trueno::Vector"));
    assert!(rust_code.contains(".add("));

    let output = compile_and_run(&rust_code).unwrap();
    assert_eq!(output.trim(), "[6.0, 8.0, 10.0, 12.0]");
}
```

### 8.5 Mutation Testing

Run targeted mutation testing on the trueno bridge and transpiler lowering:

```bash
cargo mutants --file src/stdlib/trueno_bridge.rs --timeout 300
cargo mutants --file src/backend/transpiler/statements.rs --timeout 300
```

Target: at least 75% CAUGHT/MISSED ratio for both files.
