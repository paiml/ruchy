//! File Execution Handler
//!
//! Handles direct execution of Ruchy script files and stdin input.

use anyhow::Result;
use ruchy::backend::module_resolver::ModuleResolver;
use ruchy::frontend::ast::{Expr, ExprKind};
use ruchy::runtime::interpreter::Interpreter;
use ruchy::Parser as RuchyParser;
use std::path::Path;

/// Handle file execution - run a Ruchy script file directly (not via subcommand)
///
/// # Arguments
/// * `file` - Path to the Ruchy file to execute
///
/// # Errors
/// Returns error if file cannot be read, parsed, or executed
pub fn handle_file_execution(file: &Path) -> Result<()> {
    let source = super::read_file_with_context(file)?;

    // ISSUE-106: Parse and check for module declarations
    let mut parser = RuchyParser::new(&source);
    let ast = parser
        .parse()
        .map_err(|e| anyhow::anyhow!("Syntax error: {e}"))?;

    // Check if we need module resolution
    if needs_module_resolution(&ast) {
        // Resolve module declarations (mod name;) and imports
        let resolved_ast = resolve_modules_for_execution(file, ast)?;

        // Use interpreter to evaluate the resolved AST
        let mut interpreter = Interpreter::new();
        interpreter
            .eval_expr(&resolved_ast)
            .map_err(|e| anyhow::anyhow!("Evaluation error: {e:?}"))?;
        return Ok(());
    }

    // CLI-UNIFY-002: Use REPL-based evaluation for simple scripts
    let mut repl = super::create_repl()?;

    match repl.eval(&source) {
        Ok(_result) => {
            // After evaluating the file, call main() if it exists
            match repl.eval("main()") {
                Ok(_) => Ok(()),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

/// Handle stdin/piped input - evaluate input from standard input
///
/// # Arguments
/// * `input` - The input string to evaluate
///
/// # Errors
/// Returns error if input cannot be parsed or evaluated
pub fn handle_stdin_input(input: &str) -> Result<()> {
    let mut repl = super::create_repl()?;
    match repl.eval(input) {
        Ok(result) => {
            println!("{result}");
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {e}");
            Err(e)
        }
    }
}

/// Check if AST contains module declarations that need resolution
pub(crate) fn needs_module_resolution(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::ModuleDeclaration { .. } => true,
        ExprKind::Module { .. } => true,
        ExprKind::Import { .. } => true,
        ExprKind::ImportAll { .. } => true,
        ExprKind::ImportDefault { .. } => true,
        ExprKind::Block(exprs) => exprs.iter().any(needs_module_resolution),
        ExprKind::Function { body, .. } => needs_module_resolution(body),
        ExprKind::Let { value, body, .. } => {
            needs_module_resolution(value) || needs_module_resolution(body)
        }
        _ => false,
    }
}

/// ISSUE-106: Resolve module declarations and imports for script execution
pub(crate) fn resolve_modules_for_execution(source_path: &Path, ast: Expr) -> Result<Expr> {
    let mut resolver = ModuleResolver::new();

    // Add the source file's directory to the module search path
    if let Some(parent_dir) = source_path.parent() {
        resolver.add_search_path(parent_dir);

        // Also search in standard project layout directories
        if let Some(project_root) = parent_dir.parent() {
            resolver.add_search_path(project_root.join("src"));
            resolver.add_search_path(project_root.join("lib"));
            resolver.add_search_path(project_root.join("modules"));
        }
    }

    resolver
        .resolve_imports(ast)
        .map_err(|e| anyhow::anyhow!("Module resolution error: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_handle_stdin_input_simple_expression() {
        // This would require a more sophisticated test setup
        // Just verify the function signature works
        let _ = handle_stdin_input("2 + 2");
    }

    #[test]
    fn test_handle_file_execution_nonexistent() {
        let path = PathBuf::from("/nonexistent/file.ruchy");
        let result = handle_file_execution(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_needs_module_resolution_empty_block() {
        let expr = Expr {
            kind: ExprKind::Block(vec![]),
            span: ruchy::frontend::ast::Span::new(0, 0),
        };
        assert!(!needs_module_resolution(&expr));
    }

    #[test]
    fn test_needs_module_resolution_with_import() {
        let expr = Expr {
            kind: ExprKind::Import {
                module: "test".to_string(),
                items: vec![],
            },
            span: ruchy::frontend::ast::Span::new(0, 0),
        };
        assert!(needs_module_resolution(&expr));
    }

    #[test]
    fn test_needs_module_resolution_with_module_decl() {
        let expr = Expr {
            kind: ExprKind::ModuleDeclaration {
                name: "test".to_string(),
            },
            span: ruchy::frontend::ast::Span::new(0, 0),
        };
        assert!(needs_module_resolution(&expr));
    }

    #[test]
    fn test_needs_module_resolution_with_import_all() {
        let expr = Expr {
            kind: ExprKind::ImportAll {
                module: "test".to_string(),
            },
            span: ruchy::frontend::ast::Span::new(0, 0),
        };
        assert!(needs_module_resolution(&expr));
    }

    #[test]
    fn test_needs_module_resolution_nested_in_block() {
        let import = Expr {
            kind: ExprKind::Import {
                module: "test".to_string(),
                items: vec![],
            },
            span: ruchy::frontend::ast::Span::new(0, 0),
        };
        let block = Expr {
            kind: ExprKind::Block(vec![import]),
            span: ruchy::frontend::ast::Span::new(0, 0),
        };
        assert!(needs_module_resolution(&block));
    }

    #[test]
    fn test_resolve_modules_for_execution_nonexistent_path() {
        let expr = Expr {
            kind: ExprKind::Block(vec![]),
            span: ruchy::frontend::ast::Span::new(0, 0),
        };
        let path = PathBuf::from("/nonexistent/dir/file.ruchy");
        // Should succeed for empty block (no modules to resolve)
        let result = resolve_modules_for_execution(&path, expr);
        assert!(result.is_ok());
    }
}
