//! Transpilation report generation module.
//!
//! Extracted from ruchy.rs to reduce file size.
//! Handles report generation in JSON, Markdown, SARIF, and human-readable formats.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use super::scan_ruchy_files;

/// Handle report command - generate transpilation reports
pub(super) fn handle_report_command(
    target: &Path,
    format: &str,
    output: Option<&Path>,
    verbose: bool,
) -> Result<()> {
    use colored::Colorize;
    use ruchy::reporting::formats::{
        HumanFormatter, JsonFormatter, MarkdownFormatter, SarifFormatter,
    };

    println!("{}", "📊 Generating Transpilation Report".bold());
    println!("   Target: {}", target.display());
    println!("   Format: {}", format);
    println!();

    // Scan for .ruchy files
    let ruchy_files = scan_ruchy_files(target)?;
    if ruchy_files.is_empty() {
        println!("{}", "⚠ No .ruchy files found".yellow());
        return Ok(());
    }

    // Collect results
    let mut results = Vec::new();
    let mut success_count = 0;
    let mut failure_count = 0;

    for file_path in &ruchy_files {
        if verbose {
            println!("  Analyzing: {}", file_path.display());
        }

        match analyze_file_for_report(file_path) {
            Ok(result) => {
                if result.success {
                    success_count += 1;
                } else {
                    failure_count += 1;
                }
                results.push(result);
            }
            Err(e) => {
                failure_count += 1;
                results.push(FileResult {
                    path: file_path.clone(),
                    success: false,
                    error: Some(e.to_string()),
                    warnings: vec![],
                });
            }
        }
    }

    // Format output
    let report_content = match format {
        "json" => {
            let formatter = JsonFormatter::pretty();
            format_report_json(&results, &formatter)
        }
        "markdown" | "md" => {
            let formatter = MarkdownFormatter;
            format_report_markdown(&results, &formatter)
        }
        "sarif" => {
            let formatter = SarifFormatter;
            format_report_sarif(&results, &formatter)
        }
        _ => {
            let formatter = HumanFormatter::default();
            format_report_human(&results, &formatter)
        }
    };

    // Output
    if let Some(output_path) = output {
        fs::write(output_path, &report_content)?;
        println!(
            "{}",
            format!("📝 Report written to: {}", output_path.display()).green()
        );
    } else {
        println!("{}", report_content);
    }

    // Summary
    println!();
    println!("{}", "━━━ Report Summary ━━━".bold());
    println!("  Total Files: {}", results.len());
    println!("  {} Successful", format!("{}", success_count).green());
    println!("  {} Failed", format!("{}", failure_count).red());

    Ok(())
}

/// Result of analyzing a single file
struct FileResult {
    path: PathBuf,
    success: bool,
    error: Option<String>,
    warnings: Vec<String>,
}

/// Analyze a file for the report
fn analyze_file_for_report(file_path: &Path) -> Result<FileResult> {
    use ruchy::{Parser as RuchyParser, Transpiler};

    let source = fs::read_to_string(file_path)?;
    let mut parser = RuchyParser::new(&source);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            return Ok(FileResult {
                path: file_path.to_path_buf(),
                success: false,
                error: Some(format!("Parse error: {}", e)),
                warnings: vec![],
            });
        }
    };

    let mut transpiler = Transpiler::new();
    match transpiler.transpile(&ast) {
        Ok(_) => Ok(FileResult {
            path: file_path.to_path_buf(),
            success: true,
            error: None,
            warnings: vec![],
        }),
        Err(e) => Ok(FileResult {
            path: file_path.to_path_buf(),
            success: false,
            error: Some(format!("Transpile error: {}", e)),
            warnings: vec![],
        }),
    }
}

/// Format report as JSON
fn format_report_json(
    results: &[FileResult],
    _formatter: &ruchy::reporting::formats::JsonFormatter,
) -> String {
    let json = serde_json::json!({
        "total": results.len(),
        "success": results.iter().filter(|r| r.success).count(),
        "failed": results.iter().filter(|r| !r.success).count(),
        "files": results.iter().map(|r| {
            serde_json::json!({
                "path": r.path.display().to_string(),
                "success": r.success,
                "error": r.error,
                "warnings": r.warnings
            })
        }).collect::<Vec<_>>()
    });
    serde_json::to_string_pretty(&json).unwrap_or_default()
}

/// Format report as Markdown
fn format_report_markdown(
    results: &[FileResult],
    _formatter: &ruchy::reporting::formats::MarkdownFormatter,
) -> String {
    let mut md = String::from("# Transpilation Report\n\n");
    md.push_str("## Summary\n\n");
    md.push_str(&format!("- **Total Files**: {}\n", results.len()));
    md.push_str(&format!(
        "- **Successful**: {}\n",
        results.iter().filter(|r| r.success).count()
    ));
    md.push_str(&format!(
        "- **Failed**: {}\n\n",
        results.iter().filter(|r| !r.success).count()
    ));

    md.push_str("## Results\n\n");
    for result in results {
        let status = if result.success { "✅" } else { "❌" };
        md.push_str(&format!("### {} {}\n\n", status, result.path.display()));
        if let Some(ref error) = result.error {
            md.push_str(&format!("**Error**: {}\n\n", error));
        }
    }
    md
}

/// Format report as SARIF
fn format_report_sarif(
    results: &[FileResult],
    _formatter: &ruchy::reporting::formats::SarifFormatter,
) -> String {
    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "ruchy",
                    "version": env!("CARGO_PKG_VERSION")
                }
            },
            "results": results.iter().filter(|r| !r.success).map(|r| {
                serde_json::json!({
                    "ruleId": "TRANSPILE001",
                    "level": "error",
                    "message": {
                        "text": r.error.as_deref().unwrap_or("Unknown error")
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": r.path.display().to_string()
                            }
                        }
                    }]
                })
            }).collect::<Vec<_>>()
        }]
    });
    serde_json::to_string_pretty(&sarif).unwrap_or_default()
}

/// Format report as human-readable text
fn format_report_human(
    results: &[FileResult],
    _formatter: &ruchy::reporting::formats::HumanFormatter,
) -> String {
    let mut output = String::from("Transpilation Report\n");
    output.push_str(&"=".repeat(40));
    output.push('\n');
    output.push_str(&format!("\nTotal: {} files\n", results.len()));
    output.push_str(&format!(
        "Success: {}\n",
        results.iter().filter(|r| r.success).count()
    ));
    output.push_str(&format!(
        "Failed: {}\n\n",
        results.iter().filter(|r| !r.success).count()
    ));

    for result in results {
        let status = if result.success { "[OK]" } else { "[FAIL]" };
        output.push_str(&format!("{} {}\n", status, result.path.display()));
        if let Some(ref error) = result.error {
            output.push_str(&format!("     Error: {}\n", error));
        }
    }
    output
}
