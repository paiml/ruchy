# Ruchy Programming Language

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/ruchy.svg)](https://crates.io/crates/ruchy)
[![Test Coverage](https://img.shields.io/badge/coverage-46.41%25-green.svg)](https://github.com/noahgift/ruchy)

A modern, expressive programming language for data science and scientific computing, featuring a self-hosting compiler, comprehensive tooling, and enterprise-grade quality standards.

## Features

- **Self-Hosting Compiler**: Written in Rust with full bootstrapping capabilities
- **Interactive REPL**: Advanced REPL with syntax highlighting and completion
- **WebAssembly Support**: Compile to WASM for browser and edge deployment
- **Notebook Integration**: Jupyter-style notebooks with testing framework
- **Type System**: Bidirectional type checking with inference
- **Actor Model**: Built-in concurrency with supervision trees
- **Quality First**: Toyota Way principles with PMAT A+ code standards

## Installation

```bash
# Install from crates.io
cargo install ruchy

# Or build from source
git clone https://github.com/noahgift/ruchy
cd ruchy
cargo build --release
```

## Quick Start

```bash
# Start the interactive REPL
ruchy repl

# Run a Ruchy script
ruchy run script.ruchy

# Format code
ruchy fmt src/

# Run tests
ruchy test run tests/
```

## Language Examples

### Basic Syntax
```ruchy
// Variables and functions
let x = 42
let add = fn(a, b) => a + b

// Pattern matching
match value {
    Some(x) => println(f"Got {x}"),
    None => println("Nothing"),
}

// Async/await
async fn fetch_data(url) {
    let response = await http.get(url)
    response.json()
}
```

### Data Science Features
```ruchy
// DataFrame operations
let df = read_csv("data.csv")
let result = df
    |> filter(row => row.age > 18)
    |> group_by("category")
    |> agg(mean("value"))
    |> sort_by("mean_value", descending=true)

// Plotting
plot(df.x, df.y, kind="scatter", title="Analysis")
```

## CLI Commands

### Core Commands
- `ruchy repl` - Start interactive REPL
- `ruchy run <file>` - Execute a Ruchy script
- `ruchy fmt <path>` - Format code (supports --check flag)

### WebAssembly
- `ruchy wasm compile <input> -o <output>` - Compile to WASM
- `ruchy wasm validate <module>` - Validate WASM module
- `ruchy wasm run <module>` - Execute WASM module

### Notebook
- `ruchy notebook serve` - Start notebook server
- `ruchy notebook test <file>` - Test notebook with coverage
- `ruchy notebook convert <input> <output>` - Convert notebook format

### Testing
- `ruchy test run <path>` - Run tests with optional coverage
- `ruchy test report` - Generate test report (HTML/JSON/JUnit)

## Project Structure

```
ruchy/
├── src/
│   ├── frontend/       # Parser and AST
│   ├── middleend/      # Type system and inference
│   ├── backend/        # Code generation and transpilation
│   ├── runtime/        # REPL and interpreter
│   ├── lsp/           # Language server protocol
│   └── wasm/          # WebAssembly support
├── tests/             # Integration tests
├── examples/          # Example programs
└── docs/             # Documentation
```

## Quality Standards

This project follows strict quality engineering practices:

- **Test Coverage**: 46.41% line coverage, 50.79% branch coverage
- **Complexity Limits**: All functions ≤10 cyclomatic complexity
- **Zero Technical Debt**: No TODO/FIXME comments allowed
- **PMAT A+ Grade**: Enforced via automated quality gates
- **TDD Practice**: Test-first development methodology

## Development

```bash
# Run tests
cargo test

# Check coverage
cargo llvm-cov

# Run quality checks
cargo clippy -- -D warnings
cargo fmt -- --check

# Build documentation
cargo doc --open
```

## Documentation

- [Language Specification](docs/SPECIFICATION.md)
- [Development Roadmap](docs/execution/roadmap.md)
- [Contributing Guide](CONTRIBUTING.md)
- [Architecture Overview](docs/architecture/README.md)

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on our code of conduct and development process.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with Rust and the incredible Rust ecosystem
- Inspired by Python's expressiveness and Rust's safety
- Quality practices from Toyota Way and PMAT methodologies

## Contact

- **Author**: Noah Gift
- **Repository**: [github.com/noahgift/ruchy](https://github.com/noahgift/ruchy)
- **Issues**: [GitHub Issues](https://github.com/noahgift/ruchy/issues)