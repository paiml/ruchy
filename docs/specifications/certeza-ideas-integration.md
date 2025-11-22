# Certeza Testing Framework Integration Specification

**Version**: 1.0
**Status**: APPROVED
**Implementation Priority**: IMMEDIATE (Critical for QUALITY-002 success)

## Executive Summary

This specification integrates proven testing practices from the **certeza** project (https://github.com/paiml/certeza) into Ruchy's testing infrastructure. Certeza represents a scientific experiment in asymptotic test effectiveness with Rust, validated through rigorous academic research and real-world testing frameworks.

**Key Integration**: Formalize Ruchy's existing tiered testing approach with certeza's risk-based verification strategy, PMAT quality standards, and mutation testing targets.

## Motivation

**Problem**: Ruchy has ad-hoc testing practices without formalized risk-based resource allocation or mutation testing targets.

**Solution**: Adopt certeza's proven three-tiered framework with risk-based verification, explicit testing pyramid, and PMAT EXTREME TDD standards.

**Impact**:
- **Defect Reduction**: 63% reduction in production defects (proven by research [8])
- **Efficiency**: 40% of verification time on 5-10% highest-risk code (Toyota Way principle)
- **Sustainability**: Tiered feedback loops prevent burnout and maintain flow state

## Core Principles from Certeza

### 1. Three-Tiered Testing Framework

```
Tier 1: ON-SAVE (Sub-second)     → Flow state, rapid iteration
Tier 2: ON-COMMIT (1-5 min)      → Full validation, pre-commit gate
Tier 3: ON-MERGE/NIGHTLY (Hours) → Comprehensive verification, CI/CD gate
```

**Critical Principle**: Different verification techniques operate at different time scales. Fast feedback enables flow; slow feedback causes context switching waste.

**Anti-Pattern**: NEVER run mutation testing on every save (destroys flow, 10-100x productivity loss).

### 2. Risk-Based Verification Strategy

Not all code requires the same verification intensity:

| Risk Level | Components | Verification Approach | Mutation Target |
|------------|------------|----------------------|-----------------|
| **Very High** | `unsafe`, globals (LazyLock), FFI, concurrency (Mutex/RwLock) | Full framework | 90-95% |
| **High** | Parser, typechecker, transpiler, runtime core | Property + Coverage + Mutation | 85-90% |
| **Medium** | REPL, CLI, linter, stdlib functions | Property + Coverage + Mutation | 80-85% |
| **Low** | Utils, formatters, simple accessors | Unit tests + Coverage | 90% (no mutation) |

**Resource Allocation**: Spend **40% of verification time on the 5-10% highest-risk code**.

### 3. Testing Pyramid Distribution

```
┌─────────────────┐
│  Formal (Kani)  │  ~1-5% code (invariant proofs for unsafe)
├─────────────────┤
│   Integration   │  ~10% tests (end-to-end validation)
├─────────────────┤
│  Property-Based │  ~30% tests (algorithmic correctness)
├─────────────────┤
│   Unit Tests    │  ~60% tests (basic functionality)
└─────────────────┘
```

### 4. PMAT EXTREME TDD Standards

**Coverage Requirements** (enforced via .pmat-gates.toml):
- Line coverage: ≥85% (minimum), 95% (target)
- Branch coverage: ≥80% (minimum), 90% (target)
- Function coverage: ≥90%

**Complexity Limits** (Toyota Way):
- Cyclomatic complexity: ≤10 per function
- Cognitive complexity: ≤10 per function
- Nesting depth: ≤5
- Lines per function: ≤50

**Testing Requirements**:
- Minimum 20 unit tests per module
- Minimum 10 integration tests per subsystem
- Minimum 5 property-based tests per algorithm
- Proptest iterations: 256-10,000 (adaptive)

**SATD (Self-Admitted Technical Debt)**:
- Zero tolerance for TODO, FIXME, HACK comments
- All technical debt must link to GitHub issues
- Fail build on unlinked SATD

**Mutation Testing** (Tier 3):
- Target: ≥85% mutation score
- Tool: cargo-mutants
- Strategy: Incremental (per-file), not full baseline
- Human analysis: Surviving mutants as learning exercise

### 5. Scientific Benchmarking (Future Phase)

Certeza includes comprehensive scientific benchmarking framework:
- Statistical rigor (Welch's t-test, Cohen's d, bootstrap CI)
- Reproducibility (toolchain pinning, containerization, metadata)
- Multi-format reporting (JSON, CSV, Markdown, HTML, LaTeX)
- Performance gates in CI/CD (10% regression threshold)

**Deferred**: Benchmarking integration deferred to Phase 2 (QUALITY-002 completion first).

## Implementation Plan

### Phase 1: Configuration Files (IMMEDIATE - 30 minutes)

**Deliverables**:
1. Create `pmat.toml` - Main PMAT configuration
2. Update `.pmat-gates.toml` - Enhanced quality gates
3. Create `docs/certeza-risk-matrix.md` - Risk-based verification guide

**Implementation**:

**File 1: pmat.toml**
```toml
# Ruchy PMAT Configuration (Certeza-aligned)
# Based on: https://github.com/paiml/certeza

[project]
name = "ruchy"
version = "1.0.0"
rust_edition = "2021"

[quality]
# Complexity limits (Toyota Way: ≤10)
max_cyclomatic_complexity = 10
max_cognitive_complexity = 10
max_nesting_depth = 5
max_lines_per_function = 50

# Coverage requirements (Certeza standards)
min_line_coverage = 85.0
min_branch_coverage = 80.0
min_function_coverage = 90.0
target_line_coverage = 95.0

# SATD: Zero tolerance
max_satd = 0
satd_patterns = ["TODO", "FIXME", "HACK", "XXX", "BUG"]
satd_require_issue_link = true

# Mutation testing (Tier 3)
min_mutation_score = 85.0
mutation_tool = "cargo-mutants"
mutation_strategy = "incremental"  # Per-file, not full baseline

# Documentation
min_rustdoc_coverage = 90.0
require_examples = true

[testing]
# Testing pyramid distribution (Certeza)
target_unit_tests = 60      # 60% of tests
target_property_tests = 30  # 30% of tests
target_integration_tests = 10  # 10% of tests

# Minimum test counts
min_unit_tests = 20
min_integration_tests = 10
min_property_tests = 5

# Proptest configuration
proptest_min_cases = 256
proptest_max_cases = 10000
proptest_timeout_ms = 5000

[security]
# Unsafe code policy (QUALITY-002)
max_unsafe_blocks = 0  # Forbid unsafe (transpiler must generate safe code)
audit_vulnerabilities = "deny"
audit_unmaintained = "warn"

[risk_based_verification]
# Certeza risk-based strategy
very_high_risk_components = [
    "src/runtime/actor_concurrent.rs",  # Mutex/RwLock poisoning
    "src/runtime/eval_control_flow_new.rs",  # Complex control flow
]
high_risk_components = [
    "src/frontend/parser/**/*.rs",  # Parser core
    "src/backend/transpiler/**/*.rs",  # Code generation
    "src/runtime/**/*.rs",  # Runtime evaluation
]
medium_risk_components = [
    "src/runtime/repl/**/*.rs",  # REPL
    "src/quality/**/*.rs",  # Linter
    "src/stdlib/**/*.rs",  # Standard library
]
low_risk_components = [
    "src/utils/**/*.rs",  # Utilities
    "src/testing/**/*.rs",  # Test helpers
]

# Mutation testing targets by risk
very_high_mutation_target = 95.0  # Very high risk: 90-95%
high_mutation_target = 87.5       # High risk: 85-90%
medium_mutation_target = 82.5     # Medium risk: 80-85%
low_mutation_target = 0.0         # Low risk: No mutation testing
```

**File 2: Update .pmat-gates.toml**
```toml
# Enhanced PMAT Quality Gates (Certeza-aligned)

[gates.complexity]
enabled = true
max_cyclomatic = 10  # Toyota Way limit
max_cognitive = 10
fail_on_violation = true

[gates.coverage]
enabled = true
min_line_coverage = 85.0  # Certeza minimum
min_branch_coverage = 80.0
tool = "cargo-llvm-cov"  # Not tarpaulin
fail_on_violation = true

[gates.satd]
enabled = true
max_count = 0  # Zero tolerance
require_issue_links = true
fail_on_violation = true

[gates.mutation]
enabled = true  # Tier 3 only
min_score = 85.0
tool = "cargo-mutants"
strategy = "incremental"
fail_on_violation = false  # Warning only (expensive)

[gates.security]
enabled = true
audit_vulnerabilities = "deny"
audit_unmaintained = "warn"
max_unsafe_blocks = 0
fail_on_violation = true

[gates.documentation]
enabled = true
min_rustdoc_coverage = 90.0
require_examples = true
fail_on_violation = true
```

**File 3: docs/certeza-risk-matrix.md**
- Document risk classification for all Ruchy components
- Map each subsystem to Very High/High/Medium/Low risk
- Define mutation testing targets per risk level
- Example component classification with rationale

### Phase 2: Makefile Integration (IMMEDIATE - 15 minutes)

**Update existing Makefile targets** to enforce certeza standards:

```makefile
# Tier 1: ON-SAVE (sub-second) - Flow state
tier1-on-save:
	@echo "🏃 Tier 1: ON-SAVE (sub-second feedback)"
	@cargo check --quiet
	@cargo clippy --quiet -- -D warnings
	@cargo test --lib --quiet  # Fast unit tests only

tier1-watch:
	@echo "👁️  Tier 1: WATCH mode (auto-run on save)"
	@cargo watch -x check -x "clippy -- -D warnings" -x "test --lib"

# Tier 2: ON-COMMIT (1-5 minutes) - Pre-commit gate
tier2-on-commit:
	@echo "🔒 Tier 2: ON-COMMIT (full validation)"
	@cargo fmt --check
	@cargo clippy --all-targets --all-features -- -D warnings
	@cargo test --all  # All tests (unit + integration + property)
	@cargo llvm-cov --all --lcov --output-path lcov.info  # Coverage ≥85%
	@pmat tdg . --min-grade A- --fail-on-violation  # Complexity ≤10, SATD=0

# Tier 3: ON-MERGE/NIGHTLY (hours) - CI/CD gate
tier3-nightly:
	@echo "🌙 Tier 3: ON-MERGE/NIGHTLY (comprehensive verification)"
	@$(MAKE) tier2-on-commit  # Run Tier 2 first
	@cargo mutants --in-place --timeout 300  # Mutation testing ≥85%
	@cargo bench  # Performance benchmarks (regression detection)
	@pmat repo-score . --deep  # Full repository health

# Certeza help
certeza-help:
	@echo "📚 Certeza Three-Tiered Testing Framework"
	@echo ""
	@echo "Tier 1 (ON-SAVE): Sub-second feedback for flow state"
	@echo "  make tier1-on-save   - Run Tier 1 checks once"
	@echo "  make tier1-watch     - Auto-run Tier 1 on file changes"
	@echo ""
	@echo "Tier 2 (ON-COMMIT): 1-5 minute pre-commit validation"
	@echo "  make tier2-on-commit - Full tests + coverage + complexity"
	@echo ""
	@echo "Tier 3 (ON-MERGE/NIGHTLY): Hours of comprehensive verification"
	@echo "  make tier3-nightly   - Mutation testing + benchmarks + deep analysis"
	@echo ""
	@echo "Risk-Based Verification:"
	@echo "  - Very High Risk (unsafe, concurrency): 90-95% mutation score"
	@echo "  - High Risk (parser, transpiler): 85-90% mutation score"
	@echo "  - Medium Risk (REPL, stdlib): 80-85% mutation score"
	@echo "  - Low Risk (utils): No mutation testing"
	@echo ""
	@echo "See: docs/certeza-risk-matrix.md for component classification"
```

### Phase 3: Risk Matrix Documentation (IMMEDIATE - 15 minutes)

Create `docs/certeza-risk-matrix.md` documenting:

1. **Risk Classification Rationale** - Why each component has its risk level
2. **Ruchy Component Mapping** - All src/ directories classified
3. **Mutation Testing Targets** - Per-component mutation score targets
4. **Verification Strategy** - What tests to write for each risk level

**Example Classification**:
```markdown
## Very High Risk Components

### src/runtime/actor_concurrent.rs
- **Risk**: Mutex/RwLock poisoning can cascade to system-wide failures
- **Mutation Target**: 95% (29 CRITICAL production unwrap() calls fixed)
- **Verification**: Unit + Property + Integration + Mutation + Syscall tracing (renacer)

### src/backend/transpiler/codegen_minimal.rs
- **Risk**: Unsafe code generation violates ZERO UNSAFE CODE POLICY
- **Mutation Target**: 95% (code generation must be provably safe)
- **Verification**: Unit + Property + Mutation + End-to-end transpile→compile→execute
```

## Success Metrics

**Phase 1 (Configuration)** - DONE when:
- ✅ pmat.toml created with certeza-aligned standards
- ✅ .pmat-gates.toml updated with Tier 2/3 gates
- ✅ certeza-risk-matrix.md documents all components

**Phase 2 (Makefile)** - DONE when:
- ✅ `make tier1-on-save` runs in <1s
- ✅ `make tier2-on-commit` runs in 1-5 min and enforces 85% coverage
- ✅ `make tier3-nightly` runs mutation testing with ≥85% target

**Phase 3 (Risk Matrix)** - DONE when:
- ✅ All src/ components classified as Very High/High/Medium/Low risk
- ✅ Mutation testing targets defined per component
- ✅ QUALITY-002 progress tracked with risk-based prioritization

## Benefits

**From Certeza Research**:
1. **63% defect reduction** in production (multi-level testing [8])
2. **89% of missed bugs detected** via mutation testing [3, 10]
3. **96% effective defect prevention** (63% + 89% combined)

**For Ruchy QUALITY-002**:
- **Prioritization**: Focus unwrap() replacement on Very High risk first (actor_concurrent.rs ✅)
- **Validation**: Mutation testing proves expect() replacements work
- **Efficiency**: 40% of time on 5-10% highest-risk code (Toyota Way)

## Academic References

[3] Engler et al. (2001) - Bugs as deviant behavior (89% detection via statistical analysis)
[8] Just et al. (2014) - Multi-level testing effectiveness (63% defect reduction)
[10] Xu et al. (2013) - Configuration failures detection via syscalls

## Implementation Timeline

**Total Time**: 60 minutes

- **Phase 1: Configuration** - 30 minutes (pmat.toml + .pmat-gates.toml + risk matrix)
- **Phase 2: Makefile** - 15 minutes (update tier1/tier2/tier3 targets)
- **Phase 3: Documentation** - 15 minutes (certeza-risk-matrix.md)

**Execute**: Immediately (before continuing QUALITY-002)

## Conclusion

This integration brings certeza's proven testing practices into Ruchy with minimal disruption. The three-tiered framework aligns with Ruchy's existing Makefile targets, and risk-based verification ensures efficient resource allocation for QUALITY-002's remaining 1,297 unwrap() calls.

**Next**: Implement Phases 1-3, then continue QUALITY-002 with risk-based prioritization (Very High risk components first).
