# Sub-spec: Standard Library — Six Pillars of Accelerated Computing

**Parent:** [trueno-aprender-stdlib-core-language-spec.md](../trueno-aprender-stdlib-core-language-spec.md) Section 5

---

## 5. The Six Pillars of Accelerated Computing

### 5.1 Pillar 1: Compute (trueno) — THE FOUNDATION

**Purpose**: SIMD/GPU/WASM-accelerated tensor operations as the **UNIVERSAL COMPUTE SUBSTRATE**.

> "Trueno is not a library—it is the execution model."

```ruchy
# Matrix multiplication (DIRECT to SIMD, not auto-vectorized)
let a = Tensor::rand([1000, 1000])
let b = Tensor::rand([1000, 1000])
let c = a @ b  # Compiles to: trueno::simd::matmul_f64_avx512(&a, &b)
```

**Compilation Targets** (automatically selected at compile time):

| Platform | Backend | Intrinsics | Width |
|----------|---------|------------|-------|
| x86_64 | `trueno::avx512` | `_mm512_*` | 8×f64 |
| x86_64 (fallback) | `trueno::avx2` | `_mm256_*` | 4×f64 |
| ARM64 | `trueno::neon` | `vld1q_*` | 2×f64 |
| ARM64 (SVE) | `trueno::sve` | `svld1_*` | 2-8×f64 |
| WASM | `trueno::simd128` | `v128_*` | 2×f64 |
| CUDA | `trueno::cuda` | PTX | 256+ threads |
| Metal | `trueno::metal` | MSL | 256+ threads |

**Why Trueno, Not NumPy/ndarray?**

```
NumPy call stack (10+ layers):
  Python → C API → NumPy → BLAS → OpenBLAS → Fortran → SIMD

Trueno call stack (1 layer):
  Ruchy → trueno::simd::* (direct intrinsics)
```

**Performance Guarantee**: Trueno operations SHALL be within 5% of hand-written assembly [51].

### 5.2 Pillar 2: Data Loading (alimentar)

**Purpose**: Zero-copy Arrow-based data loading with transforms and drift detection [46].

```ruchy
# Load dataset from multiple sources
let train = Dataset::from_parquet("data/train.parquet")
let test = Dataset::from_hub("paiml/mnist")  # HuggingFace Hub

# Apply transforms (lazy, composable)
let processed = train
    .select(["features", "label"])
    .filter(col("label") < 5)
    .shuffle(seed=42)
    .normalize()

# Create batched DataLoader
let loader = DataLoader::new(processed)
    .batch_size(32)
    .num_workers(4)

for batch in loader {
    let (X, y) = batch.split_features_target("label")
    model.train_step(&X, &y)
}
```

**Key Features**:

| Feature | Description | Reference |
|---------|-------------|-----------|
| **Zero-Copy Arrow** | All data as Arrow RecordBatches | [9] |
| **Multiple Backends** | Local, S3, HTTP, HuggingFace Hub | [46] |
| **Streaming** | Memory-efficient lazy loading | [47] |
| **Transforms** | Filter, shuffle, sample, normalize | [48] |
| **Data Quality** | Null detection, duplicates, outliers | [49] |
| **Drift Detection** | KS test, Chi-square, PSI | [50] |
| **Federated Splits** | IID, Dirichlet partitioning | [51] |
| **Built-in Datasets** | MNIST, CIFAR-10, Iris, etc. | - |

**Drift Detection Example**:

```ruchy
# Monitor for data drift in production
let reference = Dataset::from_parquet("baseline.parquet")
let current = Dataset::from_parquet("today.parquet")

let drift = detect_drift(&reference, &current, method="ks_test")
if drift.p_value < 0.05 {
    alert("Data drift detected! Retrain model.")
}
```

**ALD Format** (Alimentar Dataset):

```ruchy
# Save dataset with metadata and versioning
dataset.save("train.ald", version="1.0.0", tags=["production"])

# Load with provenance tracking
let loaded = Dataset::load("train.ald")
print(f"Version: {loaded.version}, Hash: {loaded.sha256}")
```

### 5.3 Pillar 3: Data Analytics (trueno-db) — VECTORIZED QUERY ENGINE

**Purpose**: Embedded analytics database with **vectorized execution** (not row-at-a-time).

> "Process 1024 values per CPU cycle, not 1."

```ruchy
# DataFrame operations (vectorized, not interpreted)
let df = DataFrame::read_csv("sales.csv")
let result = df
    .filter(col("region") == "West")  # SIMD string comparison
    .group_by(["product"])            # Hash aggregation with SIMD
    .agg([col("revenue").sum()])      # Kahan sum via Trueno
    .sort("revenue", descending=true) # SIMD radix sort
    .collect()  # Executes optimized query plan
```

**Vectorized Execution** per MonetDB/X100 [16]:

| Operation | Row-at-a-time | Vectorized (Trueno) |
|-----------|---------------|---------------------|
| Filter 1M rows | 25ms | 3ms (8x faster) |
| Aggregate | 15ms | 2ms (7x faster) |
| Sort | 100ms | 12ms (8x faster) |
| Join | 200ms | 25ms (8x faster) |

**Why Vectorized?**
- Process 1024 rows per function call (amortizes call overhead)
- SIMD-friendly memory access patterns
- CPU cache-optimal batch processing
- Compiles to `trueno::simd::*` primitives

### 5.4 Pillar 4: Learning (aprender) — SIMD-ACCELERATED ML

**Purpose**: Production-ready ML estimators **built on Trueno primitives**.

> "Every matrix operation in aprender compiles to Trueno SIMD."

```ruchy
# Train a model (uses trueno::simd::matmul internally)
let model = RandomForest::new()
    .n_estimators(100)
    .max_depth(10)
model.fit(&X_train, &y_train)  # Compiles to SIMD decision tree splits

# Evaluate (SIMD-accelerated prediction)
let predictions = model.predict(&X_test)  # Vectorized tree traversal
let score = accuracy(&y_test, &predictions)
print(f"Accuracy: {score:.2%}")
```

**Aprender → Trueno Lowering**:

| ML Operation | Trueno Primitive | Acceleration |
|--------------|------------------|--------------|
| `LinearRegression.fit` | `trueno::simd::matmul` | 8x faster |
| `PCA.transform` | `trueno::simd::svd` | 5x faster |
| `KMeans.fit` | `trueno::simd::distance` | 10x faster |
| `RandomForest.predict` | `trueno::simd::tree_eval` | 15x faster |

**Estimator API** per Buitinck et al. [31]:
- `fit(X, y)` - Train on data (SIMD-accelerated)
- `predict(X)` - Generate predictions (SIMD-accelerated)
- `transform(X)` - Feature transformation (SIMD-accelerated)
- `fit_predict(X, y)` - Combined fit and predict

**GPU Training** (opt-in):
```ruchy
# Enable GPU training for large datasets
let model = RandomForest::new()
    .backend(Backend::CUDA)  # Uses trueno::cuda::*
    .n_estimators(1000)
model.fit(&X_train, &y_train)  # Parallel tree training on GPU
```

### 5.5 Pillar 5: Visualization (trueno-viz) — GPU-ACCELERATED RENDERING

**Purpose**: GPU/WASM-accelerated charts that render identically everywhere.

> "WebGPU is not optional—it is the default rendering backend."

```ruchy
# Create interactive visualization (renders via WebGPU)
let chart = Plot::new()
    .data(&df)
    .x(col("date"))
    .y(col("revenue"))
    .color(col("region"))
    .mark_line()
    .title("Revenue Over Time")

chart.show()  # WebGPU in browser, Metal/Vulkan on desktop
```

**Rendering Backends**:

| Platform | Backend | Performance |
|----------|---------|-------------|
| Browser | WebGPU | 60fps @ 1M points |
| macOS | Metal | 60fps @ 10M points |
| Linux/Windows | Vulkan | 60fps @ 10M points |
| Fallback | Canvas 2D | 30fps @ 100K points |

**Rendering Pipeline** per Satyanarayan et al.'s Vega-Lite [32]:
- Declarative grammar of graphics
- GPU compute shaders for binning/aggregation
- Instanced rendering for scatter plots
- 60fps interactive pan/zoom/brush

### 5.6 Pillar 6: Interaction (presentar) — WASM-NATIVE UI

**Purpose**: WASM-first widgets for interactive notebooks and dashboards.

> "The entire UI runs in WASM—no JavaScript framework required."

```ruchy
# Build interactive dashboard (compiles to WASM)
let app = Column::new([
    Text::new("Sales Dashboard"),
    Input::new("filter").placeholder("Search..."),
    chart,  # Embed GPU-accelerated visualization
    Button::new("Export").on_click(|_| export_pdf())
])

app.serve(8080)  # Launches WASM app, zero JS
```

**WASM Execution Model**:

| Component | Technology | Size |
|-----------|------------|------|
| UI Runtime | Rust → WASM | ~500KB |
| Event Loop | `wasm-bindgen` | Zero JS |
| Rendering | WebGPU/Canvas | GPU-accelerated |
| State | Rust ownership | No GC |

**Widget System** per Flutter's composition model [33]:
- Declarative widget tree (compiles to WASM)
- Reactive state management (Rust ownership, no GC)
- Accessibility built-in (WCAG 2.1 AA)
- Hot reload in development
- **No React/Vue/Angular required**

---

