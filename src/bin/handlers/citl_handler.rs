//! `ruchy transpile --citl-export PATH` (ruchy#165): transpile as usual, then
//! compile the generated Rust and write its diagnostics as CITL records.

use super::transpile_handler::{parse_source, read_source_file, transpile_ast, write_output};
use anyhow::Result;
use ruchy::citl_export::export_rustc_diagnostics;
use std::fs;
use std::path::Path;

/// Transpile `file`, write the Rust (stdout or `output`), and write the CITL
/// export JSON to `export`. A program with compiler errors still exits 0: the
/// errors are the data.
///
/// # Errors
///
/// Returns an error when the input cannot be read or parsed, or the export
/// cannot be written.
pub fn handle_transpile_citl(
    file: &Path,
    output: Option<&Path>,
    minimal: bool,
    export: &Path,
    verbose: bool,
) -> Result<()> {
    let source = read_source_file(file, verbose)?;
    let ast = parse_source(&source)?;
    let rust_code = transpile_ast(&ast, minimal)?;
    write_output(&rust_code, output, verbose)?;
    let records = export_rustc_diagnostics(&rust_code, &file.display().to_string())?;
    fs::write(export, serde_json::to_string_pretty(&records)?)?;
    Ok(())
}
