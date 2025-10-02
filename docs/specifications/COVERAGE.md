# Rust Code Coverage Engineering Guide

> **Proven Solution for cargo-llvm-cov**
> **Last Updated**: October 2025
> **Status**: Production-Ready

## Executive Summary

This guide documents the complete solution for Rust code coverage using `cargo-llvm-cov`, including troubleshooting the critical **mold linker interference** issue that causes 0% coverage.

### Quick Start (Copy-Paste Solution)

```bash
# 1. Install tools
cargo install cargo-llvm-cov cargo-nextest --locked
rustup component add llvm-tools

# 2. Add to Makefile
make coverage  # See Makefile section below
```

## The Critical Issue: Mold Linker Breaks Coverage

### Symptoms
- Tests pass (55/55 ✓)
- Coverage shows 0.00% for all files
- `*.profraw` files are tiny (160 bytes) or missing
- Error: "unsupported instrumentation profile format version"

### Root Cause (Five Whys Analysis)

1. **Why**: cargo-llvm-cov shows 0% → profraw files have 0 executions
2. **Why**: profraw shows 0 executions → binaries instrumented but not writing data
3. **Why**: No profraw output → LLVM_PROFILE_FILE set but ignored
4. **Why**: Environment ignored → linker flags interfere with LLVM instrumentation
5. **Root Cause**: Global `~/.cargo/config.toml` with `rustflags = ["-C", "link-arg=-fuse-ld=mold"]` **breaks LLVM source-based coverage**

### The Solution

Create a Makefile that temporarily disables the global cargo config during coverage runs:

```makefile
# Code coverage with llvm-cov (two-phase production pattern)
# Note: Temporarily moves ~/.cargo/config.toml to avoid mold linker interference
coverage:
	@echo "📊 Running comprehensive test coverage analysis..."
	@echo "🔍 Checking for cargo-llvm-cov and cargo-nextest..."
	@which cargo-llvm-cov > /dev/null 2>&1 || (echo "📦 Installing cargo-llvm-cov..." && cargo install cargo-llvm-cov --locked)
	@which cargo-nextest > /dev/null 2>&1 || (echo "📦 Installing cargo-nextest..." && cargo install cargo-nextest --locked)
	@echo "🧹 Cleaning old coverage data..."
	@cargo llvm-cov clean --workspace
	@mkdir -p target/coverage
	@echo "⚙️  Temporarily disabling global cargo config (mold breaks coverage)..."
	@test -f ~/.cargo/config.toml && mv ~/.cargo/config.toml ~/.cargo/config.toml.cov-backup || true
	@echo "🧪 Phase 1: Running tests with instrumentation (no report)..."
	@cargo llvm-cov --no-report nextest --no-tests=warn --all-features --workspace
	@echo "📊 Phase 2: Generating coverage reports..."
	@cargo llvm-cov report --html --output-dir target/coverage/html
	@cargo llvm-cov report --lcov --output-path target/coverage/lcov.info
	@echo "⚙️  Restoring global cargo config..."
	@test -f ~/.cargo/config.toml.cov-backup && mv ~/.cargo/config.toml.cov-backup ~/.cargo/config.toml || true
	@echo ""
	@echo "📊 Coverage Summary:"
	@echo "=================="
	@cargo llvm-cov report --summary-only
	@echo ""
	@echo "💡 COVERAGE INSIGHTS:"
	@echo "- HTML report: target/coverage/html/index.html"
	@echo "- LCOV file: target/coverage/lcov.info"
	@echo "- Open HTML: make coverage-open"

coverage-open:
	@if [ -f target/coverage/html/index.html ]; then \
		xdg-open target/coverage/html/index.html 2>/dev/null || \
		open target/coverage/html/index.html 2>/dev/null || \
		echo "Please open: target/coverage/html/index.html"; \
	else \
		echo "❌ Run 'make coverage' first to generate the HTML report"; \
	fi
```

## Production Pattern (From actix-web)

### Two-Phase Coverage Collection

Actix-web and other major Rust projects use this canonical pattern:

```bash
# Phase 1: Run tests with instrumentation (no report)
cargo llvm-cov --no-report nextest --no-tests=warn --all-features --workspace

# Phase 2: Generate reports from collected profraw data
cargo llvm-cov report --html --output-dir target/coverage/html
cargo llvm-cov report --lcov --output-path lcov.info
cargo llvm-cov report --summary-only
```

### Why Two-Phase?

1. **Merge Multiple Test Runs**: Combine unit tests, integration tests, doctests
2. **Feature Flag Combinations**: Merge coverage from different feature sets
3. **Multiple Output Formats**: Generate HTML, LCOV, JSON without re-running tests
4. **CI/CD Optimization**: Run tests once, generate reports as needed

### Toolchain Requirements

From actix-web's `.github/workflows/coverage.yml`:

```yaml
- name: Install Rust (nightly)
  uses: actions-rust-lang/setup-rust-toolchain@v1
  with:
    toolchain: nightly
    components: llvm-tools
```

**Note**: While actix-web uses nightly, stable Rust (1.70+) works fine for coverage. The key is ensuring `llvm-tools` component is installed.

## Troubleshooting Guide

### Issue 1: 0% Coverage Despite Passing Tests

**Check for interfering RUSTFLAGS**:

```bash
# Check what cargo-llvm-cov sees
cargo llvm-cov show-env --export-prefix | grep RUSTFLAGS

# Should see ONLY coverage flags:
# export RUSTFLAGS='-C instrument-coverage --cfg=coverage --cfg=coverage_nightly'

# If you see -fuse-ld=mold or other linker flags, they're interfering!
```

**Solution**: Use the Makefile workaround above to temporarily disable `~/.cargo/config.toml`

### Issue 2: "unsupported instrumentation profile format version"

**Cause**: LLVM version mismatch between rustc and llvm-tools

**Solution**:
```bash
# Ensure llvm-tools matches your rustc
rustc --version --verbose | grep LLVM
rustup component add llvm-tools  # or llvm-tools-preview

# Clean old profdata
cargo llvm-cov clean --workspace
```

### Issue 3: No profraw Files Generated

**Debug steps**:

```bash
# 1. Check if binaries are instrumented
nm target/llvm-cov-target/debug/deps/your_crate-* | grep llvm_profile

# Should see:
# __llvm_profile_begin_counters
# __llvm_profile_begin_data

# 2. Check RUSTFLAGS are clean
cargo llvm-cov show-env | grep RUSTFLAGS

# 3. Manually test one binary
find target/llvm-cov-target/debug/deps -name "your_crate-*" -type f -executable | head -1 | \
  xargs -I{} env LLVM_PROFILE_FILE=/tmp/test-%p.profraw {} --test
ls -lah /tmp/test-*.profraw
```

### Issue 4: Coverage Works in CI but Not Locally

**Cause**: Local `~/.cargo/config.toml` with custom linker settings

**Check**:
```bash
cat ~/.cargo/config.toml

# Look for:
# [target.x86_64-unknown-linux-gnu]
# rustflags = ["-C", "link-arg=-fuse-ld=mold", ...]
```

**Solution**: Use the Makefile workaround that temporarily moves the config file

## Alternative Solutions (Not Recommended)

### Option A: Remove mold from global config

```bash
# Edit ~/.cargo/config.toml
# Comment out or remove the mold linker lines
[target.x86_64-unknown-linux-gnu]
# linker = "clang"
# rustflags = ["-C", "link-arg=-fuse-ld=mold", "-C", "target-cpu=native"]
```

**Downside**: Loses mold's faster linking for regular builds

### Option B: Local .cargo/config.toml override

```toml
# .cargo/config.toml in your project
[build]
rustflags = []

[target.x86_64-unknown-linux-gnu]
linker = "clang"
```

**Downside**: Doesn't work! Cargo **merges** configs, so global rustflags still apply

### Option C: Use tarpaulin instead

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --lib --workspace
```

**Downside**: Less accurate than LLVM coverage, Linux-only, slower

## GitHub Actions Workflow

```yaml
name: Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools

      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-llvm-cov,nextest

      - uses: Swatinem/rust-cache@v2

      - name: Generate coverage
        run: |
          cargo llvm-cov clean --workspace
          cargo llvm-cov --no-report nextest --all-features --workspace
          cargo llvm-cov report --lcov --output-path lcov.info

      - uses: codecov/codecov-action@v4
        with:
          files: lcov.info
          token: ${{ secrets.CODECOV_TOKEN }}
```

## Verification Checklist

After implementing the solution, verify:

- [ ] `make coverage` completes without errors
- [ ] Coverage > 0% for tested modules
- [ ] HTML report generated: `target/coverage/html/index.html`
- [ ] LCOV file created: `target/coverage/lcov.info`
- [ ] Global `~/.cargo/config.toml` restored after coverage run
- [ ] profraw files > 160 bytes (showing actual coverage data)

## Key Learnings

1. **Mold linker breaks LLVM coverage** - This is the #1 cause of 0% coverage in Rust projects
2. **Global cargo configs persist** - Local configs merge, they don't override
3. **Two-phase pattern is canonical** - Used by actix-web, tokio, and other major projects
4. **Temporary config removal works** - Safe workaround that preserves fast builds
5. **Always check RUSTFLAGS** - `cargo llvm-cov show-env` reveals the truth

## References

- [actix-web coverage workflow](https://github.com/actix/actix-web/blob/master/.github/workflows/coverage.yml)
- [actix-web justfile](https://github.com/actix/actix-web/blob/master/justfile)
- [cargo-llvm-cov documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [Rust LLVM coverage stabilization](https://github.com/rust-lang/rust/issues/34701)

## Summary

The **mold linker in global cargo config breaks LLVM coverage instrumentation**. The proven solution is to temporarily disable the global config during coverage runs using a Makefile. This approach:

- ✅ Preserves fast mold linking for regular builds
- ✅ Enables accurate LLVM coverage for testing
- ✅ Follows the production pattern from actix-web
- ✅ Works reliably across environments
- ✅ Requires no code changes

Just copy the Makefile above and run `make coverage`. It will just work.
