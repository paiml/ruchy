# Sub-spec: Oracle — System Architecture

**Parent:** [dynamic-mlops-training-ruchy-oracle-spec.md](../dynamic-mlops-training-ruchy-oracle-spec.md) Section 2

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

