# Sub-spec: REPL Grammar Testing — Example Validation and Execution

**Parent:** [test-grammer-repl.md](../test-grammer-repl.md) Sections 4-7

---


### 4. Example-Based Validation

```rust
// examples/validate_grammar.rs

use ruchy::{Repl, GrammarCoverageMatrix, GRAMMAR_PRODUCTIONS, AST_VARIANT_COUNT};
use std::{fs, time::{Duration, Instant}};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let coverage_mode = args.contains(&"--coverage".to_string());
    let strict_mode = args.contains(&"--strict".to_string());
    
    let mut coverage = GrammarCoverageMatrix::default();
    let mut repl = Repl::new();
    let mut failures = Vec::new();
    
    // Process all grammar example files
    for entry in fs::read_dir("examples/grammar").unwrap() {
        let path = entry.unwrap().path();
        if path.extension() != Some("ruchy".as_ref()) {
            continue;
        }
        
        let content = fs::read_to_string(&path).unwrap();
        
        println!("\n═══ {} ═══", path.file_name().unwrap().to_string_lossy());
        
        for (line_no, line) in content.lines().enumerate() {
            if line.trim().is_empty() || line.starts_with("//") {
                continue;
            }
            
            let start = Instant::now();
            match repl.eval(line) {
                Ok(val) => {
                    let elapsed = start.elapsed();
                    println!("✓ L{:03}: {} => {} [{:?}]", 
                        line_no + 1, 
                        truncate(line, 50), 
                        truncate(&val.to_string(), 30),
                        elapsed);
                    
                    if elapsed > Duration::from_millis(15) {
                        failures.push(format!("Slow: {} ({:?})", line, elapsed));
                    }
                    
                    coverage.record(line, Ok(()), elapsed);
                }
                Err(e) => {
                    println!("✗ L{:03}: {} => {}", line_no + 1, line, e);
                    failures.push(format!("Parse: {}", line));
                    coverage.record(line, Err(e), start.elapsed());
                }
            }
        }
    }
    
    // Coverage report
    if coverage_mode {
        print_coverage_report(&coverage);
    }
    
    // Failure summary
    if !failures.is_empty() {
        println!("\n═══ FAILURES ═══");
        for f in &failures {
            println!("  {}", f);
        }
    }
    
    // Exit code
    let exit_code = if strict_mode && !failures.is_empty() {
        1
    } else if strict_mode && coverage.productions.len() < GRAMMAR_PRODUCTIONS.len() {
        2
    } else {
        0
    };
    
    std::process::exit(exit_code);
}

fn print_coverage_report(coverage: &GrammarCoverageMatrix) {
    println!("\n═══ COVERAGE REPORT ═══");
    println!("Productions:  {}/{} ({:.1}%)", 
        coverage.productions.len(), 
        GRAMMAR_PRODUCTIONS.len(),
        coverage.productions.len() as f64 / GRAMMAR_PRODUCTIONS.len() as f64 * 100.0);
    println!("AST Variants: {}/{} ({:.1}%)", 
        coverage.ast_variants.len(),
        AST_VARIANT_COUNT,
        coverage.ast_variants.len() as f64 / AST_VARIANT_COUNT as f64 * 100.0);
    
    if !coverage.uncovered.is_empty() {
        println!("\nMissing Productions:");
        for prod in &coverage.uncovered {
            println!("  - {}", prod);
        }
    }
    
    // Performance stats
    let mut slowest: Vec<_> = coverage.productions.iter()
        .map(|(name, stats)| (name, stats.avg_latency_ns))
        .collect();
    slowest.sort_by_key(|&(_, ns)| ns);
    slowest.reverse();
    
    println!("\nSlowest Productions:");
    for (name, ns) in slowest.iter().take(5) {
        println!("  {} - {:.2}ms", name, *ns as f64 / 1_000_000.0);
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { 
        s 
    } else { 
        &s[..max.saturating_sub(3)].trim_end() 
    }
}
```

### 5. Example Grammar Files

Create directory structure:
```
examples/grammar/
├── 01_literals.ruchy
├── 02_operators.ruchy
├── 03_control_flow.ruchy
├── 04_pattern_matching.ruchy
├── 05_functions.ruchy
├── 06_pipelines.ruchy
├── 07_actors.ruchy
├── 08_dataframes.ruchy
└── 09_generics.ruchy
```

```rust
// examples/grammar/01_literals.ruchy

// Integer literals
42
0xFF
0b1010
1_000_000

// Float literals
3.14
2.71e-10
1.0e+10

// String literals
"hello"
"escape: \n \t \\ \""
r#"raw string with "quotes""#

// String interpolation
f"The value is {42}"
f"Complex: {compute(x):.2f}"

// Boolean literals
true
false

// Character literals
'a'
'\n'
'\u{1F600}'
```

```rust
// examples/grammar/06_pipelines.ruchy

// Basic pipeline
[1, 2, 3] |> filter(|x| x > 1) |> map(|x| x * 2)

// Method chaining
"hello world" |> split(" ") |> map(capitalize) |> join("_")

// DataFrame pipeline
DataFrame::read_csv("data.csv")
    |> filter(col("age") > 21)
    |> groupby("department")
    |> agg([mean("salary"), count()])
    |> sort_by("mean_salary", desc: true)

// Nested pipelines
data 
    |> map(|row| row.values |> sum())
    |> filter(|total| total > 100)

// Pipeline with error handling
read_file("input.txt")?
    |> lines()
    |> filter(|l| !l.is_empty())
    |> parse_records()?
    |> validate()?

// Pipeline with pattern matching
results
    |> map(|r| match r {
        Ok(v) => v * 2,
        Err(_) => 0,
    })
    |> filter(|x| x > 0)
```

## Execution Protocol

### Local Development
```bash
# Run exhaustive grammar tests
cargo test test_grammar_complete

# Run property tests (quick check)
cargo test prop_ -- --nocapture

# Run property tests (thorough - 1M iterations)
PROPTEST_CASES=1000000 cargo test prop_

# Validate examples
cargo run --example validate_grammar

# Generate coverage report
cargo run --example validate_grammar -- --coverage

# Strict mode (fail on any missing production)
cargo run --example validate_grammar -- --strict
```

### CI Pipeline
```yaml
name: Grammar Validation

on: [push, pull_request]

jobs:
  grammar:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Exhaustive Grammar Tests
        run: cargo test test_grammar_complete
        
      - name: Property Tests (1M iterations)
        run: PROPTEST_CASES=1000000 cargo test prop_
        
      - name: Example Validation
        run: cargo run --example validate_grammar -- --strict
        
      - name: Coverage Report
        run: |
          cargo run --example validate_grammar -- --coverage > coverage.txt
          if grep "Missing:" coverage.txt; then exit 1; fi
          
      - name: Benchmark Grammar
        run: cargo bench --bench grammar_perf
```

### Cargo.toml Configuration
```toml
[[example]]
name = "validate_grammar"
path = "examples/validate_grammar.rs"

[dev-dependencies]
proptest = "1.0"
criterion = "0.5"

[[bench]]
name = "grammar_perf"
harness = false
```

## Success Criteria

1. **Coverage**: 100% of grammar productions exercised
2. **Latency**: P99 < 15ms for any single production
3. **Stability**: 24-hour fuzzing without panic
4. **Properties**: 1M iterations without failure
5. **Examples**: All grammar files parse successfully
6. **Determinism**: Identical input → identical AST
7. **No Spurious Dependencies**: Simple code generates minimal Rust
8. **REPL Ergonomics**: Statement forms work without semicolons

## Failure Modes

### Production Not Implemented
```
thread 'test_grammar_complete' panicked at tests/grammar_exhaustive.rs:89:
Production 'actor_decl' failed without error tracking: actor Counter { state count: Int = 0 }
```

### Performance Regression
```
thread 'test_grammar_complete' panicked at tests/grammar_exhaustive.rs:95:
Production 'df_groupby' too slow: 47.3ms
```

### AST Variant Unreachable
```
thread 'test_grammar_complete' panicked at tests/grammar_exhaustive.rs:101:
AST variant 'Expr::Effect' never constructed
```

### Non-Deterministic Parsing
```
thread 'prop_deterministic_parsing' panicked:
Non-deterministic parsing for input: "x |> f() |> g()"
```

### Critical Regression: Interpolation
```
thread 'test_string_interpolation_no_f_prefix' panicked:
String with braces parsed as interpolation without 'f' prefix
Generated code contains 'Any' type for: "Hello, {name}!"
```

### Critical Regression: REPL Ergonomics
```
thread 'test_let_statement_without_semicolon' panicked:
Let statement without semicolon must parse in REPL
Input: "let x = 1"
Error: Failed to parse input
```

## Maintenance Protocol

Grammar test suite must be updated atomically with grammar changes:

1. Add production to `GRAMMAR_PRODUCTIONS` constant
2. Create example in `examples/grammar/`
3. Update property generators if new construct
4. Ensure CI passes before merge

Any grammar change without corresponding test update is a blocking issue.

## Performance Monitoring

Track grammar parsing performance over time:

```rust
// benches/grammar_perf.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_grammar_productions(c: &mut Criterion) {
    let mut group = c.benchmark_group("grammar");
    
    for (name, input) in GRAMMAR_PRODUCTIONS {
        group.bench_function(name, |b| {
            let mut repl = Repl::new();
            b.iter(|| {
                repl.eval(black_box(input))
            });
        });
    }
}

criterion_group!(benches, bench_grammar_productions);
criterion_main!(benches);
```

