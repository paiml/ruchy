# Reproducibility Guide

This document explains how to reproduce all results, benchmarks, and model training in the Ruchy project.

## Quick Start

```bash
# Clone with full history
git clone --depth=1 https://github.com/noahgift/ruchy.git
cd ruchy

# Option 1: Nix (recommended for full reproducibility)
nix develop

# Option 2: Docker
docker build -t ruchy .
docker run -it ruchy

# Option 3: Manual setup
rustup install 1.83.0
rustup default 1.83.0
cargo build --release
```

## Environment Reproducibility

### Nix Flake (Recommended)

The `flake.nix` pins all dependencies including:
- Rust toolchain version (1.83.0)
- System dependencies (openssl, pkg-config)
- Development tools (cargo-criterion, cargo-mutants)

```bash
# Enter reproducible environment
nix develop

# Build with exact pinned dependencies
nix build

# Run tests
nix flake check
```

### Docker

The `Dockerfile` provides a containerized build environment:

```bash
# Build image
docker build -t ruchy:1.0.0 .

# Run tests in container
docker run ruchy:1.0.0 cargo test

# Interactive development
docker run -it -v $(pwd):/src ruchy:1.0.0 /bin/bash
```

### Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `RUCHY_TEST_SEED` | Random seed for tests | 42 |
| `PROPTEST_SEED` | Property test seed | 12345678901234567890123456789012 |
| `RUCHY_ORACLE_SEED` | Oracle ML seed | 42 |
| `RUCHY_ORACLE_DETERMINISTIC` | Force deterministic mode | 1 |
| `SOURCE_DATE_EPOCH` | Build timestamp | 1704067200 |

## Random Seed Management

### Configuration File

All seeds are centralized in `.ruchy/reproducibility.toml`:

```toml
[random_seeds]
master_seed = 42
parser_seed = 42
oracle_seed = 42
fuzzer_seed = 42
property_test_seed = 12345678901234567890123456789012
```

### Usage in Code

```rust
use ruchy::reproducibility::get_seed;

fn main() {
    let seed = get_seed("parser"); // Returns 42
    let rng = StdRng::seed_from_u64(seed);
}
```

### Property Tests

Proptest seeds are logged for reproduction:

```bash
# If a test fails, you'll see:
# proptest: Seed: 0x123456789abcdef0123456789abcdef0

# Reproduce with:
PROPTEST_SEED=0x123456789abcdef0123456789abcdef0 cargo test
```

## Model Versioning (DVC)

### Setup

```bash
# Initialize DVC (already done)
dvc init

# Configure remote storage
dvc remote add -d local /tmp/ruchy-dvc-cache
```

### Reproduce Training Pipeline

```bash
# Reproduce entire pipeline
dvc repro

# Reproduce specific stage
dvc repro train_model

# Check pipeline status
dvc status
```

### Model Checkpoints

Models are versioned with checksums:

```toml
# .ruchy/reproducibility.toml
[oracle]
model_version = "v1.2.0"
model_checksum = "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
```

## Benchmark Reproduction

### Run Benchmarks

```bash
# Full benchmark suite
cargo bench

# Specific benchmark
cargo bench --bench parser_benchmarks

# With reproducibility settings
RUCHY_BENCH_SEED=42 cargo bench
```

### Compare Results

```bash
# Save baseline
cargo bench -- --save-baseline main

# Compare to baseline
cargo bench -- --baseline main
```

### Statistical Verification

All benchmarks include:
- 95% confidence intervals
- Effect sizes (Cohen's d)
- Sample sizes ≥ 64

See `docs/BENCHMARK_METHODOLOGY.md` for details.

## Test Reproduction

### Unit Tests

```bash
# Deterministic test run
RUCHY_TEST_SEED=42 cargo test

# Specific test with seed
RUCHY_TEST_SEED=42 cargo test test_parser_expression
```

### Property Tests

```bash
# Standard run (100 cases)
cargo test --test property_based_tests

# Extended run (10,000 cases)
PROPTEST_CASES=10000 cargo test --test property_based_tests
```

### Mutation Tests

```bash
# Incremental (fast)
cargo mutants --file src/frontend/parser/core.rs

# Full suite (slow)
cargo mutants --timeout 300
```

## Verification Checklist

Before claiming reproducibility:

- [ ] `nix build` succeeds
- [ ] `docker build` succeeds
- [ ] `cargo test` passes with `RUCHY_TEST_SEED=42`
- [ ] `cargo bench` results within CI ± 10%
- [ ] `dvc repro` reproduces model training
- [ ] Model checksum matches `.ruchy/reproducibility.toml`

## Troubleshooting

### Different Results Across Runs

1. Check environment variables are set
2. Verify Rust toolchain version: `rustc --version`
3. Check Cargo.lock is committed
4. Ensure no parallel test execution: `cargo test -- --test-threads=1`

### Model Checksum Mismatch

1. Verify DVC cache is populated: `dvc status`
2. Pull cached artifacts: `dvc pull`
3. Check training seed: `grep train_seed .ruchy/reproducibility.toml`

## References

- [Nix Flakes](https://nixos.wiki/wiki/Flakes)
- [DVC Documentation](https://dvc.org/doc)
- [Criterion.rs](https://bheisler.github.io/criterion.rs/book/)
- [Proptest](https://proptest-rs.github.io/proptest/proptest/index.html)
