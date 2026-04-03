# Sub-spec: Stdlib and Web Server — Web Server Architecture and Testing

**Parent:** [stdlib-webserver-spec.md](../stdlib-webserver-spec.md) Sections 4-5

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

