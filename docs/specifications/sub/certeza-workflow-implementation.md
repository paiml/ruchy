# Sub-spec: Testing Quality — Workflow and Implementation

**Parent:** [improve-testing-quality-using-certeza-concepts.md](../improve-testing-quality-using-certeza-concepts.md) Sections 2-4

---

## 2. Certeza Three-Tiered Workflow

### 2.1 Tier 1: On-Save (Sub-Second Feedback)

**Goal**: Enable developer flow state through instant feedback.

**Time Budget**: <1 second per save.

**Verification Techniques**:
1. **Cargo check**: Syntax and type checking (0.1-0.5s)
2. **Cargo clippy**: Linting (0.2-0.8s)
3. **Fast unit tests**: Critical path tests only (0.1-0.3s)

**Implementation**:
```makefile
# Makefile target for on-save hook
.PHONY: tier1-on-save
tier1-on-save:
	@cargo check --quiet
	@cargo clippy --quiet -- -D warnings
	@cargo test --lib fast:: --quiet
```

**Pre-commit Hook Integration**:
```bash
# .git/hooks/pre-save (via watchman or cargo-watch)
#!/bin/bash
make tier1-on-save || exit 1
```

**Current Ruchy Implementation**: Partial (cargo check via PMAT, no on-save automation)

**Gap**: Need file watcher integration (`cargo watch -x "make tier1-on-save"`)

---

### 2.2 Tier 2: On-Commit (1-5 Minutes)

**Goal**: Prevent problematic commits from entering repository.

**Time Budget**: 1-5 minutes per commit.

**Verification Techniques**:
1. **Full unit test suite**: All `cargo test --lib`
2. **Property-based tests**: `PROPTEST_CASES=100` (Sprint 88 standard)
3. **Integration tests**: End-to-end transpile→compile→execute
4. **Coverage analysis**: Enforce ≥95% line, ≥90% branch
5. **Complexity gates**: PMAT TDG ≥A- (≤10 cyclomatic complexity)

**Implementation**:
```makefile
# Makefile target for pre-commit hook
.PHONY: tier2-on-commit
tier2-on-commit:
	@echo "Tier 2: Full test suite + coverage..."
	@cargo test --lib --release --quiet
	@cargo test --test --release --quiet
	@env PROPTEST_CASES=100 cargo test property:: --release --quiet
	@cargo llvm-cov --branch --fail-under-lines 95 --fail-under-branch 90
	@pmat tdg . --min-grade A- --fail-on-violation
```

**Pre-commit Hook** (already implemented via PMAT):
```bash
#!/bin/bash
make tier2-on-commit || {
  echo "❌ Tier 2 verification failed. Fix violations before committing."
  exit 1
}
```

**Current Ruchy Implementation**: Strong (PMAT hooks, property tests, coverage tracking)

**Gap**: Branch coverage not enforced (only line coverage)

---

### 2.3 Tier 3: On-Merge/Nightly (Hours)

**Goal**: Maximum confidence before main branch integration.

**Time Budget**: Hours (nightly CI or pre-merge).

**Verification Techniques**:
1. **Mutation testing**: `cargo mutants` targeting ≥85% mutation score
2. **Formal verification**: Kani for unsafe blocks and critical invariants
3. **Performance benchmarks**: Ensure no regressions
4. **Cross-platform validation**: Linux, macOS, Windows
5. **RuchyRuchy smoke testing**: Validate with `ruchydbg` (v1.13.0+)

**Implementation**:
```makefile
# Makefile target for nightly CI
.PHONY: tier3-nightly
tier3-nightly:
	@echo "Tier 3: Mutation testing + formal verification..."
	# Incremental mutation testing (5-30 min per file)
	@for file in src/frontend/parser/*.rs; do \
		cargo mutants --file $$file --timeout 300 || exit 1; \
	done
	# Formal verification for unsafe blocks (if any)
	@cargo kani --harness verify_unsafe_globals || true
	# Performance benchmarks
	@cargo bench --no-fail-fast
	# RuchyRuchy smoke testing
	@cd ../ruchyruchy && cargo test --test property_based_tests --release
```

**CI/CD Integration** (GitHub Actions):
```yaml
# .github/workflows/tier3-nightly.yml
name: Tier 3 Nightly Verification
on:
  schedule:
    - cron: '0 2 * * *'  # 2 AM daily
jobs:
  mutation-testing:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: cargo install cargo-mutants
      - run: make tier3-nightly
```

**Current Ruchy Implementation**: Partial (mutation testing ad-hoc, no formal verification)

**Gap**: Need systematic nightly CI for Tier 3 verification

---

## 3. Risk-Based Resource Allocation

### 3.1 Risk Stratification Model

Certeza recommends spending **40% of verification time on the 5-10% highest-risk code**. For Ruchy:

**Very High-Risk (5% of codebase, 40% of verification time)**:
- **Components**: Unsafe blocks, globals (`LazyLock<Mutex<T>>`), FFI, WASM bindings
- **Verification**:
  - Unit tests: 100% coverage
  - Property tests: Comprehensive invariants
  - Mutation tests: 95%+ score
  - Formal verification: Kani proofs for memory safety
- **Rationale**: Unsafe code can cause undefined behavior, memory corruption, security vulnerabilities

**High-Risk (15% of codebase, 35% of verification time)**:
- **Components**: Parser, type inference, code generation, borrow checker integration
- **Verification**:
  - Unit tests: 95%+ coverage
  - Property tests: Algorithmic invariants (e.g., parser always produces valid AST)
  - Mutation tests: 85%+ score
  - Integration tests: Full transpile→compile→execute pipeline
- **Rationale**: Correctness bugs in these modules produce wrong Rust code (subtle runtime failures)

**Medium-Risk (50% of codebase, 20% of verification time)**:
- **Components**: REPL, CLI, linter, formatter, runtime evaluator
- **Verification**:
  - Unit tests: 85%+ coverage
  - Property tests: Selected modules (REPL state management, linter rules)
  - Mutation tests: File-by-file as time permits
- **Rationale**: Bugs are visible and debuggable (REPL crashes, linter false positives)

**Low-Risk (30% of codebase, 5% of verification time)**:
- **Components**: Simple accessors, utilities, string formatting, documentation
- **Verification**:
  - Unit tests: 70%+ coverage
  - Doctests: Public API examples
- **Rationale**: Bugs are trivial to detect and fix

---

### 3.2 Verification Time Budget (Example Sprint)

Assume 40-hour sprint with 25% allocated to testing (10 hours):

| Risk Level | Codebase % | Time Budget | Modules |
|------------|-----------|-------------|---------|
| Very High  | 5%        | 4.0 hours   | Unsafe blocks, globals, FFI |
| High       | 15%       | 3.5 hours   | Parser, type inference, codegen |
| Medium     | 50%       | 2.0 hours   | REPL, CLI, linter, runtime |
| Low        | 30%       | 0.5 hours   | Utilities, formatters, docs |
| **Total**  | **100%**  | **10 hours** | |

**Current Ruchy Practice**: Roughly uniform allocation (wasteful on low-risk code, insufficient on high-risk)

**Certeza Optimization**: Concentrate effort on critical modules

---

## 4. Implementing Certeza in Ruchy

### 4.1 Phase 1: Infrastructure (Sprint 1-2)

**Objective**: Enable three-tiered workflow with tooling.

**Tasks**:
1. **Tier 1 Automation**: File watcher with `cargo-watch`
   ```bash
   cargo install cargo-watch
   cargo watch -x "make tier1-on-save"
   ```

2. **Tier 2 Enhancement**: Add branch coverage to pre-commit
   ```makefile
   tier2-on-commit:
       @cargo llvm-cov --branch --fail-under-lines 95 --fail-under-branch 90
   ```

3. **Tier 3 CI**: GitHub Actions nightly pipeline
   - Mutation testing (incremental, file-by-file)
   - RuchyRuchy property tests (14,000+ cases)
   - Performance benchmarks

**Success Criteria**:
- Developers get <1s feedback on save
- Commits blocked if coverage <95% or TDG <A-
- Nightly CI runs full mutation suite

---

### 4.2 Phase 2: Risk Stratification (Sprint 3-4)

**Objective**: Map Ruchy modules to risk levels, allocate verification accordingly.

**Tasks**:
1. **Risk Assessment**: Classify all modules into Very High/High/Medium/Low
   ```yaml
   # docs/testing/risk-stratification.yaml
   very_high_risk:
     - src/codegen/unsafe_globals.rs  # Uses LazyLock<Mutex<T>>
     - src/wasm/bindings.rs           # FFI boundary
   high_risk:
     - src/frontend/parser/           # Parser correctness critical
     - src/typechecker/inference.rs   # Type soundness critical
     - src/codegen/transpiler.rs      # Code generation correctness
   medium_risk:
     - src/repl/evaluator.rs          # Runtime errors visible
     - src/cli/commands.rs            # CLI bugs user-facing
   low_risk:
     - src/utils/string_helpers.rs    # Simple utilities
   ```

2. **Test Coverage Audit**: Measure current coverage by risk level
   ```bash
   # Very High-Risk modules
   cargo llvm-cov --branch --package ruchy --lib \
     --include-files src/codegen/unsafe_globals.rs

   # Expected: 100% line, 100% branch, 95%+ mutation
   ```

3. **Gap Analysis**: Identify under-tested high-risk modules

**Success Criteria**:
- Very High-Risk: 100% line, 100% branch, 95%+ mutation
- High-Risk: 95%+ line, 90%+ branch, 85%+ mutation
- Medium-Risk: 85%+ line, 80%+ branch, mutation as time permits
- Low-Risk: 70%+ line, doctests for public API

---

### 4.3 Phase 3: Property Testing Expansion (Sprint 5-6)

**Objective**: Achieve 80% property test coverage (Sprint 88 pattern).

**Tasks**:
1. **Parser Properties** (High-Risk):
   ```rust
   // tests/properties/parser_properties.rs
   use proptest::prelude::*;

   proptest! {
       #[test]
       fn parse_always_produces_valid_ast(code in ".*") {
           let result = parse_ruchy_code(&code);
           // Property: Parser never panics, always returns Result
           assert!(result.is_ok() || result.is_err());
       }

       #[test]
       fn parse_roundtrip_preserves_semantics(ast: RuchyAST) {
           let code = ast.to_string();
           let reparsed = parse_ruchy_code(&code).unwrap();
           assert_eq!(ast, reparsed);
       }
   }
   ```

2. **Type Inference Properties** (High-Risk):
   ```rust
   proptest! {
       #[test]
       fn type_inference_is_deterministic(expr: Expr) {
           let type1 = infer_type(&expr);
           let type2 = infer_type(&expr);
           assert_eq!(type1, type2);
       }

       #[test]
       fn unification_is_idempotent(t1: Type, t2: Type) {
           let unified = unify(&t1, &t2);
           if let Ok(u) = unified {
               assert_eq!(unify(&u, &u), Ok(u.clone()));
           }
       }
   }
   ```

3. **Code Generation Properties** (High-Risk):
   ```rust
   proptest! {
       #[test]
       fn generated_rust_always_compiles(ast: RuchyAST) {
           let rust_code = transpile_to_rust(&ast);
           let compile_result = rustc_compile(&rust_code);
           assert!(compile_result.is_ok());
       }

       #[test]
       fn no_unsafe_in_generated_code(ast: RuchyAST) {
           let rust_code = transpile_to_rust(&ast);
           assert!(!rust_code.contains("unsafe {"));
       }
   }
   ```

**Success Criteria**: 80% of modules have property tests (current: ~40%)

---

### 4.4 Phase 4: Mutation Testing Systematic Coverage (Sprint 7-8)

**Objective**: Achieve ≥85% mutation score for High and Very High-Risk modules.

**Tasks**:
1. **Incremental Mutation Testing**:
   ```bash
   # Run file-by-file to avoid 10+ hour baseline
   for file in src/frontend/parser/*.rs; do
     cargo mutants --file $file --timeout 300 --output mutations.txt
   done
   ```

2. **Mutation-Driven Test Writing**:
   - Run mutation test to find gaps
   - Write targeted tests for MISSED mutations
   - Re-run to validate improvement
   - Repeat until ≥85% mutation score

3. **Pre-commit Mutation Gate** (High-Risk files only):
   ```bash
   # .git/hooks/pre-commit
   CHANGED_FILES=$(git diff --cached --name-only | grep -E 'src/(frontend|typechecker|codegen)')
   for file in $CHANGED_FILES; do
     cargo mutants --file $file --timeout 300 --min-score 85 || exit 1
   done
   ```

**Success Criteria**: ≥85% mutation score for all High/Very High-Risk modules

---

### 4.5 Phase 5: Selective Formal Verification (Sprint 9-10)

**Objective**: Prove critical invariants using Kani for Very High-Risk modules.

**Tasks**:
1. **Kani Setup**:
   ```bash
   cargo install --locked kani-verifier
   cargo kani setup
   ```

2. **Verify Unsafe Blocks** (GitHub Issue #132):
   ```rust
   // src/codegen/globals.rs
   use std::sync::LazyLock;
   use std::sync::Mutex;

   static GLOBALS: LazyLock<Mutex<HashMap<String, Value>>> =
       LazyLock::new(|| Mutex::new(HashMap::new()));

   #[cfg(kani)]
   #[kani::proof]
   fn verify_globals_thread_safety() {
       // Kani proof that GLOBALS is thread-safe
       kani::assume(/* thread safety invariants */);
       let handle1 = std::thread::spawn(|| {
           GLOBALS.lock().unwrap().insert("x".to_string(), Value::Int(42));
       });
       let handle2 = std::thread::spawn(|| {
           GLOBALS.lock().unwrap().get("x");
       });
       handle1.join().unwrap();
       handle2.join().unwrap();
       // Kani verifies no data races
   }
   ```

3. **Verify Critical Invariants**:
   ```rust
   #[cfg(kani)]
   #[kani::proof]
   fn verify_parser_no_panic_on_any_input() {
       let input: String = kani::any();
       let _ = parse_ruchy_code(&input); // Must not panic
   }
   ```

**Success Criteria**: Kani proofs pass for all unsafe blocks and critical invariants

---
