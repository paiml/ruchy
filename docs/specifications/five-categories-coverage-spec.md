# Five Categories Coverage Specification

## Executive Summary

This specification establishes a systematic divide-and-conquer approach to achieve >80% test coverage across five major components of the Ruchy compiler. By isolating each component and applying rigorous quality standards, we transform an overwhelming coverage goal into five manageable sub-sprints.

## Core Principles

### 1. Toyota Way Foundation
- **Stop the Line**: Any test failure, build warning, or quality violation immediately halts progress
- **Five Whys**: Every coverage gap must be analyzed to root cause
- **Jidoka**: Quality built into each component, not inspected in later
- **Kaizen**: Continuous improvement through incremental, focused sprints

### 2. Quality Gates (Non-Negotiable)
- **TDD Mandatory**: Write test FIRST, then implementation
- **Complexity Limit**: Cyclomatic complexity ≤10 per function
- **TDG Grade**: Maintain A+ (≥95 points) for each category
- **Zero Tolerance**: No broken tests, no clippy warnings, no build failures

## The Five Categories

### Category 1: Frontend (Parser & Lexer)
**Target**: >80% coverage for `src/frontend/`

**Scope**:
- `lexer.rs` - Tokenization
- `parser/` - AST construction
- `ast.rs` - AST definitions
- `error_recovery.rs` - Parser error handling
- `diagnostics.rs` - Error reporting

**Current Coverage**: ~45% (estimated)

### Category 2: Backend (Transpiler & Compiler)
**Target**: >80% coverage for `src/backend/`

**Scope**:
- `transpiler/` - Rust code generation
- `compiler.rs` - Binary compilation
- `module_loader.rs` - Module system
- `module_resolver.rs` - Import resolution
- `arrow_integration.rs` - DataFrame operations

**Current Coverage**: ~50% (estimated)

### Category 3: Runtime (Interpreter & REPL)
**Target**: >80% coverage for `src/runtime/`

**Scope**:
- `interpreter.rs` - Direct execution
- `repl.rs` - Interactive shell
- `actor.rs` - Actor system
- `cache.rs` - Performance optimization
- `grammar_coverage.rs` - Language feature tracking

**Current Coverage**: ~40% (estimated)

### Category 4: WebAssembly Support
**Target**: >80% coverage for `src/wasm/`

**Scope**:
- `component.rs` - WASM generation
- `deployment.rs` - Platform deployment
- `notebook.rs` - Jupyter integration
- `portability.rs` - Cross-platform support

**Current Coverage**: ~15% (estimated)

### Category 5: Quality & Testing Infrastructure
**Target**: >80% coverage for `src/testing/` and `src/quality/`

**Scope**:
- `testing/generators.rs` - Property test generation
- `testing/harness.rs` - Test execution framework
- `quality/scoring.rs` - Code quality metrics
- `utils/` - Common utilities

**Current Coverage**: ~60% (estimated)

## Makefile Targets

### Main Targets
```makefile
# Individual category coverage targets
coverage-frontend:
	@echo "🎯 Frontend Coverage Sprint"
	cargo llvm-cov --lib --html --include-path "src/frontend/**/*.rs" \
		--output-path target/coverage/frontend
	@echo "✅ Frontend coverage report: target/coverage/frontend/index.html"

coverage-backend:
	@echo "🎯 Backend Coverage Sprint"
	cargo llvm-cov --lib --html --include-path "src/backend/**/*.rs" \
		--output-path target/coverage/backend
	@echo "✅ Backend coverage report: target/coverage/backend/index.html"

coverage-runtime:
	@echo "🎯 Runtime Coverage Sprint"
	cargo llvm-cov --lib --html --include-path "src/runtime/**/*.rs" \
		--output-path target/coverage/runtime
	@echo "✅ Runtime coverage report: target/coverage/runtime/index.html"

coverage-wasm:
	@echo "🎯 WebAssembly Coverage Sprint"
	cargo llvm-cov --lib --html --include-path "src/wasm/**/*.rs" \
		--output-path target/coverage/wasm
	@echo "✅ WASM coverage report: target/coverage/wasm/index.html"

coverage-quality:
	@echo "🎯 Quality Infrastructure Coverage Sprint"
	cargo llvm-cov --lib --html \
		--include-path "src/testing/**/*.rs" \
		--include-path "src/quality/**/*.rs" \
		--include-path "src/utils/**/*.rs" \
		--output-path target/coverage/quality
	@echo "✅ Quality coverage report: target/coverage/quality/index.html"

# Combined target to check all categories
coverage-all: coverage-frontend coverage-backend coverage-runtime coverage-wasm coverage-quality
	@echo "📊 All category coverage reports generated"
```

### Quality Gate Targets
```makefile
# Pre-sprint quality gates for each category
gate-frontend:
	@echo "🔒 Frontend Quality Gate"
	pmat tdg src/frontend --min-grade A+ --fail-on-violation
	cargo clippy --all-features -- -D warnings
	cargo test frontend:: --no-fail-fast

gate-backend:
	@echo "🔒 Backend Quality Gate"
	pmat tdg src/backend --min-grade A+ --fail-on-violation
	cargo clippy --all-features -- -D warnings
	cargo test backend:: --no-fail-fast

gate-runtime:
	@echo "🔒 Runtime Quality Gate"
	pmat tdg src/runtime --min-grade A+ --fail-on-violation
	cargo clippy --all-features -- -D warnings
	cargo test runtime:: --no-fail-fast

gate-wasm:
	@echo "🔒 WASM Quality Gate"
	pmat tdg src/wasm --min-grade A+ --fail-on-violation
	cargo clippy --all-features -- -D warnings
	cargo test wasm:: --no-fail-fast

gate-quality:
	@echo "🔒 Quality Infrastructure Gate"
	pmat tdg src/testing src/quality src/utils --min-grade A+ --fail-on-violation
	cargo clippy --all-features -- -D warnings
	cargo test testing:: quality:: utils:: --no-fail-fast
```

## Sprint Protocol

### Pre-Sprint Checklist
```bash
# For each category sprint, execute:
make gate-<category>           # Ensure quality baseline
make coverage-<category>        # Measure current coverage
pmat tdg src/<category> --format=markdown > baseline.md  # Document baseline
```

### During Sprint (TDD Cycle)
```bash
# For each uncovered function:
1. Write failing test first
2. Run: cargo test <test_name> -- --nocapture
3. Implement minimum code to pass
4. Check complexity: pmat analyze complexity <file> --max-cyclomatic 10
5. Run: make gate-<category>
6. Measure: make coverage-<category>
```

### Post-Sprint Validation
```bash
# Must pass ALL quality gates:
make gate-<category>           # Zero failures allowed
make coverage-<category>        # Must show >80%
pmat tdg src/<category> --min-grade A+ --fail-on-violation
cargo test <category>:: --no-fail-fast  # 100% pass rate
```

## Five Whys Analysis Template

When coverage is below 80%, apply Five Whys:

**Problem**: Category X has only Y% coverage

1. **Why?** → Specific modules have low coverage
2. **Why?** → Complex functions weren't tested
3. **Why?** → Functions exceed complexity limit
4. **Why?** → Original implementation didn't follow TDD
5. **Why?** → No systematic testing strategy existed

**Root Cause**: Lack of systematic TDD approach
**Solution**: This specification with enforced quality gates

## Success Metrics

### Per-Category Metrics
- **Coverage**: ≥80% line coverage
- **TDG Score**: ≥95 points (A+ grade)
- **Complexity**: 100% functions ≤10 cyclomatic
- **Test Pass Rate**: 100%
- **Clippy Warnings**: 0
- **Build Errors**: 0

### Overall Project Metrics
- **Total Coverage**: ≥75% (weighted average)
- **Test Count**: >2000 tests
- **Test Execution Time**: <5 seconds
- **Property Test Coverage**: >50% of modules

## Implementation Schedule

### Sprint Sequence (Recommended)
1. **Sprint 1**: Quality Infrastructure (1 week)
   - Already at ~60%, easiest to reach 80%
   - Sets testing patterns for other categories

2. **Sprint 2**: Frontend (1 week)
   - Core functionality, most mature
   - Good test patterns already exist

3. **Sprint 3**: Backend (1 week)
   - Builds on frontend tests
   - Critical for compilation

4. **Sprint 4**: Runtime (1 week)
   - Can reuse frontend/backend test cases
   - Interactive testing possible

5. **Sprint 5**: WebAssembly (2 weeks)
   - Most complex, lowest current coverage
   - May require mocking/stubbing

## Enforcement Mechanism

### Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

CATEGORY=$(git diff --cached --name-only | grep -E "src/(frontend|backend|runtime|wasm|testing|quality)" | head -1 | cut -d'/' -f2)

if [ -n "$CATEGORY" ]; then
    echo "🔒 Quality gate for $CATEGORY"
    make gate-$CATEGORY || {
        echo "❌ Quality gate failed for $CATEGORY"
        echo "Run: make coverage-$CATEGORY to see coverage"
        echo "Run: pmat tdg src/$CATEGORY to see quality issues"
        exit 1
    }
fi
```

### CI/CD Integration
```yaml
# .github/workflows/coverage.yml
coverage:
  strategy:
    matrix:
      category: [frontend, backend, runtime, wasm, quality]
  steps:
    - name: Quality Gate
      run: make gate-${{ matrix.category }}
    - name: Coverage Check
      run: |
        make coverage-${{ matrix.category }}
        # Parse coverage and fail if <80%
```

## Common Pitfalls & Solutions

### Pitfall 1: Writing Tests After Code
**Solution**: Enforce TDD through pre-commit hooks that check test files are modified before implementation files

### Pitfall 2: Complex Functions Resist Testing
**Solution**: Refactor FIRST using PMAT guidance, then test smaller functions

### Pitfall 3: Mocking Dependencies
**Solution**: Create `testing/mocks/` module with standard test doubles for each category

### Pitfall 4: Slow Test Execution
**Solution**: Use `cargo nextest` for parallel execution, property test sampling for speed

### Pitfall 5: Coverage vs Quality Trade-off
**Solution**: TDG score ensures quality isn't sacrificed for coverage numbers

## Appendix: Coverage Commands Reference

```bash
# Detailed coverage with source
cargo llvm-cov --lib --show-instantiations --show-line-counts-or-regions

# Coverage for specific test
cargo llvm-cov --lib --test frontend_parser_test

# Coverage diff between commits
cargo llvm-cov --lib --json > new.json
git stash && cargo llvm-cov --lib --json > old.json && git stash pop
diff old.json new.json

# Find uncovered functions
cargo llvm-cov --lib --json | jq '.functions[] | select(.coverage < 0.8)'

# Generate lcov for IDE integration
cargo llvm-cov --lib --lcov --output-path lcov.info
```

## Conclusion

This specification transforms the daunting task of achieving high test coverage into five focused, manageable sprints. By applying Toyota Way principles, enforcing TDD, and maintaining strict quality gates, we ensure that coverage improvements enhance rather than compromise code quality.

Success is not measured by coverage percentage alone, but by the systematic application of quality practices that make 80% coverage a natural outcome of disciplined development.