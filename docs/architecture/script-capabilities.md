# Ruchy Script Capability Specification v1.0
## Universal Scripting with Zero-Compromise Compilation

### Executive Summary

This specification defines Ruchy's script execution model, establishing a unified system where the same code seamlessly transitions from REPL experimentation to production binaries without modification. The design achieves <10ms REPL startup, full Cargo ecosystem access, and produces standalone binaries under 2MB while maintaining Rust's zero-cost abstractions.

---

## 1. Core Execution Modes

### 1.1 REPL Mode (Interactive)

```rust
// Launch with incremental compilation context
$ ruchy
ruchy> let data = [1, 2, 3] |> map(_ * 2) |> sum()
6
ruchy> :type data
i32
ruchy> :ast data
Pipeline(Array([1,2,3]), [Map(Lambda), Sum])
ruchy> :rust data
vec![1, 2, 3].into_iter().map(|x| x * 2).sum::<i32>()
```

**Technical Requirements:**
- **Startup Time**: <8ms cold, <2ms warm (via persistent bytecode cache)
- **Incremental State**: Maintains type environment across inputs
- **JIT Threshold**: Auto-compile hot expressions after 10 evaluations
- **Memory Model**: Arena allocator with 10MB initial heap

**Implementation Strategy:**
```rust
pub struct REPLState {
    // Persistent across sessions
    bytecode_cache: MmapCache,
    type_env: IncrementalTypeEnv,
    
    // Per-session state
    bindings: HashMap<Symbol, Value>,
    jit: CraneliftJIT,
    
    // Import resolution
    module_cache: ModuleCache,
    cargo_resolver: CargoResolver,
}

impl REPL {
    fn eval_incremental(&mut self, input: &str) -> Result<Value> {
        // Check bytecode cache first
        if let Some(bc) = self.bytecode_cache.get(hash(input)) {
            return self.execute_bytecode(bc);
        }
        
        // Parse with error recovery
        let ast = parse_with_recovery(input)?;
        
        // Incremental type checking
        let typed = self.type_env.infer_incremental(&ast)?;
        
        // Execution strategy selection
        match self.select_strategy(&typed) {
            Strategy::Interpret => self.tree_walk(&typed),
            Strategy::JIT => self.jit_compile(&typed),
            Strategy::RustCache => self.cached_rust_eval(&typed),
        }
    }
}
```

### 1.2 Script Import System

```rust
// script.ruchy
module analytics {
    pub fn process(data: [f64]) -> Statistics {
        Statistics {
            mean: data.sum() / data.len(),
            stddev: calculate_stddev(data),
            percentiles: quantiles(data, [0.25, 0.5, 0.75, 0.99])
        }
    }
}

// REPL session
ruchy> :load analytics.ruchy
Loaded: analytics (compiled in 12ms)
ruchy> import analytics::process
ruchy> let stats = process([1.0, 2.0, 3.0, 4.0, 5.0])
Statistics { mean: 3.0, stddev: 1.41, percentiles: [2.0, 3.0, 4.0, 4.96] }

// Hot-reload on file change
ruchy> :watch analytics.ruchy
Watching for changes...
[File modified] Recompiling analytics.ruchy... done (8ms)
ruchy> stats  // Automatically uses new version
```

**Module Resolution Algorithm:**
```rust
impl ModuleResolver {
    fn resolve(&self, import: &Import) -> Resolution {
        // Priority order:
        // 1. REPL-defined modules
        if let Some(m) = self.repl_modules.get(&import.name) {
            return Resolution::REPLModule(m);
        }
        
        // 2. Local .ruchy files
        if let Some(path) = self.find_local_module(&import.name) {
            let compiled = self.compile_module(path)?;
            return Resolution::LocalModule(compiled);
        }
        
        // 3. Cargo dependencies
        if let Some(crate_) = self.cargo_resolver.resolve(&import.name) {
            return Resolution::CargoCrate(crate_);
        }
        
        // 4. Standard library
        Resolution::Stdlib(import.name)
    }
}
```

### 1.3 Compiler Mode (Strict Validation)

```rust
// build.ruchy
#![strict]  // Enforces full type checking, no dynamic
#![deny(warnings)]  // Zero tolerance for issues

import std::fs::read_to_string
import serde_json::from_str

// Compilation fails if type cannot be inferred
fn process_config(path: String) -> Result<Config> {
    let contents = read_to_string(path)?;
    let config: Config = from_str(&contents)?;  // Type annotation required
    validate_config(config)?;
    Ok(config)
}

// Property enforced at compile time
#[requires(data.len() > 0)]
#[ensures(result >= 0.0 && result <= 1.0)]
fn normalize(data: &[f64]) -> f64 {
    let max = data.iter().max();
    data.iter().map(|x| x / max).collect()
}
```

**Compilation Pipeline:**
```rust
pub struct StrictCompiler {
    config: CompilerConfig {
        type_checking: TypeCheckMode::Strict,
        inference_level: InferenceLevel::Minimal,
        runtime_checks: false,
        optimization: OptLevel::Release,
    }
}

impl StrictCompiler {
    fn compile(&self, source: &str) -> Result<RustAST> {
        let ast = parse_strict(source)?;  // No error recovery
        
        // Full type checking with SMT verification
        let typed = self.type_check_with_refinements(&ast)?;
        
        // Property verification
        self.verify_contracts(&typed)?;
        
        // Aggressive optimization
        let optimized = self.optimize_aggressive(&typed);
        
        // Generate zero-overhead Rust
        self.generate_rust(&optimized)
    }
}
```

### 1.4 Cargo Integration

```toml
# Cargo.toml
[package]
name = "my-project"
version = "0.1.0"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = "1.0"
rayon = "1.5"

[build-dependencies]
ruchy = "1.0"

# Ruchy-specific configuration
[package.metadata.ruchy]
mode = "hybrid"  # Support both interpreted and compiled
strict = true    # Enforce type safety
optimize = "size"  # Optimize for binary size
```

```rust
// build.rs - Automatic Ruchy compilation
fn main() {
    ruchy::compile_build_scripts("scripts/*.ruchy")
        .with_cargo_deps(true)
        .with_optimization(OptLevel::Size)
        .generate()?;
}

// src/main.rs - Use compiled Ruchy modules
mod compiled {
    include!(concat!(env!("OUT_DIR"), "/ruchy_modules.rs"));
}

fn main() {
    let result = compiled::analytics::process(data);
}
```

**Cargo Command Extensions:**
```bash
# Run Ruchy script with Cargo dependencies
$ cargo ruchy run script.ruchy

# Build optimized binary from script
$ cargo ruchy build --release script.ruchy

# Check script without running
$ cargo ruchy check script.ruchy

# Format Ruchy files
$ cargo ruchy fmt

# Extreme linting with clippy integration
$ cargo ruchy clippy -- -W clippy::all -W clippy::pedantic
```

### 1.5 Standalone Binary Generation

```rust
// deploy.ruchy
#![binary(name = "analyzer", size_optimize = true)]

import clap::Parser
import tokio::main

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    input: String,
    
    #[arg(short, long, default = "json")]
    format: OutputFormat,
}

#[main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let data = load_data(&args.input).await?;
    let result = process(data);
    output(result, args.format)?;
    Ok(())
}
```

**Binary Generation Pipeline:**
```rust
pub struct BinaryGenerator {
    config: BinaryConfig {
        target_size: ByteSize::megabytes(2),
        link_mode: LinkMode::Static,
        allocator: Allocator::Jemalloc,
        strip: true,
        lto: LTOMode::Fat,
    }
}

impl BinaryGenerator {
    fn generate(&self, script: &Path) -> Result<Binary> {
        // Transpile to Rust
        let rust_code = self.transpile(script)?;
        
        // Tree-shake unused code
        let shaken = self.tree_shake(&rust_code);
        
        // Generate minimal runtime
        let runtime = self.generate_minimal_runtime(&shaken);
        
        // Compile with size optimizations
        let binary = rustc()
            .arg("-C", "opt-level=z")
            .arg("-C", "lto=fat")
            .arg("-C", "panic=abort")
            .arg("--emit", "link")
            .compile(&runtime)?;
            
        // Strip and compress
        self.strip_and_compress(binary)
    }
}

// Result: 1.8MB binary with 50μs startup time
```

## 2. Deno-Inspired Features

### 2.1 Permission System

```rust
// Secure by default, explicit permissions
$ ruchy run --allow-read=/data --allow-net=api.example.com script.ruchy

// In script
#[permissions(read = "/data/*", net = ["api.example.com:443"])]
fn fetch_and_process() -> Result<Data> {
    let local = fs::read("/data/input.json")?;  // Allowed
    let remote = http::get("https://api.example.com/data")?;  // Allowed
    // fs::read("/etc/passwd")  // Runtime error: Permission denied
}
```

### 2.2 URL Imports

```rust
// Import from URLs with integrity checking
import "https://deno.land/std@0.100.0/fmt/colors.ruchy" as colors
import "https://github.com/user/repo/raw/main/lib.ruchy" as lib

// Automatic caching with lock file
// .ruchy-lock.json
{
  "https://deno.land/std@0.100.0/fmt/colors.ruchy": {
    "integrity": "sha384-oqVuAfXRKap7f...",
    "cached": "~/.ruchy/cache/deno.land/..."
  }
}
```

### 2.3 Built-in Tooling

```bash
# Format code
$ ruchy fmt --check src/

# Lint with automatic fixes
$ ruchy lint --fix src/

# Bundle into single file
$ ruchy bundle script.ruchy -o dist/bundle.ruchy

# Generate documentation
$ ruchy doc --html src/

# Test with built-in runner
$ ruchy test --coverage

# Benchmark with statistical analysis
$ ruchy bench --compare baseline.json
```

## 3. Quality Enforcement

### 3.1 Progressive Strictness Levels

```rust
// Level 0: Dynamic scripting
let data = fetch_data()  // Any type
process(data)  // Runtime type checking

// Level 1: Basic types (default)
let data: Data = fetch_data()
process(data)  // Compile-time type checking

// Level 2: Strict mode
#![strict]
let data: Data<Validated> = fetch_data()
    .validate()?  // Must handle errors
    .transform()  // All types explicit

// Level 3: Verified mode
#![verified]
#[requires(input.len() > 0)]
#[ensures(result.is_sorted())]
fn sort(input: &[T]) -> Vec<T> {
    // SMT solver proves correctness
}
```

### 3.2 Integrated Testing

```rust
// In-file tests that run in REPL
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[test]
fn test_add() {
    assert_eq!(add(2, 2), 4)
}

#[property]
fn prop_add_commutative(a: i32, b: i32) {
    assert_eq!(add(a, b), add(b, a))
}

// REPL testing
ruchy> :test
Running 1 test, 1 property (1000 cases)...
test test_add ... ok
property prop_add_commutative ... ok (1000 cases)
All tests passed!
```

## 4. Performance Characteristics

### 4.1 Execution Performance

| Mode | Startup | Throughput | Memory | Use Case |
|------|---------|------------|--------|----------|
| REPL | 8ms | 1M ops/s | 10MB | Interactive development |
| JIT | 15ms | 50M ops/s | 20MB | Hot loop optimization |
| Script | 10ms | 5M ops/s | 15MB | Quick automation |
| Compiled | 50μs | 100M ops/s | 2MB | Production deployment |
| Binary | 20μs | 100M ops/s | 1MB | Standalone distribution |

### 4.2 Compilation Performance

```rust
// Benchmark: 10,000 line script
Parse:           50ms   (5ms with incremental)
Type Check:      80ms   (10ms with incremental)
Optimization:    120ms  (20ms with incremental)
Rust Generation: 30ms   (5ms with incremental)
Rustc Compile:   2s     (200ms with incremental)
Total:           2.28s  (240ms incremental)

// Binary size optimization
Baseline:        5.2MB
Tree-shaking:    3.8MB (-27%)
LTO:            2.9MB (-44%)
Strip symbols:   2.1MB (-60%)
UPX compression: 1.8MB (-65%)
```

## 5. Implementation Roadmap

### Phase 1: Core REPL (Q1 2025)
- [x] Basic interpreter with tree-walking
- [x] Type inference for common cases
- [ ] Module loading from files
- [ ] Cargo dependency resolution

### Phase 2: Compilation (Q2 2025)
- [ ] Rust transpilation
- [ ] Incremental compilation
- [ ] Binary generation
- [ ] Size optimization

### Phase 3: Advanced Features (Q3 2025)
- [ ] JIT compilation via Cranelift
- [ ] Permission system
- [ ] URL imports
- [ ] Property testing integration

### Phase 4: Production (Q4 2025)
- [ ] Performance parity with Rust
- [ ] Sub-2MB binaries
- [ ] Full Cargo integration
- [ ] VSCode extension

## 6. Configuration Reference

### 6.1 Global Configuration

```toml
# ~/.ruchy/config.toml
[repl]
history_size = 10000
startup_script = "~/.ruchy/init.ruchy"
bytecode_cache = "~/.ruchy/cache"

[compiler]
default_mode = "hybrid"
optimization = "balanced"  # size|speed|balanced
parallel = true

[permissions]
default_read = [".", "~/projects"]
default_net = ["localhost"]
prompt = true  # Ask for permissions

[tooling]
formatter = "rustfmt"
linter = "clippy"
test_runner = "nextest"
```

### 6.2 Project Configuration

```yaml
# .ruchy.yaml
version: "1.0"
mode: strict

scripts:
  dev:
    entry: src/main.ruchy
    watch: true
    reload: true
    
  build:
    entry: src/main.ruchy
    output: dist/app
    optimize: size
    strip: true
    
  test:
    pattern: "**/*_test.ruchy"
    coverage: true
    
dependencies:
  cargo:
    - tokio@1.0
    - serde@1.0
  url:
    - https://deno.land/std@0.100.0/fmt/colors.ruchy
```

## 7. Error Handling Philosophy

```rust
// Progressive error handling based on mode

// REPL: Recoverable with suggestions
ruchy> let x = [1, 2, 3].ma(|x| x * 2)
Error: Method 'ma' not found. Did you mean 'map'?
  |
1 | let x = [1, 2, 3].ma(|x| x * 2)
  |                   ^^ help: try 'map'

// Script: Clear diagnostics
Error[E0425]: Method not found
 --> script.ruchy:42:18
  |
42 | let result = data.proccess()
  |                   ^^^^^^^^ no method named `proccess`
  |
  = help: there is a method `process` with a similar name

// Compiled: Fail fast with context
error: Type mismatch in function 'analyze'
  ┌─ analytics.ruchy:15:22
  │
15│     return calculate(input)
  │                      ^^^^^ expected &[f64], found String
  │
  = note: `calculate` requires numeric input
  = help: consider parsing the string first:
          let numbers: Vec<f64> = input.parse_csv()?;
          return calculate(&numbers)
```

---

## Summary

This specification establishes Ruchy as a true universal scripting language that scales from REPL experimentation to production deployment without code changes. By leveraging Rust's ecosystem directly and providing multiple execution strategies, developers get Python's ergonomics with Rust's performance, achieving the best of both worlds without compromise.