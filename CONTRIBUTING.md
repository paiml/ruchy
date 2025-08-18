# Contributing to Ruchy

Thank you for your interest in contributing to Ruchy! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and constructive in all interactions. We aim to maintain a welcoming and inclusive community.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/ruchy`
3. Create a feature branch: `git checkout -b feature/your-feature`
4. Make your changes
5. Run tests: `make test` (fast tests, ~5 seconds)
6. Run linting: `make lint`
7. Commit with descriptive message
8. Push to your fork
9. Create a Pull Request

## Development Setup

### Prerequisites

- Rust 1.75 or later
- cargo-nextest: `cargo install cargo-nextest`
- cargo-llvm-cov: `cargo install cargo-llvm-cov`

### Building

```bash
cargo build --all-features --workspace
```

### Testing

```bash
# Run fast tests only (~5 seconds after initial build)
make test

# Run all tests including slow/integration tests
make test-all

# Run tests with nextest (better output)
make test-nextest

# Run specific test
cargo test test_name

# Run with coverage
make coverage
```

### Linting

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Or use make
make lint
```

## Quality Standards

### Code Quality

- **Zero SATD**: No TODO/FIXME/HACK comments
- **Lint Clean**: Production code must pass all clippy checks
- **Tested**: New features require tests
- **Documented**: Public APIs need documentation

### Testing Requirements

- Unit tests for new functionality
- Property-based tests for complex logic
- Integration tests for user-facing features
- Maintain >90% test pass rate

### Extreme Quality Engineering

We follow extreme quality engineering principles:

1. **Canonical AST**: All transformations use normalized representation
2. **Reference Interpreter**: Semantic verification against ground truth
3. **Deterministic Builds**: Reproducible compilation guaranteed
4. **Error Recovery**: Parser continues on malformed input
5. **Provenance Tracking**: Full audit trail of transformations

## Architecture

### Project Structure

```
ruchy/
├── src/
│   ├── frontend/      # Parser and lexer
│   ├── middleend/     # Type inference and analysis
│   ├── backend/       # Code generation
│   ├── runtime/       # REPL and execution
│   ├── transpiler/    # Core transpilation logic
│   ├── parser/        # Error recovery system
│   └── testing/       # Test infrastructure
├── ruchy-cli/         # Command-line interface
├── tests/             # Integration tests
├── examples/          # Example programs
└── docs/              # Documentation
```

### Key Components

- **Parser**: Recursive descent with Pratt parsing
- **Type System**: Hindley-Milner with Algorithm W
- **Transpiler**: AST to Rust code generation
- **REPL**: Interactive development environment

## Submitting Changes

### Pull Request Process

1. Ensure all tests pass
2. Update documentation if needed
3. Add entry to CHANGELOG.md
4. Ensure PR description explains:
   - What changes were made
   - Why they were needed
   - How they were tested

### Commit Messages

Follow conventional commits format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Testing
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `chore`: Maintenance

Example:
```
feat(repl): add variable persistence across commands

Implements state management for REPL sessions to maintain
variable bindings between commands.

Fixes #123
```

## Testing Guidelines

### Test Categories

1. **Unit Tests**: In `src/*/tests.rs` or `#[cfg(test)]` modules
2. **Integration Tests**: In `tests/` directory
3. **Property Tests**: Using proptest for invariants
4. **Snapshot Tests**: For regression detection
5. **Chaos Tests**: Environmental variance testing

### Writing Tests

```rust
#[test]
fn test_feature() {
    // Arrange
    let input = "test input";
    
    // Act
    let result = function_under_test(input);
    
    // Assert
    assert_eq!(result, expected);
}
```

## Documentation

### Code Documentation

```rust
/// Brief description of the function.
///
/// # Arguments
///
/// * `param` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Examples
///
/// ```
/// use ruchy::function;
/// let result = function(input);
/// ```
pub fn function(param: Type) -> Result<Output> {
    // Implementation
}
```

### User Documentation

- Update README.md for user-facing changes
- Add examples to `examples/` directory
- Update CLI help text if needed

## Release Process

1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Create release notes
4. Tag release: `git tag v0.x.x`
5. Push tag: `git push origin v0.x.x`

## Getting Help

- Open an issue for bugs or features
- Ask questions in discussions
- Check existing issues before creating new ones

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).