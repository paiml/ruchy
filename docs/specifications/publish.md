# Ruchy Library Publishing Specification

## Abstract

This document specifies the publishing pipeline for Ruchy libraries to crates.io, enabling zero-friction consumption by standard Rust projects without Ruchy toolchain requirements.

## Core Publishing Model

### Dual-Artifact Strategy

Published crates contain both Ruchy sources and pre-transpiled Rust code:

```
package.crate
├── src/
│   ├── *.ruchy          # Original sources (development)
│   └── gen/*.rs         # Pre-transpiled (consumption)
├── Cargo.toml           # Standard manifest
└── build.rs             # Conditional transpilation
```

### Build Script Architecture

```rust
// build.rs - Enhanced diagnostic version
use std::env;
use std::io::{self, Write};

fn main() {
    let verbose = env::var("RUCHY_BUILD_VERBOSE").is_ok();
    
    // Diagnostic output to stderr (won't pollute build)
    macro_rules! build_log {
        ($($arg:tt)*) => {
            if verbose {
                eprintln!("[ruchy-build] {}", format!($($arg)*));
            }
        }
    }
    
    build_log!("Detecting build environment...");
    
    if let Ok(toolchain) = detect_ruchy_toolchain() {
        build_log!("Found Ruchy {}, transpiling from source", toolchain.version);
        build_log!("Source dir: src/, Target: src/gen/");
        
        match ruchy_build::transpile_project() {
            Ok(stats) => {
                build_log!("Transpiled {} modules in {}ms", 
                          stats.module_count, stats.elapsed_ms);
            }
            Err(e) => {
                eprintln!("Ruchy transpilation failed: {}", e);
                eprintln!("Falling back to pre-generated code");
                println!("cargo:rustc-cfg=use_pregenerated");
            }
        }
    } else {
        build_log!("No Ruchy toolchain found, using pre-generated code");
        println!("cargo:rustc-cfg=use_pregenerated");
    }
}

struct RuchyToolchain {
    version: String,
    path: PathBuf,
}

fn detect_ruchy_toolchain() -> Option<RuchyToolchain> {
    // Check explicit environment variable first
    if let Ok(compiler) = env::var("RUCHY_COMPILER") {
        return Some(RuchyToolchain {
            version: get_version(&compiler)?,
            path: PathBuf::from(compiler),
        });
    }
    
    // Check PATH
    which::which("ruchy").ok().and_then(|path| {
        Some(RuchyToolchain {
            version: get_version(&path)?,
            path,
        })
    })
}
```

## Manifest Configuration

### Package Metadata

```toml
[package]
name = "example-ruchy"
version = "0.1.0"
edition = "2021"
include = ["src/**/*.ruchy", "src/gen/**/*.rs", "build.rs"]

[package.metadata.ruchy]
version = "1.0"                    # Minimum Ruchy version
source_dir = "src"                 # Ruchy source location
output_dir = "src/gen"             # Pre-generated output
transpile_mode = "library"         # library|binary|hybrid
preserve_docs = true               # Maintain doc comments

[dependencies]
# Optional runtime for advanced features
ruchy-runtime = { version = "1.0", optional = true }

[build-dependencies]
ruchy-build = { version = "1.0", optional = true }

[features]
default = ["pregenerated"]
pregenerated = []                  # Use pre-transpiled code
from-source = ["ruchy-build"]      # Transpile from source
ruchy-runtime = ["dep:ruchy-runtime"]
slim = []                          # Exclude .ruchy sources (smaller download)
```

## Publishing Pipeline

### Pre-publish Validation

```bash
#!/bin/bash
# .cargo/publish.sh

set -euo pipefail

echo "==> Validating Ruchy sources"
ruchy check --edition 2021

echo "==> Generating release artifacts"
RUCHY_EMIT=release cargo build --release

echo "==> Verifying generated code"
cd target/package-verify
cp -r ../../src/gen src/
cargo check --no-default-features

echo "==> Running compatibility tests"
cargo test --features pregenerated

echo "==> Package audit"
cargo package --list | grep -E '\.(ruchy|rs)$' | wc -l
```

### Transpilation Modes

```rust
pub enum TranspileMode {
    /// Preserve all Ruchy idioms via runtime
    Full,
    
    /// Zero-dependency pure Rust
    Standalone,
    
    /// Optimize for compile time
    Fast,
    
    /// Maximize runtime performance  
    Release,
}

impl TranspileMode {
    fn configure(&self) -> TranspileConfig {
        match self {
            Full => TranspileConfig {
                inline_threshold: 0,
                monomorphize: false,
                preserve_sources: true,
            },
            Standalone => TranspileConfig {
                inline_threshold: 100,
                monomorphize: true,
                preserve_sources: false,
            },
            Fast => TranspileConfig {
                inline_threshold: 20,
                monomorphize: false,
                preserve_sources: false,
            },
            Release => TranspileConfig {
                inline_threshold: 50,
                monomorphize: true,
                preserve_sources: true,
            },
        }
    }
}
```

## Package Optimization

### Feature-Based Size Control

```toml
# Single crate, feature-controlled content
[features]
default = ["pregenerated"]
pregenerated = []                  # Use pre-transpiled code
from-source = ["ruchy-build"]      # Transpile from source
minimal = []                       # Exclude docs, examples, tests

# Build script respects features
[package]
exclude = [
    "tests/**/*",           # Excluded in minimal builds
    "benches/**/*",         # Excluded in minimal builds
    "examples/**/*",        # Excluded in minimal builds
]
```

### Conditional Compilation

```rust
// build.rs - Feature-aware packaging
fn main() {
    if cfg!(feature = "minimal") {
        // Skip source map generation
        // Exclude debug symbols
        // Minimize metadata
        println!("cargo:rustc-link-arg=-s");
    }
    
    if cfg!(feature = "from-source") {
        transpile_with_full_diagnostics()?;
    } else {
        use_pregenerated_minimal()?;
    }
}
```

## Bidirectional Transpilation Architecture

### Canonical Form Enforcement

Forward transpilation generates deterministic Rust patterns to enable reverse translation:

```rust
pub struct CanonicalPattern {
    id: PatternId,
    rust_template: TokenStream,
    ruchy_ast: AstNode,
}

impl ForwardTranspiler {
    fn emit(&self, node: &RuchyAst) -> TokenStream {
        let pattern = self.canonical_patterns.get(&node.kind);
        let rust = pattern.instantiate(node);
        
        // Embed reversibility marker
        quote! {
            #[cfg_attr(not(doc), ruchy::marker(#node.id))]
            #rust
        }
    }
}
```

### Semantic Preservation

Metadata tracks transformation intent through compilation:

```rust
#[derive(Serialize, Deserialize)]
struct TransformMetadata {
    version: Version,
    mappings: Vec<Mapping>,
    semantic_hashes: HashMap<SpanId, u64>,
}

struct Mapping {
    rust_span: (usize, usize),
    ruchy_span: (usize, usize),
    transform: TransformType,
    confidence: f32,  // 1.0 = lossless, <1.0 = approximation
}
```

### Diff-Based Reconciliation

Pure-Rust contributions undergo incremental reverse transpilation:

```rust
impl DiffReconciler {
    pub fn apply_rust_patch(&self, patch: &Patch) -> Result<String> {
        let mut ruchy_ast = self.parse_original()?;
        
        for hunk in patch.hunks {
            match self.reverse_hunk(&hunk) {
                Ok(ruchy_hunk) => ruchy_ast.apply(ruchy_hunk),
                Err(Irreversible) => {
                    // Mark for manual review
                    self.flag_manual_intervention(&hunk)?;
                }
            }
        }
        
        // Validate round-trip
        let regenerated = self.forward_transpile(&ruchy_ast)?;
        assert_semantic_equivalence(&regenerated, &patch.result)?;
        
        Ok(ruchy_ast.to_source())
    }
}
```

## Rust Package Import Architecture

### Direct Cargo Integration

Ruchy consumes unmodified Rust crates through standard dependency management:

```toml
# Cargo.toml - Standard Rust dependencies work unchanged
[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
sqlx = { version = "0.7", features = ["postgres"] }
```

```rust
// main.ruchy - Direct import of Rust crates
use serde::{Serialize, Deserialize}
use tokio::time::sleep
use sqlx::PgPool

#[derive(Serialize, Deserialize)]
struct User {
    id: i64,
    name: String,
}

async fun fetch_users(pool: &PgPool) -> Vec<User> {
    sqlx::query_as!(User, "SELECT * FROM users")
        |> fetch_all(pool)
        |> await
}
```

### Type System Bridge

Bidirectional type mapping maintains semantic equivalence:

```rust
pub struct TypeBridge {
    rust_to_ruchy: HashMap<RustType, RuchyType>,
    ruchy_to_rust: HashMap<RuchyType, RustType>,
    opaque_types: HashSet<TypeId>,  // Types that pass through unchanged
}

impl TypeBridge {
    fn map_rust_type(&self, ty: &syn::Type) -> RuchyType {
        match ty {
            // Core type mappings
            syn::Type::Path(p) if is_primitive(p) => {
                self.primitive_mapping(p)
            }
            
            // Generic containers
            syn::Type::Path(p) if is_container(p) => {
                let inner = extract_generic_args(p);
                self.container_mapping(p, inner)
            }
            
            // References require lifetime inference
            syn::Type::Reference(r) => {
                RuchyType::Ref {
                    inner: Box::new(self.map_rust_type(&r.elem)),
                    mutability: r.mutability.is_some(),
                }
            }
            
            // Unknown types remain opaque
            _ => RuchyType::Opaque(ty.clone())
        }
    }
}
```

### Lifetime Inference Engine

Automatic lifetime parameter insertion at transpilation:

```rust
impl LifetimeInferrer {
    fn infer_function(&mut self, func: &RuchyFunction) -> Result<Lifetimes> {
        let mut graph = LifetimeGraph::new();
        
        // Build constraint graph from data flow
        for param in &func.params {
            if param.is_reference() {
                graph.add_node(param.id);
            }
        }
        
        // Trace reference propagation through body
        self.analyze_body(&func.body, &mut graph)?;
        
        // Solve constraints using region inference
        let solution = graph.solve()?;
        
        // Generate minimal lifetime parameters
        self.minimize_lifetimes(solution)
    }
}

// Example transformation
// Ruchy: fun parse(input: &str) -> &str
// Rust:  fn parse<'a>(input: &'a str) -> &'a str
```

### Macro Transparency Layer

Rust macros pass through unmodified:

```rust
// Ruchy preserves macro invocations
fun get_user(id: i64) -> Result<User> {
    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1",
        id
    )
    |> fetch_one(&pool)
    |> await
}

// Transpiles to identical macro call
fn get_user(id: i64) -> Result<User> {
    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1",
        id
    )
    .fetch_one(&pool)
    .await
}
```

### Trait Implementation Bridge

External trait implementations with orphan rule compliance:

```rust
// Ruchy: Extending external types
extend DateTime<Utc> {
    fun tomorrow(&self) -> DateTime<Utc> {
        *self + Duration::days(1)
    }
}

// Generated newtype wrapper
#[repr(transparent)]
struct DateTimeExt(pub DateTime<Utc>);

impl DateTimeExt {
    fn tomorrow(&self) -> DateTime<Utc> {
        self.0 + Duration::days(1)
    }
}

impl Deref for DateTimeExt {
    type Target = DateTime<Utc>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

// Automatic conversion at call sites
impl From<DateTime<Utc>> for DateTimeExt {
    fn from(dt: DateTime<Utc>) -> Self { Self(dt) }
}
```

### Zero-Cost Abstraction Guarantee

All interop resolves at compile time:

```rust
// Performance test validating zero overhead
#[bench]
fn interop_overhead() {
    let rust_native = measure(|| {
        serde_json::to_string(&data).unwrap()
    });
    
    let via_ruchy = measure(|| {
        // Ruchy transpiled code calling same function
        ruchy_transpiled::serialize(&data)
    });
    
    assert_eq!(rust_native.instructions, via_ruchy.instructions);
    assert!(via_ruchy.time / rust_native.time < 1.001); // <0.1% overhead
}
```

### API Discovery and Binding Generation

Build-time extraction of Rust crate APIs:

```rust
fn generate_bindings(manifest: &Manifest) -> Result<()> {
    for (name, dep) in &manifest.dependencies {
        // Use cargo metadata for API extraction
        let metadata = MetadataCommand::new()
            .exec()?;
        
        let package = metadata.packages
            .iter()
            .find(|p| p.name == name)
            .ok_or("Package not found")?;
        
        // Generate type signatures
        let signatures = extract_public_api(package)?;
        
        // Create Ruchy binding file
        let binding = RuchyBinding {
            crate_name: name.clone(),
            version: dep.version(),
            items: signatures.to_ruchy_items(),
        };
        
        // Write to generated bindings directory
        let path = format!("target/ruchy-bindings/{}.rbi", name);
        binding.write_to(&path)?;
    }
    Ok(())
}
```

## Consumer Integration Patterns

### Pure Rust Consumer

```toml
# No Ruchy toolchain required
[dependencies]
ruchy-lib = "1.0"
```

Automatically uses pre-generated code via `src/gen/*.rs`.

### Ruchy Project Consumer

```toml
# Full Ruchy integration
[dependencies]
ruchy-lib = { version = "1.0", features = ["from-source"] }
```

Rebuilds from `.ruchy` sources for optimal error messages.

### Hybrid Project

```rust
// lib.rs - Seamless mixing
mod rust_code;

#[path = "gen/ruchy_algo.rs"]
mod ruchy_algo;

pub use ruchy_algo::optimize;
pub use rust_code::validate;
```

## Version Resolution

### Compatibility Matrix

| Ruchy | Min Rust | Transpiler Features |
|-------|----------|-------------------|
| 1.0   | 1.70     | Basic, async/await |
| 1.1   | 1.75     | + Async traits |
| 1.2   | 1.75     | + Const generics |
| 1.3   | 1.80     | + Polonius |

### Version Detection

```rust
fn detect_rust_version() -> Version {
    let output = Command::new("rustc")
        .arg("--version")
        .output()?;
    
    parse_version(&output.stdout)
}

fn select_transpile_target(rust_version: Version) -> Target {
    match rust_version {
        v if v >= Version::new(1, 80, 0) => Target::Rust2024,
        v if v >= Version::new(1, 75, 0) => Target::Rust2021Plus,
        _ => Target::Rust2021,
    }
}
```

## Source Map Strategy

### Embedded Mapping

```rust
// Generated: src/gen/algo.rs
#![allow(dead_code)]
// SOURCE: src/algo.ruchy

#[doc = "Original: src/algo.ruchy:15-27"]
pub fn calculate(input: Vec<f64>) -> f64 {
    // ruchy:17: let sum = input |> fold(0.0, +)
    let sum = input.iter().fold(0.0, |a, b| a + b);
    
    // ruchy:18: sum / input.len() as f64
    sum / input.len() as f64
}
```

### Embedded Metadata

```rust
// Generated header in every transpiled file
pub const RUCHY_METADATA: &str = concat!(
    "transpiler=", env!("RUCHY_VERSION"),
    ";rust_target=", env!("RUSTC_VERSION"),
    ";timestamp=", env!("RUCHY_BUILD_TIME"),
);

// Runtime compatibility check
#[cfg(not(use_pregenerated))]
fn verify_compatibility() {
    let required = parse_rust_version(RUCHY_METADATA);
    let current = rustc_version::version().unwrap();
    
    if current < required {
        panic!(
            "This library requires Rust >= {} (transpiled with Ruchy {}), \
             but you have Rust {}. \
             Consider using the 'from-source' feature to rebuild.",
            required, 
            extract_transpiler_version(RUCHY_METADATA),
            current
        );
    }
}
```

### Debug Information

```rust
#[cfg(debug_assertions)]
macro_rules! ruchy_trace {
    ($line:expr) => {
        if std::env::var("RUCHY_TRACE").is_ok() {
            eprintln!("ruchy:{}: {}", $line, file!());
        }
    };
}
```

## Source Synchronization (Poka-yoke)

### Incremental Verification

```rust
// Cached incremental verification for speed
use std::collections::HashMap;
use sha2::{Sha256, Digest};

pub struct VerifyCache {
    entries: HashMap<PathBuf, CacheEntry>,
}

struct CacheEntry {
    source_hash: [u8; 32],
    generated_hash: [u8; 32],
    last_verified: SystemTime,
}

impl VerifyCache {
    pub fn verify_incremental(&mut self, sources: &[PathBuf]) -> Result<()> {
        let cache_path = Path::new("target/.ruchy-verify-cache");
        self.load_cache(cache_path)?;
        
        let mut errors = Vec::new();
        
        for source in sources {
            let source_hash = hash_file(source)?;
            let gen_path = source.with_extension("rs");
            
            // Skip if unchanged since last verify
            if let Some(entry) = self.entries.get(source) {
                if entry.source_hash == source_hash {
                    if gen_path.exists() {
                        let gen_hash = hash_file(&gen_path)?;
                        if entry.generated_hash == gen_hash {
                            continue; // Still in sync
                        }
                    }
                }
            }
            
            // Only transpile changed files
            let expected = transpile_single(source)?;
            let actual = std::fs::read_to_string(&gen_path)?;
            
            if expected != actual {
                errors.push(format!("{}: drift detected", source.display()));
            } else {
                // Update cache
                self.entries.insert(source.clone(), CacheEntry {
                    source_hash,
                    generated_hash: hash_file(&gen_path)?,
                    last_verified: SystemTime::now(),
                });
            }
        }
        
        self.save_cache(cache_path)?;
        
        if !errors.is_empty() {
            Err(format!("Verification failed:\n{}", errors.join("\n")))
        } else {
            Ok(())
        }
    }
}

fn hash_file(path: &Path) -> Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    let mut file = File::open(path)?;
    io::copy(&mut file, &mut hasher)?;
    Ok(hasher.finalize().into())
}
```

### Pre-commit Hook (Fast)

```bash
#!/bin/bash
# .git/hooks/pre-commit
# Runs in <100ms for unchanged files

if command -v ruchy &> /dev/null; then
    ruchy verify --incremental --changed-only
fi
```

## Publishing Checklist

### Automated Validation

```yaml
# .github/workflows/publish.yml
name: Publish
on:
  push:
    tags: ['v*']

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: ruchy-lang/setup@v1
      
      - name: Validate package
        run: |
          ruchy verify
          cargo package --no-verify
          
      - name: Test consumers
        run: |
          ./scripts/test-consumer-rust.sh
          ./scripts/test-consumer-ruchy.sh
          
      - name: Publish
        run: cargo publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}
```

### Manual Requirements

1. Update CHANGELOG.md with breaking changes
2. Verify minimum supported Rust version
3. Run `cargo semver-checks` for API compatibility
4. Update compatibility matrix in README
5. Tag release with semantic version

## Performance Guarantees

```rust
#[test]
fn publish_overhead() {
    let baseline = measure_rust_compile("pure-rust");
    let with_ruchy = measure_rust_compile("with-pregenerated");
    
    assert!(with_ruchy / baseline < 1.05); // <5% overhead
}

#[test]
fn package_size() {
    let size = package_size("example.crate");
    let rust_only = size_without_ruchy_sources();
    
    assert!(size / rust_only < 1.3); // <30% size increase
}
```

## Security Model

### Build Isolation

- No network access during transpilation
- Deterministic output (reproducible builds)
- No filesystem access outside project root
- Sandboxed macro expansion

### Supply Chain

```toml
[profile.release]
codegen-units = 1
lto = true
strip = true
panic = "abort"

[profile.release.build-override]
opt-level = 0  # Fast build scripts
```

## Tooling Integration

### cargo-ruchy Subcommand

```rust
// Unified workflow orchestration
impl CargoRuchy {
    pub fn commands() -> Vec<Command> {
        vec![
            // Development commands
            Command::new("test")
                .about("Test both source and generated code")
                .action(|args| {
                    run_tests("--features from-source")?;
                    run_tests("--features pregenerated")?;
                    Ok(())
                }),
            
            // Publishing commands
            Command::new("publish")
                .about("Publish with full validation")
                .action(|args| {
                    verify_sync()?;
                    generate_release_artifacts()?;
                    validate_both_modes()?;
                    run_semver_check()?;
                    cargo_publish(args)
                }),
            
            // Maintenance commands
            Command::new("sync")
                .about("Regenerate all transpiled code")
                .action(|_| regenerate_all()),
            
            Command::new("upgrade")
                .about("Upgrade transpiler and regenerate")
                .action(|_| {
                    upgrade_ruchy_build()?;
                    regenerate_all()
                }),
        ]
    }
}
```

### Integration Points

```toml
# .cargo/config.toml
[alias]
ruchy-test = "ruchy test"
ruchy-publish = "ruchy publish --validate"
ruchy-check = "ruchy verify --strict"
```

## Migration Path

### Gradual Adoption

```bash
# Phase 1: Initialize hybrid project
ruchy init --mode=hybrid

# Phase 2: Migrate hot paths
ruchy migrate src/bottleneck.rs --optimize

# Phase 3: Publish dual-mode crate
cargo publish --features "pregenerated,from-source"
```

### Deprecation Strategy

```rust
#[deprecated(
    since = "2.0.0",
    note = "Use `ruchy::transpile!` macro instead"
)]
pub fn old_api() {
    compile_error!("This API requires Ruchy 1.x");
}
```

## Pure-Rust Contribution Workflow

### Contributing Without Ruchy Toolchain

```markdown
# CONTRIBUTING.md excerpt

## For Rust-Only Contributors

You can contribute without installing Ruchy:

1. Modify only files in `src/gen/*.rs`
2. Open PR with `[rust-only]` prefix
3. CI bot will attempt reverse-transpilation
4. If successful: Bot updates `.ruchy` sources
5. If failed: Maintainer manually reconciles

### Automated Patch Application

```yaml
# .github/workflows/rust-contrib.yml
on:
  pull_request:
    paths:
      - 'src/gen/**/*.rs'

jobs:
  reverse-transpile:
    runs-on: ubuntu-latest
    steps:
      - name: Detect Rust-only contribution
        id: check
        run: |
          if ! git diff --name-only origin/main..HEAD | grep -q '\.ruchy$'; then
            echo "rust_only=true" >> $GITHUB_OUTPUT
          fi
      
      - name: Attempt reverse transpilation
        if: steps.check.outputs.rust_only == 'true'
        run: |
          ruchy reverse \
            --from src/gen \
            --to src \
            --mode=patch
      
      - name: Commit reconciled sources
        uses: actions-bot@v1
        with:
          message: "Auto-reconcile Rust contribution"
```

## Build Infrastructure Architecture

### Bootstrapping Requirements

```toml
# ruchy-build/Cargo.toml
[package]
name = "ruchy-build"
version = "1.0.0"
description = "Pure-Rust build utilities (no Ruchy code)"

# Critical: This crate MUST be pure Rust
# It bootstraps the entire Ruchy ecosystem
# Including any .ruchy files would create circular dependency

[dependencies]
which = "4.0"      # Find ruchy compiler
toml = "0.7"       # Parse configs
sha2 = "0.10"      # Cache verification
```

### Toolchain Management

```toml
# ruchy-toolchain.toml - Project-specific version pinning
[toolchain]
channel = "1.2.0"              # Exact version for reproducibility
components = ["ruchy-std"]     # Required components
profile = "release"            # Transpilation profile

[fallback]
min_version = "1.1.0"          # Minimum compatible version
max_version = "1.3.0"          # Maximum compatible version
```

```rust
// Toolchain resolution with fallback
impl ToolchainResolver {
    pub fn resolve(&self) -> Result<Compiler> {
        // Priority order:
        // 1. Exact pinned version
        if let Ok(exact) = self.get_exact_version() {
            return Ok(exact);
        }
        
        // 2. Compatible range
        if let Ok(compat) = self.find_compatible() {
            eprintln!("Warning: Using ruchy {} instead of {}", 
                     compat.version, self.pinned_version);
            return Ok(compat);
        }
        
        // 3. Auto-download
        if env::var("CI").is_ok() || env::var("RUCHY_AUTO_INSTALL").is_ok() {
            return self.download_and_cache();
        }
        
        // 4. Fail with actionable error
        Err(format!(
            "Ruchy {} required but not found.\n\
             Install with: cargo install ruchy-cli --version {}\n\
             Or set RUCHY_AUTO_INSTALL=1 to download automatically",
            self.pinned_version, self.pinned_version
        ))
    }
}
```

## Migration Path

### Gradual Adoption

```bash
# Phase 1: Initialize hybrid project
ruchy init --mode=hybrid

# Phase 2: Migrate hot paths
ruchy migrate src/bottleneck.rs --optimize

# Phase 3: Publish dual-mode crate
cargo publish --features "pregenerated,from-source"
```

### Deprecation Strategy

```rust
#[deprecated(
    since = "2.0.0",
    note = "Use `ruchy::transpile!` macro instead"
)]
pub fn old_api() {
    compile_error!("This API requires Ruchy 1.x");
}
```

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| Missing `.ruchy` files | Check `include` in Cargo.toml |
| Transpilation fails | Pin `ruchy-build` version |
| Consumer build fails | Verify `pregenerated` feature |
| Source maps broken | Enable `preserve_sources` |

### Debug Mode

```bash
RUCHY_DEBUG=1 cargo build
# Emits transpilation details to stderr
# Preserves intermediate representations
# Generates trace logs in target/ruchy-debug/
```

## Bidirectional Ecosystem Integration Summary

### Architectural Coherence

The publishing specification achieves true bidirectional integration through three pillars:

1. **Forward Path (Ruchy → Rust)**: Deterministic transpilation with canonical patterns
2. **Reverse Path (Rust → Ruchy)**: Pattern recognition with semantic preservation
3. **Import Path (Rust crates → Ruchy)**: Zero-cost type bridging with automatic bindings

This creates a complete ecosystem loop where:
- Ruchy libraries publish as standard Rust crates
- Rust developers contribute without Ruchy knowledge
- Ruchy projects consume any Rust crate unchanged
- All transformations maintain semantic equivalence

### Performance Invariants

```rust
// Core guarantee: Zero runtime overhead
const PERFORMANCE_INVARIANTS: &[Invariant] = &[
    Invariant::TranspilationOverhead < 1.15,  // 15% compile time
    Invariant::RuntimeOverhead == 1.00,       // 0% runtime cost
    Invariant::BinarySize < 1.05,            // 5% size increase
    Invariant::InteropCost == 0,             // Zero-cost abstraction
];
```

### Quality Gates

Every published crate passes through:
1. Source synchronization verification (`ruchy verify`)
2. Bidirectional round-trip testing
3. API compatibility checking (`cargo semver-checks`)
4. Performance regression testing
5. Cross-version compatibility matrix

This ensures ecosystem stability while enabling rapid iteration.

## Known Risks and Mitigations

### Risk Matrix

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Source/Generated Drift | High | Medium | `ruchy verify` in CI, pre-commit hooks |
| Transpiler Bugs | High | Low | Differential testing, fuzzing, stable/beta channels |
| Package Bloat | Medium | High | Slim variants, compression, CDN caching |
| Version Matrix Explosion | Medium | Medium | Clear compatibility errors, toolchain pinning |
| Build Script Failures | Medium | Low | Fallback modes, verbose diagnostics, support matrix |

### Mitigation Strategies

```rust
// Differential testing framework
#[test]
fn verify_semantic_equivalence() {
    let ruchy_src = "test.ruchy";
    let rust_gen = transpile(ruchy_src)?;
    
    // Run both through interpreter
    let ruchy_result = ruchy_interpret(ruchy_src)?;
    let rust_result = rust_interpret(rust_gen)?;
    
    assert_eq!(ruchy_result, rust_result);
}

// Automated compatibility testing
#[test_matrix]
fn test_across_rust_versions() {
    for rust_version in ["1.70", "1.75", "1.80"] {
        docker_test(rust_version)?;
    }
}
```

## Future Considerations

- WebAssembly package support
- Incremental transpilation caching
- Cross-compilation matrices
- Package signing with transpiler version

## Experimental Features

### Unstable Transpilation Target

```toml
# Cargo.toml - Opt into experimental features
[package.metadata.ruchy]
transpile_mode = "unstable"    # Allows nightly Rust features
required_rust = "nightly-2024-01-15"

[features]
unstable = ["ruchy-runtime/unstable"]
```

```rust
// Generated code with nightly features
#![feature(async_fn_in_trait)]
#![feature(return_position_impl_trait_in_trait)]

// Metadata enforcement
const _REQUIRE_NIGHTLY: () = {
    #[cfg(not(nightly))]
    compile_error!(
        "This library requires Rust nightly.\n\
         Install with: rustup install nightly-2024-01-15\n\
         Use with: cargo +nightly build"
    );
};
```

### Progressive Stability

```rust
pub enum TranspileTarget {
    /// Conservative - Works on 6-month-old stable
    Stable { min_rust: Version },
    
    /// Current - Requires latest stable
    Current,
    
    /// Experimental - Requires specific nightly
    Unstable { nightly_date: NaiveDate },
}

impl TranspileTarget {
    fn validate_compiler(&self) -> Result<()> {
        let rustc = detect_rustc_version()?;
        
        match self {
            Stable { min_rust } if rustc < *min_rust => {
                Err(format!("Requires Rust >= {}", min_rust))
            }
            Unstable { .. } if !rustc.is_nightly() => {
                Err("Unstable features require nightly Rust".into())
            }
            _ => Ok(())
        }
    }
}
```