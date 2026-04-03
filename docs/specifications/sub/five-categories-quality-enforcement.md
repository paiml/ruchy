# Sub-spec: Five Categories Coverage — Quality Enforcement and Tooling

**Parent:** [five-categories-coverage-spec.md](../five-categories-coverage-spec.md) Sections 5-9

---


## Quality Enforcement Framework

### Five-Category Coverage Strategy

Systematic decomposition into orthogonal test domains ensures comprehensive validation without redundancy.

#### Category Boundaries

| Category | Modules | Coverage Target | Complexity Limit | Current |
|----------|---------|-----------------|------------------|---------|
| Frontend | lexer, parser, ast, diagnostics | 80% | CC ≤ 10 | 45% |
| Backend | transpiler, compiler, modules | 80% | CC ≤ 10 | 50% |
| Runtime | interpreter, repl, actors | 80% | CC ≤ 10 | 40% |
| WASM | component, deployment, notebook | 80% | CC ≤ 10 | 15% |
| Quality | generators, harness, scoring | 80% | CC ≤ 10 | 60% |

### Test Development Protocol

#### Pre-Implementation Gate
```bash
pmat tdg src/$MODULE --min-grade A+ || exit 1
cargo clippy -- -D warnings || exit 1
```

#### TDD Cycle
1. **Red**: Write failing test capturing specification
2. **Green**: Minimal implementation to pass
3. **Refactor**: Extract abstractions, reduce complexity
4. **Measure**: `cargo llvm-cov --lib --include-path "src/$MODULE"`
5. **Gate**: Complexity ≤10, coverage ≥80%, TDG A+

#### Property Test Requirements

Every module must include property tests for invariants:

```rust
#[proptest]
fn parser_ast_roundtrip(source: ValidRuchySource) {
    let ast = parse(&source).unwrap();
    let rendered = ast.to_source();
    let reparsed = parse(&rendered).unwrap();
    prop_assert_eq!(ast, reparsed);
}

#[proptest]
fn type_preservation(expr: TypedExpr) {
    let rust_ast = transpile(&expr);
    let original_type = infer(&expr);
    let rust_type = extract_type(&rust_ast);
    prop_assert!(types_equivalent(original_type, rust_type));
}
```

### Mutation Testing Integration

```toml
[dev-dependencies]
mutagen = "0.3"

[profile.mutants]
inherits = "test"
opt-level = 1  # Faster mutation builds
```

Mutation score targets:
- Parser: >70% mutant kill rate
- Type checker: >80% mutant kill rate
- Transpiler: >75% mutant kill rate

## MCP Integration Specification

### Compiler-Level Protocol Support

```rust
#[mcp::tool]
fn analyze_code(source: String) -> Analysis {
    // Automatic protocol generation from signature
}

// Generates:
// 1. JSON Schema for tool description
// 2. Runtime validation
// 3. Async message handling
// 4. Context management
```

### Native Primitives

```ruchy
// First-class MCP support
async fn handle_request(ctx: MCP.Context) -> Response {
    match ctx.tool {
        "code_analysis" => analyze(ctx.params),
        "completion" => complete(ctx.params),
        _ => MCP.error("Unknown tool")
    }
}
```

## Memory Management Strategy

### Escape Analysis

Static analysis determines allocation strategy:

1. **Stack**: Non-escaping, known size
2. **Arena**: Non-escaping, unknown size, batch deallocation
3. **Rc**: Escaping, acyclic
4. **Arc**: Escaping, concurrent access
5. **Box**: Escaping, unique ownership

### Automatic Memory Optimization

```rust
// Ruchy source
fn process(data: Vec<Item>) -> Summary {
    let temp = data.map(transform);  // Arena allocated
    let result = temp.fold(combine); // Stack allocated
    result                            // Moved, not copied
}

// Generated Rust (optimized)
fn process(data: Vec<Item>) -> Summary {
    let arena = Arena::new();
    let temp = arena.alloc_slice(
        data.iter().map(transform)
    );
    temp.fold(Summary::default(), combine)
}
```

## Incremental Compilation Architecture

### Dependency Tracking

```rust
struct CompilationUnit {
    ast_hash: u64,        // Structural hash
    type_hash: u64,       // Type signature hash
    impl_hash: u64,       // Implementation hash
    dependencies: Set<ModuleId>,
    reverse_deps: Set<ModuleId>,
}
```

### Incremental Strategy

1. **Parse**: Cache AST by file hash
2. **Type Check**: Cache by AST hash + dependency types
3. **Transpile**: Cache by type-checked AST hash
4. **Link**: Incremental via rustc

## Tooling Specifications

### Language Server Protocol

Full LSP 3.17 compliance with extensions:

- **Custom**: Property test generation
- **Custom**: Refinement type suggestions
- **Custom**: Transpilation preview
- **Standard**: Completion, hover, goto, rename, format

### REPL Enhancement

```rust
struct ReplState {
    bindings: HashMap<String, Value>,
    types: TypeEnv,
    jit_cache: CraneliftModule,
    history: Vec<Statement>,
}

impl Repl {
    fn eval(&mut self, input: &str) -> Result<Value> {
        let ast = parse(input)?;
        let typed = self.types.infer(&ast)?;
        
        if self.should_jit(&typed) {
            self.jit_cache.compile(&typed)
        } else {
            self.interpret(&typed)
        }
    }
}
```

## Benchmarking Suite

### Microbenchmarks

| Benchmark | Target | Measurement |
|-----------|--------|-------------|
| Lexer throughput | >10MB/s | `criterion::black_box` |
| Parser throughput | >5MB/s | AST nodes/sec |
| Type inference | <1ms/function | Constraint generation |
| Transpilation | <2ms/function | Rust AST generation |
| JIT compilation | <10ms/function | Machine code generation |

### Macrobenchmarks

Comparison against Python 3.11, Ruby 3.2, Rust 1.75:

| Task | Ruchy Target | Python Baseline | Rust Baseline |
|------|--------------|-----------------|---------------|
| HTTP server | 50K req/s | 5K req/s | 100K req/s |
| JSON parsing | 500MB/s | 50MB/s | 1GB/s |
| Fibonacci(40) | <1s | 30s | <0.5s |
| Regex matching | 200MB/s | 20MB/s | 400MB/s |

## Error Message Quality Standards

### Diagnostic Requirements

Every error must include:
1. **Span**: Exact location with line/column
2. **Message**: Clear problem statement
3. **Suggestion**: Actionable fix
4. **Note**: Additional context
5. **Help**: Link to documentation

```rust
error[E0423]: cannot find value `x` in this scope
  --> src/main.ruchy:5:10
   |
 5 |     print(x + 1)
   |          ^ not found in this scope
   |
   = help: did you mean `y`?
   = note: `y` is defined at line 3
   = suggestion: replace `x` with `y`
```

## Sprint Implementation Schedule

### Phase 1: Foundation (Q1 2025)
- **Week 1-2**: Parser completion (target: 80% coverage)
- **Week 3-4**: Type inference (basic HM)
- **Week 5-6**: Minimal transpiler (expressions only)
- **Week 7-8**: REPL prototype
- **Week 9-12**: Property test infrastructure

### Phase 2: Core Features (Q2 2025)
- **Month 1**: Full type system (refinements, gradual)
- **Month 2**: Pattern matching, ADTs
- **Month 3**: Module system, Cargo integration

### Phase 3: Performance (Q3 2025)
- **Month 1**: JIT compilation via Cranelift
- **Month 2**: Actor system implementation
- **Month 3**: Optimization passes

### Phase 4: Production (Q4 2025)
- **Month 1**: MCP integration
- **Month 2**: Tooling (LSP, formatter, linter)
- **Month 3**: Documentation, 1.0 release

## Makefile Orchestration

```makefile
.PHONY: all test coverage quality release

# Development workflow
dev: quality-gate test-watch

quality-gate:
	@pmat tdg src --min-grade A+ --fail-on-violation
	@cargo clippy --all-features -- -D warnings
	@cargo fmt --check

test-watch:
	cargo watch -x "nextest run" -i "target/*"

# Coverage enforcement
coverage-enforce:
	@for cat in frontend backend runtime wasm quality; do \
		echo "Checking $$cat coverage..."; \
		cargo llvm-cov --lib --json \
			--include-path "src/$$cat/**/*.rs" | \
		jq -e '.coverage.lines.percentage >= 80' || \
		(echo "❌ $$cat below 80% coverage" && exit 1); \
	done

# Property testing
proptest:
	cargo test --features proptest -- --nocapture

mutants:
	cargo mutants --timeout 30 --jobs 4

# Benchmarking
bench:
	cargo criterion --message-format json | \
		jq '.median.point_estimate'

# Release build
release: quality-gate coverage-enforce
	cargo build --release --features "jit mcp"
	strip target/release/ruchy
	ls -lh target/release/ruchy
```

## Continuous Integration Matrix

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, beta, nightly]
    category: [frontend, backend, runtime, wasm, quality]

steps:
  - uses: actions-rust-lang/setup-rust-toolchain@v1
    with:
      toolchain: ${{ matrix.rust }}
      components: rustfmt, clippy, llvm-tools-preview

  - name: Quality Gate
    run: make gate-${{ matrix.category }}

  - name: Coverage Check
    run: |
      make coverage-${{ matrix.category }}
      coverage=$(cargo llvm-cov --json | jq '.coverage.lines.percentage')
      if (( $(echo "$coverage < 80" | bc -l) )); then
        exit 1
      fi

  - name: Mutation Testing
    if: matrix.category != 'wasm'
    run: cargo mutants --package ruchy-${{ matrix.category }}
```

## Technical Decisions Log

### ADR-001: Transpile to Rust vs LLVM
**Decision**: Generate Rust source code
**Rationale**:
- Inherit borrow checker validation
- Zero-effort ecosystem integration
- Debuggable intermediate representation
- Leverage rustc's optimization pipeline

### ADR-002: Gradual Typing Strategy
**Decision**: Explicit dynamic boundaries with `?` syntax
**Rationale**:
- Clear performance model (dynamic = slow)
- Static analysis can eliminate most checks
- Compatible with refinement types
- Preserves type inference quality

### ADR-003: Actor Model for Concurrency
**Decision**: Erlang-style actors with supervision trees
**Rationale**:
- Natural fit for scripting use cases
- Fault isolation by default
- Composable concurrency patterns
- Maps well to Rust's ownership model

### ADR-004: Property Testing Mandatory
**Decision**: Every module requires property tests
**Rationale**:
- Catches edge cases humans miss
- Documents invariants formally
- Enables confident refactoring
- Supports specification mining

## Appendix: Grammar Quick Reference

```ebnf
program     = declaration*
declaration = fn_decl | type_decl | const_decl | import
fn_decl     = "fn" IDENT "(" params ")" type? "=" expr
type_decl   = "type" IDENT type_params? "=" type
const_decl  = "const" pattern ":" type "=" expr

expr        = lambda | match | if | let | binary
lambda      = "\" pattern+ "->" expr
match       = "match" expr "{" arm+ "}"
arm         = pattern guard? "=>" expr
guard       = "if" expr

type        = type_atom ("->" type)?
type_atom   = IDENT | "{" fields "}" | "(" type ")"
refinement  = type "&" "{" predicate "}"
```

## Contact & Contribution

- **Architecture**: Document changes as ADRs in `docs/architecture/`
- **Performance**: Include benchmarks proving no regression
- **API Changes**: Provide migration guide with semver impact
