# Sub-spec: WASM Quality Testing -- Property, Mutation, and Quality Gates

**Parent:** [wasm-quality-testing-spec.md](../wasm-quality-testing-spec.md) Sections 5-7

---

## 5. Property-Based Testing

### 5.1 Why Property Testing for WASM?

**Goal**: Verify invariants hold across thousands of random inputs.

**Benefits**:
- Finds edge cases human testers miss
- Tests mathematical properties automatically
- Validates parser/transpiler correctness
- Ensures WASM output stability

### 5.2 Property Test Categories

#### Category 1: Parser Invariants

**Invariant**: Parse → Pretty Print → Parse = Identity

```rust
// tests/property/parser_properties.rs
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn proptest_parser_roundtrip(
        code in arb_ruchy_expression()
    ) {
        let ast1 = ruchy::parse(&code).unwrap();
        let pretty = ruchy::pretty_print(&ast1);
        let ast2 = ruchy::parse(&pretty).unwrap();

        // Invariant: AST should be identical after roundtrip
        assert_eq!(ast1, ast2);
    }
}
```

#### Category 2: Transpiler Invariants

**Invariant**: Transpiled Rust always compiles

```rust
proptest! {
    #[test]
    fn proptest_transpiler_always_compiles(
        code in arb_ruchy_function()
    ) {
        let rust_code = ruchy::transpile(&code).unwrap();

        // Invariant: Transpiled Rust must compile
        let result = compile_rust(&rust_code);
        assert!(result.is_ok(), "Transpiled code failed to compile: {:?}", result);
    }
}
```

#### Category 3: Interpreter Invariants

**Invariant**: Evaluation is deterministic

```rust
proptest! {
    #[test]
    fn proptest_interpreter_deterministic(
        code in arb_ruchy_expression()
    ) {
        let result1 = ruchy::eval(&code).unwrap();
        let result2 = ruchy::eval(&code).unwrap();

        // Invariant: Same input = same output
        assert_eq!(result1, result2);
    }
}
```

#### Category 4: WASM Invariants

**Invariant**: WASM output matches interpreter

```rust
proptest! {
    #[test]
    fn proptest_wasm_matches_interpreter(
        code in arb_ruchy_expression()
    ) {
        let interpreter_result = ruchy::eval(&code).unwrap();
        let wasm_result = ruchy_wasm::eval(&code).unwrap();

        // Invariant: WASM and interpreter produce same result
        assert_eq!(interpreter_result, wasm_result);
    }
}
```

### 5.3 Custom Generators

```rust
use proptest::prelude::*;

// Generate arbitrary Ruchy expressions
fn arb_ruchy_expression() -> impl Strategy<Value = String> {
    prop_oneof![
        arb_integer_expr(),
        arb_binary_expr(),
        arb_if_expr(),
        arb_function_call(),
    ]
}

fn arb_integer_expr() -> impl Strategy<Value = String> {
    any::<i32>().prop_map(|n| format!("{}", n))
}

fn arb_binary_expr() -> impl Strategy<Value = String> {
    (any::<i32>(), prop_oneof!["+", "-", "*", "/"], any::<i32>())
        .prop_map(|(a, op, b)| format!("{} {} {}", a, op, b))
}

fn arb_if_expr() -> impl Strategy<Value = String> {
    (any::<bool>(), any::<i32>(), any::<i32>())
        .prop_map(|(cond, then_val, else_val)| {
            format!("if {} {{ {} }} else {{ {} }}", cond, then_val, else_val)
        })
}
```

### 5.4 Property Test Configuration

```rust
// Run more cases for critical invariants
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn proptest_critical_invariant(input in arb_input()) {
        // This runs 10,000 random test cases
        assert!(invariant_holds(input));
    }
}
```

### 5.5 Property Test Metrics

**Target**: ≥20 property tests covering:
- Parser correctness (5 tests)
- Transpiler correctness (5 tests)
- Interpreter correctness (5 tests)
- WASM correctness (5 tests)

**Configuration**: 10,000 cases per test (configurable)

---

## 6. Mutation Testing

### 6.1 Why Mutation Testing?

**Goal**: Verify tests actually catch bugs.

**Process**:
1. Mutate code (change `==` to `!=`, `0` to `1`, etc.)
2. Run tests
3. Tests should FAIL (catch the mutation)

**Metric**: % of mutants killed by tests (target: ≥90%)

### 6.2 Mutation Testing Setup

**Tool**: `cargo-mutants` (v24.0+)

**Configuration**: `.cargo/mutants.toml`

```toml
# .cargo/mutants.toml
exclude_globs = [
    # Exclude WASM bindings (auto-generated)
    "src/wasm/bindings.rs",
    "src/wasm/wasm_bindgen_*",

    # Exclude test files
    "tests/**",
    "benches/**",

    # Exclude metadata-only changes
    "**/metadata.rs",
]

# Exclude non-behavioral mutants
exclude_re = [
    # Don't mutate RNG seeds (non-behavioral)
    "seed.*=.*42",

    # Don't mutate version strings
    "version.*=",

    # Don't mutate error message strings (cosmetic)
    'Error::.*\(".*"\)',
]

# Timeout per test (prevent infinite loops)
timeout = "300s"

# Show progress
show_all_logs = true
```

### 6.3 Running Mutation Tests

```bash
# Run mutation testing
make mutation

# Generate HTML report
cargo mutants --output target/mutants/report.html

# List survivors (mutants not caught by tests)
cargo mutants --list --caught false

# Test specific file
cargo mutants --file src/parser/mod.rs
```

### 6.4 Interpreting Results

```
140 mutants tested in 1m 29s:
- 126 caught (killed by tests)  ✓ GOOD
-   9 missed (survived)          ✗ BAD
-   5 unviable (don't compile)   ~ NEUTRAL

Kill rate: 93.3% (target: 90%) ✅
```

### 6.5 Improving Mutation Score

**Step 1**: Find survivors

```bash
cargo mutants --list --caught false
```

Output:
```
src/parser/mod.rs:218:9: replace parse_expr -> Result<Expr> with Ok(Expr::Null)
```

**Step 2**: Write test to catch mutation

```rust
#[test]
fn test_parse_expr_returns_correct_ast() {
    let input = "2 + 3";
    let ast = parse_expr(input).unwrap();

    // This test would fail if parse_expr always returned Expr::Null
    match ast.kind {
        ExprKind::Binary { op, left, right } => {
            assert_eq!(op, BinaryOp::Add);
            assert!(matches!(left.kind, ExprKind::Integer(2)));
            assert!(matches!(right.kind, ExprKind::Integer(3)));
        }
        _ => panic!("Expected Binary expression, got {:?}", ast.kind),
    }
}
```

**Step 3**: Verify mutation killed

```bash
make mutation
# Should now show this mutant as "caught"
```

### 6.6 Mutation Testing Targets

**Target Kill Rate**: ≥90%

**Per Module**:
- Parser: ≥90%
- Transpiler: ≥90%
- Interpreter: ≥90%
- WASM bindings: ≥85% (some auto-generated code)

---

## 7. Quality Gates

### 7.1 Comprehensive Quality Metrics

| Gate | Metric | Target | Enforcement |
|------|--------|--------|-------------|
| **Formatting** | cargo fmt | 100% | ✅ Blocking |
| **Linting** | clippy -D warnings | 0 warnings | ✅ Blocking |
| **Unit Tests** | cargo test | 100% passing | ✅ Blocking |
| **Property Tests** | proptest (10K cases) | 100% passing | ✅ Blocking |
| **E2E Tests** | Playwright (39 tests) | 100% passing | ✅ Blocking |
| **Coverage** | Line coverage | ≥85% | ✅ Blocking |
| **Mutation** | Kill rate | ≥90% | ⚠️ Warning |
| **Complexity** | Cyclomatic | ≤10 | ✅ Blocking |
| **Cognitive** | Cognitive load | ≤15 | ✅ Blocking |
| **SATD** | TODO/FIXME | 0 | ✅ Blocking |
| **Dead Code** | Unused functions | 0 | ✅ Blocking |
| **WASM Size** | Binary size | <500KB | ⚠️ Warning |

### 7.2 Makefile Targets

```makefile
# Quality gates for WASM backend
.PHONY: wasm-quality-gate
wasm-quality-gate: wasm-test wasm-e2e wasm-coverage wasm-mutation
	@echo "✅ All WASM quality gates passed"

# WASM tests
.PHONY: wasm-test
wasm-test:
	@echo "🧪 Running WASM tests..."
	cargo test --target wasm32-unknown-unknown --all-features
	@echo "✓ WASM tests passed"

# E2E browser tests
.PHONY: wasm-e2e
wasm-e2e: wasm-build
	@echo "🌐 Running E2E browser tests..."
	npm run test:e2e
	@echo "✓ E2E tests passed (39/39)"

# WASM coverage
.PHONY: wasm-coverage
wasm-coverage:
	@echo "📊 Generating WASM coverage report..."
	cargo llvm-cov --target wasm32-unknown-unknown --html
	@echo "✓ Coverage: $(shell cargo llvm-cov --target wasm32-unknown-unknown --summary-only | grep 'TOTAL' | awk '{print $$10}')"

# Mutation testing
.PHONY: wasm-mutation
wasm-mutation:
	@echo "🧬 Running mutation tests..."
	cargo mutants --target wasm32-unknown-unknown
	@echo "✓ Mutation kill rate: $(shell cargo mutants --json | jq '.kill_rate')"

# Property tests
.PHONY: wasm-proptest
wasm-proptest:
	@echo "🎲 Running property tests (10,000 cases each)..."
	PROPTEST_CASES=10000 cargo test --target wasm32-unknown-unknown proptest
	@echo "✓ Property tests passed"
```

### 7.3 Pre-commit Hooks

```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

echo "🔒 Running WASM quality gates..."

# Fast checks first
cargo fmt --check || {
    echo "❌ Formatting failed. Run: cargo fmt"
    exit 1
}

cargo clippy --target wasm32-unknown-unknown --all-features -- -D warnings || {
    echo "❌ Clippy failed. Fix warnings first."
    exit 1
}

# Unit tests
cargo test --target wasm32-unknown-unknown || {
    echo "❌ Unit tests failed."
    exit 1
}

# E2E tests (critical for WASM)
make wasm-e2e || {
    echo "❌ E2E tests failed. WASM deployment blocked."
    exit 1
}

echo "✅ All WASM quality gates passed - commit allowed"
```

---

