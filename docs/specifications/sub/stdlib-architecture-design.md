# Sub-spec: Stdlib and Web Server — Architecture and Standard Library Design

**Parent:** [stdlib-webserver-spec.md](../stdlib-webserver-spec.md) Sections 1-3

---

## 1. Executive Summary

### 1.1 Purpose

This specification defines Ruchy's standard library and web server capabilities to enable scripting workflows similar to Python, Deno, and Node.js, with a focus on hosting WebAssembly applications in production environments.

### 1.2 Key Objectives

- **Standard Library**: Comprehensive, secure-by-default API surface comparable to Python/Deno/Node.js
- **Web Server**: High-performance WASM application hosting with sandboxing and isolation
- **Quality**: SQLite-style extreme testing (≥1000x test-to-code ratio, 100% branch coverage)
- **Security**: Deno-inspired permission model with zero-trust defaults
- **Performance**: Sub-100ms cold start, <5s setup time (following wos benchmarks)

### 1.3 Design Philosophy

**Inspired by Scientific Research**:
- ACM/IEEE WebAssembly runtime research (98 papers analyzed, 2024)
- SQLite testing methodology (1177x test-to-code ratio, 100% branch coverage)
- Deno secure-by-default architecture (permission-based sandboxing)
- Python flat API design (PEP 8 compliance, type annotations)
- Node.js event-driven patterns (async/await, middleware pipelines)

**Core Principles**:
1. **Security First**: Permission-based access control (no implicit privileges)
2. **Quality First**: Extreme TDD with mutation/property/fuzz testing
3. **Performance First**: WASM-native with zero-copy operations where possible
4. **Developer Experience First**: Flat API structure, comprehensive documentation

---

## 2. Architecture Principles

### 2.1 System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Ruchy Application Layer                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐            │
│  │   Standard   │  │  HTTP Server │  │     WASM     │            │
│  │   Library    │  │   (axum)     │  │   Runtime    │            │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘            │
│         │                 │                  │                     │
│         └─────────────────┴──────────────────┘                     │
│                           │                                        │
│                  ┌────────▼────────┐                              │
│                  │  Permission      │                              │
│                  │  Sandbox         │                              │
│                  └─────────────────┘                              │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Zero Backend Dependency (Client-Side)

Following `interactive.paiml.com` patterns:
- All WASM execution runs in browser (Pyodide/wasmtime pattern)
- Static site generation with zero server dependencies
- CloudFront CDN + S3 for static asset delivery
- Progressive enhancement (works without JS, enhanced with WASM)

### 2.3 Pure Functional Design (Server-Side)

Following `wos` microkernel patterns:
- Explicit state management (no hidden mutation)
- Deterministic execution (pure functions only)
- Type-safe sandboxing (Rust memory safety guarantees)
- Reproducible builds (no non-deterministic timestamps)

### 2.4 Permission Model (Deno-Inspired)

**Secure by Default**:
```ruchy
// EXPLICIT permission required (Deno pattern)
ruchy run --allow-read --allow-net script.ruchy

// Granular permissions
ruchy run --allow-read=/tmp --allow-net=api.example.com script.ruchy

// Zero permissions (sandbox mode)
ruchy run --sandbox script.ruchy
```

**Permission Categories** (based on Deno 2024 research):
- `--allow-read[=<path>]`: File system read access
- `--allow-write[=<path>]`: File system write access
- `--allow-net[=<domain>]`: Network access
- `--allow-env[=<var>]`: Environment variable access
- `--allow-run[=<cmd>]`: Subprocess execution
- `--allow-ffi`: Foreign function interface (unsafe)
- `--allow-all`: Grant all permissions (NOT RECOMMENDED)

### 2.5 Performance Targets

Based on `wos` benchmarks and WASM research (Hardware-Based WASM Accelerator, 2024):
- **Cold Start**: <100ms (WASM load in browser)
- **HTTP Request**: <10ms (simple GET/POST) - **AVERAGE ONLY**
- **HTTP Request (Tail Latency)**: <50ms p95, <100ms p99 (**NEW: Production gate**)
- **WASM Execution**: <5s setup, 142x speedup potential
- **File I/O**: <1ms (virtual filesystem, O(1) operations via im-rs)
- **Compilation**: <500ms for typical Ruchy scripts

**Production Performance Monitoring** (Toyota Way: *Genchi Genbutsu*):

Following Dean & Barroso (2013) "The Tail at Scale" research, performance gates must measure **tail latency** (p95, p99) under realistic load, not just lab averages. Lab benchmarks do not capture production conditions (noisy neighbors, network jitter, unpredictable I/O).

**Mandatory Production Gates**:
- Measure performance in production-like environment (multi-tenant, realistic load)
- Gate on p95 and p99 latency, NOT just average latency
- Include warmup period (discard first 1000 requests)
- Run for ≥60 seconds to capture variance
- Report percentiles: p50, p75, p90, p95, p99, p99.9

**Citation**: Dean, J., & Barroso, L. A. (2013). *The Tail at Scale*. Communications of the ACM, 56(2), 74-80.

### 2.6 Risk-Based Quality Standards (Toyota Way: Eliminating *Mura*)

**Rationale**: Uniform quality targets (e.g., 90% mutation coverage for all modules) create *Mura* (unevenness) by over-testing simple modules while potentially under-protecting critical ones. The Toyota Way dictates applying effort where it is most needed.

**Risk-Based Testing Strategy** (Forrester et al., 2011):

| Module | Criticality | Complexity | Primary Testing | Mutation Target | Special Requirement |
|--------|-------------|------------|-----------------|-----------------|---------------------|
| `crypto` | **Critical** | High | Property, Fuzzing, Formal | **95%** | **Constant-Time Analysis** (dudect) |
| `wasm` | **Critical** | High | Fuzzing, Formal Verification | **95%** | **Side-Channel Analysis** (Spectre mitigations) |
| `http` | High | Medium | Fuzzing, E2E, Fault Injection | **90%** | Performance Benchmarking (p99 gates) |
| `fs` | High | Medium | Fault Injection, Property | **90%** | Permission Model Stress Testing |
| `process` | High | Medium | Fault Injection, Property | **90%** | Signal handling, zombie processes |
| `env` | Medium | Low | Property, Unit | **85%** | Permission boundary testing |
| `json` | Medium | Medium | Fuzzing, Property | **90%** | Malformed input resilience |
| `time` | Low | Low | Unit, Property | **85%** | Timezone handling, leap seconds |
| `path` | Low | Low | Unit, Property | **85%** | Cross-platform consistency |
| `test` | Low | Low | Metamorphic Testing | **85%** | Test the tests themselves |

**Specialized Testing Requirements**:

1. **Constant-Time Cryptography** (`crypto` module):
   - Use `dudect` or similar tool to detect timing leaks
   - All cryptographic operations must complete in constant time
   - Gate: Zero timing leaks detected in 1,000,000 iterations

2. **Side-Channel Attack Mitigations** (`wasm` module):
   - Enable Wasmtime Spectre mitigations (branch target clobbering)
   - Isolate multi-tenant WASM modules in separate processes
   - Use memory sanitizers to detect out-of-bounds access

3. **Fault Injection Testing** (`fs`, `http`, `process` modules):
   - Simulate disk full, OOM, network failures
   - Verify graceful degradation (no panics, clean error returns)
   - Use `fail-rs` framework for systematic fault injection

4. **Metamorphic Testing** (`test` module):
   - Test relationships between test outcomes
   - Apply mutation testing to the test framework itself
   - Verify test runner correctness via metamorphic relations

**Citation**: Forrester, J. C., et al. (2011). *An Industrial Case Study in Applying a Risk-Based Approach to Software Testing*. IEEE ICST 2011.

---

## 3. Standard Library Design

### 3.1 API Design Principles (Empirically Grounded)

**Cognitive Science Foundation** (Stylos & Myers, 2006):

API design must be grounded in empirical research on developer cognition, not aesthetic preferences like "Pythonic." Research shows developers struggle with:
- Large numbers of classes or methods (cognitive overload)
- Long parameter lists (working memory limits)
- Inconsistent naming (violated expectations)

**Empirical API Usability Principles**:

1. **Low Cognitive Load**:
   - Functions MUST have ≤5 parameters (working memory constraint)
   - Modules MUST have ≤20 public functions (cognitive chunk limit)
   - Class hierarchies MUST be ≤3 levels deep (complexity budget)

2. **Fail-Fast Defaults**:
   - Default settings MUST be most secure/robust, NOT most convenient
   - Dangerous operations (e.g., `--allow-all`) require explicit opt-in
   - Functions return `Result<T, E>`, never panic on invalid input

3. **Principle of Least Astonishment**:
   - Function name MUST accurately predict behavior
   - No hidden side effects (e.g., `read_text` must not delete files)
   - Type signatures MUST be self-documenting

4. **Consistency**:
   - Uniform naming: `read_text`, `write_text` (not `readText`, `writeFile`)
   - Uniform error handling: All functions return `Result<T, ModuleError>`
   - Uniform async: All I/O operations use `async`/`await`

**Python PEP 8 Structural Patterns** (retained for consistency):
- Flat module structure: `import http` not `from http.server import HTTPServer`
- Absolute imports only (no relative imports)
- Type annotations on all public APIs (self-documentation)
- ASCII-only identifiers in standard library

**API Design Quality Gate**:
- Static analysis MUST check for "API design smells":
  - Functions with >5 parameters (FAIL)
  - Modules with >20 public functions (WARNING)
  - Inconsistent naming patterns (FAIL)

**Citation**: Stylos, J., & Myers, B. A. (2006). *Mica: A web-based tool for tracking and analyzing API usage*. Eclipse technology eXchange workshop, ACM.

### 3.2 Standard Library Modules

#### 3.2.1 Core Modules (Phase 1)

**`fs` - File System Operations**
```ruchy
// Pythonic async/await + Deno permissions
import fs

// Requires --allow-read=/tmp
let content = await fs.read_text("/tmp/data.txt")

// Requires --allow-write=/tmp
await fs.write_text("/tmp/output.txt", "Hello, Ruchy!")

// Directory operations
let entries = await fs.read_dir("/tmp")
await fs.create_dir("/tmp/new_folder")
await fs.remove("/tmp/old_file.txt")

// Metadata
let info = await fs.stat("/tmp/data.txt")
println(f"Size: {info.size} bytes")
```

**`http` - HTTP Client/Server**
```ruchy
import http

// Client (requires --allow-net=api.github.com)
let response = await http.get("https://api.github.com/repos/paiml/ruchy")
let json = await response.json()
println(f"Stars: {json.stargazers_count}")

// Server (requires --allow-net=0.0.0.0:8080)
let server = http.Server::new("0.0.0.0:8080")
server.route("/", fn(req) {
    http.Response::ok("Hello from Ruchy!")
})
await server.listen()
```

**`path` - Path Manipulation**
```ruchy
import path

let p = path.join("/home", "user", "documents", "file.txt")
println(path.basename(p))        // "file.txt"
println(path.dirname(p))         // "/home/user/documents"
println(path.extension(p))       // ".txt"
println(path.is_absolute(p))     // true
```

**`env` - Environment Variables**
```ruchy
import env

// Requires --allow-env=HOME
let home = env.get("HOME")

// Requires --allow-env
env.set("MY_VAR", "value")

// List all (requires --allow-env)
let vars = env.vars()
```

**`process` - Process Management**
```ruchy
import process

// Current process
println(f"PID: {process.pid()}")
println(f"Args: {process.args()}")

// Subprocess (requires --allow-run=ls)
let output = await process.run(["ls", "-la"])
println(output.stdout)

// Exit
process.exit(0)
```

#### 3.2.2 Advanced Modules (Phase 2)

**`wasm` - WebAssembly Runtime**
```ruchy
import wasm

// Load WASM module (requires --allow-read)
let module = await wasm.load("./app.wasm")

// Instantiate with imports
let instance = await module.instantiate({
    env: {
        println: fn(msg) { println(msg) }
    }
})

// Call exported function
let result = instance.call("add", [1, 2])
println(f"Result: {result}")  // 3
```

**`crypto` - Cryptography**
```ruchy
import crypto

// Hash
let hash = crypto.sha256("Hello, Ruchy!")
println(f"SHA256: {hash}")

// Random (requires --allow-env for seed)
let bytes = crypto.random_bytes(32)
let uuid = crypto.uuid_v4()
```

**`json` - JSON Parsing**
```ruchy
import json

let data = json.parse('{"name": "Ruchy", "version": "3.96.0"}')
println(data.name)  // "Ruchy"

let serialized = json.stringify(data)
println(serialized)
```

**`time` - Time and Date**
```ruchy
import time

let now = time.now()
println(f"Unix timestamp: {now.unix()}")
println(f"ISO 8601: {now.iso8601()}")

// Sleep (async)
await time.sleep(1000)  // 1 second
```

**`test` - Testing Framework**
```ruchy
import test

test.describe("Math operations", fn() {
    test.it("adds numbers correctly", fn() {
        test.assert_eq(1 + 1, 2)
    })

    test.it("handles edge cases", fn() {
        test.assert_eq(i64::MAX + 1, i64::MIN)  // Overflow wraps
    })
})
```

### 3.3 Error Handling

**Result Type Pattern** (Rust-inspired):
```ruchy
import fs

match await fs.read_text("/tmp/data.txt") {
    Ok(content) => println(content),
    Err(e) => {
        match e {
            fs.Error::NotFound => println("File not found"),
            fs.Error::PermissionDenied => println("Permission denied"),
            _ => println(f"Error: {e}")
        }
    }
}
```

### 3.4 Module Organization

**Flat Structure** (Python PEP 8):
```
src/stdlib/
├── fs.ruchy         # File system operations
├── http.ruchy       # HTTP client/server
├── path.ruchy       # Path manipulation
├── env.ruchy        # Environment variables
├── process.ruchy    # Process management
├── wasm.ruchy       # WASM runtime
├── crypto.ruchy     # Cryptography
├── json.ruchy       # JSON parsing
├── time.ruchy       # Time and date
└── test.ruchy       # Testing framework
```

---

