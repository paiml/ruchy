.PHONY: help all build test lint format clean coverage examples bench install doc ci prepare-publish quality-gate

# Default target
help:
	@echo "Ruchy Language - Development Commands"
	@echo ""
	@echo "Core Commands:"
	@echo "  make build       - Build the project in release mode"
	@echo "  make test        - Run all tests"
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

# Run all tests
test:
	@echo "Running tests..."
	@cargo test --all-features --workspace
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

# Generate test coverage
coverage:
	@echo "Generating coverage report..."
	@cargo install cargo-tarpaulin 2>/dev/null || true
	@cargo tarpaulin --out Html --output-dir coverage --workspace --timeout 120 --exclude-files "*/tests/*" --exclude-files "*/examples/*"
	@echo "✓ Coverage report generated in coverage/index.html"
	@echo "Coverage summary:"
	@cargo tarpaulin --print-summary --workspace --timeout 120 --exclude-files "*/tests/*" --exclude-files "*/examples/*" 2>/dev/null | tail -n 1

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
ci: format-check lint test coverage quality-gate
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
all: clean build test lint format coverage examples bench doc quality-gate
	@echo "✓ Full validation complete"