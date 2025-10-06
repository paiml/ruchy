# Ruchy Fuzz Testing Specification

**Version**: 1.0.0
**Status**: MANDATORY for all LANG-COMP tickets
**Tool**: `ruchy fuzz`

## 1. Overview

Fuzz testing validates code robustness by feeding millions of randomly-generated or mutated inputs to find crashes, panics, infinite loops, and undefined behavior that traditional tests miss.

**Requirement**: ALL LANG-COMP tickets MUST include fuzz targets with ≥1M iterations and zero crashes/panics.

## 2. Command Specification

### 2.1 Basic Usage

```bash
# Run fuzz tests on parser
ruchy fuzz parser --iterations 1000000

# Run all fuzz targets
ruchy fuzz --all

# Run specific fuzz target
ruchy fuzz targets/parse_expression.rs

# Generate fuzz report
ruchy fuzz parser --format json --output fuzz-report.json
```

### 2.2 Command Options

| Option | Description | Default |
|--------|-------------|---------|
| `--iterations <N>` | Number of fuzz iterations | 1000000 |
| `--timeout <SEC>` | Timeout per iteration (ms) | 1000 |
| `--format <FMT>` | Output format (text, json, markdown, sarif) | text |
| `--output <FILE>` | Write report to file | stdout |
| `--corpus <DIR>` | Corpus directory for inputs | `fuzz/corpus/` |
| `--jobs <N>` | Parallel fuzz jobs | CPU count |
| `--dict <FILE>` | Dictionary file for mutations | none |
| `--max-len <N>` | Maximum input length (bytes) | 4096 |
| `--seed <N>` | Random seed for reproducibility | random |

### 2.3 Exit Codes

| Code | Meaning |
|------|---------|
| 0 | No crashes/panics found (≥1M iterations) |
| 1 | Crashes or panics discovered |
| 2 | Timeout occurred |
| 3 | Configuration error |
| 4 | Fuzz execution error |

## 3. Fuzz Targets

### 3.1 Core Fuzz Targets

Every LANG-COMP ticket must include fuzz targets for:

#### Target 1: Parser Fuzzing
```rust
// fuzz/fuzz_targets/parse_expression.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use ruchy::frontend::parser::parse_expression;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Should never panic, even on invalid input
        let _ = parse_expression(s);
    }
});
```

#### Target 2: Evaluation Fuzzing
```rust
// fuzz/fuzz_targets/eval_code.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use ruchy::runtime::eval;

fuzz_target!(|data: &[u8]| {
    if let Ok(code) = std::str::from_utf8(data) {
        // Should handle any input gracefully
        let _ = eval(code);
    }
});
```

#### Target 3: Type Inference Fuzzing
```rust
// fuzz/fuzz_targets/infer_types.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use ruchy::typechecker::infer;

fuzz_target!(|data: &[u8]| {
    if let Ok(code) = std::str::from_utf8(data) {
        if let Ok(ast) = parse(code) {
            // Type inference should never panic
            let _ = infer(&ast);
        }
    }
});
```

### 3.2 Structured Fuzzing

Use `arbitrary` crate for structured input generation:

```rust
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    variables: Vec<String>,
    operations: Vec<Operation>,
    literals: Vec<i64>,
}

#[derive(Arbitrary, Debug)]
enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

fuzz_target!(|input: FuzzInput| {
    let code = generate_code(&input);
    let _ = eval(&code);
});
```

## 4. Output Format

### 4.1 Text Format (Default)

```
Fuzz Test Report
================

Target: parse_expression
Iterations: 1,000,000
Duration: 3m 42s
Executions/sec: 4,500

Results:
  ✅ No crashes found
  ✅ No panics found
  ✅ No timeouts
  ✅ No undefined behavior

Coverage:
  Lines: 1,234 / 1,500 (82.3%)
  Functions: 45 / 50 (90.0%)
  Branches: 234 / 300 (78.0%)

Corpus:
  Initial: 100 inputs
  Final: 1,523 inputs (+1,423 interesting)
  Total size: 2.3 MB

Longest Input: 3,892 bytes
Slowest Input: 892 ms

Status: ✅ PASSED (1M iterations, 0 crashes)
```

### 4.2 JSON Format

```json
{
  "target": "parse_expression",
  "iterations": 1000000,
  "duration_seconds": 222,
  "executions_per_second": 4500,
  "results": {
    "crashes": 0,
    "panics": 0,
    "timeouts": 0,
    "undefined_behavior": 0
  },
  "coverage": {
    "lines": 0.823,
    "functions": 0.900,
    "branches": 0.780
  },
  "corpus": {
    "initial": 100,
    "final": 1523,
    "interesting": 1423,
    "size_bytes": 2400000
  },
  "longest_input_bytes": 3892,
  "slowest_input_ms": 892,
  "status": "passed"
}
```

## 5. Integration with cargo-fuzz

The `ruchy fuzz` command wraps `cargo fuzz`:

```bash
# Under the hood, ruchy fuzz runs:
cargo fuzz run parse_expression \
  -- -runs=1000000 \
     -timeout=1000 \
     -max_len=4096
```

## 6. Quality Gates

### 6.1 Pre-commit Hook Integration

```bash
#!/bin/bash
# Run quick fuzz smoke test before allowing commit

echo "Running fuzz smoke test (10K iterations)..."
ruchy fuzz parser --iterations 10000 || exit 1
ruchy fuzz eval --iterations 10000 || exit 1

echo "✅ Fuzz smoke test passed"
```

### 6.2 CI/CD Integration

```yaml
# .github/workflows/fuzz-tests.yml
name: Fuzz Tests

on:
  schedule:
    - cron: '0 2 * * *'  # Run nightly at 2 AM
  workflow_dispatch:

jobs:
  fuzz-tests:
    runs-on: ubuntu-latest
    timeout-minutes: 120
    steps:
      - uses: actions/checkout@v2
      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz
      - name: Run fuzz tests
        run: |
          ruchy fuzz --all \
            --iterations 10000000 \
            --format sarif \
            --output fuzz-tests.sarif
      - name: Upload results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: fuzz-tests.sarif
```

## 7. Implementation Strategy (EXTREME TDD)

### 7.1 Phase 1: CLI Wrapper (RED→GREEN)

**RED Phase**: Create failing test
```rust
#[test]
fn test_fuzz_command_runs() {
    let output = Command::new("ruchy")
        .args(["fuzz", "parser", "--iterations", "1000"])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Fuzz Test Report"));
}
```

**GREEN Phase**: Implement minimal command
```rust
// src/bin/commands/fuzz.rs
pub fn run_fuzz(target: &str, iterations: usize) -> Result<()> {
    let output = Command::new("cargo")
        .args([
            "fuzz", "run", target,
            "--",
            &format!("-runs={}", iterations),
            "-timeout=1000"
        ])
        .output()?;

    if output.status.success() {
        println!("✅ Fuzz test passed: {} iterations, 0 crashes", iterations);
        Ok(())
    } else {
        eprintln!("❌ Fuzz test failed: crashes or panics found");
        Err(Error::FuzzFailure)
    }
}
```

### 7.2 Phase 2: Coverage Tracking

Parse cargo-fuzz output to extract coverage metrics.

### 7.3 Phase 3: Corpus Management

Automatically save interesting inputs to corpus for regression testing.

## 8. Crash Reproduction

### 8.1 Crash Report Example

```
❌ FUZZ FAILURE

Target: parse_expression
Iteration: 42,853

Crash Type: PANIC
Message: "attempt to subtract with overflow"
Location: src/frontend/parser.rs:234

Input (hex):
  ff ff ff ff 2b 31

Input (ascii):
  ÿÿÿÿ+1

Reproducer:
  echo -n "ÿÿÿÿ+1" | cargo fuzz run parse_expression

Stack Trace:
  #0 parser::parse_expression (parser.rs:234)
  #1 parser::parse (parser.rs:45)
  #2 fuzz_target (parse_expression.rs:8)

Saved to: fuzz/artifacts/parse_expression/crash-42853
```

### 8.2 Crash Regression Test

Automatically convert crashes to regression tests:

```rust
#[test]
fn test_fuzz_crash_42853() {
    // Regression test for fuzz crash #42853
    let input = b"\xff\xff\xff\xff+1";
    let code = std::str::from_utf8(input).unwrap();

    // Should not panic
    let result = parse_expression(code);
    assert!(result.is_err(), "Invalid input should return error, not panic");
}
```

## 9. Dictionary-Based Fuzzing

### 9.1 Custom Dictionary

```
# fuzz/dictionaries/ruchy_keywords.dict
"let"
"fn"
"if"
"else"
"match"
"true"
"false"
"=>"
"|>"
```

### 9.2 Usage

```bash
ruchy fuzz parser \
  --dict fuzz/dictionaries/ruchy_keywords.dict \
  --iterations 1000000
```

## 10. LANG-COMP Integration

Every LANG-COMP ticket MUST include:

1. **Fuzz Targets**: Minimum 2 targets (parser + eval)
2. **≥1M Iterations**: Per target (or continuous fuzzing)
3. **Zero Crashes**: No panics, crashes, or undefined behavior
4. **Coverage Report**: Line/branch coverage from fuzzing
5. **Corpus**: Saved interesting inputs for regression

Example documentation section:

```markdown
**Fuzz Testing**:
- Targets: parse_expression, eval_code
- Iterations: 1,000,000 per target (total: 2M)
- Crashes: 0
- Panics: 0
- Coverage: 82.3% lines, 78.0% branches
- Corpus: 1,523 interesting inputs saved
- Status: ✅ PASSED

**Interesting Findings**:
1. Parser handles deeply nested expressions (depth: 500)
2. Eval handles large integers (up to i64::MAX)
3. UTF-8 edge cases handled correctly
```

## 11. Tooling Requirements

### 11.1 Dependencies

- `cargo-fuzz` - Fuzz testing tool
- `libfuzzer-sys` - LibFuzzer bindings
- `arbitrary` - Structured fuzzing
- `serde_json` - JSON report generation

### 11.2 Installation

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Initialize fuzz targets
cargo fuzz init

# Verify installation
cargo fuzz list
```

### 11.3 PMAT Integration

```bash
# Combine fuzz tests with PMAT quality gates
ruchy fuzz --all --format json | pmat quality-gate --check fuzz-coverage
```

## 12. Success Criteria

A fuzz test suite is considered successful when:

1. **≥1M iterations per target** without crashes
2. **Zero panics or crashes** found
3. **≥75% code coverage** from fuzzing
4. **Corpus saved** for regression testing
5. **Fast execution** (<10 minutes for 1M iterations)
6. **CI integration** - Runs nightly

## 13. Best Practices

### 13.1 Writing Fuzz-Resistant Code

**Bad** (panic on invalid input):
```rust
pub fn parse(code: &str) -> Ast {
    let tokens = tokenize(code).unwrap(); // PANIC!
    build_ast(tokens)
}
```

**Good** (return error on invalid input):
```rust
pub fn parse(code: &str) -> Result<Ast, ParseError> {
    let tokens = tokenize(code)?; // Returns error
    build_ast(tokens)
}
```

### 13.2 Timeouts and Resource Limits

```rust
#[no_mangle]
pub extern "C" fn LLVMFuzzerTestOneInput(data: *const u8, size: usize) -> i32 {
    let data = unsafe { std::slice::from_raw_parts(data, size) };

    // Limit input size to prevent OOM
    if size > 10_000 {
        return 0;
    }

    if let Ok(s) = std::str::from_utf8(data) {
        // Set timeout to prevent infinite loops
        let _ = std::panic::catch_unwind(|| {
            let _ = parse(s);
        });
    }

    0
}
```

### 13.3 Combining with Property Tests

```rust
// Use fuzz findings as property test inputs
proptest! {
    #[test]
    fn prop_parser_never_panics(code in any::<String>()) {
        // Fuzz findings show these inputs are interesting
        let _ = parse(&code); // Should not panic
    }
}
```

## 14. Performance Optimization

### 14.1 Parallel Fuzzing

```bash
# Run 8 parallel fuzz jobs
ruchy fuzz parser --iterations 10000000 --jobs 8
```

### 14.2 Coverage-Guided Fuzzing

LibFuzzer automatically focuses on inputs that increase coverage.

### 14.3 Corpus Minimization

```bash
# Minimize corpus to smallest set with same coverage
cargo fuzz cmin parse_expression
```

## 15. Continuous Fuzzing

### 15.1 OSS-Fuzz Integration

Submit Ruchy to Google's OSS-Fuzz for continuous fuzzing at scale.

### 15.2 Local Continuous Fuzzing

```bash
# Run fuzz tests indefinitely
ruchy fuzz --all --iterations infinite
```

## 16. Future Enhancements

- **Structure-aware fuzzing** - Generate valid Ruchy programs
- **Differential fuzzing** - Compare Ruchy vs Rust output
- **Symbolic execution** - Combine fuzzing with SMT solving
- **Grammar-based fuzzing** - Use Ruchy grammar to generate inputs
- **Fuzzing feedback loop** - Use fuzz findings to improve test generation
