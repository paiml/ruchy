# Ruchy Programming Language

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/ruchy.svg)](https://crates.io/crates/ruchy)
[![Test Coverage](https://img.shields.io/badge/coverage-85%25+-brightgreen.svg)](https://github.com/noahgift/ruchy)

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

# Install with MCP server support
cargo install ruchy --features mcp

# Or build from source
git clone https://github.com/noahgift/ruchy
cd ruchy
cargo build --release
```

## MCP Server

Ruchy provides a Model Context Protocol (MCP) server that exposes code analysis, scoring, linting, and transpilation capabilities to Claude and other MCP clients.

### Installation

```bash
# Install Ruchy with MCP support
cargo install ruchy --features mcp
```

### Configuration

Add to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "ruchy": {
      "command": "ruchy",
      "args": ["mcp"]
    }
  }
}
```

### Available Tools

The Ruchy MCP server provides 7 tools:

- **ruchy-score**: Analyze code quality with unified 0.0-1.0 scoring system
- **ruchy-lint**: Real-time code linting with auto-fix suggestions
- **ruchy-format**: Format Ruchy source code with configurable style
- **ruchy-analyze**: Comprehensive code analysis with AST, metrics, and insights
- **ruchy-eval**: Evaluate Ruchy expressions with type safety
- **ruchy-transpile**: Transpile Ruchy code to Rust
- **ruchy-type-check**: Type check Ruchy expressions

### Usage

```bash
# Start MCP server (typically called by Claude Desktop)
ruchy mcp --verbose
```

For more details, see [docs/mcp-registry-publish.md](docs/mcp-registry-publish.md).

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

// Async/await with blocks and lambdas (NEW in v3.45.0)
async fn fetch_data(url) {
    let response = await http.get(url)
    response.json()
}

// Async blocks
let future_result = async {
    let data = await fetch_data("api/users")
    data.length
}

// Async lambdas
let processors = urls.map(async |url| await fetch_data(url))
let transformer = async |x, y| x + await compute(y)
```

### Actor System (NEW in v3.46.0)
```ruchy
// Define actors with state and message handlers
actor ChatAgent {
    name: String,
    message_count: i32,

    receive process_message(content: String, sender: String) {
        self.message_count = self.message_count + 1;
        println("[" + self.name + "] From " + sender + ": " + content)
    }

    receive get_stats() -> String {
        self.name + " processed " + self.message_count.to_string() + " messages"
    }
}

actor BankAccount {
    balance: i32,
    account_number: String,

    receive deposit(amount: i32) {
        self.balance = self.balance + amount;
        println("Deposited " + amount.to_string() + ". Balance: " + self.balance.to_string())
    }

    receive withdraw(amount: i32) {
        if amount <= self.balance {
            self.balance = self.balance - amount;
            println("Withdrew " + amount.to_string() + ". Balance: " + self.balance.to_string())
        }
    }

    receive get_balance() -> i32 {
        self.balance
    }
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
- **Mutation Testing**: 80%+ mutation coverage via cargo-mutants (Sprint 8 goal)
- **Complexity Limits**: All functions ≤10 cyclomatic complexity
- **Zero Technical Debt**: No TODO/FIXME comments allowed
- **PMAT A+ Grade**: Enforced via automated quality gates
- **TDD Practice**: Test-first development methodology

### Mutation Testing Strategy

We use **cargo-mutants v25.3.1** for empirical test quality validation:

- **Incremental Testing**: File-by-file mutation testing (5-30 min vs 10+ hours baseline)
- **Evidence-Based**: Tests target specific mutations identified by empirical analysis
- **Pattern Recognition**: Reusable test strategies (match arms, boundaries, stubs)
- **Quality Metrics**: 80%+ mutation coverage target across all modules

```bash
# Run mutation tests on specific file
cargo mutants --file src/frontend/parser/core.rs --timeout 300

# Run mutation tests on module
cargo mutants --file "src/frontend/parser/*.rs" --timeout 600

# See mutation test results
make mutation-test
```

## Development

### Basic Development Commands
```bash
# Run tests
make test

# Check coverage
make coverage

# Run quality checks
make lint

# Build documentation
make doc
```

### WebAssembly QA Framework
The project includes a comprehensive WebAssembly Quality Assurance Framework v3.0 with 4 validation phases:

```bash
# Run complete QA validation
make qa-framework

# Individual phases
make qa-foundation    # Coverage & infrastructure
make qa-browser       # Browser testing & WASM validation
make qa-quality       # Security & complexity analysis
make qa-optimization  # Performance & regression testing

# Quick quality checks
make qa-security      # Security analysis
make qa-complexity    # Complexity analysis
make qa-dashboard     # Interactive quality dashboard

# See all QA commands
make qa-help
```

**Quality Targets:**
- 90% branch coverage
- ≤10 cyclomatic complexity per function
- Zero security vulnerabilities
- <500KB optimized WASM binaries
- <5% performance regression tolerance

## Documentation

- [Language Specification](docs/SPECIFICATION.md)
- [Development Roadmap](docs/execution/roadmap.md)
- [Contributing Guide](CONTRIBUTING.md)
- [Architecture Overview](docs/architecture/README.md)

## Related Resources

- **[Ruchy Book](https://github.com/noahgift/ruchy-book)** - Comprehensive language guide with 259 examples
- **[Rosetta Ruchy](https://github.com/noahgift/rosetta-ruchy)** - 100+ algorithm implementations showcasing language features
- **[Ruchy REPL Demos](https://github.com/noahgift/ruchy-repl-demos)** - 180+ interactive REPL examples and tutorials
- **[Ruchy Ruchy](https://github.com/noahgift/ruchyruchy)** - Self-hosting compiler demos and integration tests

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