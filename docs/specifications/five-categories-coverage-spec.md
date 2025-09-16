# Ruchy Testing Strategy & Technical Specification

## Executive Summary

This document defines a systematic testing strategy to achieve >80% coverage across the Ruchy compiler through disciplined application of Toyota Way principles. We decompose the challenge into five orthogonal categories, each with enforced quality gates, TDD requirements, and complexity limits.

## Toyota Way Testing Philosophy

### Core Principles
- **Jidoka (自働化)**: Build quality into each component through TDD
- **Andon (アンドン)**: Stop immediately when quality gates fail
- **Kaizen (改善)**: Continuous improvement through incremental coverage gains
- **Genchi Genbutsu (現地現物)**: Measure actual coverage, not estimates

### Quality Gate Requirements (Non-Negotiable)
```
✓ TDD: Test written BEFORE implementation
✓ Complexity: Cyclomatic complexity ≤10 per function  
✓ PMAT Score: TDG grade ≥A+ (95 points)
✓ Coverage: ≥80% per category
✓ Zero Tolerance: No clippy warnings, no broken tests
```

## Five-Category Coverage Strategy

### Category Distribution & Current State

| Category | Target Coverage | Current | Gap | Sprint Priority |
|----------|----------------|---------|-----|-----------------|
| Frontend | 80% | 45% | 35% | 2 |
| Backend | 80% | 50% | 30% | 3 |
| Runtime | 80% | 40% | 40% | 4 |
| WASM | 80% | 15% | 65% | 5 |
| Quality | 80% | 60% | 20% | 1 |

### Sprint 1: Quality Infrastructure (Week 1)
**Modules**: `testing/`, `quality/`, `utils/`
```bash
make gate-quality       # Pre-sprint validation
make coverage-quality   # Baseline measurement
# TDD cycle for 20% gap
make coverage-quality   # Verify 80% achieved
```

### Sprint 2: Frontend (Week 2)
**Modules**: `lexer.rs`, `parser/`, `ast.rs`, `diagnostics.rs`
```bash
make gate-frontend
make coverage-frontend  # Current: 45%
# TDD cycle for 35% gap
make coverage-frontend  # Target: 80%
```

### Sprint 3: Backend (Week 3)
**Modules**: `transpiler/`, `compiler.rs`, `module_*.rs`
```bash
make gate-backend
make coverage-backend   # Current: 50%
# TDD cycle for 30% gap
make coverage-backend   # Target: 80%
```

### Sprint 4: Runtime (Week 4)
**Modules**: `interpreter.rs`, `repl.rs`, `actor.rs`
```bash
make gate-runtime
make coverage-runtime   # Current: 40%
# TDD cycle for 40% gap
make coverage-runtime   # Target: 80%
```

### Sprint 5: WASM (Weeks 5-6)
**Modules**: `component.rs`, `deployment.rs`, `notebook.rs`
```bash
make gate-wasm
make coverage-wasm      # Current: 15%
# TDD cycle for 65% gap
make coverage-wasm      # Target: 80%
```

## Makefile Implementation

```makefile
# ============================================================================
# COVERAGE TARGETS - One command per category
# ============================================================================

coverage-frontend:
	@echo "🎯 Frontend Coverage Analysis"
	@cargo llvm-cov --lib --html \
		--include-path "src/frontend/**/*.rs" \
		--output-path target/coverage/frontend
	@coverage=$$(cargo llvm-cov --lib --json \
		--include-path "src/frontend/**/*.rs" | \
		jq '.files[].summary.lines.percent' | \
		awk '{sum+=$$1; count++} END {print sum/count}'); \
	echo "📊 Frontend Coverage: $${coverage}%"; \
	if (( $$(echo "$$coverage < 80" | bc -l) )); then \
		echo "❌ Below 80% target"; exit 1; \
	else \
		echo "✅ Meets 80% target"; \
	fi

coverage-backend:
	@echo "🎯 Backend Coverage Analysis"
	@cargo llvm-cov --lib --html \
		--include-path "src/backend/**/*.rs" \
		--output-path target/coverage/backend
	@coverage=$$(cargo llvm-cov --lib --json \
		--include-path "src/backend/**/*.rs" | \
		jq '.files[].summary.lines.percent' | \
		awk '{sum+=$$1; count++} END {print sum/count}'); \
	echo "📊 Backend Coverage: $${coverage}%"; \
	if (( $$(echo "$$coverage < 80" | bc -l) )); then \
		echo "❌ Below 80% target"; exit 1; \
	else \
		echo "✅ Meets 80% target"; \
	fi

coverage-runtime:
	@echo "🎯 Runtime Coverage Analysis"
	@cargo llvm-cov --lib --html \
		--include-path "src/runtime/**/*.rs" \
		--output-path target/coverage/runtime
	@coverage=$$(cargo llvm-cov --lib --json \
		--include-path "src/runtime/**/*.rs" | \
		jq '.files[].summary.lines.percent' | \
		awk '{sum+=$$1; count++} END {print sum/count}'); \
	echo "📊 Runtime Coverage: $${coverage}%"; \
	if (( $$(echo "$$coverage < 80" | bc -l) )); then \
		echo "❌ Below 80% target"; exit 1; \
	else \
		echo "✅ Meets 80% target"; \
	fi

coverage-wasm:
	@echo "🎯 WASM Coverage Analysis"
	@cargo llvm-cov --lib --html \
		--include-path "src/wasm/**/*.rs" \
		--output-path target/coverage/wasm
	@coverage=$$(cargo llvm-cov --lib --json \
		--include-path "src/wasm/**/*.rs" | \
		jq '.files[].summary.lines.percent' | \
		awk '{sum+=$$1; count++} END {print sum/count}'); \
	echo "📊 WASM Coverage: $${coverage}%"; \
	if (( $$(echo "$$coverage < 80" | bc -l) )); then \
		echo "❌ Below 80% target"; exit 1; \
	else \
		echo "✅ Meets 80% target"; \
	fi

coverage-quality:
	@echo "🎯 Quality Infrastructure Coverage Analysis"
	@cargo llvm-cov --lib --html \
		--include-path "src/testing/**/*.rs" \
		--include-path "src/quality/**/*.rs" \
		--include-path "src/utils/**/*.rs" \
		--output-path target/coverage/quality
	@coverage=$$(cargo llvm-cov --lib --json \
		--include-path "src/testing/**/*.rs" \
		--include-path "src/quality/**/*.rs" \
		--include-path "src/utils/**/*.rs" | \
		jq '.files[].summary.lines.percent' | \
		awk '{sum+=$$1; count++} END {print sum/count}'); \
	echo "📊 Quality Coverage: $${coverage}%"; \
	if (( $$(echo "$$coverage < 80" | bc -l) )); then \
		echo "❌ Below 80% target"; exit 1; \
	else \
		echo "✅ Meets 80% target"; \
	fi

# ============================================================================
# QUALITY GATES - Enforce before any coverage improvement
# ============================================================================

gate-frontend:
	@echo "🔒 Frontend Quality Gate Check"
	@pmat tdg src/frontend --min-grade A+ --fail-on-violation || \
		(echo "❌ TDG score below A+"; exit 1)
	@cargo clippy --all-features -- -D warnings || \
		(echo "❌ Clippy warnings found"; exit 1)
	@pmat analyze complexity src/frontend --max-cyclomatic 10 || \
		(echo "❌ Functions exceed complexity 10"; exit 1)
	@cargo test frontend:: --no-fail-fast || \
		(echo "❌ Test failures"; exit 1)
	@echo "✅ Frontend passes all quality gates"

gate-backend:
	@echo "🔒 Backend Quality Gate Check"
	@pmat tdg src/backend --min-grade A+ --fail-on-violation || \
		(echo "❌ TDG score below A+"; exit 1)
	@cargo clippy --all-features -- -D warnings || \
		(echo "❌ Clippy warnings found"; exit 1)
	@pmat analyze complexity src/backend --max-cyclomatic 10 || \
		(echo "❌ Functions exceed complexity 10"; exit 1)
	@cargo test backend:: --no-fail-fast || \
		(echo "❌ Test failures"; exit 1)
	@echo "✅ Backend passes all quality gates"

gate-runtime:
	@echo "🔒 Runtime Quality Gate Check"
	@pmat tdg src/runtime --min-grade A+ --fail-on-violation || \
		(echo "❌ TDG score below A+"; exit 1)
	@cargo clippy --all-features -- -D warnings || \
		(echo "❌ Clippy warnings found"; exit 1)
	@pmat analyze complexity src/runtime --max-cyclomatic 10 || \
		(echo "❌ Functions exceed complexity 10"; exit 1)
	@cargo test runtime:: --no-fail-fast || \
		(echo "❌ Test failures"; exit 1)
	@echo "✅ Runtime passes all quality gates"

gate-wasm:
	@echo "🔒 WASM Quality Gate Check"
	@pmat tdg src/wasm --min-grade A+ --fail-on-violation || \
		(echo "❌ TDG score below A+"; exit 1)
	@cargo clippy --all-features -- -D warnings || \
		(echo "❌ Clippy warnings found"; exit 1)
	@pmat analyze complexity src/wasm --max-cyclomatic 10 || \
		(echo "❌ Functions exceed complexity 10"; exit 1)
	@cargo test wasm:: --no-fail-fast || \
		(echo "❌ Test failures"; exit 1)
	@echo "✅ WASM passes all quality gates"

gate-quality:
	@echo "🔒 Quality Infrastructure Gate Check"
	@pmat tdg src/testing src/quality src/utils --min-grade A+ --fail-on-violation || \
		(echo "❌ TDG score below A+"; exit 1)
	@cargo clippy --all-features -- -D warnings || \
		(echo "❌ Clippy warnings found"; exit 1)
	@pmat analyze complexity src/testing src/quality src/utils --max-cyclomatic 10 || \
		(echo "❌ Functions exceed complexity 10"; exit 1)
	@cargo test testing:: quality:: utils:: --no-fail-fast || \
		(echo "❌ Test failures"; exit 1)
	@echo "✅ Quality infrastructure passes all gates"

# ============================================================================
# COMBINED TARGETS
# ============================================================================

coverage-all: coverage-frontend coverage-backend coverage-runtime coverage-wasm coverage-quality
	@echo "📊 All category coverage reports generated"

gate-all: gate-frontend gate-backend gate-runtime gate-wasm gate-quality
	@echo "✅ All quality gates passed"

# ============================================================================
# TDD WORKFLOW HELPERS
# ============================================================================

tdd-watch:
	@cargo watch -x "nextest run" -i "target/*" -i "*.md"

complexity-check:
	@pmat analyze complexity src --max-cyclomatic 10 --format json | \
		jq -r '.violations[] | "❌ \(.file):\(.line) - \(.name) (CC=\(.complexity))"'

uncovered-functions:
	@cargo llvm-cov --lib --json | \
		jq -r '.functions[] | select(.summary.lines.percent < 80) | \
		"📍 \(.name): \(.summary.lines.percent)% coverage"'
```

## TDD Protocol (Mandatory)

### The Red-Green-Refactor Cycle

```bash
# Step 1: RED - Write failing test
cat > tests/frontend_test.rs << 'EOF'
#[test]
fn parser_handles_unicode() {
    let source = "let π = 3.14159";
    let ast = parse(source).unwrap();
    assert_eq!(ast.bindings[0].name, "π");
}
EOF
cargo test parser_handles_unicode  # MUST FAIL

# Step 2: GREEN - Minimal implementation
# Write ONLY enough code to pass

# Step 3: REFACTOR - Reduce complexity
pmat analyze complexity src/frontend/parser.rs
# If CC > 10, extract functions

# Step 4: VERIFY - Check coverage impact
make coverage-frontend
```

### Complexity Enforcement

Every function MUST satisfy:
```rust
// ❌ REJECTED: Complexity = 15
fn parse_expression(tokens: &[Token]) -> Expr {
    match tokens[0] {
        Token::If => { /* 5 branches */ }
        Token::Match => { /* 6 branches */ }
        Token::Let => { /* 4 branches */ }
    }
}

// ✅ ACCEPTED: Complexity = 5
fn parse_expression(tokens: &[Token]) -> Expr {
    match tokens[0] {
        Token::If => parse_if(tokens),
        Token::Match => parse_match(tokens),
        Token::Let => parse_let(tokens),
    }
}
```

### PMAT TDG Score Requirements

```bash
# Minimum A+ (95 points) enforced via:
pmat tdg src/frontend --format json | jq '.score'

# Score breakdown:
# - Test coverage: 40 points (need >80%)
# - Documentation: 20 points (all public items)
# - Complexity: 20 points (all functions ≤10)
# - Dependencies: 10 points (minimal coupling)
# - Security: 10 points (no unsafe without justification)
```

## Five Whys Analysis Template

When coverage remains below target:

**Problem**: Frontend at 65% after TDD cycle

1. **Why?** → Error recovery module at 20%
2. **Why?** → Complex state machine resists testing
3. **Why?** → Single function with CC=25
4. **Why?** → Monolithic error handler
5. **Why?** → No separation of concerns

**Root Cause**: Violation of complexity limit
**Solution**: Refactor into 5 functions (CC≤5 each), then test individually

## Language Architecture

### Compilation Pipeline

```
Source → Lexer → Parser → Type Elaboration → MIR → Optimization → Rust AST → rustc
                    ↓            ↓              ↓
                  REPL     Type Checking   Cranelift JIT
```

### Core Components

#### Frontend (45% coverage)
- **Lexer**: Maximal munch with Unicode support, 2-token lookahead
- **Parser**: Recursive descent + Pratt precedence (operators ≥10 levels)
- **AST**: Immutable, span-preserving, with visitor pattern
- **Error Recovery**: Panic mode with synchronization points
- **Diagnostics**: Rust-style with fix suggestions via tree-diff

#### Type System
- **Inference**: Algorithm W with level-based generalization
- **Extensions**: Row polymorphism, refinement types, gradual typing
- **Checking**: Bidirectional with local inference
- **Elaboration**: Explicit type applications, dictionary passing
- **Verification**: SMT-backed refinement checking (Z3 integration)

#### Backend (50% coverage)
- **Transpiler**: Direct Rust AST generation via syn/quote
- **Module System**: Cargo-compatible with implicit std prelude
- **FFI**: Zero-cost through type-directed marshalling
- **Optimization**: Escape analysis, devirtualization, inline caching
- **Targets**: Native (via rustc), WASM (wasm-bindgen), LLVM (future)

#### Runtime (40% coverage)
- **Interpreter**: Tree-walk with computed goto dispatch
- **JIT**: Cranelift for hot loops (>1000 iterations)
- **Memory**: Arena allocation with generation-based collection
- **Concurrency**: Actor model with mailbox isolation
- **MCP**: Native protocol support with compile-time validation

### Performance Invariants

| Metric | Target | Current | Method |
|--------|--------|---------|---------|
| Startup | <10ms | 12ms | Lazy std loading |
| REPL latency | <15ms | 18ms | Incremental parsing |
| Transpile overhead | <5% | 8% | Smarter monomorphization |
| Binary size | <5MB | 4.2MB | LTO + strip |
| Type check (1KLOC) | <100ms | 95ms | Constraint caching |

## Type System Specification

### Core Types
```
τ ::= α                    -- Type variable
    | τ → τ               -- Function
    | ∀α.τ                -- Universal quantification
    | {l₁:τ₁, ..., lₙ:τₙ} -- Record
    | τ₁ + ... + τₙ       -- Sum type
    | τ & φ               -- Refinement
    | τ?                  -- Gradual boundary
```

### Refinement Language
```
φ ::= e₁ ≡ e₂             -- Equality
    | e₁ < e₂             -- Ordering
    | φ ∧ φ               -- Conjunction
    | ∃x:τ.φ              -- Existential
    | μφ.φ                -- Recursive predicate
```

### Inference Algorithm

```rust
fn infer(env: &TypeEnv, expr: &Expr) -> Result<(Type, Constraints)> {
    match expr {
        Expr::Var(x) => env.lookup(x).instantiate(),
        Expr::App(f, x) => {
            let (tf, cf) = infer(env, f)?;
            let (tx, cx) = infer(env, x)?;
            let ret = Type::fresh();
            Ok((ret, cf + cx + unify(tf, tx → ret)))
        }
        Expr::Lam(x, body) => {
            let tx = Type::fresh();
            let env2 = env.extend(x, tx);
            let (tbody, cbody) = infer(&env2, body)?;
            Ok((tx → tbody, cbody))
        }
        // ... pattern matching, let-polymorphism
    }
}
```

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
- **Test Coverage**: Maintain >80% per category or justify exception