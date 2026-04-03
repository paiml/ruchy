# WASM Quality Testing Specification for Ruchy

**Version**: 1.0.0
**Status**: DRAFT
**Author**: Ruchy Development Team
**Based On**: wasm-labs v1.0.0 E2E Testing Methodology
**Date**: 2025-10-04

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Quality Philosophy](#2-quality-philosophy)
3. [Test Infrastructure Requirements](#3-test-infrastructure-requirements)
4. E2E Browser Testing -- [Sub-spec](sub/wasm-quality-testing-e2e.md)
5. Property-Based Testing -- [Sub-spec](sub/wasm-quality-testing-property-mutation-gates.md)
6. Mutation Testing -- [Sub-spec](sub/wasm-quality-testing-property-mutation-gates.md)
7. Quality Gates -- [Sub-spec](sub/wasm-quality-testing-property-mutation-gates.md)
8. CI/CD Integration -- [Sub-spec](sub/wasm-quality-testing-cicd-roadmap.md)
9. Critical Learnings -- [Sub-spec](sub/wasm-quality-testing-cicd-roadmap.md)
10. Implementation Roadmap -- [Sub-spec](sub/wasm-quality-testing-cicd-roadmap.md)

---

## Sub-spec Index

| Sub-spec | Sections | Description |
|----------|----------|-------------|
| [wasm-quality-testing-e2e.md](sub/wasm-quality-testing-e2e.md) | 4 | E2E Browser Testing with Playwright |
| [wasm-quality-testing-property-mutation-gates.md](sub/wasm-quality-testing-property-mutation-gates.md) | 5-7 | Property-Based Testing, Mutation Testing, Quality Gates |
| [wasm-quality-testing-cicd-roadmap.md](sub/wasm-quality-testing-cicd-roadmap.md) | 8-13 | CI/CD Integration, Learnings, Roadmap, Success Metrics, Maintenance, References |

---

## 1. Executive Summary

### Vision

Establish **world-class WASM quality assurance** for Ruchy compiler's WASM backend using:
- **E2E browser testing** with Playwright (3 browsers)
- **Property-based testing** (10,000+ inputs per invariant)
- **Mutation testing** (>=90% kill rate)
- **Comprehensive coverage** (>=85% line coverage)

### Target Metrics (Based on wasm-labs Success)

| Metric | Target | wasm-labs Achievement | Ruchy Status |
|--------|--------|----------------------|--------------|
| **E2E Tests** | 39+ (13 scenarios x 3 browsers) | 39 passing | TODO |
| **Line Coverage** | >=85% | 87% | 33.34% |
| **Mutation Kill Rate** | >=90% | 99.4% | TODO |
| **Property Tests** | 20+ invariants | 24 tests | TODO |
| **Test Speed** | <10s E2E suite | ~6s | N/A |
| **Cross-Browser** | Chromium, Firefox, WebKit | All passing | TODO |

### Critical Success Factors

**Non-Negotiable Requirements**:
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

**NEVER**:
- Skip E2E tests due to time constraints
- Deploy WASM without all browsers passing
- Use `JsValue::from_str()` for errors (strings, not Error objects)
- Allow flaky tests in test suite
- Disable tests "temporarily"

**ALWAYS**:
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
- **proptest**: Property-based testing (>=0.10)
- **cargo-mutants**: Mutation testing (>=24.0)
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

## Appendix A: Makefile Integration

```makefile
# WASM Quality Testing Targets

.PHONY: e2e-install
e2e-install:
	@echo "Installing Playwright and dependencies..."
	npm ci
	npx playwright install --with-deps
	@echo "E2E dependencies installed"

.PHONY: e2e-install-deps
e2e-install-deps:
	@echo "Installing system dependencies for Playwright..."
	sudo npx playwright install-deps
	@echo "System dependencies installed"

.PHONY: wasm-build
wasm-build:
	@echo "Building WASM..."
	wasm-pack build --target web --out-dir pkg
	@echo "WASM built: pkg/ruchy_bg.wasm"

.PHONY: test-e2e
test-e2e: wasm-build
	@echo "Running E2E tests (39 total: 3 browsers x 13 scenarios)..."
	npm run test:e2e
	@echo "E2E tests passed"

.PHONY: test-e2e-ui
test-e2e-ui: wasm-build
	@echo "Opening Playwright UI..."
	npm run test:e2e:ui

.PHONY: test-e2e-debug
test-e2e-debug: wasm-build
	@echo "Running E2E tests in debug mode..."
	npm run test:e2e:debug

.PHONY: wasm-proptest
wasm-proptest:
	@echo "Running property tests (10,000 cases each)..."
	PROPTEST_CASES=10000 cargo test --target wasm32-unknown-unknown proptest
	@echo "Property tests passed"

.PHONY: wasm-mutation
wasm-mutation:
	@echo "Running mutation tests..."
	cargo mutants --target wasm32-unknown-unknown
	@echo "Mutation testing complete"

.PHONY: wasm-quality-gate
wasm-quality-gate: wasm-test test-e2e wasm-proptest
	@echo "All WASM quality gates passed"
```

---

**End of Specification**

**Status**: DRAFT - Ready for Review
**Next Steps**: Team review -> Implementation Phase 1
