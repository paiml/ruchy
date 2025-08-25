# Test Coverage Guide for Ruchy

Toyota Way: "Build quality into the process, not inspect it afterward."

## Quick Start

### Get Coverage Information

```bash
# Quick coverage check (fast, no HTML)
make coverage-quick

# Comprehensive coverage report with HTML
make coverage

# Generate and open coverage report in browser
make coverage-open

# Legacy coverage (CI compatible)
make coverage-legacy
```

### Coverage Targets

| Target | Description | When to Use |
|--------|-------------|-------------|
| `coverage-quick` | Fast summary, no HTML | Development workflow |
| `coverage` | Full HTML + analysis | Weekly reviews |
| `coverage-open` | Generate and open in browser | Investigating specific issues |

## Current Status

**Overall Coverage: ~36%** (as of latest analysis)

### High Coverage Modules (>80%)
- `lib.rs`: 98.40% - Excellent, maintain this level
- `frontend/arena.rs`: 84.80% - Very good  
- `frontend/ast.rs`: 86.38% - Very good
- `runtime/repl_function_tests.rs`: 98.19% - Excellent

### Medium Coverage Modules (50-80%)
- `backend/transpiler/mod.rs`: 56.55% - Needs improvement
- `runtime/interpreter.rs`: 74.28% - Good, but critical module
- `frontend/parser/functions.rs`: 75.37% - Good

### Low Coverage Modules (<50%)
- `backend/transpiler/dataframe.rs`: 0.00% - **Critical gap**
- `lsp/`: Most modules at 0.00% - **Feature not tested**
- `optimization/`: All modules at 0.00% - **Future feature**
- `proving/`: Most modules <30% - **Advanced feature**

## Toyota Way Analysis

### Quality Gates

**Minimum Coverage**: 80% (current: ~36% ❌)
**Target Coverage**: 90% (aspirational)

### Improvement Plan

#### Phase 1: Critical Infrastructure (Target: 60%)
1. **Backend Transpiler**: Core functionality, needs 70%+ coverage
2. **Runtime Interpreter**: Critical path, target 85%+ 
3. **Frontend Parser**: Core parsing logic, target 80%+

#### Phase 2: Feature Completeness (Target: 75%)
1. **LSP Module**: Enable all language server features
2. **MCP Integration**: Communication protocol testing
3. **Type Inference**: Middle-end coverage improvement

#### Phase 3: Advanced Features (Target: 85%)
1. **Dataframe Operations**: Data science functionality
2. **Actor System**: Concurrent execution testing
3. **Optimization Pipeline**: Performance feature testing

## Coverage Analysis Tools

### HTML Reports
```bash
make coverage
open target/llvm-cov/html/index.html
```

The HTML report provides:
- **Line-by-line coverage** highlighting
- **Branch coverage** analysis
- **Function coverage** statistics
- **Module-by-module** breakdown

### LCOV Integration
```bash
# Generate LCOV format for external tools
make coverage
# Output: target/llvm-cov/lcov.info
```

### JSON Analysis
```bash
# For programmatic analysis
cargo llvm-cov --json --output-path coverage.json
```

## Writing Tests for Coverage

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_function() {
        // Test specific functionality
        assert_eq!(my_function(input), expected_output);
    }
}
```

### Property Tests
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn property_always_holds(input in any::<i32>()) {
        // Property that should always be true
        prop_assert!(my_function(input) >= 0);
    }
}
```

### Integration Tests
```rust
// tests/integration_test.rs
use ruchy::*;

#[test] 
fn integration_test_example() {
    // End-to-end functionality testing
    let result = full_workflow_test();
    assert!(result.is_ok());
}
```

## Coverage Configuration

### Excluded Files
We exclude test files, benchmarks, and examples from coverage:
```bash
--ignore-filename-regex "tests/|benches/|examples/"
```

### Coverage Aliases
Defined in `.cargo/config.toml`:
```toml
[alias]
cov = "llvm-cov --html"
cov-report = "llvm-cov report"
cov-json = "llvm-cov --json"
cov-lcov = "llvm-cov --lcov --output-path lcov.info"
cov-open = "llvm-cov --html --open"
```

## Continuous Improvement

### Weekly Coverage Review
1. Run `make coverage-open`
2. Identify modules below target coverage
3. Priority: Core functionality > Features > Advanced features
4. Create focused test additions

### Coverage-Driven Development
1. **Red**: Write test that fails (shows uncovered code)
2. **Green**: Implement minimal code to pass test
3. **Refactor**: Clean up while maintaining coverage
4. **Verify**: Ensure coverage increased

### Toyota Way Recommendations

#### Below 60% Coverage ❌
- **CRITICAL**: Add basic unit tests immediately
- Focus on: Error paths, edge cases, main workflows
- Create integration tests for user journeys

#### 60-80% Coverage ⚠️
- **GOOD**: Add property tests for business logic
- Focus on: Branch coverage, complex conditions
- Add negative test cases

#### Above 80% Coverage ✅
- **EXCELLENT**: Maintain through test-first development
- Focus on: Mutation testing, performance tests
- Document testing strategies

## Troubleshooting

### Coverage Too Low
```bash
# Find uncovered lines
make coverage
# Open HTML report, look for red highlighted lines
# Add tests specifically targeting those lines
```

### Coverage Flaky
```bash
# Clean coverage data
cargo llvm-cov clean --workspace

# Run with deterministic settings
RUST_TEST_THREADS=1 make coverage
```

### Performance Issues
```bash
# Use quick coverage for development
make coverage-quick

# Full coverage only for weekly reviews
make coverage
```

## Integration with CI/CD

### GitHub Actions Example
```yaml
- name: Generate Coverage
  run: make coverage
  
- name: Upload Coverage to Codecov
  uses: codecov/codecov-action@v3
  with:
    files: target/llvm-cov/lcov.info
    fail_ci_if_error: true
```

### Quality Gate Integration
```bash
# Exit code 1 if below minimum coverage
make coverage  # Will fail if <80%
```

---

**Remember**: Coverage is a tool for quality, not a goal. Focus on meaningful tests that catch real bugs and verify correct behavior. 100% coverage with poor tests is worse than 80% coverage with excellent tests.