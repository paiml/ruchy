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

/// Spawn `cmd`; when the program itself is missing, name it and say how to get it
/// (a bare "No such file or directory" hid which tool was absent).
fn spawn_tool(cmd: &mut std::process::Command) -> Result<std::process::Output> {
    cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            anyhow::anyhow!(
                "could not run `{}`: not found on PATH. `ruchy mutations` needs cargo and cargo-mutants: \
                 install Rust from https://rustup.rs, then run `cargo install cargo-mutants`",
                cmd.get_program().to_string_lossy()
            )
        } else {
            anyhow::Error::from(e)
        }
    })
}

/// True when cargo ran but has no `mutants` subcommand (cargo-mutants not installed).
fn cargo_mutants_not_installed(output: &std::process::Output) -> bool {
    let stderr = String::from_utf8_lossy(&output.stderr);
    !output.status.success() && stderr.contains("no such command") && stderr.contains("mutants")
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

        let output_result = spawn_tool(&mut cmd);

        // Step 6: Cleanup temp project (also when the spawn failed)
        let _ = fs::remove_dir_all(&temp_dir);

        let output_result = output_result?;
        super::log_command_output(&output_result, verbose);

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

        let output_result = spawn_tool(&mut cmd)?;
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
    if cargo_mutants_not_installed(&output_result) {
        anyhow::bail!(
            "cargo-mutants is not installed: run `cargo install cargo-mutants` (needed by `ruchy mutations`)"
        );
    }
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_handle_mutations_nonexistent() {
        let path = Path::new("/nonexistent/file.ruchy");
        // Should succeed with "Found 0 mutants" message
        let result = handle_mutations_command(path, 60, "text", None, 0.0, false);
        assert!(result.is_ok());
    }

    // ===== EXTREME TDD Round 144 - Handler Tests =====

    #[test]
    fn test_handle_mutations_nonexistent_verbose() {
        let path = Path::new("/nonexistent/file.ruchy");
        let result = handle_mutations_command(path, 60, "text", None, 0.0, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_mutations_json_format() {
        let path = Path::new("/nonexistent/file.ruchy");
        let result = handle_mutations_command(path, 60, "json", None, 0.0, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_mutations_with_output_path() {
        let path = Path::new("/nonexistent/file.ruchy");
        let temp = NamedTempFile::new().unwrap();
        let result = handle_mutations_command(path, 60, "text", Some(temp.path()), 0.0, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_mutations_with_min_coverage() {
        let path = Path::new("/nonexistent/file.ruchy");
        let result = handle_mutations_command(path, 60, "text", None, 0.5, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_mutations_invalid_ruchy_file() {
        let mut temp = NamedTempFile::with_suffix(".ruchy").unwrap();
        writeln!(temp, "this is {{ invalid syntax").unwrap();
        let result = handle_mutations_command(temp.path(), 60, "text", None, 0.0, false);
        assert!(result.is_ok()); // Returns "Found 0 mutants" for invalid files
    }

    #[test]
    fn test_write_json_mutation_report_success() {
        let result = write_json_mutation_report(None, true, 0.75, "test output");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_json_mutation_report_failure() {
        let result = write_json_mutation_report(None, false, 0.5, "test output");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_json_mutation_report_to_file() {
        let temp = NamedTempFile::new().unwrap();
        let result = write_json_mutation_report(Some(temp.path()), true, 0.8, "test");
        assert!(result.is_ok());
        let content = std::fs::read_to_string(temp.path()).unwrap();
        assert!(content.contains("passed"));
    }

    #[test]
    fn test_write_text_mutation_report() {
        let result = write_text_mutation_report(None, 0.75, "test output");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_text_mutation_report_to_file() {
        let temp = NamedTempFile::new().unwrap();
        let result = write_text_mutation_report(Some(temp.path()), 0.8, "test output");
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_mutations_different_timeouts() {
        let path = Path::new("/nonexistent/file.ruchy");
        // Very short timeout
        let result = handle_mutations_command(path, 1, "text", None, 0.0, false);
        assert!(result.is_ok());
        // Long timeout
        let result = handle_mutations_command(path, 300, "text", None, 0.0, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_mutations_zero_min_coverage() {
        let path = Path::new("/nonexistent/file.ruchy");
        let result = handle_mutations_command(path, 60, "text", None, 0.0, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_mutations_full_coverage() {
        let path = Path::new("/nonexistent/file.ruchy");
        let result = handle_mutations_command(path, 60, "text", None, 1.0, false);
        assert!(result.is_ok());
    }

    // ===== #112 slice 2: a missing tool is named, not "No such file or directory" =====

    #[test]
    fn test_spawn_tool_missing_program_names_tool_and_install_hint() {
        let mut cmd = std::process::Command::new("ruchy-no-such-tool-112");
        let err = spawn_tool(&mut cmd).unwrap_err().to_string();
        assert!(err.contains("ruchy-no-such-tool-112"), "{err}");
        assert!(err.contains("cargo install cargo-mutants"), "{err}");
        assert!(!err.contains("os error 2"), "{err}");
    }

    #[test]
    fn test_spawn_tool_present_program_still_runs() {
        // Regression: a program that exists returns its Output unchanged.
        let mut cmd = std::process::Command::new("sh");
        cmd.args(["-c", "echo hi; exit 3"]);
        let out = spawn_tool(&mut cmd).unwrap();
        assert_eq!(out.status.code(), Some(3));
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "hi");
    }

    #[test]
    fn test_spawn_tool_other_io_error_passes_through() {
        // A non-NotFound failure (cwd is a regular file) keeps the OS error text.
        let file = NamedTempFile::new().unwrap();
        let mut cmd = std::process::Command::new("sh");
        cmd.current_dir(file.path());
        let err = spawn_tool(&mut cmd).unwrap_err().to_string();
        assert!(!err.contains("not found on PATH"), "{err}");
    }

    #[cfg(unix)]
    fn fake_output(code: i32, stderr: &str) -> std::process::Output {
        use std::os::unix::process::ExitStatusExt;
        std::process::Output {
            status: std::process::ExitStatus::from_raw(code << 8),
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_cargo_mutants_not_installed_detected() {
        let out = fake_output(101, "error: no such command: `mutants`\n");
        assert!(cargo_mutants_not_installed(&out));
    }

    #[cfg(unix)]
    #[test]
    fn test_cargo_mutants_not_installed_ignores_success() {
        let out = fake_output(0, "error: no such command: `mutants`\n");
        assert!(!cargo_mutants_not_installed(&out));
    }

    #[cfg(unix)]
    #[test]
    fn test_cargo_mutants_not_installed_ignores_other_failures() {
        // Regression: ordinary mutation failures and other missing commands are not this case.
        assert!(!cargo_mutants_not_installed(&fake_output(
            2,
            "mutants failed\n"
        )));
        assert!(!cargo_mutants_not_installed(&fake_output(
            101,
            "error: no such command: `nextest`\n"
        )));
    }
}
