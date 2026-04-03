# Sub-spec: WASM Quality Testing -- E2E Browser Testing

**Parent:** [wasm-quality-testing-spec.md](../wasm-quality-testing-spec.md) Section 4

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

