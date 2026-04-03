# Sub-spec: Oracle — Drift Detection, Quality Gates & Reviews

**Parent:** [dynamic-mlops-training-ruchy-oracle-spec.md](../dynamic-mlops-training-ruchy-oracle-spec.md) Sections 4-12

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

