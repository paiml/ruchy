# Sub-spec: Probar (jugar-probar) as First-Class Testing Framework

**Parent:** [docs/SPECIFICATION.md](../SPECIFICATION.md)
**Ticket:** PROBAR-001
**Status:** Proposed
**Priority:** High
**Version:** 1.0.0
**Date:** 2026-04-03

---

## 0. Prerequisites

- **Current**: jugar-probar 1.0.2, **target**: 1.0.4 (latest on crates.io)
- **`#[probar_test]` macro must be created** -- this attribute macro does not exist in jugar-probar; it must be implemented either as a proc-macro crate or as synthetic code generation by the Ruchy transpiler
- **Playbook codegen** (YAML-to-Rust) requires proc-macro or `build.rs` infrastructure to read YAML at compile time and expand function bodies
- **"Zero-JS" clarification**: wasm-bindgen generates minimal JS glue code (e.g., `pkg/ruchy_wasm.js`); "zero-JS" means no user-written JavaScript, not zero JS files in build output

## 1. Overview

### 1.1 Current State

Probar (`jugar-probar = "1.0.2"`) is a dev-dependency used for WASM API coverage
tracking. Existing integration is limited to three test files and two examples:

| File | Purpose |
|------|---------|
| `ruchy-wasm/tests/probar_wasm_tests.rs` | GUI coverage tracking of `RuchyCompiler` API |
| `tests/probar_wasm_qa.rs` | Parse/transpile coverage with `UxCoverageTracker` |
| `tests/probar_worker_harness.rs` | WorkerBrick lifecycle and distributed execution |
| `examples/probar_distributed.rs` | BackendSelector and BrickCoordinator demo |
| `examples/probar_worker_brick.rs` | WorkerBrick code generation demo |

Usage follows a manual pattern: test authors call `UxCoverageBuilder::new()`,
register elements, and call `.visit()` / `.interact()` within each test. The
transpiler has no awareness of probar; coverage tracking is bolted on, not built in.

### 1.2 Target State

Probar becomes the native testing substrate for Ruchy:

1. **Declarative test syntax** -- `#[probar_test]` macro with automatic coverage injection.
2. **GUI coverage as language feature** -- the transpiler emits `UxCoverageTracker` calls.
3. **Brick architecture enforcement** -- assertions-first design where tests define the interface.
4. **Playbook state machines** -- YAML-driven deterministic test scenarios with codegen.

### 1.3 Probar Capabilities (v1.0.4)

| Property | Value |
|----------|-------|
| Language | Pure Rust |
| User-written JavaScript | Zero (wasm-bindgen glue excluded) |
| Playwright compatibility | Full API parity |
| Modules | 24+ (`brick`, `assertions`, `locator`, `worker_harness`, ...) |
| Locator strategies | CSS, test ID, text, entity, role, label |
| Coverage model | Element + State + Interaction |
| Architecture | Brick -- tests ARE the interface |

### 1.4 Design Principle

Ruchy users write tests in Ruchy syntax. The transpiler lowers `test` blocks to Rust
code that calls probar APIs, injects coverage tracking, and enforces Brick budgets.
No manual `UxCoverageBuilder` wiring; the compiler handles it.

## 2. Declarative Test Syntax

### 2.1 `#[probar_test]` Macro Semantics

> **Implementation note:** The `#[probar_test]` attribute macro does not currently exist in jugar-probar. It must be created -- either as a standalone proc-macro crate (e.g., `probar-macros`) or as synthetic attribute expansion within the Ruchy transpiler. The transpiler approach is preferred since it avoids an external proc-macro dependency and keeps all code generation in one place.

The `#[probar_test]` attribute on a Ruchy test function triggers three compiler
behaviors:

1. **Coverage injection** -- the transpiler wraps the test body with
   `UxCoverageTracker` setup/teardown and auto-visits every element referenced
   by locator calls.
2. **Assertion enrichment** -- `assert_eq` and `assert_contains` are rewritten
   to probar's `BrickAssertions` with structured error messages including
   element context.
3. **Timeout enforcement** -- every `#[probar_test]` runs under a configurable
   timeout (default 5000 ms), preventing zombie test processes.

### 2.2 Locator API

Ruchy exposes probar's six locator strategies as built-in functions within
`#[probar_test]` blocks:

| Locator | Ruchy Syntax | Transpiled Rust |
|---------|-------------|-----------------|
| CSS selector | `css(".btn-primary")` | `Locator::css(".btn-primary")` |
| Test ID | `test_id("submit")` | `Locator::test_id("submit")` |
| Text content | `text("Click me")` | `Locator::text("Click me")` |
| Entity name | `entity("compiler")` | `Locator::entity("compiler")` |
| ARIA role | `role("button")` | `Locator::role("button")` |
| Label | `label("Username")` | `Locator::label("Username")` |

### 2.3 Assertion Modes

```ruchy
#[probar_test]
fun test_compiler_output():
    let result = compile("fun main(): print(42)")

    # Hard assertion -- test fails immediately
    assert_eq(result.exit_code, 0)

    # Soft assertion -- collects failures, reports at end
    soft_assert(result.output.contains("42"))
    soft_assert(result.warnings.is_empty())

    # Retry assertion -- polls up to N ms with interval
    assert_retry(result.ready(), timeout=3000, interval=100)
```

Transpiled output wraps soft assertions in `SoftAssertionCollector` and retry
assertions in a polling loop with `std::thread::sleep`.

### 2.4 Coverage Tracking Injection

Given this Ruchy test:

```ruchy
#[probar_test]
fun test_parse_function():
    let ast = parse("fun foo(): return 1")
    assert_eq(ast.kind, "FunctionDecl")
```

The transpiler emits:

```rust
#[test]
fn test_parse_function() {
    let mut _coverage = UxCoverageBuilder::new()
        .button("parse")
        .screen("ast_generation")
        .build();
    _coverage.visit_screen("ast_generation");
    _coverage.interact_button("parse");

    let ast = parse("fun foo(): return 1");
    assert_eq!(ast.kind, "FunctionDecl");

    let report = _coverage.report();
    assert!(report.coverage_percent() >= 85.0,
        "GUI coverage {:.1}% below 85% threshold", report.coverage_percent());
}
```

## 3. Playbook State Machine Tests

### 3.1 YAML Playbook Format

Playbooks define deterministic test scenarios as state machines. Each playbook
specifies an initial state, a sequence of actions with assertions, and
transitions to subsequent states.

```yaml
playbook: compiler_e2e
seed: 42
timeout_ms: 10000

states:
  - name: source_ready
    initial: true
    actions:
      - action: parse
        input: "fun add(a: int, b: int) -> int: return a + b"
        verify:
          - assert_eq: [ast.functions.len(), 1]
          - assert_eq: [ast.functions[0].name, "add"]
        next: parsed

  - name: parsed
    actions:
      - action: transpile
        verify:
          - assert_contains: [output, "fn add"]
          - assert_eq: [errors.len(), 0]
        next: transpiled

  - name: transpiled
    actions:
      - action: compile
        verify:
          - assert_eq: [exit_code, 0]
        next: compiled

  - name: compiled
    actions:
      - action: execute
        args: ["3", "4"]
        verify:
          - assert_eq: [stdout.trim(), "7"]
        next: done

  - name: done
    terminal: true
```

### 3.2 Codegen Pipeline

> **Implementation note:** YAML-to-Rust codegen requires either a `build.rs` script that reads playbook YAML files and generates Rust test functions at compile time, or a proc-macro that expands the `#[playbook("path")]` attribute. The `build.rs` approach is simpler and avoids proc-macro complexity; it should write generated test files to `$OUT_DIR` and include them via `include!()`.

```
playbook.yaml  -->  ruchy codegen  -->  #[probar_test] functions  -->  rustc
```

Each state becomes a test function. The `seed` field controls deterministic
replay: given the same seed, the playbook produces identical execution order and
timing. This guarantees any failure is reproducible without flakiness.

### 3.3 Ruchy Syntax for Playbooks

```ruchy
#[playbook("tests/playbooks/compiler_e2e.yaml")]
fun test_compiler_pipeline():
    # Body auto-generated from YAML states
    pass
```

The transpiler reads the YAML at compile time and expands the function body
into sequential state transitions with assertions.

## 4. Coverage Integration

### 4.1 Coverage Model

Probar tracks three orthogonal coverage dimensions:

| Dimension | What It Measures | Metric |
|-----------|-----------------|--------|
| Element coverage | Which UI/API elements are exercised | % of registered elements visited |
| State coverage | Which states each element passes through | % of state transitions observed |
| Interaction coverage | Which user actions are performed | % of registered interactions fired |

### 4.2 UxCoverageTracker as Language Feature

The transpiler auto-generates `UxCoverageTracker` registration based on static
analysis of the module under test:

- Public functions become `button` elements.
- Modules become `screen` elements.
- Public struct fields become `input` elements.
- Enum variants become `state` transitions.

This removes the manual `UxCoverageBuilder` boilerplate present in current tests.

### 4.3 Pixel Coverage Heatmaps

For WASM UI tests, probar generates pixel-level heatmaps showing which regions
of the rendered interface were exercised. The heatmap overlays interaction
density on a screenshot baseline, highlighting untested regions in red.

Output format: PNG with alpha overlay, stored in `target/probar/heatmaps/`.

### 4.4 Coverage Thresholds

| Category | Threshold | Enforcement |
|----------|-----------|-------------|
| Compiler public API methods | 100% | CI gate, hard fail |
| Error path coverage | 95% | CI gate, hard fail |
| UI element coverage | 85% | CI gate, warning at 80% |
| State transition coverage | 80% | CI gate, warning at 75% |
| Pixel coverage (WASM) | 70% | Advisory, no gate |

Thresholds are configured in `probar.toml` at the project root and enforced
by `ruchy test --probar`.

## 5. Brick Architecture Enforcement

### 5.1 BrickAssertions

Brick assertions verify non-functional properties alongside correctness:

| Assertion | Parameters | What It Validates |
|-----------|-----------|-------------------|
| `TextVisible` | `(selector)` | Text content is rendered and not clipped |
| `ContrastRatio` | `(selector, min_ratio)` | WCAG contrast ratio meets threshold (default 4.5:1) |
| `MaxLatencyMs` | `(operation, max_ms)` | Operation completes within time budget (default 16 ms) |
| `ElementPresent` | `(selector)` | Element exists in the DOM/component tree |
| `Focusable` | `(selector)` | Element can receive keyboard focus (a11y) |
| `NoLayoutShift` | `(selector)` | Element does not cause cumulative layout shift |
| `MemoryBound` | `(operation, max_bytes)` | Operation stays within memory allocation budget |

### 5.2 BrickBudget

A `BrickBudget` defines time limits for the three rendering phases:

```ruchy
#[probar_test]
#[brick_budget(measure=4, layout=4, paint=8)]
fun test_ast_visualization():
    let viz = render_ast("fun main(): print(1)")
    assert_brick(MaxLatencyMs("render_ast", 16))
    assert_brick(ElementPresent(css("#ast-tree")))
    assert_brick(TextVisible(css(".node-label")))
```

The transpiler emits timing instrumentation around each phase. If any phase
exceeds its budget, the test fails with a structured report showing measured
vs. allowed time.

### 5.3 BrickHouse Composition

A `BrickHouse` composes multiple Bricks under a shared budget:

```ruchy
#[probar_test]
#[brick_house(total_budget_ms=32)]
fun test_editor_composition():
    let editor = compose([
        brick("syntax_highlighter", budget_ms=8),
        brick("line_numbers", budget_ms=4),
        brick("minimap", budget_ms=8),
        brick("scrollbar", budget_ms=4),
    ])
    assert_eq(editor.total_budget_ms(), 24)  # 8ms slack
    assert_brick(MaxLatencyMs("compose", 32))
```

### 5.4 JIDOKA: Fail-Fast on Violation

Following the Toyota Way, any BrickAssertion violation triggers immediate test
termination with full diagnostic output. The test does not continue collecting
soft failures -- a Brick violation is a stop-the-line event.

Diagnostics include:
- Measured value vs. threshold
- Stack trace to the violating operation
- Screenshot at time of failure (WASM tests)
- Suggested fix based on violation category

## 6. Visual Regression Testing

### 6.1 Baseline Capture

On first run, `ruchy test --visual-regression` captures baseline screenshots
for every `#[probar_test]` that renders output. Baselines are stored in
`tests/visual-baselines/` with deterministic filenames derived from test name
and viewport dimensions (e.g., `test_ast_visualization_1280x720.png`).

### 6.2 Perceptual Diffing

Subsequent runs compare rendered output against baselines using perceptual
hashing (pHash). Configurable per-test via `#[visual_threshold(max_diff=0.02)]`.

| Threshold | Meaning | Action |
|-----------|---------|--------|
| < 1% diff | Sub-pixel antialiasing | Pass |
| 1-5% diff | Minor rendering change | Warning, review required |
| > 5% diff | Significant regression | Fail |

### 6.3 Cross-Browser Matrix and Artifacts

For WASM targets, visual regression runs across Chromium (Blink), Firefox
(Gecko), and WebKit via Docker containers (`playwright/*:latest`).

Failed tests produce three artifacts in `target/probar/artifacts/`: screenshot
PNG, diff image highlighting regressions in red, and WebM video recording
(WASM tests only).

## 7. CLI Commands

### 7.1 Command Summary

| Command | Purpose |
|---------|---------|
| `ruchy test --probar` | Run all `#[probar_test]` functions with GUI coverage |
| `ruchy test --playbook <file.yaml>` | Execute a state machine playbook |
| `ruchy test --visual-regression` | Run visual baseline comparison |
| `ruchy test --mutations` | Mutation testing quality validation |
| `ruchy test --probar --coverage-report` | Generate HTML coverage report |
| `ruchy test --probar --brick-budget` | Enforce Brick timing budgets |

### 7.2 Usage Examples

```bash
# Run probar tests with coverage enforcement from probar.toml
ruchy test --probar

# Execute a playbook with deterministic seed for replay
ruchy test --playbook tests/playbooks/compiler_e2e.yaml --seed 42

# Visual regression: compare or update baselines
ruchy test --visual-regression
ruchy test --visual-regression --update-baselines
```

## 8. WASM Boundary Testing

### 8.1 Zero-JS Validation

> **Clarification:** "Zero-JS" means no **user-written** JavaScript, not zero JS files in the build output. `wasm-bindgen` necessarily generates minimal JS glue (e.g., `pkg/ruchy_wasm.js`) for WASM initialization and JS interop. The validator allowlists these generated glue files.

Probar enforces that the Ruchy-to-WASM compilation chain produces no user-written
JavaScript. The validator scans the build output directory for `.js` files
that are not explicitly allowlisted:

```ruchy
#[probar_test]
#[zero_js]
fun test_wasm_purity():
    let artifacts = build_wasm("fun main(): print(1)")
    assert_eq(artifacts.js_files().len(), 0)
    assert(artifacts.wasm_file().exists())
```

Allowlist entries (e.g., `wasm-bindgen` glue) are declared in `probar.toml`:

```toml
[wasm.zero_js]
allowlist = ["pkg/ruchy_wasm.js"]  # wasm-bindgen entry point
```

### 8.2 Thread Capability Detection

WASM environments vary in threading support. Probar detects capabilities at
test startup and gates thread-dependent tests accordingly:

| Capability | Detection | Gate |
|------------|-----------|------|
| `SharedArrayBuffer` | Feature detection | Skip if absent |
| `Atomics` | Feature detection | Skip if absent |
| Web Workers | Spawn + message round-trip | Skip if timeout |
| WASM threads | `wasm32` target feature check | Skip if absent |

### 8.3 Web Worker Lifecycle Testing

Probar's `WorkerTestHarness` validates the full worker lifecycle:

```ruchy
#[probar_test]
fun test_worker_lifecycle():
    let harness = WorkerTestHarness::new(WorkerTestConfig {
        worker_count: 4,
        message_count: 100,
        timeout_ms: 5000,
    })
    harness.test_lifecycle()       # spawn -> ready -> busy -> idle -> terminate
    harness.test_message_ordering() # FIFO guarantee
    harness.test_error_recovery()   # crash -> restart -> resume
```

### 8.4 WASM Compliance Linting

The `--wasm-lint` flag performs static analysis on generated WASM to detect
common issues:

| Lint | Description | Severity |
|------|-------------|----------|
| `stale-state-sync` | State read without preceding sync barrier | Error |
| `unbounded-alloc` | Allocation in hot loop without pool | Warning |
| `missing-error-propagation` | WASM trap not converted to Result | Error |
| `js-interop-leak` | JS object reference not freed | Warning |

## 9. Testing Requirements

### 9.1 Coverage Targets

| Metric | Minimum | Stretch |
|--------|---------|---------|
| Compiler public API coverage | 100% | -- |
| Error path coverage | 95% | 98% |
| UI element coverage | 85% | 90% |
| Mutation testing score | 75% | 85% |
| Playbook state coverage | 100% | -- |

### 9.2 Deterministic Replay

Every test failure must be reproducible via seed replay. The test runner
captures the seed for each run and prints it on failure:

```
FAILED: test_compiler_pipeline (seed=8273651)
  Replay: ruchy test --playbook compiler_e2e.yaml --seed 8273651
```

### 9.3 Mutation Testing Integration

`ruchy test --mutations` runs `cargo-mutants` against all `#[probar_test]`
functions. The mutation score must exceed 75% (CAUGHT / (CAUGHT + MISSED)).
Surviving mutants are reported with source location and suggested test.

### 9.4 Acceptance Criteria

This spec is complete when:

1. `#[probar_test]` macro compiles and injects coverage tracking.
2. At least one YAML playbook exercises the full parse-transpile-compile-execute pipeline.
3. `ruchy test --probar` enforces coverage thresholds from `probar.toml`.
4. BrickBudget timing instrumentation produces accurate measurements within 1 ms.
5. Visual regression captures baselines and detects diffs above threshold.
6. Zero-JS validation passes for the standard WASM build.
7. All existing `probar_wasm_tests.rs` tests migrate to `#[probar_test]` syntax.
8. Mutation testing score for probar-annotated tests exceeds 75%.
