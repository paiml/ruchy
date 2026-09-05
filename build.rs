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

    emit_simd_cfgs();
    enforce_contract_bindings();
}

/// Detect target features for SIMD availability reporting.
fn emit_simd_cfgs() {
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

/// The sibling `provable-contracts` binding manifest.
#[derive(serde::Deserialize)]
struct BindingFile {
    #[allow(dead_code)]
    version: String,
    bindings: Vec<Binding>,
}

/// One contract equation and its implementation status.
#[derive(serde::Deserialize)]
struct Binding {
    contract: String,
    equation: String,
    status: String,
}

/// `../../provable-contracts/contracts/ruchy/binding.yaml`, relative to this crate.
fn binding_manifest_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("provable-contracts/contracts/ruchy/binding.yaml")
}

/// `CONTRACT_<CONTRACT>_<EQUATION>` — the env var exported for a binding.
fn binding_env_var(b: &Binding) -> String {
    format!(
        "CONTRACT_{}_{}",
        b.contract
            .trim_end_matches(".yaml")
            .to_uppercase()
            .replace('-', "_"),
        b.equation.to_uppercase().replace('-', "_")
    )
}

/// provable-contracts binding enforcement (AllImplemented).
///
/// With the sibling checkout present, every binding is exported as a `CONTRACT_*`
/// env var and any binding that is not `implemented` fails the build. Without it
/// the check is skipped *visibly* — a `cargo:warning` names the missing path —
/// because a gate that cannot run must never look like a gate that passed.
fn enforce_contract_bindings() {
    let binding_path = binding_manifest_path();
    println!("cargo:rerun-if-changed={}", binding_path.display());

    if !binding_path.exists() {
        println!(
            "cargo:warning=[contract] {} not found; AllImplemented binding check skipped (no sibling provable-contracts checkout)",
            binding_path.display()
        );
        return;
    }
    let Ok(yaml) = std::fs::read_to_string(&binding_path) else {
        return;
    };
    let Ok(bf) = serde_yaml_ng::from_str::<BindingFile>(&yaml) else {
        return;
    };
    let (implemented, gaps) = export_bindings(&bf.bindings);
    report_gaps(implemented, bf.bindings.len() as u32, &gaps);
}

/// Export each binding's status as an env var; return the implemented count and
/// the env var names of every binding that is not implemented.
fn export_bindings(bindings: &[Binding]) -> (u32, Vec<String>) {
    let (mut implemented, mut gaps) = (0u32, Vec::new());
    for b in bindings {
        let var = binding_env_var(b);
        println!("cargo:rustc-env={var}={}", b.status);
        if b.status == "implemented" {
            implemented += 1;
        } else {
            gaps.push(var);
        }
    }
    (implemented, gaps)
}

/// Print the AllImplemented summary and fail the build on any gap.
fn report_gaps(implemented: u32, total: u32, gaps: &[String]) {
    println!(
        "cargo:warning=[contract] AllImplemented: {implemented}/{total} implemented, {} gaps",
        gaps.len()
    );
    if gaps.is_empty() {
        return;
    }
    for g in gaps {
        println!("cargo:warning=[contract] UNALLOWED GAP: {g}");
    }
    panic!(
        "[contract] AllImplemented: {} gap(s). Fix bindings or update status.",
        gaps.len()
    );
}
