# Sub-spec: Cargo Integration — Architecture and Transpilation

**Parent:** [cargo-integration-ruchy.md](../cargo-integration-ruchy.md) Sections 1-4

---

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
