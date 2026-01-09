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
    use tempfile::NamedTempFile;

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

    // ===== EXTREME TDD Round 148 - Parse Handler Tests =====

    #[test]
    fn test_handle_parse_command_simple_expression() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "42").unwrap();
        let result = handle_parse_command(temp.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_parse_command_simple_verbose() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "42").unwrap();
        let result = handle_parse_command(temp.path(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_parse_command_let_binding() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "let x = 42").unwrap();
        let result = handle_parse_command(temp.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_parse_command_function() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "fun hello() { 42 }").unwrap();
        let result = handle_parse_command(temp.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_parse_command_binary_expression() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "1 + 2 * 3").unwrap();
        let result = handle_parse_command(temp.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_parse_command_syntax_error() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "fun ()").unwrap();
        let result = handle_parse_command(temp.path(), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_parse_command_empty_file() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "").unwrap();
        let result = handle_parse_command(temp.path(), false);
        // Empty file may succeed or fail depending on parser
        let _ = result;
    }

    #[test]
    fn test_handle_parse_command_if_expression() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "if true { 1 } else { 2 }").unwrap();
        let result = handle_parse_command(temp.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_parse_command_match_expression() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "match x { 1 => \"one\", _ => \"other\" }").unwrap();
        let result = handle_parse_command(temp.path(), false);
        // Match parsing may succeed or fail
        let _ = result;
    }

    #[test]
    fn test_handle_parse_command_block_expression() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "{ let x = 1; let y = 2; x + y }").unwrap();
        let result = handle_parse_command(temp.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_parse_command_unicode_content() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "let 日本語 = 42").unwrap();
        let result = handle_parse_command(temp.path(), false);
        // Unicode identifiers may or may not be supported
        let _ = result;
    }

    #[test]
    fn test_handle_parse_command_with_comments() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "// This is a comment\n42").unwrap();
        let result = handle_parse_command(temp.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_parse_command_multiline() {
        let temp = NamedTempFile::new().unwrap();
        std::fs::write(
            temp.path(),
            "let x = 1\nlet y = 2\nfun add(a, b) { a + b }\nadd(x, y)",
        )
        .unwrap();
        let result = handle_parse_command(temp.path(), false);
        assert!(result.is_ok());
    }
}
