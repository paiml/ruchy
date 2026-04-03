# Sub-spec: Stdlib and Web Server — Quality Gates, Roadmap, and References

**Parent:** [stdlib-webserver-spec.md](../stdlib-webserver-spec.md) Sections 5.5-9 and Appendices

---

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
