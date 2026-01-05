//! Parse Command Handler
//!
//! Handles parsing Ruchy files and displaying the AST.

use anyhow::Result;
use ruchy::Parser as RuchyParser;
use std::path::Path;

/// Handle parse command - parse a Ruchy file and display the AST
///
/// # Arguments
/// * `file` - Path to the Ruchy file to parse
/// * `verbose` - Enable verbose output
///
/// # Errors
/// Returns error if file cannot be read or parsed
pub fn handle_parse_command(file: &Path, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!("Parsing file: {}", file.display());
    }
    let source = super::read_file_with_context(file)?;
    let mut parser = RuchyParser::new(&source);
    match parser.parse() {
        Ok(ast) => {
            println!("{ast:#?}");
            Ok(())
        }
        Err(e) => {
            eprintln!("Parse error: {e}");
            Err(anyhow::anyhow!("Parse error: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_handle_parse_command_nonexistent_file() {
        let path = PathBuf::from("/nonexistent/file.ruchy");
        let result = handle_parse_command(&path, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_parse_command_verbose_nonexistent() {
        let path = PathBuf::from("/nonexistent/file.ruchy");
        let result = handle_parse_command(&path, true);
        assert!(result.is_err());
    }
}
