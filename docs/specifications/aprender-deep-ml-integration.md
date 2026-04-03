# Sub-spec: Aprender Deep ML Integration for Ruchy

**Parent:** [trueno-aprender-stdlib-core-language-spec.md](trueno-aprender-stdlib-core-language-spec.md) (Pillar 4: Learning)

---

## 0. Prerequisites

- **Current**: aprender 0.26.3, **target**: 0.27.5 (NOT 0.27.6 -- 0.27.6 does not exist on crates.io)
- **aprender_bridge.rs** is 359 lines (not 80) -- includes re-exports, hand-rolled metrics, and Oracle wiring
- **Oracle module** (`src/oracle/`) has local implementations (e.g., `compute_r2`, `compute_mse`, `compute_mae`, `compute_accuracy`) that may duplicate aprender functionality -- consolidation needed
- **Module paths caveat**: Paths listed for `GradientBoosting`, `KNearestNeighbors`, `SupportVectorMachine`, `GaussianNaiveBayes`, `GaussianMixture`, `IsolationForest`, `SpectralClustering`, `AgglomerativeClustering`, `ARIMA`, and `TPEOptimizer` are based on source inspection -- verify against aprender 0.27.5 published API before implementation

## 1. Overview

### 1.1 Current State

The Ruchy-aprender integration consists of a 359-line bridge module (`src/stdlib/aprender_bridge.rs`) that exposes a subset of aprender types and four hand-rolled metric functions (`compute_r2`, `compute_mse`, `compute_mae`, `compute_accuracy`). The Oracle module (`src/oracle/`) uses `RandomForestClassifier` for error classification with drift detection via `aprender::online::drift`. Model persistence, quantization, and GGUF export are re-exported but not wired into the Ruchy language surface.

### 1.2 Target State

Full ML pipeline as a first-class language feature: users write `train`, `predict`, `score`, `save`, and `deploy` in Ruchy source code. The compiler transpiles these to aprender API calls with type-safe pipelines, and the CLI provides model lifecycle management via `ruchy apr` subcommands.

### 1.3 Aprender Baseline

| Metric | Value |
|--------|-------|
| Version | 0.27.5 |
| Tests | 12,974+ |
| Coverage | 96.35% |
| Provable contracts | 73 |
| Compute substrate | trueno (SIMD/GPU) |

### 1.4 Success Criteria

| Criterion | Threshold |
|-----------|-----------|
| Estimators exposed to Ruchy | 20+ (currently ~12 re-exports) |
| Oracle accuracy (holdout) | >=95% |
| Model round-trip fidelity | bit-exact for f32, MSE <0.01 for Q8_0 |
| CLI subcommands | 6 (`run`, `serve`, `quantize`, `inspect`, `bench`, `eval`) |
| Language-level syntax coverage | train/predict/score/save/load |

---

## 2. Estimator Integration

All estimators below MUST be exposed to Ruchy via the aprender bridge with scikit-learn-compatible `fit`/`predict`/`transform` API.

### 2.1 Supervised Learning

| Estimator | aprender Module | Task | Status |
|-----------|----------------|------|--------|
| `LinearRegression` | `aprender::prelude` | Regression | Re-exported |
| `LogisticRegression` | `aprender::prelude` | Classification | Re-exported |
| `DecisionTreeClassifier` | `aprender::prelude` | Classification | Re-exported |
| `DecisionTreeRegressor` | `aprender::prelude` | Regression | Re-exported |
| `RandomForestClassifier` | `aprender::tree` | Classification | Used by Oracle |
| `RandomForestRegressor` | `aprender::prelude` | Regression | Re-exported |
| `GradientBoosting` | `aprender::ensemble` | Classification/Regression | Not exposed |
| `kNN` | `aprender::neighbors` | Classification/Regression | Not exposed |
| `SVM` | `aprender::svm` | Classification/Regression | Not exposed |
| `NaiveBayes` | `aprender::naive_bayes` | Classification | Not exposed |
| `ElasticNet` | `aprender::prelude` | Regression | Re-exported |
| `Lasso` | `aprender::prelude` | Regression | Re-exported |
| `Ridge` | `aprender::prelude` | Regression | Re-exported |

### 2.2 Unsupervised Learning

| Estimator | aprender Module | Task | Status |
|-----------|----------------|------|--------|
| `KMeans` | `aprender::prelude` | Clustering | Re-exported |
| `DBSCAN` | `aprender::prelude` | Clustering | Re-exported |
| `GaussianMixture` | `aprender::mixture` | Clustering/Density | Not exposed |
| `IsolationForest` | `aprender::ensemble` | Anomaly Detection | Not exposed |
| `PCA` | `aprender::preprocessing` | Dimensionality Reduction | Re-exported |
| `SpectralClustering` | `aprender::cluster` | Clustering | Not exposed |
| `Agglomerative` | `aprender::cluster` | Hierarchical Clustering | Not exposed |

### 2.3 Time Series and AutoML

| Estimator | aprender Module | Task | Status |
|-----------|----------------|------|--------|
| `ARIMA` | `aprender::timeseries` | Forecasting | Not exposed |
| `TPEOptimizer` | `aprender::automl` | Hyperparameter Tuning | Not exposed |

### 2.4 Bridge Implementation

New re-exports in `aprender_bridge.rs` (module paths based on source inspection -- verify against aprender 0.27.5 published API):

```rust
// Supervised - new additions
pub use aprender::ensemble::GradientBoosting;
pub use aprender::neighbors::KNearestNeighbors;
pub use aprender::svm::SupportVectorMachine;
pub use aprender::naive_bayes::GaussianNaiveBayes;

// Unsupervised - new additions
pub use aprender::mixture::GaussianMixture;
pub use aprender::ensemble::IsolationForest;
pub use aprender::cluster::{SpectralClustering, AgglomerativeClustering};

// Time Series + AutoML
pub use aprender::timeseries::ARIMA;
pub use aprender::automl::TPEOptimizer;
```

---

## 3. Oracle Lifecycle (Deep Integration)

The Oracle (`src/oracle/`) currently uses `RandomForestClassifier` with bootstrap training. This section specifies the full lifecycle with aprender's online learning infrastructure.

### 3.1 OnlineLearner Trait

The Oracle classifier MUST implement `aprender::online::OnlineLearner`:

```rust
impl OnlineLearner for OracleClassifier {
    fn partial_fit(&mut self, x: &Matrix, y: &[usize]) -> Result<()>;
    fn predict_online(&self, x: &Matrix) -> Vec<usize>;
    fn reset(&mut self);
}
```

This enables micro-batch updates after each transpilation cycle without full retraining.

### 3.2 Drift Detection

| Algorithm | aprender Type | Use Case | Default |
|-----------|--------------|----------|---------|
| ADWIN | `aprender::online::drift::Adwin` | Concept drift in error distribution | Yes |
| DDM | `aprender::online::drift::Ddm` | Sudden distribution shifts | No |
| Page-Hinkley | `aprender::online::drift::PageHinkley` | Gradual drift detection | No |

Drift detection triggers retraining when the error distribution shifts beyond the configured threshold. ADWIN is the default because it adapts window size automatically.

### 3.3 Corpus Management

The Oracle corpus uses `aprender::online::corpus::CorpusBuffer`:

- **Capacity**: 50,000 labeled samples (configurable via `OracleConfig`)
- **Deduplication**: Feature-hash deduplication prevents duplicate training samples
- **Stratified sampling**: Maintains class balance across all 8 error categories
- **Four sources**: Bootstrap, transpilation feedback, user corrections, synthetic augmentation

### 3.4 Curriculum Learning

| Strategy | aprender Type | Description |
|----------|--------------|-------------|
| Linear | `aprender::online::curriculum::LinearCurriculum` | Easy-to-hard ordering by confidence |
| Self-Paced | `aprender::online::curriculum::SelfPacedCurriculum` | Model selects next training batch |

Curriculum learning orders training samples from high-confidence (easy) to low-confidence (hard), improving convergence speed by 2-3x on the Oracle's error classification task.

### 3.5 Knowledge Distillation

Large RandomForest (500 trees) distills to a smaller model (50 trees) for fast inference:

```rust
use aprender::online::distillation::KnowledgeDistiller;

let teacher = RandomForestClassifier::new(500);
let student = RandomForestClassifier::new(50);
let distiller = KnowledgeDistiller::new(teacher, student);
distiller.distill(&corpus)?;
```

Target: student achieves >=95% of teacher accuracy with 10x faster inference.

### 3.6 Active Learning

Uncertainty sampling selects the most informative samples for human labeling:

```rust
use aprender::online::active::UncertaintySampler;

let sampler = UncertaintySampler::new(&model);
let candidates = sampler.select(unlabeled_pool, budget: 100);
```

This reduces the labeling burden by focusing on samples where the model is least confident.

---

## 4. Model Persistence (APR Format)

### 4.1 Native APR Format

The `.apr` format is aprender's native model serialization, built on SafeTensors with added provenance metadata:

| Feature | Implementation | Status |
|---------|---------------|--------|
| SafeTensors core | `aprender::serialization::SafeTensorsMetadata` | Re-exported |
| Ed25519 signing | `aprender::format::save_signed` / `load_verified` | Re-exported |
| AES-256-GCM encryption | `aprender::format::encrypt` | Feature-gated |
| Version metadata | Embedded in SafeTensors header | Available |

### 4.2 Format Interoperability

| Format | Direction | aprender API | Use Case |
|--------|-----------|-------------|----------|
| SafeTensors | Import/Export | `aprender::serialization` | HuggingFace ecosystem |
| GGUF | Export | `aprender::format::gguf::export_tensors_to_gguf` | llama.cpp / Ollama serving |
| ONNX | Export | `aprender::format::onnx` (planned) | Cross-framework inference |

### 4.3 Quantization

GGUF-compatible quantization is already re-exported in the bridge:

| Type | Bits | Size Reduction | Accuracy Loss | aprender API |
|------|------|----------------|---------------|-------------|
| Q8_0 | 8 | 4x | <0.1% | `quantize(&w, &s, QuantType::Q8_0)` |
| Q4_0 | 4 | 8x | <1.0% | `quantize(&w, &s, QuantType::Q4_0)` |

Block size is 32 (GGUF standard). MSE thresholds enforced by property tests.

### 4.4 CLI Commands

```
ruchy model save <path.apr>        Save trained model to APR format
ruchy model load <path.apr>        Load model from APR format
ruchy model export <path> --format <safetensors|gguf>
                                   Export to external format
ruchy model import <path>          Import from SafeTensors
ruchy model inspect <path.apr>     Show model metadata, signature, quantization info
ruchy model verify <path.apr> --key <pubkey>
                                   Verify Ed25519 signature
```

---

## 5. Language-Level ML Syntax

### 5.1 Training Workflow

```ruchy
import ml

# Load data
let data = ml.load_csv("housing.csv")
let (x_train, x_test, y_train, y_test) = ml.train_test_split(data, test_size=0.2)

# Preprocessing pipeline
let scaler = ml.StandardScaler()
let x_train_scaled = scaler.fit_transform(x_train)
let x_test_scaled = scaler.transform(x_test)

# Train
let model = ml.LinearRegression()
model.fit(x_train_scaled, y_train)

# Predict and evaluate
let predictions = model.predict(x_test_scaled)
let score = ml.r_squared(y_test, predictions)
print(f"R² = {score}")
```

### 5.2 Classification Workflow

```ruchy
import ml

let model = ml.RandomForest(n_trees=100)
model.fit(x_train, y_train)

let accuracy = ml.accuracy(y_test, model.predict(x_test))
print(f"Accuracy: {accuracy}")

# Save with signing
model.save("classifier.apr", sign=true)
```

### 5.3 Unsupervised Workflow

```ruchy
import ml

let kmeans = ml.KMeans(k=3)
let labels = kmeans.fit_predict(data)

let pca = ml.PCA(n_components=2)
let reduced = pca.fit_transform(data)
```

### 5.4 Type-Safe Pipelines

The transpiler generates type-checked Rust code from Ruchy ML syntax. Each pipeline stage has a typed signature:

```
StandardScaler: Matrix -> Matrix
LinearRegression: (Matrix, Vector) -> Model
Model.predict: Matrix -> Vector
r_squared: (Vector, Vector) -> f64
```

Type mismatches (passing a `Vector` where `Matrix` is expected) are caught at transpile time, not runtime.

### 5.5 Transpilation Target

Ruchy ML syntax transpiles to direct aprender API calls. Example: `ml.LinearRegression()` becomes `aprender::prelude::LinearRegression::new()`, `model.fit(x, y)` becomes `model.fit(&x_matrix, &y_vector)?`, and `ml.r_squared(y, p)` becomes `aprender::prelude::r_squared(&y_vec, &p_vec)`.

---

## 6. APR CLI Integration

The `ruchy apr` subcommand provides model lifecycle management outside Ruchy source files.

### 6.1 Subcommands

| Command | Description | Example |
|---------|-------------|---------|
| `ruchy apr run <model.apr> --input <data.csv>` | Execute model inference | Batch prediction |
| `ruchy apr serve <model.apr> --port 8080` | REST API server for inference | Production deployment |
| `ruchy apr quantize <model.apr> --type q8_0` | Compress model weights | Edge deployment |
| `ruchy apr inspect <model.apr>` | Display model metadata | Debugging, auditing |
| `ruchy apr bench <model.apr> --samples 10000` | Throughput benchmarking | Performance validation |
| `ruchy apr eval <model.apr> --test <data.csv>` | Compute evaluation metrics | Model validation |

### 6.2 Serve Endpoint Specification

`ruchy apr serve` exposes three REST endpoints:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/predict` | POST | JSON body `{"features": [...]}`, returns `{"prediction": ..., "confidence": ...}` |
| `/health` | GET | Returns `{"status": "ok", "model": "...", "uptime_s": ...}` |
| `/metadata` | GET | Returns estimator type, feature count, signing status |

### 6.3 Bench Output

`ruchy apr bench` reports: model name, quantization type, throughput (predictions/sec), latency percentiles (p50, p99, max), and resident memory.

---

## 7. Advanced Features

### 7.1 Transfer Learning

Pre-train on the Rust compiler error corpus (rustc diagnostics), then fine-tune on Ruchy-specific patterns:

```rust
use aprender::transfer::TransferLearner;

let pretrained = RandomForestClassifier::load("rust_errors.apr")?;
let learner = TransferLearner::new(pretrained);
learner.fine_tune(&ruchy_corpus, freeze_layers: 0.5)?;
```

This bootstraps the Oracle with knowledge from 100K+ Rust error patterns, reducing Ruchy-specific training data requirements from 12,000 to ~2,000 samples.

### 7.2 Graph Algorithms

Exposed for call graph analysis and code structure reasoning:

| Algorithm | aprender Module | Use Case in Ruchy |
|-----------|----------------|-------------------|
| PageRank | `aprender::graph::pagerank` | Function importance ranking (pmat) |
| Betweenness Centrality | `aprender::graph::centrality` | Bottleneck detection in call graphs |
| Community Detection | `aprender::graph::community` | Module clustering suggestions |

### 7.3 Bayesian Methods

Conjugate priors for uncertainty quantification in the Oracle:

| Prior | aprender Type | Application |
|-------|--------------|-------------|
| Beta-Binomial | `aprender::bayes::BetaBinomial` | Fix success probability estimation |
| Dirichlet-Multinomial | `aprender::bayes::DirichletMultinomial` | Error category distribution |

### 7.4 Calibration

Post-hoc calibration ensures Oracle confidence scores are well-calibrated:

| Method | aprender Type | Description |
|--------|--------------|-------------|
| Platt Scaling | `aprender::calibration::PlattScaling` | Logistic sigmoid fit on logits |
| Isotonic Regression | `aprender::calibration::IsotonicRegression` | Non-parametric monotone fit |
| Temperature Scaling | `aprender::calibration::TemperatureScaling` | Single parameter scaling |

Calibration target: ECE (Expected Calibration Error) < 0.05.

### 7.5 Speech (Feature-Gated)

Available only with `features = ["audio"]` in Cargo.toml:

| Component | aprender Module | Description |
|-----------|----------------|-------------|
| VAD | `aprender::audio::vad` | Voice Activity Detection |
| ASR | `aprender::audio::asr` | Automatic Speech Recognition |
| TTS | `aprender::audio::tts` | Text-to-Speech synthesis |

These are exposed for Ruchy applications that process audio, not for the compiler itself.

---

## 8. Cargo.toml Changes

### 8.1 Dependency Upgrade

```toml
# Before (current)
aprender = { version = "0.26", default-features = false, features = ["format-quantize", "format-signing"] }

# After
aprender = { version = "0.27.5", default-features = false, features = ["format-quantize", "format-signing"] }
```

### 8.2 Feature Gates

```toml
[features]
default = ["ml-core"]

# Core ML: estimators, metrics, preprocessing (always available)
ml-core = ["aprender/default"]

# Full ML: all estimators + online learning + persistence + calibration
ml-full = ["ml-core", "ml-persistence", "ml-online", "ml-advanced"]

# Model persistence: APR format, SafeTensors, GGUF export
ml-persistence = ["aprender/format-quantize", "aprender/format-signing", "aprender/format-encryption"]

# Online learning: drift detection, corpus, curriculum, distillation
ml-online = ["aprender/online"]

# Advanced: graph algorithms, Bayesian, calibration
ml-advanced = ["aprender/graph", "aprender/bayes", "aprender/calibration"]

# Audio: VAD, ASR, TTS (large dependency tree, opt-in only)
ml-audio = ["aprender/audio"]

# GPU acceleration via trueno
ml-gpu = ["aprender/gpu", "trueno/gpu"]
```

### 8.3 Dependency Rationale

| Feature | Size Impact | Why Gated |
|---------|------------|-----------|
| `format-quantize` | ~50 KB | Always needed for model compression |
| `format-signing` | ~200 KB (ed25519) | Always needed for model provenance |
| `format-encryption` | ~150 KB (aes-gcm) | Optional: not all models need encryption |
| `audio` | ~5 MB (whisper, etc.) | Optional: most Ruchy programs have no audio |
| `gpu` | ~20 MB (CUDA/Metal) | Optional: SIMD sufficient for most workloads |

---

## 9. Testing Requirements

### 9.1 Property Tests (Numerical Correctness)

All metric functions and estimator wrappers require `proptest` with `ProptestConfig::with_cases(10_000)`. Required invariants:

| Property | Assertion |
|----------|-----------|
| MSE non-negative | `compute_mse(y_true, y_pred) >= 0.0` for all inputs |
| MAE non-negative | `compute_mae(y_true, y_pred) >= 0.0` for all inputs |
| R2 upper-bounded | `compute_r2(y_true, y_pred) <= 1.0` for all inputs |
| Accuracy bounded | `0.0 <= compute_accuracy(y_true, y_pred) <= 1.0` |
| RMSE = sqrt(MSE) | `compute_rmse(y, p) == compute_mse(y, p).sqrt()` |

### 9.2 Oracle Regression Tests

Oracle accuracy MUST NOT decrease across releases:

| Test | Assertion |
|------|-----------|
| `test_oracle_accuracy_baseline` | accuracy >= 0.95 on holdout set |
| `test_oracle_drift_detection` | ADWIN triggers on synthetic drift |
| `test_oracle_online_update` | partial_fit improves on new patterns |
| `test_oracle_distillation_fidelity` | student >= 0.95 * teacher accuracy |

### 9.3 Model Persistence Round-Trip

- **f32 models**: bit-exact round-trip (save APR -> load APR -> predict produces identical output)
- **Q8_0 quantized**: MSE < 0.01 after round-trip
- **Q4_0 quantized**: MSE < 0.50 after round-trip
- **Edge cases**: all-zeros (MSE == 0.0), max-range values (MSE < 1.0)

### 9.4 Mutation Testing

Target: >=75% CAUGHT/MISSED ratio via `cargo mutants --file src/stdlib/aprender_bridge.rs --timeout 300`. All five metric functions MUST have mutations caught.

### 9.5 Integration Tests

| Test | Pipeline |
|------|----------|
| `test_ruchy_ml_train_predict` | Ruchy source -> transpile -> compile -> train -> predict |
| `test_ruchy_ml_save_load` | Train -> save APR -> load APR -> predict (same results) |
| `test_ruchy_ml_pipeline` | Scaler -> Estimator -> Metrics (type-safe chain) |
| `test_ruchy_apr_cli` | `ruchy apr inspect` / `ruchy apr bench` on test model |
