# Sprint 7 Phase 4: Mutation Testing Approach

**Date**: 2025-10-04
**Version**: v3.67.0
**Status**: ⏳ **INFRASTRUCTURE COMPLETE** - Ready for execution
**Target**: ≥90% mutation kill rate (wasm-labs target: 99.4%)

## Executive Summary

Phase 4 establishes mutation testing infrastructure using cargo-mutants to verify test suite effectiveness. The configuration is complete and ready for execution. Mutation testing will run iteratively to achieve ≥90% kill rate across core modules.

## Mutation Testing Overview

### What is Mutation Testing?

Mutation testing verifies that tests actually catch bugs by:
1. **Mutating code** - Change `==` to `!=`, `0` to `1`, `+` to `-`, etc.
2. **Running tests** - Execute full test suite against mutated code
3. **Checking results** - Tests should FAIL (catch the mutation)

### Why Mutation Testing?

- **Validates test quality**: Ensures tests detect real bugs, not just pass
- **Finds test gaps**: Reveals untested code paths
- **Prevents regression**: Strong mutation score means comprehensive coverage
- **Quality assurance**: Proven by wasm-labs (99.4% kill rate)

## Infrastructure Setup

### cargo-mutants Configuration

**File**: `.cargo/mutants.toml`

```toml
# Exclude files that should not be mutated
exclude_globs = [
    "src/wasm/bindings.rs",        # Auto-generated
    "src/wasm/wasm_bindgen_*",     # WASM bindings
    "tests/**",                     # Test files
    "benches/**",                   # Benchmarks
    "**/metadata.rs",              # Metadata only
    "build.rs",                     # Build scripts
    "examples/**",                  # Example code
]

# Exclude non-behavioral mutants
exclude_re = [
    "seed.*=.*42",                 # RNG seeds
    "version.*=",                   # Version strings
    'Error::.*\(".*"\)',           # Error messages
    'format!\(".*"\)',             # Debug formatting
    '".*" =>',                      # String literals in matches
]

# Timeout configuration
timeout_multiplier = 3.0           # 3x normal test time
minimum_test_timeout = 20.0        # 20 seconds minimum

# Output
output = "target/mutants"
```

### Verification

**Sample Analysis** (src/wasm/repl.rs):
```
Found 34 mutants to test:
- replace WasmRepl::eval -> Result<String, JsValue> with Ok(String::new())
- replace WasmRepl::eval -> Result<String, JsValue> with Ok("xyzzy".into())
- replace - with + in WasmRepl::eval
- replace - with / in WasmRepl::eval
- replace < with == in WasmHeap::major_gc
- replace < with > in WasmHeap::major_gc
- replace && with || in WasmHeap::major_gc
... and 27 more
```

**Status**: ✅ Configuration validated, ready for execution

## Execution Strategy

### Phase 4.1: Core Module Testing (Priority)

1. **Parser Module** (`src/frontend/parser/`)
   ```bash
   cargo mutants --file src/frontend/parser/mod.rs
   ```
   - Target: ≥90% kill rate
   - Critical for language correctness
   - Property tests should catch most mutations

2. **Transpiler Module** (`src/backend/transpiler/`)
   ```bash
   cargo mutants --file src/backend/transpiler/mod.rs
   ```
   - Target: ≥90% kill rate
   - Ensures correct Rust generation
   - Integration tests validate output

3. **Interpreter Module** (`src/runtime/interpreter.rs`)
   ```bash
   cargo mutants --file src/runtime/interpreter.rs
   ```
   - Target: ≥90% kill rate
   - Critical for evaluation correctness
   - Property tests verify arithmetic invariants

4. **WASM REPL** (`src/wasm/repl.rs`)
   ```bash
   cargo mutants --file src/wasm/repl.rs
   ```
   - Target: ≥90% kill rate
   - Focus of Sprint 7
   - E2E tests validate browser behavior

### Phase 4.2: Iterative Improvement

For each module with <90% kill rate:

1. **Find Survivors**:
   ```bash
   cargo mutants --file <module> --list --caught false
   ```

2. **Analyze Why They Survived**:
   - Missing test coverage?
   - Weak assertions?
   - Non-behavioral mutation?

3. **Add Targeted Tests**:
   ```rust
   #[test]
   fn test_catches_specific_mutation() {
       // Test that would fail if mutation applied
       let result = function_under_test();
       assert_eq!(result, expected_value); // Specific assertion
   }
   ```

4. **Retest**:
   ```bash
   cargo mutants --file <module>
   ```

5. **Repeat until ≥90%**

### Phase 4.3: Full Codebase Analysis

```bash
# Run mutation testing on entire codebase
cargo mutants --workspace

# Generate HTML report
cargo mutants --workspace --output target/mutants/report.html

# Summary statistics
cargo mutants --workspace --list
```

## Expected Results Format

```
Mutation Testing Results
========================
Module: src/frontend/parser/mod.rs

Total mutants: 145
- Caught: 131 (90.3%)  ✅ TARGET MET
- Missed: 9 (6.2%)     🔍 Need analysis
- Unviable: 5 (3.4%)   ⚪ Don't compile

Kill Rate: 93.6% ✅

Survivors requiring attention:
1. src/frontend/parser/mod.rs:218: replace return Ok(expr) with Ok(Expr::Null)
   → Add test verifying correct AST structure

2. src/frontend/parser/mod.rs:445: replace == with != in error check
   → Add test with intentional parse error
```

## Success Criteria

### Phase 4 Targets
- ✅ cargo-mutants installed and configured
- ✅ Configuration validated with sample run
- ⏳ Parser module: ≥90% kill rate
- ⏳ Transpiler module: ≥90% kill rate
- ⏳ Interpreter module: ≥90% kill rate
- ⏳ WASM REPL: ≥90% kill rate
- ⏳ Overall codebase: ≥90% kill rate

### Quality Metrics
- **Kill Rate**: Percentage of mutants caught by tests
- **Test Effectiveness**: How well tests detect bugs
- **Coverage Gaps**: Areas needing more tests
- **Regression Prevention**: Strong mutation score prevents bugs

## Time Estimates

Based on cargo-mutants performance:
- **Single file** (50-100 mutants): 5-15 minutes
- **Module** (200-500 mutants): 30-60 minutes
- **Full codebase** (2000+ mutants): 4-8 hours

**Strategy**: Run targeted tests on core modules first, then full codebase overnight

## Integration with Existing Tests

### Test Suite Synergy

1. **Unit Tests**: Catch basic mutations (return values, operators)
2. **Property Tests**: Catch mathematical/logical mutations (200K cases)
3. **E2E Tests**: Catch integration mutations (39 scenarios)
4. **Mutation Tests**: Validate all above actually work

**Hypothesis**: Our comprehensive test suite (3405+ tests + 200K property cases + 39 E2E) should achieve high kill rate with minimal additions.

## Next Steps

### Immediate (This Session)
1. ✅ Configure cargo-mutants
2. ✅ Verify infrastructure works
3. ✅ Document approach
4. ⏳ Update roadmap

### Follow-up (Next Session)
1. Run mutation tests on parser module
2. Analyze survivors and add tests
3. Run mutation tests on transpiler
4. Run mutation tests on interpreter
5. Run full codebase mutation testing
6. Generate comprehensive report
7. Achieve ≥90% overall kill rate

## References

- **Tool**: cargo-mutants v25.3.1
- **Configuration**: .cargo/mutants.toml
- **Specification**: docs/specifications/wasm-quality-testing-spec.md (Section 6)
- **wasm-labs target**: 99.4% mutation kill rate
- **Sprint 7 target**: ≥90% mutation kill rate

## Risk Assessment

**Risk Level**: LOW

**Mitigation**:
- ✅ Infrastructure tested and working
- ✅ Comprehensive test suite already in place (3405+ tests)
- ✅ Property tests cover invariants (200K cases)
- ✅ E2E tests validate integration (39 scenarios)
- ✅ Iterative approach allows incremental progress
- ✅ Clear methodology for improvement

**Expected Outcome**: High initial kill rate (likely 85%+) with targeted improvements to reach ≥90%

---

**Status**: ✅ Infrastructure Complete - Ready for Execution
**Configuration**: .cargo/mutants.toml
**Next**: Run mutation tests on core modules
**Target**: ≥90% mutation kill rate
**Timeline**: 1-2 sessions for completion
