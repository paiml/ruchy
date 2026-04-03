# Sub-spec: HTTP Server — Performance Validation and Competitive Analysis

**Parent:** [http-server-mvp-spec.md](../http-server-mvp-spec.md) Sections 6, 8.5, 9

---

## 6. Performance Targets

**Baseline** (Python http.server):
- Small files (1KB): ~50ms latency, ~20 req/s
- Large files (1MB): ~500ms latency, ~2 req/s
- Concurrent: Single-threaded, blocks on each request

**Ruchy Target** (10-100x improvement):
- Small files (1KB): <1ms latency, >1000 req/s
- Large files (1MB): <10ms latency, >100 req/s
- Concurrent: Async, handles thousands of concurrent connections

**Measurement**:
```bash
# Benchmark with wrk
wrk -t4 -c100 -d30s http://localhost:8080/index.html

# Expected (Ruchy):
# Requests/sec:  10,000+
# Latency avg:   10ms
# Latency p99:   50ms

# Expected (Python):
# Requests/sec:  100-200
# Latency avg:   500ms
# Latency p99:   2000ms
```

## 8.5 Performance Validation & Benchmarking (MANDATORY)

**CRITICAL REQUIREMENT**: Performance claims MUST be empirically validated via built-in benchmarks.

### 8.5.1 Scientific Method Protocol

**NO GUESSING ALLOWED**:
- ❌ FORBIDDEN: "10-100x faster" without proof
- ✅ REQUIRED: Empirical benchmarks with reproducible methodology
- ✅ REQUIRED: Statistical significance (p < 0.05, 95% confidence)
- ✅ REQUIRED: Multiple workloads (small files, large files, many files)

**Minimum Performance Standard**: ≥10X better than what it replaces

### 8.5.2 Built-in Benchmark Command

**CLI Interface**:
```bash
# Run comprehensive benchmark suite
ruchy serve --benchmark

# Benchmark against Python http.server
ruchy serve --benchmark --compare-python

# Benchmark against Node.js http-server
ruchy serve --benchmark --compare-node

# Custom benchmark (specify file size, concurrency, duration)
ruchy serve --benchmark --file-size 1MB --concurrency 100 --duration 30s
```

**Output Format**:
```
🔬 Ruchy HTTP Server Benchmark Suite
====================================

Test: Small Files (1KB, 1000 requests, 10 concurrent)
  Ruchy:         12,543 req/s (avg: 0.8ms, p99: 2.1ms)
  Python:            87 req/s (avg: 114ms, p99: 245ms)
  Node.js:        1,234 req/s (avg: 8.1ms, p99: 18.2ms)

  Performance vs Python:  144x faster ✅ (exceeds 10x minimum)
  Performance vs Node.js:  10x faster ✅ (meets 10x minimum)

Test: Large Files (10MB, 100 requests, 5 concurrent)
  Ruchy:          2,341 req/s (avg: 2.1ms, p99: 5.8ms)
  Python:            12 req/s (avg: 416ms, p99: 892ms)
  Node.js:          234 req/s (avg: 21ms, p99: 54ms)

  Performance vs Python:  195x faster ✅ (exceeds 10x minimum)
  Performance vs Node.js:  10x faster ✅ (meets 10x minimum)

Test: WASM Bundle (2MB .wasm file, 500 requests, 20 concurrent)
  Ruchy:          5,432 req/s (avg: 3.7ms, p99: 9.2ms)
  Python:            18 req/s (avg: 555ms, p99: 1.2s)
  Node.js:          512 req/s (avg: 39ms, p99: 89ms)

  Performance vs Python:  302x faster ✅ (exceeds 10x minimum)
  Performance vs Node.js:  11x faster ✅ (meets 10x minimum)

✅ ALL BENCHMARKS PASS: Minimum 10x improvement achieved
```

### 8.5.3 Benchmark Implementation Requirements

**Workload Scenarios** (MUST test all):
1. **Small Files**: 1KB HTML/CSS/JS (typical web assets)
2. **Medium Files**: 100KB images/fonts
3. **Large Files**: 10MB video/bundles
4. **WASM Files**: 2MB .wasm bundles (critical for modern web)
5. **Many Small Files**: 1000x 1KB requests (concurrency test)
6. **Directory Listing**: 100-file directory (metadata operations)

**Metrics to Measure** (for each workload):
- Throughput (requests/second)
- Latency (mean, p50, p95, p99, max)
- Memory usage (RSS, heap)
- CPU usage (%)
- Time to first byte (TTFB)
- Startup time (cold start)

**Statistical Rigor**:
- Warmup period (5 seconds minimum)
- Multiple runs (≥10 iterations)
- Standard deviation reporting
- Outlier detection and removal
- Confidence intervals (95%)

### 8.5.4 Feature Parity Checklist

**CRITICAL**: Must do EVERYTHING Python/Node does, PLUS work extremely well for WASM.

#### Python http.server Feature Parity

| Feature | Python http.server | Ruchy Status | Notes |
|---------|-------------------|--------------|-------|
| Serve static files | ✅ | ✅ | Core functionality |
| Directory listing | ✅ | 🔄 [HTTP-011] | Sprint 3 |
| MIME type detection | ✅ | 🔄 [HTTP-002] | Sprint 1 |
| Custom port | ✅ | ✅ | Implemented |
| Custom host/interface | ✅ | ✅ | Implemented |
| HEAD requests | ✅ | 🔄 [HTTP-009] | Sprint 2 |
| Range requests | ✅ | 🔄 [HTTP-007] | Sprint 2 |
| If-Modified-Since | ✅ | 🔄 [HTTP-008] | Sprint 2 |
| CGI support | ✅ | ❌ | Out of scope (legacy) |

#### Node.js http-server Feature Parity

| Feature | Node.js http-server | Ruchy Status | Notes |
|---------|---------------------|--------------|-------|
| Serve static files | ✅ | ✅ | Core functionality |
| Directory listing | ✅ | 🔄 [HTTP-011] | Sprint 3 |
| MIME types | ✅ | 🔄 [HTTP-002] | Sprint 1 |
| CORS headers | ✅ | 🔄 [HTTP-004] | Sprint 1 |
| Cache headers | ✅ | 🔄 [HTTP-003] | Sprint 1 |
| Gzip compression | ✅ | ❌ | Phase 2 |
| SSL/TLS | ✅ | ❌ | Phase 2 |
| Proxy support | ✅ | ❌ | Phase 2 |
| Custom headers | ✅ | 🔄 [HTTP-006] | Sprint 2 |

### 8.5.5 WASM-Specific Features (MANDATORY)

**CRITICAL**: Must work extremely well for modern WASM applications.

**WASM Optimizations**:
1. **Content-Type Detection**: Automatic `application/wasm` for .wasm files
2. **WASM Streaming**: Support `Content-Encoding: identity` for streaming compilation
3. **MIME Types**: Complete WASM ecosystem support
   - `.wasm` → `application/wasm`
   - `.wat` → `application/wasm-text`
   - `.wasm.map` → `application/json`
4. **Cross-Origin Headers**: Automatic COOP/COEP headers for SharedArrayBuffer
   ```
   Cross-Origin-Opener-Policy: same-origin
   Cross-Origin-Embedder-Policy: require-corp
   ```
5. **Performance**: WASM bundles served with <5ms latency (critical for load times)
6. **Testing**: Validate with real WASM apps (../wos, ../interactive.paiml.com)

**WASM Benchmark Requirements**:
```bash
# WASM-specific benchmark (MANDATORY)
ruchy serve --benchmark --wasm

# Expected: ≥10x faster than Python for 2MB .wasm files
# Target: <3ms latency for WASM streaming compilation
```

### 8.5.6 Benchmark Automation

**Pre-commit Hook Integration**:
```bash
# Benchmarks run automatically before major releases
make bench

# Output saved to docs/benchmarks/results.md
# CI/CD fails if performance regresses >5%
```

**Continuous Benchmarking**:
- Run nightly benchmarks on dedicated hardware
- Track performance trends over time
- Alert on regressions >5%
- Publish results to docs/benchmarks/

---

## 9. Competitive Positioning

### 9.1 Why Ruchy > Python http.server

**NOTE**: Performance numbers below are TARGETS validated by Section 8.5 benchmarks.

| Feature | Python http.server | Ruchy HTTP Server (Target) |
|---------|-------------------|----------------------------|
| **Performance** | Baseline (see benchmarks) | ≥10x faster (validated) |
| **Latency** | Baseline (see benchmarks) | <5ms (validated) |
| **Memory Safety** | ❌ (C extensions) | ✅ (Rust) |
| **Concurrency** | ❌ (single-threaded) | ✅ (async/await, tokio) |
| **Security** | ⚠️ (path traversal bugs) | ✅ (canonical paths) |
| **Dependencies** | ✅ (stdlib) | ✅ (zero npm/pip) |
| **Startup Time** | Baseline (see benchmarks) | <100ms (validated) |
| **WASM Support** | ❌ (wrong MIME types) | ✅ (COOP/COEP headers) |

**Empirical Validation**: See `ruchy serve --benchmark --compare-python`

### 9.2 Why Ruchy > Node.js http-server

**NOTE**: Performance numbers below are TARGETS validated by Section 8.5 benchmarks.

| Feature | Node.js http-server | Ruchy HTTP Server (Target) |
|---------|---------------------|----------------------------|
| **Performance** | Baseline (see benchmarks) | ≥10x faster (validated) |
| **Memory Usage** | Baseline (see benchmarks) | <10MB (validated) |
| **Dependencies** | ❌ (npm hell, 100+ deps) | ✅ (zero dependencies) |
| **Type Safety** | ⚠️ (TypeScript optional) | ✅ (compile-time) |
| **Security** | ⚠️ (npm supply chain) | ✅ (Rust memory safety) |
| **WASM Support** | ⚠️ (manual CORS setup) | ✅ (automatic COOP/COEP) |

**Empirical Validation**: See `ruchy serve --benchmark --compare-node`

### 9.3 Marketing Message (POST-VALIDATION)

**CRITICAL**: Update this section ONLY AFTER benchmarks validate ≥10x improvement.

**Tagline** (DRAFT - pending validation):
- "The HTTP server Python should have built-in"
- "10X faster than Python, built for modern WASM"
- "Next-generation HTTP serving, built on Rust"

**Elevator Pitch** (DRAFT - pending validation):
> Ruchy HTTP Server is a production-ready static file server built on Rust.
> Benchmarks prove it's ≥10x faster than Python's http.server and Node.js alternatives,
> with first-class WASM support (automatic COOP/COEP headers) and zero dependencies.
> Replace `python3 -m http.server` with `ruchy serve` today.

**Evidence-Based Claims** (ONLY use after validation):
```bash
# Run benchmarks to validate claims
ruchy serve --benchmark

# Example validated output:
# ✅ 144x faster than Python (small files)
# ✅ 195x faster than Python (large files)
# ✅ 302x faster than Python (WASM files)
# ✅ 10-11x faster than Node.js (all workloads)
```

---
