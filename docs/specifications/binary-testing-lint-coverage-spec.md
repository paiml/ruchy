# Binary Testing, Lint, and Coverage Specification

## Overview
Comprehensive testing infrastructure for Ruchy compiler including binary validation, lint enforcement, and coverage tracking.

## 1. Binary Testing Framework

### 1.1 Test Binary Generation
```bash
# Generate test binaries for all examples
cargo build --examples --release
```

### 1.2 Binary Validation Suite
```rust
// tests/binary_validation.rs
#[test]
fn test_binary_execution() {
    let examples = fs::read_dir("examples").unwrap();
    for example in examples {
        let path = example.unwrap().path();
        if path.extension() == Some("ruchy") {
            let output = Command::new("ruchy")
                .arg("run")
                .arg(&path)
                .output()
                .expect("Failed to execute");
            
            assert!(output.status.success());
            validate_output(&output.stdout);
        }
    }
}
```

### 1.3 Snapshot Testing
```rust
// Use insta for snapshot testing
#[test]
fn test_transpiler_output() {
    let input = "let x = 42";
    let output = transpile(input);
    insta::assert_snapshot!(output);
}
```

## 2. Lint Infrastructure

### 2.1 Clippy Configuration
```toml
# clippy.toml
cognitive-complexity-threshold = 10
too-many-arguments-threshold = 5
type-complexity-threshold = 250
```

### 2.2 Custom Lints
```rust
// src/lints/mod.rs
pub struct RuchyLint {
    rules: Vec<Box<dyn LintRule>>,
}

trait LintRule {
    fn check(&self, ast: &Ast) -> Vec<LintViolation>;
    fn severity(&self) -> Severity;
}

// Example: No unwrap() in production code
struct NoUnwrapRule;

impl LintRule for NoUnwrapRule {
    fn check(&self, ast: &Ast) -> Vec<LintViolation> {
        ast.find_all("unwrap()")
            .map(|loc| LintViolation {
                location: loc,
                message: "Use ? or expect() instead of unwrap()",
                severity: Severity::Error,
            })
            .collect()
    }
}
```

### 2.3 Make Lint Target
```makefile
# Makefile addition
.PHONY: lint
lint:
	@echo "Running Clippy with strict settings..."
	cargo clippy --all-targets --all-features -- -D warnings
	@echo "Running custom lints..."
	cargo run --bin ruchy-lint -- src/
	@echo "Checking documentation..."
	cargo doc --no-deps --document-private-items
```

## 3. Coverage Infrastructure

### 3.1 Tarpaulin Configuration
```toml
# tarpaulin.toml
[default]
workspace = true
all-features = true
engine = "llvm"
exclude-files = ["*/tests/*", "*/benches/*"]
ignore-panics = true
timeout = "600s"
```

### 3.2 Coverage Targets
```makefile
# Coverage targets
coverage:
	cargo tarpaulin --out Html --output-dir target/coverage

coverage-ci:
	cargo tarpaulin --out Xml --output-dir target/coverage --fail-under 80
```

### 3.3 Coverage Reports
```yaml
# .github/workflows/coverage.yml
name: Coverage
on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      
      - name: Generate coverage
        run: cargo tarpaulin --out Xml
      
      - name: Upload to codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml
          fail_ci_if_error: true
```

## 4. Integration Testing

### 4.1 End-to-End Tests
```rust
// tests/e2e/mod.rs
#[test]
fn test_full_compilation_pipeline() {
    let source = r#"
        fn fibonacci(n: i32) -> i32 {
            match n {
                0 | 1 => n,
                _ => fibonacci(n - 1) + fibonacci(n - 2)
            }
        }
        
        println(fibonacci(10))
    "#;
    
    // Parse
    let ast = parse(source).unwrap();
    
    // Type check
    let typed_ast = type_check(ast).unwrap();
    
    // Transpile
    let rust_code = transpile(typed_ast).unwrap();
    
    // Compile and run
    let output = compile_and_run(rust_code).unwrap();
    
    assert_eq!(output.trim(), "55");
}
```

### 4.2 Property-Based Testing
```rust
// tests/property.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parser_round_trip(input in valid_ruchy_code()) {
        let ast = parse(&input).unwrap();
        let output = pretty_print(&ast);
        let reparsed = parse(&output).unwrap();
        
        prop_assert_eq!(ast, reparsed);
    }
    
    #[test]
    fn test_type_inference_soundness(expr in typed_expression()) {
        let inferred = infer_type(&expr).unwrap();
        let checked = check_type(&expr, &inferred);
        
        prop_assert!(checked.is_ok());
    }
}
```

## 5. Benchmark Suite

### 5.1 Criterion Benchmarks
```rust
// benches/parser_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn parser_benchmark(c: &mut Criterion) {
    let input = include_str!("../corpus/large.ruchy");
    
    c.bench_function("parse 10k LOC", |b| {
        b.iter(|| parse(black_box(input)))
    });
    
    c.bench_function("parse with recovery", |b| {
        b.iter(|| parse_with_recovery(black_box(input)))
    });
}

criterion_group!(benches, parser_benchmark);
criterion_main!(benches);
```

### 5.2 Performance Regression Tests
```rust
// tests/performance.rs
#[test]
fn test_parsing_throughput() {
    let input = generate_ruchy_code(1_000_000); // 1MB
    let start = Instant::now();
    
    let _ = parse(&input);
    
    let elapsed = start.elapsed();
    let throughput = 1_000_000.0 / elapsed.as_secs_f64();
    
    // Must maintain >50MB/s
    assert!(throughput > 50_000_000.0, 
            "Parsing throughput {:.2}MB/s below 50MB/s", 
            throughput / 1_000_000.0);
}
```

## 6. Mutation Testing

### 6.1 Cargo Mutants Configuration
```toml
# .cargo/mutants.toml
[mutants]
exclude_globs = ["tests/**", "benches/**"]
timeout = 30
jobs = 4
```

### 6.2 Mutation Testing Target
```makefile
mutation-test:
	cargo install cargo-mutants
	cargo mutants --jobs 4 --timeout 30
```

## 7. Fuzz Testing

### 7.1 Fuzz Targets
```rust
// fuzz/fuzz_targets/parser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = ruchy::parse(s);
    }
});
```

### 7.2 Fuzz Configuration
```toml
# fuzz/Cargo.toml
[dependencies]
libfuzzer-sys = "0.4"
ruchy = { path = ".." }

[[bin]]
name = "parser"
path = "fuzz_targets/parser.rs"
```

## 8. Quality Gates

### 8.1 Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

echo "Running quality gates..."

# Lint check
make lint || exit 1

# Test check  
cargo test --quiet || exit 1

# Coverage check
cargo tarpaulin --fail-under 80 --print-summary || exit 1

# Complexity check
find src -name "*.rs" -exec ruchy-complexity {} \; | \
    awk '$2 > 10 {exit 1}'

echo "All quality gates passed!"
```

### 8.2 CI Quality Enforcement
```yaml
# .github/workflows/quality.yml
name: Quality Gates
on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Lint
        run: make lint
        
      - name: Test
        run: cargo test --all-features
        
      - name: Coverage
        run: cargo tarpaulin --fail-under 80
        
      - name: Mutation Test
        run: cargo mutants --timeout 30
        
      - name: Benchmark
        run: cargo bench --no-fail-fast
```

## 9. Documentation Testing

### 9.1 Doc Tests
```rust
/// Example usage of the parser
/// 
/// ```
/// use ruchy::parse;
/// 
/// let input = "let x = 42";
/// let ast = parse(input).unwrap();
/// assert_eq!(ast.kind, AstKind::Let);
/// ```
pub fn parse(input: &str) -> Result<Ast, ParseError> {
    // Implementation
}
```

### 9.2 Example Validation
```makefile
test-examples:
	@for example in examples/*.ruchy; do \
		echo "Testing $$example..."; \
		ruchy run $$example || exit 1; \
	done
```

## 10. Implementation Tasks

### Task ID: RUCHY-0500
**Priority**: P0 (Critical)
**Complexity**: 8/10
**Dependencies**: None

### Subtasks:
1. [ ] Set up tarpaulin for coverage tracking
2. [ ] Implement snapshot testing with insta
3. [ ] Create property-based test suite
4. [ ] Add mutation testing with cargo-mutants
5. [ ] Set up fuzz testing infrastructure
6. [ ] Create benchmark suite with criterion
7. [ ] Implement custom lint rules
8. [ ] Add pre-commit hooks
9. [ ] Configure CI/CD quality gates
10. [ ] Document testing best practices

### Acceptance Criteria:
- Coverage >80% on all modules
- Zero clippy warnings with -D warnings
- All examples execute successfully
- Mutation score >75%
- Parsing throughput >50MB/s
- All quality gates enforced in CI

### Performance Targets:
- Test suite execution <60s
- Coverage generation <2min
- Lint check <10s
- Binary validation <30s