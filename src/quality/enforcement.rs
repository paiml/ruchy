//! Quality gate enforcement implementation for CLI

use std::path::Path;
use anyhow::Result;
use crate::quality::gates::{QualityGateEnforcer, QualityGateConfig};
use crate::quality::scoring::{ScoreEngine, AnalysisDepth};

/// Enforce quality gates on a file or directory
pub fn enforce_quality_gates(
    path: &Path,
    config: Option<&Path>,
    depth: &str,
    fail_fast: bool,
    format: &str,
    export: Option<&Path>,
    ci: bool,
    verbose: bool,
) -> Result<()> {
    // Load configuration
    let project_root = find_project_root(path)?;
    let mut gate_config = if let Some(config_path) = config {
        QualityGateEnforcer::load_config(config_path.parent().unwrap_or(Path::new(".")))?
    } else {
        QualityGateEnforcer::load_config(&project_root)?
    };
    
    // Apply CI mode overrides (stricter thresholds)
    if ci {
        gate_config = apply_ci_overrides(gate_config);
    }
    
    let enforcer = QualityGateEnforcer::new(gate_config);
    
    // Parse analysis depth
    let analysis_depth = match depth {
        "shallow" => AnalysisDepth::Shallow,
        "standard" => AnalysisDepth::Standard,
        "deep" => AnalysisDepth::Deep,
        _ => return Err(anyhow::anyhow!("Invalid depth: {}", depth)),
    };
    
    // Process file or directory
    let mut all_results = Vec::new();
    
    if path.is_file() {
        let result = process_file(&enforcer, path, analysis_depth, verbose)?;
        all_results.push(result);
    } else if path.is_dir() {
        let results = process_directory(&enforcer, path, analysis_depth, fail_fast, verbose)?;
        all_results.extend(results);
    } else {
        return Err(anyhow::anyhow!("Invalid path: {}", path.display()));
    }
    
    // Output results
    match format {
        "console" => print_console_results(&all_results, verbose)?,
        "json" => print_json_results(&all_results)?,
        "junit" => print_junit_results(&all_results)?,
        _ => return Err(anyhow::anyhow!("Invalid format: {}", format)),
    }
    
    // Export CI results if requested
    if let Some(export_path) = export {
        std::fs::create_dir_all(export_path)?;
        enforcer.export_ci_results(&all_results, export_path)?;
        println!("📊 Results exported to {}", export_path.display());
    }
    
    // Check if any gates failed
    let failed_gates = all_results.iter().filter(|r| !r.passed).count();
    
    if failed_gates > 0 {
        eprintln!("❌ {failed_gates} quality gate(s) failed");
        std::process::exit(1);
    } else {
        println!("✅ All quality gates passed!");
    }
    
    Ok(())
}

fn find_project_root(path: &Path) -> Result<std::path::PathBuf> {
    let mut current = if path.is_file() {
        path.parent().unwrap_or(Path::new("."))
    } else {
        path
    };
    
    loop {
        if current.join("Cargo.toml").exists() || current.join(".ruchy").exists() {
            return Ok(current.to_path_buf());
        }
        
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            // Default to current directory
            return Ok(Path::new(".").to_path_buf());
        }
    }
}

fn apply_ci_overrides(mut config: QualityGateConfig) -> QualityGateConfig {
    // Apply stricter thresholds for CI
    config.min_score = config.min_score.max(0.8); // Higher overall score
    config.component_thresholds.correctness = config.component_thresholds.correctness.max(0.9);
    config.component_thresholds.safety = config.component_thresholds.safety.max(0.9);
    config.anti_gaming.min_confidence = config.anti_gaming.min_confidence.max(0.8);
    config.ci_integration.fail_on_violation = true;
    config
}

fn process_file(
    enforcer: &QualityGateEnforcer,
    file_path: &Path,
    depth: AnalysisDepth,
    verbose: bool,
) -> Result<crate::quality::gates::GateResult> {
    if verbose {
        println!("🔍 Analyzing {}", file_path.display());
    }
    
    // Read and parse file
    let content = std::fs::read_to_string(file_path)?;
    let mut parser = crate::Parser::new(&content);
    let ast = parser.parse()?;
    
    // Calculate score
    let score_config = crate::quality::scoring::ScoreConfig::default();
    let mut score_engine = ScoreEngine::new(score_config);
    let score = score_engine.score_incremental(&ast, file_path.to_path_buf(), &content, depth);
    
    // Enforce gates
    let result = enforcer.enforce_gates(&score, Some(&file_path.to_path_buf()));
    
    Ok(result)
}

fn process_directory(
    enforcer: &QualityGateEnforcer,
    dir_path: &Path,
    depth: AnalysisDepth,
    fail_fast: bool,
    verbose: bool,
) -> Result<Vec<crate::quality::gates::GateResult>> {
    use std::fs;
    
    let mut results = Vec::new();
    
    // Find all Ruchy files
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().is_some_and(|ext| ext == "ruchy") {
            match process_file(enforcer, &path, depth, verbose) {
                Ok(result) => {
                    if fail_fast && !result.passed {
                        eprintln!("❌ Failed fast on {}", path.display());
                        return Ok(vec![result]);
                    }
                    results.push(result);
                }
                Err(e) => {
                    eprintln!("⚠️ Error processing {}: {}", path.display(), e);
                    if fail_fast {
                        return Err(e);
                    }
                }
            }
        } else if path.is_dir() && !path.file_name().unwrap_or_default().to_string_lossy().starts_with('.') {
            // Recursively process subdirectories
            let subdir_results = process_directory(enforcer, &path, depth, fail_fast, verbose)?;
            results.extend(subdir_results);
        }
    }
    
    Ok(results)
}

fn print_console_results(results: &[crate::quality::gates::GateResult], verbose: bool) -> Result<()> {
    for (i, result) in results.iter().enumerate() {
        println!("\n📋 Quality Gate #{}: {}", i + 1, if result.passed { "✅ PASSED" } else { "❌ FAILED" });
        println!("   Score: {:.1}% ({})", result.score * 100.0, result.grade);
        println!("   Confidence: {:.1}%", result.confidence * 100.0);
        
        if !result.violations.is_empty() {
            println!("   Violations:");
            for violation in &result.violations {
                println!("     • {}", violation.message);
                if verbose {
                    println!("       Type: {:?}, Severity: {:?}", violation.violation_type, violation.severity);
                    println!("       Required: {:.3}, Actual: {:.3}", violation.required, violation.actual);
                }
            }
        }
        
        if !result.gaming_warnings.is_empty() {
            println!("   Warnings:");
            for warning in &result.gaming_warnings {
                println!("     ⚠️ {warning}");
            }
        }
    }
    
    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    
    println!("\n📊 Summary: {passed}/{total} gates passed");
    
    Ok(())
}

fn print_json_results(results: &[crate::quality::gates::GateResult]) -> Result<()> {
    let json = serde_json::to_string_pretty(results)?;
    println!("{json}");
    Ok(())
}

fn print_junit_results(results: &[crate::quality::gates::GateResult]) -> Result<()> {
    let total = results.len();
    let failures = results.iter().filter(|r| !r.passed).count();
    
    println!(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    println!(r#"<testsuite name="Quality Gates" tests="{total}" failures="{failures}" time="0.0">"#);
    
    for (i, result) in results.iter().enumerate() {
        let test_name = format!("quality-gate-{i}");
        if result.passed {
            println!(r#"  <testcase name="{test_name}" classname="QualityGate" time="0.0"/>"#);
        } else {
            println!(r#"  <testcase name="{test_name}" classname="QualityGate" time="0.0">"#);
            println!(r#"    <failure message="Quality gate violation">Score: {:.1}%, Grade: {}</failure>"#, 
                result.score * 100.0, result.grade);
            println!(r"  </testcase>");
        }
    }
    
    println!("</testsuite>");
    Ok(())
}