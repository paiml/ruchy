# Ruchy Mutation Testing Specification

**Version**: 1.0.0
**Status**: MANDATORY for all LANG-COMP tickets
**Tool**: `ruchy mutations`

## 1. Overview

Mutation testing validates test suite quality by introducing deliberate bugs (mutations) into source code and verifying that tests catch them. A high-quality test suite kills all mutations.

**Requirement**: ALL LANG-COMP tickets MUST achieve ≥75% mutation coverage (target: 95%+).

## 2. Command Specification

### 2.1 Basic Usage

```bash
# Run mutation tests on specific module
ruchy mutations tests/lang_comp/basic_syntax/variables_test.rs

# Run on entire test suite
ruchy mutations tests/lang_comp/

# Generate mutation report
ruchy mutations . --format json --output mutation-report.json

# Run with timeout
ruchy mutations src/frontend/parser/ --timeout 600
```

### 2.2 Command Options

| Option | Description | Default |
|--------|-------------|---------|
| `--timeout <SEC>` | Timeout per mutation (seconds) | 300 |
| `--format <FMT>` | Output format (text, json, markdown, sarif) | text |
| `--output <FILE>` | Write report to file | stdout |
| `--min-coverage <PCT>` | Minimum mutation coverage (0.0-1.0) | 0.75 |
| `--exclude <PATTERN>` | Exclude files matching pattern | none |
| `--jobs <N>` | Parallel jobs | CPU count |
| `--fail-fast` | Stop on first surviving mutant | false |
| `--show-diffs` | Show code diffs for each mutation | false |

### 2.3 Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Mutation coverage ≥ threshold |
| 1 | Mutation coverage < threshold |
| 2 | Timeout occurred |
| 3 | Configuration error |
| 4 | Mutation execution error |

## 3. Mutation Operators

### 3.1 Arithmetic Operators

| Original | Mutant | Description |
|----------|--------|-------------|
| `+` | `-` | Addition to subtraction |
| `-` | `+` | Subtraction to addition |
| `*` | `/` | Multiplication to division |
| `/` | `*` | Division to multiplication |
| `%` | `*` | Modulo to multiplication |

### 3.2 Comparison Operators

| Original | Mutant | Description |
|----------|--------|-------------|
| `==` | `!=` | Equality to inequality |
| `!=` | `==` | Inequality to equality |
| `<` | `<=` | Less than to less or equal |
| `<=` | `<` | Less or equal to less than |
| `>` | `>=` | Greater than to greater or equal |
| `>=` | `>` | Greater or equal to greater than |

### 3.3 Logical Operators

| Original | Mutant | Description |
|----------|--------|-------------|
| `&&` | `\|\|` | AND to OR |
| `\|\|` | `&&` | OR to AND |
| `!` | (remove) | NOT removal |

### 3.4 Literal Mutations

| Original | Mutant | Description |
|----------|--------|-------------|
| `true` | `false` | Boolean flip |
| `false` | `true` | Boolean flip |
| `0` | `1` | Zero to one |
| `1` | `0` | One to zero |
| `n` | `n + 1` | Integer increment |
| `n` | `n - 1` | Integer decrement |

### 3.5 Statement Mutations

| Original | Mutant | Description |
|----------|--------|-------------|
| `return x` | `return !x` | Return value negation |
| `if (cond)` | `if (!cond)` | Condition negation |
| `x = y` | `x = !y` | Assignment negation |

## 4. Output Format

### 4.1 Text Format (Default)

```
Mutation Test Report
====================

File: tests/lang_comp/basic_syntax/variables_test.rs
Module: Basic Syntax - Variables

Mutations Found: 45
Mutations Killed: 43 (95.6%)
Mutations Survived: 2 (4.4%)
Mutations Timeout: 0 (0%)
Mutations Unviable: 0 (0%)

🎯 CAUGHT (95.6%):
  ✅ src/frontend/parser.rs:142 - Changed + to - (caught by test_arithmetic)
  ✅ src/frontend/parser.rs:156 - Changed == to != (caught by test_comparison)
  ... (41 more)

❌ MISSED (4.4%):
  ❌ src/frontend/parser.rs:203 - Changed && to || (SURVIVED)
     No test caught this mutation
     Suggestion: Add test for logical operator combinations

  ❌ src/runtime/eval.rs:89 - Changed > to >= (SURVIVED)
     No test caught this mutation
     Suggestion: Add boundary condition test

Summary:
  Mutation Coverage: 95.6%
  Target: ≥75.0%
  Status: ✅ PASSED (exceeded target by 20.6%)
```

### 4.2 JSON Format

```json
{
  "file": "tests/lang_comp/basic_syntax/variables_test.rs",
  "module": "Basic Syntax - Variables",
  "mutations": {
    "found": 45,
    "killed": 43,
    "survived": 2,
    "timeout": 0,
    "unviable": 0
  },
  "coverage": 0.956,
  "target": 0.75,
  "status": "passed",
  "caught": [
    {
      "location": "src/frontend/parser.rs:142",
      "operator": "arithmetic",
      "mutation": "+ -> -",
      "killed_by": "test_arithmetic",
      "status": "caught"
    }
  ],
  "survived": [
    {
      "location": "src/frontend/parser.rs:203",
      "operator": "logical",
      "mutation": "&& -> ||",
      "status": "survived",
      "suggestion": "Add test for logical operator combinations"
    }
  ]
}
```

## 5. Integration with cargo-mutants

The `ruchy mutations` command wraps `cargo-mutants`:

```bash
# Under the hood, ruchy mutations runs:
cargo mutants \
  --file "tests/lang_comp/basic_syntax/variables_test.rs" \
  --timeout 300 \
  --no-times
```

## 6. Quality Gates

### 6.1 Pre-commit Hook Integration

```bash
#!/bin/bash
# Validate mutation coverage before allowing commit

changed_files=$(git diff --cached --name-only | grep "src/.*\.rs$\|tests/.*\.rs$")

for file in $changed_files; do
    echo "Running mutation tests: $file"
    ruchy mutations "$file" --min-coverage 0.75 || exit 1
done

echo "✅ All mutation tests passed (≥75% coverage)"
```

### 6.2 CI/CD Integration

```yaml
# .github/workflows/mutation-tests.yml
name: Mutation Tests

on: [push, pull_request]

jobs:
  mutation-tests:
    runs-on: ubuntu-latest
    timeout-minutes: 60
    steps:
      - uses: actions/checkout@v2
      - name: Run mutation tests
        run: |
          ruchy mutations tests/lang_comp/ \
            --min-coverage 0.75 \
            --timeout 600 \
            --format sarif \
            --output mutation-tests.sarif
      - name: Upload results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: mutation-tests.sarif
```

## 7. Implementation Strategy (EXTREME TDD)

### 7.1 Phase 1: CLI Wrapper (RED→GREEN)

**RED Phase**: Create failing test
```rust
#[test]
fn test_mutations_command_runs() {
    let output = Command::new("ruchy")
        .args(["mutations", "tests/lang_comp/basic_syntax/"])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Mutation Test Report"));
}
```

**GREEN Phase**: Implement minimal command
```rust
// src/bin/commands/mutations.rs
pub fn run_mutations(path: &str, timeout: u32, min_coverage: f64) -> Result<()> {
    let output = Command::new("cargo")
        .args([
            "mutants",
            "--file", path,
            "--timeout", &timeout.to_string(),
            "--no-times"
        ])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let coverage = parse_mutation_coverage(&stdout)?;

    if coverage >= min_coverage {
        println!("✅ Mutation coverage: {:.1}%", coverage * 100.0);
        Ok(())
    } else {
        eprintln!("❌ Mutation coverage: {:.1}% (target: {:.1}%)",
                  coverage * 100.0, min_coverage * 100.0);
        Err(Error::InsufficientMutationCoverage)
    }
}
```

### 7.2 Phase 2: Report Parsing

Parse cargo-mutants output and generate structured reports:

```rust
fn parse_mutation_coverage(output: &str) -> Result<f64> {
    // Parse output like:
    // "45 mutants tested: 43 caught, 2 missed, 0 timeout, 0 unviable"
    let re = Regex::new(r"(\d+) mutants tested: (\d+) caught")?;
    if let Some(caps) = re.captures(output) {
        let total: f64 = caps[1].parse()?;
        let caught: f64 = caps[2].parse()?;
        Ok(caught / total)
    } else {
        Err(Error::ParseError)
    }
}
```

### 7.3 Phase 3: Enhanced Reporting

Generate detailed reports with suggestions for missing tests.

## 8. Mutation Coverage Thresholds

### 8.1 LANG-COMP Requirements

| Component | Minimum | Target | Notes |
|-----------|---------|--------|-------|
| Parser | 75% | 95% | Critical path |
| Runtime | 75% | 90% | Core evaluation |
| Linter | 75% | 85% | Quality tools |
| Type Inference | 75% | 90% | Type safety |
| LANG-COMP Examples | 75% | 99% | Feature validation |

### 8.2 Phased Rollout

**Phase 1** (Current): 75% minimum, warnings for <95%
**Phase 2** (Sprint 10): 85% minimum, warnings for <95%
**Phase 3** (Sprint 11): 95% minimum, hard requirement

## 9. Surviving Mutant Analysis

### 9.1 Common Causes

1. **Equivalent Mutants**: Mutation doesn't change behavior
   - Example: `x + 0` → `x - 0` (equivalent for zero)

2. **Missing Test Cases**: Tests don't cover mutation
   - Solution: Add property tests or edge case tests

3. **Weak Assertions**: Tests don't check specific values
   - Solution: Use precise assertions (`assert_eq!` vs `assert!`)

### 9.2 Investigation Workflow

```bash
# Find surviving mutants
ruchy mutations src/module.rs --show-diffs > mutations.txt

# Analyze each surviving mutant
grep "SURVIVED" mutations.txt

# Add tests to kill mutants
# Re-run mutation tests
ruchy mutations src/module.rs --min-coverage 0.95
```

## 10. LANG-COMP Integration

Every LANG-COMP ticket MUST include:

1. **Mutation Test Run**: `ruchy mutations tests/lang_comp/<feature>/`
2. **≥75% Coverage**: Minimum mutation coverage threshold
3. **Investigation**: Document any surviving mutants and justification
4. **Test Improvements**: Add tests to kill surviving mutants
5. **Validation**: Report mutation coverage in documentation

Example documentation section:

```markdown
**Mutation Testing**:
- Mutations Found: 45
- Mutations Killed: 43 (95.6%)
- Mutations Survived: 2 (4.4%)
- Coverage: 95.6% (target: ≥75%)
- Status: ✅ PASSED

**Surviving Mutants**:
1. `src/parser.rs:203` - `&& -> ||` - Equivalent mutant (short-circuit)
2. `src/eval.rs:89` - `> -> >=` - Boundary condition added to test suite
```

## 11. Tooling Requirements

### 11.1 Dependencies

- `cargo-mutants` - Mutation testing tool
- `cargo test` - Test runner
- `serde_json` - JSON report generation
- `regex` - Output parsing

### 11.2 Installation

```bash
# Install cargo-mutants
cargo install cargo-mutants

# Verify installation
cargo mutants --version
```

### 11.3 PMAT Integration

```bash
# Combine mutation tests with PMAT quality gates
ruchy mutations . --format json | pmat quality-gate --check mutation-coverage
```

## 12. Success Criteria

A mutation test suite is considered successful when:

1. **≥75% mutations killed** (minimum threshold)
2. **≥95% mutations killed** (target for LANG-COMP)
3. **All surviving mutants documented** with justification
4. **Test improvements added** to kill non-equivalent mutants
5. **Fast execution** (<10 minutes for full suite)
6. **CI integration** - Runs on every PR

## 13. Best Practices

### 13.1 Writing Mutation-Resistant Tests

**Bad** (weak assertion):
```rust
#[test]
fn test_addition() {
    let result = eval("2 + 2");
    assert!(result.is_ok()); // Doesn't check value!
}
```

**Good** (strong assertion):
```rust
#[test]
fn test_addition() {
    let result = eval("2 + 2");
    assert_eq!(result.unwrap(), Value::Integer(4)); // Exact value check
}
```

### 13.2 Testing Edge Cases

```rust
#[test]
fn test_boundary_conditions() {
    assert_eq!(eval("i64::MIN + 1"), Ok(Value::Integer(i64::MIN + 1)));
    assert_eq!(eval("i64::MAX - 1"), Ok(Value::Integer(i64::MAX - 1)));
    assert_eq!(eval("0 - 1"), Ok(Value::Integer(-1)));
}
```

### 13.3 Combining with Property Tests

```rust
proptest! {
    #[test]
    fn prop_arithmetic_correct(a in -100i64..100, b in -100i64..100) {
        let result = eval(&format!("{} + {}", a, b))?;
        prop_assert_eq!(result, Value::Integer(a + b)); // Kills many mutants
    }
}
```

## 14. Performance Optimization

### 14.1 Parallel Execution

```bash
# Run mutations in parallel (default: CPU count)
ruchy mutations . --jobs 8
```

### 14.2 Incremental Mutation Testing

```bash
# Only test changed files
ruchy mutations --changed-files --since HEAD~1
```

### 14.3 Mutation Caching

Cache mutation results to avoid re-testing unchanged code.

## 15. Future Enhancements

- **Semantic mutations** - Higher-level code transformations
- **Differential mutations** - Compare against reference implementation
- **Mutation prioritization** - Focus on high-value mutations first
- **ML-guided mutations** - Learn which mutations are most valuable
- **Cross-language mutations** - Ruchy ↔ Rust mutation testing
