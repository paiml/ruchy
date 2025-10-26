// Module Resolution Infrastructure (PARSER-060)
//
// Scope: File resolution, loading, symbol extraction, imports
// Architecture: Resolver → Loader → Extractor → Importer → Cache

use crate::frontend::ast::{Expr, ExprKind};
use crate::frontend::parser::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::cell::RefCell;

/// Errors that can occur during module resolution
#[derive(Debug, Clone)]
pub enum ModuleError {
    FileNotFound(PathBuf),
    ParseError(String),
    SymbolNotFound { module: String, symbol: String },
    IoError(String),
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path) => write!(f, "Module file not found: {}", path.display()),
            Self::ParseError(msg) => write!(f, "Parse error in module: {msg}"),
            Self::SymbolNotFound { module, symbol } => {
                write!(f, "Symbol '{symbol}' not found in module '{module}'")
            }
            Self::IoError(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for ModuleError {}

/// A loaded and parsed module with extracted symbols
#[derive(Debug, Clone)]
pub struct LoadedModule {
    path: PathBuf,
    ast: Expr,
    symbols: HashMap<String, Symbol>,
}

impl LoadedModule {
    pub fn new(path: PathBuf, ast: Expr) -> Self {
        let symbols = extract_symbols_from_ast(&ast);
        Self { path, ast, symbols }
    }

    pub fn is_loaded(&self) -> bool {
        true // If we constructed it, it's loaded
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn symbols(&self) -> &HashMap<String, Symbol> {
        &self.symbols
    }

    pub fn get_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn ast(&self) -> &Expr {
        &self.ast
    }
}

/// Symbol types that can be imported from modules
#[derive(Debug, Clone)]
pub enum Symbol {
    Function(FunctionSymbol),
    Struct(StructSymbol),
    Const(ConstSymbol),
}

#[derive(Debug, Clone)]
pub struct FunctionSymbol {
    pub name: String,
    pub expr: Expr, // The function definition expression
}

#[derive(Debug, Clone)]
pub struct StructSymbol {
    pub name: String,
    pub expr: Expr, // The struct definition expression
}

#[derive(Debug, Clone)]
pub struct ConstSymbol {
    pub name: String,
    pub expr: Expr, // The const definition expression
}

/// Extracted symbols grouped by type
#[derive(Debug, Default)]
pub struct AllSymbols {
    pub functions: Vec<FunctionSymbol>,
    pub structs: Vec<StructSymbol>,
    pub consts: Vec<ConstSymbol>,
}

/// Module cache to avoid re-parsing the same files
pub struct ModuleCache {
    cache: RefCell<HashMap<PathBuf, Rc<LoadedModule>>>,
}

impl ModuleCache {
    pub fn new() -> Self {
        Self {
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn load(&self, path: &PathBuf) -> Result<Rc<LoadedModule>, ModuleError> {
        // Check cache first
        if let Some(cached) = self.cache.borrow().get(path) {
            return Ok(Rc::clone(cached));
        }

        // Load and parse the module
        let module = load_module_from_file(path)?;
        let module_rc = Rc::new(module);

        // Cache it
        self.cache
            .borrow_mut()
            .insert(path.clone(), Rc::clone(&module_rc));

        Ok(module_rc)
    }
}

impl Default for ModuleCache {
    fn default() -> Self {
        Self::new()
    }
}

// -------------------------------------------------------------------
// Module Resolution (use foo::bar → ./foo/bar.ruchy)
// -------------------------------------------------------------------

/// Resolve Rust-style module path to filesystem path
/// Example: `foo::bar` → `./foo/bar.ruchy`
pub fn resolve_module_path(module: &str, base: &Path) -> Result<PathBuf, ModuleError> {
    let path_parts: Vec<&str> = module.split("::").collect();
    let mut path = base.to_path_buf();

    // Navigate nested modules
    for (i, part) in path_parts.iter().enumerate() {
        if i == path_parts.len() - 1 {
            // Last part: add .ruchy extension
            path.push(format!("{part}.ruchy"));
        } else {
            // Intermediate part: directory
            path.push(part);
        }
    }

    // Verify file exists
    if !path.exists() {
        return Err(ModuleError::FileNotFound(path));
    }

    Ok(path)
}

/// Resolve Python-style module path (dot notation) to filesystem path
/// Example: `foo.bar` → `./foo/bar.ruchy`
pub fn resolve_module_path_dot_notation(module: &str, base: &Path) -> Result<PathBuf, ModuleError> {
    let path_parts: Vec<&str> = module.split('.').collect();
    let mut path = base.to_path_buf();

    for (i, part) in path_parts.iter().enumerate() {
        if i == path_parts.len() - 1 {
            path.push(format!("{part}.ruchy"));
        } else {
            path.push(part);
        }
    }

    if !path.exists() {
        return Err(ModuleError::FileNotFound(path));
    }

    Ok(path)
}

// -------------------------------------------------------------------
// Module Loading & Parsing
// -------------------------------------------------------------------

/// Load and parse a module from a file
pub fn load_module_from_file(path: &Path) -> Result<LoadedModule, ModuleError> {
    // Read file contents
    let source = fs::read_to_string(path)
        .map_err(|e| ModuleError::IoError(format!("Failed to read {}: {e}", path.display())))?;

    // Parse the source code
    let ast = Parser::new(&source)
        .parse()
        .map_err(|e| ModuleError::ParseError(format!("{e:?}")))?;

    Ok(LoadedModule::new(path.to_path_buf(), ast))
}

// For backward compatibility with test naming
pub fn load_module(path: &Path) -> Result<LoadedModule, String> {
    load_module_from_file(path).map_err(|e| e.to_string())
}

// -------------------------------------------------------------------
// Symbol Extraction
// -------------------------------------------------------------------

/// Extract all symbols (functions, structs, consts) from a parsed AST
fn extract_symbols_from_ast(ast: &Expr) -> HashMap<String, Symbol> {
    let mut symbols = HashMap::new();

    // Get list of expressions to process
    let exprs: Vec<&Expr> = match &ast.kind {
        ExprKind::Block(exprs) => exprs.iter().collect(),
        _ => vec![ast],  // Single expression
    };

    for expr in exprs {
        match &expr.kind {
            // Function definitions
            ExprKind::Function { name, .. } => {
                symbols.insert(
                    name.clone(),
                    Symbol::Function(FunctionSymbol {
                        name: name.clone(),
                        expr: expr.clone(),
                    }),
                );
            }

            // Struct definitions
            ExprKind::Struct { name, .. } => {
                symbols.insert(
                    name.clone(),
                    Symbol::Struct(StructSymbol {
                        name: name.clone(),
                        expr: expr.clone(),
                    }),
                );
            }

            // Let bindings (includes const)
            ExprKind::Let { name, .. } => {
                symbols.insert(
                    name.clone(),
                    Symbol::Const(ConstSymbol {
                        name: name.clone(),
                        expr: expr.clone(),
                    }),
                );
            }

            _ => {} // Ignore other expression types
        }
    }

    symbols
}

/// Extract only function symbols from a module
pub fn extract_functions(module: &LoadedModule) -> Vec<FunctionSymbol> {
    module
        .symbols
        .values()
        .filter_map(|sym| {
            if let Symbol::Function(f) = sym {
                Some(f.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Extract only struct symbols from a module
pub fn extract_structs(module: &LoadedModule) -> Vec<StructSymbol> {
    module
        .symbols
        .values()
        .filter_map(|sym| {
            if let Symbol::Struct(s) = sym {
                Some(s.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Extract only const symbols from a module
pub fn extract_consts(module: &LoadedModule) -> Vec<ConstSymbol> {
    module
        .symbols
        .values()
        .filter_map(|sym| {
            if let Symbol::Const(c) = sym {
                Some(c.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Extract all symbols grouped by type
pub fn extract_all_symbols(module: &LoadedModule) -> AllSymbols {
    AllSymbols {
        functions: extract_functions(module),
        structs: extract_structs(module),
        consts: extract_consts(module),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_simple_module() {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join("utils.ruchy");
        fs::write(&module_path, "fun test() { 1 }").unwrap();

        let resolved = resolve_module_path("utils", temp_dir.path()).unwrap();
        assert_eq!(resolved, module_path);
    }

    #[test]
    fn test_resolve_nested_module() {
        let temp_dir = TempDir::new().unwrap();
        let foo_dir = temp_dir.path().join("foo");
        fs::create_dir(&foo_dir).unwrap();
        let bar_path = foo_dir.join("bar.ruchy");
        fs::write(&bar_path, "fun test() { 1 }").unwrap();

        let resolved = resolve_module_path("foo::bar", temp_dir.path()).unwrap();
        assert_eq!(resolved, bar_path);
    }

    #[test]
    fn test_load_simple_module() {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "fun add(x, y) { x + y }").unwrap();

        let module = load_module_from_file(&module_path).unwrap();
        assert_eq!(module.path(), &module_path);
        assert!(module.is_loaded());
        assert_eq!(module.symbols().len(), 1);
        assert!(module.symbols().contains_key("add"));
    }

    #[test]
    fn test_extract_multiple_functions() {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join("math.ruchy");
        fs::write(
            &module_path,
            "fun add(x, y) { x + y }\nfun sub(x, y) { x - y }",
        )
        .unwrap();

        let module = load_module_from_file(&module_path).unwrap();
        let functions = extract_functions(&module);

        assert_eq!(functions.len(), 2);
        assert!(functions.iter().any(|f| f.name == "add"));
        assert!(functions.iter().any(|f| f.name == "sub"));
    }

    #[test]
    fn test_module_cache() {
        let temp_dir = TempDir::new().unwrap();
        let module_path = temp_dir.path().join("cached.ruchy");
        fs::write(&module_path, "fun test() { 1 }").unwrap();

        let cache = ModuleCache::new();
        let module1 = cache.load(&module_path).unwrap();
        let module2 = cache.load(&module_path).unwrap();

        // Same Rc pointer means same cached instance
        assert!(Rc::ptr_eq(&module1, &module2));
    }
}
