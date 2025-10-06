# Ruchy Property Testing Specification

**Version**: 1.0.0
**Status**: MANDATORY for all LANG-COMP tickets
**Tool**: `ruchy property-tests`

## 1. Overview

Property-based testing validates language features through automated generation of thousands of test cases, proving correctness via mathematical invariants rather than hand-written examples.

**Requirement**: ALL LANG-COMP tickets MUST include property tests with ≥10,000 test cases per feature.

## 2. Command Specification

### 2.1 Basic Usage

```bash
# Run property tests for a specific feature
ruchy property-tests examples/lang_comp/01-basic-syntax/

# Run with custom case count
ruchy property-tests examples/lang_comp/01-basic-syntax/ --cases 50000

# Run specific test file
ruchy property-tests tests/lang_comp/basic_syntax/variables_test.rs

# Generate property test report
ruchy property-tests . --format json --output property-report.json
```

### 2.2 Command Options

| Option | Description | Default |
|--------|-------------|---------|
| `--cases <N>` | Number of test cases per property | 10000 |
| `--format <FMT>` | Output format (text, json, markdown, sarif) | text |
| `--output <FILE>` | Write report to file | stdout |
| `--seed <N>` | Random seed for reproducibility | random |
| `--verbose` | Show each test case execution | false |
| `--fail-fast` | Stop on first failure | false |
| `--min-success-rate <PCT>` | Minimum success rate (0.0-1.0) | 1.0 |
| `--shrink-limit <N>` | Maximum shrinking iterations | 1000 |

### 2.3 Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All property tests passed |
| 1 | Property test failures found |
| 2 | Insufficient test coverage (<10K cases) |
| 3 | Configuration error |
| 4 | Test execution error |

## 3. Property Test Requirements

### 3.1 Minimum Coverage

Every LANG-COMP feature MUST have:
- **At least 3 property tests** covering different invariants
- **≥10,000 test cases per property** (configurable via `--cases`)
- **100% success rate** (all properties must hold)
- **Shrinking verification** (minimal failing cases found)

### 3.2 Property Categories

#### Category 1: Roundtrip Properties
Verify that transformations are reversible:

```rust
proptest! {
    #[test]
    fn prop_parse_print_roundtrip(code in valid_ruchy_program()) {
        let ast = parse(&code)?;
        let printed = print_ast(&ast);
        let ast2 = parse(&printed)?;
        prop_assert_eq!(ast, ast2, "AST roundtrip failed");
    }
}
```

#### Category 2: Invariant Properties
Verify that operations preserve invariants:

```rust
proptest! {
    #[test]
    fn prop_variable_bindings_preserve_values(
        name in "[a-z][a-z0-9_]{0,15}",
        value in -1000i64..1000i64
    ) {
        let code = format!("let {} = {}; {}", name, value, name);
        let result = run_repl_code(&code)?;
        prop_assert!(result.contains(&value.to_string()));
    }
}
```

#### Category 3: Oracle Properties
Compare against reference implementation:

```rust
proptest! {
    #[test]
    fn prop_arithmetic_matches_rust(a in -100i64..100, b in 1i64..100) {
        let ruchy_result = eval(&format!("{} + {}", a, b))?;
        let rust_result = a + b;
        prop_assert_eq!(ruchy_result, rust_result.into());
    }
}
```

#### Category 4: Metamorphic Properties
Verify relationships between different inputs:

```rust
proptest! {
    #[test]
    fn prop_addition_commutative(a in -100i64..100, b in -100i64..100) {
        let result1 = eval(&format!("{} + {}", a, b))?;
        let result2 = eval(&format!("{} + {}", b, a))?;
        prop_assert_eq!(result1, result2, "Addition not commutative");
    }
}
```

## 4. Output Format

### 4.1 Text Format (Default)

```
Property Test Report
====================

Feature: Basic Syntax - Variables
Location: tests/lang_comp/basic_syntax/variables_test.rs

✅ prop_variable_names_valid
   Cases: 10,000
   Passed: 10,000 (100%)
   Failed: 0
   Time: 2.3s

✅ prop_integer_literals
   Cases: 2,000
   Passed: 2,000 (100%)
   Failed: 0
   Shrinks: 0
   Time: 0.8s

Summary:
  Total Properties: 5
  Total Cases: 50,000
  Passed: 50,000 (100%)
  Failed: 0
  Total Time: 8.4s

Status: ✅ ALL PROPERTIES HOLD
```

### 4.2 JSON Format

```json
{
  "feature": "Basic Syntax - Variables",
  "location": "tests/lang_comp/basic_syntax/variables_test.rs",
  "properties": [
    {
      "name": "prop_variable_names_valid",
      "cases": 10000,
      "passed": 10000,
      "failed": 0,
      "success_rate": 1.0,
      "shrinks": 0,
      "time_ms": 2300,
      "status": "passed"
    }
  ],
  "summary": {
    "total_properties": 5,
    "total_cases": 50000,
    "passed": 50000,
    "failed": 0,
    "success_rate": 1.0,
    "total_time_ms": 8400
  },
  "status": "passed"
}
```

## 5. Integration with cargo test

The `ruchy property-tests` command wraps `cargo test` with proptest:

```bash
# Under the hood, ruchy property-tests runs:
PROPTEST_CASES=10000 cargo test --test lang_comp_suite -- --nocapture
```

## 6. Quality Gates

### 6.1 Pre-commit Hook Integration

```bash
#!/bin/bash
# Validate property tests before allowing commit

changed_tests=$(git diff --cached --name-only | grep "tests/lang_comp/.*_test.rs$")

for test in $changed_tests; do
    echo "Running property tests: $test"
    ruchy property-tests "$test" --cases 10000 || exit 1
done

echo "✅ All property tests passed"
```

### 6.2 CI/CD Integration

```yaml
# .github/workflows/property-tests.yml
name: Property Tests

on: [push, pull_request]

jobs:
  property-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run property tests
        run: |
          ruchy property-tests tests/lang_comp/ \
            --cases 50000 \
            --format sarif \
            --output property-tests.sarif
      - name: Upload results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: property-tests.sarif
```

## 7. Implementation Strategy

### 7.1 Phase 1: CLI Wrapper (EXTREME TDD)

**RED Phase**: Create failing test
```rust
#[test]
fn test_property_tests_command_runs() {
    let output = Command::new("ruchy")
        .args(["property-tests", "tests/lang_comp/basic_syntax/"])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Property Test Report"));
}
```

**GREEN Phase**: Implement minimal command
```rust
// src/bin/commands/property_tests.rs
pub fn run_property_tests(path: &str, cases: usize) -> Result<()> {
    let output = Command::new("cargo")
        .args(["test", "--test", "lang_comp_suite", "--", "--nocapture"])
        .env("PROPTEST_CASES", cases.to_string())
        .output()?;

    if output.status.success() {
        println!("✅ All property tests passed");
        Ok(())
    } else {
        Err(Error::PropertyTestFailure)
    }
}
```

### 7.2 Phase 2: Report Generation

Parse cargo test output and generate structured reports in multiple formats.

### 7.3 Phase 3: Coverage Tracking

Track which properties have been tested and ensure ≥10K cases per property.

## 8. Error Handling

### 8.1 Property Failure Example

```
❌ PROPERTY FAILURE

Property: prop_integer_literals
Location: tests/lang_comp/basic_syntax/variables_test.rs:98

Failed case (shrunk):
  Input: n = -5
  Expected: Output contains "-5"
  Actual: Output was empty

Shrinking steps: 1000 -> 100 -> 10 -> 5 -> -5

Reproduce with:
  PROPTEST_CASES=1 cargo test prop_integer_literals -- --exact

Status: ❌ PROPERTY VIOLATED
```

## 9. Best Practices

### 9.1 Property Design

1. **Test Invariants, Not Implementations**
   - ✅ Good: "Parsing and printing should roundtrip"
   - ❌ Bad: "Parser should create 5 AST nodes"

2. **Use Domain-Specific Generators**
   ```rust
   fn valid_identifier() -> impl Strategy<Value = String> {
       "[a-z][a-z0-9_]{0,15}"
   }
   ```

3. **Combine Multiple Properties**
   - Roundtrip + Invariant + Oracle = High Confidence

4. **Test Edge Cases Explicitly**
   ```rust
   proptest! {
       #[test]
       fn prop_with_edge_cases(
           n in prop_oneof![
               Just(i64::MIN),
               Just(i64::MAX),
               Just(0),
               -1000i64..1000i64,
           ]
       ) {
           // Property test with edge cases included
       }
   }
   ```

## 10. LANG-COMP Integration

Every LANG-COMP ticket MUST include:

1. **Property Test File**: `tests/lang_comp/<feature>/<feature>_test.rs`
2. **Minimum 3 Properties**: Covering different invariant categories
3. **≥10,000 Cases**: Per property (configurable)
4. **Documentation**: Which properties are tested and why
5. **Validation**: `ruchy property-tests` must pass with 100% success rate

## 11. Tooling Requirements

### 11.1 Dependencies

- `proptest = "1.0"` - Property-based testing framework
- `cargo test` - Test runner
- `serde_json` - JSON report generation

### 11.2 PMAT Integration

```bash
# Combine property tests with PMAT quality gates
ruchy property-tests . --format json | pmat quality-gate --check property-coverage
```

## 12. Success Criteria

A property test suite is considered successful when:

1. **All properties pass** with 100% success rate
2. **≥10,000 cases** executed per property
3. **Shrinking works** (can find minimal failing cases if failure occurs)
4. **Coverage complete** (all feature invariants tested)
5. **Fast execution** (<10s for 50K total cases)
6. **Reproducible** (same seed produces same results)

## 13. Future Enhancements

- **Parallel execution** - Run properties concurrently
- **Custom generators** - Domain-specific value generation
- **Hypothesis integration** - Cross-language property testing
- **Mutation-based fuzzing** - Combine with mutation testing
- **Symbolic execution** - Prove properties via SMT solvers
