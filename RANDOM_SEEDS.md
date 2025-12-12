# Random Seed Management

This document describes how random seeds are managed in the Ruchy project to ensure reproducible results.

## Overview

All randomized operations in Ruchy use deterministic seeds to ensure reproducibility. Seeds are centralized in `.ruchy/reproducibility.toml` and can be overridden via environment variables.

## Seed Configuration

### Primary Configuration File

Location: `.ruchy/reproducibility.toml`

```toml
[random_seeds]
master_seed = 42
parser_seed = 42
oracle_seed = 42
fuzzer_seed = 42
property_test_seed = 12345678901234567890123456789012
```

### Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `RUCHY_SEED` | Master seed for all operations | 42 |
| `RUCHY_TEST_SEED` | Unit test random seed | 42 |
| `RUCHY_ORACLE_SEED` | Oracle ML model seed | 42 |
| `RUCHY_FUZZER_SEED` | Fuzzing seed | 42 |
| `PROPTEST_SEED` | Property-based test seed | 12345678901234567890123456789012 |

## Usage

### Running Tests Deterministically

```bash
# Use default seeds
cargo test

# Override seed
RUCHY_TEST_SEED=12345 cargo test

# Reproduce specific proptest failure
PROPTEST_SEED=0x123456789abcdef0 cargo test
```

### Benchmarks

```bash
# Deterministic benchmarks
RUCHY_SEED=42 cargo bench
```

### Oracle Training

```bash
# Train with deterministic seed
RUCHY_ORACLE_SEED=42 cargo run --bin train-oracle
```

## Implementation Details

### Rust Code Pattern

```rust
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn get_deterministic_rng() -> ChaCha8Rng {
    let seed = std::env::var("RUCHY_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);
    ChaCha8Rng::seed_from_u64(seed)
}
```

### Property Tests

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_parser_roundtrip(input in any::<String>()) {
        // Test body
    }
}
```

## Reproducing Results

1. Clone repository at specific commit
2. Set environment variables (or use defaults)
3. Run with `--test-threads=1` for deterministic ordering

```bash
git checkout v1.2.0
export RUCHY_SEED=42
cargo test -- --test-threads=1
```

## Seed Derivation

Component seeds are derived from the master seed:

```
master_seed = 42
├── parser_seed  = hash(master_seed, "parser")  = 42
├── oracle_seed  = hash(master_seed, "oracle")  = 42
├── fuzzer_seed  = hash(master_seed, "fuzzer")  = 42
└── proptest_seed = hash(master_seed, "proptest") = 12345...
```

## CI/CD Integration

GitHub Actions uses fixed seeds:

```yaml
env:
  RUCHY_SEED: 42
  PROPTEST_SEED: 12345678901234567890123456789012
```

## Verification

To verify reproducibility:

```bash
# Run twice with same seed
RUCHY_SEED=42 cargo test 2>&1 | tee run1.log
RUCHY_SEED=42 cargo test 2>&1 | tee run2.log

# Compare outputs (should be identical)
diff run1.log run2.log
```
