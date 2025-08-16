# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-01-16

### Added
- **Type Inference Engine** - Complete Hindley-Milner type inference with Algorithm W
  - Automatic type inference for all expressions
  - Unification algorithm with occurs check
  - Type environment tracking bindings and schemes
  - Integration with REPL `:type` command for type inspection
  - Support for polymorphic types and type schemes
  
- **Method Call Syntax** - Object-oriented programming style
  - Parse `x.method(args)` syntax with dot operator
  - Support for method chaining
  - Field access notation
  - Type inference for built-in methods (len, push, pop, chars)
  - Transpilation to idiomatic Rust method calls

- **Gradual Typing** - Progressive type safety
  - Mix typed and untyped code seamlessly
  - Optional type annotations on function parameters
  - Default to 'Any' type for untyped parameters
  - Type inference fills in missing types automatically

- **String Interpolation** - Python-style string formatting
  - Support for `{variable}` in strings
  - Transpiles to Rust's `format!` macro
  - Works with println! and other formatting functions

### Changed
- **REPL Improvements**
  - `:type` command now shows actual inferred types instead of placeholder
  - Better differentiation between expressions and statements
  - Improved error messages for type errors
  
- **Parser Enhancements**
  - Support for multi-statement programs
  - Better handling of optional semicolons
  - Import system improvements with `::` separators

### Fixed
- REPL evaluation correctly handles expressions vs statements
- String literals with interpolation markers parse correctly
- Type annotations on functions work properly
- Multi-statement programs parse into Block expressions
- Import parsing handles complex paths and braced imports

### Quality
- **Zero Technical Debt** - No TODO/FIXME/HACK comments
- **100% Lint-Free** - Zero clippy warnings
- **High Test Coverage** - ~77% line coverage
- **171 Tests Passing** - Comprehensive test suite
- All PMAT quality gates passing

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