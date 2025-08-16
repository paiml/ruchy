# Changelog

All notable changes to the Ruchy programming language will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2024-01-16

### Added
- **REPL State Persistence**: Functions, structs, traits, and impl blocks defined in REPL are now preserved across commands
- **String Interpolation**: Full support for string interpolation with `"Hello, {name}!"` syntax
- **REPL Grammar Coverage Testing**: Comprehensive testing framework to ensure all language constructs work in REPL
- **Property-Based Testing**: Integrated proptest for robust testing of parser and transpiler
- **Fuzzing Support**: Added libfuzzer integration for finding edge cases
- **Performance Benchmarks**: Criterion-based benchmarks for REPL operations
- **Usage Documentation**: Added comprehensive Usage section to README

### Fixed
- **Function Transpilation**: Fixed double braces issue in function bodies
- **Return Types**: Functions without explicit return types now correctly default to `-> ()`
- **Type Inference**: Fixed "Any" type mapping to use `impl std::fmt::Display`
- **REPL Commands**: All special commands (`:rust`, `:ast`, `:type`) now work correctly

### Changed
- **Code Quality**: Achieved zero SATD (Self-Admitted Technical Debt) - no TODO/FIXME/HACK comments
- **Test Coverage**: Increased test suite to 227 tests with comprehensive coverage
- **Documentation**: Improved inline documentation and examples

### Technical Improvements
- Fixed all clippy linting warnings
- Reduced PMAT quality violations from 125 to 124
- Improved code organization with better module structure

## [0.2.0] - 2024-01-15

### Added
- Basic REPL implementation
- AST-based transpilation to Rust
- Hindley-Milner type inference (Algorithm W)
- Pattern matching support
- Pipeline operators
- List comprehensions
- Actor model primitives
- Property test attributes

### Changed
- Complete rewrite of parser for better error recovery
- Improved transpilation accuracy

## [0.1.0] - 2024-01-10

### Added
- Initial release of Ruchy
- Basic lexer and parser
- Simple transpilation to Rust
- CLI interface
- Basic type system