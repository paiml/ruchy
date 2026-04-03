# Sub-spec: HTTP Server — Implementation, Security, and Quality

**Parent:** [http-server-mvp-spec.md](../http-server-mvp-spec.md) Sections 3-5

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
