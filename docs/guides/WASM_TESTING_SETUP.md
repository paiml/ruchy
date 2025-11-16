# WASM Testing Setup Guide

**Version**: 1.0.0
**Last Updated**: 2025-10-08
**Audience**: Ruchy developers setting up WASM quality testing environment

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Detailed Setup](#detailed-setup)
4. [Running Tests](#running-tests)
5. [Troubleshooting](#troubleshooting)
6. [CI/CD Integration](#cicd-integration)

---

## Prerequisites

### System Requirements

- **Operating System**: Linux, macOS, or Windows (WSL2)
- **Rust**: 1.75.0 or later
- **Node.js**: 18.0.0 or later (for Playwright)
- **Disk Space**: ~2GB for browsers and dependencies
- **Memory**: 4GB RAM minimum (8GB recommended)

### Required Tools

```bash
# Rust toolchain (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# WASM target
rustup target add wasm32-unknown-unknown

# wasm-pack (for building WASM modules)
cargo install wasm-pack

# Node.js (via nvm recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18
```

---

## Quick Start

### 1. Clone and Build

```bash
git clone https://github.com/paiml/ruchy.git
cd ruchy
cargo build --release
```

### 2. Install Playwright

```bash
# Install Playwright with browsers
npx playwright install --with-deps

# Verify installation
npx playwright --version
```

### 3. Run All WASM Tests

```bash
# E2E tests (all 3 browsers)
make test-wasm-e2e

# Property tests
cargo test --test wasm_memory_property_tests

# Memory model tests
cargo test --test wasm_memory_model

# All WASM tests
make test-wasm-all
```

### 4. Verify Setup

```bash
# Should show: 39/39 E2E tests passing
make test-wasm-e2e

# Should show: 17/17 E2E tests passing
cargo test --test wasm_memory_model

# Should show: 16/16 tests passing (9 property + 7 invariant)
cargo test --test wasm_memory_property_tests
```

**Expected Output**:
```
✅ Total WASM tests: 72/72 passing
   - E2E: 39/39 (13 scenarios × 3 browsers)
   - Memory Model E2E: 17/17
   - Property: 9/9
   - Invariant: 7/7
```

---

## Detailed Setup

### Step 1: Install Playwright Dependencies

Playwright requires system libraries for browser automation:

#### Linux (Ubuntu/Debian)
```bash
# Playwright will prompt for sudo password
npx playwright install --with-deps

# Manual installation if needed
sudo apt-get update
sudo apt-get install -y \
    libnss3 libnspr4 libatk1.0-0 libatk-bridge2.0-0 \
    libcups2 libdrm2 libdbus-1-3 libxkbcommon0 \
    libxcomposite1 libxdamage1 libxfixes3 libxrandr2 \
    libgbm1 libpango-1.0-0 libcairo2 libasound2
```

#### macOS
```bash
# Playwright handles dependencies automatically
npx playwright install --with-deps

# If WebKit fails, install via Homebrew
brew install webkit
```

#### Windows (WSL2)
```bash
# Follow Linux instructions in WSL2
npx playwright install --with-deps
```

### Step 2: Configure Test Environment

Create test configuration (already exists in repo):

**File**: `playwright.config.ts`
```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    trace: 'on-first-retry',
  },

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

  webServer: {
    command: 'python3 -m http.server 8000',
    url: 'http://localhost:8000',
    reuseExistingServer: !process.env.CI,
  },
});
```

### Step 3: Build WASM Module

```bash
# Build WASM module for testing
make wasm-build

# Verify build output
ls -lh pkg/ruchy_wasm_bg.wasm
# Should show ~942KB file

# Check WASM magic number
head -c 4 pkg/ruchy_wasm_bg.wasm | od -An -tx1
# Should show: 00 61 73 6d (WASM magic number)
```

### Step 4: Start Development Server

```bash
# Terminal 1: Start HTTP server for WASM testing
python3 -m http.server 8000

# Verify server is running
curl http://localhost:8000/index.html
# Should return HTML content
```

### Step 5: Run Tests

```bash
# Terminal 2: Run E2E tests
npx playwright test

# Expected output:
# 39 passed (13 scenarios × 3 browsers)
# Execution time: ~6.5s
```

---

## Running Tests

### Make Targets (Recommended)

```bash
# All WASM tests
make test-wasm-all

# E2E tests only
make test-wasm-e2e

# Property tests only
make test-wasm-property

# Memory model tests only
make test-wasm-memory

# Quick validation (E2E + build check)
make wasm-check
```

### Cargo Commands

```bash
# Memory model E2E tests
cargo test --test wasm_memory_model

# Property tests (including ignored tests)
cargo test --test wasm_memory_property_tests
cargo test --test wasm_memory_property_tests property_tests -- --ignored

# Invariant tests
cargo test --test wasm_memory_property_tests invariant_tests

# Specific test
cargo test --test wasm_memory_model test_wasm_array_mutation
```

### Playwright Commands

```bash
# All E2E tests (all browsers)
npx playwright test

# Specific browser
npx playwright test --project=chromium
npx playwright test --project=firefox
npx playwright test --project=webkit

# Headed mode (see browser)
npx playwright test --headed

# Debug mode (step through tests)
npx playwright test --debug

# Specific test file
npx playwright test tests/e2e/repl.spec.ts

# Show test report
npx playwright show-report
```

### Test Filters

```bash
# Run tests matching pattern
npx playwright test --grep "REPL"

# Exclude tests matching pattern
npx playwright test --grep-invert "slow"

# Run only failed tests
npx playwright test --last-failed
```

---

## Troubleshooting

### Common Issues

#### 1. WebKit Installation Fails (Linux)

**Symptom**: `Error: Failed to launch webkit`

**Solution**:
```bash
# Install WebKit dependencies manually
sudo apt-get install -y libwpe-1.0-3 libwpebackend-fdo-1.0-1

# Reinstall WebKit
npx playwright install webkit
```

#### 2. Port 8000 Already in Use

**Symptom**: `OSError: [Errno 98] Address already in use`

**Solution**:
```bash
# Find process using port 8000
lsof -i :8000

# Kill process
kill -9 <PID>

# Or use different port
python3 -m http.server 8001
# Update playwright.config.ts webServer.url to match
```

#### 3. WASM Build Fails

**Symptom**: `error: could not compile 'ruchy-wasm'`

**Solution**:
```bash
# Clean build artifacts
cargo clean

# Rebuild WASM target
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release

# Or use wasm-pack
wasm-pack build --target web
```

#### 4. Tests Timeout

**Symptom**: `Test timeout of 30000ms exceeded`

**Solution**:
```bash
# Increase timeout in playwright.config.ts
export default defineConfig({
  timeout: 60000, // 60 seconds
  // ...
});

# Or for specific test
test('slow test', async ({ page }) => {
  test.setTimeout(60000);
  // ...
});
```

#### 5. Property Tests Fail

**Symptom**: `Property test failed for input: ...`

**Solution**:
```bash
# Run with detailed output
cargo test --test wasm_memory_property_tests -- --nocapture

# Check for specific failure pattern
cargo test --test wasm_memory_property_tests prop_tuple_creation_always_valid -- --ignored --nocapture

# Verify proptest shrinking
# Proptest will minimize failing input automatically
```

#### 6. Memory Model Tests Fail

**Symptom**: `assertion failed: wasm_bytes.starts_with(b"\0asm")`

**Solution**:
```bash
# Check WASM compilation
cargo build --target wasm32-unknown-unknown --release

# Verify WASM magic number
xxd pkg/ruchy_wasm_bg.wasm | head -n 1
# Should show: 00000000: 0061 736d ...

# Clean and rebuild
cargo clean
make wasm-build
cargo test --test wasm_memory_model
```

---

## CI/CD Integration

### GitHub Actions Workflow

Create `.github/workflows/wasm-quality.yml`:

```yaml
name: WASM Quality Tests

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  wasm-tests:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: wasm32-unknown-unknown
        override: true

    - name: Install wasm-pack
      run: cargo install wasm-pack

    - name: Install Playwright
      run: npx playwright install --with-deps

    - name: Build WASM
      run: make wasm-build

    - name: Run E2E Tests
      run: make test-wasm-e2e

    - name: Run Memory Model Tests
      run: cargo test --test wasm_memory_model

    - name: Run Property Tests
      run: cargo test --test wasm_memory_property_tests

    - name: Upload Test Report
      if: always()
      uses: actions/upload-artifact@v3
      with:
        name: playwright-report
        path: playwright-report/
        retention-days: 30
```

### Pre-commit Hooks

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash

set -e

echo "🧪 Running WASM quality checks..."

# Build WASM module
echo "📦 Building WASM..."
make wasm-build || exit 1

# Run quick E2E smoke test
echo "🌐 Running E2E smoke tests..."
npx playwright test --grep "REPL loads successfully" || exit 1

# Run memory model tests
echo "💾 Running memory model tests..."
cargo test --test wasm_memory_model -- --test-threads=1 || exit 1

echo "✅ All WASM quality checks passed!"
```

Make executable:
```bash
chmod +x .git/hooks/pre-commit
```

---

## Test Coverage and Quality Metrics

### Current Coverage

```bash
# Generate coverage report
cargo llvm-cov --html --test wasm_memory_model
cargo llvm-cov --html --test wasm_memory_property_tests

# View coverage
open target/llvm-cov/html/index.html
```

### Quality Metrics Dashboard

Run quality checks:

```bash
# PMAT quality gates
pmat quality-gates validate

# Complexity analysis
pmat analyze complexity --max-cyclomatic 10

# Test coverage
cargo llvm-cov --branch --fail-under-branches 80
```

### Mutation Testing (Phase 4 - Blocked)

```bash
# When integration tests are fixed:
cargo mutants --file src/backend/wasm/mod.rs --timeout 300
```

---

## Development Workflow

### Daily Workflow

```bash
# 1. Morning: Check project health
make wasm-check

# 2. During development: Quick validation
cargo test --test wasm_memory_model --test-threads=1

# 3. Before commit: Full test suite
make test-wasm-all

# 4. Pre-push: Verify all browsers
npx playwright test --project=chromium --project=firefox --project=webkit
```

### Adding New Tests

#### E2E Test (Playwright)

```typescript
// tests/e2e/my-feature.spec.ts
import { test, expect } from '@playwright/test';

test('my feature works in WASM', async ({ page }) => {
  await page.goto('http://localhost:8000');

  // Wait for WASM to load
  await page.waitForFunction(() => window.ruchy_wasm !== undefined);

  // Test your feature
  const result = await page.evaluate(() => {
    return window.ruchy_wasm.transpile('my code here');
  });

  expect(result).toContain('expected output');
});
```

#### Memory Model Test

```rust
// tests/wasm_memory_model.rs
#[test]
fn test_wasm_my_feature() {
    let code = r#"
fn main() {
    // Your test code
}
"#;
    let ruchy_file = temp_ruchy_file("my_feature", code);
    let wasm_file = temp_wasm_file("my_feature");

    ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .success();

    assert!(wasm_file.exists());
    let wasm_bytes = fs::read(&wasm_file).expect("Failed to read WASM file");
    assert!(
        wasm_bytes.starts_with(b"\0asm"),
        "Invalid WASM magic number"
    );

    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();
}
```

#### Property Test

```rust
// tests/wasm_memory_property_tests.rs
proptest! {
    #[test]
    #[ignore]
    fn prop_my_invariant(
        input in -1000i32..1000
    ) {
        let code = format!(r#"
fn main() {{
    let x = {}
    println(x)
}}
"#, input);

        prop_assert!(
            compiles_to_valid_wasm(&code, "my_test"),
            "Input {} should compile to valid WASM",
            input
        );
    }
}
```

---

## Resources

### Documentation
- [WASM Quality Sprint 7 Completion Report](../execution/WASM_QUALITY_SPRINT7_COMPLETION.md)
- [WASM Memory Model Achievement](../execution/WASM_MEMORY_MODEL_ACHIEVEMENT.md)
- [WASM Memory Model Design](../execution/WASM_MEMORY_MODEL.md)
- [WASM Limitations](../execution/WASM_LIMITATIONS.md)

### External Resources
- [Playwright Documentation](https://playwright.dev/)
- [wasm-pack Guide](https://rustwasm.github.io/docs/wasm-pack/)
- [Proptest Book](https://altsysrq/proptest-book)
- [WASM Specification](https://webassembly.github.io/spec/)

### Tools
- [Playwright Debugging](https://playwright.dev/docs/debug)
- [WASM Binary Toolkit (wabt)](https://github.com/WebAssembly/wabt)
- [wasm2wat](https://webassembly.github.io/wabt/demo/wasm2wat/) - WASM to WAT converter

---

## Support

### Getting Help

1. **Check Documentation**: Read troubleshooting section above
2. **Run Diagnostics**: `make wasm-check` for quick health check
3. **Check Logs**: `npx playwright test --debug` for detailed output
4. **GitHub Issues**: Open issue with reproduction steps

### Reporting Bugs

Include in bug report:
- OS and version
- Rust version (`rustc --version`)
- Node version (`node --version`)
- Playwright version (`npx playwright --version`)
- Full error message
- Steps to reproduce
- Expected vs actual behavior

---

**Version**: 1.0.0
**Last Updated**: 2025-10-08
**Maintainer**: Ruchy Development Team
