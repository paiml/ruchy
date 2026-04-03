# Sub-spec: Cargo Integration — Quality, Showcase, and Roadmap

**Parent:** [cargo-integration-ruchy.md](../cargo-integration-ruchy.md) Sections 5-End

---


### Mutation Testing Strategy

```rust
pub struct MutationTester {
    mutators: Vec<Box<dyn Mutator>>,
    test_runner: TestRunner,
}

impl MutationTester {
    pub fn verify_coverage(&mut self, code: &str) -> Result<f64> {
        let mutations = self.generate_mutations(code);
        let total = mutations.len();
        let mut killed = 0;
        
        for mutation in mutations {
            let mutated = self.apply_mutation(code, &mutation);
            
            // Run tests against mutation
            let result = self.test_runner.run(&mutated);
            
            if result.any_failed() {
                killed += 1;
            } else {
                println!("Survived mutation: {:?}", mutation);
            }
        }
        
        let coverage = killed as f64 / total as f64;
        if coverage < 0.8 {
            Err(Error::InsufficientMutationCoverage(coverage))
        } else {
            Ok(coverage)
        }
    }
}
```

### Performance Validation

```rust
#[test]
fn verify_performance_bounds() {
    let ruchy_source = include_str!("../benchmarks/suite.ruchy");
    let rust_baseline = include_str!("../benchmarks/baseline.rs");
    
    let ruchy_binary = compile_ruchy(ruchy_source);
    let rust_binary = compile_rust(rust_baseline);
    
    // Sequential performance
    let ruchy_seq = benchmark_sequential(&ruchy_binary);
    let rust_seq = benchmark_sequential(&rust_binary);
    assert!(ruchy_seq < rust_seq * 1.05, "Sequential overhead > 5%");
    
    // Actor performance  
    let ruchy_actor = benchmark_actors(&ruchy_binary);
    let rust_async = benchmark_async(&rust_binary);
    assert!(ruchy_actor < rust_async * 1.10, "Actor overhead > 10%");
}
```

## Complete Showcase Package

### Project Structure

```
ruchy-showcase/
├── Cargo.toml
├── build.rs
├── src/
│   ├── lib.rs          # Rust entry point
│   ├── main.ruchy      # Application
│   └── domain/
│       ├── mod.ruchy
│       └── types.ruchy
├── tests/
│   └── properties.ruchy
└── benches/
    └── perf.rs
```

### Main Application

```rust
// src/main.ruchy
use serde::{Serialize, Deserialize};
use tokio::net::TcpListener;

// Algebraic data types
type Result<T, E> = Ok(T) | Err(E)

// Refinement types with compile-time verification
type Port = u16 where |x| 1024 <= x <= 65535
type Email = String where |s| s.contains('@') && s.len() < 255

// Pattern matching with guards and destructuring
fun validate_request(req: Request) -> Result<Valid, Error> {
    match req {
        Request { email, port, .. } 
            when email.is_valid() && port.is_privileged() => {
            Ok(Valid::new(email, port))
        },
        Request { email, .. } when !email.is_valid() => {
            Err(Error::InvalidEmail(email))
        },
        _ => Err(Error::InvalidPort)
    }
}

// Actor with supervision
actor ConnectionPool {
    connections: Vec<Connection>,
    max_size: usize where |x| x > 0 && x <= 1000
    
    receive acquire() -> Result<Connection, PoolError> {
        self.connections.pop()
            .ok_or(PoolError::Exhausted)
    }
    
    receive release(conn: Connection) {
        if self.connections.len() < self.max_size {
            self.connections.push(conn);
        }
    }
    
    supervise with restart_on_panic(max_restarts: 3)
}

// Property specifications
#[property]
fun prop_connection_pool_bounded(
    pool: ConnectionPool,
    ops: Vec<PoolOp>
) -> bool {
    ops.iter().fold(pool, |p, op| {
        let p' = p.apply(op);
        assert!(p'.connections.len() <= p'.max_size);
        p'
    });
    true
}

// Compile-time verified unsafe
#[unsafe_proof(method = "smt")]
unsafe fun zero_copy_parse(bytes: &[u8]) -> Header 
    where bytes.len() >= size_of::<Header>()
{
    ptr::read(bytes.as_ptr() as *const Header)
}

// Entry point with pipeline operators
fun main() -> Result<(), Box<dyn Error>> {
    let config = fs::read_to_string("config.toml")?
        |> toml::from_str()?
        |> validate_config()?;
    
    let pool = ConnectionPool::spawn(
        max_size: config.pool_size
    );
    
    TcpListener::bind((config.host, config.port))?
        |> accept_loop(pool)
        |> instrument_with_metrics()
        |> run_until_shutdown()
        |> await
}
```

### Generated Rust (Abbreviated)

```rust
// target/ruchy-gen/main.rs
#[derive(Debug, Clone)]
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Refinement type with compile-time verification
#[derive(Debug, Clone, Copy)]
pub struct Port(u16);

impl Port {
    #[inline]
    pub const unsafe fn new_unchecked(value: u16) -> Self {
        Self(value)
    }
    
    pub fn new(value: u16) -> std::result::Result<Self, String> {
        if value >= 1024 && value <= 65535 {
            Ok(Self(value))
        } else {
            Err(format!("Port {} out of range", value))
        }
    }
}

// Actor compiled to tokio task
pub struct ConnectionPool {
    state: Arc<Mutex<ConnectionPoolState>>,
    supervisor: Supervisor,
}

impl ConnectionPool {
    pub fn spawn(max_size: usize) -> Handle<Self> {
        let (tx, mut rx) = mpsc::channel(1000);
        
        let state = Arc::new(Mutex::new(ConnectionPoolState {
            connections: Vec::new(),
            max_size,
        }));
        
        let supervisor = Supervisor::new(
            RestartStrategy::RestartOnPanic { max_restarts: 3 }
        );
        
        let actor = Self { state: state.clone(), supervisor };
        
        supervisor.spawn(async move {
            while let Some(msg) = rx.recv().await {
                actor.handle_message(msg).await;
            }
        });
        
        Handle { tx, state }
    }
}

// Property test via proptest
#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn prop_connection_pool_bounded(
            pool in arb_connection_pool(),
            ops in prop::collection::vec(arb_pool_op(), 0..100)
        ) {
            let final_state = ops.iter().fold(pool, |p, op| {
                let p_next = p.apply(op);
                prop_assert!(p_next.connections.len() <= p_next.max_size);
                p_next
            });
            prop_assert!(final_state.connections.len() <= final_state.max_size);
        }
    }
}

// Verified unsafe with zero-copy parsing
#[inline]
pub unsafe fn zero_copy_parse(bytes: &[u8]) -> Header {
    debug_assert!(bytes.len() >= std::mem::size_of::<Header>());
    std::ptr::read(bytes.as_ptr() as *const Header)
}
```

## Prior Art Synthesis

| Language | Strategy | Lesson for Ruchy |
|----------|----------|------------------|
| **ReScript** | Separate bsb tool, JSON config | Keep build in Cargo, not separate tool |
| **Nim** | Compiles to C, uses system compiler | Direct Rust generation avoids C complexities |
| **F# Fable** | JavaScript focus, source maps | Source map generation critical for debugging |
| **PureScript** | Multiple backends via plugins | Single Rust target reduces complexity |
| **Kotlin/Native** | LLVM-based, memory management | Rust's ownership superior to reference counting |
| **Crystal** | Type inference, macro expansion | Bidirectional checking more predictable |

**Key Differentiator**: Ruchy uniquely combines refinement types with SMT verification at compile time, achieving safety guarantees beyond any listed language.

## Performance Model

```toml
# performance.toml - Enforced by CI
[bounds]
# Transpilation performance
transpile_time_ratio = 1.5      # vs rustc parsing
incremental_speedup = 10.0       # for single file change

# Runtime performance (vs equivalent Rust)
sequential_overhead = 1.05      # 5% max
actor_overhead = 1.10           # 10% max for message passing
refinement_check = "compile_time" # Zero runtime cost when proven

# Build artifacts
binary_size_ratio = 1.05        # 5% larger max
memory_usage_mb = 500           # For 100k LOC

[verification]
mutation_coverage = 0.8          # 80% minimum
property_tests = true
smt_timeout_ms = 5000
max_cyclomatic_complexity = 15
```

## Migration Path

### Existing Rust → Hybrid Ruchy

```bash
# Gradual migration with verification
ruchy init --mode=hybrid
ruchy migrate src/module.rs --verify

# Validates semantic equivalence
[INFO] Parsing src/module.rs...
[INFO] Generating src/module.ruchy...
[INFO] Verifying equivalence with 1000 test cases...
[INFO] ✓ Migration successful, 100% behavior preserved
```

### Rust Interop

```rust
// Ruchy calling Rust
extern crate my_rust_lib;
use my_rust_lib::process;

fun integrate(data: Vec<u8>) -> Result<Output> {
    data |> process::transform()
        |> process::validate()?
        |> Ok
}

// Rust calling Ruchy (generated bindings)
use ruchy_gen::domain;

fn use_ruchy() {
    let result = domain::calculate(42);
    assert_eq!(result, domain::Result::Ok(84));
}
```

## Delivery Roadmap

### v1.0 Complete Language (Target: 6 months)

All features ship together, no partial implementation:

- **Core**: Parser, type system, transpiler
- **Actors**: Full supervisor trees, backpressure
- **Refinements**: SMT verification, compile-time proofs  
- **Quality**: 80% mutation coverage enforcement
- **Performance**: Meeting all bounds in performance.toml

### Implementation Priority

1. **Month 1-2**: Parser, basic transpilation, module system
2. **Month 3-4**: Type inference, actor runtime, property testing
3. **Month 5**: Refinement types, SMT integration, unsafe verification
4. **Month 6**: Performance optimization, polish, documentation

## Technical Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| SMT solver timeout on complex refinements | Runtime checks required | Incremental solving, constraint caching |
| Actor overhead exceeds 10% | Performance goals unmet | Custom allocator, message batching |
| Mutation testing too slow | CI bottleneck | Parallel mutation, incremental testing |
| Source maps break on macro expansion | Poor debugging | Preserve macro expansion history |

## Summary

Ruchy delivers a complete systems scripting language via pragmatic transpilation to Rust. Key achievements:

1. **2-5% overhead** for safety features, **5-10%** for actors - honest, achievable targets
2. **Compile-time SMT verification** for refinement types - zero runtime cost when proven
3. **80% mutation coverage** enforced - high but realistic quality bar
4. **Direct crates.io usage** - no wrapper generation needed
5. **Semantic equivalence testing** via observational behavior, not AST comparison

The architecture prioritizes correctness over performance, debuggability over cleverness, and Rust idioms over novel constructs. This is systems programming with scripting ergonomics, not scripting with systems performance.
