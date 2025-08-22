# Deno-Style Binary Tooling Development Story

## Executive Summary

This specification defines first-class development tooling for Ruchy, modeled after Deno's excellent developer experience. These tools will enable the ubuntu-config-scripts team to complete their 45-script TypeScript→Ruchy migration with professional-grade development infrastructure.

## Strategic Context

### Current State (From Reports)
- **v0.9.6**: 65% production ready, pattern matching + arrays working
- **Basic tooling**: `ruchy check`, `ruchy run` functional but limited
- **Migration blocker**: Missing comprehensive development tools for professional workflows

### Target State
- **Complete Deno-equivalent tooling** for seamless TypeScript developer migration
- **Zero-friction development experience** matching modern JavaScript/TypeScript tooling
- **Production-ready quality gates** for large-scale script migration projects

## Tool Specifications

### Phase 1: Core Development Tools (Immediate Priority)

#### 1. `ruchy test` - Test Runner with Coverage
**Current State**: Basic test execution, no coverage reporting  
**Target**: Full-featured test framework with coverage analysis

```bash
# Test execution
ruchy test                          # Run all tests
ruchy test specific_test.ruchy      # Run specific test file
ruchy test --watch                  # Watch mode for development
ruchy test --parallel              # Parallel test execution

# Coverage reporting
ruchy test --coverage              # Generate coverage report
ruchy test --coverage --html       # HTML coverage report
ruchy test --coverage --json       # JSON coverage for CI/CD
ruchy test --coverage --threshold=80  # Fail if coverage below threshold
```

**Implementation Requirements**:
- **Test discovery**: Automatic detection of test files (`*_test.ruchy`, `test_*.ruchy`)
- **Test annotations**: Support for `#[test]` attribute syntax
- **Assertion framework**: Built-in `assert_eq`, `assert_ne`, `assert` functions
- **Coverage analysis**: Line coverage, branch coverage, function coverage
- **Output formats**: Console, HTML, JSON, JUnit XML
- **Watch mode**: File system monitoring with automatic re-runs
- **Parallel execution**: Multi-threaded test running for performance

#### 2. `ruchy lint` - Grammar-Based Code Analysis
**Current State**: Basic linting, permissive rules  
**Target**: Comprehensive code quality analysis using Ruchy grammar

```bash
# Basic linting
ruchy lint                         # Lint all .ruchy files
ruchy lint script.ruchy           # Lint specific file
ruchy lint --fix                  # Auto-fix issues where possible

# Advanced analysis
ruchy lint --strict               # Strict mode with all rules
ruchy lint --config=lint.toml     # Custom linting configuration
ruchy lint --format=json          # JSON output for tooling integration
ruchy lint --rules=unused,style   # Specific rule categories
```

**Linting Rules Implementation**:
- **Grammar violations**: Syntax errors, deprecated constructs
- **Style consistency**: Naming conventions, formatting standards
- **Code quality**: Unused variables, dead code, complexity metrics
- **Performance hints**: Inefficient patterns, optimization suggestions
- **Security analysis**: Potential vulnerabilities, unsafe patterns
- **Best practices**: Idiomatic Ruchy patterns, anti-patterns detection

#### 3. `ruchy fmt` - Code Formatter
**Current State**: Outputs AST instead of formatted code  
**Target**: Production-ready code formatting with configurable styles

```bash
# Basic formatting
ruchy fmt                         # Format all .ruchy files in place
ruchy fmt script.ruchy           # Format specific file
ruchy fmt --check                # Check if files are formatted (CI mode)

# Configuration
ruchy fmt --config=fmt.toml      # Custom formatting rules
ruchy fmt --line-width=100       # Configure line width
ruchy fmt --indent=4             # Configure indentation
```

**Formatting Features**:
- **Consistent styling**: Automatic indentation, spacing, line breaks
- **Configurable rules**: Line width, tab vs spaces, bracket styles
- **Preserve semantics**: No behavior changes, only style improvements
- **Fast processing**: <10ms formatting for typical files
- **Editor integration**: Language server protocol support

#### 4. `ruchy ast` - AST Analysis and Inspection
**Current State**: Basic AST generation  
**Target**: Comprehensive AST tooling for development and debugging

```bash
# AST generation
ruchy ast script.ruchy           # Pretty-printed AST
ruchy ast --json script.ruchy    # JSON AST for tooling
ruchy ast --graph script.ruchy   # Visual AST graph (DOT format)

# Analysis
ruchy ast --metrics script.ruchy # Complexity metrics
ruchy ast --symbols script.ruchy # Symbol table analysis
ruchy ast --deps script.ruchy    # Dependency analysis
```

**AST Analysis Features**:
- **Pretty printing**: Human-readable AST representation
- **JSON export**: Machine-readable AST for tooling integration
- **Visual graphs**: DOT/Graphviz output for AST visualization
- **Metrics calculation**: Cyclomatic complexity, depth analysis
- **Symbol analysis**: Variable usage, scope analysis, dead code detection
- **Dependency tracking**: Module dependencies, import analysis

#### 5. `ruchy provability` - Formal Verification Support
**Innovation**: Advanced static analysis for correctness guarantees

```bash
# Correctness analysis
ruchy provability script.ruchy           # Basic provability analysis
ruchy provability --verify script.ruchy  # Full formal verification
ruchy provability --contracts script.ruchy # Contract verification

# Advanced analysis
ruchy provability --invariants script.ruchy # Loop invariant checking
ruchy provability --termination script.ruchy # Termination analysis
ruchy provability --bounds script.ruchy     # Array bounds checking
```

**Provability Features**:
- **Contract verification**: Pre/post-conditions, invariants
- **Termination analysis**: Loop and recursion termination proofs
- **Memory safety**: Bounds checking, null pointer analysis
- **Logic verification**: Boolean satisfiability, theorem proving
- **Correctness guarantees**: Mathematical proofs of program properties

#### 6. `ruchy runtime` - Performance Analysis (BigO)
**Innovation**: Algorithmic complexity analysis and performance profiling

```bash
# Performance analysis
ruchy runtime script.ruchy              # Basic performance metrics
ruchy runtime --profile script.ruchy    # Detailed profiling
ruchy runtime --bigo script.ruchy       # Algorithmic complexity analysis

# Benchmarking
ruchy runtime --bench script.ruchy      # Benchmark execution
ruchy runtime --compare v1.ruchy v2.ruchy # Performance comparison
ruchy runtime --memory script.ruchy     # Memory usage analysis
```

**Runtime Analysis Features**:
- **Execution profiling**: Function-level timing, call graphs
- **Memory analysis**: Allocation patterns, memory leaks, usage optimization
- **Complexity analysis**: Automatic BigO detection for algorithms
- **Performance regression**: Automated performance testing
- **Benchmarking**: Statistical performance measurement
- **Optimization hints**: Suggestions for performance improvements

## Phase 2: Advanced Testing Tools (Post Self-Hosting)

#### 7. `ruchy property-tests` - Property-Based Testing
```bash
ruchy property-tests script.ruchy       # Run property tests
ruchy property-tests --iterations=1000  # Custom iteration count
ruchy property-tests --shrink           # Minimal failing case generation
```

#### 8. `ruchy doctests` - Documentation Testing
```bash
ruchy doctests                          # Run all documentation tests
ruchy doctests --update                 # Update expected outputs
ruchy doctests README.md               # Test specific documentation
```

#### 9. `ruchy fuzz-tests` - Fuzzing Framework
```bash
ruchy fuzz-tests script.ruchy           # Fuzz testing
ruchy fuzz-tests --corpus=data/         # Custom fuzzing corpus
ruchy fuzz-tests --timeout=60s          # Fuzzing timeout
```

## Implementation Architecture

### Tool Integration Strategy
```rust
// Unified CLI architecture
pub enum RuchyCommand {
    Test(TestOptions),
    Lint(LintOptions), 
    Fmt(FmtOptions),
    Ast(AstOptions),
    Provability(ProvabilityOptions),
    Runtime(RuntimeOptions),
}

// Shared infrastructure
pub struct RuchyToolchain {
    parser: Parser,
    type_checker: TypeChecker,
    analyzer: StaticAnalyzer,
    formatter: CodeFormatter,
    test_runner: TestRunner,
}
```

### Performance Targets
- **Test execution**: <50ms for typical test suite
- **Linting**: <20ms for 1000-line files
- **Formatting**: <10ms for typical files
- **AST generation**: <5ms for moderate complexity
- **Provability analysis**: <100ms for basic verification
- **Runtime analysis**: <200ms for complexity analysis

### Quality Standards
- **Memory usage**: <50MB for all tools combined
- **Error messages**: Elm-style helpful diagnostics
- **Exit codes**: Standard UNIX conventions
- **Output formats**: JSON, HTML, console-friendly
- **Configuration**: TOML-based configuration files

## Migration Enablement

### TypeScript Developer Experience Parity
```bash
# Deno commands → Ruchy equivalents
deno test              → ruchy test
deno lint              → ruchy lint  
deno fmt               → ruchy fmt
deno check             → ruchy check
deno run               → ruchy run

# Enhanced capabilities
deno test --coverage   → ruchy test --coverage --html
                      → ruchy provability --verify
                      → ruchy runtime --bigo
```

### CI/CD Integration
```yaml
# GitHub Actions example
- name: Test with coverage
  run: ruchy test --coverage --threshold=80

- name: Lint code quality
  run: ruchy lint --strict --format=json

- name: Verify formatting
  run: ruchy fmt --check

- name: Performance regression
  run: ruchy runtime --bench --compare=baseline
```

### Development Workflow
```bash
# Recommended development cycle
ruchy fmt                    # Format code
ruchy lint --fix            # Fix linting issues
ruchy test --coverage       # Run tests with coverage
ruchy provability           # Verify correctness
ruchy runtime --profile     # Check performance
```

## Roadmap Integration

### Task Definitions
- **RUCHY-0750**: Implement `ruchy test` with coverage reporting
- **RUCHY-0751**: Enhance `ruchy lint` with grammar-based analysis
- **RUCHY-0752**: Complete `ruchy fmt` code formatting
- **RUCHY-0753**: Expand `ruchy ast` analysis capabilities
- **RUCHY-0754**: Implement `ruchy provability` formal verification
- **RUCHY-0755**: Create `ruchy runtime` performance analysis
- **RUCHY-0756**: Package and release enhanced binary tooling
- **RUCHY-0757**: Publish to crates.io with new capabilities

### Development Timeline
- **Week 1**: Core test framework and coverage reporting
- **Week 2**: Enhanced linting and code formatting
- **Week 3**: AST analysis and provability verification
- **Week 4**: Runtime analysis and performance tooling
- **Week 5**: Integration testing and binary packaging
- **Week 6**: Release and crates.io publication

### Success Criteria
- **100% feature parity** with Deno development experience
- **PAIML team validation** for 45-script migration readiness
- **Performance targets met** for all tool operations
- **Production deployment** of enhanced Ruchy binary
- **Community adoption** of new development workflow

## Innovation Differentiators

### Beyond Deno Capabilities
1. **Formal verification** (`ruchy provability`) - Mathematical correctness guarantees
2. **Algorithmic analysis** (`ruchy runtime --bigo`) - Automatic complexity detection
3. **Grammar-based linting** - Language-aware code analysis
4. **Unified toolchain** - Single binary for all development needs
5. **Performance-first design** - Sub-50ms tool execution

### Competitive Advantages
- **Faster than Deno**: 5-10x performance improvement for common operations
- **More comprehensive**: Formal verification + performance analysis
- **Better integration**: Unified AST and type system across all tools
- **Production-ready**: Designed for large-scale system programming projects

## Conclusion

This specification defines a comprehensive development tooling strategy that will:
1. **Enable immediate TypeScript→Ruchy migration** for the ubuntu-config-scripts team
2. **Establish Ruchy as a first-class system programming language** with professional tooling
3. **Differentiate from existing solutions** through formal verification and performance analysis
4. **Accelerate ecosystem adoption** by removing development experience barriers

The implementation of these tools represents a critical milestone for Ruchy's transition from experimental language to production-ready development platform.

---

**Priority**: P0 - CRITICAL for migration enablement  
**Dependencies**: Current parser and type checker infrastructure  
**Impact**: Enables 45-script migration and establishes professional development workflow  
**Timeline**: 6 weeks for complete implementation and release