//! System Built-in Function Transpilation
//!
//! This module handles transpilation of system-level built-in functions:
//! - Environment: `env_args`, `env_var`, `env_set_var`, `env_remove_var`, `env_vars`,
//!   `env_current_dir`, `env_set_current_dir`, `env_temp_dir`
//! - Filesystem: `fs_read`, `fs_write`, `fs_exists`, `fs_create_dir`, `fs_remove_file`,
//!   `fs_remove_dir`, `fs_copy`, `fs_rename`, `fs_metadata`, `fs_read_dir`,
//!   `fs_canonicalize`, `fs_is_file`
//! - Path: `path_join`, `path_join_many`, `path_parent`, `path_file_name`, `path_file_stem`,
//!   `path_extension`, `path_is_absolute`, `path_is_relative`, `path_canonicalize`,
//!   `path_with_extension`, `path_with_file_name`, `path_components`, `path_normalize`
//!
//! **EXTREME TDD Round 61**: Extracted from statements.rs for modularization.

use super::Transpiler;
use crate::frontend::ast::Expr;
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    /// Handle environment functions (`env_args`, `env_var`, etc.)
    ///
    /// # Complexity
    /// Cyclomatic complexity: 9 (within Toyota Way limits)
    pub fn try_transpile_environment_function(
        &self,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        match base_name {
            "env_args" => {
                if !args.is_empty() {
                    bail!("env_args() expects no arguments");
                }
                Ok(Some(quote! {
                    std::env::args().collect::<Vec<String>>()
                }))
            }
            "env_var" => {
                if args.len() != 1 {
                    bail!("env_var() expects 1 argument");
                }
                let key = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::env::var(#key).expect("Environment variable not found")
                }))
            }
            "env_set_var" => {
                if args.len() != 2 {
                    bail!("env_set_var() expects 2 arguments");
                }
                let key = self.transpile_expr(&args[0])?;
                let value = self.transpile_expr(&args[1])?;
                Ok(Some(quote! {
                    std::env::set_var(#key, #value)
                }))
            }
            "env_remove_var" => {
                if args.len() != 1 {
                    bail!("env_remove_var() expects 1 argument");
                }
                let key = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::env::remove_var(#key)
                }))
            }
            "env_vars" => {
                if !args.is_empty() {
                    bail!("env_vars() expects no arguments");
                }
                Ok(Some(quote! {
                    std::env::vars().collect::<std::collections::HashMap<String, String>>()
                }))
            }
            "env_current_dir" => {
                if !args.is_empty() {
                    bail!("env_current_dir() expects no arguments");
                }
                Ok(Some(quote! {
                    std::env::current_dir()
                        .expect("Failed to get current directory")
                        .to_string_lossy()
                        .to_string()
                }))
            }
            "env_set_current_dir" => {
                if args.len() != 1 {
                    bail!("env_set_current_dir() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::env::set_current_dir(#path).expect("Failed to set current directory")
                }))
            }
            "env_temp_dir" => {
                if !args.is_empty() {
                    bail!("env_temp_dir() expects no arguments");
                }
                Ok(Some(quote! {
                    std::env::temp_dir().to_string_lossy().to_string()
                }))
            }
            _ => Ok(None),
        }
    }

    /// Transpile file system functions (fs_*)
    ///
    /// Layer 2 of three-layer builtin pattern (proven from env functions)
    /// Complexity: 10 (within Toyota Way limits)
    pub fn try_transpile_fs_function(
        &self,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        match base_name {
            "fs_read" => {
                if args.len() != 1 {
                    bail!("fs_read() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::fs::read_to_string(#path).expect("Failed to read file")
                }))
            }
            "fs_write" => {
                if args.len() != 2 {
                    bail!("fs_write() expects 2 arguments");
                }
                let path = self.transpile_expr(&args[0])?;
                let content = self.transpile_expr(&args[1])?;
                Ok(Some(quote! {
                    std::fs::write(#path, #content).expect("Failed to write file")
                }))
            }
            "fs_exists" => {
                if args.len() != 1 {
                    bail!("fs_exists() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::path::Path::new(&#path).exists()
                }))
            }
            "fs_create_dir" => {
                if args.len() != 1 {
                    bail!("fs_create_dir() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::fs::create_dir_all(#path).expect("Failed to create directory")
                }))
            }
            "fs_remove_file" => {
                if args.len() != 1 {
                    bail!("fs_remove_file() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::fs::remove_file(#path).expect("Failed to remove file")
                }))
            }
            "fs_remove_dir" => {
                if args.len() != 1 {
                    bail!("fs_remove_dir() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::fs::remove_dir(#path).expect("Failed to remove directory")
                }))
            }
            "fs_copy" => {
                if args.len() != 2 {
                    bail!("fs_copy() expects 2 arguments");
                }
                let from = self.transpile_expr(&args[0])?;
                let to = self.transpile_expr(&args[1])?;
                Ok(Some(quote! {
                    std::fs::copy(#from, #to).expect("Failed to copy file")
                }))
            }
            "fs_rename" => {
                if args.len() != 2 {
                    bail!("fs_rename() expects 2 arguments");
                }
                let from = self.transpile_expr(&args[0])?;
                let to = self.transpile_expr(&args[1])?;
                Ok(Some(quote! {
                    std::fs::rename(#from, #to).expect("Failed to rename file")
                }))
            }
            "fs_metadata" => {
                if args.len() != 1 {
                    bail!("fs_metadata() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::fs::metadata(#path).expect("Failed to get metadata")
                }))
            }
            "fs_read_dir" => {
                if args.len() != 1 {
                    bail!("fs_read_dir() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::fs::read_dir(#path)
                        .expect("Failed to read directory")
                        .filter_map(|e| e.ok())
                        .map(|e| e.path().display().to_string())
                        .collect::<Vec<String>>()
                }))
            }
            "fs_canonicalize" => {
                if args.len() != 1 {
                    bail!("fs_canonicalize() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::fs::canonicalize(#path)
                        .expect("Failed to canonicalize path")
                        .display()
                        .to_string()
                }))
            }
            "fs_is_file" => {
                if args.len() != 1 {
                    bail!("fs_is_file() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::path::Path::new(&#path).is_file()
                }))
            }
            _ => Ok(None),
        }
    }

    /// Transpile path functions (path_*)
    ///
    /// Layer 2 of three-layer builtin pattern (proven from env/fs functions)
    /// Phase 3: `STDLIB_ACCESS_PLAN` - Path Module (13 functions)
    /// Complexity: 10 (within Toyota Way limits)
    pub fn try_transpile_path_function(
        &self,
        base_name: &str,
        args: &[Expr],
    ) -> Result<Option<TokenStream>> {
        match base_name {
            "path_join" => {
                if args.len() != 2 {
                    bail!("path_join() expects 2 arguments");
                }
                let base = self.transpile_expr(&args[0])?;
                let component = self.transpile_expr(&args[1])?;
                Ok(Some(quote! {
                    std::path::Path::new(&#base).join(#component).to_string_lossy().to_string()
                }))
            }
            "path_join_many" => {
                if args.len() != 1 {
                    bail!("path_join_many() expects 1 argument");
                }
                let components = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    {
                        let mut path = std::path::PathBuf::new();
                        for component in #components {
                            path.push(component);
                        }
                        path.to_string_lossy().to_string()
                    }
                }))
            }
            "path_parent" => {
                if args.len() != 1 {
                    bail!("path_parent() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::path::Path::new(&#path).parent().map(|p| p.to_string_lossy().to_string())
                }))
            }
            "path_file_name" => {
                if args.len() != 1 {
                    bail!("path_file_name() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::path::Path::new(&#path).file_name().map(|n| n.to_string_lossy().to_string())
                }))
            }
            "path_file_stem" => {
                if args.len() != 1 {
                    bail!("path_file_stem() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::path::Path::new(&#path).file_stem().map(|s| s.to_string_lossy().to_string())
                }))
            }
            "path_extension" => {
                if args.len() != 1 {
                    bail!("path_extension() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::path::Path::new(&#path).extension().map(|e| e.to_string_lossy().to_string())
                }))
            }
            "path_is_absolute" => {
                if args.len() != 1 {
                    bail!("path_is_absolute() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::path::Path::new(&#path).is_absolute()
                }))
            }
            "path_is_relative" => {
                if args.len() != 1 {
                    bail!("path_is_relative() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::path::Path::new(&#path).is_relative()
                }))
            }
            "path_canonicalize" => {
                if args.len() != 1 {
                    bail!("path_canonicalize() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::fs::canonicalize(#path).expect("Failed to canonicalize path").to_string_lossy().to_string()
                }))
            }
            "path_with_extension" => {
                if args.len() != 2 {
                    bail!("path_with_extension() expects 2 arguments");
                }
                let path = self.transpile_expr(&args[0])?;
                let ext = self.transpile_expr(&args[1])?;
                Ok(Some(quote! {
                    std::path::Path::new(&#path).with_extension(#ext).to_string_lossy().to_string()
                }))
            }
            "path_with_file_name" => {
                if args.len() != 2 {
                    bail!("path_with_file_name() expects 2 arguments");
                }
                let path = self.transpile_expr(&args[0])?;
                let name = self.transpile_expr(&args[1])?;
                Ok(Some(quote! {
                    std::path::Path::new(&#path).with_file_name(#name).to_string_lossy().to_string()
                }))
            }
            "path_components" => {
                if args.len() != 1 {
                    bail!("path_components() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    std::path::Path::new(&#path)
                        .components()
                        .map(|c| c.as_os_str().to_string_lossy().to_string())
                        .collect::<Vec<String>>()
                }))
            }
            "path_normalize" => {
                if args.len() != 1 {
                    bail!("path_normalize() expects 1 argument");
                }
                let path = self.transpile_expr(&args[0])?;
                Ok(Some(quote! {
                    {
                        let p = std::path::Path::new(&#path);
                        let mut normalized = std::path::PathBuf::new();
                        for component in p.components() {
                            match component {
                                std::path::Component::CurDir => {},
                                std::path::Component::ParentDir => { normalized.pop(); },
                                _ => normalized.push(component),
                            }
                        }
                        normalized.to_string_lossy().to_string()
                    }
                }))
            }
            _ => Ok(None),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span};

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn string_expr(s: &str) -> Expr {
        make_expr(ExprKind::Literal(Literal::String(s.to_string())))
    }

    fn ident_expr(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    // ========================================================================
    // Environment function tests
    // ========================================================================

    #[test]
    fn test_env_args() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_environment_function("env_args", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("env :: args"));
        assert!(tokens_str.contains("collect"));
    }

    #[test]
    fn test_env_args_with_args_error() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("test")];
        let result = transpiler.try_transpile_environment_function("env_args", &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no arguments"));
    }

    #[test]
    fn test_env_var() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("PATH")];
        let result = transpiler.try_transpile_environment_function("env_var", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("env :: var"));
    }

    #[test]
    fn test_env_var_wrong_args() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_environment_function("env_var", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_env_set_var() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("KEY"), string_expr("VALUE")];
        let result = transpiler.try_transpile_environment_function("env_set_var", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("set_var"));
    }

    #[test]
    fn test_env_set_var_wrong_args() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("KEY")];
        let result = transpiler.try_transpile_environment_function("env_set_var", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_env_remove_var() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("KEY")];
        let result = transpiler.try_transpile_environment_function("env_remove_var", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("remove_var"));
    }

    #[test]
    fn test_env_vars() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_environment_function("env_vars", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("env :: vars"));
        assert!(tokens_str.contains("HashMap"));
    }

    #[test]
    fn test_env_current_dir() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_environment_function("env_current_dir", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("current_dir"));
    }

    #[test]
    fn test_env_set_current_dir() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp")];
        let result = transpiler.try_transpile_environment_function("env_set_current_dir", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("set_current_dir"));
    }

    #[test]
    fn test_env_temp_dir() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_environment_function("env_temp_dir", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("temp_dir"));
    }

    #[test]
    fn test_env_unknown() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_environment_function("env_unknown", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ========================================================================
    // Filesystem function tests
    // ========================================================================

    #[test]
    fn test_fs_read() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/test.txt")];
        let result = transpiler.try_transpile_fs_function("fs_read", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("read_to_string"));
    }

    #[test]
    fn test_fs_read_wrong_args() {
        let transpiler = Transpiler::new();
        let args: Vec<Expr> = vec![];
        let result = transpiler.try_transpile_fs_function("fs_read", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_fs_write() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/test.txt"), string_expr("content")];
        let result = transpiler.try_transpile_fs_function("fs_write", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("fs :: write"));
    }

    #[test]
    fn test_fs_exists() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp")];
        let result = transpiler.try_transpile_fs_function("fs_exists", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("exists"));
    }

    #[test]
    fn test_fs_create_dir() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/newdir")];
        let result = transpiler.try_transpile_fs_function("fs_create_dir", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("create_dir_all"));
    }

    #[test]
    fn test_fs_remove_file() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/test.txt")];
        let result = transpiler.try_transpile_fs_function("fs_remove_file", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("remove_file"));
    }

    #[test]
    fn test_fs_remove_dir() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/dir")];
        let result = transpiler.try_transpile_fs_function("fs_remove_dir", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("remove_dir"));
    }

    #[test]
    fn test_fs_copy() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/src.txt"), string_expr("/tmp/dst.txt")];
        let result = transpiler.try_transpile_fs_function("fs_copy", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("fs :: copy"));
    }

    #[test]
    fn test_fs_rename() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/old.txt"), string_expr("/tmp/new.txt")];
        let result = transpiler.try_transpile_fs_function("fs_rename", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("rename"));
    }

    #[test]
    fn test_fs_metadata() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/test.txt")];
        let result = transpiler.try_transpile_fs_function("fs_metadata", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("metadata"));
    }

    #[test]
    fn test_fs_read_dir() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp")];
        let result = transpiler.try_transpile_fs_function("fs_read_dir", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("read_dir"));
    }

    #[test]
    fn test_fs_canonicalize() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr(".")];
        let result = transpiler.try_transpile_fs_function("fs_canonicalize", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("canonicalize"));
    }

    #[test]
    fn test_fs_is_file() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/test.txt")];
        let result = transpiler.try_transpile_fs_function("fs_is_file", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("is_file"));
    }

    #[test]
    fn test_fs_unknown() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp")];
        let result = transpiler.try_transpile_fs_function("fs_unknown", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ========================================================================
    // Path function tests
    // ========================================================================

    #[test]
    fn test_path_join() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp"), string_expr("file.txt")];
        let result = transpiler.try_transpile_path_function("path_join", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("join"));
    }

    #[test]
    fn test_path_join_wrong_args() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp")];
        let result = transpiler.try_transpile_path_function("path_join", &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_path_join_many() {
        let transpiler = Transpiler::new();
        let args = vec![ident_expr("components")];
        let result = transpiler.try_transpile_path_function("path_join_many", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("PathBuf"));
    }

    #[test]
    fn test_path_parent() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/file.txt")];
        let result = transpiler.try_transpile_path_function("path_parent", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("parent"));
    }

    #[test]
    fn test_path_file_name() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/file.txt")];
        let result = transpiler.try_transpile_path_function("path_file_name", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("file_name"));
    }

    #[test]
    fn test_path_file_stem() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/file.txt")];
        let result = transpiler.try_transpile_path_function("path_file_stem", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("file_stem"));
    }

    #[test]
    fn test_path_extension() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/file.txt")];
        let result = transpiler.try_transpile_path_function("path_extension", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("extension"));
    }

    #[test]
    fn test_path_is_absolute() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp")];
        let result = transpiler.try_transpile_path_function("path_is_absolute", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("is_absolute"));
    }

    #[test]
    fn test_path_is_relative() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("./file.txt")];
        let result = transpiler.try_transpile_path_function("path_is_relative", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("is_relative"));
    }

    #[test]
    fn test_path_canonicalize() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr(".")];
        let result = transpiler.try_transpile_path_function("path_canonicalize", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("canonicalize"));
    }

    #[test]
    fn test_path_with_extension() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/file.txt"), string_expr("md")];
        let result = transpiler.try_transpile_path_function("path_with_extension", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("with_extension"));
    }

    #[test]
    fn test_path_with_file_name() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/file.txt"), string_expr("other.txt")];
        let result = transpiler.try_transpile_path_function("path_with_file_name", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("with_file_name"));
    }

    #[test]
    fn test_path_components() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/dir/file.txt")];
        let result = transpiler.try_transpile_path_function("path_components", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("components"));
    }

    #[test]
    fn test_path_normalize() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp/../tmp/./file.txt")];
        let result = transpiler.try_transpile_path_function("path_normalize", &args);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(tokens.is_some());
        let tokens_str = tokens.unwrap().to_string();
        assert!(tokens_str.contains("Component"));
    }

    #[test]
    fn test_path_unknown() {
        let transpiler = Transpiler::new();
        let args = vec![string_expr("/tmp")];
        let result = transpiler.try_transpile_path_function("path_unknown", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
