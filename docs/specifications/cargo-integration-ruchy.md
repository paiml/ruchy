# Ruchy Cargo Integration Architecture

## Executive Summary

Ruchy transpiles to idiomatic Rust through `build.rs`, implementing all language features in v1.0. Runtime overhead: 2-5% for safety features, 5-10% for actor systems. Compile-time verification via SMT solvers ensures correctness.

## Core Design Constraints

1. **Zero Cargo Modification**: Stock `cargo` commands work unchanged
2. **Performance Bounds**: Runtime ≤5% overhead (sequential), ≤10% (concurrent)
3. **Verification Depth**: SMT-provable refinements, 80% mutation coverage
4. **Ecosystem Compatibility**: Any crates.io package directly usable

## Architecture

### Build Pipeline

```toml
# Cargo.toml
[package]
name = "ruchy-project"
version = "0.1.0"

[build-dependencies]
ruchy = "1.0"
z3 = "0.12"  # SMT solver for refinement types

[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }

[package.metadata.ruchy]
actor_runtime = "tokio"        # Actor backend
verify_refinements = true       # Compile-time SMT verification
mutation_coverage = 0.8         # Required kill rate
max_complexity = 15             # Cyclomatic complexity limit
```

### Build Script Implementation

```rust
// build.rs
use ruchy::build::{Config, VerificationLevel};

fn main() {
    let config = Config::from_cargo_metadata()
        .verification_level(VerificationLevel::Strict)
        .parallel_jobs(num_cpus::get());
    
    match ruchy::build::transpile(config) {
        Ok(metrics) => {
            println!("cargo:warning=Transpiled {} files in {:?}", 
                     metrics.file_count, metrics.duration);
            println!("cargo:warning=SMT proved {} refinements", 
                     metrics.refinements_proved);
        }
        Err(e) => match e.kind() {
            ErrorKind::RefinementUnprovable(ref_type) => {
                panic!("Cannot prove refinement: {}", ref_type);
            }
            ErrorKind::MutationSurvived(mutation) => {
                panic!("Mutation survived tests: {:?}", mutation);
            }
            _ => panic!("Build failed: {}", e),
        }
    }
}
```

### Module Resolution

```rust
// src/lib.rs - Standard Rust entry point
#[path = "../target/ruchy-gen/mod.rs"]
mod generated;

pub use generated::*;

// target/ruchy-gen/mod.rs (auto-generated)
#[path = "core.rs"]
pub mod core;
#[path = "actors.rs"] 
pub mod actors;
#[path = "domain/mod.rs"]
pub mod domain;
```

## Transpilation Strategy

### Pipeline Architecture

```rust
pub struct Transpiler {
    parser: IncrementalParser,
    type_checker: BidirectionalChecker,
    mir_gen: MirGenerator,
    optimizer: PeepholeOptimizer,
    codegen: RustEmitter,
    verifier: Z3Verifier,
}

impl Transpiler {
    pub fn transpile(&mut self, source: &str) -> Result<String> {
        // Parse with error recovery
        let ast = self.parser.parse_with_recovery(source)?;
        
        // Bidirectional type checking
        let typed_ast = self.type_checker.check(ast)?;
        
        // Verify refinement types
        let proofs = self.verifier.prove_refinements(&typed_ast)?;
        
        // Lower to MIR for optimization
        let mir = self.mir_gen.lower(typed_ast)?;
        
        // Peephole optimizations only
        let opt_mir = self.optimizer.optimize(mir);
        
        // Emit idiomatic Rust
        let rust_ast = self.codegen.emit(opt_mir, proofs)?;
        
        // Format with rustfmt
        Ok(rustfmt::format(rust_ast)?)
    }
}
```

### Actor System Translation

Actors compile to bounded-channel tokio tasks with backpressure:

```rust
// Input: Ruchy actor
actor RateLimiter {
    tokens: u32,
    rate: Duration
    
    receive request(id: RequestId) -> Result<Token> {
        if self.tokens > 0 {
            self.tokens -= 1;
            Ok(Token::new(id))
        } else {
            Err("Rate limit exceeded")
        }
    }
}

// Output: Idiomatic Rust
pub struct RateLimiter {
    state: Arc<Mutex<RateLimiterState>>,
    rx: mpsc::Receiver<Message>,
}

impl RateLimiter {
    pub fn spawn(tokens: u32, rate: Duration) -> Handle {
        let (tx, rx) = mpsc::channel(1000); // Bounded
        
        let actor = Self {
            state: Arc::new(Mutex::new(RateLimiterState { tokens, rate })),
            rx,
        };
        
        tokio::spawn(async move {
            while let Some(msg) = actor.rx.recv().await {
                actor.handle_message(msg).await;
            }
        });
        
        Handle { tx }
    }
    
    async fn handle_message(&self, msg: Message) {
        match msg {
            Message::Request { id, reply } => {
                let mut state = self.state.lock().await;
                let result = if state.tokens > 0 {
                    state.tokens -= 1;
                    Ok(Token::new(id))
                } else {
                    Err("Rate limit exceeded".into())
                };
                let _ = reply.send(result);
            }
        }
    }
}
```

**Performance Impact**: 5-10% overhead vs manual async Rust due to message boxing and dynamic dispatch. Acceptable for actor use cases.

### Refinement Type Implementation

```rust
// Refinement type verification
pub struct RefinementVerifier {
    ctx: z3::Context,
    solver: z3::Solver,
}

impl RefinementVerifier {
    pub fn verify_type(&mut self, ty: &RefinementType) -> Result<Proof> {
        // type Port = u16 where |x| 1 <= x <= 65535
        let var = self.ctx.int_var("x");
        let constraint = self.encode_predicate(&ty.predicate, &var);
        
        // Check satisfiability
        self.solver.push();
        self.solver.assert(&constraint);
        
        let result = match self.solver.check() {
            SatResult::Sat => {
                // Extract witness
                let model = self.solver.get_model();
                Ok(Proof::Valid(model))
            }
            SatResult::Unsat => {
                Err(VerificationError::Unsatisfiable(ty.clone()))
            }
            SatResult::Unknown => {
                // Timeout - fall back to runtime checks
                Ok(Proof::RuntimeCheck)
            }
        };
        
        self.solver.pop();
        result
    }
}

// Code generation for refinement types
fn generate_refinement_type(ty: &RefinementType, proof: &Proof) -> TokenStream {
    match proof {
        Proof::Valid(_) => {
            // Compile-time verified - zero runtime cost
            quote! {
                #[derive(Debug, Clone, Copy)]
                pub struct #name(#base_type);
                
                impl #name {
                    #[inline(always)]
                    pub const unsafe fn new_unchecked(value: #base_type) -> Self {
                        Self(value)
                    }
                    
                    pub fn new(value: #base_type) -> Result<Self, String> {
                        // Static assertion for compile-time known values
                        if #predicate {
                            Ok(Self(value))
                        } else {
                            Err(format!("Value {} violates refinement", value))
                        }
                    }
                }
            }
        }
        Proof::RuntimeCheck => {
            // SMT timeout - runtime validation required
            quote! {
                #[derive(Debug, Clone, Copy)]
                pub struct #name(#base_type);
                
                impl #name {
                    pub fn new(value: #base_type) -> Result<Self, String> {
                        if #predicate {
                            Ok(Self(value))
                        } else {
                            Err(format!("Value {} violates refinement", value))
                        }
                    }
                }
                
                impl TryFrom<#base_type> for #name {
                    type Error = String;
                    fn try_from(value: #base_type) -> Result<Self, Self::Error> {
                        Self::new(value)
                    }
                }
            }
        }
    }
}
```

### Unsafe Code Verification

```rust
// Unsafe blocks require proof annotations
#[unsafe_proof(
    method = "smt",
    solver = "z3", 
    timeout = 5000
)]
unsafe fun unchecked_index<T>(
    arr: &[T], 
    i: usize where |x| x < arr.len()
) -> &T {
    arr.get_unchecked(i)
}

// Verification process
impl UnsafeVerifier {
    fn verify_unsafe_block(&mut self, block: &UnsafeBlock) -> Result<()> {
        let proof_attr = block.get_proof_annotation()
            .ok_or(Error::MissingProof)?;
        
        match proof_attr.method {
            ProofMethod::SMT => {
                // Generate verification condition
                let vc = self.generate_vc(block)?;
                
                // Prove with Z3
                self.solver.assert(&vc);
                match self.solver.check_with_timeout(proof_attr.timeout) {
                    SatResult::Unsat => Ok(()), // Proven safe
                    _ => Err(Error::UnprovenUnsafe(block.span))
                }
            }
            ProofMethod::Manual(ref evidence) => {
                // Human-provided proof
                self.validate_evidence(evidence)
            }
        }
    }
}
```

## Quality Enforcement

### Semantic Equivalence Testing

```rust
// Use observational equivalence, not AST comparison
pub fn verify_semantic_equivalence(
    ruchy_ast: &RuchyAst, 
    rust_ast: &RustAst
) -> Result<()> {
    // Generate 1000 test inputs
    let inputs = proptest::generate_inputs(1000);
    
    for input in inputs {
        let ruchy_result = interpret_ruchy(ruchy_ast, &input)?;
        let rust_result = execute_rust(rust_ast, &input)?;
        
        // Compare observable behavior
        if !observationally_equivalent(&ruchy_result, &rust_result) {
            return Err(Error::SemanticDivergence {
                input,
                ruchy_output: ruchy_result,
                rust_output: rust_result,
            });
        }
    }
    
    Ok(())
}

fn observationally_equivalent(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Error(e1), Value::Error(e2)) => {
            // Errors equivalent if same class
            e1.kind() == e2.kind()
        }
        // Side effects must occur in same order
        (Value::IO(effects1), Value::IO(effects2)) => {
            effects1.len() == effects2.len() &&
            effects1.iter().zip(effects2).all(|(e1, e2)| {
                e1.operation == e2.operation
            })
        }
        _ => false
    }
}
```

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
