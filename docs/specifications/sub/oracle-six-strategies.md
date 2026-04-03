# Sub-spec: Oracle — Six-Strategy Acceleration Pipeline

**Parent:** [dynamic-mlops-training-ruchy-oracle-spec.md](../dynamic-mlops-training-ruchy-oracle-spec.md) Section 3

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

