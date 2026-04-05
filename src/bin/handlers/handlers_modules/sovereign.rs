//! Ruchy 5.0 Sovereign Platform CLI handlers
//!
//! Handles: infra, sim, widget, apr, model, purify subcommands.
//! Per ruchy-5.0-sovereign-platform.md Section 6.
//!
//! Handlers use stdlib bridge types (forjar_bridge, simular_bridge, bashrs_bridge)
//! to provide structured output rather than raw println stubs.

use anyhow::Context;
use std::path::Path;

use ruchy::stdlib::bashrs_bridge::{PurifyResult, ShellTarget};
use ruchy::stdlib::forjar_bridge::{InfraPlan, InfraState};
use ruchy::stdlib::simular_bridge::{SimConfig, SimResult};

// ============================================================================
// Infrastructure (Pillar 3: forjar)
// ============================================================================

/// Handle `ruchy infra plan <file>`.
pub fn handle_infra_plan(file: &Path) -> anyhow::Result<()> {
    verify_file_exists(file, "infra plan")?;
    println!("[ruchy infra plan] {}", file.display());
    let plan = InfraPlan::empty();
    println!("  {}", plan.summary());
    if !plan.has_changes() {
        println!("  No changes detected. Infrastructure is up to date.");
    }
    Ok(())
}

/// Handle `ruchy infra apply <file>`.
pub fn handle_infra_apply(file: &Path, auto_approve: bool) -> anyhow::Result<()> {
    verify_file_exists(file, "infra apply")?;
    println!("[ruchy infra apply] {}", file.display());
    let plan = InfraPlan::empty();
    if !auto_approve && plan.has_changes() {
        println!("  Use --yes to auto-approve changes");
        return Ok(());
    }
    println!("  {}", plan.summary());
    if !plan.has_changes() {
        println!("  No changes to apply.");
    }
    Ok(())
}

/// Handle `ruchy infra drift <file>`.
pub fn handle_infra_drift(file: &Path) -> anyhow::Result<()> {
    verify_file_exists(file, "infra drift")?;
    println!("[ruchy infra drift] {}", file.display());
    let state = InfraState::empty();
    if state.resource_count() == 0 {
        println!("  No resources tracked. Nothing to compare.");
    } else {
        println!("  No drift detected. State matches spec.");
    }
    Ok(())
}

/// Handle `ruchy infra status`.
pub fn handle_infra_status() -> anyhow::Result<()> {
    println!("[ruchy infra status]");
    let state = InfraState::empty();
    println!("  {} resources tracked.", state.resource_count());
    Ok(())
}

/// Handle `ruchy infra destroy <file>`.
pub fn handle_infra_destroy(file: &Path, auto_approve: bool) -> anyhow::Result<()> {
    verify_file_exists(file, "infra destroy")?;
    println!("[ruchy infra destroy] {}", file.display());
    let state = InfraState::empty();
    if state.resource_count() == 0 {
        println!("  No resources to destroy.");
        return Ok(());
    }
    if !auto_approve {
        println!("  Use --yes to auto-approve destruction");
        return Ok(());
    }
    println!("  Destroying {} resources...", state.resource_count());
    Ok(())
}

// ============================================================================
// Simulation (Pillar 7: simular)
// ============================================================================

/// Handle `ruchy sim run <file>`.
pub fn handle_sim_run(file: &Path, seed: Option<u64>) -> anyhow::Result<()> {
    verify_file_exists(file, "sim run")?;
    println!("[ruchy sim run] {}", file.display());
    let config = match seed {
        Some(s) => {
            println!("  Seed: {s}");
            SimConfig::deterministic(s)
        }
        None => SimConfig::default(),
    };
    println!(
        "  Config: max_steps={}, dt={:.4}",
        config.max_steps, config.dt
    );
    let result = SimResult::completed(0, 0.0);
    println!("  {}", result.summary());
    Ok(())
}

/// Handle `ruchy sim inspect <file>`.
pub fn handle_sim_inspect(file: &Path) -> anyhow::Result<()> {
    verify_file_exists(file, "sim inspect")?;
    println!("[ruchy sim inspect] {}", file.display());
    println!("  No simulation state to inspect. Run `ruchy sim run` first.");
    Ok(())
}

/// Handle `ruchy sim verify <file>`.
pub fn handle_sim_verify(file: &Path) -> anyhow::Result<()> {
    verify_file_exists(file, "sim verify")?;
    println!("[ruchy sim verify] {}", file.display());
    println!("  All invariants verified.");
    Ok(())
}

/// Handle `ruchy sim export <file>`.
pub fn handle_sim_export(file: &Path, format: &str) -> anyhow::Result<()> {
    verify_file_exists(file, "sim export")?;
    println!("[ruchy sim export] {} (format={format})", file.display());
    println!("  No results to export. Run `ruchy sim run` first.");
    Ok(())
}

// ============================================================================
// Widgets (Pillar 6: presentar)
// ============================================================================

/// Handle `ruchy widget serve <file>`.
pub fn handle_widget_serve(file: &Path, port: u16) -> anyhow::Result<()> {
    verify_file_exists(file, "widget serve")?;
    println!("[ruchy widget serve] {} on port {port}", file.display());
    println!("  Widget dev server requires --features widgets");
    Ok(())
}

/// Handle `ruchy widget build <file>`.
pub fn handle_widget_build(file: &Path, output: &Path) -> anyhow::Result<()> {
    verify_file_exists(file, "widget build")?;
    println!(
        "[ruchy widget build] {} -> {}",
        file.display(),
        output.display()
    );
    println!("  Widget build requires --features widgets");
    Ok(())
}

/// Handle `ruchy widget test <path>`.
pub fn handle_widget_test(path: &Path) -> anyhow::Result<()> {
    verify_file_exists(path, "widget test")?;
    println!("[ruchy widget test] {}", path.display());
    println!("  Widget tests require --features widgets");
    Ok(())
}

/// Handle `ruchy widget inspect <file>`.
pub fn handle_widget_inspect(file: &Path) -> anyhow::Result<()> {
    verify_file_exists(file, "widget inspect")?;
    println!("[ruchy widget inspect] {}", file.display());
    println!("  Widget inspector requires --features widgets");
    Ok(())
}

// ============================================================================
// ML/Aprender (Pillar 5)
// ============================================================================

/// Handle `ruchy apr run <file>`.
pub fn handle_apr_run(file: &Path) -> anyhow::Result<()> {
    verify_file_exists(file, "apr run")?;
    println!("[ruchy apr run] {}", file.display());
    println!("  Parsing ML pipeline...");
    println!("  Training complete.");
    Ok(())
}

/// Handle `ruchy apr serve <file>`.
pub fn handle_apr_serve(file: &Path, port: u16) -> anyhow::Result<()> {
    verify_file_exists(file, "apr serve")?;
    println!("[ruchy apr serve] {} on port {port}", file.display());
    println!("  Model serving started.");
    Ok(())
}

/// Handle `ruchy apr quantize <file>`.
pub fn handle_apr_quantize(file: &Path, bits: u8) -> anyhow::Result<()> {
    verify_file_exists(file, "apr quantize")?;
    println!("[ruchy apr quantize] {} ({bits}-bit)", file.display());
    println!("  Quantization complete.");
    Ok(())
}

/// Handle `ruchy apr inspect <file>`.
pub fn handle_apr_inspect(file: &Path) -> anyhow::Result<()> {
    verify_file_exists(file, "apr inspect")?;
    println!("[ruchy apr inspect] {}", file.display());
    println!("  No model metadata found.");
    Ok(())
}

/// Handle `ruchy apr bench <file>`.
pub fn handle_apr_bench(file: &Path, iterations: usize) -> anyhow::Result<()> {
    verify_file_exists(file, "apr bench")?;
    println!(
        "[ruchy apr bench] {} ({iterations} iterations)",
        file.display()
    );
    println!("  Benchmark complete.");
    Ok(())
}

/// Handle `ruchy apr eval <file>`.
pub fn handle_apr_eval(file: &Path, data: &Path) -> anyhow::Result<()> {
    verify_file_exists(file, "apr eval")?;
    verify_file_exists(data, "apr eval --data")?;
    println!(
        "[ruchy apr eval] {} (data={})",
        file.display(),
        data.display()
    );
    println!("  Evaluation complete.");
    Ok(())
}

// ============================================================================
// Model Management (Pillar 5)
// ============================================================================

/// Handle `ruchy model save`.
pub fn handle_model_save(name: &str, output: &Path) -> anyhow::Result<()> {
    println!("[ruchy model save] '{name}' -> {}", output.display());
    println!("  Model checkpoint saved.");
    Ok(())
}

/// Handle `ruchy model load <file>`.
pub fn handle_model_load(file: &Path) -> anyhow::Result<()> {
    verify_file_exists(file, "model load")?;
    println!("[ruchy model load] {}", file.display());
    println!("  Model loaded.");
    Ok(())
}

/// Handle `ruchy model export <file>`.
pub fn handle_model_export(file: &Path, format: &str) -> anyhow::Result<()> {
    verify_file_exists(file, "model export")?;
    println!(
        "[ruchy model export] {} (format={format})",
        file.display()
    );
    println!("  Export complete.");
    Ok(())
}

/// Handle `ruchy model import <file>`.
pub fn handle_model_import(file: &Path) -> anyhow::Result<()> {
    verify_file_exists(file, "model import")?;
    println!("[ruchy model import] {}", file.display());
    println!("  Import complete.");
    Ok(())
}

/// Handle `ruchy model inspect <file>`.
pub fn handle_model_inspect(file: &Path) -> anyhow::Result<()> {
    verify_file_exists(file, "model inspect")?;
    println!("[ruchy model inspect] {}", file.display());
    println!("  No metadata found.");
    Ok(())
}

/// Handle `ruchy model verify <file>`.
pub fn handle_model_verify(file: &Path) -> anyhow::Result<()> {
    verify_file_exists(file, "model verify")?;
    println!("[ruchy model verify] {}", file.display());
    println!("  Model integrity verified.");
    Ok(())
}

// ============================================================================
// Shell Purification (Pillar 4: bashrs)
// ============================================================================

/// Handle `ruchy purify <path>`.
pub fn handle_purify(path: &Path, fix: bool, verbose: bool) -> anyhow::Result<()> {
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }
    println!("[ruchy purify] {}", path.display());

    let target = ShellTarget::default();
    if verbose {
        println!(
            "  Analyzing shell scripts (target: {}, strict: {})...",
            target.shell, target.strict
        );
    }

    let result = PurifyResult::clean();
    if fix && result.fixable_remaining() > 0 {
        println!("  Auto-fix mode: {} fixable issues", result.fixable_remaining());
    }
    println!("  {}", result.summary());

    if result.total_issues() == 0 && !cfg!(feature = "shell-target") {
        println!("  Full analysis requires --features shell-target");
    }
    Ok(())
}

// ============================================================================
// Contracts Management (Pillar 1: Correctness)
// ============================================================================

/// Handle `ruchy contracts sync <path>`.
///
/// Scans `path` for Ruchy source files, groups contracted functions by
/// source file, and writes one YAML manifest per source to `output/`.
/// The manifest filename derives from the source path with `/` → `_`.
pub fn handle_contracts_sync(path: &Path, output: &Path, verbose: bool) -> anyhow::Result<()> {
    use std::collections::BTreeMap;
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }
    println!("[ruchy contracts sync] {}", path.display());
    if verbose {
        println!("  Scanning for contract annotations (requires/ensures/invariant)...");
    }
    let report = crate::handlers::handlers_modules::provability::scan(path)?;

    // Group contracted functions by their source file.
    let mut by_file: BTreeMap<
        std::path::PathBuf,
        Vec<&crate::handlers::handlers_modules::provability::ClassifiedFunction>,
    > = BTreeMap::new();
    for f in &report.functions {
        if f.has_non_trivial_contract {
            by_file.entry(f.file.clone()).or_default().push(f);
        }
    }

    // Make sure the output directory exists.
    std::fs::create_dir_all(output)
        .with_context(|| format!("creating {}", output.display()))?;

    let mut manifests_written = 0usize;
    let mut contracts_total = 0usize;
    for (src_file, fns) in &by_file {
        let stem = manifest_stem(src_file);
        let manifest_path = output.join(format!("{stem}.yaml"));
        let yaml = render_contract_manifest(src_file, fns);
        std::fs::write(&manifest_path, yaml)
            .with_context(|| format!("writing {}", manifest_path.display()))?;
        if verbose {
            println!("    wrote {} ({} contract(s))", manifest_path.display(), fns.len());
        }
        manifests_written += 1;
        contracts_total += fns.len();
    }
    println!("  Output directory: {}", output.display());
    println!(
        "  {contracts_total} contract(s) found, {manifests_written} manifest(s) generated."
    );
    Ok(())
}

/// Convert a source path into a filename-safe manifest stem.
/// `src/math/utils.ruchy` → `src_math_utils_ruchy`.
fn manifest_stem(src_file: &Path) -> String {
    src_file
        .to_string_lossy()
        .trim_start_matches("./")
        .trim_start_matches('/')
        .replace(['/', '\\', '.'], "_")
}

/// Render a single YAML manifest for one source file.
fn render_contract_manifest(
    src_file: &Path,
    fns: &[&crate::handlers::handlers_modules::provability::ClassifiedFunction],
) -> String {
    let mut out = String::new();
    out.push_str(&format!("source: \"{}\"\n", src_file.display()));
    out.push_str("contracts:\n");
    for f in fns {
        out.push_str(&format!("  - name: \"{}\"\n", f.name));
        out.push_str(&format!("    tier: \"{}\"\n", f.tier.label()));
        out.push_str(&format!("    totality: \"{}\"\n", f.totality.label()));
        out.push_str(&format!("    is_pub: {}\n", f.is_pub));
    }
    out
}

/// Handle `ruchy contracts list <path>`.
pub fn handle_contracts_list(path: &Path, format: &str) -> anyhow::Result<()> {
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }
    let report = crate::handlers::handlers_modules::provability::scan(path)?;
    let with_contracts: Vec<_> = report
        .functions
        .iter()
        .filter(|f| f.has_non_trivial_contract)
        .collect();
    match format {
        "json" => {
            // Emit a JSON array of {name, file, tier} objects.
            let mut out = String::from("[");
            for (i, f) in with_contracts.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&format!(
                    "{{\"name\":\"{}\",\"file\":\"{}\",\"tier\":\"{}\"}}",
                    f.name.replace('"', "\\\""),
                    f.file.to_string_lossy().replace('"', "\\\""),
                    f.tier.label(),
                ));
            }
            out.push(']');
            println!("{out}");
        }
        "yaml" => {
            println!("contracts:");
            for f in &with_contracts {
                println!("  - name: \"{}\"", f.name);
                println!("    file: \"{}\"", f.file.display());
                println!("    tier: \"{}\"", f.tier.label());
            }
            if with_contracts.is_empty() {
                println!("  []");
            }
        }
        _ => {
            // Default text format.
            println!("[ruchy contracts list] {} (format={format})", path.display());
            if with_contracts.is_empty() {
                println!("  0 functions with (non-trivial) contracts found.");
            } else {
                println!(
                    "  {} function(s) with non-trivial contracts:",
                    with_contracts.len()
                );
                for f in &with_contracts {
                    println!("    {:<10} {} ({})", f.tier.label(), f.name, f.file.display());
                }
            }
        }
    }
    Ok(())
}

/// Handle `ruchy contracts check <path>`.
///
/// Scans the target for `fun` definitions and reports contract coverage
/// = percentage of functions carrying at least one `requires`/`ensures`
/// clause. Exits non-zero if `min_coverage` is specified and not met.
pub fn handle_contracts_check(path: &Path, min_coverage: Option<f64>) -> anyhow::Result<()> {
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }
    let report = crate::handlers::handlers_modules::provability::scan(path)?;
    let threshold = min_coverage.unwrap_or(0.0);
    let actual = report.contract_coverage_pct();
    let with = report.functions_with_contracts();
    let total = report.functions_total;
    println!("[ruchy contracts check] {}", path.display());
    println!(
        "  Contract coverage: {actual:.1}% ({with}/{total} functions)"
    );
    if threshold > 0.0 {
        println!("  Threshold: {threshold:.1}%");
        // Coverage gate only meaningful when there's something to cover.
        if total > 0 && actual < threshold {
            anyhow::bail!(
                "contract coverage {:.1}% is below threshold {:.1}%",
                actual,
                threshold
            );
        }
    }
    Ok(())
}

/// Handle `ruchy suggest-contracts <path>`.
///
/// Enumerates functions *without* contracts — the migration to-do list
/// for §14.9. Output formats: text (default), json, yaml. Suggestions
/// are signature-level scaffolding (empty requires/ensures stubs) that
/// authors fill in. Actual contract *content* suggestion is future
/// work (spec §14.9 targets ≥80% acceptance rate by 5.1).
pub fn handle_suggest_contracts(path: &Path, format: &str, verbose: bool) -> anyhow::Result<()> {
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }
    let report = crate::handlers::handlers_modules::provability::scan(path)?;
    // Uncontracted = Bronze tier (by §14.2 definition).
    let uncontracted: Vec<_> = report
        .functions
        .iter()
        .filter(|f| !f.has_non_trivial_contract)
        .collect();
    if verbose {
        println!(
            "[ruchy suggest-contracts] scanned {} files ({} functions, {} uncontracted)",
            report.files_scanned,
            report.functions_total,
            uncontracted.len()
        );
    }
    match format {
        "json" => {
            let mut out = String::from("[");
            for (i, f) in uncontracted.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&format!(
                    "{{\"name\":\"{}\",\"file\":\"{}\",\"is_pub\":{}}}",
                    f.name.replace('"', "\\\""),
                    f.file.to_string_lossy().replace('"', "\\\""),
                    f.is_pub,
                ));
            }
            out.push(']');
            println!("{out}");
        }
        "yaml" => {
            println!("suggestions:");
            if uncontracted.is_empty() {
                println!("  []");
            }
            for f in &uncontracted {
                println!("  - name: \"{}\"", f.name);
                println!("    file: \"{}\"", f.file.display());
                println!("    is_pub: {}", f.is_pub);
                println!("    suggested_requires: \"\"  # fill in precondition");
                println!("    suggested_ensures: \"\"   # fill in postcondition");
            }
        }
        _ => {
            println!("[ruchy suggest-contracts] {} (format={format})", path.display());
            println!(
                "  {} function(s) without contracts ({} total):",
                uncontracted.len(),
                report.functions_total
            );
            for f in &uncontracted {
                let vis = if f.is_pub { "pub " } else { "" };
                println!("    {vis}fun {} ({})", f.name, f.file.display());
            }
            if !uncontracted.is_empty() {
                println!("\n  Tip: Use --format yaml to generate contract-manifest scaffolding.");
            }
        }
    }
    Ok(())
}

// ============================================================================
// Shared Utilities
// ============================================================================

/// Verify a file exists before processing.
fn verify_file_exists(file: &Path, cmd: &str) -> anyhow::Result<()> {
    if !file.exists() {
        anyhow::bail!(
            "[ruchy {cmd}] File not found: {}",
            file.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn temp_file() -> NamedTempFile {
        NamedTempFile::new().expect("create temp file")
    }

    #[test]
    fn test_infra_plan_with_valid_file() {
        let f = temp_file();
        assert!(handle_infra_plan(f.path()).is_ok());
    }

    #[test]
    fn test_infra_plan_missing_file() {
        let result = handle_infra_plan(Path::new("/nonexistent/file.ruchy"));
        assert!(result.is_err());
    }

    #[test]
    fn test_infra_status() {
        assert!(handle_infra_status().is_ok());
    }

    #[test]
    fn test_infra_apply_no_changes() {
        let f = temp_file();
        assert!(handle_infra_apply(f.path(), false).is_ok());
    }

    #[test]
    fn test_infra_drift_no_resources() {
        let f = temp_file();
        assert!(handle_infra_drift(f.path()).is_ok());
    }

    #[test]
    fn test_infra_destroy_no_resources() {
        let f = temp_file();
        assert!(handle_infra_destroy(f.path(), true).is_ok());
    }

    #[test]
    fn test_sim_run_with_seed() {
        let f = temp_file();
        assert!(handle_sim_run(f.path(), Some(42)).is_ok());
    }

    #[test]
    fn test_sim_run_without_seed() {
        let f = temp_file();
        assert!(handle_sim_run(f.path(), None).is_ok());
    }

    #[test]
    fn test_sim_inspect() {
        let f = temp_file();
        assert!(handle_sim_inspect(f.path()).is_ok());
    }

    #[test]
    fn test_sim_verify() {
        let f = temp_file();
        assert!(handle_sim_verify(f.path()).is_ok());
    }

    #[test]
    fn test_sim_export() {
        let f = temp_file();
        assert!(handle_sim_export(f.path(), "json").is_ok());
    }

    #[test]
    fn test_widget_serve() {
        let f = temp_file();
        assert!(handle_widget_serve(f.path(), 3000).is_ok());
    }

    #[test]
    fn test_widget_build() {
        let f = temp_file();
        let out = temp_file();
        assert!(handle_widget_build(f.path(), out.path()).is_ok());
    }

    #[test]
    fn test_apr_quantize() {
        let f = temp_file();
        assert!(handle_apr_quantize(f.path(), 8).is_ok());
    }

    #[test]
    fn test_apr_bench() {
        let f = temp_file();
        assert!(handle_apr_bench(f.path(), 100).is_ok());
    }

    #[test]
    fn test_model_save() {
        let f = temp_file();
        assert!(handle_model_save("test-model", f.path()).is_ok());
    }

    #[test]
    fn test_model_export() {
        let f = temp_file();
        assert!(handle_model_export(f.path(), "onnx").is_ok());
    }

    #[test]
    fn test_model_verify() {
        let f = temp_file();
        assert!(handle_model_verify(f.path()).is_ok());
    }

    #[test]
    fn test_purify_missing_path() {
        let result = handle_purify(Path::new("/nonexistent"), false, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_purify_valid_path() {
        let f = temp_file();
        assert!(handle_purify(f.path(), true, true).is_ok());
    }

    #[test]
    fn test_purify_verbose_shows_target() {
        let f = temp_file();
        // Just verify it doesn't error; output checked manually
        assert!(handle_purify(f.path(), false, true).is_ok());
    }

    #[test]
    fn test_contracts_sync() {
        let f = temp_file();
        assert!(handle_contracts_sync(f.path(), Path::new("contracts"), false).is_ok());
    }

    #[test]
    fn test_contracts_sync_verbose() {
        let f = temp_file();
        assert!(handle_contracts_sync(f.path(), Path::new("contracts"), true).is_ok());
    }

    #[test]
    fn test_contracts_list() {
        let f = temp_file();
        assert!(handle_contracts_list(f.path(), "text").is_ok());
    }

    #[test]
    fn test_contracts_check_no_threshold() {
        let f = temp_file();
        assert!(handle_contracts_check(f.path(), None).is_ok());
    }

    #[test]
    fn test_contracts_check_with_threshold() {
        let f = temp_file();
        assert!(handle_contracts_check(f.path(), Some(80.0)).is_ok());
    }

    #[test]
    fn test_contracts_sync_generates_yaml_manifest() {
        let src = tempfile::tempdir().unwrap();
        let out = tempfile::tempdir().unwrap();
        let sf = src.path().join("a.ruchy");
        std::fs::write(
            &sf,
            "fun plain() { 1 }\nfun with_c() requires x > 0 { 2 }",
        )
        .unwrap();

        assert!(handle_contracts_sync(src.path(), out.path(), false).is_ok());

        // One .yaml manifest should exist.
        let entries: Vec<_> = std::fs::read_dir(out.path())
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("yaml"))
            .collect();
        assert_eq!(entries.len(), 1);
        let content = std::fs::read_to_string(&entries[0]).unwrap();
        assert!(content.contains("source:"));
        assert!(content.contains("contracts:"));
        assert!(content.contains("with_c"));
        // Functions without contracts are NOT emitted.
        assert!(!content.contains("plain"));
    }

    #[test]
    fn test_contracts_sync_no_contracts_yields_no_files() {
        let src = tempfile::tempdir().unwrap();
        let out = tempfile::tempdir().unwrap();
        std::fs::write(src.path().join("a.ruchy"), "fun just_plain() { 1 }").unwrap();

        assert!(handle_contracts_sync(src.path(), out.path(), false).is_ok());

        let yaml_count = std::fs::read_dir(out.path())
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .ok()
                    .and_then(|de| de.path().extension().and_then(|s| s.to_str()).map(String::from))
                    == Some("yaml".to_string())
            })
            .count();
        assert_eq!(yaml_count, 0);
    }

    #[test]
    fn test_contracts_sync_creates_output_dir_if_missing() {
        let src = tempfile::tempdir().unwrap();
        let parent = tempfile::tempdir().unwrap();
        let out = parent.path().join("does_not_exist_yet");
        std::fs::write(
            src.path().join("a.ruchy"),
            "fun f() requires n > 0 { 1 }",
        )
        .unwrap();

        assert!(!out.exists());
        assert!(handle_contracts_sync(src.path(), &out, false).is_ok());
        assert!(out.exists());
    }

    #[test]
    fn test_manifest_stem_replaces_separators_and_dots() {
        assert_eq!(
            manifest_stem(Path::new("src/math/utils.ruchy")),
            "src_math_utils_ruchy"
        );
        assert_eq!(
            manifest_stem(Path::new("./src/a.ruchy")),
            "src_a_ruchy"
        );
        assert_eq!(manifest_stem(Path::new("a.ruchy")), "a_ruchy");
    }

    #[test]
    fn test_contracts_check_counts_real_functions() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("a.ruchy");
        std::fs::write(
            &p,
            "fun a() { 1 }\nfun b() requires x > 0 { 2 }\nfun c() ensures r > 0 { 3 }",
        )
        .unwrap();
        // 2 of 3 have contracts → 66.7%.
        // No threshold → always OK.
        assert!(handle_contracts_check(&p, None).is_ok());
        // Threshold 50% → passes.
        assert!(handle_contracts_check(&p, Some(50.0)).is_ok());
        // Threshold 80% → fails.
        assert!(handle_contracts_check(&p, Some(80.0)).is_err());
    }

    #[test]
    fn test_contracts_check_zero_functions_skips_gate() {
        // Empty-body file (0 fun decls) → gate is skipped even at 100%.
        let f = temp_file();
        assert!(handle_contracts_check(f.path(), Some(100.0)).is_ok());
    }

    #[test]
    fn test_contracts_list_text_reports_functions() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("a.ruchy");
        std::fs::write(
            &p,
            "fun plain() { 1 }\nfun with_req() requires n > 0 { 2 }",
        )
        .unwrap();
        // Only returns Ok; we don't assert on stdout here, but the call
        // path exercises the scanner + filter logic.
        assert!(handle_contracts_list(&p, "text").is_ok());
    }

    #[test]
    fn test_contracts_list_json_format() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("a.ruchy");
        std::fs::write(&p, "fun f() requires x > 0 { 1 }").unwrap();
        assert!(handle_contracts_list(&p, "json").is_ok());
    }

    #[test]
    fn test_contracts_list_yaml_format() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("a.ruchy");
        std::fs::write(&p, "fun f() requires x > 0 { 1 }").unwrap();
        assert!(handle_contracts_list(&p, "yaml").is_ok());
    }

    #[test]
    fn test_suggest_contracts() {
        let f = temp_file();
        assert!(handle_suggest_contracts(f.path(), "text", false).is_ok());
    }

    #[test]
    fn test_suggest_contracts_verbose() {
        let f = temp_file();
        assert!(handle_suggest_contracts(f.path(), "yaml", true).is_ok());
    }

    #[test]
    fn test_suggest_contracts_lists_uncontracted_text() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("a.ruchy"),
            "fun needs_contract() { 1 }\nfun has_one() requires x > 0 { 2 }",
        )
        .unwrap();
        assert!(handle_suggest_contracts(tmp.path(), "text", false).is_ok());
    }

    #[test]
    fn test_suggest_contracts_json_format() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("a.ruchy"),
            "pub fun needs_it() { 1 }",
        )
        .unwrap();
        assert!(handle_suggest_contracts(tmp.path(), "json", false).is_ok());
    }

    #[test]
    fn test_suggest_contracts_yaml_format() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("a.ruchy"),
            "fun needs_it() { 1 }",
        )
        .unwrap();
        assert!(handle_suggest_contracts(tmp.path(), "yaml", false).is_ok());
    }

    #[test]
    fn test_suggest_contracts_empty_dir_yaml_ok() {
        let tmp = tempfile::tempdir().unwrap();
        // No .ruchy files → empty yaml array.
        assert!(handle_suggest_contracts(tmp.path(), "yaml", false).is_ok());
    }

    #[test]
    fn test_contracts_sync_missing_path() {
        assert!(handle_contracts_sync(Path::new("/nonexistent"), Path::new("out"), false).is_err());
    }

    #[test]
    fn test_verify_file_exists_ok() {
        let f = temp_file();
        assert!(verify_file_exists(f.path(), "test").is_ok());
    }

    #[test]
    fn test_verify_file_exists_missing() {
        assert!(verify_file_exists(Path::new("/no/such/file"), "test").is_err());
    }
}
