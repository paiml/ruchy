# Dynamic MLOps Training for Ruchy Oracle

**Specification Version**: 1.1.0
**Status**: REVISED - Incorporating Team Review
**Author**: Claude Code
**Date**: 2025-12-08
**Revised**: 2025-12-08
**Ticket**: ORACLE-001
**Review**: Team feedback incorporated (Cold Start, GNN Complexity, Feedback Latency)

---

## Executive Summary

This specification defines a **self-improving MLOps system** for the Ruchy Oracle that continuously learns from transpilation outcomes. The system grows more intelligent over time through automated data collection, drift detection, curriculum learning, and knowledge distillation - adapting the battle-tested techniques from depyler's production Oracle.

**Core Principle**: The Oracle should never be "done" - it is a living system that improves with every transpilation cycle.

---

## 1. Introduction

### 1.1 Problem Statement

The current Ruchy Oracle uses bootstrap training with 30 hardcoded samples (`src/oracle/classifier.rs:266-346`). This approach has fundamental limitations:

1. **Static Knowledge**: Model cannot learn from new error patterns
2. **No Feedback Loop**: Transpilation outcomes are discarded
3. **Single Training Event**: No continuous improvement mechanism
4. **Missing Production Data**: Real-world errors not incorporated

### 1.2 Solution Overview

Implement a **Six-Strategy Acceleration Pipeline** that:

1. **Collects** error patterns from every transpilation
2. **Detects** model drift and triggers retraining
3. **Applies** curriculum learning (easy → hard)
4. **Distills** knowledge from high-confidence predictions
5. **Embeds** errors using GNN for structural similarity
6. **Monitors** via Hansei (反省) reflection analysis

### 1.3 Success Criteria

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| Single-shot fix rate | 40% | 80% | Production transpilations |
| Error classification accuracy | 70% | 95% | Holdout test set |
| Model staleness | N/A | <7 days | Drift detection |
| Training data size | 30 samples | 12,000+ | Corpus size |

---

## 2. Architecture

### 2.1 System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        RUCHY TRANSPILATION PIPELINE                      │
├─────────────────────────────────────────────────────────────────────────┤
│  Ruchy Source → Parser → AST → Transpiler → Rust Code → rustc          │
│                                                    │                     │
│                                                    ▼                     │
│                                           Compilation Result             │
│                                           (Success / Errors)             │
└───────────────────────────────────────────────────┬─────────────────────┘
                                                    │
                    ┌───────────────────────────────┼───────────────────────┐
                    │                               ▼                       │
                    │  ┌─────────────────────────────────────────────────┐ │
                    │  │            ORACLE CLASSIFICATION                 │ │
                    │  │  ┌─────────────────────────────────────────────┐│ │
                    │  │  │ Feature Extraction (73 features)            ││ │
                    │  │  │ • Error code ONE-HOT (40)                   ││ │
                    │  │  │ • Keyword detection (21)                    ││ │
                    │  │  │ • Handcrafted features (12)                 ││ │
                    │  │  └─────────────────────────────────────────────┘│ │
                    │  │                      ↓                          │ │
                    │  │  ┌─────────────────────────────────────────────┐│ │
                    │  │  │ RandomForest Classifier (100 trees)         ││ │
                    │  │  │ → Category + Confidence Score               ││ │
                    │  │  └─────────────────────────────────────────────┘│ │
                    │  └─────────────────────────────────────────────────┘ │
                    │                         │                             │
                    │                         ▼                             │
                    │  ┌─────────────────────────────────────────────────┐ │
                    │  │              CONTINUOUS LEARNING                 │ │
                    │  │                                                  │ │
                    │  │  ┌──────────┐  ┌──────────┐  ┌──────────────┐  │ │
                    │  │  │ Corpus   │  │ Drift    │  │ Curriculum   │  │ │
                    │  │  │ Collector│→ │ Detector │→ │ Scheduler    │  │ │
                    │  │  └──────────┘  └──────────┘  └──────────────┘  │ │
                    │  │        │              │              │          │ │
                    │  │        ▼              ▼              ▼          │ │
                    │  │  ┌──────────────────────────────────────────┐  │ │
                    │  │  │         RETRAINING PIPELINE              │  │ │
                    │  │  │  • Merge 4 data sources                  │  │ │
                    │  │  │  • Deduplicate by hash                   │  │ │
                    │  │  │  • Train RandomForest                    │  │ │
                    │  │  │  • Save to ruchy_oracle.apr              │  │ │
                    │  │  └──────────────────────────────────────────┘  │ │
                    │  └─────────────────────────────────────────────────┘ │
                    │                                                       │
                    │              DYNAMIC MLOPS TRAINING                   │
                    └───────────────────────────────────────────────────────┘
```

### 2.2 Data Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    FOUR-SOURCE TRAINING DATA PIPELINE                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Source 1: SYNTHETIC (12,000 samples)                                   │
│  ├── Type pair variations (30 combinations)                             │
│  ├── Borrow/lifetime patterns                                           │
│  ├── Trait bound scenarios                                              │
│  └── Seeds all 8 categories evenly                                      │
│                                                                          │
│  Source 2: RUCHY CORPUS (Hand-crafted from tickets)                     │
│  ├── Real failures with fixes from GitHub issues                        │
│  ├── Annotated with error categories                                    │
│  └── Priority: Quality > Quantity                                       │
│                                                                          │
│  Source 3: EXAMPLES CORPUS (from examples/*.ruchy)                      │
│  ├── Transpile all 40+ examples                                         │
│  ├── Collect any compilation errors                                     │
│  └── Associate with known fix patterns                                  │
│                                                                          │
│  Source 4: PRODUCTION CORPUS (Runtime collection)                       │
│  ├── Errors from actual user transpilations                             │
│  ├── Filtered by frequency/importance                                   │
│  └── GDPR-compliant (no PII)                                            │
│                                                                          │
├─────────────────────────────────────────────────────────────────────────┤
│                              MERGE PIPELINE                              │
│                                                                          │
│        Source 1 ─┐                                                       │
│        Source 2 ─┼─→ Deduplicate(hash) → Shuffle(seed=42) → Train       │
│        Source 3 ─┤                                                       │
│        Source 4 ─┘                                                       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Transfer Learning for Cold Start Mitigation

**REVIEW FEEDBACK**: Synthetic data lacks "long tail" distribution of real errors.

**Solution**: Pre-train on Rust error corpus before fine-tuning on Ruchy-specific data.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    TRANSFER LEARNING PIPELINE                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Stage 1: PRE-TRAINING (Rust Error Corpus)                              │
│  ├── Source: rustc error database (100K+ samples)                       │
│  ├── Coverage: All E0XXX error codes                                    │
│  ├── Model: RandomForest encoder learns Rust error semantics            │
│  └── Output: Pre-trained feature embeddings                             │
│                                                                          │
│  Stage 2: FINE-TUNING (Ruchy-Specific)                                  │
│  ├── Source: Ruchy synthetic + production corpus                        │
│  ├── Transfer: Reuse pre-trained encoder weights                        │
│  ├── Adaptation: Train classification head on Ruchy categories          │
│  └── Output: ruchy_oracle.apr with transferred knowledge                │
│                                                                          │
│  Benefits:                                                               │
│  ✓ Leverages vast Rust compiler error knowledge                         │
│  ✓ Better "long tail" coverage from real-world Rust errors              │
│  ✓ Faster convergence on Ruchy-specific patterns                        │
│  ✓ More robust to unseen error combinations                             │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**Implementation**:

```rust
pub struct TransferLearningConfig {
    /// Path to pre-trained Rust error model
    pub pretrained_model: PathBuf,

    /// Layers to freeze during fine-tuning
    pub frozen_layers: Vec<String>,

    /// Learning rate for fine-tuning (lower than pre-training)
    pub fine_tune_lr: f64,  // default: 0.001

    /// Whether to use feature extraction only (freeze all)
    pub feature_extraction_only: bool,
}

impl RuchyOracle {
    /// Load pre-trained model and fine-tune on Ruchy data
    pub fn from_pretrained(
        pretrained: &Path,
        ruchy_corpus: &Corpus,
        config: TransferLearningConfig,
    ) -> Result<Self, OracleError> {
        // 1. Load pre-trained encoder
        let encoder = load_rust_error_encoder(pretrained)?;

        // 2. Freeze specified layers
        let frozen_encoder = freeze_layers(encoder, &config.frozen_layers);

        // 3. Fine-tune on Ruchy corpus
        let oracle = Self::fine_tune(frozen_encoder, ruchy_corpus, config.fine_tune_lr)?;

        Ok(oracle)
    }
}
```

### 2.4 Online Learning with Micro-Batching

**REVIEW FEEDBACK**: Weekly retraining too slow for fast-moving language.

**Solution**: Hot-fix model layer with immediate high-confidence corrections.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    ONLINE LEARNING ARCHITECTURE                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     MAIN MODEL (Weekly)                          │   │
│  │  • Full RandomForest (100 trees)                                 │   │
│  │  • Trained on complete corpus                                    │   │
│  │  • Updated every Friday                                          │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              ↑                                          │
│                              │ merge                                    │
│                              │                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                   HOT-FIX LAYER (Real-time)                      │   │
│  │  • Lightweight model (10 trees)                                  │   │
│  │  • High-confidence corrections only (≥0.95)                      │   │
│  │  • Updated on every successful fix                               │   │
│  │  • Overrides main model when applicable                          │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              ↑                                          │
│                              │ immediate                                │
│                              │                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                   MICRO-BATCH BUFFER                             │   │
│  │  • Collects corrections in real-time                             │   │
│  │  • Triggers hot-fix update every 10 samples                      │   │
│  │  • Validates before promotion                                    │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**Implementation**:

```rust
pub struct OnlineLearningConfig {
    /// Micro-batch size before hot-fix update
    pub micro_batch_size: usize,  // default: 10

    /// Confidence threshold for hot-fix promotion
    pub hotfix_confidence: f64,  // default: 0.95

    /// Maximum hot-fix model size (samples)
    pub max_hotfix_samples: usize,  // default: 500

    /// Merge hot-fix to main on weekly retrain
    pub merge_on_retrain: bool,  // default: true
}

pub struct HotFixLayer {
    /// Lightweight correction model
    model: RandomForestClassifier,

    /// Samples in current hot-fix
    samples: Vec<Sample>,

    /// Override rules (error_hash → category)
    overrides: HashMap<u64, ErrorCategory>,
}

impl RuchyOracle {
    /// Classify with hot-fix layer override
    pub fn classify_with_hotfix(&self, error: &CompilationError) -> Classification {
        let hash = error.content_hash();

        // Check hot-fix overrides first
        if let Some(category) = self.hotfix.overrides.get(&hash) {
            return Classification::new(*category, 1.0);
        }

        // Try hot-fix model
        if let Some(prediction) = self.hotfix.predict(error) {
            if prediction.confidence >= self.config.hotfix_confidence {
                return prediction;
            }
        }

        // Fall back to main model
        self.classify(error)
    }

    /// Record successful fix for online learning
    pub fn record_fix(&mut self, error: &CompilationError, category: ErrorCategory) {
        self.hotfix.add_sample(error, category);

        if self.hotfix.samples.len() >= self.config.micro_batch_size {
            self.hotfix.retrain();
        }
    }
}
```

**Latency Comparison**:

| Update Type | Latency | Frequency | Coverage |
|-------------|---------|-----------|----------|
| Hot-fix override | <1ms | Immediate | Exact matches |
| Micro-batch | ~100ms | Every 10 samples | Similar patterns |
| Weekly retrain | ~60s | Fridays | Full corpus |

---

## 3. Six-Strategy Acceleration Pipeline

### 3.1 Strategy 1: TARANTULA Fault Localization

**Purpose**: Identify suspicious transpiler decisions using Spectrum-Based Fault Localization (SBFL).

**Algorithm**: Tarantula suspiciousness formula [1]:

```
suspiciousness(s) = (failed(s) / total_failed) /
                    ((failed(s) / total_failed) + (passed(s) / total_passed))
```

**Implementation**:

```rust
/// SBFL formulas for fault localization
pub enum SbflFormula {
    Tarantula,  // Jones & Harrold (2005)
    Ochiai,     // Ochiai (1957) - geometric mean
    Jaccard,    // Jaccard similarity coefficient
    WongII,     // Wong et al. (2007)
    DStar,      // Wong et al. (2014) - D* with star=2
}

/// Decision types to track in transpiler
pub enum DecisionType {
    TypeInference,
    BorrowInsertion,
    LifetimeAnnotation,
    TraitBoundResolution,
    MethodResolution,
    // ... 15 decision types total
}
```

**Output**: Suspiciousness scores per transpiler decision, enabling targeted debugging.

### 3.2 Strategy 2: Error Pattern Library (CITL)

**Purpose**: Store and retrieve fix patterns using Continuous Incremental Training from Labels [2].

**Pattern Schema**:

```rust
pub struct FixPattern {
    /// Error code (e.g., "E0308")
    pub error_code: Option<String>,

    /// Regex pattern for error message
    pub message_pattern: Regex,

    /// Code transformation (before → after)
    pub fix_diff: String,

    /// Success rate tracking
    pub applications: u32,
    pub successes: u32,

    /// Confidence score
    pub confidence: f64,
}
```

**Lifecycle**:
1. **Discovery**: New pattern from successful fix
2. **Validation**: Track success rate over 10+ applications
3. **Promotion**: High-confidence patterns → hardcoded rules
4. **Retirement**: Low-confidence patterns removed

### 3.3 Strategy 3: Curriculum Learning

**Purpose**: Train on progressively harder examples [3].

**Difficulty Levels**:

| Level | Difficulty | Examples |
|-------|------------|----------|
| Easy | 0.25 | Single type mismatch, missing semicolon |
| Medium | 0.50 | Borrow checker single violation |
| Hard | 0.75 | Multiple lifetime annotations |
| Expert | 1.00 | Complex trait bounds, generic constraints |

**Algorithm**:

```rust
pub struct CurriculumScheduler {
    /// Current difficulty level
    current_level: DifficultyLevel,

    /// Accuracy threshold to advance
    advance_threshold: f64,  // default: 0.85

    /// Samples per level before evaluation
    samples_per_level: usize,  // default: 100
}

impl CurriculumScheduler {
    /// Get next training batch sorted by difficulty
    pub fn next_batch(&mut self, corpus: &Corpus) -> Vec<Sample> {
        corpus.samples
            .iter()
            .filter(|s| s.difficulty <= self.current_level.score())
            .take(self.samples_per_level)
            .cloned()
            .collect()
    }

    /// Advance to next level if threshold met
    pub fn maybe_advance(&mut self, accuracy: f64) {
        if accuracy >= self.advance_threshold {
            self.current_level = self.current_level.next();
        }
    }
}
```

### 3.4 Strategy 4: Knowledge Distillation

**Purpose**: Transfer knowledge from high-confidence predictions to expand training data [4].

**Temperature-Scaled Soft Targets**:

```
soft_target(i) = exp(z_i / T) / Σ_j exp(z_j / T)
```

Where `T=3.0` (temperature) smooths the probability distribution.

**Distillation Pipeline**:

```rust
pub struct KnowledgeDistiller {
    /// Temperature for soft targets
    temperature: f64,  // default: 3.0

    /// Confidence threshold for distillation
    confidence_threshold: f64,  // default: 0.95
}

impl KnowledgeDistiller {
    /// Generate soft labels from teacher model
    pub fn distill(&self, teacher: &RuchyOracle, samples: &[Sample]) -> Vec<SoftLabel> {
        samples.iter()
            .filter_map(|s| {
                let prediction = teacher.classify(&s.error);
                if prediction.confidence >= self.confidence_threshold {
                    Some(SoftLabel {
                        sample: s.clone(),
                        soft_targets: self.temperature_scale(&prediction.probabilities),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}
```

### 3.5 Strategy 5: Code2Vec AST Embeddings (BASELINE)

**REVIEW FEEDBACK**: Start with simpler AST-path embeddings before GNN.

**Purpose**: Generate embeddings from AST path contexts [6]. **This is the recommended baseline.**

**Path Context**: `(start_terminal, path, end_terminal)`

```
Example: fn add(x: i32, y: i32) -> i32 { x + y }

Path contexts:
  (x, Param↑FnDecl↓ReturnType, i32)
  (x, Param↑FnDecl↓Body↓BinaryOp, y)
  (i32, ReturnType↑FnDecl↓Body↓BinaryOp↓Operand, x)
```

**Configuration**:

```rust
pub struct Code2VecConfig {
    /// Maximum path length (AST levels)
    max_path_length: usize,  // default: 8

    /// Maximum paths per sample
    max_paths: usize,  // default: 200

    /// Embedding dimension
    embedding_dim: usize,  // default: 128
}
```

**Why Code2Vec First**:
- **Latency**: <5ms inference (vs. ~50ms for GNN)
- **Simplicity**: No graph construction overhead
- **Proven**: 82% accuracy on similar tasks [6]
- **CLI-friendly**: Fast feedback loop preserved

### 3.6 Strategy 6: GNN Error Encoder (OPTIONAL - Phase 3+)

**REVIEW FEEDBACK**: GNNs add significant latency. Only upgrade if >5% accuracy gain.

**Purpose**: Embed errors with structural context using Graph Neural Networks [5].

**Status**: OPTIONAL - Implement only if Code2Vec accuracy plateaus below target.

**Upgrade Criteria**:
- Code2Vec accuracy < 90% after 30 days
- Accuracy gain from GNN > 5% on validation set
- Inference latency acceptable (<50ms p99)

**Program-Feedback Graph**:

```
┌─────────────────────────────────────────────────────────────────┐
│                    PROGRAM-FEEDBACK GRAPH                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   AST Nodes                    Diagnostic Nodes                  │
│   ──────────                   ────────────────                  │
│   ┌─────────┐                  ┌─────────────┐                  │
│   │ FnDecl  │◄────────────────►│ E0308 Error │                  │
│   │ "foo"   │   references     │ line 42     │                  │
│   └────┬────┘                  └──────┬──────┘                  │
│        │                              │                          │
│        │ contains                     │ caused_by                │
│        ▼                              ▼                          │
│   ┌─────────┐                  ┌─────────────┐                  │
│   │ LetExpr │◄────────────────►│ Type hint   │                  │
│   │ x: i32  │   type_of        │ "expected   │                  │
│   └─────────┘                  │  String"    │                  │
│                                └─────────────┘                  │
│                                                                  │
│   Edge Types: references, contains, type_of, caused_by,         │
│               suggests, located_at                               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**GNN Architecture** (if needed):

```rust
pub struct GnnErrorEncoder {
    /// Message passing layers
    num_layers: usize,  // default: 3

    /// Hidden dimension
    hidden_dim: usize,  // default: 128

    /// Aggregation function
    aggregation: Aggregation,  // Mean, Max, or Sum
}
```

**Decision Matrix**:

| Metric | Code2Vec (Baseline) | GNN (Optional) | Decision |
|--------|---------------------|----------------|----------|
| Latency (p99) | <5ms | ~50ms | Code2Vec wins |
| Accuracy | ~82% | ~87% | GNN +5% |
| Complexity | Low | High | Code2Vec wins |
| **Recommendation** | **START HERE** | Only if needed | |

---

## 4. Drift Detection and Retraining

### 4.1 ADWIN Algorithm

The system uses Adaptive Windowing (ADWIN) for concept drift detection [7]:

```rust
pub struct DriftDetector {
    /// Sliding window of predictions
    window: VecDeque<bool>,

    /// Window size
    window_size: usize,  // default: 100

    /// Drift threshold
    threshold: f64,  // default: 0.05
}

impl DriftDetector {
    pub fn check_drift(&self) -> DriftStatus {
        let current = self.current_accuracy();
        let historical = self.historical_accuracy();
        let deviation = (current - historical).abs();

        if deviation > self.threshold {
            DriftStatus::DriftDetected {
                historical,
                current,
                recommendation: "Retrain Oracle".into(),
            }
        } else if deviation > self.threshold / 2.0 {
            DriftStatus::Warning { accuracy: current, trend: "declining".into() }
        } else {
            DriftStatus::Stable { accuracy: current }
        }
    }
}
```

### 4.2 Retraining Triggers

| Trigger | Condition | Action |
|---------|-----------|--------|
| Performance Drop | accuracy < baseline - 0.05 | Schedule retraining |
| Data Drift | new patterns not in training | Force retraining |
| Scheduled | weekly (Fridays) | Batch retraining |
| Manual | API call | Immediate retraining |

### 4.3 Model Versioning

```
~/.ruchy/oracle/
├── ruchy_oracle.apr           # Current model
├── ruchy_oracle.apr.backup    # Previous version
├── oracle_params.json         # Hyperparameters
├── training_corpus.parquet    # Training data
└── drift_history.json         # Drift detection log
```

---

## 5. Hansei (反省) Reflection Analysis

### 5.1 Toyota Way Integration

**Hansei** (反省) is the Toyota Way principle of continuous self-reflection [8].

```rust
pub struct HanseiReport {
    /// Analysis period
    period: DateRange,

    /// Category-wise success rates
    category_rates: HashMap<ErrorCategory, f64>,

    /// Trend analysis
    trend: Trend,  // Improving, Degrading, Stable, Oscillating

    /// Issues identified
    issues: Vec<HanseiIssue>,

    /// Recommendations
    recommendations: Vec<String>,
}

pub enum HanseiSeverity {
    Info,      // Observation
    Warning,   // Attention needed
    Error,     // Action required
    Critical,  // Immediate action
}
```

### 5.2 Metrics Dashboard

```
┌─────────────────────────────────────────────────────────────────┐
│                    RUCHY ORACLE HANSEI REPORT                    │
│                    Period: 2025-12-01 to 2025-12-08             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Overall Accuracy: 87.3% (↑2.1% from last week)                 │
│  Single-Shot Fix Rate: 72.5% (target: 80%)                      │
│  Model Age: 3 days (threshold: 7 days)                          │
│                                                                  │
│  Category Breakdown:                                             │
│  ┌─────────────────┬──────────┬─────────┬────────────┐         │
│  │ Category        │ Accuracy │ Samples │ Trend      │         │
│  ├─────────────────┼──────────┼─────────┼────────────┤         │
│  │ TypeMismatch    │ 94.2%    │ 1,247   │ ↑ Improving│         │
│  │ BorrowChecker   │ 88.1%    │ 892     │ → Stable   │         │
│  │ LifetimeError   │ 76.3%    │ 234     │ ↓ Degrading│         │
│  │ TraitBound      │ 82.5%    │ 567     │ → Stable   │         │
│  │ MissingImport   │ 91.8%    │ 1,102   │ ↑ Improving│         │
│  │ MutabilityError │ 85.7%    │ 421     │ → Stable   │         │
│  │ SyntaxError     │ 89.4%    │ 678     │ ↑ Improving│         │
│  │ Other           │ 45.2%    │ 156     │ ↓ Degrading│         │
│  └─────────────────┴──────────┴─────────┴────────────┘         │
│                                                                  │
│  Issues:                                                         │
│  ⚠ WARNING: LifetimeError accuracy below threshold (80%)        │
│  ⚠ WARNING: "Other" category growing (156 → 234 samples)        │
│                                                                  │
│  Recommendations:                                                │
│  1. Add more lifetime error training samples                     │
│  2. Review "Other" samples for new category patterns            │
│  3. Consider GNN encoder for structural lifetime analysis       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 6. Implementation Plan

### 6.1 Phase 1: Foundation (Week 1-2)

| Task | Description | Complexity |
|------|-------------|------------|
| 6.1.1 | Corpus collector infrastructure | Medium |
| 6.1.2 | Four-source data merger | Medium |
| 6.1.3 | Deduplication by hash | Low |
| 6.1.4 | Model persistence (.apr format) | Medium |

### 6.2 Phase 2: Continuous Learning (Week 3-4)

| Task | Description | Complexity |
|------|-------------|------------|
| 6.2.1 | Drift detection integration | Medium |
| 6.2.2 | Automatic retraining pipeline | High |
| 6.2.3 | Curriculum learning scheduler | Medium |
| 6.2.4 | Knowledge distillation | High |

### 6.3 Phase 3: Advanced Strategies (Week 5-6)

| Task | Description | Complexity |
|------|-------------|------------|
| 6.3.1 | TARANTULA fault localization | High |
| 6.3.2 | Error pattern library (CITL) | Medium |
| 6.3.3 | GNN error encoder | Very High |
| 6.3.4 | Code2Vec embeddings | High |

### 6.4 Phase 4: Monitoring & Reflection (Week 7-8)

| Task | Description | Complexity |
|------|-------------|------------|
| 6.4.1 | Hansei report generation | Medium |
| 6.4.2 | Metrics dashboard | Medium |
| 6.4.3 | Alerting integration | Low |
| 6.4.4 | Documentation | Low |

---

## 7. Quality Gates

### 7.1 Testing Requirements

| Test Type | Coverage Target | Tool |
|-----------|-----------------|------|
| Unit Tests | 95% | cargo test |
| Property Tests | 10K+ cases | proptest |
| Mutation Tests | 80% killed | cargo-mutants |
| Integration Tests | All pipelines | custom harness |

### 7.2 Performance Requirements

| Metric | Requirement |
|--------|-------------|
| Classification latency | <10ms p99 |
| Training time (12K samples) | <60s |
| Model size (.apr) | <10MB |
| Memory usage | <100MB |

### 7.3 PMAT Quality Gates

```bash
# Pre-commit validation
pmat tdg . --min-grade A- --fail-on-violation

# Complexity limits
# - Cyclomatic complexity: ≤10
# - Cognitive complexity: ≤10
# - Function size: ≤30 lines
```

---

## 8. Toyota Way Review

### 8.1 Jidoka (自働化) - Autonomation

**Principle**: Build quality in, stop on defects.

| Aspect | Implementation |
|--------|----------------|
| Stop the line | Drift detection triggers immediate alert |
| Andon signal | Hansei report severity levels |
| Root cause | Five Whys analysis in issue tracking |
| Poka-yoke | Type-safe API prevents misuse |

### 8.2 Kaizen (改善) - Continuous Improvement

**Principle**: Small, incremental improvements daily.

| Aspect | Implementation |
|--------|----------------|
| Daily learning | Every transpilation adds to corpus |
| Weekly reflection | Hansei reports generated |
| Monthly review | Model performance analysis |
| Quarterly planning | Strategy evaluation |

### 8.3 Genchi Genbutsu (現地現物) - Go and See

**Principle**: Base decisions on firsthand observation.

| Aspect | Implementation |
|--------|----------------|
| Real data | Production corpus from actual usage |
| Examples corpus | Transpile examples/*.ruchy |
| No synthetic-only | Always include real errors |
| Observability | Full tracing and metrics |

### 8.4 Nemawashi (根回し) - Consensus Building

**Principle**: Build consensus before implementation.

| Aspect | Implementation |
|--------|----------------|
| Team review | This spec requires approval |
| Stakeholder input | Gather feedback before coding |
| Incremental rollout | Feature flags for new strategies |
| Rollback plan | Model versioning enables revert |

---

## 9. Google AI/ML Engineering Review

### 9.1 ML System Design Principles [9]

| Principle | Application |
|-----------|-------------|
| **Data quality > model complexity** | Four-source corpus with deduplication |
| **Simple baselines first** | Rule-based fallback before ML |
| **Monitor everything** | Drift detection, Hansei reports |
| **Automate cautiously** | Human approval for major retraining |

### 9.2 Technical Debt in ML Systems [10]

| Debt Type | Mitigation |
|-----------|------------|
| **Data dependencies** | Schema versioning, validation |
| **Feedback loops** | Separate training/serving data |
| **Entanglement** | Modular strategy architecture |
| **Pipeline jungles** | Unified four-source merger |
| **Dead features** | Feature importance tracking |
| **Glue code** | Type-safe Rust interfaces |

### 9.3 ML Test Score

Target: **Level 3** (Good ML practices)

| Category | Tests |
|----------|-------|
| Features | Feature coverage, schema tests |
| Model | Unit tests, integration tests |
| Training | Reproducibility, convergence |
| Serving | Latency, accuracy monitoring |
| Monitoring | Drift detection, alerting |

---

## 10. References

### Peer-Reviewed Citations

1. **Jones, J. A., & Harrold, M. J.** (2005). "Empirical Evaluation of the Tarantula Automatic Fault-Localization Technique." *Proceedings of the 20th IEEE/ACM International Conference on Automated Software Engineering (ASE)*, 273-282. DOI: 10.1145/1101908.1101949

2. **Amershi, S., Cakmak, M., Knox, W. B., & Kulesza, T.** (2014). "Power to the People: The Role of Humans in Interactive Machine Learning." *AI Magazine*, 35(4), 105-120. DOI: 10.1609/aimag.v35i4.2513

3. **Bengio, Y., Louradour, J., Collobert, R., & Weston, J.** (2009). "Curriculum Learning." *Proceedings of the 26th International Conference on Machine Learning (ICML)*, 41-48. DOI: 10.1145/1553374.1553380

4. **Hinton, G., Vinyals, O., & Dean, J.** (2015). "Distilling the Knowledge in a Neural Network." *NIPS Deep Learning Workshop*. arXiv:1503.02531

5. **Allamanis, M., Brockschmidt, M., & Khademi, M.** (2018). "Learning to Represent Programs with Graphs." *International Conference on Learning Representations (ICLR)*. arXiv:1711.00740

6. **Alon, U., Zilberstein, M., Levy, O., & Yahav, E.** (2019). "code2vec: Learning Distributed Representations of Code." *Proceedings of the ACM on Programming Languages (POPL)*, 3(POPL), Article 40. DOI: 10.1145/3290353

7. **Bifet, A., & Gavalda, R.** (2007). "Learning from Time-Changing Data with Adaptive Windowing." *Proceedings of the 2007 SIAM International Conference on Data Mining (SDM)*, 443-448. DOI: 10.1137/1.9781611972771.42

8. **Liker, J. K.** (2004). "The Toyota Way: 14 Management Principles from the World's Greatest Manufacturer." *McGraw-Hill Education*. ISBN: 978-0071392310

9. **Sculley, D., et al.** (2015). "Hidden Technical Debt in Machine Learning Systems." *Advances in Neural Information Processing Systems (NeurIPS)*, 28, 2503-2511.

10. **Breck, E., Cai, S., Nielsen, E., Salib, M., & Sculley, D.** (2017). "The ML Test Score: A Rubric for ML Production Readiness and Technical Debt Reduction." *IEEE International Conference on Big Data*, 1123-1132. DOI: 10.1109/BigData.2017.8258038

### Additional Citations (From Team Review)

11. **Vaswani, A., Shazeer, N., Parmar, N., et al.** (2017). "Attention Is All You Need." *Advances in Neural Information Processing Systems (NeurIPS)*, 30, 5998-6008. arXiv:1706.03762 *(Future consideration for Transformer-based error embeddings)*

12. **Chen, T., & Guestrin, C.** (2016). "XGBoost: A Scalable Tree Boosting System." *Proceedings of the 22nd ACM SIGKDD International Conference on Knowledge Discovery and Data Mining*, 785-794. DOI: 10.1145/2939672.2939785 *(Supports tree ensemble choice for RandomForest/Gradient Boosting)*

---

## 11. Appendices

### Appendix A: Error Category Mapping

```rust
pub enum ErrorCategory {
    TypeMismatch,      // E0308, E0271, E0606
    BorrowChecker,     // E0382, E0499, E0502, E0505
    LifetimeError,     // E0597, E0716, E0621, E0106
    TraitBound,        // E0277, E0599, E0609
    MissingImport,     // E0433, E0425, E0412, E0432
    MutabilityError,   // E0596, E0594
    SyntaxError,       // E0658, E0061, E0063
    Other,             // Uncategorized
}
```

### Appendix B: Feature Vector Schema

| Index Range | Feature Group | Count |
|-------------|---------------|-------|
| 0-39 | Error code ONE-HOT | 40 |
| 40-60 | Keyword detection | 21 |
| 61-72 | Handcrafted features | 12 |
| **Total** | | **73** |

### Appendix C: Model Persistence Format

```
ruchy_oracle.apr (APR format via aprender)
├── Header
│   ├── magic: "APRN"
│   ├── version: 1
│   ├── model_name: "ruchy-oracle"
│   └── created_at: timestamp
├── Metadata (JSON)
│   ├── training_samples: 12000
│   ├── accuracy: 0.873
│   └── feature_count: 73
├── Model Weights (SafeTensors)
│   ├── trees: 100
│   ├── max_depth: 10
│   └── weights: [compressed]
└── Checksum (SHA-256)
```

---

## 12. Approval

**Status**: REVISED - Team Review Incorporated

| Role | Name | Status | Date |
|------|------|--------|------|
| Author | Claude Code | Draft | 2025-12-08 |
| Team Review | Team | Feedback Provided | 2025-12-08 |
| Revision | Claude Code | Incorporated | 2025-12-08 |
| Technical Lead | | Pending Final Approval | |
| ML Engineer | | Pending Final Approval | |
| QA Lead | | Pending Final Approval | |

---

**Review Feedback Addressed**:

| Critique | Resolution | Section |
|----------|------------|---------|
| Cold Start Problem | Added Transfer Learning from Rust error corpus | §2.3 |
| GNN Complexity | Made GNN optional; Code2Vec as baseline | §3.5, §3.6 |
| Feedback Loop Latency | Added Online Learning with Hot-Fix layer | §2.4 |
| Additional Citations | Added Vaswani (2017), Chen (2016) | §10 |

---

## 13. Unified Training Loop UX

### 13.1 Design Philosophy

**Core Principle**: The Oracle should improve **by default** with every transpilation - no special commands required.

**Toyota Way Alignment**:
- **Jidoka**: Visual feedback acts as Andon board - shows system health at a glance
- **Kaizen**: Every iteration is a small improvement
- **Genchi Genbutsu**: Real metrics from actual usage, not estimates

### 13.2 Default-On Behavior

The training loop activates automatically during normal transpilation:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    UNIFIED TRAINING LOOP (Default-On)                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  User runs: ruchy transpile foo.ruchy                                   │
│                     │                                                    │
│                     ▼                                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ 1. TRANSPILE                                                     │   │
│  │    • Generate Rust code                                          │   │
│  │    • Compile with rustc                                          │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                     │                                                    │
│                     ▼                                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ 2. COLLECT (Automatic)                                           │   │
│  │    • Parse rustc errors → corpus samples                         │   │
│  │    • Deduplicate by feature hash                                 │   │
│  │    • Store in ~/.ruchy/oracle/corpus.parquet                     │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                     │                                                    │
│                     ▼                                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ 3. EVALUATE (Background)                                         │   │
│  │    • Check drift status (ADWIN)                                  │   │
│  │    • Update running accuracy                                     │   │
│  │    • Trigger retrain if threshold met                            │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  No "apex hunt" required - learning happens transparently               │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 13.3 Visual Feedback Format

#### 13.3.1 Andon TUI (Toyota Way Visual Management)

**Primary Display** - The Andon board provides at-a-glance system health:

```
╔══════════════════════════════════════════════════════════════════════╗
║  Iteration: [████████████░░░░░░░░] 12/20 (60%)                      ║
║  Estimated Convergence: 83.2% → Target: 80.0%  ✓ ON TRACK           ║
║  Last Trained:    2025-12-08 20:22:15 UTC (3 min ago)               ║
║  Model Size:      503 KB (zstd compressed)                          ║
║  Accuracy:        ▁▂▃▄▅▆▇█ 85.3% (+2.1%)                           ║
║  Drift Status:    ● STABLE                                          ║
╚══════════════════════════════════════════════════════════════════════╝
```

**Toyota Way Principles Applied**:

| Principle | Visual Element | Purpose |
|-----------|----------------|---------|
| **Jidoka** (自働化) | Drift Status indicator (●) | Stop-the-line signal when RED |
| **Kaizen** (改善) | Accuracy sparkline (▁▂▃▄▅▆▇█) | Visual trend of continuous improvement |
| **Genchi Genbutsu** (現地現物) | Real metrics, not estimates | "Go and see" actual performance |
| **Andon** (行灯) | Color-coded status board | Visual factory floor signaling |

**Andon Color States**:

```
● GREEN  (STABLE)   - System healthy, no action needed
● YELLOW (WARNING)  - Attention required, monitor closely
● RED    (DRIFT)    - Stop the line! Immediate retraining required
```

#### 13.3.2 Detailed View (Verbose Mode)

**Iteration Display** (shown during active training or verbose mode):

```
┌─────────────────────────────────────────────────────────────────────────┐
│ 🔄 ORACLE TRAINING                                                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  iteration[12/50] ████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░ 24%           │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Model Stats                                                      │   │
│  ├─────────────────────────────────────────────────────────────────┤   │
│  │ Last trained:    2025-12-08 15:42:31 (3 hours ago)              │   │
│  │ Model size:      847 KB (.apr)                                   │   │
│  │ Corpus size:     2,847 samples                                   │   │
│  │ Trees:           100                                             │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Current Evaluation                                               │   │
│  ├─────────────────────────────────────────────────────────────────┤   │
│  │ Accuracy:        87.3% (target: 80%)  ✓                         │   │
│  │ Convergence:     ~3 iterations to 90%                           │   │
│  │ Drift status:    STABLE ●                                        │   │
│  │ Fix rate:        72% single-shot                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  Category Breakdown:                                                     │
│  TypeMismatch    ████████████████████ 94.2%  ↑                          │
│  BorrowChecker   ████████████████░░░░ 88.1%  →                          │
│  LifetimeError   ███████████████░░░░░ 76.3%  ↓  ⚠                       │
│  TraitBound      ████████████████░░░░ 82.5%  →                          │
│  MissingImport   ████████████████████ 91.8%  ↑                          │
│  MutabilityError █████████████████░░░ 85.7%  →                          │
│  SyntaxError     ████████████████████ 89.4%  ↑                          │
│  Other           █████████░░░░░░░░░░░ 45.2%  ↓  ⚠                       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 13.3.3 Compact Mode (Default)

**Compact Mode** (default during transpilation):

```
🔄 Oracle: iteration[12/50] 87.3% acc | 847KB | trained 3h ago | STABLE
```

#### 13.3.4 Andon TUI Implementation

**Rust Implementation**:

```rust
use std::io::Write;

/// Andon status (Toyota Way visual signaling)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndonStatus {
    /// GREEN - System healthy
    Green,
    /// YELLOW - Attention needed
    Yellow,
    /// RED - Stop the line!
    Red,
}

impl AndonStatus {
    /// Convert from drift status
    pub fn from_drift(drift: &DriftStatus) -> Self {
        match drift {
            DriftStatus::Stable => AndonStatus::Green,
            DriftStatus::Warning => AndonStatus::Yellow,
            DriftStatus::Drift => AndonStatus::Red,
        }
    }

    /// Get display string with color
    pub fn display(&self) -> &'static str {
        match self {
            AndonStatus::Green => "● STABLE",
            AndonStatus::Yellow => "● WARNING",
            AndonStatus::Red => "● DRIFT",
        }
    }

    /// Get ANSI color code
    pub fn color_code(&self) -> &'static str {
        match self {
            AndonStatus::Green => "\x1b[32m",   // Green
            AndonStatus::Yellow => "\x1b[33m",  // Yellow
            AndonStatus::Red => "\x1b[31m",     // Red
        }
    }
}

/// Sparkline for accuracy trend visualization (Kaizen principle)
pub fn render_sparkline(history: &[f64], width: usize) -> String {
    const CHARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    if history.is_empty() {
        return "─".repeat(width);
    }

    let min = history.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = history.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(0.01);  // Avoid division by zero

    history.iter()
        .take(width)
        .map(|&v| {
            let normalized = ((v - min) / range * 7.0).round() as usize;
            CHARS[normalized.min(7)]
        })
        .collect()
}

/// Render the Andon TUI board
pub fn render_andon_tui(
    iteration: usize,
    max_iterations: usize,
    accuracy: f64,
    target: f64,
    accuracy_delta: f64,
    last_trained: &str,
    model_size_kb: usize,
    accuracy_history: &[f64],
    drift: &DriftStatus,
) -> String {
    let progress = (iteration as f64 / max_iterations as f64 * 20.0) as usize;
    let progress_bar = format!(
        "[{}{}]",
        "█".repeat(progress),
        "░".repeat(20 - progress)
    );

    let on_track = if accuracy >= target { "✓ ON TRACK" } else { "⚠ BELOW TARGET" };
    let delta_sign = if accuracy_delta >= 0.0 { "+" } else { "" };
    let sparkline = render_sparkline(accuracy_history, 8);
    let andon = AndonStatus::from_drift(drift);

    format!(
        r#"╔══════════════════════════════════════════════════════════════════════╗
║  Iteration: {} {}/{} ({:.0}%){}║
║  Estimated Convergence: {:.1}% → Target: {:.1}%  {}{}║
║  Last Trained:    {}{}║
║  Model Size:      {} KB (zstd compressed){}║
║  Accuracy:        {} {:.1}% ({}{:.1}%){}║
║  Drift Status:    {}{}{}{}║
╚══════════════════════════════════════════════════════════════════════╝"#,
        progress_bar, iteration, max_iterations,
        (iteration as f64 / max_iterations as f64 * 100.0),
        " ".repeat(22 - progress_bar.len()),
        accuracy * 100.0, target * 100.0, on_track,
        " ".repeat(11 - on_track.len()),
        last_trained, " ".repeat(30 - last_trained.len()),
        model_size_kb, " ".repeat(40 - format!("{}", model_size_kb).len()),
        sparkline, accuracy * 100.0, delta_sign, accuracy_delta * 100.0,
        " ".repeat(30 - sparkline.len()),
        andon.color_code(), andon.display(), "\x1b[0m",
        " ".repeat(50 - andon.display().len())
    )
}

/// Render compact one-line status
pub fn render_compact(
    iteration: usize,
    max_iterations: usize,
    accuracy: f64,
    model_size_kb: usize,
    last_trained_ago: &str,
    drift: &DriftStatus,
) -> String {
    let andon = AndonStatus::from_drift(drift);
    format!(
        "🔄 Oracle: iteration[{}/{}] {:.1}% acc | {}KB | {} | {}",
        iteration, max_iterations,
        accuracy * 100.0,
        model_size_kb,
        last_trained_ago,
        andon.display()
    )
}
```

**Usage Example**:

```rust
// Verbose mode (--oracle-verbose)
let tui = render_andon_tui(
    12, 20,                          // iteration 12 of 20
    0.853, 0.80,                     // 85.3% accuracy, 80% target
    0.021,                           // +2.1% improvement
    "2025-12-08 20:22:15 UTC (3 min ago)",
    503,                             // 503 KB model
    &[0.72, 0.75, 0.78, 0.81, 0.83, 0.85],  // accuracy history
    &DriftStatus::Stable,
);
println!("{}", tui);

// Compact mode (default)
let compact = render_compact(12, 20, 0.853, 503, "3 min ago", &DriftStatus::Stable);
println!("{}", compact);
```

**Convergence Estimation Algorithm** [13]:

```rust
/// Estimate iterations to target accuracy using exponential smoothing
pub fn estimate_convergence(
    current_accuracy: f64,
    target_accuracy: f64,
    accuracy_history: &[f64],
    smoothing_factor: f64,  // α = 0.3 recommended
) -> Option<usize> {
    if accuracy_history.len() < 3 {
        return None;  // Need history for estimation
    }

    // Calculate smoothed improvement rate
    let improvements: Vec<f64> = accuracy_history
        .windows(2)
        .map(|w| w[1] - w[0])
        .collect();

    let smoothed_rate = improvements.iter()
        .rev()
        .enumerate()
        .fold(0.0, |acc, (i, &delta)| {
            acc + delta * smoothing_factor.powi(i as i32)
        });

    if smoothed_rate <= 0.0 {
        return None;  // Not converging
    }

    let gap = target_accuracy - current_accuracy;
    Some((gap / smoothed_rate).ceil() as usize)
}
```

### 13.4 Customization for Other Environments

**Configuration File**: `~/.ruchy/oracle/config.toml`

```toml
[oracle]
# Enable/disable automatic learning (default: true)
auto_learn = true

# Retrain threshold (samples since last train)
retrain_threshold = 100

# Target accuracy for convergence estimation
target_accuracy = 0.80

# Model persistence path
model_path = "~/.ruchy/oracle/ruchy_oracle.apr"

# Corpus persistence path
corpus_path = "~/.ruchy/oracle/corpus.parquet"

[display]
# Visual feedback mode: "compact", "verbose", "silent"
feedback_mode = "compact"

# Show iteration progress
show_iterations = true

# Show category breakdown
show_categories = false

# Refresh rate for live updates (ms)
refresh_rate = 500

[drift]
# Drift detection algorithm: "adwin", "ddm", "page_hinkley"
algorithm = "adwin"

# Drift sensitivity (0.01 = sensitive, 0.1 = tolerant)
threshold = 0.05

# Window size for drift detection
window_size = 100

[curriculum]
# Enable curriculum learning
enabled = true

# Advance threshold (accuracy to next level)
advance_threshold = 0.85

# Samples per difficulty level
samples_per_level = 100

[transfer]
# Pre-trained model for cold start
pretrained_model = ""

# Freeze encoder layers during fine-tuning
freeze_encoder = false

[enterprise]
# Company-specific corpus source
custom_corpus_url = ""

# API key for corpus sync
api_key = ""

# Telemetry opt-in (anonymous usage stats)
telemetry = false

# Custom error categories (extend base 8)
custom_categories = []
```

**Programmatic Configuration** [14]:

```rust
/// Builder pattern for Oracle configuration
pub struct OracleConfigBuilder {
    config: OracleConfig,
}

impl OracleConfigBuilder {
    pub fn new() -> Self {
        Self { config: OracleConfig::default() }
    }

    /// Disable automatic learning (for CI/CD pipelines)
    pub fn no_auto_learn(mut self) -> Self {
        self.config.auto_learn = false;
        self
    }

    /// Custom corpus source (enterprise)
    pub fn with_corpus_url(mut self, url: &str) -> Self {
        self.config.enterprise.custom_corpus_url = url.to_string();
        self
    }

    /// Custom error categories
    pub fn with_categories(mut self, categories: Vec<String>) -> Self {
        self.config.enterprise.custom_categories = categories;
        self
    }

    /// Silent mode for scripting
    pub fn silent(mut self) -> Self {
        self.config.display.feedback_mode = FeedbackMode::Silent;
        self
    }

    pub fn build(self) -> OracleConfig {
        self.config
    }
}

// Usage:
let config = OracleConfigBuilder::new()
    .with_corpus_url("https://corp.example.com/ruchy-corpus")
    .with_categories(vec!["CustomError1".into(), "CustomError2".into()])
    .build();

let oracle = RuchyOracle::with_config(config);
```

### 13.5 Environment-Specific Presets

```rust
/// Pre-configured environments
pub enum OraclePreset {
    /// Default: auto-learn, compact display
    Default,

    /// CI/CD: silent, no auto-learn, fast
    CI,

    /// Development: verbose, all metrics
    Development,

    /// Enterprise: custom corpus, telemetry
    Enterprise { corpus_url: String, api_key: String },

    /// Offline: no network, local corpus only
    Offline,
}

impl From<OraclePreset> for OracleConfig {
    fn from(preset: OraclePreset) -> Self {
        match preset {
            OraclePreset::Default => OracleConfig::default(),
            OraclePreset::CI => OracleConfigBuilder::new()
                .no_auto_learn()
                .silent()
                .build(),
            OraclePreset::Development => OracleConfigBuilder::new()
                .verbose()
                .show_all_metrics()
                .build(),
            OraclePreset::Enterprise { corpus_url, api_key } => {
                OracleConfigBuilder::new()
                    .with_corpus_url(&corpus_url)
                    .with_api_key(&api_key)
                    .enable_telemetry()
                    .build()
            }
            OraclePreset::Offline => OracleConfigBuilder::new()
                .no_network()
                .local_corpus_only()
                .build(),
        }
    }
}
```

### 13.6 CLI Integration

**Commands**:

```bash
# Normal transpilation (Oracle learns automatically)
ruchy transpile foo.ruchy

# Verbose mode - show full training stats
ruchy transpile foo.ruchy --oracle-verbose

# Silent mode - no Oracle output
ruchy transpile foo.ruchy --oracle-silent

# Force retrain now
ruchy oracle train --force

# Show current Oracle status
ruchy oracle status

# Show detailed metrics (Hansei report)
ruchy oracle status --detailed

# Export training data
ruchy oracle export corpus.parquet

# Import external corpus
ruchy oracle import external_corpus.parquet

# Reset Oracle to defaults
ruchy oracle reset --confirm
```

**Environment Variables**:

```bash
# Override config file location
RUCHY_ORACLE_CONFIG=./custom-oracle.toml

# Force preset
RUCHY_ORACLE_PRESET=ci

# Disable Oracle entirely
RUCHY_ORACLE_DISABLED=1

# Custom model path
RUCHY_ORACLE_MODEL=/path/to/model.apr
```

### 13.7 Integration with Hunt Mode

When `apex hunt` is used, it leverages the unified loop with enhanced feedback:

```
┌─────────────────────────────────────────────────────────────────────────┐
│ 🎯 APEX HUNT MODE - PDCA Cycle                                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Cycle: 3/10 (max)                                                       │
│  Defects fixed: 7                                                        │
│  Defects remaining: ~3 (estimated)                                       │
│                                                                          │
│  Oracle Integration:                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ iteration[47/50] 91.2% acc | 1.2MB | trained 12m ago | STABLE   │   │
│  │                                                                  │   │
│  │ Hunt-specific metrics:                                           │   │
│  │ • Errors classified this cycle: 23                               │   │
│  │ • Auto-fixes applied: 18 (78%)                                   │   │
│  │ • New patterns learned: 5                                        │   │
│  │ • Corpus growth: +23 samples                                     │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  Current defect: DEFECT-031                                              │
│  Category: BorrowChecker (confidence: 0.94)                             │
│  Suggested fix: Add .clone() before move                                 │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 13.8 Metrics Persistence

**Metrics History** (`~/.ruchy/oracle/metrics_history.json`):

```json
{
  "version": "1.0.0",
  "entries": [
    {
      "timestamp": "2025-12-08T15:42:31Z",
      "iteration": 47,
      "accuracy": 0.912,
      "corpus_size": 2870,
      "model_size_bytes": 867234,
      "drift_status": "stable",
      "category_accuracies": {
        "TypeMismatch": 0.942,
        "BorrowChecker": 0.881,
        "LifetimeError": 0.763,
        "TraitBound": 0.825,
        "MissingImport": 0.918,
        "MutabilityError": 0.857,
        "SyntaxError": 0.894,
        "Other": 0.452
      },
      "convergence_estimate": 3
    }
  ]
}
```

**Visualization** (via `ruchy oracle history`):

```
Accuracy over time (last 7 days):
100% ┤
 95% ┤                                    ╭──────
 90% ┤                           ╭────────╯
 85% ┤               ╭───────────╯
 80% ┤ ─────────────╯
 75% ┤
 70% ┼────┬────┬────┬────┬────┬────┬────┬────
     Dec1 Dec2 Dec3 Dec4 Dec5 Dec6 Dec7 Dec8

Corpus growth:
3000 ┤                                    ████
2500 ┤                           █████████████
2000 ┤               ████████████████████████
1500 ┤ ███████████████████████████████████████
1000 ┼────┬────┬────┬────┬────┬────┬────┬────
```

---

## 14. Extended References

### Additional Peer-Reviewed Citations (13-25)

13. **Holt, C. C.** (2004). "Forecasting seasonals and trends by exponentially weighted moving averages." *International Journal of Forecasting*, 20(1), 5-10. DOI: 10.1016/j.ijforecast.2003.09.015 *(Convergence estimation via exponential smoothing)*

14. **Gamma, E., Helm, R., Johnson, R., & Vlissides, J.** (1994). "Design Patterns: Elements of Reusable Object-Oriented Software." *Addison-Wesley*. ISBN: 978-0201633610 *(Builder pattern for configuration)*

15. **Domingos, P., & Hulten, G.** (2000). "Mining High-Speed Data Streams." *Proceedings of the 6th ACM SIGKDD International Conference on Knowledge Discovery and Data Mining*, 71-80. DOI: 10.1145/347090.347107 *(Online learning fundamentals)*

16. **Gama, J., Medas, P., Castillo, G., & Rodrigues, P.** (2004). "Learning with Drift Detection." *Advances in Artificial Intelligence - SBIA 2004*, 286-295. DOI: 10.1007/978-3-540-28645-5_29 *(DDM algorithm for drift detection)*

17. **Page, E. S.** (1954). "Continuous Inspection Schemes." *Biometrika*, 41(1/2), 100-115. DOI: 10.2307/2333009 *(Page-Hinkley test for change detection)*

18. **Baena-García, M., del Campo-Ávila, J., Fidalgo, R., Bifet, A., Gavaldà, R., & Morales-Bueno, R.** (2006). "Early Drift Detection Method." *Fourth International Workshop on Knowledge Discovery from Data Streams*, 77-86. *(EDDM for early drift warning)*

19. **Sugiyama, M., Krauledat, M., & Müller, K. R.** (2007). "Covariate Shift Adaptation by Importance Weighted Cross Validation." *Journal of Machine Learning Research*, 8, 985-1005. *(Handling distribution shift)*

20. **Gama, J., Žliobaitė, I., Bifet, A., Pechenizkiy, M., & Bouchachia, A.** (2014). "A Survey on Concept Drift Adaptation." *ACM Computing Surveys*, 46(4), Article 44. DOI: 10.1145/2523813 *(Comprehensive drift detection survey)*

21. **Tsymbal, A.** (2004). "The Problem of Concept Drift: Definitions and Related Work." *Computer Science Department, Trinity College Dublin*, Technical Report TCD-CS-2004-15. *(Concept drift taxonomy)*

22. **Widmer, G., & Kubat, M.** (1996). "Learning in the Presence of Concept Drift and Hidden Contexts." *Machine Learning*, 23(1), 69-101. DOI: 10.1007/BF00116900 *(FLORA system for concept drift)*

23. **Klinkenberg, R., & Joachims, T.** (2000). "Detecting Concept Drift with Support Vector Machines." *Proceedings of the 17th International Conference on Machine Learning*, 487-494. *(SVM-based drift detection)*

24. **Krawczyk, B., Minku, L. L., Gama, J., Stefanowski, J., & Woźniak, M.** (2017). "Ensemble Learning for Data Stream Analysis: A Survey." *Information Fusion*, 37, 132-156. DOI: 10.1016/j.inffus.2017.02.004 *(Ensemble methods for streaming data)*

25. **Lu, J., Liu, A., Dong, F., Gu, F., Gama, J., & Zhang, G.** (2019). "Learning under Concept Drift: A Review." *IEEE Transactions on Knowledge and Data Engineering*, 31(12), 2346-2363. DOI: 10.1109/TKDE.2018.2876857 *(Modern concept drift review)*

26. **Le Goues, C., Nguyen, T. V., Forrest, S., & Weimer, W.** (2012). "GenProg: A Generic Method for Automatic Software Repair." *IEEE Transactions on Software Engineering*, 38(1), 54-72. DOI: 10.1109/TSE.2011.104 *(Foundational Automated Software Repair)*

27. **Monperrus, M.** (2018). "Automatic Software Repair: A Bibliography." *ACM Computing Surveys*, 51(1), Article 17. DOI: 10.1145/3105906 *(Contextualizing the error-fix pattern approach)*

28. **Kreuzberger, D., Kühl, N., & Hirschl, S.** (2023). "Machine Learning Operations (MLOps): Overview, Definition, and Architecture." *IEEE Access*, 11, 31866-31879. DOI: 10.1109/ACCESS.2023.3262138 *(Formalizing the Continuous Training pipeline)*

29. **Sambasivan, N., et al.** (2021). "Everyone wants to do the model work, not the data work": Data Cascades in High-Stakes AI. *Proceedings of the 2021 CHI Conference on Human Factors in Computing Systems*, Article 39. DOI: 10.1145/3411764.3445518 *(Supporting Data Quality > Model Complexity)*

30. **Satyanarayanan, M.** (2017). "The Emergence of Edge Computing." *Computer*, 50(1), 30-39. DOI: 10.1109/MC.2017.9 *(Justification for local-first, low-latency inference)*

31. **Warden, P., & Situnayake, D.** (2019). "TinyML: Machine Learning with TensorFlow Lite on Arduino and Ultra-Low-Power Microcontrollers." *O'Reilly Media*. ISBN: 978-1492052043 *(Constraints for <1MB model size)*

32. **Gunning, D., & Aha, D.** (2019). "DARPA's Explainable Artificial Intelligence (XAI) Program." *AI Magazine*, 40(2), 44-58. DOI: 10.1609/aimag.v40i2.2850 *(Theoretical basis for Hansei reflection)*

33. **ISO/IEC TR 24029-1:2021**. "Artificial Intelligence (AI) — Assessment of the robustness of neural networks — Part 1: Overview." *(Standardization compliance)*

34. **Paleyes, A., Urma, R. G., & Lawrence, N. D.** (2022). "Challenges in Deploying Machine Learning: A Survey of Case Studies." *ACM Computing Surveys*, 55(6), Article 114. DOI: 10.1145/3533378 *(Addressing real-world deployment drift)*

35. **IEEE 2830-2021**. "IEEE Standard for Technical Verification of Explainable Artificial Intelligence (XAI)." *(Standards for visual feedback transparency)*

---

**Next Steps**:
1. ~~Team review of this specification~~ ✓ Complete
2. ~~Gather feedback and address concerns~~ ✓ Complete
3. ~~Update spec based on review~~ ✓ Complete
4. ~~Add Unified Training Loop UX (§13)~~ ✓ Complete
5. Final approval from Technical Lead, ML Engineer, QA Lead
6. Create implementation tickets in roadmap.yaml
7. Begin Phase 1 implementation

---

*This document follows the Toyota Way principle of Nemawashi (根回し) - building consensus before implementation.*