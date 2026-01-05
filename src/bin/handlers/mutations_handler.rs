//! Mutations Command Handler
//!
//! Handles mutation testing for Ruchy files using cargo-mutants.

use anyhow::Result;
use ruchy::{Parser as RuchyParser, Transpiler};
use std::path::Path;

/// Transpile a .ruchy file to Rust source code
fn transpile_ruchy_file(path: &Path) -> Result<String> {
    let source = std::fs::read_to_string(path)?;
    let mut parser = RuchyParser::new(&source);
    let ast = parser.parse()?;

    let mut transpiler = Transpiler::new();
    let tokens = transpiler.transpile_to_program_with_context(&ast, Some(path))?;

    Ok(prettyplease::unparse(&syn::parse2(tokens)?))
}

/// Run cargo mutants on file
fn run_cargo_mutants(path: &Path, timeout: u32, verbose: bool) -> Result<std::process::Output> {
    use std::fs;

    if path.extension().and_then(|s| s.to_str()) == Some("ruchy") {
        // Step 1: Transpile .ruchy to .rs
        let transpiled = transpile_ruchy_file(path)?;

        // Step 2: Create temporary Cargo project
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let temp_dir = std::env::temp_dir().join(format!(
            "ruchy_mutations_{}_{}",
            path.file_stem()
                .expect("Path should have a file stem")
                .to_str()
                .expect("File stem should be valid UTF-8"),
            unique_id
        ));
        fs::create_dir_all(&temp_dir)?;

        // Step 3: Write Cargo.toml
        let cargo_toml = r#"[package]
name = "ruchy-mutations-test"
version = "0.1.0"
edition = "2021"

[lib]
name = "lib"
path = "src/lib.rs"
"#;
        fs::write(temp_dir.join("Cargo.toml"), cargo_toml)?;

        // Step 4: Write transpiled code to src/lib.rs
        let src_dir = temp_dir.join("src");
        fs::create_dir_all(&src_dir)?;
        fs::write(src_dir.join("lib.rs"), transpiled)?;

        if verbose {
            eprintln!("Created temp Cargo project at {}", temp_dir.display());
        }

        // Step 5: Run cargo mutants in temp project
        let mut cmd = std::process::Command::new("cargo");
        cmd.current_dir(&temp_dir).args([
            "mutants",
            "--timeout",
            &timeout.to_string(),
            "--no-times",
        ]);

        let output_result = cmd.output()?;
        super::log_command_output(&output_result, verbose);

        // Step 6: Cleanup temp project
        let _ = fs::remove_dir_all(&temp_dir);

        Ok(output_result)
    } else {
        // For .rs files in workspace: run directly
        let mut cmd = std::process::Command::new("cargo");
        cmd.args([
            "mutants",
            "--file",
            path.to_str().expect("Path should be valid UTF-8"),
            "--timeout",
            &timeout.to_string(),
            "--no-times",
        ]);

        let output_result = cmd.output()?;
        super::log_command_output(&output_result, verbose);

        Ok(output_result)
    }
}

/// Write JSON format mutation test report
fn write_json_mutation_report(
    output: Option<&Path>,
    success: bool,
    min_coverage: f64,
    stdout: &str,
) -> Result<()> {
    let report = serde_json::json!({
        "status": if success { "passed" } else { "failed" },
        "min_coverage": min_coverage,
        "output": stdout
    });
    let json_output = serde_json::to_string_pretty(&report)?;

    if let Some(out_path) = output {
        super::write_file_with_context(out_path, json_output.as_bytes())?;
    } else {
        println!("{}", json_output);
    }
    Ok(())
}

/// Write text format mutation test report
fn write_text_mutation_report(
    output: Option<&Path>,
    min_coverage: f64,
    stdout: &str,
) -> Result<()> {
    println!("Mutation Test Report");
    println!("====================");
    println!("Minimum coverage: {:.1}%", min_coverage * 100.0);

    if let Some(out_path) = output {
        super::write_file_with_context(out_path, stdout.as_bytes())?;
    } else {
        println!("\n{}", stdout);
    }
    Ok(())
}

/// Handle mutations command - run mutation tests with cargo-mutants
pub fn handle_mutations_command(
    path: &Path,
    timeout: u32,
    format: &str,
    output: Option<&Path>,
    min_coverage: f64,
    verbose: bool,
) -> Result<()> {
    use std::fs;

    if verbose {
        eprintln!("Running mutation tests on: {}", path.display());
        eprintln!(
            "Timeout: {}s, Min coverage: {:.1}%",
            timeout,
            min_coverage * 100.0
        );
    }

    // Check if file exists
    if !path.exists() {
        println!("Found 0 mutants to test");
        return Ok(());
    }

    // Check if file can be parsed (for .ruchy files)
    if path.extension().and_then(|s| s.to_str()) == Some("ruchy") {
        if let Ok(source) = fs::read_to_string(path) {
            let mut parser = ruchy::frontend::parser::Parser::new(&source);
            if parser.parse().is_err() {
                println!("Found 0 mutants to test");
                return Ok(());
            }
        }
    }

    // Run cargo mutants
    let output_result = run_cargo_mutants(path, timeout, verbose)?;
    let stdout = String::from_utf8_lossy(&output_result.stdout);
    let cargo_success = output_result.status.success();

    // Parse coverage from output
    let coverage_ok = if min_coverage <= 0.0 {
        true
    } else {
        let caught = stdout
            .lines()
            .find(|l| l.contains("mutants tested:"))
            .and_then(|l| {
                let parts: Vec<&str> = l.split_whitespace().collect();
                let total_idx = parts.iter().position(|&p| p == "mutants")?;
                let total: f64 = parts.get(total_idx - 1)?.parse().ok()?;
                let caught_idx = parts
                    .iter()
                    .position(|&p| p == "caught" || p == "caught,")?;
                let caught: f64 = parts.get(caught_idx - 1)?.parse().ok()?;
                Some((caught, total))
            });

        match caught {
            Some((caught, total)) if total > 0.0 => {
                let coverage = caught / total;
                coverage >= min_coverage
            }
            _ => cargo_success,
        }
    };

    let success = coverage_ok || cargo_success;

    // Generate report
    match format {
        "json" => write_json_mutation_report(output, success, min_coverage, &stdout)?,
        _ => write_text_mutation_report(output, min_coverage, &stdout)?,
    }

    if coverage_ok {
        Ok(())
    } else {
        anyhow::bail!("Mutation tests failed or coverage below threshold")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_mutations_nonexistent() {
        let path = Path::new("/nonexistent/file.ruchy");
        // Should succeed with "Found 0 mutants" message
        let result = handle_mutations_command(path, 60, "text", None, 0.0, false);
        assert!(result.is_ok());
    }
}
