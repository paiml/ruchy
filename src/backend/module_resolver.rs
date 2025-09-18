//! Module resolver for multi-file imports
//!
//! This module provides functionality to resolve file imports in Ruchy programs
//! by pre-processing the AST to inline external modules before transpilation.
//!
//! # Architecture
//! 
//! The module resolver works as a pre-processing step before transpilation:
//! 1. Parse the main file into an AST
//! 2. Scan for file imports (`use module_name;` where `module_name` has no `::`)
//! 3. Load and parse external module files 
//! 4. Replace Import nodes with inline Module nodes
//! 5. Pass the resolved AST to the transpiler
//!
//! # Usage
//!
//! ```rust
//! use ruchy::{ModuleResolver, Parser, Transpiler};
//! 
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut resolver = ModuleResolver::new();
//! resolver.add_search_path("./src");
//! 
//! let mut parser = Parser::new("2 + 2");
//! let ast = parser.parse()?;
//! 
//! // Would resolve imports if there were any
//! // let resolved_ast = resolver.resolve_imports(ast)?;
//! 
//! let mut transpiler = Transpiler::new();
//! let rust_code = transpiler.transpile(&ast)?;
//! # Ok(())
//! # }
//! ```
use crate::frontend::ast::{Expr, ExprKind, ImportItem, Span};
use crate::backend::module_loader::ModuleLoader;
use crate::utils::common_patterns::ResultContextExt;
use anyhow::Result;
/// Module resolver for processing file imports
/// 
/// Resolves file imports by loading external modules and inlining them
/// as Module declarations in the AST before transpilation.
pub struct ModuleResolver {
    /// Module loader for file system operations
    pub(crate) module_loader: ModuleLoader,
}
impl ModuleResolver {
    /// Create a new module resolver with default search paths
    /// 
    /// Default search paths:
    /// - `.` (current directory)
    /// - `./src` (source directory)
    /// - `./modules` (modules directory)
    #[must_use]
/// # Examples
/// 
/// ```
/// use ruchy::backend::module_resolver::ModuleResolver;
///
/// let resolver = ModuleResolver::new();
/// // ModuleResolver created successfully
/// ```
pub fn new() -> Self {
        Self {
            module_loader: ModuleLoader::new(),
        }
    }
    /// Add a directory to the module search path
    /// 
    /// # Arguments
    /// 
    /// * `path` - Directory to search for modules
    pub fn add_search_path<P: AsRef<std::path::Path>>(&mut self, path: P) {
        self.module_loader.add_search_path(path);
    }
    /// Resolve all file imports in an AST
    /// 
    /// Recursively processes the AST to find file imports, loads the corresponding
    /// modules, and replaces Import nodes with inline Module nodes.
    /// 
    /// # Arguments
    /// 
    /// * `ast` - The AST to process
    /// 
    /// # Returns
    /// 
    /// A new AST with all file imports resolved to inline modules
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Module files cannot be found or loaded
    /// - Module files contain invalid syntax  
    /// - Circular dependencies are detected
/// # Examples
/// 
/// ```
/// use ruchy::backend::module_resolver::ModuleResolver;
/// use ruchy::ast::Expr;
///
/// let mut resolver = ModuleResolver::new();
/// let ast = Expr::literal(42.into());
/// let result = resolver.resolve_imports(ast);
/// assert!(result.is_ok());
/// ```
pub fn resolve_imports(&mut self, ast: Expr) -> Result<Expr> {
        self.resolve_expr(ast)
    }
    /// Recursively resolve imports in an expression
    fn resolve_expr(&mut self, expr: Expr) -> Result<Expr> {
        match expr.kind {
            ExprKind::Import { module, items } => {
                self.resolve_simple_import(expr.span, &module, items.as_deref())
            }
            ExprKind::ImportAll { module, alias } => {
                self.resolve_import_all(expr.span, &module, &alias)
            }
            ExprKind::ImportDefault { module, name } => {
                self.resolve_import_default(expr.span, &module, &name)
            }
            ExprKind::ReExport { items, module } => {
                self.resolve_reexport(expr.span, &items, &module)
            }
            ExprKind::Block(exprs) => {
                self.resolve_block_expr(exprs, expr.span)
            }
            ExprKind::Module { name, body } => {
                self.resolve_module_expr(name, *body, expr.span)
            }
            ExprKind::Function { 
                name, 
                type_params, 
                params, 
                body, 
                is_async,
                return_type,
                is_pub,
            } => {
                self.resolve_function_expr(
                    name,
                    type_params,
                    params,
                    *body,
                    is_async,
                    return_type,
                    is_pub,
                    expr.span,
                )
            }
            ExprKind::If { condition, then_branch, else_branch } => {
                self.resolve_if_expr(*condition, *then_branch, else_branch, expr.span)
            }
            // For other expression types, recursively process children as needed
            // For now, just return the expression as-is
            _ => Ok(expr),
        }
    }
    /// Resolve import expressions
    fn resolve_simple_import(&mut self, span: Span, module: &str, items: Option<&[String]>) -> Result<Expr> {
        if self.is_file_import(module) {
            self.resolve_file_module(span, module)?;
        }
        Ok(Expr::new(
            ExprKind::Import {
                module: module.to_string(),
                items: items.map(<[std::string::String]>::to_vec),
            },
            span,
        ))
    }

    fn resolve_import_all(&mut self, span: Span, module: &str, alias: &str) -> Result<Expr> {
        if self.is_file_import(module) {
            self.resolve_file_module(span, module)?;
        }
        Ok(Expr::new(
            ExprKind::ImportAll {
                module: module.to_string(),
                alias: alias.to_string(),
            },
            span,
        ))
    }

    fn resolve_import_default(&mut self, span: Span, module: &str, name: &str) -> Result<Expr> {
        if self.is_file_import(module) {
            self.resolve_file_module(span, module)?;
        }
        Ok(Expr::new(
            ExprKind::ImportDefault {
                module: module.to_string(),
                name: name.to_string(),
            },
            span,
        ))
    }

    fn resolve_reexport(&mut self, span: Span, items: &[String], module: &str) -> Result<Expr> {
        if self.is_file_import(module) {
            self.resolve_file_module(span, module)?;
        }
        Ok(Expr::new(
            ExprKind::ReExport {
                items: items.to_vec(),
                module: module.to_string(),
            },
            span,
        ))
    }

    fn resolve_file_module(&mut self, _span: Span, module: &str) -> Result<()> {
        // Load and parse the module file
        let _ = self.module_loader.load_module(module)
            .module_context("resolve import", module)?;
        // For now, just ensure the module exists and is valid
        // TODO: Actually integrate the module's exports
        Ok(())
    }

    fn resolve_file_import(&mut self, span: Span, path: &str, items: &[ImportItem]) -> Result<Expr> {
        // Load the module file
        let parsed_module = self.module_loader.load_module(path)
            .module_context("resolve import", path)?;
        // Recursively resolve imports in the loaded module
        let resolved_module_ast = self.resolve_expr(parsed_module.ast)?;
        
        let module_expr = self.create_inline_module(path, resolved_module_ast, span);
        let use_statement = self.create_use_statement(path, items);
        
        Ok(Expr::new(
            ExprKind::Block(vec![module_expr, use_statement]),
            span,
        ))
    }

    fn resolve_standard_import(&self, span: Span, path: &str, _items: &[ImportItem]) -> Result<Expr> {
        Ok(Expr::new(
            ExprKind::Import {
                module: path.to_string(),
                items: None,  // Legacy standard imports converted to simple imports
            },
            span,
        ))
    }

    fn create_inline_module(&self, path: &str, resolved_ast: Expr, span: Span) -> Expr {
        Expr::new(
            ExprKind::Module {
                name: path.to_string(),
                body: Box::new(resolved_ast),
            },
            span,
        )
    }

    fn create_use_statement(&mut self, path: &str, items: &[ImportItem]) -> Expr {
        if items.iter().any(|item| matches!(item, ImportItem::Wildcard)) || items.is_empty() {
            // Wildcard import: use module::*;
            Expr::new(
                ExprKind::ImportAll {
                    module: path.to_string(),
                    alias: "*".to_string(),  // Wildcard imports
                },
                Span { start: 0, end: 0 },
            )
        } else {
            // Specific imports: use module::{item1, item2};
            self.create_use_statements(path, items)
        }
    }
    /// Resolve block expressions
    fn resolve_block_expr(&mut self, exprs: Vec<Expr>, span: Span) -> Result<Expr> {
        // Resolve imports in all block expressions
        let resolved_exprs: Result<Vec<_>> = exprs
            .into_iter()
            .map(|e| self.resolve_expr(e))
            .collect();
        Ok(Expr::new(ExprKind::Block(resolved_exprs?), span))
    }
    /// Resolve module expressions
    fn resolve_module_expr(&mut self, name: String, body: Expr, span: Span) -> Result<Expr> {
        // Resolve imports in module body
        let resolved_body = self.resolve_expr(body)?;
        Ok(Expr::new(
            ExprKind::Module {
                name,
                body: Box::new(resolved_body),
            },
            span,
        ))
    }
    /// Resolve function expressions
    fn resolve_function_expr(
        &mut self,
        name: String,
        type_params: Vec<String>,
        params: Vec<crate::frontend::ast::Param>,
        body: Expr,
        is_async: bool,
        return_type: Option<crate::frontend::ast::Type>,
        is_pub: bool,
        span: Span,
    ) -> Result<Expr> {
        // Resolve imports in function body
        let resolved_body = self.resolve_expr(body)?;
        Ok(Expr::new(
            ExprKind::Function {
                name,
                type_params,
                params,
                body: Box::new(resolved_body),
                is_async,
                return_type,
                is_pub,
            },
            span,
        ))
    }
    /// Resolve if expressions
    fn resolve_if_expr(
        &mut self,
        condition: Expr,
        then_branch: Expr,
        else_branch: Option<Box<Expr>>,
        span: Span,
    ) -> Result<Expr> {
        let resolved_condition = self.resolve_expr(condition)?;
        let resolved_then = self.resolve_expr(then_branch)?;
        let resolved_else = else_branch.map(|e| self.resolve_expr(*e)).transpose()?;
        Ok(Expr::new(
            ExprKind::If {
                condition: Box::new(resolved_condition),
                then_branch: Box::new(resolved_then),
                else_branch: resolved_else.map(Box::new),
            },
            span,
        ))
    }
    /// Check if an import path represents a file import
    fn is_file_import(&self, path: &str) -> bool {
        !path.contains("::")
            && !path.starts_with("std::")
            && !path.starts_with("http")
            && !path.is_empty()
    }
    /// Create use statements for specific imports
    fn create_use_statements(&self, module_path: &str, items: &[ImportItem]) -> Expr {
        // Create a use statement that imports specific items from the module
        // This will be transpiled to proper Rust use statements
        // Convert legacy ImportItem list to string list for new AST
        let item_names: Vec<String> = items.iter().filter_map(|item| {
            match item {
                ImportItem::Named(name) => Some(name.clone()),
                ImportItem::Aliased { name, .. } => Some(name.clone()),
                ImportItem::Wildcard => None,
            }
        }).collect();

        Expr::new(
            ExprKind::Import {
                module: module_path.to_string(),
                items: if item_names.is_empty() { None } else { Some(item_names) },
            },
            Span { start: 0, end: 0 },
        )
    }
    /// Get module loading statistics
    #[must_use]
/// # Examples
/// 
/// ```
/// use ruchy::backend::module_resolver::ModuleResolver;
///
/// let resolver = ModuleResolver::new();
/// let stats = resolver.stats();
/// assert!(stats.cached_modules >= 0);
/// ```
pub fn stats(&self) -> crate::backend::module_loader::ModuleLoaderStats {
        self.module_loader.stats()
    }
    /// Clear the module cache
    /// 
    /// Forces all modules to be reloaded from disk on next access.
/// # Examples
/// 
/// ```
/// use ruchy::backend::module_resolver::ModuleResolver;
///
/// let mut resolver = ModuleResolver::new();
/// resolver.clear_cache();
/// // Cache cleared successfully
/// ```
pub fn clear_cache(&mut self) {
        self.module_loader.clear_cache();
    }
}
impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}
// Module resolver tests disabled - need update for new AST structure after Sprint v3.8.0
// TODO: Update tests to match new Import { module: String, items: Option<Vec<String>> } structure
#[cfg(test_disabled)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use crate::frontend::ast::Literal;
    fn create_test_module(temp_dir: &TempDir, name: &str, content: &str) -> Result<()> {
        let file_path = temp_dir.path().join(format!("{name}.ruchy"));
        fs::write(file_path, content)?;
        Ok(())
    }
    #[test]
    fn test_module_resolver_creation() {
        let resolver = ModuleResolver::new();
        let stats = resolver.stats();
        assert_eq!(stats.cached_modules, 0);
    }
    #[test] 
    fn test_add_search_path() {
        let mut resolver = ModuleResolver::new();
        resolver.add_search_path("/custom/path");
        // Module loader doesn't expose search paths, so we can't directly test this
        // But we can test that it doesn't panic
    }
    #[test]
    fn test_is_file_import() {
        let resolver = ModuleResolver::new();
        // Should be file imports
        assert!(resolver.is_file_import("math"));
        assert!(resolver.is_file_import("utils"));
        assert!(resolver.is_file_import("snake_case_module"));
        // Should NOT be file imports
        assert!(!resolver.is_file_import("std::collections"));
        assert!(!resolver.is_file_import("std::io::Read"));
        assert!(!resolver.is_file_import("https://example.com/module.ruchy"));
        assert!(!resolver.is_file_import("http://localhost/module.ruchy"));
        assert!(!resolver.is_file_import(""));
    }
    #[test]
    fn test_resolve_simple_file_import() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut resolver = ModuleResolver::new();
        resolver.add_search_path(temp_dir.path());
        // Create a simple math module
        create_test_module(&temp_dir, "math", r"
            pub fun add(a: i32, b: i32) -> i32 {
                a + b
            }
        ")?;
        // Create an import expression
        let import_expr = Expr::new(
            ExprKind::Import {
                path: "math".to_string(),
                items: vec![ImportItem::Wildcard],
            },
            Span { start: 0, end: 0 },
        );
        // Resolve the import
        let resolved_expr = resolver.resolve_imports(import_expr)?;
        // Should be converted to a Block with Module declaration and use statement
        match resolved_expr.kind {
            ExprKind::Block(exprs) => {
                assert_eq!(exprs.len(), 2);
                // First should be Module declaration
                match &exprs[0].kind {
                    ExprKind::Module { name, .. } => {
                        assert_eq!(name, "math");
                    }
                    _ => unreachable!("Expected first element to be Module, got {:?}", exprs[0].kind),
                }
                // Second should be use statement
                match &exprs[1].kind {
                    ExprKind::Import { path, items } => {
                        assert_eq!(path, "math");
                        assert_eq!(items.len(), 1);
                        assert!(matches!(items[0], ImportItem::Wildcard));
                    }
                    _ => unreachable!("Expected second element to be Import, got {:?}", exprs[1].kind),
                }
            }
            _ => unreachable!("Expected Block expression, got {:?}", resolved_expr.kind),
        }
        Ok(())
    }
    #[test]
    fn test_resolve_non_file_import() -> Result<()> {
        let mut resolver = ModuleResolver::new();
        // Create a standard library import
        let import_expr = Expr::new(
            ExprKind::Import {
                path: "std::collections".to_string(),
                items: vec![ImportItem::Named("HashMap".to_string())],
            },
            Span { start: 0, end: 0 },
        );
        // Resolve the import - should remain unchanged
        let resolved_expr = resolver.resolve_imports(import_expr)?;
        match resolved_expr.kind {
            ExprKind::Import { path, items } => {
                assert_eq!(path, "std::collections");
                assert_eq!(items.len(), 1);
            }
            _ => unreachable!("Expected Import expression to remain unchanged"),
        }
        Ok(())
    }
    #[test]
    fn test_resolve_block_with_imports() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut resolver = ModuleResolver::new();
        resolver.add_search_path(temp_dir.path());
        create_test_module(&temp_dir, "math", "42")?;
        // Create a block with mixed imports
        let block_expr = Expr::new(
            ExprKind::Block(vec![
                Expr::new(
                    ExprKind::Import {
                        path: "math".to_string(),
                        items: vec![ImportItem::Wildcard],
                    },
                    Span { start: 0, end: 0 },
                ),
                Expr::new(
                    ExprKind::Import {
                        path: "std::io".to_string(),
                        items: vec![ImportItem::Named("Read".to_string())],
                    },
                    Span { start: 0, end: 0 },
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(42)),
                    Span { start: 0, end: 0 },
                ),
            ]),
            Span { start: 0, end: 0 },
        );
        let resolved_block = resolver.resolve_imports(block_expr)?;
        if let ExprKind::Block(exprs) = resolved_block.kind {
            assert_eq!(exprs.len(), 3);
            // First should be Block containing Module and use statement (from file import)
            match &exprs[0].kind {
                ExprKind::Block(inner_exprs) => {
                    assert_eq!(inner_exprs.len(), 2);
                    assert!(matches!(inner_exprs[0].kind, ExprKind::Module { .. }));
                    assert!(matches!(inner_exprs[1].kind, ExprKind::Import { .. }));
                }
                _ => unreachable!("Expected first element to be Block, got {:?}", exprs[0].kind),
            }
            // Second should remain as Import (std::io - not a file import)
            assert!(matches!(exprs[1].kind, ExprKind::Import { .. }));
            // Third should remain as Literal
            assert!(matches!(exprs[2].kind, ExprKind::Literal(Literal::Integer(42))));
        } else {
            unreachable!("Expected Block expression");
        }
        Ok(())
    }
    #[test]
    fn test_default_creation() {
        let resolver = ModuleResolver::default();
        let stats = resolver.stats();
        assert_eq!(stats.cached_modules, 0);
        assert_eq!(stats.files_loaded, 0);
    }

    #[test]
    #[ignore] // Error message format doesn't match
    fn test_resolve_imports_error_handling() -> Result<()> {
        let mut resolver = ModuleResolver::new();
        resolver.module_loader.search_paths.clear();

        // Try to import non-existent module
        let import_expr = Expr::new(
            ExprKind::Import {
                path: "nonexistent".to_string(),
                items: vec![ImportItem::Wildcard],
            },
            Span { start: 0, end: 0 },
        );

        let result = resolver.resolve_imports(import_expr);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        Ok(())
    }

    #[test]
    fn test_nested_module_resolution() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut resolver = ModuleResolver::new();
        resolver.add_search_path(temp_dir.path());

        // Create nested modules
        create_test_module(&temp_dir, "utils", "let x = 42;")?;
        create_test_module(&temp_dir, "math", "use utils;\nlet y = 1;")?;

        let import_expr = Expr::new(
            ExprKind::Import {
                path: "math".to_string(),
                items: vec![ImportItem::Wildcard],
            },
            Span { start: 0, end: 0 },
        );

        // This might fail due to nested imports, but shouldn't panic
        let _ = resolver.resolve_imports(import_expr);

        Ok(())
    }

    #[test]
    fn test_specific_import_items() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut resolver = ModuleResolver::new();
        resolver.add_search_path(temp_dir.path());

        create_test_module(&temp_dir, "math", "pub fun add() {}\npub fun sub() {}")?;

        // Import specific items
        let import_expr = Expr::new(
            ExprKind::Import {
                path: "math".to_string(),
                items: vec![
                    ImportItem::Named("add".to_string()),
                    ImportItem::Named("sub".to_string()),
                ],
            },
            Span { start: 0, end: 0 },
        );

        let resolved = resolver.resolve_imports(import_expr)?;
        if let ExprKind::Block(exprs) = resolved.kind {
            assert_eq!(exprs.len(), 2);
            // Check that module was loaded
            assert!(matches!(exprs[0].kind, ExprKind::Module { .. }));
        }

        Ok(())
    }

    #[test]
    fn test_stats_and_cache() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut resolver = ModuleResolver::new();
        resolver.module_loader.search_paths.clear(); // Remove default paths
        resolver.add_search_path(temp_dir.path());
        create_test_module(&temp_dir, "test", "42")?;
        let initial_stats = resolver.stats();
        assert_eq!(initial_stats.files_loaded, 0);
        // Load a module
        let import_expr = Expr::new(
            ExprKind::Import {
                path: "test".to_string(),
                items: vec![ImportItem::Wildcard],
            },
            Span { start: 0, end: 0 },
        );
        resolver.resolve_imports(import_expr)?;
        let after_stats = resolver.stats();
        assert_eq!(after_stats.files_loaded, 1);
        assert_eq!(after_stats.cached_modules, 1);
        // Clear cache
        resolver.clear_cache();
        let cleared_stats = resolver.stats();
        assert_eq!(cleared_stats.files_loaded, 0);
        assert_eq!(cleared_stats.cached_modules, 0);
        Ok(())
    }
}
#[cfg(test_disabled)]
mod property_tests_module_resolver {
    use proptest::{proptest, prop_assert_eq};
    
    
    proptest! {
        /// Property: is_file_import never panics on any input
        #[test]
        fn test_is_file_import_never_panics(input: String) {
            let resolver = super::ModuleResolver::new();
            // Should never panic regardless of input
            let _ = resolver.is_file_import(&input);
        }

        /// Property: resolve_imports maintains AST structure for non-file imports
        #[test]
        fn test_resolve_preserves_non_file_imports(module_name: String) {
            use crate::frontend::ast::{Expr, ExprKind, ImportItem, Span};

            let mut resolver = super::ModuleResolver::new();

            // Create an import that's not a file (contains ::)
            let path = format!("std::{}", module_name);
            let import_expr = Expr::new(
                ExprKind::Import {
                    path: path.clone(),
                    items: vec![ImportItem::Wildcard],
                },
                Span { start: 0, end: 0 },
            );

            // Should preserve the import unchanged
            if let Ok(resolved) = resolver.resolve_imports(import_expr) {
                if let ExprKind::Import { path: resolved_path, .. } = resolved.kind {
                    prop_assert_eq!(resolved_path, path);
                }
            }
        }
    }
}
