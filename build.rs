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

    // ── provable-contracts binding enforcement (AllImplemented) ──
    {
        let binding_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap_or(std::path::Path::new("."))
            .parent().unwrap_or(std::path::Path::new("."))
            .join("provable-contracts/contracts/ruchy/binding.yaml");
        
        println!("cargo:rerun-if-changed={}", binding_path.display());
        
        if binding_path.exists() {
            #[derive(serde::Deserialize)]
            struct BF { #[allow(dead_code)] version: String, bindings: Vec<B> }
            #[derive(serde::Deserialize)]
            struct B { contract: String, equation: String, status: String }
            
            if let Ok(yaml) = std::fs::read_to_string(&binding_path) {
                if let Ok(bf) = serde_yaml_ng::from_str::<BF>(&yaml) {
                    let (mut imp, mut gaps) = (0u32, Vec::new());
                    for b in &bf.bindings {
                        let var = format!("CONTRACT_{}_{}", 
                            b.contract.trim_end_matches(".yaml").to_uppercase().replace('-', "_"),
                            b.equation.to_uppercase().replace('-', "_"));
                        println!("cargo:rustc-env={var}={}", b.status);
                        if b.status == "implemented" { imp += 1; }
                        else { gaps.push(var.clone()); }
                    }
                    let total = bf.bindings.len() as u32;
                    println!("cargo:warning=[contract] AllImplemented: {imp}/{total} implemented, {} gaps", gaps.len());
                    if !gaps.is_empty() {
                        for g in &gaps { println!("cargo:warning=[contract] UNALLOWED GAP: {g}"); }
                        panic!("[contract] AllImplemented: {} gap(s). Fix bindings or update status.", gaps.len());
                    }
                }
            }
        }
    }
}
