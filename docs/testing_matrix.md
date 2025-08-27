# Ruchy Language Feature Testing Matrix

## Testing Strategy Overview

To prevent language feature regressions, we employ a multi-layered testing approach:

### Layer 1: Unit Tests (Fast, ~1-2ms each)
- Test individual language constructs
- Parser correctness
- AST generation
- Basic interpreter functionality

### Layer 2: Integration Tests (Medium, ~10-50ms each)  
- REPL vs File execution consistency
- Cross-feature interactions
- Error handling consistency

### Layer 3: System Tests (Slow, ~100-500ms each)
- End-to-end workflows
- Performance regression detection
- Golden master comparisons

### Layer 4: Property Tests (Mathematical, ~1-10s total)
- Language invariants that must NEVER break
- Mathematical properties (associativity, identity, etc.)
- Behavioral consistency across inputs

## Feature Testing Matrix

| Feature | Unit | Integration | System | Property | Golden | Regression |
|---------|------|-------------|---------|----------|---------|------------|
| **Basic Math** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Variables** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Functions** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **While Loops** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **For Loops** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Objects** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **obj.items()** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Tuple Destructuring** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Arrays** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Strings** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Pattern Matching** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **String Interpolation** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Closures** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

## Testing Automation

### Pre-commit Hooks (MANDATORY)
```bash
#!/bin/bash
# Run all test layers in order of speed

echo "🔹 Running unit tests..."
cargo test --lib --quiet || exit 1

echo "🔹 Running integration tests..."  
cargo test --test golden_master_suite --quiet || exit 1
cargo test --test differential_repl_file --quiet || exit 1

echo "🔹 Running regression tests..."
cargo test --test regression_database --quiet || exit 1

echo "🔹 Running language invariants..." 
cargo test --test language_invariants --quiet || exit 1

echo "✅ All language feature tests passed"
```

### CI/CD Pipeline  
```yaml
test_matrix:
  strategy:
    matrix:
      test_type: [unit, integration, system, property, golden, regression]
      rust_version: [stable, nightly]
  runs-on: ubuntu-latest
  steps:
    - name: Run test type
      run: cargo test --test ${{ matrix.test_type }}
```

### Nightly Comprehensive Testing
```bash
#!/bin/bash
# Run every night to catch edge cases

# Full language compatibility
cargo test compatibility_report --ignored

# Performance regression detection  
cargo test --test performance_baseline

# Fuzz testing (if available)
cargo fuzz list | xargs -I {} timeout 300 cargo fuzz run {}

# Memory usage testing
valgrind --tool=memcheck cargo test

# Coverage reporting
make coverage
```

## Test Quality Metrics

### Coverage Requirements
- **Baseline**: 37.71% (current)
- **Target**: 80% line coverage
- **Requirement**: 100% coverage for language features

### Performance Requirements
- Unit tests: <5ms each
- Integration tests: <50ms each  
- Full suite: <5 minutes
- Pre-commit: <10 seconds

### Reliability Requirements
- Zero flaky tests allowed
- 100% deterministic results
- Cross-platform consistency

## Regression Prevention Process

### When Adding New Features
1. Write failing tests first (TDD)
2. Implement minimal fix
3. Add golden master test
4. Add property tests
5. Update testing matrix
6. Document in regression database

### When Bugs are Found  
1. **HALT** - Stop all development
2. Write regression test that reproduces bug
3. Confirm test fails with current code
4. Fix the bug
5. Confirm test passes
6. Add to permanent regression suite
7. Update documentation

## Language Invariants (NEVER BREAK)

### Mathematical Properties
- Arithmetic associativity: `(a + b) + c == a + (b + c)`
- Arithmetic identity: `x + 0 == x`, `x * 1 == x`
- String concatenation identity: `s + "" == s`
- Boolean logic: `a && true == a`, `a || false == a`

### Behavioral Properties  
- Function determinism: Same inputs → Same outputs
- While loop termination: Finite iterations for finite conditions
- Object consistency: `obj.keys().len() == obj.values().len()`
- Iterator consistency: `for` loops consume exactly all elements

### System Properties
- REPL == File execution for same code
- Error messages consistent across contexts  
- Memory usage bounded
- Performance within acceptable limits

## Tools and Infrastructure

### Required Tools
- `cargo-llvm-cov` - Coverage analysis
- `cargo-fuzz` - Fuzz testing (optional)
- `criterion` - Performance benchmarking
- `proptest` - Property-based testing
- `valgrind` - Memory testing (Linux)

### Test Data Management
- Golden master outputs in `tests/golden/`
- Regression test cases in `tests/regressions/`
- Performance baselines in `tests/benchmarks/`
- Property test seeds documented

## Success Metrics

### Quality Gates (BLOCKING)
- ✅ All unit tests pass
- ✅ All integration tests pass  
- ✅ All golden masters match
- ✅ All regressions prevented
- ✅ All properties hold
- ✅ Performance within bounds
- ✅ Coverage above baseline

### Continuous Improvement
- Monthly review of test effectiveness
- Quarterly update of performance baselines
- Annual review of testing strategy
- Post-incident addition of regression tests

**GOAL: Make it impossible for language features to break silently**