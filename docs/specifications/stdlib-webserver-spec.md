# Ruchy Standard Library and Web Server Specification v1.1

**Document Status**: DRAFT (Peer-Reviewed)
**Version**: 1.1.0
**Date**: 2025-10-19
**Authors**: Ruchy Core Team
**Classification**: Technical Specification
**Peer Review**: Incorporated Toyota Way *Kaizen* and SQLite-style rigor enhancements

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Architecture Principles](#architecture-principles)
3. [Standard Library Design](#standard-library-design)
4. [Web Server Architecture](#web-server-architecture)
5. [Testing Methodology](#testing-methodology)
6. [Quality Gates](#quality-gates)
7. [Scientific Foundations](#scientific-foundations)
8. [Implementation Roadmap](#implementation-roadmap)
9. [References](#references)

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

## 4. Web Server Architecture

### 4.1 Design Goals

Based on `interactive.paiml.com` and `wos` patterns:
1. **Static Site Serving**: CloudFront + S3 pattern
2. **WASM Application Hosting**: Wasmtime runtime with sandboxing
3. **Zero Backend**: All computation client-side or at CDN edge
4. **Security**: Permission-based isolation (no arbitrary code execution)

### 4.2 Server Architecture

**HTTP Server (Axum-based)**:
```ruchy
import http

let server = http.Server::new("0.0.0.0:8080")

// Static file serving
server.static("/", "./dist")

// WASM endpoint
server.route("/api/execute", fn(req) {
    let code = req.body_json()

    // Execute in sandbox
    let result = wasm.execute_sandboxed(code, {
        permissions: ["--allow-read=/tmp"],
        timeout_ms: 5000,
        memory_limit_mb: 100
    })

    http.Response::json(result)
})

// Health check
server.route("/health", fn(req) {
    http.Response::ok("OK")
})

await server.listen()
```

### 4.3 WASM Sandbox (with Side-Channel Attack Mitigations)

**Isolation Model** (based on ACM/IEEE WASM research 2024 + Spectre mitigations):
```ruchy
import wasm

let sandbox = wasm.Sandbox::new({
    // Memory limits
    memory_limit_mb: 100,
    stack_limit_kb: 1024,

    // Time limits
    timeout_ms: 5000,
    instruction_limit: 1_000_000,

    // Permissions (Deno-style)
    allow_read: ["/tmp"],
    allow_write: [],
    allow_net: ["api.example.com"],

    // Resource limits
    max_open_files: 10,
    max_threads: 4,

    // Side-channel attack mitigations (NEW)
    enable_spectre_mitigations: true,  // Branch target clobbering
    process_isolation: true,           // Separate process per tenant
    constant_time_enforcement: true    // Detect timing leaks in crypto
})

// Execute code in sandbox
let result = sandbox.execute(wasm_bytes)
```

**Side-Channel Attack Mitigations** (Kocher et al., 2019):

**Critical Threat**: In multi-tenant WASM environments, one malicious tenant could extract secrets from another via side-channel attacks (Spectre, timing attacks).

**Mandatory Mitigations**:

1. **Spectre Mitigations** (CPU speculative execution attacks):
   - Enable Wasmtime's built-in Spectre mitigations:
     - **Branch target clobbering**: Clear speculative execution state
     - **Index masking**: Bounds checks use bitwise AND (not branches)
     - **Retpoline**: Indirect branch protection
   - Configuration: `wasmtime::Config::new().cranelift_spectre_v1_mitigation(true)`

2. **Process Isolation** (multi-tenant separation):
   - Isolate each WASM module in a separate OS process
   - Prevents memory sharing between tenants
   - Use `wasmtime::Engine` with `Config::new().parallel_compilation(false)`
   - Overhead: ~10ms per module instantiation (acceptable for security)

3. **Constant-Time Cryptography** (`crypto` module):
   - All cryptographic operations MUST complete in constant time
   - No branching on secret data (prevent timing attacks)
   - Use `dudect` to detect timing leaks:
     ```rust
     // tests/side_channel/crypto_timing_tests.rs
     use dudect::*;

     #[test]
     fn test_aes_encrypt_constant_time() {
         let detector = DudectBencher::new();
         let result = detector.bench(|classes| {
             // Run AES encryption with different secret keys
             crypto::aes_encrypt(classes.get_secret_key(), plaintext)
         });

         // MUST pass: No timing leaks detected
         assert!(result.t_statistic.abs() < 4.5);  // 99.999% confidence
     }
     ```

4. **Memory Sanitizers** (detect out-of-bounds access):
   - Run WASM modules under AddressSanitizer (ASan)
   - Detects buffer overflows that could leak adjacent memory
   - Configuration: `RUSTFLAGS="-Z sanitizer=address" cargo test`

**Security Testing Requirements**:
- **Spectre POC**: Test with known Spectre proof-of-concept exploits (MUST fail)
- **Timing Analysis**: Use `dudect` on all crypto operations (MUST pass with t-statistic <4.5)
- **Memory Sanitizers**: Run full test suite under ASan (MUST detect all buffer overflows)
- **Multi-Tenant Stress**: 1000+ concurrent tenants, verify zero information leakage

**Citation**: Kocher, P., et al. (2019). *Spectre Attacks: Exploiting Speculative Execution*. IEEE Symposium on Security and Privacy (S&P).

### 4.4 Static Site Generation

**Build Pipeline** (following `interactive.paiml.com`):
```ruchy
import { StaticSiteGenerator } from "ruchy/ssg"

let ssg = StaticSiteGenerator::new({
    input: "./content",
    output: "./dist",
    template: "./templates"
})

// Compile Ruchy to WASM
ssg.add_wasm_module("app.ruchy", {
    optimize: true,
    strip: true,
    target_size_kb: 500
})

// Generate HTML
ssg.generate()

// Deploy to S3 + CloudFront
ssg.deploy({
    bucket: "interactive.paiml.com-production",
    distribution: "ELY820FVFXAFF"
})
```

### 4.5 Performance Optimization

**WASM Compilation Cache**:
```ruchy
// Cache compiled WASM modules (following wasmtime patterns)
let cache = wasm.Cache::new("/var/cache/ruchy/wasm")

let module = cache.get_or_compile("app.ruchy", fn() {
    wasm.compile_from_file("app.ruchy")
})
```

**Zero-Copy Operations**:
```ruchy
// Avoid memory copies (WASM linear memory sharing)
let buffer = wasm.share_memory(data)  // Zero-copy view
instance.call("process_data", buffer)
```

---

## 5. Testing Methodology

### 5.1 SQLite-Style Extreme Testing

**Target Metrics** (based on SQLite 2024 documentation):
- **Test-to-Code Ratio**: ≥1000:1 (SQLite achieves 1177:1)
- **Branch Coverage**: 100% (every branch instruction tested)
- **Mutation Coverage**: ≥90% (cargo-mutants kill rate)
- **Property Tests**: 10,000+ iterations per property
- **Fuzz Tests**: 1,000,000+ random inputs per fuzzer

### 5.2 Test Harnesses

**Three Independent Test Suites** (SQLite TH3 pattern):

#### 5.2.1 Unit Tests (Rust native)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fs_read_text() {
        let content = fs::read_text("/tmp/test.txt").unwrap();
        assert_eq!(content, "Hello, Ruchy!");
    }

    #[test]
    fn test_fs_permission_denied() {
        let result = fs::read_text("/root/secret.txt");
        assert!(matches!(result, Err(FsError::PermissionDenied)));
    }
}
```

#### 5.2.2 Property Tests (proptest)
```rust
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn prop_path_join_is_deterministic(
        parts: Vec<String>
    ) {
        let result1 = path::join(&parts);
        let result2 = path::join(&parts);
        prop_assert_eq!(result1, result2);
    }

    #[test]
    fn prop_http_response_roundtrip(
        status: u16,
        body: Vec<u8>
    ) {
        let response = http::Response::new(status, body.clone());
        let serialized = response.to_bytes();
        let deserialized = http::Response::from_bytes(&serialized)?;
        prop_assert_eq!(deserialized.body, body);
    }
}
```

#### 5.2.3 Fuzz Tests (cargo-fuzz)
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // HTTP parser should never panic
    let _ = http::parse_request(data);
});

fuzz_target!(|data: String| {
    // JSON parser should never panic
    let _ = json::parse(&data);
});
```

#### 5.2.4 Fault Injection Tests (SQLite TH3 Pattern)

**Rationale**: SQLite's legendary resilience comes from its ability to handle failures in the underlying system (disk I/O errors, out-of-memory conditions). A production web server must be similarly resilient.

**Fault Injection Framework** (using `fail-rs`):

```rust
// tests/fault_injection/fs_tests.rs
use fail::FailScenario;

#[test]
fn test_write_resilience_on_disk_full() {
    let scenario = FailScenario::setup();

    // Configure fault injection: next write syscall fails with ENOSPC
    fail::cfg("syscall::write", "return(ENOSPC)").unwrap();

    let result = fs::write_text("/tmp/output.txt", "some data");

    // MUST handle gracefully, NOT panic
    assert!(matches!(result, Err(fs::Error::DiskFull)));

    scenario.teardown();
}

#[test]
fn test_http_resilience_on_connection_reset() {
    let scenario = FailScenario::setup();

    // Simulate network failure
    fail::cfg("network::send", "return(ECONNRESET)").unwrap();

    let result = http::get("https://example.com");

    // MUST return clean error, NOT panic
    assert!(matches!(result, Err(http::Error::ConnectionReset)));

    scenario.teardown();
}

#[test]
fn test_process_resilience_on_oom() {
    let scenario = FailScenario::setup();

    // Simulate out-of-memory condition
    fail::cfg("allocator::alloc", "return(ENOMEM)").unwrap();

    let result = process::run(["large-memory-command"]);

    // MUST handle gracefully
    assert!(matches!(result, Err(process::Error::OutOfMemory)));

    scenario.teardown();
}
```

**Mandatory Fault Injection Scenarios**:
- **File System**: Disk full (ENOSPC), permission denied (EACCES), file not found (ENOENT)
- **Network**: Connection reset (ECONNRESET), timeout (ETIMEDOUT), refused (ECONNREFUSED)
- **Memory**: Out of memory (ENOMEM), allocation failure
- **Process**: Signal interruption (EINTR), zombie processes, segmentation fault recovery

**Citation**: Gunawi, H. S., et al. (2011). *Towards automatically checking thousands of filesystem data consistency properties*. FAST '11.

#### 5.2.5 MC/DC Coverage Tests (Avionics-Grade Rigor)

**Rationale**: 100% branch coverage is insufficient for critical modules. Modified Condition/Decision Coverage (MC/DC) is required for avionics software and should be applied to `crypto`, `wasm`, and permission sandbox.

**MC/DC Definition**: Every condition in a decision must independently affect the decision outcome.

**Example MC/DC Test Case**:

```rust
// tests/mcdc/permission_tests.rs

// Function under test: permission check with compound condition
fn check_permission(allow_read: bool, allow_write: bool, operation: Op) -> bool {
    match operation {
        Op::Read => allow_read,
        Op::Write => allow_write,
        Op::ReadWrite => allow_read && allow_write,  // MC/DC target
    }
}

// MC/DC Test Suite: Each condition must independently affect outcome
#[test]
fn mcdc_read_write_permission() {
    // Condition 1: allow_read affects outcome
    assert_eq!(check_permission(true, true, Op::ReadWrite), true);
    assert_eq!(check_permission(false, true, Op::ReadWrite), false);

    // Condition 2: allow_write affects outcome
    assert_eq!(check_permission(true, true, Op::ReadWrite), true);
    assert_eq!(check_permission(true, false, Op::ReadWrite), false);

    // MC/DC coverage: Both conditions tested independently
}
```

**MC/DC Requirements**:
- **crypto module**: 100% MC/DC on all cryptographic logic
- **wasm module**: 100% MC/DC on sandbox permission checks
- **Permission system**: 100% MC/DC on access control decisions

**Tools**: Use `cargo-llvm-cov` with MC/DC instrumentation (requires nightly Rust)

**Citation**: Hayhurst, K. J., et al. (2001). *A Practical Tutorial on Modified Condition/Decision Coverage*. NASA/TM-2001-210876.

#### 5.2.6 Metamorphic Tests (Testing the Tests)

**Rationale**: A 90% mutation score shows tests are sensitive to implementation changes, but does not prove test *assertions* are correct. Metamorphic testing validates the test framework itself.

**Metamorphic Relations** (Chen et al., 2018):

```rust
// tests/metamorphic/test_framework_tests.rs

#[test]
fn metamorphic_relation_logging_does_not_affect_test_outcome() {
    // MR: Adding non-functional logging must not change test results

    // Original test
    let result1 = run_test("test_path_join");
    assert!(result1.passed);

    // Add logging to code under test
    inject_logging("path::join");

    // Rerun test
    let result2 = run_test("test_path_join");

    // Metamorphic relation: Both must pass (or both fail)
    assert_eq!(result1.passed, result2.passed);
}

#[test]
fn metamorphic_relation_test_order_independence() {
    // MR: Test outcome must not depend on execution order

    let results_forward = run_tests_in_order(&["test_a", "test_b", "test_c"]);
    let results_reverse = run_tests_in_order(&["test_c", "test_b", "test_a"]);

    // Each test must have same outcome regardless of order
    assert_eq!(results_forward["test_a"], results_reverse["test_a"]);
    assert_eq!(results_forward["test_b"], results_reverse["test_b"]);
    assert_eq!(results_forward["test_c"], results_reverse["test_c"]);
}

#[test]
fn metamorphic_relation_mutation_preserves_failure() {
    // MR: If test fails on buggy code, it must also fail on mutated buggy code

    let buggy_code = inject_bug("fs::read_text");
    let result1 = run_test("test_fs_read_text");
    assert!(!result1.passed);  // Test catches bug

    let mutated_bug = mutate(buggy_code);
    let result2 = run_test("test_fs_read_text");

    // Test must still fail (detecting the mutated bug)
    assert!(!result2.passed);
}
```

**Metamorphic Relations for Test Framework**:
1. **Non-functional changes preserve outcome**: Adding logging/comments must not change test results
2. **Test order independence**: Test outcomes must not depend on execution order
3. **Mutation preserves failure**: Mutating buggy code must still be caught by tests

**Citation**: Chen, T. Y., et al. (2018). *Metamorphic testing: A review of challenges and opportunities*. ACM Computing Surveys (CSUR), 51(1), 1-27.

### 5.3 Mutation Testing

**cargo-mutants Configuration** (following Ruchy Sprint 8 patterns):
```bash
# Run mutation tests on standard library
cargo mutants --file src/stdlib/fs.rs --timeout 300

# Target: ≥90% kill rate (CAUGHT / (CAUGHT + MISSED) ≥ 0.90)
cargo mutants --workspace --json --output mutants.json

# Analyze gaps
grep "MISSED" mutants.json | jq '.mutant'
```

**Mutation Test Patterns** (from Sprint 8 empirical data):
1. **Match Arm Deletions**: Test ALL match arms with assertions
2. **Function Stub Replacements**: Validate return values are real, not None/empty
3. **Boundary Conditions**: Test <, <=, ==, >, >= explicitly
4. **Boolean Negations**: Test both true AND false branches
5. **Operator Changes**: Test +/-, */%, <=/>=, &&/|| alternatives

### 5.4 Integration Tests

**End-to-End Testing** (following `interactive.paiml.com` E2E patterns):
```typescript
// tests/e2e/web_server.spec.ts
import { test, expect } from '@playwright/test';

test('WASM execution sandbox', async ({ page }) => {
    await page.goto('http://localhost:8080');

    // Execute code in sandbox
    const result = await page.evaluate(async () => {
        const response = await fetch('/api/execute', {
            method: 'POST',
            body: JSON.stringify({ code: 'println("Hello!")' })
        });
        return response.json();
    });

    expect(result.stdout).toBe('Hello!\n');
    expect(result.exit_code).toBe(0);
});

test('permission denial', async ({ page }) => {
    const result = await page.evaluate(async () => {
        const response = await fetch('/api/execute', {
            method: 'POST',
            body: JSON.stringify({ code: 'fs.read_text("/etc/passwd")' })
        });
        return response.json();
    });

    expect(result.error).toContain('PermissionDenied');
});
```

### 5.5 Coverage Requirements

**Mandatory Thresholds** (following `wos` quality gates):
- **Line Coverage**: ≥85% (measured via cargo-llvm-cov)
- **Branch Coverage**: ≥90% (all conditional branches tested)
- **Function Coverage**: 100% (all public functions have tests)
- **Mutation Coverage**: ≥90% (all mutants caught or documented as acceptable)

**Coverage Pipeline**:
```bash
# Generate coverage report
cargo llvm-cov --all-features --workspace --html

# Enforce thresholds (CI gate)
cargo llvm-cov --fail-under-lines 85 --fail-under-branches 90

# Open HTML report
xdg-open target/llvm-cov/html/index.html
```

---

## 6. Quality Gates

### 6.1 Pre-Commit Hooks

**MANDATORY** (following Ruchy CLAUDE.md):
```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

echo "Running quality gates..."

# Format check
cargo fmt --check

# Clippy (zero warnings)
cargo clippy --all-features -- -D warnings

# Unit tests
cargo nextest run --lib

# PMAT quality gates
pmat analyze complexity --max-cyclomatic 10 --max-cognitive 10
pmat analyze satd --fail-on-violation

# bashrs validation (shell scripts)
make lint-bashrs

echo "✅ All quality gates passed"
```

### 6.2 PMAT Quality Standards

**Technical Debt Grade (TDG)**:
- **Minimum**: A- (≥85 points)
- **Target**: A+ (≥95 points)

**Component Scoring** (via `pmat tdg <file> --include-components`):
- **Complexity**: ≤10 cyclomatic complexity per function
- **Duplication**: <10% code duplication
- **Documentation**: >70% documented functions
- **SATD**: Zero TODO/FIXME/HACK comments

### 6.3 Continuous Integration

**GitHub Actions Pipeline**:
```yaml
name: Quality Gates
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run tests
        run: cargo nextest run --all-features

      - name: Coverage
        run: |
          cargo llvm-cov --all-features --html
          cargo llvm-cov --fail-under-lines 85

      - name: Mutation tests
        run: cargo mutants --workspace --timeout 600

      - name: PMAT gates
        run: |
          pmat analyze complexity --max-cyclomatic 10
          pmat analyze satd --fail-on-violation
          pmat tdg . --min-grade A- --fail-on-violation

      - name: E2E tests
        run: playwright test tests/e2e/
```

---

## 7. Scientific Foundations

### 7.1 Peer-Reviewed Research

**WebAssembly Runtimes** (ACM Transactions on Software Engineering and Methodology, 2024):
- Survey of 98 research papers on WASM runtimes
- Security model: Sandboxing and isolation critical for untrusted code
- Performance: Hardware-based accelerators achieve 142x speedup
- **Citation**: Research on WebAssembly Runtimes: A Survey. ACM Trans. Softw. Eng. Methodol. 2024. DOI: 10.1145/3714465

**WebAssembly Security** (ScienceDirect, 2024):
- Analysis of 121 security-related papers
- Seven security categories identified: isolation, sandboxing, side-channels, fuzzing, formal verification, malware analysis, cryptography
- **Citation**: Cabrera Lozoya, R. WebAssembly and Security: A Review. Future Generation Computer Systems, 2025. DOI: 10.1016/j.future.2024.107538

**Comparative Study of WASM Runtimes** (April 2024):
- WasmEdge: Multi-tenant isolation for AI workloads
- Lucet: Lightweight isolation for serverless
- Wasmer: Native binding security with sandboxed execution
- **Citation**: A Comparative Study of WebAssembly Runtimes. Advances in Artificial Intelligence and Machine Learning, 2024.

### 7.2 Testing Methodologies

**SQLite Testing** (sqlite.org/testing.html):
- 1177x test-to-code ratio (91.4 MSLOC test / 77.6 KSLOC code)
- 100% branch coverage via TH3 harness
- Mutation testing to verify every branch makes a difference
- **Citation**: SQLite Testing Documentation. https://www.sqlite.org/testing.html

**Formal Verification of Standard Libraries** (Springer, 2012):
- Testing Library Specifications by Verifying Conformance Tests
- Formal CTD Process (FCTD) for Java Modeling Language (JML)
- **Citation**: Testing Library Specifications by Verifying Conformance Tests. FM 2012: Formal Methods. Springer.

### 7.3 Language Design Patterns

**Python PEP 8** (peps.python.org):
- Standard library conventions: absolute imports, ASCII identifiers
- **Citation**: van Rossum, G., Warsaw, B., & Coghlan, N. (2001). PEP 8 – Style Guide for Python Code.

**Deno Architecture** (Deno Blog, 2024):
- Secure by default: Permission-based sandboxing
- Standard library stabilization (40+ modules, 4 years, 151 releases)
- **Citation**: Deno in 2024. Deno Blog, October 9, 2024. https://deno.com/blog/deno-in-2024

**Node.js Design Patterns** (Packt Publishing, 2024):
- Module pattern, event-driven architecture, middleware pipelines
- ESM async/await patterns
- **Citation**: Casciaro, M., & Mammino, L. (2024). Node.js Design Patterns, Fourth Edition. Packt Publishing.

---

## 8. Implementation Roadmap

### 8.1 Phase 1: Core Standard Library (Weeks 1-4)

**Sprint 1: File System Module**
- [STDLIB-001] fs.read_text() with permission checks
- [STDLIB-002] fs.write_text() with sandboxing
- [STDLIB-003] fs.read_dir() iterator
- [STDLIB-004] fs.stat() metadata
- [STDLIB-005] Property tests (10,000+ iterations)
- [STDLIB-006] Mutation tests (≥90% kill rate)

**Sprint 2: HTTP Client Module**
- [STDLIB-007] http.get() with TLS support
- [STDLIB-008] http.post() with request body
- [STDLIB-009] http.Response type with JSON parsing
- [STDLIB-010] Permission model (--allow-net)
- [STDLIB-011] Property tests for HTTP parsing
- [STDLIB-012] Fuzz tests for malformed requests

**Sprint 3: Process and Environment**
- [STDLIB-013] process.run() subprocess execution
- [STDLIB-014] env.get() / env.set() with permissions
- [STDLIB-015] path.join() / path.dirname()
- [STDLIB-016] Integration tests (E2E subprocess)
- [STDLIB-017] Coverage ≥85% enforcement

**Sprint 4: Testing and Quality**
- [STDLIB-018] test.describe() / test.it() framework
- [STDLIB-019] test.assert_eq() / test.assert_ne()
- [STDLIB-020] Property test helpers
- [STDLIB-021] Mutation test suite
- [STDLIB-022] Documentation (70%+ coverage)

### 8.2 Phase 2: Web Server (Weeks 5-8)

**Sprint 5: HTTP Server Foundation**
- [WEBSERVER-001] http.Server::new() initialization
- [WEBSERVER-002] server.route() middleware pipeline
- [WEBSERVER-003] server.static() file serving
- [WEBSERVER-004] Axum integration
- [WEBSERVER-005] E2E tests (Playwright)

**Sprint 6: WASM Sandbox**
- [WEBSERVER-006] wasm.Sandbox::new() initialization
- [WEBSERVER-007] wasm.execute_sandboxed() with limits
- [WEBSERVER-008] Memory isolation (wasmtime)
- [WEBSERVER-009] Permission enforcement
- [WEBSERVER-010] Property tests (isolation guarantees)

**Sprint 7: Static Site Generation**
- [WEBSERVER-011] StaticSiteGenerator::new()
- [WEBSERVER-012] ssg.add_wasm_module() compilation
- [WEBSERVER-013] ssg.generate() HTML output
- [WEBSERVER-014] ssg.deploy() S3+CloudFront
- [WEBSERVER-015] Integration tests (full pipeline)

**Sprint 8: Performance and Security**
- [WEBSERVER-016] WASM compilation cache
- [WEBSERVER-017] Zero-copy memory sharing
- [WEBSERVER-018] TLS/HTTPS support
- [WEBSERVER-019] Rate limiting
- [WEBSERVER-020] Security audit (fuzzing)

### 8.3 Phase 3: Advanced Modules (Weeks 9-12)

**Sprint 9: Cryptography**
- [STDLIB-023] crypto.sha256() / crypto.sha512()
- [STDLIB-024] crypto.random_bytes() (ChaCha8)
- [STDLIB-025] crypto.uuid_v4()
- [STDLIB-026] Property tests (cryptographic properties)

**Sprint 10: JSON and Time**
- [STDLIB-027] json.parse() / json.stringify()
- [STDLIB-028] time.now() / time.sleep()
- [STDLIB-029] time.iso8601() formatting
- [STDLIB-030] Fuzz tests (malformed JSON)

**Sprint 11: Documentation**
- [STDLIB-031] API reference documentation
- [STDLIB-032] User guide (Ruchy Book)
- [STDLIB-033] Examples (rosetta-ruchy)
- [STDLIB-034] Tutorial notebooks

**Sprint 12: Release and Validation**
- [STDLIB-035] Final mutation testing (≥90%)
- [STDLIB-036] Coverage validation (≥85%)
- [STDLIB-037] E2E validation (all workflows)
- [STDLIB-038] v4.0.0 release to crates.io

---

## 9. References

### 9.1 Scientific Papers

1. **Research on WebAssembly Runtimes: A Survey**
   ACM Transactions on Software Engineering and Methodology, 2024
   DOI: 10.1145/3714465

2. **WebAssembly and Security: A Review**
   Future Generation Computer Systems, 2025
   DOI: 10.1016/j.future.2024.107538

3. **A Comparative Study of WebAssembly Runtimes**
   Advances in Artificial Intelligence and Machine Learning, 2024

4. **Hardware-Based WebAssembly Accelerator for Embedded System**
   Electronics 2024, 13(20), 3979
   DOI: 10.3390/electronics13203979

5. **Testing Library Specifications by Verifying Conformance Tests**
   FM 2012: Formal Methods, Springer, 2012
   DOI: 10.1007/978-3-642-30473-6_6

6. **The Tail at Scale**
   Dean, J., & Barroso, L. A. (2013)
   Communications of the ACM, 56(2), 74-80
   (Production performance monitoring, tail latency gates)

7. **An Industrial Case Study in Applying a Risk-Based Approach to Software Testing**
   Forrester, J. C., et al. (2011)
   IEEE Fourth International Conference on Software Testing, Verification and Validation (ICST)
   (Risk-based quality standards, eliminating *Mura*)

8. **Towards Automatically Checking Thousands of Filesystem Data Consistency Properties**
   Gunawi, H. S., et al. (2011)
   9th USENIX Conference on File and Storage Technologies (FAST '11)
   (Fault injection testing for resilience)

9. **A Practical Tutorial on Modified Condition/Decision Coverage**
   Hayhurst, K. J., et al. (2001)
   NASA/TM-2001-210876
   (MC/DC coverage for avionics-grade rigor)

10. **Spectre Attacks: Exploiting Speculative Execution**
    Kocher, P., et al. (2019)
    40th IEEE Symposium on Security and Privacy (S&P)
    (Side-channel attack mitigations, Spectre defense)

11. **Metamorphic Testing: A Review of Challenges and Opportunities**
    Chen, T. Y., et al. (2018)
    ACM Computing Surveys (CSUR), 51(1), 1-27
    (Testing the tests, metamorphic relations)

12. **Mica: A Web-Based Tool for Tracking and Analyzing API Usage**
    Stylos, J., & Myers, B. A. (2006)
    Eclipse technology eXchange workshop, ACM
    (Empirical API usability principles, cognitive load)

### 9.2 Standards and Specifications

13. **PEP 8 – Style Guide for Python Code**
    van Rossum, G., Warsaw, B., & Coghlan, N.
    https://peps.python.org/pep-0008/

14. **SQLite Testing Documentation**
    https://www.sqlite.org/testing.html

15. **Deno in 2024**
    Deno Blog, October 9, 2024
    https://deno.com/blog/deno-in-2024

16. **Node.js Design Patterns, Fourth Edition**
    Casciaro, M., & Mammino, L., Packt Publishing, 2024

### 9.3 Related Projects

17. **interactive.paiml.com**: WASM-based interactive book platform
    https://interactive.paiml.com

18. **wos**: Educational microkernel OS in WASM
    https://github.com/paiml/wos

19. **Ruchy Language**
    https://github.com/paiml/ruchy

---

## Appendices

### Appendix A: Permission Model Details

**Permission Syntax**:
```bash
# File system
--allow-read[=<path>[,<path>...]]
--allow-write[=<path>[,<path>...]]

# Network
--allow-net[=<domain>[:<port>][,<domain>...]]

# Environment
--allow-env[=<var>[,<var>...]]

# Subprocess
--allow-run[=<cmd>[,<cmd>...]]

# All permissions (DANGEROUS)
--allow-all
```

**Runtime Permission Checks**:
```rust
// src/runtime/permissions.rs
pub struct PermissionManager {
    allow_read: HashSet<PathBuf>,
    allow_write: HashSet<PathBuf>,
    allow_net: HashSet<(String, Option<u16>)>,
    allow_env: HashSet<String>,
    allow_run: HashSet<String>,
}

impl PermissionManager {
    pub fn check_read(&self, path: &Path) -> Result<(), PermissionError> {
        if self.allow_read.is_empty() {
            return Err(PermissionError::ReadDenied);
        }
        // Check if path is within allowed directories
        for allowed in &self.allow_read {
            if path.starts_with(allowed) {
                return Ok(());
            }
        }
        Err(PermissionError::ReadDenied)
    }
}
```

### Appendix B: Test Coverage Targets

**Module-by-Module Coverage** (following `wos` quality gates):

| Module    | Line Coverage | Branch Coverage | Mutation Coverage |
|-----------|--------------|-----------------|-------------------|
| fs        | ≥85%         | ≥90%            | ≥90%              |
| http      | ≥85%         | ≥90%            | ≥90%              |
| wasm      | ≥85%         | ≥90%            | ≥90%              |
| path      | ≥90%         | ≥95%            | ≥95%              |
| env       | ≥85%         | ≥90%            | ≥90%              |
| process   | ≥80%         | ≥85%            | ≥85%              |
| crypto    | ≥90%         | ≥95%            | ≥95%              |
| json      | ≥85%         | ≥90%            | ≥90%              |
| time      | ≥85%         | ≥90%            | ≥90%              |
| test      | ≥85%         | ≥90%            | ≥90%              |

### Appendix C: Performance Benchmarks

**Baseline Targets** (measured on AWS EC2 c7g.xlarge):

```rust
// benches/stdlib_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_fs_read(c: &mut Criterion) {
    c.bench_function("fs::read_text", |b| {
        b.iter(|| {
            fs::read_text(black_box("/tmp/test.txt"))
        });
    });
}

// Targets:
// - fs::read_text: <1ms for 1KB files
// - http::get: <10ms for simple GET
// - wasm::compile: <500ms for typical scripts
// - json::parse: <100µs for 1KB JSON
criterion_group!(benches, bench_fs_read);
criterion_main!(benches);
```

---

**End of Specification**

**Status**: DRAFT - Requires review and approval
**Next Steps**: Begin Phase 1 implementation (STDLIB-001)
**Contact**: ruchy-core-team@paiml.com
