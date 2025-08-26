# Ruchy - Self-Hosting Programming Language with Toyota Way Quality 🚀

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-374%20passing-green.svg)](./tests)
[![Coverage](https://img.shields.io/badge/coverage-87.80%25-brightgreen.svg)](./scripts/cli_coverage.sh)
[![v1.20.0](https://img.shields.io/badge/v1.20.0-QUALITY--TOOLS--COMPLETE-gold.svg)](./CHANGELOG.md)

**Ruchy is a self-hosting programming language** with comprehensive tooling (29 CLI commands), Toyota Way quality engineering, and mathematical property verification that makes regressions impossible.

## 🎯 Quick Start

```bash
# Install from crates.io
cargo install ruchy

# Run a one-liner
ruchy -e "println('Hello, World!')"

# Start the REPL
ruchy repl

# Run a script
ruchy run script.ruchy

# Check syntax
ruchy check script.ruchy

# Format code
ruchy fmt script.ruchy

# Lint code
ruchy lint script.ruchy

# Quality Tools Suite (NEW in v1.20.0!)
ruchy test tests/ --coverage              # Native test runner
ruchy lint src/ --fix                     # Code quality analysis  
ruchy prove assertions.ruchy --check      # Mathematical proof verification
ruchy score . --min=0.8                   # Unified quality scoring

# Advanced tooling
ruchy ast script.ruchy --json
ruchy runtime script.ruchy --profile
```

## ✨ Key Features

### 🏆 Complete Quality Tools Suite (v1.20.0)

**Professional-grade quality tools for enterprise development:**

#### [`ruchy test`](docs/commands/ruchy-test.md) - Native Test Runner
```bash
ruchy test tests/ --coverage --parallel --format=json
```
- **Native .ruchy file execution** with comprehensive test discovery
- **Parallel test execution** with timing and performance metrics
- **Coverage reporting** (text, HTML, JSON) with threshold enforcement
- **CI/CD integration** with proper exit codes and structured output
- **Watch mode** for continuous testing during development

#### [`ruchy lint`](docs/commands/ruchy-lint.md) - Code Quality Analysis  
```bash
ruchy lint src/ --fix --strict --format=json
```
- **Static analysis** detecting unused code, style violations, complexity issues
- **Auto-fix functionality** for formatting and simple refactoring
- **Security analysis** for hardcoded secrets, SQL injection, unsafe patterns
- **Performance analysis** for inefficient algorithms and memory usage
- **Configurable rules** with team-wide consistency enforcement

#### [`ruchy prove`](docs/commands/ruchy-prove.md) - Mathematical Proof Verification
```bash 
ruchy prove assertions.ruchy --check --counterexample --backend=z3
```
- **Formal verification** of mathematical properties using SMT solvers
- **Assertion extraction** from source code with automatic proof generation  
- **Counterexample generation** for false statements with concrete values
- **SMT solver integration** (Z3, CVC5, Yices2) for different proof strategies
- **Interactive proof mode** with tactics and goal management

#### [`ruchy score`](docs/commands/ruchy-score.md) - Unified Quality Scoring
```bash
ruchy score . --deep --baseline=main --min=0.8
```  
- **Comprehensive quality assessment** across 6 dimensions (style, complexity, security, performance, docs, coverage)
- **A+ to F grading scale** with detailed component breakdowns and improvement suggestions
- **Baseline comparison** for tracking quality improvements over time
- **Multiple analysis depths** from fast (<100ms) to deep (<30s) analysis
- **Team quality metrics** with configurable weights and thresholds

### 🎊 Self-Hosting Capability
- **Bootstrap Compiler**: Ruchy compiler written entirely in Ruchy
- **Direct Codegen**: Transpiles to Rust with `--minimal` flag
- **Type Inference**: Advanced Algorithm W with constraint solving
- **Complete Language**: All constructs needed for compiler development

### 🛠️ Professional CLI Tooling (29 Commands)
| Command | Purpose | Status |
|---------|---------|---------|
| `ruchy check` | Syntax validation | ✅ Production |
| `ruchy fmt` | Code formatting | ✅ Production |
| `ruchy lint` | Quality analysis | ✅ Production |
| `ruchy test` | Test execution | ✅ Production |
| `ruchy ast` | AST visualization | ✅ Production |
| `ruchy score` | Quality scoring | ✅ Production |
| `ruchy provability` | Formal verification | ✅ Production |
| `ruchy runtime` | Performance analysis | ✅ Production |

### 🚀 Language Excellence
- **Pipeline Operator**: `data |> transform |> filter` functional style
- **Pattern Matching**: Complete with guards: `x if x > 0 => "positive"`
- **Both Lambda Syntaxes**: `|x| x + 1` and `x => x + 1` supported
- **Module System**: `use`, `mod`, and `::` path resolution
- **Error Handling**: Result/Option types with `?` operator
- **HashMap/HashSet**: Complete collections with all methods
- **String/Array Methods**: 20+ methods each for comprehensive manipulation

### 📊 Toyota Way Quality Engineering
- **87.80% Test Coverage**: Mathematical verification of correctness
- **374 Tests Passing**: Unit, integration, CLI, property, and fuzz tests
- **Zero-Warning Build**: Complete clippy compliance (`-D warnings`)
- **Mathematical Properties**: Idempotency, determinism formally verified
- **Automated Quality Gates**: Pre-commit hooks prevent regressions

### 💻 Interactive REPL
- **Resource-bounded**: Memory limits, timeouts, stack protection
- **Syntax highlighting**: Real-time colored output  
- **Tab completion**: Context-aware completions
- **Persistent history**: Cross-session command storage
- **Multiline editing**: Automatic continuation detection

## Example Code

```rust
// Self-hosting compiler capabilities
fun parse_expr(tokens: Vec<Token>) -> Result<Expr, ParseError> {
    match tokens.first() {
        Some(Token::Number(n)) => Ok(Expr::Literal(*n)),
        Some(Token::Ident(name)) => Ok(Expr::Variable(name.clone())),
        _ => Err(ParseError::UnexpectedToken)
    }
}

// Functional programming with pipeline operator  
[1, 2, 3, 4, 5]
  |> numbers.map(|x| x * 2)
  |> numbers.filter(|x| x > 5)
  |> numbers.sum()

// Pattern matching with guards
match user_input {
    n if n > 0 => "positive",
    0 => "zero",
    1..=10 => "small range",
    _ => "other"
}

// HashMap collections
let mut map = HashMap()
map.insert("key", "value")
map.get("key").unwrap()
```

## 🧪 Testing Excellence & Quality Gates

**Toyota Way "Stop the Line" Quality: Zero regressions possible through mathematical verification.**

```bash
# Complete CLI test suite (733ms execution time)
make test-ruchy-commands

# Coverage analysis (87.80% line coverage achieved)
make test-cli-coverage  

# Performance benchmarking with hyperfine
make test-cli-performance
```

**Testing Arsenal:**
- ✅ **13 Total Tests**: 8 integration + 5 property tests
- ✅ **Mathematical Properties**: Idempotency, determinism, preservation verified
- ✅ **Fuzz Testing**: Random input robustness with libfuzzer
- ✅ **Quality Gates**: Pre-commit hooks enforce ≥80% coverage

| Test Category | Count | Execution Time | Coverage Impact |
|---------------|-------|----------------|-----------------|
| Integration Tests | 8 | 176ms | End-to-end validation |
| Property Tests | 5 | 193ms | Mathematical invariants |
| Executable Examples | 4 | <100ms | Documentation verification |
| Fuzz Targets | 2 | Variable | Edge case discovery |

See [CLI Testing Guide](./docs/testing/cli-testing-guide.md) for comprehensive methodology.

## 🚀 Ecosystem Integration

**Complete quality pipeline for production-ready development:**

### Full Development Workflow
```bash
# Development cycle with quality gates
ruchy test tests/ --watch &          # Continuous testing
ruchy lint src/ --fix               # Auto-fix style issues  
ruchy prove src/ --check            # Verify mathematical properties
ruchy score . --min=0.8             # Ensure quality threshold

# Pre-commit quality gate
ruchy score --baseline=main --min-improvement=0.00 .
```

### CI/CD Integration
```yaml
# .github/workflows/quality.yml
- name: Quality Assessment
  run: |
    ruchy test . --coverage --threshold=80 --format=json
    ruchy lint . --strict --format=json
    ruchy prove . --check --timeout=10000 --format=json  
    ruchy score . --deep --min=0.75 --baseline=origin/main
```

### Sister Project Support
- **[ruchyruchy](../ruchyruchy)**: 390,000+ validation tests now **UNBLOCKED** and executable
- **[ruchy-repl-demos](../ruchy-repl-demos)**: Gold standard TDD workflow with all quality tools
- **[ruchy-book](../ruchy-book)**: All 411 examples formally verifiable with `ruchy prove`

### Team Quality Standards
```bash
# Establish team baseline
ruchy score --baseline=main --config=.ruchy-quality.toml .

# Quality metrics tracking
ruchy score --format=json . > quality-report-$(date +%Y%m%d).json

# Automated quality enforcement  
ruchy score --min=0.80 --deny-warnings .
```

## Installation

```bash
cargo install ruchy
```

## License

MIT OR Apache-2.0
