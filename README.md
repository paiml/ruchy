# Ruchy Programming Language

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/ruchy.svg)](https://crates.io/crates/ruchy)
[![Test Coverage](https://img.shields.io/badge/coverage-70.62%25-yellow.svg)](https://github.com/paiml/ruchy)
[![Tests Passing](https://img.shields.io/badge/tests-3987%20passing-green.svg)](https://github.com/paiml/ruchy)
[![Production Ready](https://img.shields.io/badge/production-NOT%20READY-red.svg)](docs/PRODUCTION-READINESS-ASSESSMENT.md)

A modern, expressive programming language for data science and scientific computing, featuring a self-hosting compiler, comprehensive tooling, and enterprise-grade quality standards.

> ⚠️ **PRODUCTION READINESS**: Ruchy v3.94.0 is **NOT production-ready**. While demonstrating exceptional engineering quality (TDG A-, 3,987 tests, EXTREME TDD), it lacks ecosystem maturity, security audits, and stability guarantees required for production use. **Estimated time to production: 18-30 months**. See [`docs/PRODUCTION-READINESS-ASSESSMENT.md`](docs/PRODUCTION-READINESS-ASSESSMENT.md) for detailed analysis.
>
> **Appropriate uses**: Research, education, prototyping, experimentation
> **Inappropriate uses**: Production services, mission-critical systems, public-facing products

## Features

- **Self-Hosting Compiler**: Written in Rust with full bootstrapping capabilities
- **Interactive REPL**: Advanced REPL with syntax highlighting and completion
- **WebAssembly Support**: Compile to WASM for browser and edge deployment
- **Notebook Integration**: Jupyter-style notebooks with testing framework
- **Type System**: Bidirectional type checking with inference
- **Package Management**: Cargo integration with 140K+ crates via `ruchy add`
- **Quality First**: Toyota Way principles with PMAT A+ code standards

## Installation

```bash
# Install from crates.io
cargo install ruchy

# Install with MCP server support
cargo install ruchy --features mcp

# Or build from source
git clone https://github.com/paiml/ruchy
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
# Start the interactive REPL (no args)
ruchy

# Run a Ruchy script (interprets immediately, <1s)
ruchy script.ruchy
# or explicitly:
ruchy run script.ruchy

# Compile to production binary
ruchy compile script.ruchy -o myapp

# Format code
ruchy fmt src/

# Run tests
ruchy test tests/
```

## Language Examples

### Basic Syntax
```ruchy
// Variables
let x = 42
let name = "Ruchy"
println(f"Hello, {name}! x = {x}")

// Functions
fun add(a, b) {
    a + b
}
let result = add(10, 20)
println(f"10 + 20 = {result}")

// Pattern matching
let value = Some(5)
match value {
    Some(x) => println(f"Got {x}"),
    None => println("Nothing"),
}

// Collections
let numbers = [1, 2, 3, 4, 5]
println(f"Numbers: {numbers:?}")
```

### Package Management (NEW in v3.76.0)
```bash
# Create new Ruchy project with Cargo integration
ruchy new my_project

# Add dependencies from crates.io
cd my_project
ruchy add serde
ruchy add tokio@1.0

# Add dev dependencies
ruchy add --dev proptest

# Build project (auto-transpiles .ruchy → .rs)
ruchy build
ruchy build --release

# Run project
cargo run
```

### Data Science Features (DataFrame - 80% Complete)

> **Status**: DataFrame implementation is ~80% complete with 200K+ property tests proving correctness

**Implemented Features** ✅:
- ✅ DataFrame creation and basic operations (via polars-rs)
- ✅ CSV reading/writing
- ✅ **Filtering with predicates** (100K property tests - mathematical proof of correctness)
- ✅ Group by operations with aggregations
- ✅ **Aggregation functions**: sum, count, mean, min, max, std, var
- ✅ **Sorting** (ascending/descending, 100K property tests - stable sort verified)
- ✅ Joins (inner join)
- ✅ Export: to_csv(), to_json()
- ✅ Selection and slicing: select(), slice(), head(), tail()
- ✅ Metadata: shape(), columns(), rows()

**Test Quality**:
- 137 unit tests passing
- 200,000+ property test iterations (filter + sort)
- Complexity ≤10 for all functions (Toyota Way compliant)
- Comprehensive edge case coverage

**Example API** (from test suite):
```rust
// Note: DataFrame API is primarily tested at Rust level
// High-level Ruchy syntax is under development

// Create DataFrame (via polars-rs backend)
let df = dataframe::from_columns(vec![
    ("age", vec![25, 30, 35]),
    ("score", vec![95, 87, 92])
]).unwrap();

// Operations supported (tested with 200K+ property tests):
// - df.select("column_name")
// - df.filter(predicate)
// - df.sort_by("column", descending)
// - df.groupby("column")
// - df.sum(), mean(), min(), max(), std(), var()
// - df.join(other_df, "key_column")
```

**In Progress**:
- High-level Ruchy syntax for DataFrame operations
- Advanced join types (left, right, outer)
- Multi-column grouping
- Plotting integration

See `tests/dataframe_*_properties.rs` for comprehensive test examples.

## CLI Commands

### Core Commands
- `ruchy` - Start interactive REPL (no args, Deno-style UX)
- `ruchy <file>` - Execute a Ruchy script directly (interprets immediately)
- `ruchy run <file>` - Execute a Ruchy script (alias for direct execution)
- `ruchy -e "<code>"` - Evaluate code directly (e.g., `ruchy -e "println(1+1)"`)
- `ruchy compile <file>` - Compile to standalone binary
- `ruchy fmt <path>` - Format code (supports --check flag)

### WebAssembly
- `ruchy wasm compile <input> -o <output>` - Compile to WASM
- `ruchy wasm validate <module>` - Validate WASM module
- `ruchy wasm run <module>` - Execute WASM module

**WASM Distribution** (v3.99.2+):
Ruchy provides pre-built WASM binaries for browser and edge deployment:

```bash
# Build WASM package (for maintainers)
wasm-pack build --target web --no-default-features --features wasm-compile

# WASM artifacts available at:
# - pkg/ruchy_bg.wasm (~3.1MB optimized)
# - pkg/ruchy.js (JavaScript bindings)
# - pkg/ruchy_bg.wasm.d.ts (TypeScript definitions)
```

**Browser Usage**:
```html
<script type="module">
  import init, { WasmRepl } from './ruchy.js';

  await init();
  const repl = new WasmRepl();
  const result = repl.eval('1 + 2');
  console.log(result); // "3"
</script>
```

**Note**: WASM builds exclude HTTP and file I/O operations (not available in browser sandbox).

### Development Server

Ruchy provides a world-class development server with hot reload, WASM compilation, and graceful shutdown (v3.105.0+).

**Basic Usage**:
```bash
# Serve current directory on port 8080
ruchy serve

# Serve specific directory with custom port
ruchy serve ./dist --port 3000

# Enable watch mode (auto-restart on file changes)
ruchy serve --watch

# Enable WASM hot reload (.ruchy → .wasm on save)
ruchy serve --watch --watch-wasm
```

**Advanced Features**:

```bash
# Full development mode with all features
ruchy serve \
  --watch \                # Auto-restart on file changes
  --watch-wasm \           # Compile .ruchy → .wasm on save
  --debounce 200 \         # Debounce delay in ms (default: 300)
  --verbose \              # Show detailed logging
  --pid-file server.pid    # PID file for process management

# Output:
#   🚀 Ruchy Dev Server v3.105.0
#
#   ➜  Local:   http://127.0.0.1:8080
#   ➜  Network: http://192.168.1.100:8080
#   📁 Serving: ./dist
#   👀 Watching: ./dist/**/*
#   🦀 WASM: Hot reload enabled for .ruchy files
#
#   Ready Press Ctrl+C to stop
```

**Features**:
- **Hot Reload**: Automatic server restart on file changes (configurable debouncing)
- **WASM Compilation**: Auto-compile `.ruchy` files to `.wasm` on save
- **Graceful Shutdown**: Ctrl+C for clean shutdown (no `kill -9` needed!)
- **Network Access**: Displays both local and network URLs for mobile testing
- **PID Management**: RAII-based PID file with automatic cleanup
- **Beautiful UX**: Vite-style colored output with status indicators
- **Performance**: Multi-threaded async runtime with optimized TCP settings

**WASM Hot Reload Workflow**:
```bash
# 1. Start dev server with WASM watching
ruchy serve --watch --watch-wasm

# 2. Edit your .ruchy files
# When you save main.ruchy:
#   📝 Changed: main.ruchy
#   🦀 Compiling: main.ruchy
#   ✅ Compiled: main.wasm
#   ↻ Restarting server...

# 3. Browser automatically reloads with new WASM
```

**Production Deployment**:
```bash
# Build optimized WASM files
ruchy wasm compile *.ruchy -o dist/ --opt-level 3

# Serve production build (no watch mode)
ruchy serve ./dist --port 8080
```

### Notebook
- `ruchy notebook` - Start interactive notebook server on http://localhost:8080
- `ruchy notebook test <file>` - Test notebook with coverage
- `ruchy notebook convert <input> <output>` - Convert notebook format

**Notebook Features (v3.75.0+)**:
- **State Persistence**: Variables and functions persist across cell executions (SharedRepl)
- **Thread-Safe**: Arc-based concurrent access with Mutex synchronization
- **Markdown Support**: Full markdown rendering with XSS protection
- **Load/Save**: JSON-based `.rnb` notebook format
- **API Access**: RESTful API at `/api/execute`, `/api/render-markdown`, `/api/notebook/load`, `/api/notebook/save`

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
- **Thread-Safety**: Arc-based concurrency, property-tested with 10K+ iterations (v3.75.0+)
- **E2E Testing**: 21/21 Playwright tests enforced via pre-commit hooks

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

- **[Ruchy Book](https://github.com/paiml/ruchy-book)** - Comprehensive language guide with 259 examples
- **[Rosetta Ruchy](https://github.com/paiml/rosetta-ruchy)** - 100+ algorithm implementations showcasing language features
- **[Ruchy REPL Demos](https://github.com/paiml/ruchy-repl-demos)** - 180+ interactive REPL examples and tutorials
- **[Ruchy Ruchy](https://github.com/paiml/ruchyruchy)** - Self-hosting compiler demos and integration tests

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
- **Repository**: [github.com/paiml/ruchy](https://github.com/paiml/ruchy)
- **Issues**: [GitHub Issues](https://github.com/paiml/ruchy/issues)