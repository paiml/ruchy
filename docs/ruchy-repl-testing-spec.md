# REPL Testing Specification

## Executive Summary

The REPL constitutes Ruchy's primary interface. Its reliability directly impacts user trust. This specification mandates comprehensive testing through eleven complementary strategies, each targeting specific failure modes.

## Testing Architecture

### Layer 1: Correctness Verification

#### 1.1 State Machine Property Testing

The REPL operates as a deterministic state machine. All state transitions must preserve invariants.

```rust
#[derive(Debug, Clone, Arbitrary)]
enum ReplState {
    Fresh,
    Loaded { bindings: HashMap<String, Value> },
    Error { recoverable: bool },
    TypeChecking { partial: TypeEnv },
}

#[proptest]
fn prop_state_invariants(
    initial: ReplState,
    commands: Vec<ReplCommand>
) {
    let mut repl = Repl::with_state(initial);
    
    for cmd in commands {
        let prev_state = repl.state.clone();
        let result = repl.execute(cmd);
        
        // Invariant 1: Errors from Fresh state are recoverable
        assert_implies!(
            matches!(prev_state, Fresh) && result.is_err(),
            matches!(repl.state, Error { recoverable: true })
        );
        
        // Invariant 2: Type environment monotonically grows
        assert_implies!(
            matches!(prev_state, TypeChecking { .. }),
            repl.type_env().vars_count() >= prev_state.type_env().vars_count()
        );
        
        // Invariant 3: Compilation cache is transactional
        assert_implies!(
            result.is_err(),
            repl.compilation_cache == prev_state.compilation_cache
        );
    }
}
```

#### 1.2 Differential Testing

Compare REPL behavior against a reference implementation to detect semantic drift.

```rust
struct ReferenceRepl {
    // Minimal, obviously correct implementation
    bindings: BTreeMap<String, String>,
}

#[proptest]
fn prop_semantic_equivalence(program: Vec<Statement>) {
    let mut production = Repl::new();
    let mut reference = ReferenceRepl::new();
    
    for stmt in program {
        let prod_result = production.execute(&stmt);
        let ref_result = reference.execute(&stmt);
        
        // Both must agree on success/failure
        assert_eq!(
            prod_result.is_ok(), 
            ref_result.is_ok(),
            "Semantic divergence on: {:?}", stmt
        );
        
        // Observable behavior must match
        if let (Ok(p), Ok(r)) = (prod_result, ref_result) {
            assert_semantic_equiv(p, r);
        }
    }
}
```

### Layer 2: Robustness Testing

#### 2.1 Incremental Compilation Fuzzing

Test incremental compilation with pathological input sequences.

```rust
#[fuzz]
fn fuzz_incremental_cache(data: &[u8]) {
    let mut repl = Repl::new();
    let fuzzer = GrammarFuzzer::new(data);
    
    while let Some(fragment) = fuzzer.next_fragment() {
        let cache_snapshot = repl.compilation_cache.snapshot();
        let result = repl.eval_incremental(fragment);
        
        // Cache consistency invariant
        if result.is_err() {
            assert_eq!(repl.compilation_cache, cache_snapshot);
        }
        
        // Memory bound invariant (100MB)
        assert!(repl.resident_memory() < 104_857_600);
        
        // Cache size bound (1000 entries)
        assert!(repl.compilation_cache.len() < 1000);
    }
}
```

#### 2.2 Grammar Coverage Fuzzing

Ensure all language constructs are reachable and testable.

```rust
impl GrammarFuzzer {
    fn generate(depth: usize, rng: &mut impl Rng) -> Expr {
        if depth == 0 {
            return Expr::Literal(rng.gen_range(0..100));
        }
        
        match rng.gen_range(0..20) {
            0..=3 => Expr::Binary {
                op: rng.gen(),
                left: Box::new(Self::generate(depth - 1, rng)),
                right: Box::new(Self::generate(depth - 1, rng)),
            },
            4..=6 => Expr::Lambda {
                params: (0..rng.gen_range(1..4))
                    .map(|i| format!("x{}", i))
                    .collect(),
                body: Box::new(Self::generate(depth - 1, rng)),
            },
            7..=9 => Expr::Pipeline {
                expr: Box::new(Self::generate(depth - 1, rng)),
                stages: (0..rng.gen_range(1..5))
                    .map(|_| Self::generate(depth / 2, rng))
                    .collect(),
            },
            // ... all other constructs
        }
    }
}

#[test]
fn test_complete_grammar_coverage() {
    let mut coverage = BitSet::new(Expr::VARIANT_COUNT);
    let mut repl = Repl::new();
    
    for seed in 0..10_000 {
        let mut rng = StdRng::seed_from_u64(seed);
        let expr = GrammarFuzzer::generate(5, &mut rng);
        
        if repl.eval(&expr.to_string()).is_ok() {
            coverage.insert(expr.variant_id());
        }
    }
    
    assert_eq!(coverage.count(), Expr::VARIANT_COUNT);
}
```

### Layer 3: Performance Guarantees

#### 3.1 Latency Regression Prevention

```rust
#[bench]
fn bench_repl_startup(b: &mut Bencher) {
    b.iter(|| Repl::new());
    assert!(b.ns_per_iter() < 10_000_000); // <10ms
}

#[bench]
fn bench_incremental_eval(b: &mut Bencher) {
    let mut repl = Repl::new();
    repl.eval("let base = 42").unwrap();
    
    b.iter(|| repl.eval("base + 1"));
    assert!(b.ns_per_iter() < 1_000_000); // <1ms
}

#[bench]
fn bench_type_lookup(b: &mut Bencher) {
    let mut repl = Repl::new();
    for i in 0..1000 {
        repl.eval(&format!("let v{} = {}", i, i)).unwrap();
    }
    
    b.iter(|| repl.eval(":type v500"));
    assert!(b.ns_per_iter() < 10_000); // <10μs (O(1))
}
```

### Layer 4: Error Recovery

#### 4.1 Mutation Testing for Recovery Paths

```rust
#[mutate]
fn recover_from_parse_error(tokens: TokenStream) -> Result<Ast> {
    match parse(tokens) {
        Ok(ast) => Ok(ast),
        Err(ParseError::UnexpectedEof) => {
            // Mutation target: synthetic token insertion
            let recovered = tokens.with_synthetic_end();
            parse(recovered)
        }
        Err(ParseError::UnmatchedDelimiter { open, .. }) => {
            // Mutation target: delimiter matching
            let recovered = tokens.insert_matching(open);
            parse(recovered)
        }
        Err(e) => Err(e)
    }
}

#[test]
fn test_recovery_kills_mutants() {
    let cases = vec![
        ("let x = [1, 2", Ok(Ast::Let { .. })),
        ("fun f() { x", Ok(Ast::Function { .. })),
        ("match x { Some(y", Ok(Ast::Match { .. })),
    ];
    
    for (input, expected) in cases {
        let tokens = tokenize(input).unwrap();
        let result = recover_from_parse_error(tokens);
        assert!(matches!(result, expected));
    }
}
```

### Layer 5: Resource Management

#### 5.1 Chaos Engineering

```rust
#[test]
fn test_memory_pressure_recovery() {
    let mut repl = Repl::with_limits(
        MemoryLimit::Mb(50),
        TimeLimit::Millis(100),
    );
    
    // Attempt allocation beyond limit
    match repl.eval("let huge = vec![0; 100_000_000]") {
        Err(ReplError::MemoryExhausted { used, limit }) => {
            assert!(used <= limit);
            // Must recover to functional state
            assert_eq!(repl.eval("2 + 2"), Ok(Value::Int(4)));
        }
        _ => panic!("Should enforce memory limit")
    }
}

#[test]
fn test_infinite_loop_timeout() {
    let mut repl = Repl::with_timeout(Duration::from_millis(100));
    
    match repl.eval("loop {}") {
        Err(ReplError::Timeout { elapsed }) => {
            assert!(elapsed >= Duration::from_millis(100));
            assert!(elapsed < Duration::from_millis(150));
            // Must remain responsive
            assert_eq!(repl.eval("true"), Ok(Value::Bool(true)));
        }
        _ => panic!("Should timeout infinite loops")
    }
}
```

### Layer 6: Session Isolation

#### 6.1 Concurrent Session Safety

```rust
#[test]
fn test_parallel_session_isolation() {
    let shared_cache = Arc::new(TypeCache::new());
    
    let handles: Vec<_> = (0..100).map(|id| {
        let cache = Arc::clone(&shared_cache);
        thread::spawn(move || {
            let mut repl = Repl::with_cache(cache);
            
            // Each session has unique bindings
            repl.eval(&format!("let x = {}", id)).unwrap();
            
            // Verify isolation
            match repl.eval("x").unwrap() {
                Value::Int(n) => assert_eq!(n, id),
                _ => panic!("Session isolation violated")
            }
        })
    }).collect();
    
    for h in handles {
        h.join().unwrap();
    }
    
    // Shared cache remains consistent
    assert!(shared_cache.verify_consistency());
}
```

### Layer 7: Integration Testing

#### 7.1 Doctest-Driven Sessions

Every public API must include a complete REPL session as documentation:

```rust
/// Type inference through pattern matching
/// 
/// ```
/// # use ruchy_repl::Repl;
/// let mut repl = Repl::new();
/// 
/// // Pattern matching infers types
/// repl.eval(r#"
///     match Some(42) {
///         Some(x) => x * 2,
///         None => 0
///     }
/// "#).unwrap();
/// 
/// // Function types flow through
/// repl.eval("let apply = |f, x| f(x)").unwrap();
/// assert_eq!(
///     repl.type_of("apply"), 
///     "forall a b. (a -> b, a) -> b"
/// );
/// ```
pub fn type_inference() { /* ... */ }
```

#### 7.2 Example Workflows

```rust
// examples/data_science_workflow.rs
fn main() -> Result<()> {
    let mut repl = Repl::new();
    
    // DataFrame operations
    repl.eval(r#"
        let df = read_csv("data.csv")
        |> filter(col("age") > 18)
        |> groupby("category")
        |> agg([mean("value"), std("value")])
    "#)?;
    
    // Verify Rust generation
    let rust = repl.to_rust("df")?;
    assert!(rust.contains("DataFrame::from"));
    assert!(rust.contains(".lazy()"));
    assert!(rust.contains(".collect()"));
    
    Ok(())
}
```

### Layer 8: Golden Master Testing

Track output stability across releases:

```rust
#[test]
fn test_golden_outputs() {
    let test_dir = Path::new("tests/golden");
    
    for entry in fs::read_dir(test_dir)? {
        let path = entry?.path();
        if path.extension() == Some("ruchy") {
            let input = fs::read_to_string(&path)?;
            let golden_path = path.with_extension("golden");
            let golden = fs::read_to_string(&golden_path)?;
            
            let mut repl = Repl::new();
            let mut output = String::new();
            
            for line in input.lines() {
                output.push_str(&format!("> {}\n", line));
                match repl.eval(line) {
                    Ok(val) => output.push_str(&format!("{}\n", val)),
                    Err(e) => output.push_str(&format!("Error: {}\n", e)),
                }
            }
            
            assert_eq!(output, golden, "Golden test failed: {:?}", path);
        }
    }
}
```

## Implementation Plan

### Phase 1: Foundation (Week 1)
- Property tests for state machine
- Basic fuzzing harness
- Performance benchmarks

### Phase 2: Robustness (Week 2)
- Grammar coverage fuzzing
- Mutation testing setup
- Chaos engineering tests

### Phase 3: Integration (Week 3)
- Doctest coverage
- Example workflows
- Golden master suite

### Phase 4: CI Integration (Week 4)
- Automated benchmark regression detection
- Continuous fuzzing infrastructure
- Coverage reporting

## Acceptance Criteria

1. **Coverage**: Line coverage ≥ 95%, branch coverage ≥ 90%
2. **Performance**: All benchmarks pass latency requirements
3. **Fuzzing**: 24 hours without crashes or leaks
4. **Properties**: 100,000 iterations without failures
5. **Mutations**: ≥ 95% mutants killed

## Responding to Review Feedback

### Golden Master Testing (Addition #11)
Accepted. Golden master testing provides regression detection for complex output formatting. Implementation added in Layer 8.

### Cross-Platform Testing
Accepted. CI matrix must include:
- Linux (x86_64, aarch64)
- macOS (x86_64, ARM)
- Windows (x86_64)

Platform-specific tests added for path handling and terminal control.

### Snapshot Testing
Accepted with modification. We'll use `insta` for complex multi-line outputs (DataFrames, error messages) while maintaining string equality for simple values. This balances precision with maintainability.

```rust
#[test]
fn test_dataframe_display() {
    let mut repl = Repl::new();
    repl.eval("let df = read_csv('test.csv')").unwrap();
    
    let output = repl.eval("df.head()").unwrap();
    insta::assert_snapshot!(output.to_string());
}
```

## Conclusion

This specification ensures REPL reliability through defense in depth. Each testing layer targets specific failure modes. Together, they guarantee a REPL that never disappoints.

The REPL is Ruchy's face. These tests ensure it maintains composure under all conditions.

Yes, this bug exposes three distinct transpilation errors that will be fixed:

## Bug Analysis

### 1. **Incorrect Type Placeholder Generation**
The transpiler emits `_` for untyped parameters, but Rust forbids placeholders in function signatures.

**Fix**: Generate generic type parameters:
```rust
// Current (broken)
fn greet(name: _) { ... }

// Fixed
fn greet<T>(name: T) { ... }
```

### 2. **Double-Braced Block Generation**
The transpiler wraps function bodies in unnecessary braces, triggering Rust's `unused_braces` lint.

**Fix**: Detect implicit returns and avoid double-wrapping:
```rust
// Current (broken)
fn greet(name: T) { { "Hello, {name}!" } }

// Fixed
fn greet(name: T) -> &'static str { "Hello, {name}!" }
```

### 3. **Missing Return Type Inference**
The transpiler fails to infer return types for functions with implicit returns.

**Fix**: Propagate expression types through the function signature:
```rust
// Current (broken)
fn greet(name: T) { "Hello" }  // Error: expected ()

// Fixed
fn greet(name: T) -> String { format!("Hello, {name}!") }
```

## Implementation Fix

In `src/backend/rust_gen.rs`:

```rust
fn transpile_function(&self, func: &Function) -> String {
    let params = func.params.iter()
        .enumerate()
        .map(|(i, p)| match &p.ty {
            Some(ty) => format!("{}: {}", p.name, self.transpile_type(ty)),
            None => {
                // Generate fresh type variable instead of placeholder
                let ty_var = format!("T{}", i);
                self.type_params.insert(ty_var.clone());
                format!("{}: {}", p.name, ty_var)
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    
    let generics = if !self.type_params.is_empty() {
        format!("<{}>", self.type_params.iter().join(", "))
    } else {
        String::new()
    };
    
    // Infer return type from body
    let return_type = match &func.return_type {
        Some(ty) => format!(" -> {}", self.transpile_type(ty)),
        None if func.body.is_expression() => {
            let inferred = self.infer_type(&func.body);
            format!(" -> {}", self.transpile_type(&inferred))
        }
        None => String::new(),
    };
    
    // Don't double-wrap expression bodies
    let body = if func.body.is_expression() && return_type.is_empty() {
        self.transpile_expr(&func.body)
    } else {
        format!("{{{}}}", self.transpile_expr(&func.body))
    };
    
    format!("fn {}{}{}{} {}", 
        func.name, generics, params, return_type, body)
}
```

## Test Coverage

This exact bug becomes a regression test:

```rust
#[test]
fn test_untyped_function_with_string_interpolation() {
    let input = r#"fun greet(name) { "Hello, {name}!" }"#;
    let rust = transpile(input).unwrap();
    
    // Must generate valid Rust
    assert!(rust.contains("fn greet<T>"));
    assert!(rust.contains("-> String"));
    assert!(rust.contains(r#"format!("Hello, {}!", name)"#));
    
    // Must compile
    assert!(compile_rust(&rust).is_ok());
}
```

The REPL testing specification ensures this class of error never recurs through:
1. **Golden master tests** - capturing expected transpilation output
2. **Property testing** - all valid Ruchy must produce compilable Rust
3. **Fuzzing** - finding edge cases in type inference

This bug is a transpilation error, not a REPL error. The REPL correctly reported the Rust compilation failure. The fix belongs in the transpiler's type inference and code generation phases.