# Matrix Testing Infrastructure - WASM Platform

## Status: DEFERRED (Pending WASM eval() Implementation)

**Current State**:
- ✅ Native platform tests WORKING (42/42 passing)
- ⏸️  WASM platform tests DEFERRED (missing infrastructure)

## Why WASM Tests Are Deferred

Matrix testing requires evaluating Ruchy code and capturing results on BOTH platforms:

**Native Platform** (✅ WORKING):
- Uses `ruchy` CLI with `rexpect` for expect-style testing
- Directly evaluates expressions: `10 + 20` → `30`
- All 8 tests passing in 0.91s

**WASM Platform** (⏸️  BLOCKED):
- Requires `RuchyREPL.eval()` WASM binding (NOT YET IMPLEMENTED)
- Current `ruchy-wasm` only provides `compile()` (transpilation)
- Needs HTML + JS glue code for REPL interface

## Missing Infrastructure

### 1. WASM eval() Binding

`ruchy-wasm/src/lib.rs` needs:

```rust
#[wasm_bindgen]
impl RuchyREPL {
    pub fn new() -> Self { /* ... */ }

    pub fn eval(&mut self, code: &str) -> Result<String, JsValue> {
        // Evaluate Ruchy code and return result as string
        // Similar to native REPL's eval_line()
    }
}
```

### 2. Minimal REPL HTML

`index.html` at project root needs:

```html
<input id="repl-input" />
<div id="output"></div>
<span id="status" class="status-ready">Ready</span>
```

### 3. JavaScript Glue Code

Load WASM, wire up input/output, handle Enter key.

## Implementation Priority

**LOW PRIORITY** because:
1. **Native tests prove the concept** - Matrix testing works
2. **Notebook provides WASM execution** - Already has working WASM infrastructure
3. **High implementation cost** - Requires new WASM bindings + HTML + JS
4. **Better ROI elsewhere** - Focus on native-only matrix runner first

## Recommended Path Forward

### Phase 1 (CURRENT): Native-Only Matrix Testing
- ✅ Native tests working (tests/matrix_001_simple_arithmetic_native.rs)
- Build matrix test runner for native platform only
- Verify behavioral correctness on native first

### Phase 2 (FUTURE): Notebook-Based WASM Verification
- Use existing `ruchy notebook` infrastructure
- Notebook cells already execute WASM code
- Add notebook E2E tests for same operations

### Phase 3 (OPTIONAL): Standalone WASM Matrix Tests
- Implement WASM `eval()` binding
- Create minimal REPL HTML
- Enable full platform parity testing

## Current Test Files

### Matrix Test 001: Simple Arithmetic
- `tests/e2e/matrix/01-simple-arithmetic.spec.ts` - Playwright tests (DEFERRED)
- `tests/matrix_001_simple_arithmetic_native.rs` - Native tests (✅ WORKING, 8/8 passing)

### Matrix Test 002: CSV Processing Workflow
- `tests/e2e/matrix/02-csv-workflow.spec.ts` - Playwright tests (DEFERRED)
- `tests/matrix_002_csv_workflow_native.rs` - Native tests (✅ WORKING, 8/8 passing)

### Matrix Test 003: Statistical Analysis
- `tests/e2e/matrix/03-statistical-analysis.spec.ts` - Playwright tests (DEFERRED)
- `tests/matrix_003_statistical_analysis_native.rs` - Native tests (✅ WORKING, 12/12 passing)

### Matrix Test 004: Time Series Analysis
- `tests/e2e/matrix/04-time-series.spec.ts` - Playwright tests (DEFERRED)
- `tests/matrix_004_time_series_native.rs` - Native tests (✅ WORKING, 14/14 passing)

**Total Native Tests**: 42/42 passing (100%)

## References

- Phase 4 Spec: `docs/specs/PHASE4-NOTEBOOK-EXCELLENCE.md`
- Native Tests: `tests/matrix_001_simple_arithmetic_native.rs`
- WASM Bindings: `ruchy-wasm/src/lib.rs`
