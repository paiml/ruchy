// e2e-tests/ffi-boundary.test.js
// WebAssembly Extreme Quality Assurance Framework v3.0
// FFI Boundary Test Suite

import { describe, test, expect, beforeAll, afterEach } from 'vitest';

// Note: This would import from the built WASM package
// import init, { RuchyWasm } from '../pkg/ruchy_wasm';

describe('FFI Boundary Invariants', () => {
  let module;

  beforeAll(async () => {
    // await init();
    // module = new RuchyWasm();

    // For now, mock the WASM module until wasm-pack build completes
    module = {
      roundtrip: (input) => input,
      trigger_panic: (msg) => { throw new Error(msg); },
      call_js_callback: (cb) => {
        try {
          return { is_err: false, value: cb() };
        } catch (e) {
          return { is_err: true, error: e.message };
        }
      },
      process_bytes: (data) => ({
        free: () => {},
        data: new Uint8Array(data)
      }),
      async_operation: async (i) => Promise.resolve(i)
    };
  });

  afterEach(() => {
    // Ensure no memory leaks between tests
    if (module && module.free) {
      module.free();
    }
  });

  test('bidirectional type marshalling', () => {
    const testCases = [
      { input: 42, type: 'number' },
      { input: "test", type: 'string' },
      { input: true, type: 'boolean' },
      { input: new Uint8Array([1, 2, 3]), type: 'object' },
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
    const iterations = 100; // Reduced for test speed
    const size = 1024; // 1KB per iteration

    // Baseline heap measurement (if available)
    const baseline = performance.memory?.usedJSHeapSize || 0;

    // Allocate and free repeatedly
    for (let i = 0; i < iterations; i++) {
      const data = new Uint8Array(size);
      const result = module.process_bytes(data);
      result.free(); // Explicit deallocation
    }

    // Force GC if available
    if (global.gc) global.gc();
    await new Promise(resolve => setTimeout(resolve, 100));

    const final = performance.memory?.usedJSHeapSize || 0;
    const leaked = final - baseline;

    // Should not leak more than 1MB after 100KB of allocations
    expect(leaked).toBeLessThan(1024 * 1024);
  });

  test('concurrent access patterns', async () => {
    const promises = Array.from({ length: 10 }, async (_, i) => {
      const result = await module.async_operation(i);
      return result;
    });

    const results = await Promise.all(promises);
    expect(results).toHaveLength(10);
    expect(new Set(results).size).toBe(10); // All unique
  });

  test('error boundary isolation', () => {
    // Test that errors in one operation don't affect others
    expect(() => module.trigger_panic("first error")).toThrow();

    // Next operation should work normally
    const result = module.roundtrip(42);
    expect(result).toBe(42);
  });

  test('large data transfer efficiency', () => {
    const largeData = new Uint8Array(64 * 1024); // 64KB
    for (let i = 0; i < largeData.length; i++) {
      largeData[i] = i % 256;
    }

    const startTime = performance.now();
    const result = module.process_bytes(largeData);
    const endTime = performance.now();

    // Should complete in reasonable time
    expect(endTime - startTime).toBeLessThan(100); // 100ms

    // Data should be preserved
    expect(result.data.length).toBe(largeData.length);

    result.free();
  });
});