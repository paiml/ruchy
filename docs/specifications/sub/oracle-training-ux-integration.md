# Sub-spec: Oracle — Training UX Customization & Integration

**Parent:** [dynamic-mlops-training-ruchy-oracle-spec.md](../dynamic-mlops-training-ruchy-oracle-spec.md) Section 13.4-13.8

---

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

