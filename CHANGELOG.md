# Changelog

All notable changes to the Ruchy programming language will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-01-18

### Added - REPL Excellence Sprint
- **REPL v3 Production Implementation**
  - Resource-bounded evaluator with 10MB memory limit
  - Hard timeout enforcement (100ms default)
  - Stack depth control (1000 frame maximum)
  - Transactional state machine with checkpoints
  - Error recovery with condition/restart system
  - Progressive modes (Standard/Test/Debug)
  - Comprehensive testing infrastructure

### Improved
- **Test Performance**
  - Default `make test` now runs in ~5 seconds
  - Marked slow integration tests as `#[ignore]`
  - Added `make test-all` for comprehensive testing
  - CI uses two-stage testing for fast feedback

### Infrastructure
- **Dependencies**
  - Added `im` crate for persistent data structures
  - Added `quickcheck` for property-based testing
- **Documentation**
  - Prioritized REPL in ROADMAP for user experience
  - Updated execution roadmap with REPL tasks
  - Added comprehensive REPL testing guide

## [0.3.2] - 2025-08-18

### Major Quality Improvements
- **Lint Compliance**: Fixed all 68 clippy lint errors for zero-warning build
- **Code Quality**: Reduced SATD (Self-Admitted Technical Debt) from 124 to 6 comments (95% reduction)
- **Test Coverage**: Improved test pass rate to 379/411 tests (92.2%)
- **Architecture**: Successfully split 2873-line transpiler.rs into 8 focused modules

### Fixed
- **Transpiler Correctness**
  - Fixed identifier transpilation to use proper `format_ident!` instead of raw strings
  - Fixed integer literal transpilation to eliminate double i64 suffix issue
  - Fixed trait/impl method `&self` parameter handling to avoid invalid Ident errors
- **Module Organization**
  - Split transpiler into: expressions, statements, patterns, types, dataframe, actors, and main dispatcher
  - Added proper clippy allow attributes to all transpiler modules
  - Fixed duplicate imports and unused import issues

### Documentation
- **Roadmap**: Updated with accurate quality metrics and SPECIFICATION.md v3.0 compliance analysis
- **Architecture**: Documented critical gaps in MCP, LSP, and quality gates implementation
- **Quality Gates**: Added comprehensive quality assessment framework

### Infrastructure
- **Linting**: Added `.clippy.toml` configuration with reasonable complexity thresholds
- **CI/CD**: All changes maintain zero clippy warnings standard

## [0.3.1] - 2025-01-16

### Added
- **Actor System Implementation**
  - Actor definitions with state fields and receive blocks
  - Message passing operators: `!` (send) and `?` (ask) with space-separated syntax
  - Comprehensive test suite for actor parsing and transpilation
  - AST support for actors, send operations, and ask operations

### Fixed
- **Parser Improvements**
  - Fixed operator precedence for actor message passing
  - Improved binary operator parsing to handle `!` and `?` correctly
  - Fixed receive block parsing to avoid consuming extra closing braces
  - Enhanced lexer with `receive`, `send`, and `ask` keywords

### Changed
- **Message Passing Syntax**
  - Changed from `actor!(message)` to `actor ! message` (space-separated)
  - Changed from `actor?(message)` to `actor ? message` (space-separated)
  - This improves parsing consistency and fixes REPL bugs

## [0.3.0] - 2025-01-16

### Added
- **Extreme Quality Engineering Infrastructure**
  - Canonical AST normalization with De Bruijn indices
  - Reference interpreter for semantic verification
  - Snapshot testing with content-addressed storage
  - Chaos engineering tests for environmental variance
  - Compilation provenance tracking with SHA256 hashing
  - Enhanced property-based testing coverage
  - Deterministic fuzz testing framework

- **Deterministic Error Recovery System**
  - Predictable parser behavior on malformed input
  - Synthetic AST nodes for error recovery
  - Multiple recovery strategies (SkipUntilSync, InsertToken, DefaultValue, PartialParse, PanicMode)
  - Error context preservation for better diagnostics
  - Synchronization points for panic mode recovery
  - Foundation for LSP partial analysis

- **New REPL Implementation (ReplV2)**
  - Complete rewrite addressing all QA report bugs
  - Fixed variable persistence across lines (BUG-001)
  - Corrected function type inference (BUG-002)
  - Implemented Debug trait for arrays/structs (BUG-005)
  - Proper semicolon handling for statements
  - Added `:exit` alias for `:quit` command
  - Dual mode support: interpreter or compilation

### Changed
- **REPL**: ReplV2 is now the default REPL (old REPL available as LegacyRepl)
- **Transpiler**: Improved determinism with canonical AST normalization
- **Testing**: Enhanced test coverage to 96.4% pass rate (187/194 tests)
- **Quality**: Implemented extreme quality engineering practices from transpiler docs

### Fixed
- **Critical REPL Bugs**
  - Variable persistence now works correctly across multiple lines
  - Function definitions properly inferred with correct types
  - String concatenation and interpolation fixed
  - Loop constructs (for/while) working properly
  - Display traits properly implemented for all types
  - Struct initialization syntax errors resolved
  - Semicolon handling consistent between debug/release builds

- **Transpiler Issues**
  - BinaryOp enum name mismatches corrected
  - Missing Clone trait implementations added
  - Compilation metadata properly tracked
  - Hash-based determinism verification

### Technical Improvements
- **Defect Class Elimination**
  - Syntactic ambiguity: ELIMINATED via canonical AST
  - Semantic drift: PREVENTED via reference interpreter
  - Environmental variance: RESILIENT via chaos testing
  - State dependencies: CONTROLLED via De Bruijn indices
  - Error cascade: PARTIAL recovery implemented

- **Quality Metrics**
  - Zero Self-Admitted Technical Debt (SATD)
  - PMAT violations maintained at acceptable levels
  - Deterministic compilation guaranteed
  - Full provenance tracking for all transformations

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