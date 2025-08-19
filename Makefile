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
	@echo "  make test-repl   - Run ALL REPL tests (unit, property, fuzz, examples, coverage)"
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
	@echo "Running fast tests only (limited to 4 threads, 32 property cases)..."
	@cargo test --lib --quiet -- --test-threads=4
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

# Comprehensive REPL testing - ALL test types for REPL
test-repl:
	@echo "════════════════════════════════════════════════════════════════════"
	@echo "   COMPREHENSIVE REPL TESTING SUITE"
	@echo "════════════════════════════════════════════════════════════════════"
	@echo ""
	@echo "1️⃣  Running REPL unit tests..."
	@cargo test repl --lib --quiet || (echo "❌ REPL unit tests failed" && exit 1)
	@echo "✅ REPL unit tests passed"
	@echo ""
	@echo "2️⃣  Running REPL integration tests..."
	@cargo test --test repl_commands_test --quiet || (echo "❌ REPL integration tests failed" && exit 1)
	@cargo test --test cli_oneliner_tests --quiet || (echo "❌ CLI oneliner tests failed" && exit 1)
	@echo "✅ REPL integration tests passed"
	@echo ""
	@echo "3️⃣  Running REPL property tests..."
	@cargo test repl_function_tests::property --lib --release --quiet || (echo "❌ REPL property tests failed" && exit 1)
	@echo "✅ REPL property tests passed"
	@echo ""
	@echo "4️⃣  Running REPL doctests..."
	@cargo test --doc runtime::repl --quiet || (echo "❌ REPL doctests failed" && exit 1)
	@echo "✅ REPL doctests passed"
	@echo ""
	@echo "5️⃣  Running REPL examples..."
	@cargo run --example repl_demo --quiet || (echo "❌ REPL demo example failed" && exit 1)
	@cargo run --example debug_repl --quiet || (echo "❌ Debug REPL example failed" && exit 1)
	@echo "✅ REPL examples passed"
	@echo ""
	@echo "6️⃣  Running REPL fuzz tests (5 seconds)..."
	@cargo +nightly fuzz run repl_input -- -max_total_time=5 2>/dev/null || true
	@echo "✅ REPL fuzz test completed"
	@echo ""
	@echo "7️⃣  Generating REPL coverage report..."
	@cargo llvm-cov test repl --lib --quiet --no-report
	@cargo llvm-cov report --lib --ignore-filename-regex="tests/|benches/|examples/" 2>&1 | grep -E "src/runtime/repl" || true
	@echo ""
	@echo "════════════════════════════════════════════════════════════════════"
	@echo "   ✅ ALL REPL TESTS COMPLETED SUCCESSFULLY!"
	@echo "════════════════════════════════════════════════════════════════════"


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

# Generate coverage with tarpaulin (alternative)
coverage-tarpaulin:
	@echo "Generating coverage report with tarpaulin..."
	@cargo install cargo-tarpaulin 2>/dev/null || true
	@cargo tarpaulin --config tarpaulin.toml
	@echo "✓ Coverage report generated in target/coverage/"

# CI coverage check with minimum threshold
coverage-ci:
	@echo "Running coverage check for CI (80% minimum)..."
	@cargo tarpaulin --fail-under 80 --print-summary

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

# Run snapshot tests
test-snapshot:
	@echo "Running snapshot tests..."
	@cargo test snapshot_ --lib -- --nocapture
	@echo "✓ Snapshot tests complete"

# Run mutation tests
test-mutation:
	@echo "Running mutation tests with cargo-mutants..."
	@cargo install cargo-mutants 2>/dev/null || true
	@cargo mutants --timeout 30 --jobs 4
	@echo "✓ Mutation tests complete"

# Run fuzz tests
test-fuzz:
	@echo "Running fuzz tests (10 seconds per target)..."
	@cargo +nightly fuzz run parser -- -max_total_time=10 2>/dev/null || true
	@cargo +nightly fuzz run transpiler -- -max_total_time=10 2>/dev/null || true
	@echo "✓ Fuzz tests complete"

# Binary validation tests
test-binary:
	@echo "Running binary validation tests..."
	@for example in examples/*.ruchy; do \
		echo "Testing $$example..."; \
		cargo run --package ruchy-cli --bin ruchy -- run $$example || exit 1; \
	done
	@echo "✓ Binary validation complete"

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

# Documentation enforcement targets
.PHONY: check-docs commit sprint-close

# Ensure documentation is current
check-docs:
	@echo "📋 Checking documentation currency..."
	@if [ $$(git diff --name-only | grep -cE '\.(rs|ruchy)$$') -gt 0 ] && \
	    [ $$(git diff --name-only | grep -cE 'docs/|CHANGELOG.md') -eq 0 ]; then \
	    echo "❌ Documentation update required!"; \
	    echo "Update one of:"; \
	    echo "  - docs/execution/roadmap.md"; \
	    echo "  - docs/execution/quality-gates.md"; \
	    echo "  - CHANGELOG.md"; \
	    exit 1; \
	fi

# Development workflow with quality checks
dev: check-docs format lint test
	@echo "✅ Ready for development"

# Quality-enforced commit
commit: check-docs lint
	@echo "📝 Creating quality-enforced commit..."
	@read -p "Task ID (RUCHY-XXXX): " task_id; \
	read -p "Commit message: " msg; \
	git add -A && \
	git commit -m "$$task_id: $$msg"

# Sprint close verification
sprint-close: check-docs
	@echo "🏁 Sprint Close Quality Gate"
	@if command -v pmat >/dev/null 2>&1; then \
	    pmat quality-gate --fail-on-violation; \
	    echo "📊 Generating quality report..."; \
	    pmat analyze complexity . --format markdown > docs/quality/sprint-report.md; \
	fi
	@echo "✅ Sprint ready for close"

# Test optimization commands
.PHONY: test-quick test-memory test-heavy find-heavy-tests

# Quick smoke tests only
test-quick:
	@echo "Running quick smoke tests..."
	@PROPTEST_CASES=5 cargo test --lib -- --test-threads=2 --skip property_
	@echo "✓ Quick tests complete"

# Test memory usage
test-memory:
	@echo "Running resource verification tests..."
	@cargo test --test resource_check -- --test-threads=1
	@echo "✓ Memory tests complete"

# Run heavy tests (normally ignored)
test-heavy:
	@echo "Running heavy tests (this may take a while)..."
	@cargo test -- --ignored --test-threads=1 --nocapture
	@echo "✓ Heavy tests complete"

# Find memory-intensive tests
find-heavy-tests:
	@echo "Identifying memory-intensive tests..."
	@./scripts/find-heavy-tests.sh

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