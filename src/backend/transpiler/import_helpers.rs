//! Import helpers for transpiler
//!
//! This module provides functions for generating Rust code for `std::` imports,
//! particularly file operations, process management, and system information.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Generate `read_file` function for `std::fs` imports
#[must_use]
pub fn generate_read_file_function() -> TokenStream {
    quote! {
        fn read_file(filename: String) -> String {
            fs::read_to_string(filename).unwrap_or_else(|e| panic!("Failed to read file: {}", e))
        }
    }
}

/// Generate `write_file` function for `std::fs` imports
#[must_use]
pub fn generate_write_file_function() -> TokenStream {
    quote! {
        fn write_file(filename: String, content: String) {
            fs::write(filename, content).unwrap_or_else(|e| panic!("Failed to write file: {}", e));
        }
    }
}

/// Generate all file operation functions
#[must_use]
pub fn generate_all_file_operations() -> TokenStream {
    let read_func = generate_read_file_function();
    let write_func = generate_write_file_function();
    quote! {
        #read_func
        #write_func
    }
}

/// Generate process module with process management functions
#[must_use]
pub fn generate_process_module() -> TokenStream {
    quote! {
        mod process {
            pub fn current_pid() -> i32 {
                std::process::id() as i32
            }
            pub fn exit(code: i32) {
                std::process::exit(code);
            }
            pub fn spawn(command: &str) -> Result<i32, String> {
                match std::process::Command::new(command).spawn() {
                    Ok(child) => Ok(child.id() as i32),
                    Err(e) => Err(e.to_string()),
                }
            }
        }
    }
}

/// Generate system module with system information functions
#[must_use]
pub fn generate_system_module() -> TokenStream {
    quote! {
        mod system {
            pub fn get_env(key: &str) -> Option<String> {
                std::env::var(key).ok()
            }
            pub fn set_env(key: &str, value: &str) {
                std::env::set_var(key, value);
            }
            pub fn os_name() -> String {
                std::env::consts::OS.to_string()
            }
            pub fn arch() -> String {
                std::env::consts::ARCH.to_string()
            }
        }
    }
}

/// Generate function stub for unknown `std::` function
#[must_use]
pub fn generate_unknown_function_stub(name: &str) -> TokenStream {
    let func_name = format_ident!("{}", name);
    quote! {
        fn #func_name() -> ! {
            panic!("std::{} not yet implemented", #name);
        }
    }
}

/// Generate aliased file operation function
#[must_use]
pub fn generate_aliased_file_function(name: &str, alias: &str) -> Option<TokenStream> {
    let alias_ident = format_ident!("{}", alias);
    match name {
        "read_file" => Some(quote! {
            fn #alias_ident(filename: String) -> String {
                fs::read_to_string(filename).unwrap_or_else(|e| panic!("Failed to read file: {}", e))
            }
        }),
        "write_file" => Some(quote! {
            fn #alias_ident(filename: String, content: String) {
                fs::write(filename, content).unwrap_or_else(|e| panic!("Failed to write file: {}", e));
            }
        }),
        _ => None,
    }
}

/// Generate aliased unknown function stub
#[must_use]
pub fn generate_aliased_unknown_stub(name: &str, alias: &str) -> TokenStream {
    let alias_ident = format_ident!("{}", alias);
    quote! {
        fn #alias_ident() -> ! {
            panic!("std::fs::{} not yet implemented", #name);
        }
    }
}

/// Validate that module path is for a known `std::` module
#[must_use]
pub fn is_std_fs_path(path: &str) -> bool {
    path == "std::fs" || path.starts_with("std::fs::")
}

/// Validate that module path is for `std::process`
#[must_use]
pub fn is_std_process_path(path: &str) -> bool {
    path == "std::process" || path.starts_with("std::process::")
}

/// Validate that module path is for `std::system`
#[must_use]
pub fn is_std_system_path(path: &str) -> bool {
    path == "std::system" || path.starts_with("std::system::")
}

/// Extract function name from `std::fs::` path
#[must_use]
pub fn extract_function_from_path(path: &str) -> Option<&str> {
    path.strip_prefix("std::fs::")
}

/// Check if function is a known file operation
#[must_use]
pub fn is_known_file_operation(name: &str) -> bool {
    matches!(name, "read_file" | "write_file")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== generate_read_file_function Tests ====================

    #[test]
    fn test_generate_read_file_function_contains_fn() {
        let tokens = generate_read_file_function();
        let code = tokens.to_string();
        assert!(code.contains("fn read_file"));
    }

    #[test]
    fn test_generate_read_file_function_takes_string() {
        let tokens = generate_read_file_function();
        let code = tokens.to_string();
        assert!(code.contains("filename : String"));
    }

    #[test]
    fn test_generate_read_file_function_returns_string() {
        let tokens = generate_read_file_function();
        let code = tokens.to_string();
        assert!(code.contains("-> String"));
    }

    #[test]
    fn test_generate_read_file_function_uses_fs_read() {
        let tokens = generate_read_file_function();
        let code = tokens.to_string();
        assert!(code.contains("fs :: read_to_string"));
    }

    #[test]
    fn test_generate_read_file_function_has_error_handling() {
        let tokens = generate_read_file_function();
        let code = tokens.to_string();
        assert!(code.contains("unwrap_or_else"));
        assert!(code.contains("panic !"));
    }

    // ==================== generate_write_file_function Tests ====================

    #[test]
    fn test_generate_write_file_function_contains_fn() {
        let tokens = generate_write_file_function();
        let code = tokens.to_string();
        assert!(code.contains("fn write_file"));
    }

    #[test]
    fn test_generate_write_file_function_takes_two_strings() {
        let tokens = generate_write_file_function();
        let code = tokens.to_string();
        assert!(code.contains("filename : String"));
        assert!(code.contains("content : String"));
    }

    #[test]
    fn test_generate_write_file_function_uses_fs_write() {
        let tokens = generate_write_file_function();
        let code = tokens.to_string();
        assert!(code.contains("fs :: write"));
    }

    #[test]
    fn test_generate_write_file_function_has_error_handling() {
        let tokens = generate_write_file_function();
        let code = tokens.to_string();
        assert!(code.contains("unwrap_or_else"));
    }

    // ==================== generate_all_file_operations Tests ====================

    #[test]
    fn test_generate_all_file_operations_includes_read() {
        let tokens = generate_all_file_operations();
        let code = tokens.to_string();
        assert!(code.contains("fn read_file"));
    }

    #[test]
    fn test_generate_all_file_operations_includes_write() {
        let tokens = generate_all_file_operations();
        let code = tokens.to_string();
        assert!(code.contains("fn write_file"));
    }

    #[test]
    fn test_generate_all_file_operations_has_both_functions() {
        let tokens = generate_all_file_operations();
        let code = tokens.to_string();
        // Count occurrences of "fn "
        let fn_count = code.matches("fn ").count();
        assert_eq!(fn_count, 2);
    }

    // ==================== generate_process_module Tests ====================

    #[test]
    fn test_generate_process_module_creates_mod() {
        let tokens = generate_process_module();
        let code = tokens.to_string();
        assert!(code.contains("mod process"));
    }

    #[test]
    fn test_generate_process_module_has_current_pid() {
        let tokens = generate_process_module();
        let code = tokens.to_string();
        assert!(code.contains("fn current_pid"));
        assert!(code.contains("std :: process :: id"));
    }

    #[test]
    fn test_generate_process_module_has_exit() {
        let tokens = generate_process_module();
        let code = tokens.to_string();
        assert!(code.contains("fn exit"));
        assert!(code.contains("std :: process :: exit"));
    }

    #[test]
    fn test_generate_process_module_has_spawn() {
        let tokens = generate_process_module();
        let code = tokens.to_string();
        assert!(code.contains("fn spawn"));
        assert!(code.contains("Command :: new"));
    }

    // ==================== generate_system_module Tests ====================

    #[test]
    fn test_generate_system_module_creates_mod() {
        let tokens = generate_system_module();
        let code = tokens.to_string();
        assert!(code.contains("mod system"));
    }

    #[test]
    fn test_generate_system_module_has_get_env() {
        let tokens = generate_system_module();
        let code = tokens.to_string();
        assert!(code.contains("fn get_env"));
        assert!(code.contains("std :: env :: var"));
    }

    #[test]
    fn test_generate_system_module_has_set_env() {
        let tokens = generate_system_module();
        let code = tokens.to_string();
        assert!(code.contains("fn set_env"));
        assert!(code.contains("std :: env :: set_var"));
    }

    #[test]
    fn test_generate_system_module_has_os_name() {
        let tokens = generate_system_module();
        let code = tokens.to_string();
        assert!(code.contains("fn os_name"));
        assert!(code.contains("std :: env :: consts :: OS"));
    }

    #[test]
    fn test_generate_system_module_has_arch() {
        let tokens = generate_system_module();
        let code = tokens.to_string();
        assert!(code.contains("fn arch"));
        assert!(code.contains("std :: env :: consts :: ARCH"));
    }

    // ==================== generate_unknown_function_stub Tests ====================

    #[test]
    fn test_generate_unknown_function_stub_uses_name() {
        let tokens = generate_unknown_function_stub("foobar");
        let code = tokens.to_string();
        assert!(code.contains("fn foobar"));
    }

    #[test]
    fn test_generate_unknown_function_stub_returns_never() {
        let tokens = generate_unknown_function_stub("foobar");
        let code = tokens.to_string();
        assert!(code.contains("-> !"));
    }

    #[test]
    fn test_generate_unknown_function_stub_panics() {
        let tokens = generate_unknown_function_stub("foobar");
        let code = tokens.to_string();
        assert!(code.contains("panic !"));
        assert!(code.contains("not yet implemented"));
    }

    // ==================== generate_aliased_file_function Tests ====================

    #[test]
    fn test_generate_aliased_file_function_read_file() {
        let tokens = generate_aliased_file_function("read_file", "rf");
        assert!(tokens.is_some());
        let code = tokens.unwrap().to_string();
        assert!(code.contains("fn rf"));
        assert!(code.contains("fs :: read_to_string"));
    }

    #[test]
    fn test_generate_aliased_file_function_write_file() {
        let tokens = generate_aliased_file_function("write_file", "wf");
        assert!(tokens.is_some());
        let code = tokens.unwrap().to_string();
        assert!(code.contains("fn wf"));
        assert!(code.contains("fs :: write"));
    }

    #[test]
    fn test_generate_aliased_file_function_unknown() {
        let tokens = generate_aliased_file_function("unknown_func", "uf");
        assert!(tokens.is_none());
    }

    // ==================== generate_aliased_unknown_stub Tests ====================

    #[test]
    fn test_generate_aliased_unknown_stub_uses_alias() {
        let tokens = generate_aliased_unknown_stub("foo", "bar");
        let code = tokens.to_string();
        assert!(code.contains("fn bar"));
    }

    #[test]
    fn test_generate_aliased_unknown_stub_mentions_original() {
        let tokens = generate_aliased_unknown_stub("foo", "bar");
        let code = tokens.to_string();
        assert!(code.contains("foo"));
    }

    // ==================== is_std_fs_path Tests ====================

    #[test]
    fn test_is_std_fs_path_exact() {
        assert!(is_std_fs_path("std::fs"));
    }

    #[test]
    fn test_is_std_fs_path_with_function() {
        assert!(is_std_fs_path("std::fs::read_file"));
    }

    #[test]
    fn test_is_std_fs_path_other() {
        assert!(!is_std_fs_path("std::io"));
        assert!(!is_std_fs_path("fs"));
        assert!(!is_std_fs_path(""));
    }

    // ==================== is_std_process_path Tests ====================

    #[test]
    fn test_is_std_process_path_exact() {
        assert!(is_std_process_path("std::process"));
    }

    #[test]
    fn test_is_std_process_path_with_function() {
        assert!(is_std_process_path("std::process::exit"));
    }

    #[test]
    fn test_is_std_process_path_other() {
        assert!(!is_std_process_path("std::fs"));
        assert!(!is_std_process_path("process"));
    }

    // ==================== is_std_system_path Tests ====================

    #[test]
    fn test_is_std_system_path_exact() {
        assert!(is_std_system_path("std::system"));
    }

    #[test]
    fn test_is_std_system_path_with_function() {
        assert!(is_std_system_path("std::system::os_name"));
    }

    #[test]
    fn test_is_std_system_path_other() {
        assert!(!is_std_system_path("std::fs"));
        assert!(!is_std_system_path("system"));
    }

    // ==================== extract_function_from_path Tests ====================

    #[test]
    fn test_extract_function_from_path_read_file() {
        assert_eq!(
            extract_function_from_path("std::fs::read_file"),
            Some("read_file")
        );
    }

    #[test]
    fn test_extract_function_from_path_write_file() {
        assert_eq!(
            extract_function_from_path("std::fs::write_file"),
            Some("write_file")
        );
    }

    #[test]
    fn test_extract_function_from_path_no_prefix() {
        assert_eq!(extract_function_from_path("std::fs"), None);
        assert_eq!(extract_function_from_path("other::path"), None);
    }

    // ==================== is_known_file_operation Tests ====================

    #[test]
    fn test_is_known_file_operation_read() {
        assert!(is_known_file_operation("read_file"));
    }

    #[test]
    fn test_is_known_file_operation_write() {
        assert!(is_known_file_operation("write_file"));
    }

    #[test]
    fn test_is_known_file_operation_unknown() {
        assert!(!is_known_file_operation("delete_file"));
        assert!(!is_known_file_operation("copy_file"));
        assert!(!is_known_file_operation(""));
    }
}
