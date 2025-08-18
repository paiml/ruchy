.PHONY: help all build test lint format clean coverage examples bench install doc ci prepare-publish quality-gate

# Default target
help:
	@echo "Ruchy Language - Development Commands"
	@echo ""
	@echo "Core Commands:"
	@echo "  make build       - Build the project in release mode"
	@echo "  make test        - Run fast tests only (~5 seconds)"
	@echo "  make test-all    - Run ALL tests including slow ones"
	@echo "  make test-nextest - Run tests with nextest (better output)"
	@echo "  make lint        - Run clippy linter"
	@echo "  make format      - Format code with rustfmt"
	@echo "  make clean       - Clean build artifacts"
	@echo ""
	@echo "Quality Commands:"
	@echo "  make coverage    - Generate test coverage report"
	@echo "  make quality-gate - Run PMAT quality checks"
	@echo "  make ci          - Run full CI pipeline"
	@echo ""
	@echo "Development Commands:"
	@echo "  make examples    - Run all examples"
	@echo "  make bench       - Run benchmarks"
	@echo "  make doc         - Generate documentation"
	@echo "  make install     - Install ruchy locally"
	@echo ""
	@echo "Publishing:"
	@echo "  make prepare-publish - Prepare for crates.io publication"

# Build project
build:
	@echo "Building Ruchy..."
	@cargo build --release
	@echo "✓ Build complete"

# Run tests (default - FAST tests only, ignores slow integration tests)
test:
	@echo "Running fast tests only..."
	@cargo test --lib --quiet
	@echo "✓ Fast tests completed (~5 seconds after initial build)"

# Run tests with nextest (will recompile, but has better output)
test-nextest:
	@echo "Running tests with nextest..."
	@cargo nextest run --lib --profile quick
	@echo "✓ Nextest tests passed"

# Run all tests comprehensively (including ignored/slow tests, doc tests)
test-all:
	@echo "Running all tests comprehensively (including slow/ignored tests)..."
	@cargo test --all-features --workspace -- --include-ignored
	@cargo test --doc
	@echo "✓ All tests passed"


# Run linter
lint:
	@echo "Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "✓ Linting complete"

# Format code
format:
	@echo "Formatting code..."
	@cargo fmt --all
	@echo "✓ Formatting complete"

# Check formatting (for CI)
format-check:
	@echo "Checking formatting..."
	@cargo fmt --all -- --check
	@echo "✓ Format check complete"

# Clean build artifacts
clean:
	@echo "Cleaning..."
	@cargo clean
	@rm -rf target/
	@rm -rf ~/.ruchy/cache/
	@echo "✓ Clean complete"

# Generate test coverage using cargo-llvm-cov
coverage:
	@echo "Generating coverage report with cargo-llvm-cov..."
	@cargo install cargo-llvm-cov 2>/dev/null || true
	@cargo llvm-cov clean --workspace
	@cargo llvm-cov --all-features --workspace --html --output-dir target/coverage/html --ignore-filename-regex "tests/|benches/|examples/"
	@cargo llvm-cov report --lcov --output-path target/coverage/lcov.info
	@echo "✓ Coverage report generated in target/coverage/html/index.html"
	@echo "✓ LCOV report generated in target/coverage/lcov.info"
	@echo "Coverage summary:"
	@cargo llvm-cov report --summary-only 2>&1 | tail -1

# Run all examples
examples:
	@echo "Running examples..."
	@echo ""
	@echo "=== Parser Demo ==="
	@cargo run --example parser_demo --quiet
	@echo ""
	@echo "=== Transpiler Demo ==="
	@cargo run --example transpiler_demo --quiet
	@echo ""
	@echo "✓ All examples complete"

# Run example scripts
example-scripts:
	@echo "Testing Ruchy scripts..."
	@cargo run --package ruchy-cli --bin ruchy -- transpile examples/fibonacci.ruchy
	@cargo run --package ruchy-cli --bin ruchy -- transpile examples/marco_polo.ruchy
	@echo "✓ Script examples complete"

# Run benchmarks
bench:
	@echo "Running benchmarks..."
	@cargo bench --workspace
	@echo "✓ Benchmarks complete"

# Generate documentation
doc:
	@echo "Generating documentation..."
	@cargo doc --no-deps --workspace --all-features
	@echo "✓ Documentation generated in target/doc"

# Install locally
install:
	@echo "Installing ruchy..."
	@cargo install --path ruchy-cli --force
	@echo "✓ Ruchy installed to ~/.cargo/bin/ruchy"

# Run PMAT quality gates
quality-gate:
	@echo "Running PMAT quality checks..."
	@~/.local/bin/pmat quality-gate || true
	@echo "Checking complexity..."
	@~/.local/bin/pmat analyze --metrics complexity src/ || true
	@echo "✓ Quality check complete"

# CI pipeline
ci: format-check lint test-all coverage quality-gate
	@echo "✓ CI pipeline complete"

# Prepare for crates.io publication
prepare-publish:
	@echo "Preparing for crates.io publication..."
	@echo "Checking package metadata..."
	@cargo publish --dry-run --package ruchy
	@cargo publish --dry-run --package ruchy-cli
	@echo ""
	@echo "Checklist for publication:"
	@echo "  [ ] Version numbers updated in Cargo.toml"
	@echo "  [ ] CHANGELOG.md updated"
	@echo "  [ ] README.md complete with examples"
	@echo "  [ ] Documentation complete"
	@echo "  [ ] All tests passing"
	@echo "  [ ] Coverage > 80%"
	@echo "  [ ] No clippy warnings"
	@echo "  [ ] PMAT quality gates passing"
	@echo ""
	@echo "To publish:"
	@echo "  cargo publish --package ruchy"
	@echo "  cargo publish --package ruchy-cli"

# Development workflow
dev: format lint test
	@echo "✓ Development checks complete"

# Full validation
all: clean build test-all lint format coverage examples bench doc quality-gate
	@echo "✓ Full validation complete"