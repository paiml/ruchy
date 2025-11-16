# Mutation Testing Specification for Ruchy

**Version**: 1.0.0
**Status**: Draft
**Last Updated**: 2025-10-03
**Based On**: pforge mutation testing methodology

## Executive Summary

Mutation testing is a quality assurance technique that evaluates the effectiveness of our test suite by introducing small, deliberate bugs (mutations) into the code and checking if our tests catch them. This specification defines how Ruchy will implement mutation testing to achieve **90%+ mutation kill rate**, ensuring our tests are truly effective at catching bugs.

## Why Mutation Testing for Ruchy?

**Problem**: Traditional code coverage metrics (line, branch coverage) can give false confidence. You can have 100% line coverage but still have ineffective tests that don't actually verify behavior.

**Example of Ineffective Test**:
```rust
// Code
fn add(a: i32, b: i32) -> i32 {
    a + b  // Mutant: Change + to -
}

// Bad test (100% coverage, but doesn't verify correctness)
#[test]
fn test_add() {
    add(5, 3);  // No assertion!
}

// Good test (kills the mutant)
#[test]
fn test_add() {
    assert_eq!(add(5, 3), 8);  // Would fail if + changed to -
}
```

**For Ruchy**: As a compiler/interpreter, correctness is critical. Mutation testing ensures our parser, type checker, evaluator, and WASM backend tests are robust enough to catch subtle bugs.

## Goals

1. **Achieve 90%+ mutation kill rate** across all critical modules
2. **Identify weak tests** that pass even when code is broken
3. **Prevent regressions** by ensuring tests verify actual behavior
4. **Complement PMAT quality** with behavior-level validation

## Tool: cargo-mutants

We use `cargo-mutants` (https://mutants.rs/) for mutation testing in Rust.

**Installation**:
```bash
cargo install cargo-mutants
```

**Why cargo-mutants**:
- ✅ Rust-native, understands Cargo workspaces
- ✅ Fast: Caches test results, runs in parallel
- ✅ Smart mutations: Knows Rust semantics
- ✅ JSON output for CI/CD integration
- ✅ Active development, well-maintained

## Mutation Categories

### 1. Arithmetic Operators (High Priority for Ruchy)

**Mutations**:
- `a + b` → `a - b`, `a * b`, `a / b`
- `a - b` → `a + b`, `a * b`, `a / b`
- `a * b` → `a + b`, `a - b`, `a / b`
- `a / b` → `a + b`, `a - b`, `a * b`

**Ruchy Impact**: Critical for evaluator arithmetic, WASM code generation

**Example Test to Kill**:
```rust
#[test]
fn test_evaluator_addition_exact() {
    let result = eval("2 + 3").unwrap();
    assert_eq!(result, Value::Integer(5));  // Kills +→- mutation

    let result2 = eval("10 + 7").unwrap();
    assert_eq!(result2, Value::Integer(17));  // Different values ensure + not *
}
```

### 2. Boolean Operators (High Priority)

**Mutations**:
- `a && b` → `a || b`
- `a || b` → `a && b`
- `!condition` → `condition`

**Ruchy Impact**: Critical for control flow, pattern matching, type checking

**Example Test to Kill**:
```rust
#[test]
fn test_and_operator_not_or() {
    assert_eq!(eval("true && false"), Value::Bool(false));  // Kills &&→||
    assert_eq!(eval("false && false"), Value::Bool(false));
}

#[test]
fn test_or_operator_not_and() {
    assert_eq!(eval("true || false"), Value::Bool(true));  // Kills ||→&&
    assert_eq!(eval("false || false"), Value::Bool(false));
}
```

### 3. Comparison Operators (High Priority)

**Mutations**:
- `a < b` → `a <= b`, `a > b`, `a >= b`, `a == b`, `a != b`
- `a == b` → `a != b`
- `a != b` → `a == b`

**Ruchy Impact**: Critical for loop bounds, conditional evaluation

**Example Test to Kill**:
```rust
#[test]
fn test_less_than_exact() {
    assert_eq!(eval("5 < 10"), Value::Bool(true));   // true
    assert_eq!(eval("10 < 10"), Value::Bool(false)); // Kills <→<=
    assert_eq!(eval("15 < 10"), Value::Bool(false)); // Kills <→>
}
```

### 4. Return Value Replacements (Medium Priority)

**Mutations**:
- `Ok(value)` → `Ok(Default::default())`
- `Some(value)` → `Some(Default::default())`
- `function()` → `Default::default()`

**Ruchy Impact**: Critical for parser AST nodes, evaluator results

**Example Test to Kill**:
```rust
#[test]
fn test_parser_not_default() {
    let ast = parse("let x = 42").unwrap();

    // Verify it's NOT Default::default()
    assert!(!matches!(ast.kind, ExprKind::Nil));

    // Verify actual structure
    if let ExprKind::Let { name, value, .. } = &ast.kind {
        assert_eq!(name, "x");
        assert!(matches!(value.kind, ExprKind::Literal(Literal::Integer(42))));
    } else {
        panic!("Expected Let expression");
    }
}
```

### 5. Match Arm Deletions (High Priority)

**Mutations**:
- Delete individual match arms

**Ruchy Impact**: Critical for exhaustive pattern matching in parser/evaluator

**Example Test to Kill**:
```rust
#[test]
fn test_all_literal_types() {
    // Test each match arm
    assert_eq!(eval("42"), Value::Integer(42));        // Integer arm
    assert_eq!(eval("3.14"), Value::Float(3.14));      // Float arm
    assert_eq!(eval("true"), Value::Bool(true));       // Bool arm
    assert_eq!(eval("\"hi\""), Value::String("hi"));   // String arm
    assert_eq!(eval("nil"), Value::Nil);               // Nil arm
}
```

### 6. Empty Function Replacements (Low Priority)

**Mutations**:
- `fn foo() { ... }` → `fn foo() {}`
- `fn foo() -> Result<T> { ... }` → `fn foo() -> Result<T> { Ok(Default::default()) }`

**Ruchy Impact**: Medium - mostly affects utility functions

**Example Test to Kill**:
```rust
#[test]
fn test_function_actually_executes() {
    let mut called = false;

    execute_with_side_effect(|| {
        called = true;  // Verify function body runs
    });

    assert!(called, "Function should have executed");
}
```

## Priority Modules for Ruchy

### P0 - Critical (Target: 95%+ kill rate)

1. **Parser** (`src/frontend/parser/`)
   - **Why**: Bugs here corrupt the entire pipeline
   - **Focus**: Operator precedence, AST construction, error recovery
   - **Mutation types**: Arithmetic, comparison, match arms, return values

2. **Evaluator** (`src/runtime/interpreter.rs`, `src/runtime/eval_*.rs`)
   - **Why**: Correctness of execution
   - **Focus**: Arithmetic, boolean logic, pattern matching
   - **Mutation types**: All arithmetic/boolean operators, match arms

3. **Type System** (`src/frontend/type_checker.rs`)
   - **Why**: Type safety guarantees
   - **Focus**: Unification, constraint solving
   - **Mutation types**: Boolean logic, comparison operators

### P1 - High Priority (Target: 90%+ kill rate)

4. **WASM Backend** (`src/backend/wasm/`)
   - **Why**: Compilation correctness
   - **Focus**: Instruction generation, type conversions
   - **Mutation types**: Arithmetic, stack management

5. **REPL** (`src/runtime/repl/`)
   - **Why**: User-facing interface
   - **Focus**: Command parsing, state management
   - **Mutation types**: Boolean logic, return values

### P2 - Medium Priority (Target: 85%+ kill rate)

6. **Standard Library** (`src/runtime/eval_builtin.rs`)
   - **Why**: Language feature correctness
   - **Focus**: String ops, array ops, math functions
   - **Mutation types**: Arithmetic, comparison

7. **Error Handling** (`src/frontend/error.rs`)
   - **Why**: User experience
   - **Focus**: Error message construction
   - **Mutation types**: Return values, match arms

### P3 - Lower Priority (Target: 80%+ kill rate)

8. **CLI** (`src/bin/`, `src/cli/`)
   - **Why**: Less critical to core functionality
   - **Focus**: Argument parsing, file I/O
   - **Mutation types**: Empty functions, boolean logic

## Running Mutation Tests

### Basic Commands

```bash
# Run mutation tests on entire workspace
cargo mutants --workspace --output mutants.out

# Run on specific module (faster iteration)
cargo mutants --file src/frontend/parser/expressions.rs

# Run on specific function (debugging)
cargo mutants --file src/frontend/parser/expressions.rs --function parse_binary_expr

# Generate JSON report for analysis
cargo mutants --workspace --json --output mutants.json

# Check mutation kill rate (CI/CD)
cargo mutants --workspace --check --minimum-test-time 0.5
```

### Advanced Options

```bash
# Skip slow tests (focus on unit tests)
cargo mutants --workspace -- --lib

# Parallel execution (faster)
cargo mutants --workspace --jobs 8

# Exclude generated code
cargo mutants --exclude-dir target --exclude-dir tests.disabled

# Re-run only caught mutants (verify fixes)
cargo mutants --workspace --caught

# Re-run only missed mutants (focus on gaps)
cargo mutants --workspace --missed
```

### Makefile Integration

Add to `Makefile`:
```makefile
.PHONY: mutants mutants-quick mutants-check mutants-report

# Full mutation testing
mutants:
	cargo mutants --workspace --output mutants.out --json

# Quick mutation testing (lib only)
mutants-quick:
	cargo mutants --workspace -- --lib --output mutants-quick.out

# CI/CD check (fail if < 90%)
mutants-check:
	cargo mutants --workspace --check --minimum-test-time 0.5 || \
		(echo "❌ Mutation kill rate below 90%" && exit 1)

# Generate HTML report
mutants-report:
	cargo mutants --workspace --json --output mutants.json
	# TODO: Add report generator
```

## Test Writing Guidelines to Kill Mutants

### 1. Use Exact Assertions

❌ **Bad** (doesn't kill arithmetic mutants):
```rust
#[test]
fn test_addition() {
    let result = eval("2 + 3");
    assert!(result.is_ok());  // Passes even if + changed to *
}
```

✅ **Good** (kills arithmetic mutants):
```rust
#[test]
fn test_addition() {
    assert_eq!(eval("2 + 3"), Value::Integer(5));  // Fails if + → -
    assert_eq!(eval("10 + 7"), Value::Integer(17)); // Fails if + → *
}
```

### 2. Test All Match Arms

❌ **Bad** (doesn't kill match arm deletions):
```rust
#[test]
fn test_literals() {
    assert!(eval("42").is_ok());  // Only tests one arm
}
```

✅ **Good** (kills match arm deletions):
```rust
#[test]
fn test_all_literal_types() {
    assert_eq!(eval("42"), Value::Integer(42));
    assert_eq!(eval("3.14"), Value::Float(3.14));
    assert_eq!(eval("true"), Value::Bool(true));
    assert_eq!(eval("\"hi\""), Value::String("hi".into()));
    assert_eq!(eval("nil"), Value::Nil);
}
```

### 3. Test Both Branches

❌ **Bad** (doesn't kill boolean operator mutants):
```rust
#[test]
fn test_and() {
    assert_eq!(eval("true && true"), Value::Bool(true));  // Doesn't distinguish && from ||
}
```

✅ **Good** (kills boolean operator mutants):
```rust
#[test]
fn test_and_operator() {
    assert_eq!(eval("true && true"), Value::Bool(true));
    assert_eq!(eval("true && false"), Value::Bool(false));  // Kills &&→||
    assert_eq!(eval("false && true"), Value::Bool(false));
    assert_eq!(eval("false && false"), Value::Bool(false));
}
```

### 4. Test Boundary Conditions

❌ **Bad** (doesn't kill comparison mutants):
```rust
#[test]
fn test_less_than() {
    assert_eq!(eval("5 < 10"), Value::Bool(true));  // Doesn't test boundary
}
```

✅ **Good** (kills comparison mutants):
```rust
#[test]
fn test_less_than_exact() {
    assert_eq!(eval("5 < 10"), Value::Bool(true));   // Normal case
    assert_eq!(eval("10 < 10"), Value::Bool(false)); // Kills <→<=
    assert_eq!(eval("15 < 10"), Value::Bool(false)); // Kills <→>
}
```

### 5. Verify Non-Default Values

❌ **Bad** (doesn't kill Default::default() mutants):
```rust
#[test]
fn test_parser() {
    let ast = parse("let x = 42");
    assert!(ast.is_ok());  // Passes even if AST is Default
}
```

✅ **Good** (kills Default::default() mutants):
```rust
#[test]
fn test_parser_structure() {
    let ast = parse("let x = 42").unwrap();

    match &ast.kind {
        ExprKind::Let { name, value, .. } => {
            assert_eq!(name, "x");
            assert!(matches!(value.kind, ExprKind::Literal(Literal::Integer(42))));
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}
```

### 6. Test Side Effects

❌ **Bad** (doesn't kill empty function mutants):
```rust
#[test]
fn test_print() {
    eval("println(\"hello\")").unwrap();  // Doesn't verify output
}
```

✅ **Good** (kills empty function mutants):
```rust
#[test]
fn test_print_output() {
    let output = capture_stdout(|| {
        eval("println(\"hello\")").unwrap();
    });
    assert_eq!(output, "hello\n");  // Verifies println actually prints
}
```

## Integration with Quality Gates

### Pre-commit Hook

Add mutation testing to quality gates:

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Quick mutation test on changed files
changed_files=$(git diff --cached --name-only --diff-filter=ACM | grep "\.rs$")

if [ -n "$changed_files" ]; then
    for file in $changed_files; do
        echo "🧬 Running mutation tests on $file..."
        cargo mutants --file "$file" --check || {
            echo "❌ BLOCKED: Mutation tests failed for $file"
            echo "Run: cargo mutants --file $file --missed"
            exit 1
        }
    done
fi
```

### CI/CD Pipeline

Add to `.github/workflows/quality.yml`:

```yaml
mutation-testing:
  name: Mutation Testing
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - name: Install cargo-mutants
      run: cargo install cargo-mutants

    - name: Run mutation tests
      run: |
        cargo mutants --workspace --json --output mutants.json

    - name: Check mutation kill rate
      run: |
        KILL_RATE=$(jq '.summary.kill_rate' mutants.json)
        if (( $(echo "$KILL_RATE < 0.90" | bc -l) )); then
          echo "❌ Mutation kill rate $KILL_RATE below 90% threshold"
          exit 1
        fi
        echo "✅ Mutation kill rate: $KILL_RATE"

    - name: Upload mutation report
      uses: actions/upload-artifact@v4
      with:
        name: mutation-report
        path: mutants.json
```

## Roadmap to 90% Kill Rate

### Phase 1: Baseline (Week 1)

**Goal**: Establish baseline mutation kill rate

- [ ] Install cargo-mutants
- [ ] Run initial mutation tests on all modules
- [ ] Generate baseline report
- [ ] Categorize surviving mutants by priority
- [ ] Document baseline kill rate per module

**Deliverable**: `docs/execution/MUTATION_BASELINE_REPORT.md`

### Phase 2: Critical Modules (Week 2-3)

**Goal**: Achieve 95%+ kill rate on P0 modules

- [ ] Parser: Add arithmetic/comparison operator tests
- [ ] Parser: Add match arm coverage for all token types
- [ ] Evaluator: Add exact value tests for all operators
- [ ] Evaluator: Test all expression types
- [ ] Type Checker: Test all unification cases

**Target**: Parser (95%), Evaluator (95%), Type Checker (95%)

### Phase 3: High Priority Modules (Week 4)

**Goal**: Achieve 90%+ kill rate on P1 modules

- [ ] WASM: Test all instruction types
- [ ] WASM: Test type conversions
- [ ] REPL: Test all commands
- [ ] REPL: Test state management

**Target**: WASM (90%), REPL (90%)

### Phase 4: Medium Priority Modules (Week 5)

**Goal**: Achieve 85%+ kill rate on P2 modules

- [ ] Standard Library: Test all builtin functions
- [ ] Error Handling: Test all error types

**Target**: Stdlib (85%), Error (85%)

### Phase 5: CI/CD Integration (Week 6)

**Goal**: Prevent regressions

- [ ] Add mutation testing to pre-commit hooks
- [ ] Add mutation testing to CI/CD pipeline
- [ ] Set up mutation kill rate tracking
- [ ] Create mutation testing dashboard

**Target**: 90%+ overall kill rate, automated enforcement

## Metrics and Reporting

### Key Metrics

1. **Overall Kill Rate**: % of mutants caught by tests
   - **Target**: 90%+
   - **Formula**: (caught mutants / total mutants) * 100

2. **Module Kill Rate**: Per-module kill rates
   - **Target**: Varies by priority (P0: 95%, P1: 90%, P2: 85%)

3. **Mutation Category Kill Rate**: Kill rate by mutation type
   - **Target**: Arithmetic (95%), Boolean (95%), Comparison (95%)

4. **Test Efficiency**: Tests per mutant killed
   - **Target**: Minimize (reuse tests to kill multiple mutants)

### Report Format

```markdown
# Mutation Testing Report - YYYY-MM-DD

## Summary
- **Total Mutants**: 500
- **Caught**: 450
- **Missed**: 50
- **Kill Rate**: 90.0%

## By Module
| Module | Total | Caught | Missed | Kill Rate |
|--------|-------|--------|--------|-----------|
| Parser | 120 | 114 | 6 | 95.0% ✅ |
| Evaluator | 150 | 140 | 10 | 93.3% ✅ |
| Type Checker | 80 | 76 | 4 | 95.0% ✅ |
| WASM | 100 | 85 | 15 | 85.0% ⚠️ |
| REPL | 50 | 35 | 15 | 70.0% ❌ |

## By Mutation Type
| Type | Total | Caught | Missed | Kill Rate |
|------|-------|--------|--------|-----------|
| Arithmetic | 100 | 95 | 5 | 95.0% ✅ |
| Boolean | 80 | 76 | 4 | 95.0% ✅ |
| Comparison | 60 | 57 | 3 | 95.0% ✅ |
| Match Arms | 50 | 40 | 10 | 80.0% ⚠️ |
| Return Values | 100 | 70 | 30 | 70.0% ❌ |

## Action Items
1. REPL: Add command execution tests (15 mutants)
2. WASM: Add instruction tests (15 mutants)
3. All: Add Default::default() checks (30 mutants)
```

## Best Practices from pforge

Based on pforge's successful mutation testing:

1. **Target Critical Paths First**: Focus on schema generation, error handling, retry logic
2. **Use Exact Assertions**: `assert_eq!(actual, expected)` not `assert!(actual == expected)`
3. **Test Edge Cases**: Boundary conditions, empty inputs, all match arms
4. **Verify Behavior**: Test observable effects (logs, state changes, timeouts)
5. **Kill Arithmetic Mutants**: Test exact calculations, not "close enough"
6. **Kill Boolean Mutants**: Test both branches, verify operators (not just one path)

## References

- [cargo-mutants documentation](https://mutants.rs/)
- [Mutation Testing: A Comprehensive Survey](https://ieeexplore.ieee.org/document/5487526)
- [State of Mutation Testing at Google (2018)](https://research.google/pubs/pub46584/)
- [CLAUDE.md - TDD Methodology](../../CLAUDE.md)

## Appendix: Example Mutation Test Session

```bash
# 1. Run mutation tests on parser
$ cargo mutants --file src/frontend/parser/expressions.rs

Found 45 mutants in expressions.rs
Testing mutants...

❌ SURVIVED: line 234: replace + with - in parse_binary_expr
   Tests passed but mutant survived

# 2. Add test to kill the mutant
$ cat >> tests/parser_arithmetic_tdd.rs <<EOF
#[test]
fn test_addition_not_subtraction() {
    let ast = parse("2 + 3").unwrap();
    let result = eval_ast(&ast).unwrap();
    assert_eq!(result, Value::Integer(5));  // Would be -1 if + → -
}
EOF

# 3. Re-run mutation tests
$ cargo mutants --file src/frontend/parser/expressions.rs

Found 45 mutants in expressions.rs
Testing mutants...

✅ CAUGHT: line 234: replace + with - in parse_binary_expr
   test_addition_not_subtraction failed

Kill rate: 100% (45/45)
```

## Conclusion

Mutation testing is a powerful complement to traditional coverage metrics and PMAT quality enforcement. By achieving 90%+ mutation kill rate, we ensure that our tests don't just execute code—they actually verify correctness. This is critical for a compiler/interpreter where subtle bugs can have far-reaching consequences.

**Next Steps**:
1. Run baseline mutation tests
2. Document current kill rate
3. Implement Phase 1 (baseline)
4. Start Phase 2 (critical modules)
