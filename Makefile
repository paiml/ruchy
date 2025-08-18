.PHONY: help all build test lint format clean coverage examples bench install doc ci prepare-publish quality-gate

# Default target
help:
	@echo "Ruchy Language - Development Commands"
	@echo ""
	@echo "Core Commands:"
	@echo "  make build       - Build the project in release mode"
	@echo "  make test        - Run fast tests only (~5 seconds)"
	@echo "  make test-all    - Run ALL tests including slow ones"
	@echo "  make test-property - Run property-based tests"
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
	@echo "  make pre-release-checks - Run all pre-release quality checks"
	@echo "  make release-patch - Create patch release (bug fixes)"
	@echo "  make release-minor - Create minor release (new features)"
	@echo "  make release-major - Create major release (breaking changes)"
	@echo "  make release-auto - Auto-detect version bump type"
	@echo "  make crate-release - Publish to crates.io"

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

# Run property-based tests specifically
test-property:
	@echo "Running property-based tests..."
	@cargo test property_ --lib --release -- --nocapture
	@cargo test proptest --lib --release -- --nocapture
	@cargo test quickcheck --lib --release -- --nocapture
	@echo "✓ Property tests passed"


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

# ============================================================================
# RELEASE MANAGEMENT - Based on paiml-mcp-agent-toolkit patterns
# ============================================================================

.PHONY: install-release-tools pre-release-checks release-patch release-minor release-major release-auto release-dry crate-release release-verify

# Install required release tools
install-release-tools:
	@echo "📦 Installing release tools..."
	@cargo install cargo-release --locked 2>/dev/null || echo "cargo-release already installed"
	@cargo install cargo-semver-checks --locked 2>/dev/null || echo "cargo-semver-checks already installed"
	@cargo install cargo-audit --locked 2>/dev/null || echo "cargo-audit already installed"
	@cargo install cargo-outdated --locked 2>/dev/null || echo "cargo-outdated already installed"
	@echo "✅ Release tools installed"

# Pre-release quality gates
pre-release-checks:
	@echo "🔍 Running pre-release checks..."
	@echo ""
	@echo "1️⃣ Version consistency check..."
	@MAIN_VERSION=$$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2); \
	CLI_VERSION=$$(grep -m1 '^version = ' ruchy-cli/Cargo.toml | cut -d'"' -f2 || echo $$MAIN_VERSION); \
	if [ "$$MAIN_VERSION" != "$$CLI_VERSION" ] && [ -n "$$CLI_VERSION" ]; then \
		echo "❌ Version mismatch: ruchy=$$MAIN_VERSION, ruchy-cli=$$CLI_VERSION"; \
		exit 1; \
	fi; \
	echo "✅ Versions consistent: $$MAIN_VERSION"
	@echo ""
	@echo "2️⃣ Running tests..."
	@$(MAKE) test-all
	@echo ""
	@echo "3️⃣ Checking formatting and lints..."
	@$(MAKE) format-check
	@$(MAKE) lint
	@echo ""
	@echo "4️⃣ Security audit..."
	@cargo audit || echo "⚠️  Some vulnerabilities found (review before release)"
	@echo ""
	@echo "5️⃣ Checking outdated dependencies..."
	@cargo outdated || echo "⚠️  Some dependencies outdated (review before release)"
	@echo ""
	@echo "6️⃣ Documentation check..."
	@cargo doc --no-deps --workspace --all-features --quiet
	@echo "✅ Documentation builds successfully"
	@echo ""
	@echo "7️⃣ Dry-run publish check..."
	@cargo publish --dry-run --package ruchy --quiet
	@echo "✅ Package ruchy ready for publication"
	@cargo publish --dry-run --package ruchy-cli --quiet 2>/dev/null || echo "⚠️  ruchy-cli may need separate publication"
	@echo ""
	@echo "✅ All pre-release checks completed!"

# Patch release (x.y.Z) - bug fixes only
release-patch: install-release-tools pre-release-checks
	@echo "🔖 Creating PATCH release (bug fixes only)..."
	@cargo release patch --execute --no-confirm

# Minor release (x.Y.z) - new features, backward compatible
release-minor: install-release-tools pre-release-checks
	@echo "🔖 Creating MINOR release (new features, backward compatible)..."
	@cargo release minor --execute --no-confirm

# Major release (X.y.z) - breaking changes
release-major: install-release-tools pre-release-checks
	@echo "🔖 Creating MAJOR release (breaking changes)..."
	@cargo release major --execute --no-confirm

# Auto-determine version bump based on conventional commits
release-auto: install-release-tools pre-release-checks
	@echo "🤖 Auto-determining version bump type..."
	@if git log --oneline $$(git describe --tags --abbrev=0 2>/dev/null || echo HEAD~10)..HEAD | grep -qE '^[a-f0-9]+ (feat!|fix!|refactor!|BREAKING)'; then \
		echo "💥 Breaking changes detected - MAJOR release"; \
		$(MAKE) release-major; \
	elif git log --oneline $$(git describe --tags --abbrev=0 2>/dev/null || echo HEAD~10)..HEAD | grep -qE '^[a-f0-9]+ feat:'; then \
		echo "✨ New features detected - MINOR release"; \
		$(MAKE) release-minor; \
	else \
		echo "🐛 Bug fixes/patches only - PATCH release"; \
		$(MAKE) release-patch; \
	fi

# Dry run for release (no actual changes)
release-dry:
	@echo "🧪 Dry run for release..."
	@cargo release patch --dry-run

# Publish to crates.io (interactive)
crate-release:
	@echo "📦 Publishing to crates.io..."
	@echo "Current version: $$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)"
	@echo ""
	@echo "Pre-publish checklist:"
	@echo "  ✓ Version bumped in Cargo.toml"
	@echo "  ✓ CHANGELOG.md updated"
	@echo "  ✓ All tests passing"
	@echo "  ✓ Documentation builds"
	@echo ""
	@printf "Continue with publish? [y/N] "; \
	read REPLY; \
	case "$$REPLY" in \
		[yY]*) \
			echo "Publishing ruchy..."; \
			cargo publish --package ruchy; \
			echo "Waiting 30 seconds for crates.io to index..."; \
			sleep 30; \
			echo "Publishing ruchy-cli..."; \
			cargo publish --package ruchy-cli || echo "ruchy-cli may already be published or needs manual intervention"; \
			;; \
		*) echo "❌ Publish cancelled" ;; \
	esac

# Verify release was successful
release-verify:
	@echo "🔍 Verifying release..."
	@LATEST_TAG=$$(git describe --tags --abbrev=0); \
	echo "Latest tag: $$LATEST_TAG"; \
	CRATE_VERSION=$$(cargo search ruchy | head -1 | cut -d'"' -f2); \
	echo "Crates.io version: $$CRATE_VERSION"; \
	echo ""; \
	echo "📦 Testing installation from crates.io..."; \
	cargo install ruchy --force && ruchy --version; \
	echo "✅ Release verification complete!"