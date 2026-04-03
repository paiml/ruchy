# Sub-spec: Publishing — Core Model and Pipeline

**Parent:** [publish.md](../publish.md) Sections 1-6

---

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

