# Sub-spec: Mutation Testing -- Categories, Priority Modules, and Execution

**Parent:** [MUTATION_TESTING.md](../MUTATION_TESTING.md) Sections 1-6

---

**Version**: 1.0.0
**Status**: Draft
**Based On**: pforge mutation testing methodology

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
- Rust-native, understands Cargo workspaces
- Fast: Caches test results, runs in parallel
- Smart mutations: Knows Rust semantics
- JSON output for CI/CD integration
- Active development, well-maintained

## Mutation Categories

### 1. Arithmetic Operators (High Priority for Ruchy)

**Mutations**:
- `a + b` -> `a - b`, `a * b`, `a / b`
- `a - b` -> `a + b`, `a * b`, `a / b`
- `a * b` -> `a + b`, `a - b`, `a / b`
- `a / b` -> `a + b`, `a - b`, `a * b`

**Ruchy Impact**: Critical for evaluator arithmetic, WASM code generation

**Example Test to Kill**:
```rust
#[test]
fn test_evaluator_addition_exact() {
    let result = eval("2 + 3").unwrap();
    assert_eq!(result, Value::Integer(5));  // Kills +->- mutation

    let result2 = eval("10 + 7").unwrap();
    assert_eq!(result2, Value::Integer(17));  // Different values ensure + not *
}
```

### 2. Boolean Operators (High Priority)

**Mutations**:
- `a && b` -> `a || b`
- `a || b` -> `a && b`
- `!condition` -> `condition`

**Ruchy Impact**: Critical for control flow, pattern matching, type checking

**Example Test to Kill**:
```rust
#[test]
fn test_and_operator_not_or() {
    assert_eq!(eval("true && false"), Value::Bool(false));  // Kills &&->||
    assert_eq!(eval("false && false"), Value::Bool(false));
}

#[test]
fn test_or_operator_not_and() {
    assert_eq!(eval("true || false"), Value::Bool(true));  // Kills ||->&&
    assert_eq!(eval("false || false"), Value::Bool(false));
}
```

### 3. Comparison Operators (High Priority)

**Mutations**:
- `a < b` -> `a <= b`, `a > b`, `a >= b`, `a == b`, `a != b`
- `a == b` -> `a != b`
- `a != b` -> `a == b`

**Ruchy Impact**: Critical for loop bounds, conditional evaluation

**Example Test to Kill**:
```rust
#[test]
fn test_less_than_exact() {
    assert_eq!(eval("5 < 10"), Value::Bool(true));   // true
    assert_eq!(eval("10 < 10"), Value::Bool(false)); // Kills <-><=
    assert_eq!(eval("15 < 10"), Value::Bool(false)); // Kills <->>
}
```

### 4. Return Value Replacements (Medium Priority)

**Mutations**:
- `Ok(value)` -> `Ok(Default::default())`
- `Some(value)` -> `Some(Default::default())`
- `function()` -> `Default::default()`

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
- `fn foo() { ... }` -> `fn foo() {}`
- `fn foo() -> Result<T> { ... }` -> `fn foo() -> Result<T> { Ok(Default::default()) }`

**Ruchy Impact**: Medium - mostly affects utility functions

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
		(echo "Mutation kill rate below 90%" && exit 1)

# Generate HTML report
mutants-report:
	cargo mutants --workspace --json --output mutants.json
```
