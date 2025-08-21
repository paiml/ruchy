# Ruchy

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-271%20passing-green.svg)](./tests)
[![v0.8.0](https://img.shields.io/badge/v0.8.0-Interpreter%20Complexity%20Reduction-blue.svg)](./ROADMAP.md)

A functional programming language that transpiles to idiomatic Rust, featuring a production-grade REPL with complexity-optimized interpreter and comprehensive test coverage.

## 🎯 Quick Start

```bash
# Install from crates.io
cargo install ruchy

# Run a one-liner
ruchy -e "println('Hello, World!')"

# Run with JSON output  
ruchy -e "2 + 2" --format json

# Start the REPL
ruchy repl

# Run a script
ruchy run script.ruchy
```

## ✨ Key Features

### Production-Grade REPL
- **Resource-bounded evaluation**: Memory limits, timeouts, stack depth protection
- **Multiline editing**: Automatic detection of incomplete expressions
- **Syntax highlighting**: Real-time colored output with O(n) performance
- **Tab completion**: Context-aware completions with O(1) keyword lookups
- **Persistent history**: Cross-session command history

### Core Language Features
```ruchy
// Pattern matching with exhaustiveness checking
match value {
    0 => "zero",
    1..=10 => "small",
    _ => "large"
}

// String interpolation
let name = "Ruchy"
println(f"Hello from {name}!")

// Functional programming
[1, 2, 3, 4, 5]
    .map(|x| x * 2)
    .filter(|x| x > 5)
    .reduce(0, |acc, x| acc + x)

// Loop expressions
for i in 1..=5 {
    println(i)
}

// Block expressions return values
let result = {
    let a = 10
    let b = 20
    a + b  // Returns 30
}
```

### Type System
- Type inference with bidirectional checking
- Option<T> and Result<T, E> types
- Generic functions and structs
- Trait definitions and implementations

## 🏗️ Architecture

### For Rust Developers

The Ruchy interpreter showcases advanced Rust patterns and optimizations. **[Read the detailed Interpreter Architecture](./docs/interpreter.md)** to learn about:

- **Resource-bounded evaluation** with `Instant` deadlines and memory tracking
- **Zero-allocation patterns** using `&str` and arena allocators
- **O(1) HashSet lookups** replacing O(n²) algorithms
- **Modular complexity management** keeping all functions under 50 cyclomatic complexity
- **Property-based testing** with `proptest` for invariant verification
- **Fuzz testing** for crash resistance

### Interpreter (v0.8.0 - Complexity Optimized)
- **Cyclomatic complexity**: Reduced from 209 to 50 (76% reduction)
- **Display formatting**: Extracted to modular helpers (<30 complexity each)
- **O(n²) algorithms eliminated**: HashSet-based lookups for O(1) performance
- **Memory tracking**: Per-evaluation resource bounds with `AtomicUsize`

### Transpiler
- Direct Rust code generation via `quote!` macros
- Zero-cost abstractions with `syn` and `proc-macro2`
- Hygienic macro expansion
- Incremental compilation support

## 📊 Quality Metrics

```
Tests:           271 passing (library)
                 34 passing (interpreter core)
                 10 property tests
                 10 fuzz tests
                 33 doctests
Code Coverage:   >80%
Complexity:      All functions <50 (enforced by PMAT)
Performance:     50MB/s parsing throughput
```

## 🔧 Development

```bash
# Run tests
make test

# Check quality gates
make lint

# Run benchmarks
make bench

# Generate documentation
cargo doc --open
```

### Quality Gates (Toyota Way - Zero Defects)
All commits must pass:
1. Core interpreter reliability tests
2. REPL functionality tests  
3. Cyclomatic complexity <50
4. Zero clippy warnings
5. 80% test coverage minimum

See [CLAUDE.md](./CLAUDE.md) for the full development protocol.

## 📚 Documentation

- **[Interpreter Architecture](./docs/interpreter.md)** - Deep dive into the v0.8.0 complexity-optimized interpreter
- [Language Specification](./docs/SPECIFICATION.md) - Complete language reference
- [Roadmap](./ROADMAP.md) - Development progress and priorities
- [Contributing Guidelines](./CONTRIBUTING.md) - How to contribute

## 🚀 Current Focus (v0.8.0)

- ✅ Interpreter complexity reduction (209 → 50)
- ✅ O(n²) algorithm elimination
- ✅ Display formatting modularization
- ✅ Extended test coverage (property, fuzz, examples)
- 🔄 Binary compilation via LLVM
- 🔄 DataFrame operations
- 🔄 Actor system

## 📦 Installation

### From crates.io
```bash
cargo install ruchy ruchy-cli
```

### From source
```bash
git clone https://github.com/yourusername/ruchy
cd ruchy
cargo build --release
cargo install --path . --path ruchy-cli
```

## 🤝 Contributing

Contributions welcome! Please read [CONTRIBUTING.md](./CONTRIBUTING.md) first.

Key principles:
- Zero defects (Toyota Way)
- Complexity budget (<50 per function)
- Test coverage >80%
- All PRs must pass quality gates

## 📄 License

MIT - See [LICENSE](./LICENSE) for details.

---

**Made with 🦀 in Rust** | [Documentation](https://docs.rs/ruchy) | [Crates.io](https://crates.io/crates/ruchy)