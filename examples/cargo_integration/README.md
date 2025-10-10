# Ruchy Cargo Integration Example (CARGO-001)

This example demonstrates how to use Ruchy with Cargo as the build system.

## How It Works

1. **build.rs**: Custom build script that runs before compilation
   - Calls `ruchy::build_transpiler::transpile_all()` to find all `.ruchy` files
   - Transpiles them to `.rs` files in the same directory
   - Implements incremental compilation (only transpile changed files)

2. **Cargo.toml**: Standard Cargo manifest
   - `build = "build.rs"` enables the build script
   - `[build-dependencies]` includes ruchy as a build dependency
   - `[dependencies]` can include any Rust crate (reqwest, serde, tokio, etc.)

3. **src/main.ruchy**: Ruchy source code
   - Written in Ruchy syntax
   - Automatically transpiled to `src/main.rs` during build
   - Compiled as normal Rust code

## Usage

```bash
# Build the project (auto-transpiles .ruchy → .rs)
cargo build

# Run the program
cargo run

# Clean (removes transpiled .rs files)
cargo clean
```

## Benefits

- ✅ **Zero new infrastructure** - use existing crates.io
- ✅ **Immediate ecosystem** - access to 140,000+ Rust crates
- ✅ **Battle-tested tooling** - Cargo's dependency resolution
- ✅ **Familiar workflow** - Rust developers feel at home
- ✅ **Incremental builds** - only transpile changed files

## File Structure

```
ruchy-cargo-example/
├── Cargo.toml          # Standard Cargo manifest
├── build.rs            # Build script (transpiles .ruchy → .rs)
├── src/
│   ├── main.ruchy      # Ruchy source code
│   └── main.rs         # Auto-generated (gitignore this!)
└── target/             # Standard Cargo build output
```

## Adding Dependencies

Just use standard Cargo commands:

```bash
cargo add reqwest
cargo add serde_json
cargo add tokio --features full
```

Then use them in your `.ruchy` files like normal Rust crates!

## CARGO-001 Implementation Status

- ✅ Build script integration (`build.rs`)
- ✅ Automatic file discovery (glob patterns)
- ✅ Incremental compilation (only transpile changed files)
- ✅ Clear error reporting (file names + line numbers)
- ✅ Nested directory support
- ✅ Property-based tests (100% passing)
- ✅ Quality gates passed (≤10 complexity, A- TDG grade)
