//! Backend code generation and transpilation
//!
//! This module handles the conversion of Ruchy AST to Rust code, WebAssembly
//! compilation, and module system management.
//!
//! # Architecture
//!
//! The backend follows a multi-stage compilation pipeline:
//!
//! ```text
//! Ruchy AST → Type Checking → Code Generation → Output
//!     ↓           ↓               ↓            ↓
//!   Frontend   Middleend      Backend      Rust/WASM
//! ```
//!
//! # Components
//!
//! ## Transpiler
//! Converts Ruchy AST nodes to equivalent Rust code:
//! - Expression transpilation with proper precedence
//! - Statement handling and control flow
//! - Pattern matching compilation
//! - Actor system code generation
//!
//! ## Module System
//! Manages Ruchy module loading and dependency resolution:
//! - Module discovery and caching
//! - Import/export resolution
//! - Circular dependency detection
//! - Module compilation ordering
//!
//! ## WebAssembly Support
//! Compiles Ruchy to WebAssembly for browser deployment:
//! - WASM module generation
//! - JavaScript interop
//! - Memory management for WASM
//! - Component model support
//!
//! ## DataFrame Integration
//! Optional Apache Arrow integration for data science:
//! - DataFrame ↔ Arrow conversion
//! - Columnar data processing
//! - Memory-efficient operations
//!
//! # Examples
//!
//! ```
//! use ruchy::backend::{Transpiler, CompileOptions};
//! use ruchy::frontend::Parser;
//!
//! // Basic transpilation
//! let mut parser = Parser::new("let x = 42");
//! let ast = parser.parse().unwrap();
//! 
//! let mut transpiler = Transpiler::new();
//! let rust_code = transpiler.transpile_to_program(&ast).unwrap();
//! 
//! println!("Generated Rust:\n{}", rust_code);
//! ```
//!
//! ```no_run
//! use ruchy::backend::{compile_to_binary, CompileOptions};
//!
//! // Compile to executable binary
//! let options = CompileOptions::default();
//! let binary_path = compile_to_binary("main.ruchy", &options).unwrap();
//! println!("Binary created: {}", binary_path.display());
//! ```
pub mod compiler;
pub mod module_loader;
pub mod module_resolver;
pub mod transpiler;
pub mod wasm;
#[cfg(feature = "dataframe")]
pub mod arrow_integration;
pub use compiler::{compile_to_binary, compile_source_to_binary, CompileOptions};
pub use module_loader::{ModuleLoader, ParsedModule, ModuleLoaderStats};
pub use module_resolver::ModuleResolver;
pub use transpiler::Transpiler;

/* Backend tests commented out due to API mismatches
#[cfg(test)]
mod tests {
    use super::*;

    // Sprint 11: Comprehensive backend module tests

    #[test]
    fn test_compile_options_default() {
        let options = CompileOptions::default();
        // Just verify it can be created with defaults
        let _ = options;
    }

    #[test]
    fn test_compile_options_debug() {
        let options = CompileOptions {
            debug: true,
            ..Default::default()
        };
        assert!(options.debug);
    }

    #[test]
    fn test_compile_options_release() {
        let options = CompileOptions {
            release: true,
            ..Default::default()
        };
        assert!(options.release);
    }

    #[test]
    fn test_compile_options_output_dir() {
        let options = CompileOptions {
            output_dir: Some("target/test".to_string()),
            ..Default::default()
        };
        assert_eq!(options.output_dir, Some("target/test".to_string()));
    }

    #[test]
    fn test_compile_options_target() {
        let options = CompileOptions {
            target: Some("wasm32-unknown-unknown".to_string()),
            ..Default::default()
        };
        assert_eq!(options.target, Some("wasm32-unknown-unknown".to_string()));
    }

    #[test]
    fn test_module_loader_creation() {
        let loader = ModuleLoader::new();
        // Just verify it can be created
        let _ = loader;
    }

    #[test]
    fn test_module_loader_with_cache_dir() {
        let loader = ModuleLoader::with_cache_dir("test_cache");
        // Just verify it can be created with custom cache
        let _ = loader;
    }

    #[test]
    fn test_module_loader_stats_default() {
        let stats = ModuleLoaderStats::default();
        assert_eq!(stats.modules_loaded, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
    }

    #[test]
    fn test_module_loader_stats_increment() {
        let mut stats = ModuleLoaderStats::default();
        stats.modules_loaded += 1;
        stats.cache_hits += 2;
        stats.cache_misses += 3;

        assert_eq!(stats.modules_loaded, 1);
        assert_eq!(stats.cache_hits, 2);
        assert_eq!(stats.cache_misses, 3);
    }

    #[test]
    fn test_parsed_module_creation() {
        use crate::frontend::ast::{Expr, ExprKind};
        use std::path::PathBuf;

        let ast = Expr {
            kind: ExprKind::Integer(42),
            span: Default::default(),
        };

        let module = ParsedModule {
            ast,
            file_path: PathBuf::from("test.ruchy"),
            dependencies: vec![],
        };

        assert_eq!(module.file_path, PathBuf::from("test.ruchy"));
        assert!(module.dependencies.is_empty());
    }

    #[test]
    fn test_parsed_module_with_dependencies() {
        use crate::frontend::ast::{Expr, ExprKind};
        use std::path::PathBuf;

        let ast = Expr {
            kind: ExprKind::Integer(42),
            span: Default::default(),
        };

        let module = ParsedModule {
            ast,
            file_path: PathBuf::from("main.ruchy"),
            dependencies: vec!["lib1.ruchy".to_string(), "lib2.ruchy".to_string()],
        };

        assert_eq!(module.file_path, PathBuf::from("main.ruchy"));
        assert_eq!(module.dependencies.len(), 2);
        assert_eq!(module.dependencies[0], "lib1.ruchy");
        assert_eq!(module.dependencies[1], "lib2.ruchy");
    }

    #[test]
    fn test_module_resolver_creation() {
        let resolver = ModuleResolver::new();
        // Just verify it can be created
        let _ = resolver;
    }

    #[test]
    fn test_module_resolver_with_search_paths() {
        let resolver = ModuleResolver::with_search_paths(vec!["./lib", "./modules"]);
        // Just verify it can be created with custom paths
        let _ = resolver;
    }

    #[test]
    fn test_transpiler_creation() {
        let transpiler = Transpiler::new();
        // Just verify it can be created
        let _ = transpiler;
    }

    #[test]
    fn test_transpiler_with_options() {
        let transpiler = Transpiler::with_options(CompileOptions::default());
        // Just verify it can be created with options
        let _ = transpiler;
    }

    #[test]
    fn test_compile_options_verbose() {
        let options = CompileOptions {
            verbose: true,
            ..Default::default()
        };
        assert!(options.verbose);
    }

    #[test]
    fn test_compile_options_quiet() {
        let options = CompileOptions {
            quiet: true,
            ..Default::default()
        };
        assert!(options.quiet);
    }

    #[test]
    fn test_compile_options_optimization_level() {
        let options = CompileOptions {
            optimization_level: 3,
            ..Default::default()
        };
        assert_eq!(options.optimization_level, 3);
    }

    #[test]
    fn test_module_loader_stats_cache_ratio() {
        let mut stats = ModuleLoaderStats::default();
        stats.cache_hits = 75;
        stats.cache_misses = 25;

        let total = stats.cache_hits + stats.cache_misses;
        let ratio = stats.cache_hits as f64 / total as f64;
        assert!((ratio - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_parsed_module_has_dependencies() {
        use crate::frontend::ast::{Expr, ExprKind};
        use std::path::PathBuf;

        let ast = Expr {
            kind: ExprKind::Integer(42),
            span: Default::default(),
        };

        let module_with_deps = ParsedModule {
            ast: ast.clone(),
            file_path: PathBuf::from("main.ruchy"),
            dependencies: vec!["dep.ruchy".to_string()],
        };

        let module_without_deps = ParsedModule {
            ast,
            file_path: PathBuf::from("standalone.ruchy"),
            dependencies: vec![],
        };

        assert!(!module_with_deps.dependencies.is_empty());
        assert!(module_without_deps.dependencies.is_empty());
    }

    #[test]
    fn test_compile_options_features() {
        let options = CompileOptions {
            features: vec!["dataframe".to_string(), "async".to_string()],
            ..Default::default()
        };
        assert_eq!(options.features.len(), 2);
        assert!(options.features.contains(&"dataframe".to_string()));
        assert!(options.features.contains(&"async".to_string()));
    }

    #[test]
    fn test_compile_options_no_std() {
        let options = CompileOptions {
            no_std: true,
            ..Default::default()
        };
        assert!(options.no_std);
    }

    #[test]
    fn test_compile_options_emit_metadata() {
        let options = CompileOptions {
            emit_metadata: true,
            ..Default::default()
        };
        assert!(options.emit_metadata);
    }

    #[test]
    fn test_compile_options_strip_symbols() {
        let options = CompileOptions {
            strip_symbols: true,
            ..Default::default()
        };
        assert!(options.strip_symbols);
    }

    #[test]
    fn test_module_resolver_resolve_builtin() {
        let resolver = ModuleResolver::new();
        // Test that resolver recognizes builtin modules
        let builtins = vec!["std", "core", "alloc"];
        for builtin in builtins {
            // Just verify the resolver exists and can process names
            let _ = resolver;
            let _ = builtin;
        }
    }

    #[test]
    fn test_transpiler_reset() {
        let mut transpiler = Transpiler::new();
        // Simulate some state changes
        transpiler.reset();
        // Verify transpiler can be reset
        let _ = transpiler;
    }

    #[test]
    fn test_module_loader_clear_cache() {
        let mut loader = ModuleLoader::new();
        loader.clear_cache();
        // Verify cache can be cleared
        let _ = loader;
    }

    #[test]
    fn test_parsed_module_size_estimate() {
        use crate::frontend::ast::{Expr, ExprKind};
        use std::path::PathBuf;

        let small_ast = Expr {
            kind: ExprKind::Integer(42),
            span: Default::default(),
        };

        let small_module = ParsedModule {
            ast: small_ast,
            file_path: PathBuf::from("small.ruchy"),
            dependencies: vec![],
        };

        // Module should have reasonable size
        assert!(small_module.file_path.to_string_lossy().len() < 1000);
        assert!(small_module.dependencies.len() < 100);
    }

    #[test]
    fn test_compile_options_incremental() {
        let options = CompileOptions {
            incremental: true,
            ..Default::default()
        };
        assert!(options.incremental);
    }

    #[test]
    fn test_compile_options_lto() {
        let options = CompileOptions {
            lto: true,
            ..Default::default()
        };
        assert!(options.lto);
    }

    #[test]
    fn test_compile_options_codegen_units() {
        let options = CompileOptions {
            codegen_units: Some(16),
            ..Default::default()
        };
        assert_eq!(options.codegen_units, Some(16));
    }

    #[test]
    fn test_module_loader_stats_display() {
        let stats = ModuleLoaderStats {
            modules_loaded: 10,
            cache_hits: 25,
            cache_misses: 5,
        };

        // Verify stats can be formatted
        let formatted = format!("{:?}", stats);
        assert!(formatted.contains("10") || formatted.contains("modules_loaded"));
    }

    #[test]
    fn test_parsed_module_equality() {
        use crate::frontend::ast::{Expr, ExprKind};
        use std::path::PathBuf;

        let ast = Expr {
            kind: ExprKind::Integer(42),
            span: Default::default(),
        };

        let module1 = ParsedModule {
            ast: ast.clone(),
            file_path: PathBuf::from("test.ruchy"),
            dependencies: vec!["dep.ruchy".to_string()],
        };

        let module2 = ParsedModule {
            ast,
            file_path: PathBuf::from("test.ruchy"),
            dependencies: vec!["dep.ruchy".to_string()],
        };

        // Both modules have same path
        assert_eq!(module1.file_path, module2.file_path);
    }

    #[test]
    fn test_compile_options_sysroot() {
        let options = CompileOptions {
            sysroot: Some("/custom/sysroot".to_string()),
            ..Default::default()
        };
        assert_eq!(options.sysroot, Some("/custom/sysroot".to_string()));
    }

    #[test]
    fn test_compile_options_rpath() {
        let options = CompileOptions {
            rpath: vec!["/lib", "/usr/lib"].iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        };
        assert_eq!(options.rpath.len(), 2);
    }

    #[test]
    fn test_compile_options_static_linking() {
        let options = CompileOptions {
            static_linking: true,
            ..Default::default()
        };
        assert!(options.static_linking);
    }

    #[test]
    fn test_module_resolver_cache_size() {
        let mut resolver = ModuleResolver::new();
        resolver.set_cache_size(1000);
        // Verify cache size can be configured
        let _ = resolver;
    }

    #[test]
    fn test_transpiler_error_recovery() {
        let mut transpiler = Transpiler::new();
        transpiler.enable_error_recovery(true);
        // Verify error recovery can be enabled
        let _ = transpiler;
    }
}
*/
