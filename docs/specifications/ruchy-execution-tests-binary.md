# Ruchy Execution Testing Specification

## Test Organization

```
tests/
├── cli/                    # CLI command integration tests
│   ├── eval.rs            # -e flag tests
│   ├── parse.rs           # -p flag tests  
│   ├── transpile.rs       # -t flag tests
│   ├── compile.rs         # -c flag tests
│   └── fixtures/          # Test input files
├── repl/                   # REPL interaction tests
│   ├── commands.rs        # REPL metacommands
│   ├── completion.rs      # Tab completion
│   └── multiline.rs       # Multi-line input handling
├── oneliner/              # One-liner specific tests
│   ├── pipes.rs           # Unix pipe integration
│   ├── filters.rs        # Data transformation
│   └── scripts.sh         # Shell-based integration
└── execution/             # Unified execution tests
    ├── correctness.rs     # Semantic validation
    ├── performance.rs     # Benchmark suite
    └── interop.rs         # Rust crate FFI

```

## Makefile Targets

```makefile
# Primary test targets
test-execution: test-cli test-repl test-oneliner
	@echo "✓ All execution modes validated"

test-cli:
	@echo "Testing CLI commands..."
	@cargo test --test cli_integration --features test-cli
	@./tests/cli/smoke_test.sh

test-repl:
	@echo "Testing REPL..."
	@cargo test --test repl_integration --features test-repl
	@expect tests/repl/interactive.exp

test-oneliner:
	@echo "Testing one-liners..."
	@./tests/oneliner/suite.sh
	@cargo test --test oneliner_integration

# Granular targets
test-eval:
	@cargo test --test cli_integration -- eval::

test-parse:
	@cargo test --test cli_integration -- parse::

test-transpile:
	@cargo test --test cli_integration -- transpile::

# Performance validation
bench-execution:
	@cargo bench --bench execution_bench
	@./tests/execution/compare_baseline.py

# Property-based testing
test-properties:
	@cargo test --test property_execution --features proptest

# Fuzzing
fuzz-parser:
	@cargo +nightly fuzz run parser_fuzz -- -max_total_time=60

# Coverage
coverage-execution:
	@cargo tarpaulin --test-types tests --out Html \
		--exclude-files "*/tests/*" \
		--output-dir target/coverage
```

## CLI Integration Tests (`tests/cli/cli_integration.rs`)

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;

#[test]
fn eval_expression() {
    Command::cargo_bin("ruchy")
        .arg("-e")
        .arg("2 + 2")
        .assert()
        .success()
        .stdout("4\n");
}

#[test]
fn eval_with_imports() {
    Command::cargo_bin("ruchy")
        .arg("-e")
        .arg("import std::collections::HashMap; HashMap::new()")
        .assert()
        .success()
        .stdout(predicate::str::contains("{}"));
}

#[test]
fn parse_only_valid() {
    Command::cargo_bin("ruchy")
        .arg("-p")
        .arg("fn main() { println!(\"test\") }")
        .assert()
        .success()
        .stdout(predicate::str::contains("AST"));
}

#[test]
fn parse_only_invalid() {
    Command::cargo_bin("ruchy")
        .arg("-p")
        .arg("fn main() {")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unclosed brace"));
}

#[test]
fn transpile_to_rust() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "fn square(x: i32) -> i32 {{ x * x }}").unwrap();
    
    Command::cargo_bin("ruchy")
        .arg("-t")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("fn square(x: i32) -> i32"));
}

#[test]
fn compile_to_binary() {
    let src = NamedTempFile::new().unwrap();
    writeln!(src, "fn main() {{ println!(\"Hello\") }}").unwrap();
    
    Command::cargo_bin("ruchy")
        .arg("-c")
        .arg(src.path())
        .arg("-o")
        .arg("test_binary")
        .assert()
        .success();
    
    Command::new("./test_binary")
        .assert()
        .success()
        .stdout("Hello\n");
}

#[test]
fn pipeline_composition() {
    Command::cargo_bin("ruchy")
        .arg("-e")
        .arg("stdin.lines().map(|l| l.to_uppercase()).collect()")
        .write_stdin("hello\nworld")
        .assert()
        .success()
        .stdout("HELLO\nWORLD\n");
}
```

## REPL Tests (`tests/repl/repl_integration.rs`)

```rust
use rexpect::{spawn, ReadUntil};

#[test]
fn repl_basic_evaluation() {
    let mut repl = spawn("cargo run -- --repl", Some(3000)).unwrap();
    repl.exp_string("ruchy> ").unwrap();
    repl.send_line("2 + 2").unwrap();
    repl.exp_string("4").unwrap();
}

#[test]
fn repl_multiline_function() {
    let mut repl = spawn("cargo run -- --repl", Some(3000)).unwrap();
    repl.exp_string("ruchy> ").unwrap();
    repl.send_line("fn factorial(n: u64) -> u64 {").unwrap();
    repl.exp_string("     > ").unwrap();
    repl.send_line("    if n <= 1 { 1 } else { n * factorial(n - 1) }").unwrap();
    repl.exp_string("     > ").unwrap();
    repl.send_line("}").unwrap();
    repl.exp_string("ruchy> ").unwrap();
    repl.send_line("factorial(5)").unwrap();
    repl.exp_string("120").unwrap();
}

#[test]
fn repl_metacommands() {
    let mut repl = spawn("cargo run -- --repl", Some(3000)).unwrap();
    repl.exp_string("ruchy> ").unwrap();
    
    // Test :help
    repl.send_line(":help").unwrap();
    repl.exp_string("Available commands:").unwrap();
    
    // Test :type
    repl.send_line(":type vec![1, 2, 3]").unwrap();
    repl.exp_string("Vec<i32>").unwrap();
    
    // Test :time
    repl.send_line(":time (0..1000).sum()").unwrap();
    repl.exp_regex(r"Execution time: \d+\.\d+ms").unwrap();
}

#[test]
fn repl_tab_completion() {
    let mut repl = spawn("cargo run -- --repl", Some(3000)).unwrap();
    repl.exp_string("ruchy> ").unwrap();
    repl.send("std::coll\t").unwrap();
    repl.exp_string("std::collections").unwrap();
}
```

## One-liner Test Suite (`tests/oneliner/suite.sh`)

```bash
#!/usr/bin/env bash
set -euo pipefail

RUCHY="${CARGO_TARGET_DIR:-target}/debug/ruchy"
FAILURES=0

# Test helper
test_oneliner() {
    local name="$1"
    local input="$2"
    local expected="$3"
    local cmd="$4"
    
    echo -n "Testing $name... "
    result=$(echo "$input" | $RUCHY -e "$cmd" 2>&1)
    
    if [[ "$result" == "$expected" ]]; then
        echo "✓"
    else
        echo "✗"
        echo "  Expected: $expected"
        echo "  Got:      $result"
        ((FAILURES++))
    fi
}

# Basic transformations
test_oneliner "uppercase" "hello" "HELLO" \
    "stdin.read_line().to_uppercase().trim()"

test_oneliner "word_count" "one two three" "3" \
    "stdin.read_line().split_whitespace().count()"

test_oneliner "sum_numbers" "1 2 3 4 5" "15" \
    "stdin.read_line().split_whitespace().map(|s| s.parse::<i32>().unwrap()).sum::<i32>()"

# JSON processing
test_oneliner "json_extract" '{"name":"test","value":42}' "42" \
    "use serde_json::Value; let v: Value = serde_json::from_str(&stdin.read_line()).unwrap(); v[\"value\"]"

# CSV filtering
test_oneliner "csv_filter" "name,age\nalice,30\nbob,25" "alice,30" \
    "stdin.lines().skip(1).filter(|line| line.contains(\"alice\")).next().unwrap()"

# Numeric operations
test_oneliner "average" "10\n20\n30" "20" \
    "let nums: Vec<f64> = stdin.lines().map(|l| l.parse().unwrap()).collect(); (nums.iter().sum::<f64>() / nums.len() as f64) as i32"

# Exit status
if [[ $FAILURES -eq 0 ]]; then
    echo "All one-liner tests passed!"
    exit 0
else
    echo "$FAILURES one-liner tests failed"
    exit 1
fi
```

## Property Tests (`tests/execution/property_tests.rs`)

```rust
use proptest::prelude::*;
use ruchy::{eval, parse, transpile};

proptest! {
    #[test]
    fn parse_transpile_roundtrip(code in valid_ruchy_code()) {
        let ast = parse(&code).unwrap();
        let rust_code = transpile(&ast).unwrap();
        let reparsed = parse(&rust_code).unwrap();
        prop_assert_eq!(ast, reparsed);
    }
    
    #[test]
    fn eval_deterministic(expr in arithmetic_expr()) {
        let result1 = eval(&expr).unwrap();
        let result2 = eval(&expr).unwrap();
        prop_assert_eq!(result1, result2);
    }
    
    #[test]
    fn type_preservation(typed_expr in well_typed_expr()) {
        let inferred_type = infer_type(&typed_expr).unwrap();
        let runtime_type = eval(&typed_expr).unwrap().type_of();
        prop_assert_eq!(inferred_type, runtime_type);
    }
}

fn valid_ruchy_code() -> impl Strategy<Value = String> {
    prop_oneof![
        function_def(),
        expression_stmt(),
        control_flow(),
    ]
}

fn arithmetic_expr() -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        Just("0"),
        Just("1"),
        Just("42"),
    ];
    
    leaf.prop_recursive(8, 256, 10, |inner| {
        prop_oneof![
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("{} + {}", a, b)),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("{} * {}", a, b)),
            inner.prop_map(|x| format!("({})", x)),
        ]
    })
}
```

## Performance Benchmarks (`benches/execution_bench.rs`)

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ruchy::{eval, compile_jit, transpile_and_compile};

fn benchmark_execution_modes(c: &mut Criterion) {
    let code = r#"
        fn fibonacci(n: u64) -> u64 {
            if n <= 1 { n } else { fibonacci(n - 1) + fibonacci(n - 2) }
        }
        fibonacci(20)
    "#;
    
    c.bench_function("interpreter", |b| {
        b.iter(|| eval(black_box(code)))
    });
    
    c.bench_function("jit", |b| {
        let compiled = compile_jit(code).unwrap();
        b.iter(|| compiled.execute())
    });
    
    c.bench_function("transpiled", |b| {
        let binary = transpile_and_compile(code).unwrap();
        b.iter(|| binary.run())
    });
}

fn benchmark_startup_time(c: &mut Criterion) {
    c.bench_function("repl_startup", |b| {
        b.iter(|| {
            std::process::Command::new("ruchy")
                .arg("--repl")
                .arg("--eval-and-exit")
                .arg("1+1")
                .output()
                .unwrap()
        })
    });
    
    c.bench_function("script_startup", |b| {
        b.iter(|| {
            std::process::Command::new("ruchy")
                .arg("-e")
                .arg("1+1")
                .output()
                .unwrap()
        })
    });
}

criterion_group!(benches, benchmark_execution_modes, benchmark_startup_time);
criterion_main!(benches);
```

## Continuous Integration (`tests/execution/validate.rs`)

```rust
//! Performance validation harness using Rust tooling

use std::process::Command;
use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
struct BenchmarkResult {
    median: f64,
    mean: f64,
    stddev: f64,
}

const PERFORMANCE_TARGETS: &str = r#"
script_startup: 10      # ms
repl_response: 15       # ms  
transpile_overhead: 1.05 # 5% max overhead
binary_size: 5242880    # 5MB
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    
    // Parse performance targets
    let targets: serde_yaml::Value = serde_yaml::from_str(PERFORMANCE_TARGETS)?;
    
    // Run benchmarks and collect results
    let output = Command::new("cargo")
        .args(&["bench", "--bench", "execution_bench", "--", "--output-format", "json"])
        .output()?;
    
    let bench_results: Vec<BenchmarkResult> = serde_json::from_slice(&output.stdout)?;
    
    // Validate startup time
    if let Some(startup) = bench_results.iter().find(|b| b.name == "script_startup") {
        let target = targets["script_startup"].as_f64().unwrap();
        if startup.median > target {
            failures.push(format!(
                "Startup time {:.2}ms exceeds target {:.2}ms", 
                startup.median, target
            ));
        }
    }
    
    // Validate binary size
    let metadata = fs::metadata("target/release/ruchy")?;
    let binary_size = metadata.len();
    let size_target = targets["binary_size"].as_u64().unwrap();
    
    if binary_size > size_target {
        failures.push(format!(
            "Binary size {} exceeds target {}", 
            binary_size, size_target
        ));
    }
    
    // Report results
    if failures.is_empty() {
        println!("✓ All performance targets met");
        Ok(())
    } else {
        for failure in &failures {
            eprintln!("✗ {}", failure);
        }
        std::process::exit(1);
    }
}
```

Or alternatively, as a Ruchy script (`tests/execution/validate.ruchy`):

```rust
#!/usr/bin/env ruchy

use std::process::Command
use std::fs

const TARGETS = {
    script_startup: 10ms,
    repl_response: 15ms,
    transpile_overhead: 1.05,
    binary_size: 5MB
}

fn validate_performance() -> Result<(), String> {
    let failures = []
    
    // Run benchmarks via cargo
    let bench_output = Command::new("cargo")
        .args(["bench", "--bench", "execution_bench", "--quiet"])
        .output()?
    
    // Parse benchmark results
    let results = bench_output.stdout
        .lines()
        .filter_map(|line| parse_bench_line(line))
        .collect::<HashMap<_, _>>()
    
    // Check startup time
    if let Some(startup_ms) = results.get("script_startup") {
        if startup_ms > TARGETS.script_startup {
            failures.push($"Startup: {startup_ms} > {TARGETS.script_startup}")
        }
    }
    
    // Check binary size
    let binary_size = fs::metadata("target/release/ruchy")?.len()
    if binary_size > TARGETS.binary_size {
        failures.push($"Binary: {binary_size} > {TARGETS.binary_size}")
    }
    
    // Check transpilation overhead
    if let (Some(rust), Some(ruchy)) = (results.get("native_rust"), results.get("transpiled")) {
        let overhead = ruchy / rust
        if overhead > TARGETS.transpile_overhead {
            failures.push($"Overhead: {overhead:.2} > {TARGETS.transpile_overhead}")
        }
    }
    
    match failures {
        [] => Ok(println!("✓ Performance validated")),
        _ => Err(failures.join("\n"))
    }
}

fn parse_bench_line(line: &str) -> Option<(&str, f64)> {
    // Parse criterion output: "test_name ... bench: 1,234 ns/iter"
    let parts = line.split_whitespace().collect::<Vec<_>>()
    if parts.len() >= 4 && parts[2] == "bench:" {
        let name = parts[0]
        let time_str = parts[3].replace(",", "")
        let nanos = time_str.parse::<f64>().ok()?
        Some((name, nanos / 1_000_000.0)) // Convert to ms
    } else {
        None
    }
}

// Execute validation
match validate_performance() {
    Ok(()) => exit(0),
    Err(msg) => {
        eprintln!("✗ Validation failed:\n{}", msg)
        exit(1)
    }
}
```

## Test Execution Matrix

| Test Category | Command | Coverage Focus | Validation Type |
|--------------|---------|----------------|-----------------|
| CLI Arguments | `make test-cli` | Command parsing, flag handling | Integration |
| REPL Interaction | `make test-repl` | Interactive evaluation, state | Integration |
| One-liners | `make test-oneliner` | Pipeline processing, filters | End-to-end |
| Properties | `make test-properties` | Invariants, roundtrips | Property-based |
| Performance | `make bench-execution` | Latency, throughput | Benchmark |
| Fuzzing | `make fuzz-parser` | Parser robustness | Fuzz testing |

## Acceptance Criteria

1. **Correctness**: All semantic tests pass
2. **Performance**: Meets defined latency/size targets
3. **Robustness**: No crashes on fuzz inputs
4. **Coverage**: >80% line coverage on execution paths
5. **Determinism**: Property tests validate invariants

## Running the Test Suite

```bash
# Quick validation
make test-execution

# Full validation with benchmarks
make test-execution bench-execution

# CI/CD pipeline
make test-execution coverage-execution bench-execution
```