# Ruchy - Self-Hosting Programming Language

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/ruchy.svg)](https://crates.io/crates/ruchy)

Ruchy is a self-hosting programming language with comprehensive tooling and quality engineering practices based on Toyota Way principles.

## Quick Start

```bash
# Install from crates.io
cargo install ruchy

# Start the REPL
ruchy repl

# Run a script
ruchy run script.ruchy

# Format code
ruchy fmt src/ --check
```

## CLI Commands (v3.0.1)

### WebAssembly Compilation
```bash
# Compile to WASM
ruchy wasm compile script.ruchy -o output.wasm

# Validate WASM module
ruchy wasm validate module.wasm
```

### Notebook Operations
```bash
# Start notebook server
ruchy notebook serve --port 8888

# Test notebook
ruchy notebook test notebook.ipynb --coverage

# Convert notebook
ruchy notebook convert input.ipynb output.html
```

### Testing Utilities
```bash
# Run tests with coverage
ruchy test run src/ --coverage --parallel

# Generate test report
ruchy test report --format junit
```

## Current Status - v3.0.1 (September 2025)

### 🎯 WASM Excellence: 100% Pass Rate
- **8 of 8** WASM acceptance tests passing
- **902** unit tests passing
- **108.9/100** TDG quality score (A+ grade)
- **Zero** technical debt (0 SATD violations)

### 🚀 Recent Achievements (v3.0.1)
- **WASM Runtime Fixed**: 100% acceptance test pass rate achieved
- **Property Testing**: 11 comprehensive property tests for WASM
- **Fuzz Testing**: 3 specialized fuzzers (comprehensive, security, stress)
- **Professional CLI**: Complete command-line interface with subcommands
- **Quality Excellence**: All functions under 10 complexity

### 📊 Quality Metrics
- **Code Quality**: 108.9/100 TDG score (A+ grade)
- **Test Coverage**: 902 tests passing
- **WASM Tests**: 8/8 acceptance tests (100%)
- **Property Tests**: 11/11 passing
- **Complexity**: All functions <10 cyclomatic
- **Test Coverage**: 49.90% overall, 81.2% transpiler
- **Zero Technical Debt**: No TODO/FIXME/HACK comments
- **Pre-commit Gates**: Automated quality enforcement

## Key Features

### Self-Hosting Capability
- Bootstrap compiler written in Ruchy
- Transpiles to Rust for compilation
- Type inference with Algorithm W
- Complete language features for compiler development

### Language Features
- **Pipeline Operator**: `data |> transform |> filter`
- **Pattern Matching**: With guards: `x if x > 0 => "positive"`
- **Lambda Syntax**: Both `|x| x + 1` and `x => x + 1`
- **Module System**: `use`, `mod`, and `::` path resolution
- **Error Handling**: Result/Option types with `?` operator
- **Collections**: HashMap, HashSet with standard methods
- **String/Array Methods**: Comprehensive built-in methods

### CLI Commands
| Command | Purpose | 
|---------|---------|
| `ruchy check` | Syntax validation |
| `ruchy fmt` | Code formatting |
| `ruchy lint` | Quality analysis |
| `ruchy test` | Test execution |
| `ruchy ast` | AST visualization |
| `ruchy run` | Script execution |
| `ruchy repl` | Interactive environment |
| `ruchy transpile` | Convert to Rust |
| `ruchy wasm` | **🚀 NEW**: Compile to WebAssembly |
| `ruchy notebook` | **📊 NEW**: Start interactive data science notebook |

### 🚀 WebAssembly Compilation
```bash
# Compile Ruchy to WebAssembly
ruchy wasm my_program.ruchy --output program.wasm --verbose

# Generate optimized WASM for different targets
ruchy wasm script.ruchy --target browser --optimize --validate
ruchy wasm api.ruchy --target nodejs --deploy aws-lambda
```

### 📊 Data Science Notebook
```bash
# Start interactive notebook server
ruchy notebook --port 8888

# Enable all data science features (default in v1.93.0+)
cargo install ruchy

# Minimal installation (just core language)
cargo install ruchy --no-default-features --features minimal
```

**Batteries-Included Features (Default)**:
- 📊 **DataFrames**: Polars integration for data manipulation
- 🚀 **WebAssembly**: Direct compilation to WASM modules  
- 📝 **Notebooks**: Interactive Jupyter-like environment
- 🧮 **Math Libraries**: Statistical operations and linear algebra

### REPL Features
- Tab completion with context awareness
- Syntax highlighting
- Persistent history across sessions
- Multiline editing
- Magic commands (`:help`, `:load`, `:save`)
- Resource limits (memory, timeout, stack depth)

## Example Code

```rust
// Function definition
fun parse_expr(tokens: Vec<Token>) -> Result<Expr, ParseError> {
    match tokens.first() {
        Some(Token::Number(n)) => Ok(Expr::Literal(*n)),
        Some(Token::Ident(name)) => Ok(Expr::Variable(name.clone())),
        _ => Err(ParseError::UnexpectedToken)
    }
}

// Pipeline operator
[1, 2, 3, 4, 5]
  |> map(|x| x * 2)
  |> filter(|x| x > 5)
  |> sum()

// Pattern matching with guards
match user_input {
    n if n > 0 => "positive",
    0 => "zero",
    1..=10 => "small range",
    _ => "other"
}

// Collections
let mut map = HashMap()
map.insert("key", "value")
map.get("key").unwrap()
```

## Quality Engineering

### Code Quality Standards
- **Complexity Limits**: Functions must have cyclomatic complexity ≤10
- **Zero SATD Policy**: No TODO/FIXME/HACK comments allowed
- **Lint Compliance**: All clippy warnings treated as errors
- **Pre-commit Hooks**: Automated quality gates prevent regressions

### Testing
- Unit tests for core functionality
- Integration tests for CLI commands
- Property-based testing for mathematical invariants
- Fuzz testing for edge case discovery

### Quality Gate Script
```bash
# Run quality checks
./scripts/quality-gate.sh src

# Checks performed:
# - Function complexity ≤10
# - No technical debt comments
# - All tests passing
```

## Development

```bash
# Clone repository
git clone https://github.com/paiml/ruchy.git
cd ruchy

# Build
cargo build --release

# Run tests
cargo test

# Check quality
./scripts/quality-gate.sh src

# Install locally
cargo install --path .
```

## Documentation

- [Language Specification](./docs/SPECIFICATION.md)
- [Development Roadmap](./docs/execution/roadmap.md)
- [Change Log](./CHANGELOG.md)
- [Contributing Guidelines](./CLAUDE.md)

## Related Projects

- [ruchy-book](https://github.com/paiml/ruchy-book) - Language documentation and examples
- [rosetta-ruchy](https://github.com/paiml/rosetta-ruchy) - Algorithm implementations  
- [ruchyruchy](https://github.com/paiml/ruchyruchy) - Test suite

## License

MIT OR Apache-2.0