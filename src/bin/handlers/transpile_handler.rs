//! Transpile Command Handler
//!
//! Handles transpilation of Ruchy code to Rust.

use anyhow::{Context, Result};
use ruchy::frontend::ast::Expr;
use ruchy::{Parser as RuchyParser, Transpiler};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Handle transpile command - convert Ruchy to Rust
pub fn handle_transpile_command(
    file: &Path,
    output: Option<&Path>,
    minimal: bool,
    verbose: bool,
) -> Result<()> {
    log_transpile_start(file, minimal, verbose);
    let source = read_source_file(file, verbose)?;
    let ast = parse_source(&source)?;
    let rust_code = transpile_ast(&ast, minimal)?;
    // Default to stdout for backwards compatibility (many tests expect stdout output)
    // Use -o to specify file output explicitly
    write_output(&rust_code, output, verbose)?;
    Ok(())
}

/// Derive default output path from input file (QA-049)
/// Changes .ruchy extension to .rs, or appends .rs if no .ruchy extension
/// Returns None for stdin input ("-")
/// Complexity: 3 (within Toyota Way limits)
#[allow(dead_code)]
pub fn derive_default_output_path(file: &Path) -> Option<PathBuf> {
    if file.as_os_str() == "-" {
        return None; // stdin: output to stdout
    }
    let stem = file.file_stem()?;
    let parent = file.parent().unwrap_or(Path::new("."));
    Some(parent.join(format!("{}.rs", stem.to_string_lossy())))
}

/// Log transpilation start (complexity: 3)
fn log_transpile_start(file: &Path, minimal: bool, verbose: bool) {
    if !verbose {
        return;
    }
    eprintln!("Transpiling file: {}", file.display());
    if minimal {
        eprintln!("Using minimal codegen for self-hosting");
    }
}

/// Read source from file or stdin (complexity: 5)
pub fn read_source_file(file: &Path, verbose: bool) -> Result<String> {
    if file.as_os_str() == "-" {
        if verbose {
            eprintln!("Reading from stdin...");
        }
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        Ok(input)
    } else {
        fs::read_to_string(file).with_context(|| format!("Failed to read file: {}", file.display()))
    }
}

/// Parse source code to AST (complexity: 2)
pub fn parse_source(source: &str) -> Result<Expr> {
    let mut parser = RuchyParser::new(source);
    parser.parse().with_context(|| "Failed to parse input")
}

/// Transpile AST to Rust code (complexity: 4)
/// PARSER-077: Use prettyplease for proper formatting (no extra spaces)
pub fn transpile_ast(ast: &Expr, minimal: bool) -> Result<String> {
    let mut transpiler = Transpiler::new();
    if minimal {
        transpiler
            .transpile_minimal(ast)
            .with_context(|| "Failed to transpile to Rust (minimal)")
    } else {
        let tokens = transpiler
            .transpile_to_program(ast)
            .with_context(|| "Failed to transpile to Rust")?;

        // Parse TokenStream as syn::File and format with prettyplease
        let syntax_tree = syn::parse2(tokens)
            .with_context(|| "Failed to parse generated tokens as Rust syntax")?;
        Ok(prettyplease::unparse(&syntax_tree))
    }
}

/// Write output to file or stdout (complexity: 5)
/// Use "-" as output path to write to stdout explicitly
pub fn write_output(rust_code: &str, output: Option<&Path>, verbose: bool) -> Result<()> {
    if let Some(output_path) = output {
        // QA-049: "-" means stdout, for explicit stdout output
        if output_path.as_os_str() == "-" {
            print!("{rust_code}");
        } else {
            super::write_file_with_context(output_path, rust_code.as_bytes())?;
            if verbose {
                eprintln!("Output written to: {}", output_path.display());
            }
        }
    } else {
        print!("{rust_code}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_default_output_path_ruchy() {
        let path = Path::new("src/main.ruchy");
        let output = derive_default_output_path(path);
        assert_eq!(output, Some(PathBuf::from("src/main.rs")));
    }

    #[test]
    fn test_derive_default_output_path_no_extension() {
        let path = Path::new("script");
        let output = derive_default_output_path(path);
        assert_eq!(output, Some(PathBuf::from("script.rs")));
    }

    #[test]
    fn test_derive_default_output_path_stdin() {
        let path = Path::new("-");
        let output = derive_default_output_path(path);
        assert_eq!(output, None);
    }

    #[test]
    fn test_parse_source_simple() {
        let source = "42";
        let ast = parse_source(source);
        assert!(ast.is_ok());
    }

    #[test]
    fn test_parse_source_invalid() {
        let source = "let = invalid";
        let ast = parse_source(source);
        assert!(ast.is_err());
    }

    #[test]
    fn test_transpile_ast_simple() {
        let source = "fun main() { 42 }";
        let ast = parse_source(source).unwrap();
        let rust_code = transpile_ast(&ast, false);
        assert!(rust_code.is_ok());
    }

    #[test]
    fn test_transpile_ast_minimal() {
        let source = "42";
        let ast = parse_source(source).unwrap();
        let rust_code = transpile_ast(&ast, true);
        assert!(rust_code.is_ok());
    }

    #[test]
    fn test_log_transpile_start_verbose() {
        // Just verify it doesn't panic
        let file = Path::new("test.ruchy");
        log_transpile_start(file, true, true);
        log_transpile_start(file, false, true);
        log_transpile_start(file, true, false);
    }
}
