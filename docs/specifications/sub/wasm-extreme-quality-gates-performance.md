# Sub-spec: WASM Extreme Quality -- Quality Gates, Performance, and Monitoring

**Parent:** [wasm-extreme-quality.md](../wasm-extreme-quality.md) Sections 4-9

---

## 4. Progressive Quality Gates

### Layer 1: Pre-Commit Hook (Fast Path)

```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

# Timing utilities
START_TIME=$(date +%s%3N)
MAX_TIME_MS=3000

# Get changed files
CHANGED_RS=$(git diff --cached --name-only --diff-filter=ACMR | grep '\.rs$' || true)

if [ -z "$CHANGED_RS" ]; then
    exit 0
fi

echo "🚀 Running fast quality checks..."

# 1. Format check (~100ms)
if ! cargo fmt -- --check $CHANGED_RS 2>/dev/null; then
    echo "❌ Formatting issues found. Run: cargo fmt"
    exit 1
fi

# 2. Fast clippy on changed files (~2s)
if ! cargo clippy --tests -- \
    -D warnings \
    -W clippy::cognitive_complexity \
    2>/dev/null; then
    echo "❌ Clippy warnings found"
    exit 1
fi

# 3. Check for SATD markers (~50ms)
if grep -E '(TODO|FIXME|HACK|XXX|REFACTOR)' $CHANGED_RS; then
    echo "⚠️  Self-admitted technical debt detected"
    echo "Consider addressing these before committing"
fi

# Check timing
END_TIME=$(date +%s%3N)
ELAPSED=$((END_TIME - START_TIME))

if [ $ELAPSED -gt $MAX_TIME_MS ]; then
    echo "⚠️  Pre-commit took ${ELAPSED}ms (target: <${MAX_TIME_MS}ms)"
    echo "Consider optimizing your pre-commit checks"
fi

echo "✅ Pre-commit checks passed in ${ELAPSED}ms"
```

### Layer 2: Pull Request Validation

```yaml
# .github/workflows/extreme-quality.yml
name: Extreme Quality Pipeline

on:
  pull_request:
    types: [opened, synchronize, reopened]
  push:
    branches: [main]

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always

jobs:
  quality-matrix:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
          targets: wasm32-unknown-unknown
          components: rustfmt, clippy, llvm-tools-preview
      
      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.os }}-${{ matrix.rust }}
      
      - name: Install tools
        run: |
          cargo install cargo-llvm-cov --locked
          cargo install cargo-mutants --locked
          cargo install wasm-pack --locked
          cargo install cargo-geiger --locked
      
      - name: Branch Coverage Analysis
        run: |
          cargo llvm-cov test \
            --all-features \
            --workspace \
            --branch \
            --fail-under-branches 90 \
            --fail-under-functions 95 \
            --lcov \
            --output-path coverage.lcov
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage.lcov
          flags: unittests
          fail_ci_if_error: true
      
      - name: Browser Testing Matrix
        if: matrix.os == 'ubuntu-latest'
        run: |
          # Install browsers
          npx playwright install --with-deps chromium firefox
          
          # Run tests in real browsers
          wasm-pack test --headless --chrome
          wasm-pack test --headless --firefox
      
      - name: JavaScript FFI Testing
        if: matrix.os == 'ubuntu-latest'
        run: |
          cd e2e-tests
          npm ci
          npm run test:coverage
          
          # Validate JS coverage
          npx nyc check-coverage \
            --branches 85 \
            --functions 90 \
            --lines 85
      
      - name: Mutation Testing
        if: matrix.os == 'ubuntu-latest' && matrix.rust == 'stable'
        run: |
          cargo mutants \
            --minimum-test-timeout 10 \
            --timeout-multiplier 1.5 \
            --jobs 2 \
            --output target/mutants.json
          
          # Parse results and fail if mutation score < 75%
          SCORE=$(jq '.summary.mutation_score' target/mutants.json)
          if (( $(echo "$SCORE < 0.75" | bc -l) )); then
            echo "Mutation score ${SCORE} is below 75% threshold"
            exit 1
          fi
      
      - name: Security Audit
        run: |
          cargo audit
          
          # Check for unsafe code
          if cargo geiger --all-features --output-format Json | \
             jq '.packages[].unsafety | select(.used.functions.unsafe > 0)' | \
             grep -q .; then
            echo "::warning::Unsafe code detected - requires justification"
          fi
      
      - name: Complexity Analysis
        run: |
          # Install complexity analyzer
          cargo install --git https://github.com/ruchy/pmat
          
          # Run analysis
          pmat analyze \
            --max-cyclomatic 10 \
            --max-cognitive 15 \
            --output target/complexity.json
          
          # Check thresholds
          if [ -f target/complexity.json ]; then
            HIGH_COMPLEXITY=$(jq '.violations | length' target/complexity.json)
            if [ "$HIGH_COMPLEXITY" -gt 0 ]; then
              echo "::error::Found $HIGH_COMPLEXITY functions exceeding complexity limits"
              jq '.violations' target/complexity.json
              exit 1
            fi
          fi
```

## 5. Performance Monitoring

### Benchmark Suite

```rust
// benches/wasm_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_suite(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_operations");
    
    // Test various payload sizes
    for size in [1024, 10240, 102400, 1048576].iter() {
        group.bench_with_input(
            BenchmarkId::new("allocation", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let data = vec![0u8; size];
                    black_box(data);
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("processing", size),
            size,
            |b, &size| {
                let data = vec![0u8; size];
                b.iter(|| {
                    ruchy::process_bytes(black_box(&data))
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(200)
        .measurement_time(std::time::Duration::from_secs(10))
        .warm_up_time(std::time::Duration::from_secs(3));
    targets = benchmark_suite
);
criterion_main!(benches);
```

### Size Analysis

```bash
#!/bin/bash
# scripts/analyze-size.sh

set -euo pipefail

echo "Building optimized WASM binary..."
cargo build --release --target wasm32-unknown-unknown

echo "Running wasm-opt optimization..."
wasm-opt -Oz \
  target/wasm32-unknown-unknown/release/*.wasm \
  -o target/optimized.wasm

echo "Size analysis:"
echo "==============="

# Original size
ORIGINAL=$(wc -c < target/wasm32-unknown-unknown/release/*.wasm)
echo "Original: $(numfmt --to=iec-i --suffix=B $ORIGINAL)"

# Optimized size
OPTIMIZED=$(wc -c < target/optimized.wasm)
echo "Optimized: $(numfmt --to=iec-i --suffix=B $OPTIMIZED)"

# Reduction
REDUCTION=$((ORIGINAL - OPTIMIZED))
PERCENT=$((REDUCTION * 100 / ORIGINAL))
echo "Reduction: $(numfmt --to=iec-i --suffix=B $REDUCTION) ($PERCENT%)"

# Detailed analysis with twiggy
echo -e "\nTop 10 largest functions:"
twiggy top -n 10 target/optimized.wasm

echo -e "\nMonomorphization bloat analysis:"
twiggy monos target/optimized.wasm

# Fail if binary exceeds size limit
MAX_SIZE=$((500 * 1024))  # 500KB
if [ $OPTIMIZED -gt $MAX_SIZE ]; then
    echo "ERROR: Optimized WASM size ($OPTIMIZED bytes) exceeds limit ($MAX_SIZE bytes)"
    exit 1
fi
```

## 6. Property-Based Testing

```rust
// tests/properties.rs
use proptest::prelude::*;
use quickcheck::{quickcheck, TestResult};

#[cfg(all(target_arch = "wasm32", test))]
use wasm_bindgen_test::*;

// Property: Serialization roundtrip
proptest! {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn prop_serialization_roundtrip(value: ArbitraryValue) {
        let serialized = ruchy::serialize(&value);
        let deserialized = ruchy::deserialize(&serialized).unwrap();
        prop_assert_eq!(value, deserialized);
    }
}

// Property: Memory safety across FFI
#[cfg(target_arch = "wasm32")]
proptest! {
    #[wasm_bindgen_test]
    fn prop_ffi_memory_safety(data: Vec<u8>) {
        // Ensure no buffer overflows when crossing FFI boundary
        let js_array = js_sys::Uint8Array::from(&data[..]);
        let rust_vec: Vec<u8> = js_array.to_vec();
        
        prop_assert_eq!(data.len(), rust_vec.len());
        prop_assert_eq!(data, rust_vec);
        
        // Verify no memory corruption
        let checksum_before = data.iter().fold(0u32, |acc, &x| acc.wrapping_add(x as u32));
        let checksum_after = rust_vec.iter().fold(0u32, |acc, &x| acc.wrapping_add(x as u32));
        prop_assert_eq!(checksum_before, checksum_after);
    }
}

// QuickCheck for differential testing
quickcheck! {
    fn qc_native_wasm_equivalence(input: Vec<i32>) -> TestResult {
        if input.is_empty() {
            return TestResult::discard();
        }
        
        let native_result = ruchy::native::sort(input.clone());
        let wasm_result = ruchy::wasm::sort(input.clone());
        
        TestResult::from_bool(native_result == wasm_result)
    }
}
```

## 7. Continuous Quality Dashboard

```yaml
# .github/workflows/dashboard.yml
name: Quality Dashboard

on:
  schedule:
    - cron: '0 0 * * *'  # Daily
  workflow_dispatch:

jobs:
  generate-metrics:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Collect Metrics
        run: |
          # Coverage trends
          cargo llvm-cov test --json > metrics/coverage.json
          
          # Complexity trends
          scc --format json src/ > metrics/complexity.json
          
          # Dependency audit
          cargo audit --json > metrics/audit.json
          
          # Binary size trends
          ./scripts/analyze-size.sh > metrics/size.txt
          
          # Performance benchmarks
          cargo bench --bench wasm_perf -- --save-baseline current
      
      - name: Generate Dashboard
        run: |
          python3 scripts/generate_dashboard.py \
            --coverage metrics/coverage.json \
            --complexity metrics/complexity.json \
            --audit metrics/audit.json \
            --size metrics/size.txt \
            --output dashboard.html
      
      - name: Deploy Dashboard
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./dashboard
```

## 8. Quality Metrics Summary

| Metric | Target | Rationale | Measurement |
|--------|--------|-----------|-------------|
| **Branch Coverage** | ≥90% | Ensures all decision paths tested | `cargo llvm-cov --branch` |
| **Function Coverage** | ≥95% | All public APIs must be tested | `cargo llvm-cov --function` |
| **Mutation Score** | ≥75% | Tests actually detect bugs | `cargo mutants` |
| **Cognitive Complexity** | ≤15 | Maintainable code | `cargo clippy` |
| **Cyclomatic Complexity** | ≤10 | Reduce branching complexity | `pmat analyze` |
| **WASM Size** | <500KB | Fast loading in browsers | `wc -c *.wasm` |
| **Unsafe Functions** | 0* | Memory safety | `cargo geiger` |
| **Pre-commit Time** | <3s | Developer experience | `time git commit` |
| **CI Pipeline Time** | <10min | Fast feedback | GitHub Actions |
| **FFI Test Coverage** | ≥85% | JS interop safety | `vitest coverage` |

*Exceptions allowed with safety documentation and review

## 9. Implementation Checklist

### Week 1: Foundation
- [ ] Remove global `rustflags` from `.cargo/config.toml`
- [ ] Implement fast pre-commit hook (<3s)
- [ ] Set up branch coverage pipeline
- [ ] Configure `cargo-llvm-cov` with proper exclusions

### Week 2: Browser Testing
- [ ] Set up `wasm-pack test` for Chrome/Firefox
- [ ] Create E2E test suite with Vitest
- [ ] Implement FFI boundary tests
- [ ] Add memory leak detection

### Week 3: Quality Gates
- [ ] Configure mutation testing
- [ ] Set up complexity analysis
- [ ] Implement security scanning
- [ ] Create quality dashboard

### Week 4: Optimization
- [ ] Profile WASM binary size
- [ ] Optimize critical paths
- [ ] Implement differential testing
- [ ] Add performance regression detection

