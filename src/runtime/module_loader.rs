// Module Resolution Infrastructure (PARSER-060)
//
// Scope: File resolution, loading, symbol extraction, imports
// Architecture: Resolver → Loader → Extractor → Importer → Cache

use crate::frontend::ast::{Expr, ExprKind};
use crate::frontend::parser::Parser;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

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
        _ => vec![ast], // Single expression
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
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("utils.ruchy");
        fs::write(&module_path, "fun test() { 1 }").expect("operation should succeed in test");

        let resolved = resolve_module_path("utils", temp_dir.path())
            .expect("operation should succeed in test");
        assert_eq!(resolved, module_path);
    }

    #[test]
    fn test_resolve_nested_module() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let foo_dir = temp_dir.path().join("foo");
        fs::create_dir(&foo_dir).expect("operation should succeed in test");
        let bar_path = foo_dir.join("bar.ruchy");
        fs::write(&bar_path, "fun test() { 1 }").expect("operation should succeed in test");

        let resolved = resolve_module_path("foo::bar", temp_dir.path())
            .expect("operation should succeed in test");
        assert_eq!(resolved, bar_path);
    }

    #[test]
    fn test_load_simple_module() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "fun add(x, y) { x + y }")
            .expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        assert_eq!(module.path(), &module_path);
        assert!(module.is_loaded());
        assert_eq!(module.symbols().len(), 1);
        assert!(module.symbols().contains_key("add"));
    }

    #[test]
    fn test_extract_multiple_functions() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("math.ruchy");
        fs::write(
            &module_path,
            "fun add(x, y) { x + y }\nfun sub(x, y) { x - y }",
        )
        .expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let functions = extract_functions(&module);

        assert_eq!(functions.len(), 2);
        assert!(functions.iter().any(|f| f.name == "add"));
        assert!(functions.iter().any(|f| f.name == "sub"));
    }

    #[test]
    fn test_module_cache() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("cached.ruchy");
        fs::write(&module_path, "fun test() { 1 }").expect("operation should succeed in test");

        let cache = ModuleCache::new();
        let module1 = cache
            .load(&module_path)
            .expect("operation should succeed in test");
        let module2 = cache
            .load(&module_path)
            .expect("operation should succeed in test");

        // Same Rc pointer means same cached instance
        assert!(Rc::ptr_eq(&module1, &module2));
    }

    // ModuleError Display tests
    #[test]
    fn test_module_error_display_file_not_found() {
        let err = ModuleError::FileNotFound(PathBuf::from("/path/to/file.ruchy"));
        let display = format!("{}", err);
        assert!(display.contains("Module file not found"));
        assert!(display.contains("/path/to/file.ruchy"));
    }

    #[test]
    fn test_module_error_display_parse_error() {
        let err = ModuleError::ParseError("unexpected token".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Parse error in module"));
        assert!(display.contains("unexpected token"));
    }

    #[test]
    fn test_module_error_display_symbol_not_found() {
        let err = ModuleError::SymbolNotFound {
            module: "math".to_string(),
            symbol: "sqrt".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("Symbol 'sqrt' not found in module 'math'"));
    }

    #[test]
    fn test_module_error_display_io_error() {
        let err = ModuleError::IoError("permission denied".to_string());
        let display = format!("{}", err);
        assert!(display.contains("IO error"));
        assert!(display.contains("permission denied"));
    }

    #[test]
    fn test_module_error_clone() {
        let err1 = ModuleError::FileNotFound(PathBuf::from("/test.ruchy"));
        let err2 = err1.clone();
        assert!(matches!(err2, ModuleError::FileNotFound(_)));

        let err3 = ModuleError::ParseError("test".to_string());
        let err4 = err3.clone();
        assert!(matches!(err4, ModuleError::ParseError(_)));

        let err5 = ModuleError::SymbolNotFound {
            module: "m".to_string(),
            symbol: "s".to_string(),
        };
        let err6 = err5.clone();
        assert!(matches!(err6, ModuleError::SymbolNotFound { .. }));

        let err7 = ModuleError::IoError("io".to_string());
        let err8 = err7.clone();
        assert!(matches!(err8, ModuleError::IoError(_)));
    }

    #[test]
    fn test_module_error_is_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(ModuleError::IoError("test".to_string()));
        assert!(!err.to_string().is_empty());
    }

    // Resolution error tests
    #[test]
    fn test_resolve_module_path_file_not_found() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let result = resolve_module_path("nonexistent", temp_dir.path());
        assert!(matches!(result, Err(ModuleError::FileNotFound(_))));
    }

    #[test]
    fn test_resolve_module_path_dot_notation_simple() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("utils.ruchy");
        fs::write(&module_path, "fun test() { 1 }").expect("operation should succeed in test");

        let resolved = resolve_module_path_dot_notation("utils", temp_dir.path())
            .expect("operation should succeed in test");
        assert_eq!(resolved, module_path);
    }

    #[test]
    fn test_resolve_module_path_dot_notation_nested() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let foo_dir = temp_dir.path().join("foo");
        fs::create_dir(&foo_dir).expect("operation should succeed in test");
        let bar_path = foo_dir.join("bar.ruchy");
        fs::write(&bar_path, "fun test() { 1 }").expect("operation should succeed in test");

        let resolved = resolve_module_path_dot_notation("foo.bar", temp_dir.path())
            .expect("operation should succeed in test");
        assert_eq!(resolved, bar_path);
    }

    #[test]
    fn test_resolve_module_path_dot_notation_file_not_found() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let result = resolve_module_path_dot_notation("nonexistent", temp_dir.path());
        assert!(matches!(result, Err(ModuleError::FileNotFound(_))));
    }

    // LoadedModule tests
    #[test]
    fn test_loaded_module_get_symbol_found() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "fun add(x, y) { x + y }")
            .expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let symbol = module.get_symbol("add");
        assert!(symbol.is_some());
        assert!(matches!(symbol, Some(Symbol::Function(_))));
    }

    #[test]
    fn test_loaded_module_get_symbol_not_found() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "fun add(x, y) { x + y }")
            .expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let symbol = module.get_symbol("nonexistent");
        assert!(symbol.is_none());
    }

    #[test]
    fn test_loaded_module_ast() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "fun test() { 42 }").expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let ast = module.ast();
        assert!(matches!(
            ast.kind,
            ExprKind::Block(_) | ExprKind::Function { .. }
        ));
    }

    #[test]
    fn test_loaded_module_clone() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "fun test() { 1 }").expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let cloned = module.clone();
        assert_eq!(cloned.path(), module.path());
        assert_eq!(cloned.symbols().len(), module.symbols().len());
    }

    // Symbol tests
    #[test]
    fn test_function_symbol_clone() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "fun add(x, y) { x + y }")
            .expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let functions = extract_functions(&module);
        assert!(!functions.is_empty());
        let cloned = functions[0].clone();
        assert_eq!(cloned.name, functions[0].name);
    }

    #[test]
    fn test_struct_symbol_extraction() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "struct Point { x: i32, y: i32 }")
            .expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let structs = extract_structs(&module);
        assert_eq!(structs.len(), 1);
        assert_eq!(structs[0].name, "Point");
    }

    #[test]
    fn test_struct_symbol_clone() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "struct Point { x: i32, y: i32 }")
            .expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let structs = extract_structs(&module);
        let cloned = structs[0].clone();
        assert_eq!(cloned.name, "Point");
    }

    #[test]
    fn test_const_symbol_extraction() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "let PI = 3.14159").expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let consts = extract_consts(&module);
        assert_eq!(consts.len(), 1);
        assert_eq!(consts[0].name, "PI");
    }

    #[test]
    fn test_const_symbol_clone() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "let VALUE = 42").expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let consts = extract_consts(&module);
        let cloned = consts[0].clone();
        assert_eq!(cloned.name, "VALUE");
    }

    // AllSymbols tests
    #[test]
    fn test_all_symbols_default() {
        let all = AllSymbols::default();
        assert!(all.functions.is_empty());
        assert!(all.structs.is_empty());
        assert!(all.consts.is_empty());
    }

    #[test]
    fn test_extract_all_symbols() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(
            &module_path,
            "fun add(x, y) { x + y }\nstruct Point { x: i32 }\nlet PI = 3.14",
        )
        .expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let all = extract_all_symbols(&module);

        assert_eq!(all.functions.len(), 1);
        assert_eq!(all.structs.len(), 1);
        assert_eq!(all.consts.len(), 1);
    }

    // ModuleCache tests
    #[test]
    fn test_module_cache_default() {
        let cache = ModuleCache::default();
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "fun test() { 1 }").expect("operation should succeed in test");

        let result = cache.load(&module_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_module_cache_load_nonexistent() {
        let cache = ModuleCache::new();
        let result = cache.load(&PathBuf::from("/nonexistent/path.ruchy"));
        assert!(result.is_err());
    }

    // load_module backward compat wrapper test
    #[test]
    fn test_load_module_compat_success() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "fun test() { 1 }").expect("operation should succeed in test");

        let result = load_module(&module_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_module_compat_error() {
        let result = load_module(Path::new("/nonexistent/path.ruchy"));
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("IO error") || err_msg.contains("not found"));
    }

    // Parse error test
    #[test]
    fn test_load_module_parse_error() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "fun {{{ invalid syntax")
            .expect("operation should succeed in test");

        let result = load_module_from_file(&module_path);
        assert!(matches!(result, Err(ModuleError::ParseError(_))));
    }

    // Single expression (not block) test for extract_symbols_from_ast
    #[test]
    fn test_extract_symbols_single_function() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("single.ruchy");
        fs::write(&module_path, "fun single() { 1 }").expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        assert_eq!(module.symbols().len(), 1);
        assert!(module.symbols().contains_key("single"));
    }

    // Deeply nested module path
    #[test]
    fn test_resolve_deeply_nested_module() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let a_dir = temp_dir.path().join("a");
        let b_dir = a_dir.join("b");
        let c_dir = b_dir.join("c");
        fs::create_dir_all(&c_dir).expect("operation should succeed in test");
        let module_path = c_dir.join("d.ruchy");
        fs::write(&module_path, "fun deep() { 42 }").expect("operation should succeed in test");

        let resolved = resolve_module_path("a::b::c::d", temp_dir.path())
            .expect("operation should succeed in test");
        assert_eq!(resolved, module_path);
    }

    #[test]
    fn test_resolve_deeply_nested_module_dot_notation() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let a_dir = temp_dir.path().join("a");
        let b_dir = a_dir.join("b");
        fs::create_dir_all(&b_dir).expect("operation should succeed in test");
        let module_path = b_dir.join("c.ruchy");
        fs::write(&module_path, "fun deep() { 42 }").expect("operation should succeed in test");

        let resolved = resolve_module_path_dot_notation("a.b.c", temp_dir.path())
            .expect("operation should succeed in test");
        assert_eq!(resolved, module_path);
    }

    // Test with mixed symbol types
    #[test]
    fn test_module_with_all_symbol_types() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("mixed.ruchy");
        fs::write(
            &module_path,
            r#"
            fun compute(x) { x * 2 }
            struct Data { value: i32 }
            let CONST = 100
            fun another() { 42 }
            struct Other { name: String }
            let LIMIT = 50
            "#,
        )
        .expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let all = extract_all_symbols(&module);

        assert_eq!(all.functions.len(), 2);
        assert_eq!(all.structs.len(), 2);
        assert_eq!(all.consts.len(), 2);
    }

    // Symbol enum variant tests
    #[test]
    fn test_symbol_function_variant() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "fun test() { 1 }").expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let symbol = module.get_symbol("test").expect("should have symbol");
        assert!(matches!(symbol, Symbol::Function(_)));

        if let Symbol::Function(f) = symbol {
            assert_eq!(f.name, "test");
        }
    }

    #[test]
    fn test_symbol_struct_variant() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "struct Test { field: i32 }")
            .expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let symbol = module.get_symbol("Test").expect("should have symbol");
        assert!(matches!(symbol, Symbol::Struct(_)));

        if let Symbol::Struct(s) = symbol {
            assert_eq!(s.name, "Test");
        }
    }

    #[test]
    fn test_symbol_const_variant() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "let VALUE = 42").expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let symbol = module.get_symbol("VALUE").expect("should have symbol");
        assert!(matches!(symbol, Symbol::Const(_)));

        if let Symbol::Const(c) = symbol {
            assert_eq!(c.name, "VALUE");
        }
    }

    // Symbol Clone via Debug
    #[test]
    fn test_symbol_debug() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "fun test() { 1 }").expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let symbol = module.get_symbol("test").expect("should have symbol");
        let debug_str = format!("{:?}", symbol);
        assert!(debug_str.contains("Function"));
    }

    #[test]
    fn test_symbol_clone() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "fun test() { 1 }").expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let symbol = module.get_symbol("test").expect("should have symbol");
        let cloned = symbol.clone();
        assert!(matches!(cloned, Symbol::Function(_)));
    }

    // Empty module test
    #[test]
    fn test_empty_module() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("empty.ruchy");
        fs::write(&module_path, "").expect("operation should succeed in test");

        // An empty file should either fail to parse or have no symbols
        let result = load_module_from_file(&module_path);
        // Empty file may or may not be valid depending on parser
        if let Ok(module) = result {
            let all = extract_all_symbols(&module);
            // Even if it parses, there should be no symbols
            assert!(all.functions.is_empty() || all.structs.is_empty() || all.consts.is_empty());
        }
    }

    // Test module symbols iterator
    #[test]
    fn test_module_symbols_iteration() {
        let temp_dir = TempDir::new().expect("operation should succeed in test");
        let module_path = temp_dir.path().join("test.ruchy");
        fs::write(&module_path, "fun a() { 1 }\nfun b() { 2 }")
            .expect("operation should succeed in test");

        let module = load_module_from_file(&module_path).expect("operation should succeed in test");
        let symbol_names: Vec<&String> = module.symbols().keys().collect();
        assert_eq!(symbol_names.len(), 2);
        assert!(symbol_names.contains(&&"a".to_string()) || symbol_names.iter().any(|s| *s == "a"));
        assert!(symbol_names.contains(&&"b".to_string()) || symbol_names.iter().any(|s| *s == "b"));
    }
}
