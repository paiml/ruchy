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

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: ruchy::frontend::ast::Span::new(0, 0),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

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
        let expr = make_expr(ExprKind::Block(vec![]));
        assert!(!needs_module_resolution(&expr));
    }

    #[test]
    fn test_needs_module_resolution_with_import() {
        let expr = make_expr(ExprKind::Import {
            module: "test".to_string(),
            items: None,
        });
        assert!(needs_module_resolution(&expr));
    }

    #[test]
    fn test_needs_module_resolution_with_module_decl() {
        let expr = make_expr(ExprKind::ModuleDeclaration {
            name: "test".to_string(),
        });
        assert!(needs_module_resolution(&expr));
    }

    #[test]
    fn test_needs_module_resolution_with_import_all() {
        let expr = make_expr(ExprKind::ImportAll {
            module: "test".to_string(),
            alias: "t".to_string(),
        });
        assert!(needs_module_resolution(&expr));
    }

    #[test]
    fn test_needs_module_resolution_nested_in_block() {
        let import = make_expr(ExprKind::Import {
            module: "test".to_string(),
            items: None,
        });
        let block = make_expr(ExprKind::Block(vec![import]));
        assert!(needs_module_resolution(&block));
    }

    #[test]
    fn test_resolve_modules_for_execution_nonexistent_path() {
        let expr = make_expr(ExprKind::Block(vec![]));
        let path = PathBuf::from("/nonexistent/dir/file.ruchy");
        // Should succeed for empty block (no modules to resolve)
        let result = resolve_modules_for_execution(&path, expr);
        assert!(result.is_ok());
    }

    // ===== EXTREME TDD Round 152 - Execution Handler Tests =====

    #[test]
    fn test_needs_module_resolution_simple_literal() {
        let expr = make_expr(ExprKind::Literal(ruchy::frontend::ast::Literal::Integer(42, None)));
        assert!(!needs_module_resolution(&expr));
    }

    #[test]
    fn test_needs_module_resolution_import_default() {
        let expr = make_expr(ExprKind::ImportDefault {
            module: "test".to_string(),
            name: "t".to_string(),
        });
        assert!(needs_module_resolution(&expr));
    }

    #[test]
    fn test_needs_module_resolution_module() {
        let inner = make_expr(ExprKind::Block(vec![]));
        let expr = make_expr(ExprKind::Module {
            name: "test".to_string(),
            body: Box::new(inner),
        });
        assert!(needs_module_resolution(&expr));
    }

    #[test]
    fn test_needs_module_resolution_let_binding() {
        let val = make_expr(ExprKind::Literal(ruchy::frontend::ast::Literal::Integer(1, None)));
        let body = make_expr(ExprKind::Literal(ruchy::frontend::ast::Literal::Integer(2, None)));
        let expr = make_expr(ExprKind::Let {
            name: "x".to_string(),
            is_mutable: false,
            type_annotation: None,
            value: Box::new(val),
            body: Box::new(body),
            else_block: None,
        });
        assert!(!needs_module_resolution(&expr));
    }

    #[test]
    fn test_needs_module_resolution_let_with_import_body() {
        let val = make_expr(ExprKind::Literal(ruchy::frontend::ast::Literal::Integer(1, None)));
        let body = make_expr(ExprKind::Import {
            module: "test".to_string(),
            items: None,
        });
        let expr = make_expr(ExprKind::Let {
            name: "x".to_string(),
            is_mutable: false,
            type_annotation: None,
            value: Box::new(val),
            body: Box::new(body),
            else_block: None,
        });
        assert!(needs_module_resolution(&expr));
    }

    #[test]
    fn test_needs_module_resolution_function_with_import() {
        let body = make_expr(ExprKind::Import {
            module: "test".to_string(),
            items: None,
        });
        let expr = make_expr(ExprKind::Function {
            name: "foo".to_string(),
            type_params: vec![],
            params: vec![],
            return_type: None,
            body: Box::new(body),
            is_async: false,
            is_pub: false,
        });
        assert!(needs_module_resolution(&expr));
    }

    #[test]
    fn test_handle_stdin_input_error() {
        let result = handle_stdin_input("invalid syntax {");
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_stdin_input_valid_expression() {
        let result = handle_stdin_input("42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_modules_with_parent_directory() {
        let expr = make_expr(ExprKind::Block(vec![]));
        let path = PathBuf::from("/root/src/test.ruchy");
        let result = resolve_modules_for_execution(&path, expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_modules_for_execution_relative_path() {
        let expr = make_expr(ExprKind::Block(vec![]));
        let path = PathBuf::from("test.ruchy");
        let result = resolve_modules_for_execution(&path, expr);
        assert!(result.is_ok());
    }
}
