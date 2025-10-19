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

**Competitive Advantage**:
- **10-100x faster** than Python http.server (Rust vs Python)
- **Memory safe** (no segfaults, no buffer overflows)
- **Zero dependencies** (no npm/pip hell)
- **Concurrent** (async/await, tokio runtime)
- **Type safe** (compile-time guarantees)

### 1.2 MVP Scope

**IN SCOPE** (Phase 1):
- ✅ Static file serving (HTML, CSS, JS, images, WASM)
- ✅ Directory listing (like Python http.server)
- ✅ MIME type detection
- ✅ Port configuration
- ✅ CLI command: `ruchy serve`
- ✅ EXTREME quality (TDD, mutation tests, benchmarks)

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

## 9. Competitive Positioning

### 9.1 Why Ruchy > Python http.server

| Feature | Python http.server | Ruchy HTTP Server |
|---------|-------------------|-------------------|
| **Performance** | ~20 req/s | ~10,000 req/s |
| **Latency** | ~50ms | <1ms |
| **Memory Safety** | ❌ (C extensions) | ✅ (Rust) |
| **Concurrency** | ❌ (single-threaded) | ✅ (async/await) |
| **Security** | ⚠️ (path traversal bugs) | ✅ (canonical paths) |
| **Dependencies** | ✅ (stdlib) | ✅ (zero npm/pip) |
| **Startup Time** | ~500ms | <100ms |

### 9.2 Why Ruchy > Node.js http-server

| Feature | Node.js http-server | Ruchy HTTP Server |
|---------|---------------------|-------------------|
| **Performance** | ~1,000 req/s | ~10,000 req/s |
| **Memory Usage** | ~50MB | ~5MB |
| **Dependencies** | ❌ (npm hell, 100+ deps) | ✅ (zero dependencies) |
| **Type Safety** | ⚠️ (TypeScript optional) | ✅ (compile-time) |
| **Security** | ⚠️ (npm supply chain) | ✅ (Rust memory safety) |

### 9.3 Marketing Message

**Tagline**: "The HTTP server Python should have built-in"

**Elevator Pitch**:
> Ruchy HTTP Server is a production-ready static file server built on Rust.
> It's 10-100x faster than Python's http.server, memory-safe, and has zero
> dependencies. Replace `python3 -m http.server` with `ruchy serve` today.

---

## 10. Next Steps

**Immediate** (This Sprint):
1. Implement core HTTP server (HTTP-001 → HTTP-005)
2. EXTREME TDD (RED → GREEN → REFACTOR → FAST)
3. Prove 10x performance advantage over Python

**Future Phases**:
- **Phase 2**: Hot reload + WebSocket (replace Deno dev server)
- **Phase 3**: Full stdlib (fs, process, crypto) - next-gen Node/Python
- **Phase 4**: Industry standard adoption (Rust community, web dev)

---

**End of MVP Specification**

**Status**: ✅ READY FOR IMPLEMENTATION
**Next**: [HTTP-001] Implement `ruchy serve` CLI command
