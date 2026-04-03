# Sub-spec: Five Categories Coverage — Strategy and Language Architecture

**Parent:** [five-categories-coverage-spec.md](../five-categories-coverage-spec.md) Sections 1-4

---

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
