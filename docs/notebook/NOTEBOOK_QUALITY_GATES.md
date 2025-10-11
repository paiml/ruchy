# Ruchy Notebook - EXTREME Quality Gates (wasm-labs inspired)

## 🎯 Vision: Jupyter-Level UX with Rust-Level Quality

**Goal**: Create a notebook experience that **empirically proves** all 41 language features work interactively.

---

## 🚦 3-Level Quality System (wasm-labs pattern)

### Level 1: quality-fast (<30s) - Pre-Commit
**Run**: `make notebook-quality-fast`

```makefile
notebook-quality-fast:
    @cargo fmt -- --check
    @cargo clippy -- -D warnings
    @cargo test --test notebook_core
    @echo "✅ Fast quality gate passed"
```

**Gates**:
- ✅ Format check
- ✅ Clippy warnings = errors
- ✅ Core notebook tests (20-30 tests)
- ❌ NO coverage (too slow)
- ❌ NO mutation (too slow)

---

### Level 2: quality-complete (~5min) - Pre-Push
**Run**: `make notebook-quality-complete`

```makefile
notebook-quality-complete: notebook-quality-fast
    @cargo test --features notebook --all
    @cargo llvm-cov --features notebook --fail-under-lines 85
    @cargo llvm-cov --features notebook --branch --fail-under-branches 90
    @echo "✅ Complete quality gate passed"
```

**Gates**:
- ✅ All fast checks
- ✅ All notebook tests
- ✅ Line coverage ≥85%
- ✅ Branch coverage ≥90%
- ❌ NO mutation (takes 10+ min)

---

### Level 3: quality-extreme (~10-15min) - Pre-Deploy
**Run**: `make notebook-quality-extreme`

```makefile
notebook-quality-extreme: notebook-quality-complete
    @cargo mutants --features notebook --file src/notebook/*.rs
    @# Verify mutation score ≥90%
    @echo "✅ Extreme quality gate passed"
```

**Gates**:
- ✅ All complete checks
- ✅ Mutation testing ≥90% score
- ✅ E2E tests with real browser
- ✅ WASM size check (<500KB)
- ✅ Zero WASI imports

---

## 📊 Coverage Requirements (wasm-labs standards)

```yaml
coverage:
  line_coverage:
    minimum: 85%
    target: 90%
    enforcement: "BLOCKING - CI fails below threshold"

  branch_coverage:
    minimum: 90%
    target: 95%
    enforcement: "BLOCKING - CI fails below threshold"
    note: "Branch coverage proves decision paths tested"

  mutation_coverage:
    minimum: 90%
    target: 95%
    enforcement: "BLOCKING - Pre-deploy only"
    note: "Mutation testing proves tests catch real bugs"
```

**Why Branch Coverage ≥90%?**
- Line coverage measures execution
- Branch coverage measures decisions
- Mutation coverage measures effectiveness

**Example**:
```rust
// This has 100% line coverage but only 50% branch coverage
if x > 0 { /* tested */ } else { /* NOT tested */ }
```

---

## 🧬 Mutation Testing Requirements

**Target**: ≥90% mutation score (wasm-labs standard)

**Fast Mutation Testing** (for development):
```bash
# Only test recent changes
cargo mutants --features notebook --in-diff HEAD~1
# Runtime: ~2-3 minutes
```

**Full Mutation Testing** (for CI/deploy):
```bash
# Test all notebook code
cargo mutants --features notebook --file src/notebook/*.rs
# Runtime: ~10-15 minutes
```

**Mutation Score Calculation**:
```
mutation_score = caught_mutants / (caught_mutants + missed_mutants)
Target: ≥90%
```

---

## 🎭 E2E Testing with Playwright (wasm-labs pattern)

### Test Structure
```
tests/
├── e2e/
│   ├── notebook-basic.spec.ts     # Basic notebook operations
│   ├── notebook-features.spec.ts  # All 41 language features
│   ├── notebook-error.spec.ts     # Error handling
│   └── notebook-wasm.spec.ts      # WASM-specific tests
```

### Test Scenarios (Minimum)
```typescript
test('Notebook loads and runs code', async ({ page }) => {
  await page.goto('http://localhost:8000/notebook.html');
  await page.fill('textarea', '1 + 1');
  await page.click('button.run');
  await expect(page.locator('.output')).toContainText('2');
});

test('All 41 language features work', async ({ page }) => {
  // Load feature test suite
  const features = await loadFeatureTests();

  for (const feature of features) {
    await testFeature(page, feature);
  }

  // All 41 must pass
  expect(passedFeatures).toBe(41);
});
```

### Browser Matrix
```yaml
browsers:
  - Chrome (latest)
  - Firefox (latest)
  - Safari (latest - MacOS only)

matrix_tests: 41 features × 3 browsers = 123 test runs
runtime: ~5-10 minutes
```

---

## 📦 WASM Quality Gates

### Size Requirements
```yaml
wasm_size:
  maximum: 500KB     # Hard limit
  target: 300KB      # Ideal
  current: TBD       # Measure with `make wasm-size`
```

### Purity Requirements
```yaml
wasm_purity:
  wasi_imports: 0    # Pure WASM only
  js_glue: "minimal" # Only essentials
  verification: "wasm-objdump -x notebook.wasm | grep -c wasi_"
```

### Validation Commands
```bash
# Check WASM size
make wasm-size

# Validate WASM structure
make wasm-check

# Deep inspection with PMAT
make pmat-wasm-notebook
```

---

## 🔬 Notebook-Specific Testing

### 1. Cell Execution Tests
```rust
#[test]
fn test_cell_executes_expression() {
    let mut notebook = Notebook::new();
    let result = notebook.execute_cell("1 + 1");
    assert_eq!(result, "2");
}

#[test]
fn test_cell_preserves_state() {
    let mut notebook = Notebook::new();
    notebook.execute_cell("let x = 10");
    let result = notebook.execute_cell("x + 5");
    assert_eq!(result, "15");
}
```

### 2. Output Formatting Tests
```rust
#[test]
fn test_rich_output_dataframe() {
    let mut notebook = Notebook::new();
    let result = notebook.execute_cell("df![[1, 2], [3, 4]]");
    assert!(result.contains("<table>"));
}

#[test]
fn test_rich_output_error() {
    let mut notebook = Notebook::new();
    let result = notebook.execute_cell("undefined_var");
    assert!(result.contains("Error:"));
    assert!(result.contains("undefined_var"));
}
```

### 3. WASM Integration Tests
```rust
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_notebook_init() {
    let notebook = Notebook::new();
    assert!(notebook.is_ready());
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_cell_execution() {
    let mut notebook = Notebook::new();
    let result = notebook.execute_cell("2 + 2");
    assert_eq!(result, "4");
}
```

### 4. Property Tests (10,000+ iterations)
```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn notebook_never_panics(code: String) {
            let mut notebook = Notebook::new();
            let _ = notebook.execute_cell(&code);
            // Should not panic, even on invalid input
        }

        #[test]
        fn state_isolation_property(
            var_name in "[a-z]+",
            value in any::<i64>()
        ) {
            let mut nb1 = Notebook::new();
            let mut nb2 = Notebook::new();

            nb1.execute_cell(&format!("let {} = {}", var_name, value));

            // nb2 should NOT see nb1's variables
            let result = nb2.execute_cell(&var_name);
            assert!(result.contains("Error"));
        }
    }
}
```

---

## 📚 The MD Book: Language Proof via Notebook

**Structure**:
```
docs/notebook/book/
├── src/
│   ├── SUMMARY.md                 # Table of contents
│   ├── 00-introduction.md         # Why this book exists
│   │
│   ├── 01-basic-syntax/
│   │   ├── README.md
│   │   ├── 01-literals.md         # Feature 1/41
│   │   ├── 02-variables.md        # Feature 2/41
│   │   ├── 03-comments.md         # Feature 3/41
│   │   └── proof.ruchy            # Runnable proof
│   │
│   ├── 02-operators/
│   │   ├── README.md
│   │   ├── 01-arithmetic.md       # Feature 4/41
│   │   ├── 02-comparison.md       # Feature 5/41
│   │   └── proof.ruchy
│   │
│   ├── 03-control-flow/
│   │   ├── README.md
│   │   ├── 01-if-else.md          # Feature 6/41
│   │   ├── 02-match.md            # Feature 7/41
│   │   ├── 03-for-loops.md        # Feature 8/41
│   │   └── proof.ruchy
│   │
│   ├── ... (remaining 33 features)
│   │
│   └── 15-validation/
│       ├── README.md
│       ├── 01-coverage-report.md  # Test coverage proof
│       ├── 02-mutation-report.md  # Mutation testing proof
│       └── 03-e2e-report.md       # Playwright test proof
│
└── book.toml                      # mdBook configuration
```

### Book Philosophy

**Every Chapter = Empirical Proof**:
1. **Feature Description**: What it does
2. **Notebook Code**: Copy-paste into notebook
3. **Expected Output**: What you should see
4. **Test Proof**: Link to automated test
5. **Coverage Proof**: Link to coverage report
6. **Mutation Proof**: Link to mutation test

**Example Chapter** (01-basic-syntax/01-literals.md):

```markdown
# Literals - Feature 1/41

## Description
Ruchy supports literals for integers, floats, strings, booleans, and nil.

## Try it in Notebook

Open ruchy notebook and run:

\`\`\`ruchy
# Integer literal
42

# Float literal
3.14

# String literal
"hello"

# Boolean literals
true
false

# Nil literal
nil
\`\`\`

## Expected Output

\`\`\`
42
3.14
"hello"
true
false
nil
\`\`\`

## Proof

✅ **Test**: `tests/notebook/test_literals.rs::test_integer_literal`
✅ **Coverage**: 100% (5/5 lines)
✅ **Mutation**: 100% (3/3 mutants caught)
✅ **E2E**: `tests/e2e/notebook-features.spec.ts::test_literals`

[View test source](../../tests/notebook/test_literals.rs)
[View coverage report](../../target/llvm-cov/html/test_literals.html)
```

---

## 🎯 Implementation Plan

### Phase 4A: Notebook Core (Week 1)
**Goal**: Basic REPL-style notebook with state persistence

```yaml
tasks:
  - id: "NOTEBOOK-001"
    title: "Notebook core infrastructure"
    tests: "30 unit tests"
    coverage: "≥85% line, ≥90% branch"

  - id: "NOTEBOOK-002"
    title: "Cell execution engine"
    tests: "20 unit tests + 100 property tests"
    coverage: "≥85% line, ≥90% branch"

  - id: "NOTEBOOK-003"
    title: "State persistence across cells"
    tests: "15 unit tests + 50 property tests"
    coverage: "≥85% line, ≥90% branch"
```

### Phase 4B: Rich Output (Week 2)
**Goal**: HTML tables, syntax highlighting, error formatting

```yaml
tasks:
  - id: "NOTEBOOK-004"
    title: "Rich output formatting"
    tests: "25 unit tests"
    coverage: "≥85% line, ≥90% branch"

  - id: "NOTEBOOK-005"
    title: "DataFrame HTML rendering"
    tests: "15 unit tests"
    coverage: "≥85% line, ≥90% branch"
```

### Phase 4C: WASM Integration (Week 3)
**Goal**: Run in browser with full WASM compilation

```yaml
tasks:
  - id: "NOTEBOOK-006"
    title: "WASM notebook runtime"
    tests: "20 WASM tests"
    wasm_size: "<500KB"
    wasm_purity: "0 WASI imports"

  - id: "NOTEBOOK-007"
    title: "Browser integration"
    tests: "30 E2E tests (Playwright)"
    browsers: "Chrome, Firefox, Safari"
```

### Phase 4D: The Book (Week 4)
**Goal**: 41-chapter MD book proving all features work

```yaml
tasks:
  - id: "NOTEBOOK-008"
    title: "MD book structure"
    chapters: 41
    proof_type: "Automated test + coverage + mutation"

  - id: "NOTEBOOK-009"
    title: "Automated proof generation"
    description: "Script that extracts test results into book"
```

---

## 🚀 Makefile Targets

```makefile
# ============================================================================
# Notebook Quality Gates (3-Level System)
# ============================================================================

notebook-quality-fast: fmt clippy test-notebook-core
	@echo "✅ Notebook fast quality gate passed (<30s)"

notebook-quality-complete: notebook-quality-fast test-notebook-all coverage-notebook
	@echo "✅ Notebook complete quality gate passed (~5min)"

notebook-quality-extreme: notebook-quality-complete mutants-notebook e2e-notebook
	@echo "✅ Notebook extreme quality gate passed (~10-15min)"

# ============================================================================
# Notebook Testing
# ============================================================================

test-notebook-core:
	@cargo test --features notebook --test notebook_core

test-notebook-all:
	@cargo test --features notebook --all

# ============================================================================
# Notebook Coverage
# ============================================================================

coverage-notebook:
	@cargo llvm-cov --features notebook --branch --html
	@# Verify thresholds
	@cargo llvm-cov --features notebook --fail-under-lines 85
	@# Branch coverage check
	@echo "Checking branch coverage ≥90%..."

# ============================================================================
# Notebook Mutation Testing
# ============================================================================

mutants-notebook:
	@cargo mutants --features notebook --file src/notebook/*.rs

mutants-notebook-fast:
	@cargo mutants --features notebook --in-diff HEAD~1

# ============================================================================
# Notebook E2E Testing
# ============================================================================

e2e-notebook:
	@npx playwright test tests/e2e/notebook*.spec.ts

e2e-notebook-ui:
	@npx playwright test tests/e2e/notebook*.spec.ts --ui

# ============================================================================
# Notebook WASM
# ============================================================================

wasm-notebook:
	@cargo build --features notebook --target wasm32-unknown-unknown --release
	@wasm-bindgen target/wasm32-unknown-unknown/release/ruchy.wasm \
		--out-dir dist/notebook --target web

wasm-notebook-size:
	@stat -c%s dist/notebook/ruchy_bg.wasm | awk '{print "WASM size: " $$1/1024 "KB"}'

wasm-notebook-check:
	@# Verify size <500KB
	@# Verify 0 WASI imports

# ============================================================================
# The Book
# ============================================================================

book-build:
	@mdbook build docs/notebook/book

book-serve:
	@mdbook serve docs/notebook/book --open

book-proof-generate:
	@# Extract test results, coverage, mutation into book chapters
	@python3 scripts/generate_book_proofs.py
```

---

## ✅ Success Criteria

**Notebook is production-ready when**:
1. ✅ All 41 language features work in notebook
2. ✅ Line coverage ≥85%, branch coverage ≥90%
3. ✅ Mutation score ≥90%
4. ✅ E2E tests pass on 3 browsers
5. ✅ WASM binary <500KB with 0 WASI imports
6. ✅ MD book with 41 chapters of empirical proof
7. ✅ All quality gates pass (fast/complete/extreme)

**Result**: A notebook that **empirically proves** Ruchy is production-ready.
