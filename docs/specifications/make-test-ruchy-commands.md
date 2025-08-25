# Specification: Comprehensive CLI Command Testing Infrastructure

**Document ID**: SPEC-CLI-TEST-001
**Version**: 1.0.0
**Date**: 2025-08-25
**Status**: DRAFT
**Priority**: CRITICAL - Toyota Way Quality Gate

## Executive Summary

The recent discovery of a broken `ruchy fmt` command that outputs debug AST instead of formatted code represents a critical quality failure. This specification mandates comprehensive testing infrastructure to prevent such regressions and ensure 100% reliability of all CLI commands.

## Toyota Way Principle

**STOP THE LINE**: No CLI command regression is acceptable. Every command must have:
- 80% minimum code coverage
- Executable examples (`cargo run` demonstrations)  
- Property-based tests (mathematical invariants)
- Fuzz testing (random input robustness)
- Integration tests (real file scenarios)

## Problem Statement

### Current Issues
1. **fmt command broken** - Outputs debug AST instead of formatted code
2. **Insufficient coverage** - CLI commands lack comprehensive testing
3. **No executable examples** - Users cannot verify functionality
4. **Missing property tests** - No mathematical validation
5. **No fuzz testing** - Vulnerable to edge case failures

### Root Cause Analysis (5 Whys)
1. **Why did fmt fail?** - Formatter fallback outputs debug instead of formatted code
2. **Why wasn't this caught?** - CLI tests only check for basic string presence
3. **Why are tests insufficient?** - No coverage requirements or example validation
4. **Why no coverage requirements?** - No systematic testing specification
5. **Why no specification?** - Prioritized features over quality infrastructure

## Requirements

### Functional Requirements

#### FR-001: 80% Code Coverage Mandate
- **Requirement**: All CLI commands must achieve minimum 80% code coverage
- **Measurement**: `cargo llvm-cov` with per-command breakdown
- **Enforcement**: Pre-commit hooks block commits below threshold

#### FR-002: Executable Examples
- **Requirement**: Each command must have `cargo run` demonstration
- **Format**: `examples/cli/{command}.rs` with real scenarios
- **Validation**: Examples must compile and execute successfully

#### FR-003: Property-Based Testing
- **Requirement**: Mathematical invariants for each command
- **Examples**: 
  - `fmt(fmt(x)) == fmt(x)` (idempotent)
  - `check(valid_syntax) == true` (correctness)
  - `lint(auto_fix(x)).issues.len() <= lint(x).issues.len()` (improvement)

#### FR-004: Fuzz Testing
- **Requirement**: Random input robustness for all commands
- **Tool**: `cargo-fuzz` with structured input generation
- **Duration**: Minimum 60 seconds per command in CI

#### FR-005: Integration Testing
- **Requirement**: Real file scenarios with temporary directories
- **Coverage**: Happy path, error cases, edge conditions
- **Validation**: File system effects, output correctness, exit codes

### Non-Functional Requirements

#### NFR-001: Performance
- **Requirement**: All tests complete within 120 seconds total
- **Measurement**: `hyperfine` benchmarking in CI
- **Optimization**: Parallel execution where possible

#### NFR-002: Maintainability
- **Requirement**: Single command to run all CLI tests
- **Implementation**: `make test-ruchy-commands`
- **Documentation**: Clear failure diagnostics with fix suggestions

#### NFR-003: CI/CD Integration
- **Requirement**: Automated execution in GitHub Actions
- **Reporting**: Coverage reports, test summaries, performance metrics
- **Blocking**: Failed tests prevent merges

## Architecture

### Testing Framework Structure

```
tests/
├── cli_integration/           # Integration tests per command
│   ├── test_fmt.rs           # Comprehensive fmt testing
│   ├── test_check.rs         # Syntax validation testing
│   ├── test_lint.rs          # Linting and auto-fix testing
│   └── ...                   # One file per command
├── cli_properties/           # Property-based tests
│   ├── fmt_properties.rs     # Mathematical invariants for fmt
│   ├── check_properties.rs   # Correctness properties for check
│   └── ...                   # Property tests per command
└── cli_fuzz/                 # Fuzz testing targets
    ├── fuzz_fmt.rs          # Random input testing for fmt
    ├── fuzz_check.rs        # Fuzz testing for check
    └── ...                  # Fuzz targets per command

examples/
└── cli/                     # Executable examples
    ├── fmt_example.rs       # Demonstrates fmt functionality
    ├── check_example.rs     # Shows check command usage
    └── ...                  # Example per command
```

### Makefile Targets

```makefile
# Primary target
test-ruchy-commands: test-cli-integration test-cli-properties test-cli-fuzz test-cli-examples

# Coverage reporting
test-cli-coverage:
	cargo llvm-cov --html --open --ignore-filename-regex="target/*"
	
# Performance benchmarking  
test-cli-performance:
	hyperfine --warmup 3 --runs 10 'make test-ruchy-commands'
```

## Implementation Plan

### Sprint: CLI Testing Infrastructure Revolution

**Duration**: 5 days
**Goal**: 100% CLI command reliability with comprehensive testing

#### Task Breakdown

##### Day 1: Foundation Infrastructure
1. **CLI-001**: Create `docs/specifications/make-test-ruchy-commands.md` ✓
2. **CLI-002**: Update roadmap with CLI testing sprint priority
3. **CLI-003**: Create Makefile targets for `test-ruchy-commands`
4. **CLI-004**: Setup directory structure for comprehensive testing

##### Day 2: Fix Current Failures
5. **CLI-005**: Debug and fix `ruchy fmt` command output
6. **CLI-006**: Create comprehensive integration test for `fmt`
7. **CLI-007**: Add property test for `fmt` idempotency
8. **CLI-008**: Create executable example for `fmt`

##### Day 3: Coverage Infrastructure
9. **CLI-009**: Implement per-command coverage measurement
10. **CLI-010**: Add coverage enforcement to pre-commit hooks
11. **CLI-011**: Create coverage reporting dashboard
12. **CLI-012**: Validate 80% coverage for existing commands

##### Day 4: Property & Fuzz Testing
13. **CLI-013**: Setup `cargo-fuzz` infrastructure
14. **CLI-014**: Create property tests for 5 core commands
15. **CLI-015**: Implement fuzz targets for input validation
16. **CLI-016**: Add property test validation to CI

##### Day 5: Integration & Release
17. **CLI-017**: Comprehensive integration testing
18. **CLI-018**: Performance benchmarking and optimization
19. **CLI-019**: Documentation and user guides
20. **CLI-020**: Release v1.15.0 with CLI Testing Excellence

### Success Criteria

#### Quantitative Metrics
- **Code Coverage**: ≥80% for all CLI commands
- **Test Execution Time**: ≤120 seconds total
- **Property Test Coverage**: ≥5 mathematical invariants per command
- **Fuzz Test Duration**: ≥60 seconds per command
- **Integration Test Scenarios**: ≥3 per command (happy, error, edge)

#### Qualitative Metrics
- **Zero Regressions**: All existing functionality preserved
- **Clear Documentation**: Every test failure provides fix guidance
- **Toyota Way Compliance**: Quality built into process, not bolted on
- **Developer Experience**: Single command runs comprehensive validation

## Risk Assessment

### High Risks
1. **Time Overrun**: Comprehensive testing may exceed sprint duration
   - **Mitigation**: Prioritize critical commands first
2. **Performance Impact**: Extensive testing may slow CI/CD
   - **Mitigation**: Parallel execution and caching strategies
3. **Maintenance Burden**: Complex testing infrastructure requires upkeep
   - **Mitigation**: Clear documentation and automation

### Medium Risks
1. **False Positives**: Overly strict tests may block valid changes
   - **Mitigation**: Balanced assertion levels and clear test intent
2. **Coverage Gaming**: Achieving 80% without meaningful validation
   - **Mitigation**: Code review focus on test quality, not just quantity

## Acceptance Criteria

### Must Have
- [ ] `make test-ruchy-commands` executes successfully
- [ ] All 29 CLI commands achieve ≥80% coverage
- [ ] Zero CLI command regressions in existing functionality
- [ ] Property tests validate mathematical invariants
- [ ] Fuzz tests complete without crashes

### Should Have
- [ ] Executable examples for all commands
- [ ] Performance benchmarking in CI
- [ ] Coverage reporting dashboard
- [ ] Integration with GitHub Actions

### Could Have
- [ ] Visual test result reporting
- [ ] Automated performance regression detection
- [ ] CLI command usage analytics

## Conclusion

This specification establishes the foundation for bulletproof CLI command reliability through comprehensive testing. The Toyota Way principle of "Stop the Line" for any defect ensures that no CLI regression like the fmt bug can occur again.

**Next Steps**: Update roadmap, begin implementation with CLI-002 task.

---

**Approval Required**: Technical Lead, Quality Assurance
**Implementation Start**: Immediate (Critical Priority)