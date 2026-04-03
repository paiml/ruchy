# Sub-spec: WASM Extreme Quality -- Coverage, FFI, and Multi-Environment Testing

**Parent:** [wasm-extreme-quality.md](../wasm-extreme-quality.md) Sections 1-3

---

## 1. Unified Coverage Strategy

### Configuration Foundation

```toml
# Cargo.toml
[dev-dependencies]
# Core testing infrastructure
wasm-bindgen-test = "0.3"
wasm-pack = "0.12"

# Coverage and quality metrics
cargo-llvm-cov = "0.6"
cargo-mutants = "24.7"

# Property-based testing
proptest = "1.4"
quickcheck = "1.0"
arbitrary = { version = "1.3", features = ["derive"] }

# Browser testing support
web-sys = "0.3"
js-sys = "0.3"

[profile.coverage]
inherits = "test"
opt-level = 0
debug = 2
debug-assertions = true
overflow-checks = true
lto = false
panic = 'abort'
incremental = false
codegen-units = 1  # Deterministic coverage

# Separate profile for WASM testing
[profile.wasm-test]
inherits = "test"
opt-level = "s"  # Size optimization for faster browser loading
```

```toml
# .cargo/config.toml
[target.wasm32-unknown-unknown]
# Runner for local testing only - CI uses explicit wasm-pack
runner = "wasmtime run --dir . --"

# Aliases for common operations
[alias]
wasm-test = "test --target wasm32-unknown-unknown"
coverage = "llvm-cov --branch --fail-under-branches 90"
quick-check = """sh -c '
    cargo fmt --check && 
    cargo clippy -- -D warnings -W clippy::cognitive_complexity
'"""
```

### Branch Coverage Implementation

```bash
#!/bin/bash
# scripts/coverage-unified.sh

set -euo pipefail

# Color output for better readability
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Starting unified coverage collection...${NC}"

# Clean previous artifacts
rm -rf target/coverage
mkdir -p target/coverage

# Phase 1: Native branch coverage
echo -e "${YELLOW}Phase 1: Native testing with branch coverage${NC}"
cargo llvm-cov test \
    --all-features \
    --workspace \
    --branch \
    --ignore-filename-regex '(tests?/|benches/|examples/)' \
    --no-report \
    --output-dir target/coverage/native

# Phase 2: WASM unit tests in wasmtime (fast)
echo -e "${YELLOW}Phase 2: WASM unit tests${NC}"
LLVM_PROFILE_FILE="target/coverage/wasm-%p-%m.profraw" \
cargo llvm-cov test \
    --target wasm32-unknown-unknown \
    --no-report \
    --output-dir target/coverage/wasm

# Phase 3: Browser integration tests (comprehensive)
echo -e "${YELLOW}Phase 3: Browser matrix testing${NC}"
wasm-pack test \
    --headless \
    --chrome \
    --firefox \
    -- --all-features

# Phase 4: Generate unified report with branch analysis
echo -e "${YELLOW}Phase 4: Generating unified report${NC}"
cargo llvm-cov report \
    --lcov \
    --branch \
    --output-path target/coverage/rust.lcov

# Validate branch coverage threshold
BRANCH_COV=$(cargo llvm-cov report --json | jq '.data[0].totals.branches.percent')
if (( $(echo "$BRANCH_COV < 90" | bc -l) )); then
    echo -e "${RED}ERROR: Branch coverage ${BRANCH_COV}% is below 90% threshold${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Coverage collection complete. Branch coverage: ${BRANCH_COV}%${NC}"
```

## 2. JavaScript FFI Bridge Testing

### Test Infrastructure

```javascript
// e2e-tests/package.json
{
  "name": "wasm-ffi-tests",
  "type": "module",
  "scripts": {
    "test": "vitest run",
    "test:coverage": "vitest run --coverage",
    "test:watch": "vitest watch"
  },
  "devDependencies": {
    "@vitest/coverage-istanbul": "^1.2.0",
    "playwright": "^1.40.0",
    "vitest": "^1.2.0"
  }
}
```

```javascript
// e2e-tests/vite.config.js
import { defineConfig } from 'vite';

export default defineConfig({
  test: {
    coverage: {
      provider: 'istanbul',
      reporter: ['text', 'json', 'lcov'],
      branches: 90,
      functions: 95,
      lines: 85,
      statements: 85,
      include: ['src/**/*.js', 'pkg/**/*.js'],
      exclude: ['**/*.test.js', '**/node_modules/**']
    },
    environment: 'jsdom',
    testTimeout: 30000
  }
});
```

### FFI Boundary Test Suite

```typescript
// e2e-tests/ffi-boundary.test.ts
import { describe, test, expect, beforeAll, afterEach } from 'vitest';
import init, { RuchyWasm } from '../pkg/ruchy_wasm';

describe('FFI Boundary Invariants', () => {
  let module: RuchyWasm;

  beforeAll(async () => {
    await init();
    module = new RuchyWasm();
  });

  afterEach(() => {
    // Ensure no memory leaks between tests
    if (module) {
      module.free();
      module = new RuchyWasm();
    }
  });

  test('bidirectional type marshalling', () => {
    const testCases = [
      { input: 42, type: 'number' },
      { input: "test", type: 'string' },
      { input: true, type: 'boolean' },
      { input: new Uint8Array([1, 2, 3]), type: 'Uint8Array' },
      { input: { key: 'value' }, type: 'object' }
    ];

    for (const { input, type } of testCases) {
      const result = module.roundtrip(input);
      expect(typeof result).toBe(type);
      expect(result).toEqual(input);
    }
  });

  test('exception propagation across boundary', () => {
    // Test Rust panic -> JS exception
    expect(() => module.trigger_panic("test panic"))
      .toThrow(/test panic/);

    // Test JS exception -> Rust Result::Err
    const callback = () => { throw new Error('JS error'); };
    const result = module.call_js_callback(callback);
    expect(result.is_err).toBe(true);
    expect(result.error).toContain('JS error');
  });

  test('memory management and GC pressure', async () => {
    const iterations = 1000;
    const size = 1024 * 1024; // 1MB per iteration

    // Baseline heap measurement
    if (global.gc) global.gc();
    const baseline = performance.memory?.usedJSHeapSize || 0;

    // Allocate and free repeatedly
    for (let i = 0; i < iterations; i++) {
      const data = new Uint8Array(size);
      const result = module.process_bytes(data);
      result.free(); // Explicit deallocation
    }

    // Force GC and measure
    if (global.gc) global.gc();
    await new Promise(resolve => setTimeout(resolve, 100));
    
    const final = performance.memory?.usedJSHeapSize || 0;
    const leaked = final - baseline;
    
    // Should not leak more than 10MB after 1GB of allocations
    expect(leaked).toBeLessThan(10 * 1024 * 1024);
  });

  test('concurrent access patterns', async () => {
    const promises = Array.from({ length: 100 }, async (_, i) => {
      const result = await module.async_operation(i);
      return result;
    });

    const results = await Promise.all(promises);
    expect(results).toHaveLength(100);
    expect(new Set(results).size).toBe(100); // All unique
  });
});
```

## 3. Multi-Environment Testing Matrix

### Browser Compatibility Suite

```rust
// tests/browser_compat.rs
#![cfg(all(target_arch = "wasm32", target_os = "unknown"))]

use wasm_bindgen_test::*;
use web_sys::{window, Window, Document, Element};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_browser_api_availability() {
    let window = window().expect("should have window");
    let document = window.document().expect("should have document");
    
    // Test critical APIs exist
    assert!(window.local_storage().is_ok());
    assert!(window.session_storage().is_ok());
    assert!(document.create_element("canvas").is_ok());
    
    // Test WebGL availability
    let canvas: Element = document.create_element("canvas").unwrap();
    let gl_context = canvas.dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap()
        .get_context("webgl2")
        .expect("WebGL2 should be available");
    
    assert!(gl_context.is_some());
}

#[wasm_bindgen_test]
async fn test_async_browser_apis() {
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};
    
    let opts = RequestInit::new();
    opts.set_method("HEAD");
    
    let request = Request::new_with_str_and_init(
        "https://httpbin.org/status/200",
        &opts,
    ).unwrap();
    
    let window = window().unwrap();
    let promise = window.fetch_with_request(&request);
    let response = JsFuture::from(promise).await.unwrap();
    let response: Response = response.dyn_into().unwrap();
    
    assert_eq!(response.status(), 200);
}
```

### Cross-Platform Test Unification

```rust
// tests/unified.rs
// This module demonstrates the pattern for writing tests that run identically
// on both native and WASM targets

#[cfg(all(target_arch = "wasm32", test))]
use wasm_bindgen_test::*;

#[cfg(all(target_arch = "wasm32", test))]
wasm_bindgen_test_configure!(run_in_browser);

// Macro to reduce boilerplate for cross-platform tests
macro_rules! unified_test {
    ($name:ident, $body:expr) => {
        #[cfg_attr(
            all(target_arch = "wasm32", target_os = "unknown"),
            wasm_bindgen_test
        )]
        #[cfg_attr(
            not(all(target_arch = "wasm32", target_os = "unknown")),
            test
        )]
        fn $name() {
            $body
        }
    };
}

unified_test!(test_core_algorithm, {
    let input = vec![1, 2, 3, 4, 5];
    let result = ruchy::algorithms::quicksort(input.clone());
    assert_eq!(result, input); // Already sorted
});

unified_test!(test_memory_safety, {
    // Test that our unsafe optimizations are sound
    let mut buffer = vec![0u8; 1024];
    ruchy::unsafe_ops::zero_memory(&mut buffer);
    assert!(buffer.iter().all(|&x| x == 0));
});
```

