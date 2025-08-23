# Contributing to Ruchy

Thank you for your interest in contributing to Ruchy! **With our historic self-hosting achievement (v1.5.0), you can now contribute to the Ruchy compiler using Ruchy itself!**

## 🎉 Self-Hosting Development

**NEW**: Ruchy is now fully self-hosting! This means:
- **Write compiler features in Ruchy**: Use familiar Ruchy syntax to implement parser, type inference, and codegen improvements
- **Bootstrap development cycle**: Test your changes immediately with `ruchy run` and `ruchy transpile --minimal`
- **Enhanced type safety**: Algorithm W type inference catches errors during development
- **Direct Rust output**: See exactly what Rust code your Ruchy contributions generate

## Code of Conduct

Please be respectful and constructive in all interactions. We aim to maintain a welcoming and inclusive community.

## Getting Started

### Traditional Rust Development
1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/ruchy`
3. Create a feature branch: `git checkout -b feature/your-feature`
4. Make your changes in Rust (`src/` directory)
5. Run tests: `make test` (fast tests, ~5 seconds)
6. Run linting: `make lint`
7. Commit with descriptive message
8. Push to your fork
9. Create a Pull Request

### 🆕 Self-Hosting Development (Recommended!)
1. Fork and clone as above
2. Create a feature branch: `git checkout -b feature/your-feature-ruchy`
3. **Write your feature in Ruchy** (see `bootstrap_*.ruchy` examples)
4. Test immediately: `ruchy run your_feature.ruchy`
5. Transpile to Rust: `ruchy transpile your_feature.ruchy --minimal --output feature.rs`
6. Integrate the generated Rust into the codebase
7. Run full test suite: `make test`
8. Commit both the `.ruchy` source and generated `.rs` files
9. Create a Pull Request showcasing self-hosting development!

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

## Quality Standards (v0.10.0 - Revolutionary Tools)

### Code Quality - Toyota Way (Zero Defects)

- **Zero SATD**: No TODO/FIXME/HACK comments - track in GitHub Issues instead
- **Zero Warnings**: All code must pass `make lint` with `-D warnings` flag
- **Complexity < 50**: All functions must have cyclomatic complexity under 50
- **Tested**: New features require tests with 80% minimum coverage
- **Documented**: Public APIs need documentation with examples

### Testing Requirements

- Unit tests for new functionality
- Property-based tests for complex logic  
- Integration tests for user-facing features
- Coverage minimum: 80% for all modules
- Use `ruchy test --coverage` to verify

### Revolutionary Development Tools

As of v0.10.0, Ruchy provides world-first development tools:

```bash
# Formal verification (World's First)
ruchy provability script.ruchy --verify

# Automatic BigO complexity detection (World's First)
ruchy runtime script.ruchy --bigo

# Enhanced AST analysis
ruchy ast script.ruchy --json --metrics

# Professional testing with coverage
ruchy test --coverage --threshold 80

# Code formatting
ruchy fmt script.ruchy --check

# Grammar-based linting
ruchy lint script.ruchy --strict
```

### Mandatory Quality Gates (BLOCKING)

All commits must pass these quality gates (enforced by pre-commit hooks):

```bash
# 1. Basic functionality must work
echo 'println("Hello")' | ruchy repl | grep -q "Hello"

# 2. Zero warnings allowed
make lint  # Runs: cargo clippy -- -D warnings

# 3. Zero SATD comments
! grep -r "TODO\|FIXME\|HACK" src/ --include="*.rs"

# 4. Coverage threshold
ruchy test --coverage --threshold 80

# 5. Complexity check (via PMAT if installed)
pmat check --max-complexity 50 --fail-fast
```

### Extreme Quality Engineering

We follow extreme quality engineering principles:

1. **Canonical AST**: All transformations use normalized representation
2. **Reference Interpreter**: Semantic verification against ground truth
3. **Deterministic Builds**: Reproducible compilation guaranteed
4. **Error Recovery**: Parser continues on malformed input
5. **Provenance Tracking**: Full audit trail of transformations
6. **Formal Verification**: Mathematical correctness via `ruchy provability`
7. **Performance Analysis**: Automatic BigO detection via `ruchy runtime`

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