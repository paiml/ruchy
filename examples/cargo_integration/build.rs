//! Example build.rs for Cargo Integration (CARGO-001)
//!
//! This build script demonstrates how to use Ruchy's build transpiler
//! to automatically transpile .ruchy files to .rs files during cargo build.
//!
//! # Setup
//!
//! 1. Add this file as `build.rs` in your project root
//! 2. Add ruchy as a build dependency in Cargo.toml:
//!
//! ```toml
//! [build-dependencies]
//! ruchy = "3.71"
//! ```
//!
//! 3. Place .ruchy files in your src/ directory
//! 4. Run `cargo build` - .ruchy files will auto-transpile to .rs
//!
//! # Features
//!
//! - Automatic file discovery with glob patterns
//! - Incremental compilation (only transpile changed files)
//! - Clear error reporting with file names
//! - Nested directory support

fn main() {
    // Transpile all .ruchy files in src/ to .rs files
    //
    // Parameters:
    // - "src": Base directory to search
    // - "**/*.ruchy": Glob pattern to match files (recursive)
    // - "src": Output directory for .rs files
    ruchy::build_transpiler::transpile_all("src", "**/*.ruchy", "src")
        .expect("Failed to transpile Ruchy files");

    // Tell Cargo to re-run this build script if any .ruchy files change
    println!("cargo:rerun-if-changed=src");

    // Optional: Print status
    println!("cargo:warning=Transpiled .ruchy files to .rs");
}
