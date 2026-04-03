# Sub-spec: WASM Quality Testing -- CI/CD, Learnings, and Roadmap

**Parent:** [wasm-quality-testing-spec.md](../wasm-quality-testing-spec.md) Sections 8-13

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
- [wasm-bindgen Guide](https://rustwasm.github.io/docs/wasm-bindgen/)

### 13.2 Internal Resources

- wasm-labs TESTING_GUIDE.md
- wasm-labs wasm-labs-spec-v1.md Section 7.4
- wasm-labs E2E test suite: `tests/e2e/`

### 13.3 Team Contacts

- **WASM Testing Lead**: TBD
- **E2E Infrastructure**: TBD
- **Mutation Testing**: TBD

---

