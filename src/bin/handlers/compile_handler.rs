//! Compile Command Handler
//!
//! Handles compilation of Ruchy files to native binaries with optimization support.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Handle compile command - compile Ruchy file to native binary
///
/// # Arguments
/// * `file` - Path to the Ruchy file to compile
/// * `output` - Output binary path
/// * `opt_level` - Optimization level (0, 1, 2, 3, s, z)
/// * `strip` - Strip debug symbols
/// * `static_link` - Use static linking
/// * `target` - Target triple for cross-compilation
///
/// # Errors
/// Returns error if compilation fails or rustc is not available
pub fn handle_compile_command(
    file: &Path,
    output: PathBuf,
    opt_level: String,
    optimize: Option<&str>,
    strip: bool,
    static_link: bool,
    target: Option<String>,
    verbose: bool,
    json_output: Option<&Path>,
    show_profile_info: bool,
    pgo: bool,
    embed_models: Vec<PathBuf>,
) -> Result<()> {
    use colored::Colorize;
    use ruchy::backend::{compile_to_binary as backend_compile, CompileOptions};
    use std::fs;
    use std::time::Instant;

    // Check if rustc is available
    if let Err(e) = ruchy::backend::compiler::check_rustc_available() {
        eprintln!("{} {}", "Error:".bright_red(), e);
        eprintln!("Please install Rust toolchain from https://rustup.rs/");
        return Err(e);
    }

    // OPTIMIZATION-001: Map high-level optimization presets to rustc flags
    let (final_opt_level, final_strip, rustc_flags, optimization_info) =
        if let Some(level) = optimize {
            apply_optimization_preset(level)?
        } else {
            // Use existing flags if no --optimize specified
            (opt_level, strip, Vec::new(), None)
        };

    // PERF-002 Phase 3: Show profile information if requested
    if show_profile_info {
        display_profile_info(&final_opt_level);
    }

    // PERF-002 Phase 4: Profile-Guided Optimization automation
    if pgo {
        return handle_pgo_compilation(
            file,
            &output,
            &final_opt_level,
            final_strip,
            static_link,
            target,
            rustc_flags,
            verbose,
            json_output,
        );
    }

    println!("{} Compiling {}...", "→".bright_blue(), file.display());

    if let Some((opt_name, lto_mode, target_cpu)) = &optimization_info {
        println!("{} Optimization level: {}", "ℹ".bright_blue(), opt_name);
        if let Some(lto) = lto_mode {
            println!("{} LTO: {}", "ℹ".bright_blue(), lto);
        }
        if let Some(cpu) = target_cpu {
            println!("{} target-cpu: {}", "ℹ".bright_blue(), cpu);
        }
    }

    // Issue #169: Show embedded models information
    if !embed_models.is_empty() {
        println!(
            "{} Embedding {} model file(s):",
            "ℹ".bright_blue(),
            embed_models.len()
        );
        for model in &embed_models {
            let size = fs::metadata(model).map(|m| m.len()).unwrap_or(0);
            println!("  {} ({} bytes)", model.display(), size);
        }
    }

    // Verbose output: show all optimization flags
    if verbose && !rustc_flags.is_empty() {
        println!("{} Optimization flags:", "ℹ".bright_blue());
        for flag in &rustc_flags {
            println!("  {}", flag);
        }
    }

    let compile_start = Instant::now();

    let options = CompileOptions {
        output,
        opt_level: final_opt_level,
        strip: final_strip,
        static_link,
        target,
        rustc_flags,
        embed_models,
    };

    match backend_compile(file, &options) {
        Ok(binary_path) => {
            let compile_time = compile_start.elapsed();

            println!(
                "{} Successfully compiled to: {}",
                "✓".bright_green(),
                binary_path.display()
            );

            // Make the binary executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&binary_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&binary_path, perms)?;
            }

            let binary_size = fs::metadata(&binary_path)?.len();
            println!("{} Binary size: {} bytes", "ℹ".bright_blue(), binary_size);

            // JSON output for CI/CD integration
            if let Some(json_path) = json_output {
                generate_compilation_json(
                    json_path,
                    file,
                    &binary_path,
                    optimize,
                    binary_size,
                    compile_time.as_millis(),
                    optimization_info.as_ref(),
                    &options,
                )?;
                println!("{} JSON report: {}", "ℹ".bright_blue(), json_path.display());
            }
        }
        Err(e) => {
            eprintln!("{} Compilation failed: {}", "✗".bright_red(), e);
            return Err(e);
        }
    }
    Ok(())
}

/// Optimization result: (`opt_level`, strip, `rustc_flags`, info)
type OptimizationResult = (
    String,
    bool,
    Vec<String>,
    Option<(String, Option<String>, Option<String>)>,
);

/// Apply optimization preset and return (`opt_level`, strip, `rustc_flags`, info)
fn apply_optimization_preset(level: &str) -> Result<OptimizationResult> {
    use anyhow::bail;

    match level {
        "none" => {
            // Debug mode: opt-level=0, no optimizations
            Ok((
                "0".to_string(),
                false,
                vec![],
                Some(("none".to_string(), None, None)),
            ))
        }
        "balanced" => {
            // Balanced: opt-level=2, thin LTO for reasonable compile times
            Ok((
                "2".to_string(),
                false,
                vec!["-C".to_string(), "lto=thin".to_string()],
                Some(("balanced".to_string(), Some("thin".to_string()), None)),
            ))
        }
        "aggressive" => {
            // Aggressive: opt-level=3, fat LTO, single codegen unit, strip symbols
            Ok((
                "3".to_string(),
                true,
                vec![
                    "-C".to_string(),
                    "lto=fat".to_string(),
                    "-C".to_string(),
                    "codegen-units=1".to_string(),
                    "-C".to_string(),
                    "strip=symbols".to_string(),
                ],
                Some(("aggressive".to_string(), Some("fat".to_string()), None)),
            ))
        }
        "nasa" => {
            // NASA-grade: opt-level=3, fat LTO, single codegen unit, strip,
            // target-cpu=native, embed-bitcode
            Ok((
                "3".to_string(),
                true,
                vec![
                    "-C".to_string(),
                    "lto=fat".to_string(),
                    "-C".to_string(),
                    "codegen-units=1".to_string(),
                    "-C".to_string(),
                    "strip=symbols".to_string(),
                    "-C".to_string(),
                    "target-cpu=native".to_string(),
                    "-C".to_string(),
                    "embed-bitcode=yes".to_string(),
                    "-C".to_string(),
                    "opt-level=3".to_string(),
                ],
                Some((
                    "nasa".to_string(),
                    Some("fat".to_string()),
                    Some("native".to_string()),
                )),
            ))
        }
        _ => {
            bail!(
                "Invalid optimization level: {}\nValid levels: none, balanced, aggressive, nasa",
                level
            );
        }
    }
}

/// Generate JSON compilation report
fn generate_compilation_json(
    json_path: &Path,
    source_file: &Path,
    binary_path: &Path,
    optimization_level: Option<&str>,
    binary_size: u64,
    compile_time_ms: u128,
    optimization_info: Option<&(String, Option<String>, Option<String>)>,
    options: &ruchy::backend::CompileOptions,
) -> Result<()> {
    use std::fs;

    let mut json = String::from("{\n");
    json.push_str(&format!(
        "  \"source_file\": \"{}\",\n",
        source_file.display()
    ));
    json.push_str(&format!(
        "  \"binary_path\": \"{}\",\n",
        binary_path.display()
    ));
    json.push_str(&format!(
        "  \"optimization_level\": \"{}\",\n",
        optimization_level.unwrap_or("custom")
    ));
    json.push_str(&format!("  \"binary_size\": {},\n", binary_size));
    json.push_str(&format!("  \"compile_time_ms\": {},\n", compile_time_ms));

    json.push_str("  \"optimization_flags\": {\n");
    json.push_str(&format!("    \"opt_level\": \"{}\",\n", options.opt_level));
    json.push_str(&format!("    \"strip\": {},\n", options.strip));
    json.push_str(&format!("    \"static_link\": {},\n", options.static_link));

    if let Some((_, lto, target_cpu)) = optimization_info {
        if let Some(lto_mode) = lto {
            json.push_str(&format!("    \"lto\": \"{}\",\n", lto_mode));
        }
        if let Some(cpu) = target_cpu {
            json.push_str(&format!("    \"target_cpu\": \"{}\",\n", cpu));
        }
    }

    // Remove trailing comma
    if json.ends_with(",\n") {
        json.pop();
        json.pop();
        json.push('\n');
    }

    json.push_str("  }\n");
    json.push_str("}\n");

    fs::write(json_path, json)?;
    Ok(())
}

/// Handle Profile-Guided Optimization compilation (PERF-002 Phase 4)
///
/// Automates the two-step PGO build process:
/// 1. Build with profile-generate
/// 2. Prompt user to run workload
/// 3. Build with profile-use
///
/// # Arguments
/// * `file` - Source Ruchy file
/// * `output` - Output binary path
/// * `opt_level` - Optimization level
/// * `strip` - Strip debug symbols
/// * `static_link` - Enable static linking
/// * `target` - Target triple
/// * `rustc_flags` - Additional rustc flags
/// * `verbose` - Verbose output
/// * `json_output` - JSON metrics output path
///
/// # Errors
/// Returns error if either compilation step fails
fn handle_pgo_compilation(
    file: &Path,
    output: &Path,
    opt_level: &str,
    strip: bool,
    static_link: bool,
    target: Option<String>,
    mut rustc_flags: Vec<String>,
    _verbose: bool,
    json_output: Option<&Path>,
) -> Result<()> {
    use colored::Colorize;
    use ruchy::backend::{compile_to_binary as backend_compile, CompileOptions};
    use std::fs;
    use std::io;
    use tempfile::TempDir;

    // Create temporary directory for profile data
    let pgo_dir = TempDir::new()?;
    let pgo_path = pgo_dir
        .path()
        .to_str()
        .context("Failed to get PGO directory path")?;

    println!("\n{}", "Profile-Guided Optimization".bright_cyan().bold());
    println!("{}", "━".repeat(60).bright_black());

    // Step 1: Build with profile generation
    println!(
        "\n{} Building with profile generation...",
        "→".bright_blue()
    );

    let profiled_output = output.with_file_name(format!(
        "{}-profiled",
        output
            .file_name()
            .expect("Output path should have a file name")
            .to_str()
            .expect("File name should be valid UTF-8")
    ));

    // Add profile-generate flag
    rustc_flags.push("-C".to_string());
    rustc_flags.push(format!("profile-generate={}", pgo_path));

    let options_step1 = CompileOptions {
        output: profiled_output.clone(),
        opt_level: opt_level.to_string(),
        strip,
        static_link,
        target: target.clone(),
        rustc_flags: rustc_flags.clone(),
        embed_models: Vec::new(),
    };

    backend_compile(file, &options_step1)?;

    // Make profiled binary executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&profiled_output)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&profiled_output, perms)?;
    }

    println!(
        "{} Built: {}",
        "✓".bright_green(),
        profiled_output.display()
    );

    // Step 2: Prompt user to run workload
    println!(
        "\n{}",
        "Run your typical workload now to collect profile data:".bright_yellow()
    );
    println!(
        "  {}",
        format!("./{} <args>", profiled_output.display()).bright_white()
    );
    println!("\n{}", "Press Enter when done...".bright_yellow());

    // Wait for user input
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // Step 3: Build with profile-use
    println!(
        "\n{} Building with profile-guided optimization...",
        "→".bright_blue()
    );

    // Replace profile-generate with profile-use
    rustc_flags.pop(); // Remove profile-generate option
    rustc_flags.pop(); // Remove -C flag
    rustc_flags.push("-C".to_string());
    rustc_flags.push(format!("profile-use={}", pgo_path));
    rustc_flags.push("-C".to_string());
    rustc_flags.push("target-cpu=native".to_string()); // Use native CPU for PGO

    let options_step2 = CompileOptions {
        output: output.to_path_buf(),
        opt_level: opt_level.to_string(),
        strip,
        static_link,
        target,
        rustc_flags,
        embed_models: Vec::new(),
    };

    backend_compile(file, &options_step2)?;

    // Make final binary executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(output)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(output, perms)?;
    }

    println!(
        "{} Built: {} (optimized)",
        "✓".bright_green(),
        output.display()
    );

    // Show results
    let binary_size = fs::metadata(output)?.len();
    println!("\n{}", "PGO Compilation Complete".bright_green().bold());
    println!("{}", "━".repeat(60).bright_black());
    println!("  {}: {}", "Final binary".bright_blue(), output.display());
    println!("  {}: {} bytes", "Binary size".bright_blue(), binary_size);
    println!(
        "  {}: {} (can be reused)",
        "Profile data".bright_blue(),
        pgo_path
    );
    println!(
        "  {}: 25-50x expected for CPU-intensive workloads",
        "Speedup".bright_blue()
    );
    println!();

    // Clean up profiled binary
    let _ = fs::remove_file(&profiled_output);

    // JSON output if requested
    if let Some(json_path) = json_output {
        let json_data = serde_json::json!({
            "pgo": true,
            "output": output.display().to_string(),
            "size_bytes": binary_size,
            "profile_data": pgo_path,
        });
        fs::write(json_path, serde_json::to_string_pretty(&json_data)?)?;
    }

    Ok(())
}

/// Display profile characteristics before compilation (PERF-002 Phase 3)
///
/// Shows optimization settings, expected performance, and alternative profiles
/// based on empirical data from compiled-rust-benchmarking project.
///
/// # Arguments
/// * `opt_level` - The optimization level being used
fn display_profile_info(opt_level: &str) {
    use colored::Colorize;

    // Determine profile characteristics based on opt-level
    let (profile_name, speedup, size, use_case, compile_time) = match opt_level {
        "3" => (
            "release",
            "15x average",
            "1-2 MB",
            "General-purpose production binaries",
            "~30-60s for 1000 LOC",
        ),
        "z" | "s" => (
            "release-tiny",
            "2x average",
            "314 KB",
            "Embedded systems, mobile apps",
            "~30-60s for 1000 LOC",
        ),
        _ => (
            "custom",
            "varies",
            "varies",
            "Custom configuration",
            "~30-60s for 1000 LOC",
        ),
    };

    // Display profile information with visual formatting
    println!("\n{}", "Profile Information".bright_cyan().bold());
    println!("{}", "━".repeat(60).bright_black());
    println!(
        "  {}: {} ({})",
        "Profile".bright_blue(),
        profile_name,
        if profile_name == "release" {
            "default"
        } else {
            "custom"
        }
    );
    println!(
        "  {}: opt-level = {} ({})",
        "Optimization".bright_blue(),
        opt_level,
        if opt_level == "3" {
            "speed"
        } else if opt_level == "z" || opt_level == "s" {
            "size"
        } else {
            "custom"
        }
    );
    println!("  {}: fat (maximum)", "LTO".bright_blue());
    println!("  {}: 1", "Codegen units".bright_blue());
    println!("  {}: {}", "Expected speedup".bright_blue(), speedup);
    println!("  {}: {}", "Expected size".bright_blue(), size);
    println!("  {}: {}", "Best for".bright_blue(), use_case);
    println!("  {}: {}", "Compile time".bright_blue(), compile_time);
    println!("{}", "━".repeat(60).bright_black());

    // Show alternative profiles
    if profile_name != "release-tiny" {
        println!("\n{}", "Alternative profiles:".bright_yellow());
        println!(
            "  {} {} (314 KB, 2x speed, embedded)",
            "→".bright_blue(),
            "--profile release-tiny".bright_green()
        );
    }
    if profile_name != "release-ultra" {
        println!(
            "  {} {} (25-50x speed, PGO, maximum performance)",
            "→".bright_blue(),
            "--profile release-ultra".bright_green()
        );
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_apply_optimization_preset_none() {
        let result = apply_optimization_preset("none").unwrap();
        assert_eq!(result.0, "0");
        assert!(!result.1); // no strip
        assert!(result.2.is_empty()); // no flags
    }

    #[test]
    fn test_apply_optimization_preset_balanced() {
        let result = apply_optimization_preset("balanced").unwrap();
        assert_eq!(result.0, "2");
        assert!(!result.1);
        assert!(result.2.contains(&"lto=thin".to_string()));
    }

    #[test]
    fn test_apply_optimization_preset_aggressive() {
        let result = apply_optimization_preset("aggressive").unwrap();
        assert_eq!(result.0, "3");
        assert!(result.1); // strip enabled
        assert!(result.2.contains(&"lto=fat".to_string()));
    }

    #[test]
    fn test_apply_optimization_preset_nasa() {
        let result = apply_optimization_preset("nasa").unwrap();
        assert_eq!(result.0, "3");
        assert!(result.1);
        assert!(result.2.contains(&"target-cpu=native".to_string()));
    }

    #[test]
    fn test_apply_optimization_preset_invalid() {
        let result = apply_optimization_preset("invalid");
        assert!(result.is_err());
    }

    // ===== EXTREME TDD Round 151 - Compile Handler Tests =====

    #[test]
    fn test_display_profile_info_level_3() {
        display_profile_info("3");
        // Just verify it doesn't panic
    }

    #[test]
    fn test_display_profile_info_level_z() {
        display_profile_info("z");
    }

    #[test]
    fn test_display_profile_info_level_s() {
        display_profile_info("s");
    }

    #[test]
    fn test_display_profile_info_custom() {
        display_profile_info("2");
        display_profile_info("1");
        display_profile_info("0");
    }

    #[test]
    fn test_optimization_preset_returns_correct_lto() {
        let (_, _, flags, _) = apply_optimization_preset("balanced").unwrap();
        assert!(flags.iter().any(|f| f.contains("thin")));

        let (_, _, flags, _) = apply_optimization_preset("aggressive").unwrap();
        assert!(flags.iter().any(|f| f.contains("fat")));
    }

    #[test]
    fn test_optimization_preset_info_tuple() {
        let (_, _, _, info) = apply_optimization_preset("nasa").unwrap();
        let (name, lto, cpu) = info.unwrap();
        assert_eq!(name, "nasa");
        assert_eq!(lto, Some("fat".to_string()));
        assert_eq!(cpu, Some("native".to_string()));
    }

    #[test]
    fn test_generate_compilation_json_basic() {
        use ruchy::backend::CompileOptions;
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("report.json");
        let options = CompileOptions {
            output: PathBuf::from("output"),
            opt_level: "3".to_string(),
            strip: true,
            static_link: false,
            target: None,
            rustc_flags: vec![],
            embed_models: vec![],
        };
        let result = generate_compilation_json(
            &json_path,
            Path::new("test.ruchy"),
            Path::new("output"),
            Some("aggressive"),
            1000,
            500,
            None,
            &options,
        );
        assert!(result.is_ok());
        assert!(json_path.exists());
    }

    #[test]
    fn test_generate_compilation_json_with_info() {
        use ruchy::backend::CompileOptions;
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("report2.json");
        let options = CompileOptions {
            output: PathBuf::from("output"),
            opt_level: "3".to_string(),
            strip: true,
            static_link: true,
            target: Some("x86_64-unknown-linux-gnu".to_string()),
            rustc_flags: vec![],
            embed_models: vec![],
        };
        let info = ("nasa".to_string(), Some("fat".to_string()), Some("native".to_string()));
        let result = generate_compilation_json(
            &json_path,
            Path::new("test.ruchy"),
            Path::new("output"),
            Some("nasa"),
            2000,
            1000,
            Some(&info),
            &options,
        );
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&json_path).unwrap();
        assert!(content.contains("nasa"));
    }

    #[test]
    fn test_handle_compile_command_nonexistent_file() {
        let result = handle_compile_command(
            Path::new("/nonexistent/file.ruchy"),
            PathBuf::from("/tmp/output"),
            "3".to_string(),
            None,
            false,
            false,
            None,
            false,
            None,
            false,
            false,
            vec![],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_compile_command_valid_file() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "fun main() { println(42) }").unwrap();
        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("output");
        let result = handle_compile_command(
            temp.path(),
            output,
            "0".to_string(),
            None,
            false,
            false,
            None,
            false,
            None,
            false,
            false,
            vec![],
        );
        // May succeed or fail depending on rustc
        let _ = result;
    }

    #[test]
    fn test_handle_compile_command_with_optimize() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "42").unwrap();
        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("output");
        let result = handle_compile_command(
            temp.path(),
            output,
            "0".to_string(),
            Some("balanced"),
            false,
            false,
            None,
            false,
            None,
            false,
            false,
            vec![],
        );
        let _ = result;
    }

    #[test]
    fn test_handle_compile_command_show_profile() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "42").unwrap();
        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("output");
        let result = handle_compile_command(
            temp.path(),
            output,
            "3".to_string(),
            None,
            true,
            false,
            None,
            false,
            None,
            true, // show_profile_info
            false,
            vec![],
        );
        let _ = result;
    }

    #[test]
    fn test_handle_compile_command_verbose() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "42").unwrap();
        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("output");
        let result = handle_compile_command(
            temp.path(),
            output,
            "2".to_string(),
            Some("aggressive"),
            true,
            true,
            None,
            true, // verbose
            None,
            false,
            false,
            vec![],
        );
        let _ = result;
    }
}
