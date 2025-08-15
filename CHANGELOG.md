# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-15

### Added
- Initial release of Ruchy programming language
- Complete recursive descent parser with error recovery
- Transpiler generating idiomatic Rust code  
- Interactive REPL with multiple commands:
  - `:ast` - Display Abstract Syntax Tree
  - `:rust` - Show Rust transpilation
  - `:help` - Display help information
  - `:save`/`:load` - Session management
- CLI with multiple subcommands:
  - `repl` - Start interactive REPL (default)
  - `parse` - Parse and display AST
  - `transpile` - Convert Ruchy to Rust
  - `run` - Execute Ruchy scripts
  - `compile` - Compile to native executable
- Language features:
  - Literals (integers, floats, strings, booleans)
  - Binary operators with precedence
  - Control flow (if/else, while, for)
  - Functions and lambdas
  - Arrays and ranges
  - Pipeline operators
  - Pattern matching basics
- Comprehensive test suite with 146 passing tests
- Property-based testing with proptest
- Code coverage >80% via cargo-llvm-cov
- Zero clippy warnings with strict linting
- Benchmarks for parser and transpiler performance
- Example scripts demonstrating language features

### Technical Details
- Parser: ~100k LOC/sec performance
- Zero unsafe code
- Minimal dependencies
- Support for Rust 1.75+

### Known Limitations
- REPL eval function needs refinement for direct expression execution
- Type inference not yet implemented (placeholder in REPL)
- MCP integration pending
- Actor system not yet implemented

[0.1.0]: https://github.com/paiml/ruchy/releases/tag/v0.1.0