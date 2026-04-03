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

---

## Sub-spec Index

| Sub-spec | Sections | Description | Lines |
|----------|----------|-------------|-------|
| [http-server-implementation-quality.md](sub/http-server-implementation-quality.md) | 3-5 | Implementation architecture (Axum/Tokio stack, core components, CLI integration), security requirements (path traversal, MIME safety), and EXTREME quality standards (unit/property/integration/mutation tests, benchmarks) | 287 |
| [http-server-performance-competitive.md](sub/http-server-performance-competitive.md) | 6, 8.5, 9 | Performance targets (10-100x vs Python), empirical validation methodology (scientific benchmarking, workload scenarios, feature parity checklists, WASM-specific features), and competitive positioning (vs Python/Node.js) | 263 |

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
