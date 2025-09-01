# Systems Programming Language Design Principles

## Core Design Philosophy

Systems programming requires languages that minimize configuration overhead while providing explicit security boundaries when needed. This specification outlines proven patterns that reduce cognitive load and accelerate development without compromising system reliability.

## Fundamental Principles

### Single Binary Distribution

Static compilation with aggressive dead code elimination produces minimal deployment artifacts. The linker includes only referenced functions and data, eliminating unused library code regardless of import scope.

```bash
ruchy --compile script.ruchy  # Produces minimal static binary
./script                      # Runs anywhere, no runtime required
```

**Dead code elimination semantics:**
- Unused functions eliminated at link time
- Unreferenced data structures removed from binary
- Library dependencies included only for called functions
- Generic instantiations generated only for used type combinations

**Binary size optimization:**
```rust
// Example: DataFrame script
import std::polars::DataFrame as df
import std::fs::read_to_string as read

let data = read("input.csv")
// df never used - Polars code eliminated from binary
```

The resulting binary contains only file I/O functions, not DataFrame processing logic. This produces sub-megabyte binaries for simple scripts while maintaining access to comprehensive standard libraries.

**Technical advantages:**
- Container images reduced to <5MB through aggressive optimization
- Cross-compilation generates platform-specific minimal binaries
- Link-time optimization (LTO) eliminates cross-module dead code
- Binary analysis simplified through precise inclusion tracking

### Integrated Development Toolchain

Unified tooling eliminates configuration debates and ensures consistent code quality across teams.

```bash
ruchy fmt                     # Code formatting (zero configuration)
ruchy lint                    # Static analysis with fix suggestions
ruchy test                    # Property-based and unit testing
ruchy check                   # Type verification and inference
ruchy doc                     # Documentation generation
ruchy compile                 # Native compilation with optimization
```

**Implementation benefits:**
- Zero configuration required for standard workflows
- Deterministic formatting eliminates style debates
- Consistent linting rules across all projects
- Single binary installation for entire development environment

### Opt-In Security Model

Scripts execute with normal system privileges by default. Security restrictions activate through explicit flags when executing untrusted code or implementing defense-in-depth strategies.

**Execution interface:**
```bash
# Default execution - normal system access
ruchy automation.ruchy

# Secure execution - explicit permission grants required
ruchy --secure --allow-read=/data --allow-net=internal.api deployment.ruchy
ruchy --secure --allow-exec=git,docker --allow-write=/tmp build.ruchy
```

**Security boundary implementation:**
- `--secure`: Activates permission enforcement
- `--allow-read=<path>`: Filesystem read access to specified paths
- `--allow-write=<path>`: Filesystem write access to specified paths
- `--allow-net=<domain>`: Network access to domains/IP ranges
- `--allow-exec=<binary>`: Process execution of named binaries
- `--allow-env=<vars>`: Environment variable access

This model supports trusted automation workflows (default) and untrusted code execution (secure mode) through a unified interface.

### Performance-First Architecture

Native compilation with zero-cost abstractions delivers predictable performance characteristics.

**Execution semantics:**
```bash
ruchy script.ruchy            # Direct execution (default mode)
ruchy --compile script.ruchy  # Explicit AOT compilation  
```

The primary interface eliminates subcommand overhead. `ruchy script.ruchy` performs incremental compilation and execution as a single atomic operation, optimized for the edit-run development cycle.

**Runtime characteristics:**
- Sub-10ms startup time for cached compilations
- Direct system calls without interpreter overhead  
- Memory usage determined at compile time
- LLVM optimization passes for maximum performance

**Implementation strategy:**
- First execution: compile to memory-mapped bytecode, execute immediately
- Subsequent executions: validate source hash, execute cached bytecode
- Cache invalidation through modification timestamp comparison
- Parallel compilation of independent modules

### Zero-Configuration Defaults

Standard workflows require no configuration files. Tools operate with production-ready settings immediately upon installation.

**Default behaviors:**
- Code formatting follows established conventions
- Linting enables all correctness and safety checks
- Testing discovers and executes all test functions
- Documentation generates from code comments and examples
- Compilation optimizes for release builds

Configuration files become necessary only for non-standard requirements. The `ruchy.toml` file provides overrides when defaults prove insufficient.

### Comprehensive Error Diagnostics

Error messages provide specific remediation guidance rather than cryptic compiler output.

**Error message structure:**
- Exact source location with surrounding context
- Plain language explanation of the issue
- Suggested fixes for common error patterns
- Links to relevant documentation sections

The type system enables early error detection with human-readable explanations that guide developers toward correct solutions.

## Implementation Architecture

### Compilation Pipeline

**Hybrid execution model optimizes for development velocity:**
- Development: Incremental compilation with memory-based execution
- Production: Ahead-of-time compilation to optimized native binary

**Caching strategy:**
- LLVM IR cached by content hash in `~/.ruchy/cache/`
- Incremental recompilation of changed modules only
- Dependency analysis enables parallel compilation
- Cache invalidation through file modification timestamps

### Module System

**Dependency management through explicit imports:**
```rust
import std::fs::read_to_string
import std::process::Command
import "./config.ruchy" as config
```

**Resolution priorities:**
1. Standard library modules (built-in)
2. Local filesystem paths (relative to current file)
3. Absolute filesystem paths
4. Package registry (future extension point)

This approach eliminates complex package managers while maintaining explicit dependency declarations.

### Type System Integration

**Gradual typing supports both rapid prototyping and production reliability:**
- Dynamic regions marked explicitly with `dyn` keyword
- Static typing enforced by default
- Type inference reduces annotation burden
- Compile-time verification prevents runtime type errors

The type checker provides actionable feedback for type mismatches and suggests appropriate conversions or annotations.

### Testing Framework

**Built-in testing combines unit tests with property-based verification:**
```rust
#[test]
fn test_sort_correctness() {
    assert_eq![1, 2, 3, 4], sort([3, 1, 4, 2])
}

#[property]
fn prop_sort_idempotent(xs: Vec<int>) {
    let once = sort(xs)
    let twice = sort(once)
    assert_eq(once, twice)
}
```

Property-based testing generates random inputs to verify behavioral invariants, catching edge cases that unit tests miss.

## Development Workflow

### Interactive Development

**REPL integration supports exploratory programming:**
```bash
ruchy repl                    # Interactive development environment
```

The REPL provides tab completion, type inference feedback, and inline documentation to accelerate learning and debugging.

### Documentation Generation

**Automatic documentation from code structure:**
```bash
ruchy doc                     # Generate HTML documentation
ruchy doc --format=markdown   # Generate Markdown files
```

Documentation extraction processes code comments, type signatures, and usage examples to produce comprehensive API references without additional tooling.

### Performance Analysis

**Built-in profiling identifies optimization opportunities:**
```bash
ruchy --profile script.ruchy         # Generate performance profile
ruchy --compile --benchmark          # Comparative performance testing
```

Profiling output integrates with standard analysis tools to identify bottlenecks and memory usage patterns.

## Quality Assurance

### Static Analysis

**Comprehensive checking prevents common error classes:**
- Null pointer dereference prevention through ownership tracking
- Buffer overflow elimination through bounds checking
- Race condition detection in concurrent code
- Resource leak prevention through automatic cleanup

Static analysis runs automatically during compilation, providing immediate feedback on potential issues.

### Memory Safety

**Rust's ownership model eliminates entire vulnerability classes:**
- Use-after-free prevented through lifetime tracking
- Double-free eliminated through move semantics  
- Memory leaks caught through Drop trait enforcement
- Data races prevented through Send/Sync trait system

These guarantees apply at compile time with zero runtime overhead.

### Reproducible Builds

**Deterministic compilation enables verified deployments:**
- Source code hashing ensures consistent inputs
- Compiler version pinning prevents toolchain drift
- Dependency locking fixes transitive dependencies
- Build environment isolation through containerization

Reproducible builds enable cryptographic verification of deployed binaries against source code.

## Success Metrics

### Performance Targets
- Startup latency: <10ms for typical applications
- Memory overhead: <5% compared to equivalent C programs
- Compilation speed: <2s for 10,000-line projects
- Binary size: <10MB for standard applications

### Developer Experience Goals
- Setup time: <5 minutes from download to productive development
- Error resolution: Clear remediation guidance for 95% of compile errors
- Documentation coverage: All public APIs include examples and explanations
- Tool consistency: Identical behavior across all supported platforms

### Security Objectives
- Default execution: No security overhead for trusted code
- Secure mode: Zero privilege escalation vulnerabilities
- Permission model: Explicit grants for all system access
- Audit capability: Complete logging of security-relevant operations

## Runtime Characteristics

### Memory Model

Ruchy uses reference counting with cycle detection for automatic memory management:

```ruchy
# Automatic memory management
let data = [1, 2, 3, 4, 5]  # Reference counted allocation
let view = data[1:3]        # Shared reference (no copy)
let copy = data.clone()     # Explicit deep copy
# Memory freed when all references dropped
```

**Stack vs Heap:**
- Primitives (int, float, bool): Stack allocated
- Small strings (<24 chars): Stack with SSO
- Collections and objects: Heap with reference counting
- Closures: Heap allocated with captured environment

### Error Handling

Result-based error propagation ensures robust error handling:

```ruchy
# Explicit error handling
fn read_file(path: String) -> Result<String, Error> {
    match File::open(path) {
        Ok(file) => Ok(file.read_to_string()),
        Err(e) => Err(Error::FileNotFound(path))
    }
}

# Using ? operator for propagation
fn process_config() -> Result<Config, Error> {
    let content = read_file("config.toml")?
    let config = parse_toml(content)?
    Ok(config)
}
```

### Concurrency Model

Green threads with async/await for scalable concurrency:

```ruchy
# Async function definition
async fn fetch_data(url: String) -> Result<Data, Error> {
    let response = http::get(url).await?
    response.json().await
}

# Concurrent execution
async fn main() {
    let tasks = urls.map(|url| spawn(fetch_data(url)))
    for task in tasks {
        match task.await {
            Ok(data) => process(data),
            Err(e) => log_error(e)
        }
    }
}
```

### Performance Profile

**Startup Times:**
- REPL initialization: ~100ms
- Script execution: ~50ms (includes compilation)
- Cached execution: <10ms
- Binary execution: <5ms

**Memory Usage:**
- Base interpreter: ~5MB
- Per-thread stack: 2MB default
- Reference count overhead: 16 bytes/object
- String overhead: 24 bytes (inline) or 32 bytes + data

**Execution Speed (vs Python):**
- Numeric operations: 10-50x faster
- String processing: 5-20x faster
- Collection operations: 10-30x faster
- I/O operations: 2-5x faster

### Runtime Introspection

Built-in monitoring and profiling capabilities:

```ruchy
# Performance monitoring
let stats = runtime::stats()
println(f"Memory: {stats.heap_bytes / 1024 / 1024}MB")
println(f"Threads: {stats.thread_count}")
println(f"GC runs: {stats.gc_count}")

# Execution timing
let start = Instant::now()
expensive_operation()
println(f"Elapsed: {start.elapsed()}ms")

# Memory profiling
runtime::track_allocations(true)
suspect_function()
let report = runtime::allocation_report()
```

## Implementation Strategy

### Phase 1: Core Infrastructure ✅
- Basic language syntax and semantics
- Native compilation pipeline
- Essential standard library functions
- Integrated formatting and linting tools

### Phase 2: Development Experience 🔄
- Interactive REPL with completion ✅
- Comprehensive error diagnostics ✅
- Documentation generation ✅
- Testing framework integration 🔄

### Phase 3: Security and Performance
- Opt-in security model implementation
- Performance profiling and optimization
- Cross-compilation support
- Production deployment tooling

This phased approach ensures core functionality stabilizes before adding advanced features, reducing complexity and implementation risk.