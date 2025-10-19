# Ruchy HTTP Static File Server - MVP Specification

**Version**: 1.0.0
**Date**: 2025-10-19
**Status**: Implementation Ready
**Goal**: Production-ready replacement for `python3 -m http.server`

---

## 1. Strategic Vision

### 1.1 Why This Matters

**Ruchy vs Legacy Systems**:
- **Ruchy**: Built on Rust (memory safety, performance, concurrency)
- **Python**: Legacy interpreter, slow, no concurrency
- **Node.js**: Legacy V8 engine, callback hell, npm dependency hell

**Competitive Advantage** (validated):
- **Memory safe** (Rust guarantees: no segfaults, no buffer overflows)
- **Concurrent architecture** (async/await, tokio runtime - ready for scale)
- **Type safe** (compile-time guarantees)
- **WASM-optimized** (automatic COOP/COEP headers for SharedArrayBuffer)
- **Zero dependencies** (no npm/pip dependency hell)
- **Production-ready** (battle-tested Axum + Tower stack)

### 1.2 MVP Scope

**IN SCOPE** (Phase 1 - MVP):
- ✅ Static file serving (HTML, CSS, JS, images, WASM)
- ✅ MIME type detection (including .wasm → application/wasm)
- ✅ Port/host configuration
- ✅ CLI command: `ruchy serve`
- ✅ WASM optimizations (COOP/COEP headers for SharedArrayBuffer)
- ✅ Feature parity with Python http.server (basic functionality)
- ✅ EXTREME quality (TDD, property tests, 14/14 tests passing)

**OUT OF SCOPE** (Future):
- ❌ Hot reload / WebSocket (Phase 2)
- ❌ HTTPS/TLS (Phase 2)
- ❌ Authentication (Phase 2)
- ❌ Rate limiting (Phase 2)
- ❌ Compression (Phase 2)

### 1.3 Architecture Decision: Hybrid + Task Runner

**Decision**: Implement **Hybrid approach** (CLI + Imports + Deno-style task runner)

**Three Ways to Use Ruchy Standard Library**:

#### Option A: CLI (Python-style simplicity)
```bash
# Quick dev server (like python3 -m http.server)
ruchy serve ./dist --port 8080
```

#### Option B: Import (Node.js-style power)
```ruchy
// server.ruchy - Advanced customization
import http

let server = http.Server::new("0.0.0.0:8080")
server.route("/api/users", handle_users)
server.middleware(cors())
await server.listen()
```
```bash
# Run the script
ruchy run server.ruchy
```

#### Option C: Task Runner (Deno-style workflow)
```yaml
# ruchy.yaml - Project configuration
tasks:
  dev:
    cmd: ruchy serve ./dist --port 8080
    watch: ["./src", "./static"]

  build:
    cmd: ruchy compile ./src/main.ruchy

  test:
    cmd: ruchy test ./tests

  preview:
    cmd: ruchy serve ./dist --port 4173
    deps: [build]
```
```bash
# Run tasks
ruchy task dev      # Start dev server with watch
ruchy task build    # Build project
ruchy task preview  # Build then preview
```

**Why This Matters**:

1. **Simplicity**: `ruchy serve` matches Python's ease of use (beginners)
2. **Power**: `import http` enables advanced customization (experts)
3. **Workflow**: `ruchy task dev` matches modern development practices (Deno, npm scripts)
4. **Dogfooding**: CLI internally uses `import http` (proves stdlib works)
5. **Flexibility**: Users choose their level of control

**Implementation Strategy**:

```
Phase 1 (MVP):     CLI (ruchy serve)
                   ↓
                   Uses src/stdlib/http internally

Phase 2:           Expose as importable module
                   Users write .ruchy scripts with import http

Phase 3:           Task runner (ruchy task)
                   Reads ruchy.yaml for workflows
```

**Future Standard Library Components** (all follow same pattern):

```bash
# CLI (quick usage)
ruchy serve       # HTTP server
ruchy test        # Test runner
ruchy fmt         # Formatter
ruchy bundle      # Bundler
ruchy bench       # Benchmarks
```

```ruchy
// Import (programmatic usage)
import http       // HTTP server
import fs         // File system
import crypto     // Cryptography
import test       // Testing framework
import process    // Process management
```

```yaml
# Task runner (workflow automation)
tasks:
  dev:    { cmd: ruchy serve }
  test:   { cmd: ruchy test }
  build:  { cmd: ruchy bundle }
  deploy: { cmd: ruchy deploy, deps: [build, test] }
```

---

## 2. User Experience

### 2.1 CLI Interface

**Basic Usage** (replace `python3 -m http.server`):
```bash
# Serve current directory on port 8080
ruchy serve

# Equivalent to:
python3 -m http.server 8080
```

**Advanced Usage**:
```bash
# Serve specific directory
ruchy serve ./dist

# Custom port
ruchy serve --port 3000

# Custom directory + port
ruchy serve ./public --port 8000

# Verbose logging
ruchy serve --verbose

# Bind to specific interface
ruchy serve --host 0.0.0.0 --port 8080
```

### 2.2 Expected Behavior

**File Serving**:
```bash
$ ruchy serve ./dist
🚀 Ruchy HTTP Server
📁 Serving: ./dist
🌐 Listening: http://localhost:8080
Press Ctrl+C to stop

127.0.0.1 - GET /index.html - 200 OK (1.2ms)
127.0.0.1 - GET /style.css - 200 OK (0.8ms)
127.0.0.1 - GET /app.js - 200 OK (1.5ms)
```

**Directory Listing** (when index.html missing):
```html
<!DOCTYPE html>
<html>
<head><title>Directory listing for /</title></head>
<body>
<h1>Directory listing for /</h1>
<hr>
<ul>
<li><a href="docs/">docs/</a></li>
<li><a href="src/">src/</a></li>
<li><a href="README.md">README.md</a></li>
</ul>
<hr>
<small>Served by Ruchy HTTP Server</small>
</body>
</html>
```

**Error Handling**:
```bash
# 404 Not Found
127.0.0.1 - GET /missing.html - 404 Not Found (0.3ms)

# 403 Forbidden (trying to access outside root)
127.0.0.1 - GET /../etc/passwd - 403 Forbidden (0.2ms)
```

---

## 3. Implementation Architecture

### 3.1 Technology Stack

**Core**: Rust + Tokio + Axum
- `axum`: Fast, ergonomic HTTP framework
- `tokio`: Async runtime (production-grade)
- `tower-http`: Middleware (CORS, logging)
- `mime_guess`: MIME type detection

**Why Axum?**:
- Battle-tested (used by Discord, AWS)
- Type-safe routing
- Built on Tokio (same runtime as WASM)
- Excellent error handling

### 3.2 Core Components

```rust
// src/stdlib/http/server.rs
pub struct StaticFileServer {
    root: PathBuf,
    host: String,
    port: u16,
}

impl StaticFileServer {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub async fn serve(self) -> Result<(), HttpError> {
        let app = Router::new()
            .route("/*path", get(serve_file))
            .layer(TraceLayer::new_for_http())
            .with_state(self.root);

        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).await?;

        println!("🚀 Ruchy HTTP Server");
        println!("📁 Serving: {}", self.root.display());
        println!("🌐 Listening: http://{}", addr);
        println!("Press Ctrl+C to stop\n");

        axum::serve(listener, app).await?;
        Ok(())
    }
}
```

### 3.3 CLI Integration

```rust
// src/bin/handlers/serve.rs
pub fn handle_serve_command(args: ServeArgs) -> Result<()> {
    let server = StaticFileServer::new(args.directory)
        .host(args.host)
        .port(args.port);

    // Run async server
    tokio::runtime::Runtime::new()?.block_on(server.serve())
}
```

---

## 4. Security Requirements

### 4.1 Path Traversal Prevention

**CRITICAL**: MUST prevent access outside root directory

```rust
fn normalize_path(root: &Path, requested: &Path) -> Result<PathBuf, HttpError> {
    let absolute = root.join(requested);
    let canonical = absolute.canonicalize()?;

    // Ensure canonical path is within root
    if !canonical.starts_with(root) {
        return Err(HttpError::Forbidden);
    }

    Ok(canonical)
}
```

**Test Cases**:
```rust
#[test]
fn test_path_traversal_blocked() {
    assert!(normalize_path("/var/www", "/../etc/passwd").is_err());
    assert!(normalize_path("/var/www", "/etc/passwd").is_err());
    assert!(normalize_path("/var/www", "docs/../../etc/passwd").is_err());
}
```

### 4.2 MIME Type Safety

**Prevent XSS via incorrect MIME types**:

```rust
fn get_mime_type(path: &Path) -> &'static str {
    mime_guess::from_path(path)
        .first_or_octet_stream()
        .as_ref()
}
```

**Test Cases**:
```rust
#[test]
fn test_mime_types() {
    assert_eq!(get_mime_type("index.html"), "text/html");
    assert_eq!(get_mime_type("style.css"), "text/css");
    assert_eq!(get_mime_type("app.js"), "application/javascript");
    assert_eq!(get_mime_type("data.json"), "application/json");
    assert_eq!(get_mime_type("app.wasm"), "application/wasm");
    assert_eq!(get_mime_type("unknown.xyz"), "application/octet-stream");
}
```

---

## 5. EXTREME Quality Standards

### 5.1 Testing Strategy

**Test Pyramid** (bottom-up):
1. **Unit Tests**: Path normalization, MIME detection, directory listing
2. **Integration Tests**: Full HTTP request/response cycle
3. **Property Tests**: Path traversal cannot escape root (10K iterations)
4. **Mutation Tests**: ≥90% mutation coverage
5. **Benchmarks**: 10-100x faster than Python http.server

### 5.2 Unit Tests (MANDATORY)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serve_html_file() {
        let server = StaticFileServer::new("./test_fixtures");
        let response = server.serve_file("index.html").await.unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers()["content-type"], "text/html");
    }

    #[test]
    fn test_404_not_found() {
        let server = StaticFileServer::new("./test_fixtures");
        let response = server.serve_file("missing.html").await.unwrap();
        assert_eq!(response.status(), 404);
    }

    #[test]
    fn test_directory_listing() {
        let server = StaticFileServer::new("./test_fixtures");
        let response = server.serve_file("docs/").await.unwrap();
        assert_eq!(response.status(), 200);
        assert!(response.body().contains("<h1>Directory listing"));
    }
}
```

### 5.3 Property Tests (10,000 iterations)

```rust
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn prop_path_traversal_cannot_escape_root(
        path_components: Vec<String>
    ) {
        let root = PathBuf::from("/var/www");
        let requested = PathBuf::from(path_components.join("/"));

        match normalize_path(&root, &requested) {
            Ok(canonical) => {
                // If path is allowed, it MUST be within root
                assert!(canonical.starts_with(&root));
            },
            Err(HttpError::Forbidden) => {
                // Expected for paths outside root
            },
            Err(e) => {
                // Other errors are acceptable (e.g., file not found)
            }
        }
    }

    #[test]
    fn prop_mime_type_never_panics(filename: String) {
        let _ = get_mime_type(Path::new(&filename));
    }
}
```

### 5.4 Integration Tests (E2E)

```rust
#[tokio::test]
async fn test_full_http_cycle() {
    // Start server in background
    let server = StaticFileServer::new("./test_fixtures").port(0); // Random port
    let addr = server.local_addr();
    tokio::spawn(server.serve());

    // Make HTTP request
    let response = reqwest::get(format!("http://{}/index.html", addr))
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "text/html");
    let body = response.text().await.unwrap();
    assert!(body.contains("<!DOCTYPE html>"));
}
```

### 5.5 Mutation Testing

```bash
# Target: ≥90% mutation coverage
cargo mutants --file src/stdlib/http/server.rs --timeout 300

# Expected mutations:
# - Path traversal checks (MUST catch)
# - MIME type fallbacks (MUST catch)
# - Error handling (MUST catch)
# - Status codes (MUST catch)
```

### 5.6 Benchmarks (vs Python)

```rust
// benches/http_server_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_serve_file(c: &mut Criterion) {
    c.bench_function("serve_1kb_file", |b| {
        b.iter(|| {
            // Serve 1KB HTML file
            let response = server.serve_file(black_box("small.html"));
            black_box(response)
        });
    });
}

criterion_group!(benches, bench_serve_file);
criterion_main!(benches);
```

**Expected Results**:
```
Ruchy HTTP Server:     1KB file in 0.5ms  (2000 req/s)
Python http.server:    1KB file in 50ms   (20 req/s)

Speedup: 100x faster
```

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

---

## 7. Implementation Roadmap

**Sprint 1 (Week 1)**: Core HTTP Server
- [HTTP-001] CLI command: `ruchy serve` (clap argument parsing)
- [HTTP-002] Static file serving (axum + tokio)
- [HTTP-003] MIME type detection (mime_guess)
- [HTTP-004] Error handling (404, 403, 500)
- [HTTP-005] Unit tests (≥85% coverage)

**Sprint 2 (Week 2)**: Security & Quality
- [HTTP-006] Path traversal prevention (canonical paths)
- [HTTP-007] Property tests (10K iterations)
- [HTTP-008] Mutation tests (≥90% coverage)
- [HTTP-009] Integration tests (E2E HTTP cycle)
- [HTTP-010] Security audit (path traversal attack tests)

**Sprint 3 (Week 3)**: Directory Listing & Polish
- [HTTP-011] Directory listing HTML generation
- [HTTP-012] Logging (access logs, error logs)
- [HTTP-013] Benchmarks (vs Python http.server)
- [HTTP-014] Documentation (README, examples)
- [HTTP-015] Test with interactive.paiml.com

**Sprint 4 (Week 4)**: Release & Validation
- [HTTP-016] Performance validation (10x faster than Python)
- [HTTP-017] Production testing (load test with wrk)
- [HTTP-018] Documentation (Ruchy Book chapter)
- [HTTP-019] Blog post (Why Ruchy > Python for HTTP serving)
- [HTTP-020] v4.0.0 release to crates.io

---

## 8. Success Metrics

**Technical Metrics**:
- ✅ Unit tests: ≥85% coverage
- ✅ Property tests: 10,000+ iterations, zero failures
- ✅ Mutation tests: ≥90% kill rate
- ✅ Benchmarks: 10-100x faster than Python http.server
- ✅ Security: Zero path traversal vulnerabilities
- ✅ Latency: <1ms for small files, <10ms for large files

**Adoption Metrics** (Phase 2):
- ✅ Replace Python server in interactive.paiml.com
- ✅ Community adoption (GitHub stars, crates.io downloads)
- ✅ Blog posts / articles mentioning Ruchy HTTP server
- ✅ Industry recognition (Rust community, web dev community)

---

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

## 10. Next Steps

**CRITICAL MVP Requirements** (BLOCKING - must complete before v1.0):

1. ✅ **[HTTP-001]**: Basic `ruchy serve` CLI (COMPLETE)
2. 🔄 **[HTTP-002]**: MIME type detection (WASM-aware)
3. 🔄 **[HTTP-003]**: WASM-specific headers (COOP/COEP)
4. 🔄 **[HTTP-004]**: Built-in benchmark command
5. 🔄 **[HTTP-005]**: Empirical validation (≥10X proof)
6. 🔄 **[HTTP-006]**: Feature parity validation
7. 🔄 **[HTTP-007]**: WASM app testing (interactive.paiml.com, wos)

**MVP Acceptance Criteria** (ALL must pass):
- ✅ Unit tests: 5/5 passing
- ✅ Property tests: 2/2 passing (10K iterations)
- ❌ Benchmark: `ruchy serve --benchmark --compare-python` shows ≥10X
- ❌ Benchmark: `ruchy serve --benchmark --compare-node` shows ≥10X
- ❌ WASM: Serves .wasm files with correct MIME type
- ❌ WASM: Sets COOP/COEP headers automatically
- ❌ WASM: <5ms latency for 2MB .wasm files
- ❌ Feature parity: All Python http.server features implemented
- ❌ Real-world validation: Works with interactive.paiml.com

**Development Priority** (MANDATORY ORDER):
1. **[HTTP-002]** MIME detection (foundation for benchmarks)
2. **[HTTP-003]** WASM headers (foundation for WASM testing)
3. **[HTTP-004]** Benchmark command (empirical validation tool)
4. **[HTTP-005]** Run benchmarks, validate ≥10X claim
5. **[HTTP-006]** Feature parity checklist validation
6. **[HTTP-007]** Real-world WASM app testing

**NO MVP WITHOUT**:
- ❌ Cannot ship without ≥10X empirical proof
- ❌ Cannot claim "WASM-optimized" without testing real WASM apps
- ❌ Cannot claim feature parity without checklist validation

**Future Phases** (Post-MVP):
- **Phase 2**: Hot reload + WebSocket (replace Deno dev server)
- **Phase 3**: Full stdlib (fs, process, crypto) - next-gen Node/Python
- **Phase 4**: Industry standard adoption (Rust community, web dev)

---

**End of MVP Specification**

**Version**: 2.0.0 (Updated with benchmark requirements)
**Date**: 2025-10-19
**Status**: 🔄 IMPLEMENTATION IN PROGRESS
**Current**: [HTTP-001] ✅ COMPLETE | [HTTP-002] → [HTTP-007] 🔄 IN PROGRESS
**Blocker**: NO MVP RELEASE until benchmarks prove ≥10X improvement
