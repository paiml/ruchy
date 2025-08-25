# Ruchy - Modern Systems Scripting Language 🚀

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-374%20passing-green.svg)](./tests)
[![v1.14.0](https://img.shields.io/badge/v1.14.0-CLI--TOOLING-gold.svg)](./CHANGELOG.md)

## 🛠️ Comprehensive CLI Tooling (v1.14.0)

**Ruchy provides 29 CLI commands** for a complete development experience including syntax checking, formatting, linting, testing, AST analysis, formal verification, performance analysis, and quality scoring.

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

# Run tests
ruchy test script.ruchy

# Advanced tooling
ruchy ast script.ruchy --json
ruchy score script.ruchy
ruchy provability script.ruchy --verify
```

## ✨ Key Features

### 🛠️ CLI TOOLING
- **Syntax Checking**: `ruchy check` - Validate syntax without execution
- **Code Formatting**: `ruchy fmt` - Consistent code style
- **Linting**: `ruchy lint` - Code quality analysis with auto-fix
- **Testing**: `ruchy test` - Test runner with coverage reporting
- **AST Analysis**: `ruchy ast` - Abstract syntax tree visualization
- **Performance Analysis**: `ruchy runtime` - BigO complexity detection
- **Quality Scoring**: `ruchy score` - Unified code quality metrics
- **Formal Verification**: `ruchy provability` - Correctness analysis

### 🚀 LANGUAGE FEATURES
- **Pipeline Operator (`|>`)**: Functional programming support
- **String Methods**: Complete manipulation (contains, starts_with, replace, etc.)
- **Array Operations**: Comprehensive methods (push, pop, insert, remove, etc.)
- **Type System**: Static typing with inference

### 📊 QUALITY ENGINEERING
- **374 Tests Passing**: Unit, integration, CLI, and documentation tests
- **Zero-Warning Build**: All clippy warnings resolved
- **Toyota Way**: Zero-defect quality gates with comprehensive testing
- **Automated Quality Gates**: Pre-commit hooks ensure code quality

### 🎊 SELF-HOSTING CAPABILITY
- **Bootstrap Compilation**: Ruchy compiler written in Ruchy itself
- **Direct Codegen**: Rust code generation with `--minimal` flag
- **Type Inference**: Algorithm W with constraint solving
- **Parser Self-Compilation**: Complete parsing support

### Advanced Language Features
- **Both Lambda Syntaxes**: `|x| x + 1` and `x => x + 1` fully supported
- **Pattern Matching**: Comprehensive match expressions with guards
- **Struct & Impl Blocks**: Complete object-oriented programming support
- **Module System**: `use`, `mod`, and path resolution with `::` syntax
- **Error Handling**: Result types, Option types, and try operator `?`

### Development Tools
- **Formal Verification**: Mathematical correctness analysis
- **BigO Analysis**: Algorithmic complexity detection
- **AST Analysis**: Semantic analysis with metrics
- **Quality Scoring**: Code quality assessment
- **Performance Profiling**: Memory and bottleneck analysis

### REPL Features
- **Resource-bounded evaluation**: Memory limits, timeouts, stack depth protection
- **Multiline editing**: Automatic detection of incomplete expressions
- **Syntax highlighting**: Real-time colored output
- **Tab completion**: Context-aware completions
- **Persistent history**: Cross-session command history

## Example Code

### Basic Syntax
```rust
// Variables and functions
let x = 42
fun add(a: i32, b: i32) -> i32 { a + b }

// Pattern matching
match value {
    0 => "zero",
    1..=10 => "small", 
    _ => "large"
}

// Arrays and functions
let numbers = [1, 2, 3, 4, 5]
numbers.map(|x| x * 2).filter(|x| x > 5)

// Control flow
for i in 1..=5 {
    println(i)
}

if x > 0 { "positive" } else { "negative" }
```

## Installation

```bash
cargo install ruchy
```

## License

MIT OR Apache-2.0
