# ADR-004: Oracle ML Error Classification Architecture

## Status

Accepted

## Date

2024-03-01

## Context

Ruchy's Oracle system provides ML-powered error classification to help users understand and fix compiler errors. Design requirements:

- Classify errors into actionable categories
- Provide contextual fix suggestions
- Support offline/embedded operation
- Ensure reproducible predictions
- Enable continuous improvement via user feedback

## Decision

We implement a **lightweight ensemble classifier** with:

1. **Feature extraction**: AST-aware error context
2. **Classification**: Gradient boosted decision tree ensemble
3. **Suggestion generation**: Template-based with context filling
4. **Feedback loop**: RLHF-style preference learning

Architecture:
```
Error → FeatureExtractor → Classifier → Category + Confidence
                                ↓
                        SuggestionGenerator → Fix Template
                                ↓
                        UserFeedback → Model Update (optional)
```

Key design choices:
- Model embedded in binary (no network dependency)
- Deterministic inference with fixed random seeds
- 8 error categories (TypeMismatch, BorrowChecker, LifetimeError, etc.)
- Confidence threshold 0.7 for suggestions

## Consequences

### Positive

- **Offline operation**: No API calls, works airgapped
- **Fast inference**: <1ms classification time
- **Reproducibility**: Fixed seeds ensure deterministic output
- **Privacy**: No error data sent externally

### Negative

- **Model size**: ~2MB embedded model adds to binary size
- **Update friction**: New model requires new release
- **Limited context**: Cannot query external knowledge

### Neutral

- Future: Optional cloud model for enhanced suggestions

## Reproducibility Configuration

```toml
# .ruchy/oracle.toml
[reproducibility]
random_seed = 42
deterministic_mode = true
model_version = "v1.2.0"
model_checksum = "sha256:abc123..."

[training]
seed = 42
cv_folds = 5
test_split = 0.2
```

Environment variables:
- `RUCHY_ORACLE_SEED=42`: Override random seed
- `RUCHY_ORACLE_DETERMINISTIC=1`: Force deterministic mode

## Alternatives Considered

### Large Language Model (GPT/Claude API)

Rejected:
- Network dependency
- Privacy concerns with error data
- Latency (100ms+ vs <1ms)
- Cost per query

### Rule-Based System

Rejected:
- Brittle pattern matching
- Cannot generalize to novel errors
- High maintenance burden

### Neural Network Classifier

Rejected:
- Larger model size
- Slower inference
- Harder to interpret

## References

- scikit-learn Gradient Boosting documentation
- Ruchy Oracle implementation: `src/oracle/`
- Model training pipeline: `scripts/train_oracle.py`
