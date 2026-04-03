# Sub-spec: Publishing — Workflow, Tooling, and Ecosystem

**Parent:** [publish.md](../publish.md) Sections 11-18

---

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