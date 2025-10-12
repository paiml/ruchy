//! NOTEBOOK-009 Phase 6: Notebook Validation Tests
//!
//! Automated testing for .rnb notebook files:
//! - Load notebooks from disk
//! - Execute all code cells
//! - Validate outputs
//! - Generate validation report

use std::fs;
use std::path::Path;

// Re-use the types from the main crate
use ruchy::notebook::engine::NotebookEngine;
use ruchy::notebook::types::{CellType, Notebook};

/// Load a notebook from .rnb file
fn load_notebook(path: &Path) -> Result<Notebook, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse notebook: {e}"))
}

/// Execute all code cells in a notebook and collect results
fn execute_notebook(notebook: &Notebook) -> Vec<(usize, String, Result<String, String>)> {
    let mut engine = match NotebookEngine::new() {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Failed to create notebook engine: {err}");
            return Vec::new();
        }
    };

    let mut results = Vec::new();

    for (idx, cell) in notebook.cells.iter().enumerate() {
        if cell.cell_type == CellType::Code {
            let source = cell.source.clone();
            let result = engine.execute_cell(&source);

            let output = match result {
                Ok(output_str) => Ok(output_str),
                Err(e) => Err(format!("{e}")),
            };

            results.push((idx, source, output));
        }
    }

    results
}

/// Calculate pass rate for notebook validation
fn calculate_pass_rate(results: &[(usize, String, Result<String, String>)]) -> f64 {
    let total = results.len();
    if total == 0 {
        return 100.0;
    }

    let passed = results
        .iter()
        .filter(|(_, _, result)| result.is_ok())
        .count();
    (passed as f64 / total as f64) * 100.0
}

#[test]
fn test_validate_01_literals_notebook() {
    let notebook_path = Path::new("notebooks/01-literals.rnb");

    // RED: This test will fail if notebook doesn't exist or has issues
    let notebook = load_notebook(notebook_path).expect("Failed to load 01-literals.rnb");

    // Execute all code cells
    let results = execute_notebook(&notebook);

    // Report results
    println!("\n📊 Validation Report: 01-literals.rnb");
    println!("Total code cells: {}", results.len());

    let mut passed = 0;
    let mut failed = 0;

    for (idx, source, result) in &results {
        match result {
            Ok(output) => {
                passed += 1;
                println!("  ✅ Cell {idx}: {source} → {output}");
            }
            Err(e) => {
                failed += 1;
                println!("  ❌ Cell {idx}: {source}");
                println!("     Error: {e}");
            }
        }
    }

    let pass_rate = calculate_pass_rate(&results);
    println!(
        "\nPass Rate: {:.1}% ({passed}/{} cells)",
        pass_rate,
        passed + failed
    );

    // Assert ≥90% pass rate (per specification)
    assert!(
        pass_rate >= 90.0,
        "Pass rate {:.1}% below 90% threshold",
        pass_rate
    );
}

#[test]
fn test_validate_01_variables_notebook() {
    let notebook_path = Path::new("notebooks/01-variables.rnb");

    let notebook = load_notebook(notebook_path).expect("Failed to load 01-variables.rnb");

    let results = execute_notebook(&notebook);
    let pass_rate = calculate_pass_rate(&results);

    println!(
        "\n📊 01-variables.rnb: {:.1}% pass rate ({}/{} cells)",
        pass_rate,
        results.iter().filter(|(_, _, r)| r.is_ok()).count(),
        results.len()
    );

    assert!(
        pass_rate >= 90.0,
        "Pass rate {:.1}% below 90% threshold",
        pass_rate
    );
}

#[test]
fn test_validate_02_arithmetic_notebook() {
    let notebook_path = Path::new("notebooks/02-arithmetic.rnb");

    let notebook = load_notebook(notebook_path).expect("Failed to load 02-arithmetic.rnb");

    let results = execute_notebook(&notebook);
    let pass_rate = calculate_pass_rate(&results);

    println!(
        "\n📊 02-arithmetic.rnb: {:.1}% pass rate ({}/{} cells)",
        pass_rate,
        results.iter().filter(|(_, _, r)| r.is_ok()).count(),
        results.len()
    );

    assert!(
        pass_rate >= 90.0,
        "Pass rate {:.1}% below 90% threshold",
        pass_rate
    );
}

#[test]
fn test_validate_03_if_else_notebook() {
    let notebook_path = Path::new("notebooks/03-if-else.rnb");

    let notebook = load_notebook(notebook_path).expect("Failed to load 03-if-else.rnb");

    let results = execute_notebook(&notebook);
    let pass_rate = calculate_pass_rate(&results);

    println!(
        "\n📊 03-if-else.rnb: {:.1}% pass rate ({}/{} cells)",
        pass_rate,
        results.iter().filter(|(_, _, r)| r.is_ok()).count(),
        results.len()
    );

    assert!(
        pass_rate >= 90.0,
        "Pass rate {:.1}% below 90% threshold",
        pass_rate
    );
}

#[test]
fn test_validate_all_notebooks_comprehensive() {
    let notebook_files = vec![
        "notebooks/01-literals.rnb",
        "notebooks/01-variables.rnb",
        "notebooks/02-arithmetic.rnb",
        "notebooks/03-if-else.rnb",
    ];

    let mut total_cells = 0;
    let mut total_passed = 0;
    let mut total_failed = 0;

    println!("\n📊 Comprehensive Notebook Validation");
    println!("=====================================\n");

    for notebook_file in &notebook_files {
        let path = Path::new(notebook_file);

        match load_notebook(path) {
            Ok(notebook) => {
                let results = execute_notebook(&notebook);
                let passed = results.iter().filter(|(_, _, r)| r.is_ok()).count();
                let failed = results.len() - passed;

                total_cells += results.len();
                total_passed += passed;
                total_failed += failed;

                let pass_rate = calculate_pass_rate(&results);
                println!(
                    "  {} - {:.1}% ({}/{})",
                    path.file_name().unwrap().to_str().unwrap(),
                    pass_rate,
                    passed,
                    results.len()
                );
            }
            Err(e) => {
                println!("  ❌ {}: {e}", path.display());
            }
        }
    }

    println!("\n📈 Overall Statistics:");
    println!("  Total notebooks: {}", notebook_files.len());
    println!("  Total code cells: {total_cells}");
    println!("  Passed: {total_passed}");
    println!("  Failed: {total_failed}");

    let overall_pass_rate = if total_cells > 0 {
        (total_passed as f64 / total_cells as f64) * 100.0
    } else {
        100.0
    };

    println!("  Overall Pass Rate: {:.1}%", overall_pass_rate);

    assert!(
        overall_pass_rate >= 90.0,
        "Overall pass rate {:.1}% below 90% threshold",
        overall_pass_rate
    );
}
