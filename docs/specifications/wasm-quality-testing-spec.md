# WASM Quality Testing Specification for Ruchy

**Version**: 1.0.0
**Status**: DRAFT
**Author**: Ruchy Development Team
**Based On**: wasm-labs v1.0.0 E2E Testing Methodology
**Date**: 2025-10-04

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Quality Philosophy](#quality-philosophy)
3. [Test Infrastructure Requirements](#test-infrastructure-requirements)
4. [E2E Browser Testing](#e2e-browser-testing)
5. [Property-Based Testing](#property-based-testing)
6. [Mutation Testing](#mutation-testing)
7. [Quality Gates](#quality-gates)
8. [CI/CD Integration](#cicd-integration)
9. [Critical Learnings from wasm-labs](#critical-learnings-from-wasm-labs)
10. [Implementation Roadmap](#implementation-roadmap)

---

## 1. Executive Summary

### Vision

Establish **world-class WASM quality assurance** for Ruchy compiler's WASM backend using:
- **E2E browser testing** with Playwright (3 browsers)
- **Property-based testing** (10,000+ inputs per invariant)
- **Mutation testing** (≥90% kill rate)
- **Comprehensive coverage** (≥85% line coverage)

### Target Metrics (Based on wasm-labs Success)

| Metric | Target | wasm-labs Achievement | Ruchy Status |
|--------|--------|----------------------|--------------|
| **E2E Tests** | 39+ (13 scenarios × 3 browsers) | ✅ 39 passing | ⏳ TODO |
| **Line Coverage** | ≥85% | ✅ 87% | 🔄 33.34% |
| **Mutation Kill Rate** | ≥90% | ✅ 99.4% | ⏳ TODO |
| **Property Tests** | 20+ invariants | ✅ 24 tests | ⏳ TODO |
| **Test Speed** | <10s E2E suite | ✅ ~6s | ⏳ N/A |
| **Cross-Browser** | Chromium, Firefox, WebKit | ✅ All passing | ⏳ TODO |

### Critical Success Factors

⚠️ **Non-Negotiable Requirements**:
1. **E2E tests MUST pass in all 3 browsers** (Chromium, Firefox, WebKit)
2. **JavaScript Error objects** required (not strings) for proper error handling
3. **Zero flaky tests** - all tests must be deterministic
4. **Fast feedback** - E2E suite completes in <10s
5. **Offline functionality** - WASM works after initial load

---

## 2. Quality Philosophy

### Extreme TDD for WASM

**Principle**: WASM integration bugs are silent killers. Only E2E tests catch them.

**Workflow**:
1. **RED**: Write failing E2E test in Playwright
2. **GREEN**: Make WASM implementation pass in all 3 browsers
3. **REFACTOR**: Improve code while keeping tests green
4. **VERIFY**: Run mutation tests to ensure test quality

### Toyota Way Application

- **Jidoka**: Automated quality gates stop deployment on test failures
- **Genchi Genbutsu**: Test in actual browsers, not just Rust
- **Kaizen**: Continuously improve test coverage and mutation score
- **Zero Defects**: No bypassing of E2E test failures

### Zero Tolerance Standards

❌ **NEVER**:
- Skip E2E tests due to time constraints
- Deploy WASM without all browsers passing
- Use `JsValue::from_str()` for errors (strings, not Error objects)
- Allow flaky tests in test suite
- Disable tests "temporarily"

✅ **ALWAYS**:
- Use `js_sys::Error::new()` for JavaScript Error objects
- Test in all 3 browsers (Chromium, Firefox, WebKit)
- Verify offline functionality works
- Include property tests for invariants
- Run mutation tests to verify test quality

---

## 3. Test Infrastructure Requirements

### 3.1 Directory Structure

```
ruchy/
├── tests/
│   ├── e2e/
│   │   ├── repl.spec.ts           # REPL E2E tests
│   │   ├── transpiler.spec.ts     # Transpiler E2E tests
│   │   ├── interpreter.spec.ts    # Interpreter E2E tests
│   │   └── offline.spec.ts        # Offline functionality
│   ├── property/
│   │   ├── parser_properties.rs   # Parser invariants
│   │   ├── transpiler_properties.rs
│   │   └── interpreter_properties.rs
│   └── mutation/
│       └── .cargo/
│           └── mutants.toml       # Mutation testing config
├── playwright.config.ts            # Playwright configuration
├── package.json                    # npm dependencies
├── index.html                      # WASM test harness
└── Makefile                        # Quality targets
```

### 3.2 Technology Stack

#### Rust Testing
- **cargo test**: Unit and integration tests
- **proptest**: Property-based testing (≥0.10)
- **cargo-mutants**: Mutation testing (≥24.0)
- **cargo-llvm-cov**: Coverage reporting

#### E2E Browser Testing
- **Playwright**: ^1.40.0 (TypeScript)
- **Browsers**: Chromium, Firefox, WebKit
- **Test Server**: Python http.server or similar
- **TypeScript**: For type-safe test code

#### CI/CD
- **GitHub Actions**: Automated test runs
- **Quality Gates**: Automated enforcement
- **Artifact Storage**: Test reports, screenshots, coverage

### 3.3 Installation Requirements

#### System Dependencies (Linux)
```bash
# Playwright system libraries (WebKit support)
sudo npx playwright install-deps

# Or via apt (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install -y \
    libwoff1 \
    libopus0 \
    libwebpdemux2 \
    libharfbuzz-icu0 \
    libgstreamer-plugins-base1.0-0 \
    libvpx7 \
    libenchant-2-2 \
    libsecret-1-0 \
    libhyphen0 \
    libgles2 \
    gstreamer1.0-libav
```

#### Rust Dependencies
```toml
# Cargo.toml
[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"  # CRITICAL: For proper Error objects
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
wasm-bindgen-test = "0.3"
proptest = "1.0"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
```

#### npm Dependencies
```json
{
  "name": "ruchy-wasm-tests",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "test:e2e": "playwright test",
    "test:e2e:ui": "playwright test --ui",
    "test:e2e:debug": "playwright test --debug",
    "test:e2e:headed": "playwright test --headed",
    "test:e2e:report": "playwright show-report",
    "lint:ts": "eslint . --ext .ts",
    "lint:html": "htmlhint *.html"
  },
  "devDependencies": {
    "@playwright/test": "^1.40.0",
    "@typescript-eslint/eslint-plugin": "^6.0.0",
    "@typescript-eslint/parser": "^6.0.0",
    "eslint": "^8.0.0",
    "htmlhint": "^1.1.4"
  }
}
```

---

## 4. E2E Browser Testing

### 4.1 Critical Context: Why E2E Testing is Non-Negotiable

**Real Failure Story from wasm-labs**:

During Phase 1, **all 39 E2E tests were silently failing for weeks**.

- **Root Cause**: `JsValue::from_str()` creates JavaScript strings, not Error objects
- **Impact**: JavaScript `catch (err) { err.message }` returned `undefined`
- **Detection**: Manual `make test-e2e` run revealed 0/39 passing
- **Fix**: Use `js_sys::Error::new()` for proper JavaScript Error objects
- **Lesson**: **Pure Rust tests cannot validate WASM-JavaScript integration**

### 4.2 Playwright Configuration

**File**: `playwright.config.ts`

```typescript
import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for Ruchy WASM E2E tests
 * See https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './tests/e2e',

  // Maximum time one test can run for
  timeout: 30 * 1000,

  // Run tests in files in parallel
  fullyParallel: true,

  // Fail the build on CI if you accidentally left test.only in the source code
  forbidOnly: !!process.env.CI,

  // Retry on CI only
  retries: process.env.CI ? 2 : 0,

  // Opt out of parallel tests on CI
  workers: process.env.CI ? 1 : undefined,

  // Reporter to use
  reporter: [
    ['html', { outputFolder: 'playwright-report' }],
    ['list'],
    ['json', { outputFile: 'test-results/e2e-results.json' }]
  ],

  // Shared settings for all the projects below
  use: {
    // Base URL to use in actions like `await page.goto('/')`
    baseURL: 'http://localhost:8000',

    // Collect trace when retrying the failed test
    trace: 'on-first-retry',

    // Screenshot on failure
    screenshot: 'only-on-failure',

    // Video on failure
    video: 'retain-on-failure',
  },

  // Configure projects for major browsers
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },

    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },

    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],

  // Run local dev server before starting the tests
  webServer: {
    command: 'python3 -m http.server 8000',
    url: 'http://localhost:8000',
    reuseExistingServer: !process.env.CI,
    stdout: 'ignore',
    stderr: 'pipe',
  },
});
```

### 4.3 JavaScript Error Handling (CRITICAL)

⚠️ **MOST COMMON WASM BUG**: Incorrect error object creation.

#### ❌ WRONG (Creates strings, not Errors)

```rust
// src/wasm/mod.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn transpile(code: String) -> Result<String, JsValue> {
    match ruchy::transpile(&code) {
        Ok(output) => Ok(output),
        Err(e) => Err(JsValue::from_str(&e.to_string())), // ❌ String!
    }
}
```

**JavaScript behavior**:
```javascript
try {
    const rust = wasm.transpile("invalid syntax");
} catch (err) {
    console.log(err.message); // undefined! ❌
    console.log(typeof err);   // "string"
}
```

#### ✅ CORRECT (Creates proper Error objects)

```rust
// src/wasm/mod.rs
use wasm_bindgen::prelude::*;
use js_sys;  // CRITICAL: Add to Cargo.toml

#[wasm_bindgen]
pub fn transpile(code: String) -> Result<String, JsValue> {
    match ruchy::transpile(&code) {
        Ok(output) => Ok(output),
        Err(e) => Err(js_sys::Error::new(&e.to_string()).into()), // ✅ Error object
    }
}
```

**JavaScript behavior**:
```javascript
try {
    const rust = wasm.transpile("invalid syntax");
} catch (err) {
    console.log(err.message); // "Parse error at line 1: unexpected token" ✅
    console.log(typeof err);   // "object"
    console.log(err instanceof Error); // true ✅
}
```

### 4.4 E2E Test Examples

#### Test 1: REPL Basic Functionality

**File**: `tests/e2e/repl.spec.ts`

```typescript
import { test, expect } from '@playwright/test';

test.describe('Ruchy REPL WASM E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Clear localStorage before each test
    await page.goto('/');
    await page.evaluate(() => localStorage.clear());
    await page.reload();
  });

  test('should load WASM and show ready status', async ({ page }) => {
    await page.goto('/');

    // Wait for WASM to load (status changes from loading to ready)
    const status = page.locator('#status');
    await expect(status).toHaveClass(/status-ready/, { timeout: 10000 });
    await expect(status).toHaveText('Ready');

    // Input should be enabled
    const input = page.locator('#repl-input');
    await expect(input).toBeEnabled();

    // Should show welcome message
    await expect(page.locator('#output')).toContainText('Welcome to Ruchy REPL');
  });

  test('should execute simple expression and display output', async ({ page }) => {
    await page.goto('/');

    // Wait for ready
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Execute expression
    await input.fill('2 + 2');
    await input.press('Enter');

    // Should show input and output
    await expect(output).toContainText('2 + 2');
    await expect(output).toContainText('4');
  });

  test('should display proper error messages for syntax errors', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Execute invalid syntax
    await input.fill('let x = ');
    await input.press('Enter');

    // Verify error message (requires js_sys::Error!)
    const errorText = await output.textContent();
    expect(errorText).toContain('Parse error');
    expect(errorText).not.toBe('undefined'); // Critical check!
  });

  test('should persist REPL history in localStorage', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');

    // Execute multiple expressions
    await input.fill('let x = 5');
    await input.press('Enter');
    await input.fill('x * 2');
    await input.press('Enter');

    // Reload page
    await page.reload();
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    // History should be restored
    const output = page.locator('#output');
    await expect(output).toContainText('let x = 5');
    await expect(output).toContainText('x * 2');
    await expect(output).toContainText('(from history)');
  });

  test('should work offline after initial load', async ({ page, context }) => {
    await page.goto('/');
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Execute expression to verify it works
    await input.fill('println("before offline")');
    await input.press('Enter');
    await expect(output).toContainText('before offline');

    // Go offline
    await context.setOffline(true);

    // Execute expressions while offline - should still work
    await input.fill('println("offline mode")');
    await input.press('Enter');
    await expect(output).toContainText('offline mode');

    await input.fill('2 + 3');
    await input.press('Enter');
    await expect(output).toContainText('5');

    // Go back online
    await context.setOffline(false);

    // Should still work
    await input.fill('println("back online")');
    await input.press('Enter');
    await expect(output).toContainText('back online');
  });

  test('should handle rapid expression execution without race conditions', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Execute multiple expressions rapidly
    for (let i = 1; i <= 10; i++) {
      await input.fill(`println("test${i}")`);
      await input.press('Enter');
    }

    // All expressions should be executed and displayed
    for (let i = 1; i <= 10; i++) {
      await expect(output).toContainText(`test${i}`);
    }
  });
});
```

#### Test 2: Transpiler E2E

**File**: `tests/e2e/transpiler.spec.ts`

```typescript
import { test, expect } from '@playwright/test';

test.describe('Ruchy Transpiler WASM E2E Tests', () => {
  test('should transpile simple function to Rust', async ({ page }) => {
    await page.goto('/transpiler.html');

    const input = page.locator('#ruchy-input');
    const output = page.locator('#rust-output');
    const transpileBtn = page.locator('#transpile-btn');

    await input.fill('fun add(a: i32, b: i32) -> i32 { a + b }');
    await transpileBtn.click();

    const rustCode = await output.textContent();
    expect(rustCode).toContain('fn add(a: i32, b: i32) -> i32');
    expect(rustCode).toContain('a + b');
  });

  test('should show transpilation errors with proper Error objects', async ({ page }) => {
    await page.goto('/transpiler.html');

    const input = page.locator('#ruchy-input');
    const error = page.locator('#error-message');
    const transpileBtn = page.locator('#transpile-btn');

    await input.fill('fun broken( {');
    await transpileBtn.click();

    const errorText = await error.textContent();
    expect(errorText).toContain('Parse error');
    expect(errorText).not.toBe('undefined');
  });
});
```

### 4.5 Test Scenarios Checklist

**Per Ruchy Feature** (13 scenarios minimum):
- [ ] REPL initialization and WASM loading
- [ ] Simple expression evaluation
- [ ] Variable declarations and scoping
- [ ] Function definitions
- [ ] Control flow (if/match/for/while)
- [ ] Error handling (syntax errors, runtime errors)
- [ ] History persistence (localStorage)
- [ ] Offline functionality
- [ ] Rapid execution (race condition testing)
- [ ] Transpiler output correctness
- [ ] Type checking integration
- [ ] Module system (import/export)
- [ ] Performance (expression execution <100ms)

**Total E2E Tests**: 13 scenarios × 3 browsers = **39 tests minimum**

### 4.6 Running E2E Tests

```bash
# Install dependencies (first time only)
make e2e-install

# Run all E2E tests (39 total: 13 scenarios × 3 browsers)
make test-e2e

# Run single browser
npx playwright test --project=chromium

# Run with UI for debugging
npx playwright test --ui

# Run headed (see browser)
npx playwright test --headed

# Run specific test file
npx playwright test tests/e2e/repl.spec.ts

# Generate HTML report
npx playwright show-report
```

**Expected Output**:
```
Running 39 tests using 3 workers
  ✓ [chromium] › repl.spec.ts:13:1 › should load WASM and show ready status (1.2s)
  ✓ [chromium] › repl.spec.ts:28:1 › should execute simple expression (0.8s)
  ...
  ✓ [webkit] › transpiler.spec.ts:45:1 › should show transpilation errors (0.9s)

  39 passed (6.2s)
```

### 4.7 Common Pitfalls

| Issue | Symptom | Solution |
|-------|---------|----------|
| **`err.message` is `undefined`** | E2E tests fail with "undefined" errors | Use `js_sys::Error::new()`, not `JsValue::from_str()` |
| **WebKit tests fail** | Only Chromium/Firefox pass | Run `make e2e-install-deps` for system libraries |
| **`npx` not found with sudo** | `sudo npx` fails | Use `sudo env PATH=$PATH npx` to preserve PATH |
| **Browsers not installed** | `browserType.launch: Executable doesn't exist` | Run `make e2e-install` |
| **Tests timeout** | Long wait times (>10s) | Check WASM binary size (<500KB), verify server running |
| **Flaky tests** | Intermittent failures | Add proper `waitFor` conditions, avoid `setTimeout` |

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

## 8. CI/CD Integration

### 8.1 GitHub Actions Workflow

**File**: `.github/workflows/wasm-quality.yml`

```yaml
name: WASM Quality Gates

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  wasm-fast-checks:
    name: Fast Quality Checks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Check formatting
        run: cargo fmt --check

      - name: Run clippy
        run: cargo clippy --target wasm32-unknown-unknown --all-features -- -D warnings

      - name: Run unit tests
        run: cargo test --target wasm32-unknown-unknown

  wasm-e2e-tests:
    name: E2E Browser Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install wasm-pack
        run: cargo install wasm-pack

      - name: Build WASM
        run: wasm-pack build --target web

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Install Playwright dependencies
        run: |
          npm ci
          npx playwright install --with-deps

      - name: Run E2E tests
        run: npm run test:e2e

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: playwright-report/
          retention-days: 30

      - name: Upload screenshots
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: test-screenshots
          path: test-results/
          retention-days: 7

  wasm-coverage:
    name: Coverage Report
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov

      - name: Generate coverage
        run: cargo llvm-cov --target wasm32-unknown-unknown --html

      - name: Check coverage threshold
        run: |
          COVERAGE=$(cargo llvm-cov --target wasm32-unknown-unknown --summary-only | grep 'TOTAL' | awk '{print $10}' | sed 's/%//')
          if (( $(echo "$COVERAGE < 85" | bc -l) )); then
            echo "❌ Coverage $COVERAGE% below 85% threshold"
            exit 1
          fi
          echo "✅ Coverage: $COVERAGE%"

      - name: Upload coverage report
        uses: actions/upload-artifact@v3
        with:
          name: coverage-report
          path: target/llvm-cov/html/

  wasm-mutation-testing:
    name: Mutation Testing
    runs-on: ubuntu-latest
    # Only run on main branch (slow)
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install cargo-mutants
        run: cargo install cargo-mutants

      - name: Run mutation testing
        run: cargo mutants --target wasm32-unknown-unknown

      - name: Check mutation score
        run: |
          KILL_RATE=$(cargo mutants --json | jq '.caught / (.caught + .missed) * 100')
          if (( $(echo "$KILL_RATE < 90" | bc -l) )); then
            echo "⚠️ Mutation kill rate $KILL_RATE% below 90% target"
            # Warning only, don't fail build
          fi
          echo "Mutation kill rate: $KILL_RATE%"
```

### 8.2 Quality Dashboard

**Automated Metrics Collection**:

```yaml
name: Quality Metrics Dashboard

on:
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight

jobs:
  collect-metrics:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Collect metrics
        run: |
          echo "📊 WASM Quality Report - $(date)" > report.md
          echo "====================================" >> report.md
          echo "" >> report.md

          # Test metrics
          echo "Tests:" >> report.md
          echo "- Unit: $(cargo test --target wasm32-unknown-unknown -- --list | wc -l)" >> report.md
          echo "- E2E: $(npx playwright test --list | wc -l)" >> report.md

          # Coverage
          echo "" >> report.md
          echo "Coverage:" >> report.md
          cargo llvm-cov --target wasm32-unknown-unknown --summary-only >> report.md

          # Mutation
          echo "" >> report.md
          echo "Mutation Testing:" >> report.md
          cargo mutants --json | jq '.kill_rate' >> report.md

      - name: Create issue with metrics
        uses: peter-evans/create-issue-from-file@v4
        with:
          title: Daily WASM Quality Metrics
          content-filepath: report.md
          labels: metrics, automated
```

---

## 9. Critical Learnings from wasm-labs

### 9.1 JavaScript Error Handling

**❌ Common Mistake**: Using `JsValue::from_str()` for errors
- Creates JavaScript strings, not Error objects
- `err.message` returns `undefined` in catch blocks
- Silent failures in E2E tests

**✅ Correct Pattern**: Use `js_sys::Error::new()`
- Creates proper JavaScript Error objects
- `err.message` works correctly
- E2E tests can verify error messages

**Code Pattern**:
```rust
// Always use this pattern
.map_err(|e| js_sys::Error::new(&e.to_string()).into())
```

### 9.2 E2E Testing Non-Negotiability

**Lesson**: Pure Rust tests cannot catch WASM-JavaScript integration bugs.

**Real Impact**:
- 39 E2E tests silently failing for weeks
- Bug only caught during manual testing
- Would have shipped broken WASM to users

**Prevention**:
- Run E2E tests in CI/CD (mandatory)
- Include E2E tests in pre-commit hooks
- Monitor E2E test results daily

### 9.3 Cross-Browser Compatibility

**Lesson**: WebKit behaves differently from Chromium/Firefox.

**Issues Found**:
- WebKit requires additional system libraries
- Different error message formats
- Timing differences in WASM loading

**Prevention**:
- Test all 3 browsers (Chromium, Firefox, WebKit)
- Document system dependencies clearly
- Use `waitFor` conditions instead of fixed delays

### 9.4 Offline Functionality

**Lesson**: WASM must work offline after initial load.

**Test Pattern**:
```typescript
// Go offline
await context.setOffline(true);

// Execute WASM functions - should still work
await executeWasmFunction();

// Go back online
await context.setOffline(false);
```

### 9.5 Performance Expectations

**Lesson**: WASM should be fast (<100ms per operation).

**Metrics**:
- WASM binary size: <500KB (target: ~130KB like wasm-labs)
- Load time: <1s
- Execution time: <100ms per operation
- E2E test suite: <10s total

---

## 10. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)

**Deliverables**:
- [ ] Install Playwright and dependencies
- [ ] Create `playwright.config.ts`
- [ ] Set up test directory structure
- [ ] Write first E2E test (REPL smoke test)
- [ ] Fix `js_sys::Error` in WASM bindings
- [ ] Verify all 3 browsers can run tests

**Success Criteria**:
- 1 E2E test passing in all 3 browsers
- No "undefined" error messages
- CI/CD running E2E tests

### Phase 2: Core E2E Coverage (Weeks 3-4)

**Deliverables**:
- [ ] 13 E2E test scenarios (39 total tests)
- [ ] REPL functionality tests (5 scenarios)
- [ ] Transpiler tests (4 scenarios)
- [ ] Error handling tests (2 scenarios)
- [ ] Offline functionality test (1 scenario)
- [ ] Performance test (1 scenario)

**Success Criteria**:
- All 39 E2E tests passing
- <10s E2E test suite execution
- Zero flaky tests

### Phase 3: Property Testing (Weeks 5-6)

**Deliverables**:
- [ ] 20 property tests with 10,000 cases each
- [ ] Parser invariant tests (5 tests)
- [ ] Transpiler invariant tests (5 tests)
- [ ] Interpreter invariant tests (5 tests)
- [ ] WASM correctness tests (5 tests)
- [ ] Custom generators for Ruchy expressions

**Success Criteria**:
- All property tests passing
- Edge cases discovered and fixed
- Custom generators for all AST nodes

### Phase 4: Mutation Testing (Weeks 7-8)

**Deliverables**:
- [ ] Install and configure cargo-mutants
- [ ] Create `.cargo/mutants.toml`
- [ ] Run mutation tests on parser
- [ ] Run mutation tests on transpiler
- [ ] Run mutation tests on interpreter
- [ ] Achieve ≥90% kill rate

**Success Criteria**:
- ≥90% mutation kill rate overall
- Per-module mutation scores documented
- Survivor mutants analyzed and tests added

### Phase 5: Integration & Documentation (Weeks 9-10)

**Deliverables**:
- [ ] CI/CD workflows for all quality gates
- [ ] Pre-commit hooks enforcing E2E tests
- [ ] Quality metrics dashboard
- [ ] Comprehensive testing documentation
- [ ] Developer setup guide
- [ ] Troubleshooting guide

**Success Criteria**:
- All quality gates automated
- Fresh checkout → all tests pass
- Documentation complete
- Team trained on testing methodology

---

## 11. Success Metrics

### 11.1 Quantitative Metrics

**Test Coverage**:
- ✅ 39+ E2E tests (13 scenarios × 3 browsers)
- ✅ 20+ property tests (10,000 cases each)
- ✅ ≥85% line coverage
- ✅ ≥90% mutation kill rate

**Quality Gates**:
- ✅ 100% tests passing
- ✅ 0 clippy warnings
- ✅ 0 SATD comments
- ✅ Complexity ≤10 per function

**Performance**:
- ✅ E2E suite <10s
- ✅ WASM binary <500KB
- ✅ Operation execution <100ms

### 11.2 Qualitative Metrics

**Developer Experience**:
- Fast feedback (<10s for E2E suite)
- Clear error messages (proper Error objects)
- Easy debugging (Playwright UI, screenshots)
- Comprehensive documentation

**User Experience**:
- Offline functionality works
- Cross-browser compatibility
- Fast WASM loading (<1s)
- Reliable error handling

---

## 12. Maintenance

### 12.1 Daily Checks

```bash
# Run before starting work
make wasm-quality-gate

# Expected output:
# ✅ All WASM quality gates passed
```

### 12.2 Weekly Reviews

- Review mutation test survivors
- Analyze E2E test failures
- Update property test generators
- Monitor quality metrics dashboard

### 12.3 Monthly Audits

- Full mutation testing run
- Cross-browser compatibility check
- Performance profiling
- Documentation updates

---

## 13. References

### 13.1 External Resources

- [Playwright Documentation](https://playwright.dev/)
- [proptest Guide](https://github.com/proptest-rs/proptest)
- [cargo-mutants](https://github.com/sourcefrog/cargo-mutants)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)

### 13.2 Internal Resources

- wasm-labs TESTING_GUIDE.md
- wasm-labs wasm-labs-spec-v1.md Section 7.4
- wasm-labs E2E test suite: `tests/e2e/`

### 13.3 Team Contacts

- **WASM Testing Lead**: TBD
- **E2E Infrastructure**: TBD
- **Mutation Testing**: TBD

---

## Appendix A: Makefile Integration

```makefile
# WASM Quality Testing Targets

.PHONY: e2e-install
e2e-install:
	@echo "📦 Installing Playwright and dependencies..."
	npm ci
	npx playwright install --with-deps
	@echo "✅ E2E dependencies installed"

.PHONY: e2e-install-deps
e2e-install-deps:
	@echo "📦 Installing system dependencies for Playwright..."
	sudo npx playwright install-deps
	@echo "✅ System dependencies installed"

.PHONY: wasm-build
wasm-build:
	@echo "🔨 Building WASM..."
	wasm-pack build --target web --out-dir pkg
	@echo "✅ WASM built: pkg/ruchy_bg.wasm"

.PHONY: test-e2e
test-e2e: wasm-build
	@echo "🌐 Running E2E tests (39 total: 3 browsers × 13 scenarios)..."
	npm run test:e2e
	@echo "✅ E2E tests passed"

.PHONY: test-e2e-ui
test-e2e-ui: wasm-build
	@echo "🌐 Opening Playwright UI..."
	npm run test:e2e:ui

.PHONY: test-e2e-debug
test-e2e-debug: wasm-build
	@echo "🐛 Running E2E tests in debug mode..."
	npm run test:e2e:debug

.PHONY: wasm-proptest
wasm-proptest:
	@echo "🎲 Running property tests (10,000 cases each)..."
	PROPTEST_CASES=10000 cargo test --target wasm32-unknown-unknown proptest
	@echo "✅ Property tests passed"

.PHONY: wasm-mutation
wasm-mutation:
	@echo "🧬 Running mutation tests..."
	cargo mutants --target wasm32-unknown-unknown
	@echo "✅ Mutation testing complete"

.PHONY: wasm-quality-gate
wasm-quality-gate: wasm-test test-e2e wasm-proptest
	@echo "🔒 All WASM quality gates passed ✅"
```

---

**End of Specification**

**Status**: DRAFT - Ready for Review
**Next Steps**: Team review → Implementation Phase 1
