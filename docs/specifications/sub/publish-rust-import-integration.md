# Sub-spec: Publishing — Rust Package Import and Integration

**Parent:** [publish.md](../publish.md) Sections 7-10

---

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

