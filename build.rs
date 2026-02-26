/// Build script for ruchy.
///
/// Exposes compile-time metadata as environment variables for use in the binary.
fn main() {
    // Re-run only when these files change
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=build.rs");

    // Expose build metadata as compile-time environment variables
    if let Ok(version) = std::env::var("CARGO_PKG_VERSION") {
        println!("cargo:rustc-env=RUCHY_VERSION={version}");
    }

    // Record build timestamp (UTC) for diagnostics
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    println!("cargo:rustc-env=RUCHY_BUILD_TIMESTAMP={now}");

    // Detect target features for SIMD availability reporting
    println!("cargo:rustc-check-cfg=cfg(has_avx2)");
    println!("cargo:rustc-check-cfg=cfg(has_neon)");

    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_features = std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();

    if target_arch == "x86_64" && target_features.contains("avx2") {
        println!("cargo:rustc-cfg=has_avx2");
    }
    if target_arch == "aarch64" && target_features.contains("neon") {
        println!("cargo:rustc-cfg=has_neon");
    }
}
