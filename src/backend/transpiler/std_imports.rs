//! Standard library import transpilation
//!
//! This module handles transpilation of std:: imports including:
//! - std::fs (file operations)
//! - std::process (process management)
//! - std::system (system information)
//! - std::signal (signal handling)
//! - std::time (time functions)
//! - std::mem (memory management)
//! - std::parallel (parallel processing)
//! - std::simd (SIMD vectorization)
//! - std::cache (caching)
//! - std::bench (benchmarking)
//! - std::profile (profiling)

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::import_helpers;
use crate::frontend::ast::ImportItem;

/// Core inline import transpilation logic - dispatches to handlers
#[must_use]
pub fn transpile_import_inline(path: &str, items: &[ImportItem]) -> TokenStream {
    if let Some(result) = handle_std_module_import(path, items) {
        return result;
    }
    handle_generic_import(path, items)
}

/// Dispatch std:: module imports to appropriate handler
#[must_use]
pub fn handle_std_module_import(path: &str, items: &[ImportItem]) -> Option<TokenStream> {
    if path.starts_with("std::fs") {
        return Some(transpile_std_fs_import_with_path(path, items));
    }
    if path.starts_with("std::process") {
        return Some(transpile_std_process_import());
    }
    if path.starts_with("std::system") {
        return Some(transpile_std_system_import());
    }
    if path.starts_with("std::signal") {
        return Some(transpile_std_signal_import());
    }
    if path.starts_with("std::time") {
        return Some(transpile_std_time_import());
    }
    if path.starts_with("std::mem") {
        return Some(transpile_std_mem_import());
    }
    if path.starts_with("std::parallel") {
        return Some(transpile_std_parallel_import());
    }
    if path.starts_with("std::simd") {
        return Some(transpile_std_simd_import());
    }
    if path.starts_with("std::cache") {
        return Some(transpile_std_cache_import());
    }
    if path.starts_with("std::bench") {
        return Some(transpile_std_bench_import());
    }
    if path.starts_with("std::profile") {
        return Some(transpile_std_profile_import());
    }
    None
}

/// Handle std::fs imports and generate file operation functions
#[must_use]
pub fn transpile_std_fs_import(items: &[ImportItem]) -> TokenStream {
    let mut tokens = TokenStream::new();
    tokens.extend(quote! { use std::fs; });

    if items.is_empty() || items.iter().any(|i| matches!(i, ImportItem::Wildcard)) {
        tokens.extend(import_helpers::generate_all_file_operations());
    } else {
        for item in items {
            match item {
                ImportItem::Named(name) => {
                    tokens.extend(generate_named_fs_function(name));
                }
                ImportItem::Aliased { name, alias } => {
                    tokens.extend(generate_aliased_fs_function(name, alias));
                }
                ImportItem::Wildcard => {
                    tokens.extend(import_helpers::generate_all_file_operations());
                }
            }
        }
    }
    tokens
}

/// Generate named fs function or stub
fn generate_named_fs_function(name: &str) -> TokenStream {
    match name {
        "read_file" => import_helpers::generate_read_file_function(),
        "write_file" => import_helpers::generate_write_file_function(),
        _ => {
            let func_name = format_ident!("{}", name);
            quote! {
                fn #func_name() -> ! {
                    panic!("std::fs::{} not yet implemented", #name);
                }
            }
        }
    }
}

/// Generate aliased fs function
fn generate_aliased_fs_function(name: &str, alias: &str) -> TokenStream {
    let alias_ident = format_ident!("{}", alias);
    match name {
        "read_file" => quote! {
            fn #alias_ident(filename: String) -> String {
                fs::read_to_string(filename).unwrap_or_else(|e| panic!("Failed to read file: {}", e))
            }
        },
        "write_file" => quote! {
            fn #alias_ident(filename: String, content: String) {
                fs::write(filename, content).unwrap_or_else(|e| panic!("Failed to write file: {}", e));
            }
        },
        _ => quote! {
            fn #alias_ident() -> ! {
                panic!("std::fs::{} not yet implemented", #name);
            }
        },
    }
}

/// Handle std::fs imports with path-based syntax
#[must_use]
pub fn transpile_std_fs_import_with_path(path: &str, items: &[ImportItem]) -> TokenStream {
    let mut tokens = TokenStream::new();
    tokens.extend(quote! { use std::fs; });

    if path == "std::fs" {
        let is_wildcard_import = items.is_empty()
            || items.iter().any(|i| matches!(i, ImportItem::Wildcard))
            || (items.len() == 1 && matches!(&items[0], ImportItem::Named(name) if name == "fs"));

        if is_wildcard_import {
            tokens.extend(import_helpers::generate_all_file_operations());
        } else {
            for item in items {
                match item {
                    ImportItem::Named(name) if is_known_fs_function(name) => {
                        tokens.extend(generate_named_fs_function(name));
                    }
                    ImportItem::Wildcard => {
                        tokens.extend(import_helpers::generate_all_file_operations());
                        break;
                    }
                    ImportItem::Aliased { name, .. } if is_known_fs_function(name) => {
                        tokens.extend(generate_named_fs_function(name));
                    }
                    _ => {}
                }
            }
        }
    } else if let Some(function_name) = path.strip_prefix("std::fs::") {
        if is_known_fs_function(function_name) {
            tokens.extend(generate_named_fs_function(function_name));
        }
    }
    tokens
}

/// Check if function name is a known fs function
#[must_use]
pub fn is_known_fs_function(name: &str) -> bool {
    matches!(name, "read_file" | "write_file")
}

/// Handle std::process imports with process management functions
#[must_use]
pub fn transpile_std_process_import() -> TokenStream {
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

/// Handle std::system imports with system information functions
#[must_use]
pub fn transpile_std_system_import() -> TokenStream {
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

/// Handle std::signal imports with signal handling functions
#[must_use]
pub fn transpile_std_signal_import() -> TokenStream {
    quote! {
        const SIGINT: i32 = 2;
        const SIGTERM: i32 = 15;
        const SIGKILL: i32 = 9;
        fn exit(code: i32) {
            std::process::exit(code);
        }
        mod signal {
            pub const SIGINT: i32 = 2;
            pub const SIGTERM: i32 = 15;
            pub const SIGKILL: i32 = 9;
            pub fn on(_signal: i32, _handler: impl Fn()) {
                // Signal handling stub
            }
        }
    }
}

/// Handle std::time imports
#[must_use]
pub fn transpile_std_time_import() -> TokenStream {
    quote! {
        mod time {
            pub fn now_millis() -> u64 {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("System time is before UNIX_EPOCH")
                    .as_millis() as u64
            }
            pub fn now_secs() -> u64 {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("System time is before UNIX_EPOCH")
                    .as_secs()
            }
            pub fn sleep(millis: u64) {
                std::thread::sleep(std::time::Duration::from_millis(millis));
            }
        }
    }
}

/// Handle std::mem imports with memory management
#[must_use]
pub fn transpile_std_mem_import() -> TokenStream {
    quote! {
        mod mem {
            pub struct Array<T> {
                data: Vec<T>,
            }
            impl<T: Clone> Array<T> {
                pub fn new(size: usize, default_value: T) -> Self {
                    Array { data: vec![default_value; size] }
                }
            }
            pub struct MemoryInfo {
                pub allocated: usize,
                pub peak: usize,
            }
            impl std::fmt::Display for MemoryInfo {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "allocated: {}KB, peak: {}KB", self.allocated / 1024, self.peak / 1024)
                }
            }
            pub fn usage() -> MemoryInfo {
                MemoryInfo { allocated: 1024 * 100, peak: 1024 * 150 }
            }
        }
    }
}

/// Handle std::parallel imports with parallel processing
#[must_use]
pub fn transpile_std_parallel_import() -> TokenStream {
    quote! {
        mod parallel {
            pub fn map<T, U, F>(data: Vec<T>, func: F) -> Vec<U>
            where
                T: Send,
                U: Send,
                F: Fn(T) -> U + Send + Sync,
            {
                data.into_iter().map(func).collect()
            }
            pub fn filter<T, F>(data: Vec<T>, predicate: F) -> Vec<T>
            where
                T: Send,
                F: Fn(&T) -> bool + Send + Sync,
            {
                data.into_iter().filter(|x| predicate(x)).collect()
            }
            pub fn reduce<T, F>(data: Vec<T>, func: F) -> Option<T>
            where
                T: Send,
                F: Fn(T, T) -> T + Send + Sync,
            {
                data.into_iter().reduce(func)
            }
        }
    }
}

/// Handle std::simd imports with SIMD vectorization
#[must_use]
pub fn transpile_std_simd_import() -> TokenStream {
    quote! {
        mod simd {
            use std::ops::Add;
            pub struct SimdVec<T> {
                data: Vec<T>,
            }
            impl<T> SimdVec<T> {
                pub fn from_slice(slice: &[T]) -> Self where T: Clone {
                    SimdVec { data: slice.to_vec() }
                }
            }
            impl<T> Add for SimdVec<T> where T: Add<Output = T> + Copy {
                type Output = SimdVec<T>;
                fn add(self, other: SimdVec<T>) -> SimdVec<T> {
                    let result: Vec<T> = self.data.iter()
                        .zip(other.data.iter())
                        .map(|(&a, &b)| a + b)
                        .collect();
                    SimdVec { data: result }
                }
            }
            impl<T: std::fmt::Display> std::fmt::Display for SimdVec<T> {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "[{}]", self.data.iter().map(|x| format!("{}", x)).collect::<Vec<_>>().join(", "))
                }
            }
            pub fn from_slice<T: Clone>(slice: &[T]) -> SimdVec<T> {
                SimdVec::from_slice(slice)
            }
        }
    }
}

/// Handle std::cache imports with caching
#[must_use]
pub fn transpile_std_cache_import() -> TokenStream {
    quote! {
        mod cache {
            use std::collections::HashMap;
            pub struct Cache<K, V> {
                data: HashMap<K, V>,
            }
            impl<K, V> Cache<K, V> where K: std::hash::Hash + Eq {
                pub fn new() -> Self {
                    Cache { data: HashMap::new() }
                }
                pub fn get(&self, key: &K) -> Option<&V> {
                    self.data.get(key)
                }
                pub fn insert(&mut self, key: K, value: V) -> Option<V> {
                    self.data.insert(key, value)
                }
            }
        }
    }
}

/// Handle std::bench imports with benchmarking
#[must_use]
pub fn transpile_std_bench_import() -> TokenStream {
    quote! {
        mod bench {
            use std::time::{Duration, Instant};
            pub struct BenchmarkResult {
                pub elapsed: u128,
            }
            impl BenchmarkResult {
                pub fn new(elapsed: Duration) -> Self {
                    BenchmarkResult { elapsed: elapsed.as_millis() }
                }
            }
            impl std::fmt::Display for BenchmarkResult {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "{}ms", self.elapsed)
                }
            }
            pub fn time<F, T>(mut func: F) -> BenchmarkResult where F: FnMut() -> T {
                let start = Instant::now();
                let _ = func();
                BenchmarkResult::new(start.elapsed())
            }
        }
    }
}

/// Handle std::profile imports with profiling
#[must_use]
pub fn transpile_std_profile_import() -> TokenStream {
    quote! {
        mod profile {
            pub struct ProfileInfo {
                pub function_name: String,
                pub call_count: usize,
                pub total_time: u128,
            }
            impl std::fmt::Display for ProfileInfo {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "{}: {} calls, {}ms total", self.function_name, self.call_count, self.total_time)
                }
            }
            pub fn get_stats(function_name: &str) -> ProfileInfo {
                ProfileInfo {
                    function_name: function_name.to_string(),
                    call_count: 42,
                    total_time: 100,
                }
            }
        }
    }
}

/// Handle generic (non-std) import paths
#[must_use]
pub fn handle_generic_import(path: &str, items: &[ImportItem]) -> TokenStream {
    let path_tokens = path_to_tokens(path);
    if items.is_empty() {
        quote! { use #path_tokens; }
    } else if items.len() == 1 {
        handle_single_import_item(&path_tokens, path, &items[0])
    } else {
        handle_multiple_import_items(&path_tokens, items)
    }
}

/// Convert path string to token stream
#[must_use]
pub fn path_to_tokens(path: &str) -> TokenStream {
    let mut path_tokens = TokenStream::new();
    let segments: Vec<_> = path.split("::").collect();
    for (i, segment) in segments.iter().enumerate() {
        if i > 0 {
            path_tokens.extend(quote! { :: });
        }
        if !segment.is_empty() {
            let seg_ident = format_ident!("{}", segment);
            path_tokens.extend(quote! { #seg_ident });
        }
    }
    path_tokens
}

/// Handle single import item
#[must_use]
pub fn handle_single_import_item(
    path_tokens: &TokenStream,
    path: &str,
    item: &ImportItem,
) -> TokenStream {
    match item {
        ImportItem::Named(name) => {
            if path.ends_with(&format!("::{name}")) {
                quote! { use #path_tokens; }
            } else {
                let item_ident = format_ident!("{}", name);
                quote! { use #path_tokens::#item_ident; }
            }
        }
        ImportItem::Aliased { name, alias } => {
            let name_ident = format_ident!("{}", name);
            let alias_ident = format_ident!("{}", alias);
            quote! { use #path_tokens::#name_ident as #alias_ident; }
        }
        ImportItem::Wildcard => quote! { use #path_tokens::*; },
    }
}

/// Handle multiple import items
#[must_use]
pub fn handle_multiple_import_items(
    path_tokens: &TokenStream,
    items: &[ImportItem],
) -> TokenStream {
    let item_tokens = process_import_items(items);
    quote! { use #path_tokens::{#(#item_tokens),*}; }
}

/// Process import items into token streams
#[must_use]
pub fn process_import_items(items: &[ImportItem]) -> Vec<TokenStream> {
    items
        .iter()
        .map(|item| match item {
            ImportItem::Named(name) => {
                let name_ident = format_ident!("{}", name);
                quote! { #name_ident }
            }
            ImportItem::Aliased { name, alias } => {
                let name_ident = format_ident!("{}", name);
                let alias_ident = format_ident!("{}", alias);
                quote! { #name_ident as #alias_ident }
            }
            ImportItem::Wildcard => quote! { * },
        })
        .collect()
}

/// Check if path is a std:: import
#[must_use]
pub fn is_std_import(path: &str) -> bool {
    path.starts_with("std::")
}

/// Get the std submodule from a path
#[must_use]
pub fn get_std_submodule(path: &str) -> Option<&str> {
    path.strip_prefix("std::").map(|rest| {
        rest.split("::").next().unwrap_or(rest)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== transpile_std_fs_import Tests ====================

    #[test]
    fn test_transpile_std_fs_import_empty_items() {
        let tokens = transpile_std_fs_import(&[]);
        let code = tokens.to_string();
        assert!(code.contains("use std :: fs"));
        assert!(code.contains("fn read_file"));
        assert!(code.contains("fn write_file"));
    }

    #[test]
    fn test_transpile_std_fs_import_wildcard() {
        let tokens = transpile_std_fs_import(&[ImportItem::Wildcard]);
        let code = tokens.to_string();
        assert!(code.contains("fn read_file"));
        assert!(code.contains("fn write_file"));
    }

    #[test]
    fn test_transpile_std_fs_import_named_read_file() {
        let tokens = transpile_std_fs_import(&[ImportItem::Named("read_file".to_string())]);
        let code = tokens.to_string();
        assert!(code.contains("fn read_file"));
        assert!(!code.contains("fn write_file"));
    }

    #[test]
    fn test_transpile_std_fs_import_named_write_file() {
        let tokens = transpile_std_fs_import(&[ImportItem::Named("write_file".to_string())]);
        let code = tokens.to_string();
        assert!(code.contains("fn write_file"));
        assert!(!code.contains("fn read_file"));
    }

    #[test]
    fn test_transpile_std_fs_import_both() {
        let tokens = transpile_std_fs_import(&[
            ImportItem::Named("read_file".to_string()),
            ImportItem::Named("write_file".to_string()),
        ]);
        let code = tokens.to_string();
        assert!(code.contains("fn read_file"));
        assert!(code.contains("fn write_file"));
    }

    #[test]
    fn test_transpile_std_fs_import_unknown() {
        let tokens = transpile_std_fs_import(&[ImportItem::Named("unknown_func".to_string())]);
        let code = tokens.to_string();
        assert!(code.contains("fn unknown_func"));
        assert!(code.contains("panic !"));
        assert!(code.contains("not yet implemented"));
    }

    #[test]
    fn test_transpile_std_fs_import_aliased_read() {
        let tokens = transpile_std_fs_import(&[ImportItem::Aliased {
            name: "read_file".to_string(),
            alias: "rf".to_string(),
        }]);
        let code = tokens.to_string();
        assert!(code.contains("fn rf"));
        assert!(code.contains("fs :: read_to_string"));
    }

    #[test]
    fn test_transpile_std_fs_import_aliased_write() {
        let tokens = transpile_std_fs_import(&[ImportItem::Aliased {
            name: "write_file".to_string(),
            alias: "wf".to_string(),
        }]);
        let code = tokens.to_string();
        assert!(code.contains("fn wf"));
        assert!(code.contains("fs :: write"));
    }

    #[test]
    fn test_transpile_std_fs_import_aliased_unknown() {
        let tokens = transpile_std_fs_import(&[ImportItem::Aliased {
            name: "unknown".to_string(),
            alias: "unk".to_string(),
        }]);
        let code = tokens.to_string();
        assert!(code.contains("fn unk"));
        assert!(code.contains("not yet implemented"));
    }

    // ==================== transpile_std_fs_import_with_path Tests ====================

    #[test]
    fn test_transpile_std_fs_with_path_exact() {
        let tokens = transpile_std_fs_import_with_path("std::fs", &[]);
        let code = tokens.to_string();
        assert!(code.contains("fn read_file"));
        assert!(code.contains("fn write_file"));
    }

    #[test]
    fn test_transpile_std_fs_with_path_specific_func() {
        let tokens = transpile_std_fs_import_with_path("std::fs::read_file", &[]);
        let code = tokens.to_string();
        assert!(code.contains("fn read_file"));
        assert!(!code.contains("fn write_file"));
    }

    #[test]
    fn test_transpile_std_fs_with_path_write_file() {
        let tokens = transpile_std_fs_import_with_path("std::fs::write_file", &[]);
        let code = tokens.to_string();
        assert!(code.contains("fn write_file"));
        assert!(!code.contains("fn read_file"));
    }

    #[test]
    fn test_transpile_std_fs_with_path_unknown() {
        let tokens = transpile_std_fs_import_with_path("std::fs::unknown", &[]);
        let code = tokens.to_string();
        // Unknown function - no output besides use statement
        assert!(code.contains("use std :: fs"));
        assert!(!code.contains("fn unknown"));
    }

    #[test]
    fn test_transpile_std_fs_with_wildcard_item() {
        let tokens = transpile_std_fs_import_with_path("std::fs", &[ImportItem::Wildcard]);
        let code = tokens.to_string();
        assert!(code.contains("fn read_file"));
        assert!(code.contains("fn write_file"));
    }

    // ==================== is_known_fs_function Tests ====================

    #[test]
    fn test_is_known_fs_function_read() {
        assert!(is_known_fs_function("read_file"));
    }

    #[test]
    fn test_is_known_fs_function_write() {
        assert!(is_known_fs_function("write_file"));
    }

    #[test]
    fn test_is_known_fs_function_unknown() {
        assert!(!is_known_fs_function("delete_file"));
        assert!(!is_known_fs_function("copy_file"));
        assert!(!is_known_fs_function(""));
    }

    // ==================== transpile_std_process_import Tests ====================

    #[test]
    fn test_transpile_std_process_import_mod() {
        let tokens = transpile_std_process_import();
        let code = tokens.to_string();
        assert!(code.contains("mod process"));
    }

    #[test]
    fn test_transpile_std_process_import_current_pid() {
        let tokens = transpile_std_process_import();
        let code = tokens.to_string();
        assert!(code.contains("fn current_pid"));
        assert!(code.contains("std :: process :: id"));
    }

    #[test]
    fn test_transpile_std_process_import_exit() {
        let tokens = transpile_std_process_import();
        let code = tokens.to_string();
        assert!(code.contains("fn exit"));
        assert!(code.contains("std :: process :: exit"));
    }

    #[test]
    fn test_transpile_std_process_import_spawn() {
        let tokens = transpile_std_process_import();
        let code = tokens.to_string();
        assert!(code.contains("fn spawn"));
        assert!(code.contains("Command :: new"));
    }

    // ==================== transpile_std_system_import Tests ====================

    #[test]
    fn test_transpile_std_system_import_mod() {
        let tokens = transpile_std_system_import();
        let code = tokens.to_string();
        assert!(code.contains("mod system"));
    }

    #[test]
    fn test_transpile_std_system_import_get_env() {
        let tokens = transpile_std_system_import();
        let code = tokens.to_string();
        assert!(code.contains("fn get_env"));
        assert!(code.contains("std :: env :: var"));
    }

    #[test]
    fn test_transpile_std_system_import_set_env() {
        let tokens = transpile_std_system_import();
        let code = tokens.to_string();
        assert!(code.contains("fn set_env"));
        assert!(code.contains("std :: env :: set_var"));
    }

    #[test]
    fn test_transpile_std_system_import_os_name() {
        let tokens = transpile_std_system_import();
        let code = tokens.to_string();
        assert!(code.contains("fn os_name"));
        assert!(code.contains("std :: env :: consts :: OS"));
    }

    #[test]
    fn test_transpile_std_system_import_arch() {
        let tokens = transpile_std_system_import();
        let code = tokens.to_string();
        assert!(code.contains("fn arch"));
        assert!(code.contains("std :: env :: consts :: ARCH"));
    }

    // ==================== transpile_std_signal_import Tests ====================

    #[test]
    fn test_transpile_std_signal_import_constants() {
        let tokens = transpile_std_signal_import();
        let code = tokens.to_string();
        assert!(code.contains("const SIGINT"));
        assert!(code.contains("const SIGTERM"));
        assert!(code.contains("const SIGKILL"));
    }

    #[test]
    fn test_transpile_std_signal_import_mod() {
        let tokens = transpile_std_signal_import();
        let code = tokens.to_string();
        assert!(code.contains("mod signal"));
    }

    #[test]
    fn test_transpile_std_signal_import_on_handler() {
        let tokens = transpile_std_signal_import();
        let code = tokens.to_string();
        assert!(code.contains("fn on"));
    }

    #[test]
    fn test_transpile_std_signal_import_exit() {
        let tokens = transpile_std_signal_import();
        let code = tokens.to_string();
        assert!(code.contains("fn exit"));
    }

    // ==================== transpile_std_time_import Tests ====================

    #[test]
    fn test_transpile_std_time_import_mod() {
        let tokens = transpile_std_time_import();
        let code = tokens.to_string();
        assert!(code.contains("mod time"));
    }

    #[test]
    fn test_transpile_std_time_import_now_millis() {
        let tokens = transpile_std_time_import();
        let code = tokens.to_string();
        assert!(code.contains("fn now_millis"));
        assert!(code.contains("as_millis"));
    }

    #[test]
    fn test_transpile_std_time_import_now_secs() {
        let tokens = transpile_std_time_import();
        let code = tokens.to_string();
        assert!(code.contains("fn now_secs"));
        assert!(code.contains("as_secs"));
    }

    #[test]
    fn test_transpile_std_time_import_sleep() {
        let tokens = transpile_std_time_import();
        let code = tokens.to_string();
        assert!(code.contains("fn sleep"));
        assert!(code.contains("thread :: sleep"));
    }

    // ==================== is_std_import Tests ====================

    #[test]
    fn test_is_std_import_true() {
        assert!(is_std_import("std::fs"));
        assert!(is_std_import("std::process"));
        assert!(is_std_import("std::fs::read_file"));
    }

    #[test]
    fn test_is_std_import_false() {
        assert!(!is_std_import("fs"));
        assert!(!is_std_import("serde::json"));
        assert!(!is_std_import(""));
    }

    // ==================== get_std_submodule Tests ====================

    #[test]
    fn test_get_std_submodule_fs() {
        assert_eq!(get_std_submodule("std::fs"), Some("fs"));
    }

    #[test]
    fn test_get_std_submodule_process() {
        assert_eq!(get_std_submodule("std::process"), Some("process"));
    }

    #[test]
    fn test_get_std_submodule_nested() {
        assert_eq!(get_std_submodule("std::fs::read_file"), Some("fs"));
    }

    #[test]
    fn test_get_std_submodule_not_std() {
        assert_eq!(get_std_submodule("serde::json"), None);
        assert_eq!(get_std_submodule("fs"), None);
    }

    // ==================== transpile_import_inline Tests ====================

    #[test]
    fn test_transpile_import_inline_std_fs() {
        let result = transpile_import_inline("std::fs", &[]);
        let code = result.to_string();
        assert!(code.contains("use std :: fs"));
    }

    #[test]
    fn test_transpile_import_inline_std_process() {
        let result = transpile_import_inline("std::process", &[]);
        let code = result.to_string();
        assert!(code.contains("mod process"));
    }

    #[test]
    fn test_transpile_import_inline_generic() {
        let result = transpile_import_inline("foo::bar", &[]);
        let code = result.to_string();
        assert!(code.contains("use foo :: bar"));
    }

    #[test]
    fn test_transpile_import_inline_with_items() {
        let items = vec![ImportItem::Named("baz".to_string())];
        let result = transpile_import_inline("foo::bar", &items);
        let code = result.to_string();
        assert!(code.contains("baz"));
    }

    // ==================== handle_std_module_import Tests ====================

    #[test]
    fn test_handle_std_module_import_fs() {
        assert!(handle_std_module_import("std::fs", &[]).is_some());
    }

    #[test]
    fn test_handle_std_module_import_process() {
        assert!(handle_std_module_import("std::process", &[]).is_some());
    }

    #[test]
    fn test_handle_std_module_import_system() {
        assert!(handle_std_module_import("std::system", &[]).is_some());
    }

    #[test]
    fn test_handle_std_module_import_signal() {
        assert!(handle_std_module_import("std::signal", &[]).is_some());
    }

    #[test]
    fn test_handle_std_module_import_time() {
        assert!(handle_std_module_import("std::time", &[]).is_some());
    }

    #[test]
    fn test_handle_std_module_import_mem() {
        assert!(handle_std_module_import("std::mem", &[]).is_some());
    }

    #[test]
    fn test_handle_std_module_import_parallel() {
        assert!(handle_std_module_import("std::parallel", &[]).is_some());
    }

    #[test]
    fn test_handle_std_module_import_simd() {
        assert!(handle_std_module_import("std::simd", &[]).is_some());
    }

    #[test]
    fn test_handle_std_module_import_cache() {
        assert!(handle_std_module_import("std::cache", &[]).is_some());
    }

    #[test]
    fn test_handle_std_module_import_bench() {
        assert!(handle_std_module_import("std::bench", &[]).is_some());
    }

    #[test]
    fn test_handle_std_module_import_profile() {
        assert!(handle_std_module_import("std::profile", &[]).is_some());
    }

    #[test]
    fn test_handle_std_module_import_unknown() {
        assert!(handle_std_module_import("std::unknown", &[]).is_none());
    }

    #[test]
    fn test_handle_std_module_import_non_std() {
        assert!(handle_std_module_import("foo::bar", &[]).is_none());
    }

    // ==================== transpile_std_mem_import Tests ====================

    #[test]
    fn test_transpile_std_mem_import_mod() {
        let tokens = transpile_std_mem_import();
        let code = tokens.to_string();
        assert!(code.contains("mod mem"));
    }

    #[test]
    fn test_transpile_std_mem_import_array() {
        let tokens = transpile_std_mem_import();
        let code = tokens.to_string();
        assert!(code.contains("struct Array"));
    }

    #[test]
    fn test_transpile_std_mem_import_memory_info() {
        let tokens = transpile_std_mem_import();
        let code = tokens.to_string();
        assert!(code.contains("struct MemoryInfo"));
    }

    #[test]
    fn test_transpile_std_mem_import_usage() {
        let tokens = transpile_std_mem_import();
        let code = tokens.to_string();
        assert!(code.contains("fn usage"));
    }

    // ==================== transpile_std_parallel_import Tests ====================

    #[test]
    fn test_transpile_std_parallel_import_mod() {
        let tokens = transpile_std_parallel_import();
        let code = tokens.to_string();
        assert!(code.contains("mod parallel"));
    }

    #[test]
    fn test_transpile_std_parallel_import_map() {
        let tokens = transpile_std_parallel_import();
        let code = tokens.to_string();
        assert!(code.contains("fn map"));
    }

    #[test]
    fn test_transpile_std_parallel_import_filter() {
        let tokens = transpile_std_parallel_import();
        let code = tokens.to_string();
        assert!(code.contains("fn filter"));
    }

    #[test]
    fn test_transpile_std_parallel_import_reduce() {
        let tokens = transpile_std_parallel_import();
        let code = tokens.to_string();
        assert!(code.contains("fn reduce"));
    }

    // ==================== transpile_std_simd_import Tests ====================

    #[test]
    fn test_transpile_std_simd_import_mod() {
        let tokens = transpile_std_simd_import();
        let code = tokens.to_string();
        assert!(code.contains("mod simd"));
    }

    #[test]
    fn test_transpile_std_simd_import_simd_vec() {
        let tokens = transpile_std_simd_import();
        let code = tokens.to_string();
        assert!(code.contains("struct SimdVec"));
    }

    #[test]
    fn test_transpile_std_simd_import_from_slice() {
        let tokens = transpile_std_simd_import();
        let code = tokens.to_string();
        assert!(code.contains("fn from_slice"));
    }

    #[test]
    fn test_transpile_std_simd_import_add() {
        let tokens = transpile_std_simd_import();
        let code = tokens.to_string();
        assert!(code.contains("impl < T > Add for SimdVec"));
    }

    // ==================== transpile_std_cache_import Tests ====================

    #[test]
    fn test_transpile_std_cache_import_mod() {
        let tokens = transpile_std_cache_import();
        let code = tokens.to_string();
        assert!(code.contains("mod cache"));
    }

    #[test]
    fn test_transpile_std_cache_import_cache_struct() {
        let tokens = transpile_std_cache_import();
        let code = tokens.to_string();
        assert!(code.contains("struct Cache"));
    }

    #[test]
    fn test_transpile_std_cache_import_new() {
        let tokens = transpile_std_cache_import();
        let code = tokens.to_string();
        assert!(code.contains("fn new"));
    }

    #[test]
    fn test_transpile_std_cache_import_get() {
        let tokens = transpile_std_cache_import();
        let code = tokens.to_string();
        assert!(code.contains("fn get"));
    }

    #[test]
    fn test_transpile_std_cache_import_insert() {
        let tokens = transpile_std_cache_import();
        let code = tokens.to_string();
        assert!(code.contains("fn insert"));
    }

    // ==================== transpile_std_bench_import Tests ====================

    #[test]
    fn test_transpile_std_bench_import_mod() {
        let tokens = transpile_std_bench_import();
        let code = tokens.to_string();
        assert!(code.contains("mod bench"));
    }

    #[test]
    fn test_transpile_std_bench_import_benchmark_result() {
        let tokens = transpile_std_bench_import();
        let code = tokens.to_string();
        assert!(code.contains("struct BenchmarkResult"));
    }

    #[test]
    fn test_transpile_std_bench_import_time() {
        let tokens = transpile_std_bench_import();
        let code = tokens.to_string();
        assert!(code.contains("fn time"));
    }

    #[test]
    fn test_transpile_std_bench_import_uses_instant() {
        let tokens = transpile_std_bench_import();
        let code = tokens.to_string();
        assert!(code.contains("Instant :: now"));
    }

    // ==================== transpile_std_profile_import Tests ====================

    #[test]
    fn test_transpile_std_profile_import_mod() {
        let tokens = transpile_std_profile_import();
        let code = tokens.to_string();
        assert!(code.contains("mod profile"));
    }

    #[test]
    fn test_transpile_std_profile_import_profile_info() {
        let tokens = transpile_std_profile_import();
        let code = tokens.to_string();
        assert!(code.contains("struct ProfileInfo"));
    }

    #[test]
    fn test_transpile_std_profile_import_get_stats() {
        let tokens = transpile_std_profile_import();
        let code = tokens.to_string();
        assert!(code.contains("fn get_stats"));
    }

    // ==================== handle_generic_import Tests ====================

    #[test]
    fn test_handle_generic_import_empty() {
        let result = handle_generic_import("foo::bar", &[]);
        let code = result.to_string();
        assert!(code.contains("use foo :: bar"));
    }

    #[test]
    fn test_handle_generic_import_single_named() {
        let items = vec![ImportItem::Named("baz".to_string())];
        let result = handle_generic_import("foo::bar", &items);
        let code = result.to_string();
        assert!(code.contains("baz"));
    }

    #[test]
    fn test_handle_generic_import_single_wildcard() {
        let items = vec![ImportItem::Wildcard];
        let result = handle_generic_import("foo::bar", &items);
        let code = result.to_string();
        assert!(code.contains("*"));
    }

    #[test]
    fn test_handle_generic_import_multiple() {
        let items = vec![
            ImportItem::Named("a".to_string()),
            ImportItem::Named("b".to_string()),
        ];
        let result = handle_generic_import("foo::bar", &items);
        let code = result.to_string();
        assert!(code.contains("a"));
        assert!(code.contains("b"));
    }

    // ==================== path_to_tokens Tests ====================

    #[test]
    fn test_path_to_tokens_simple() {
        let result = path_to_tokens("foo");
        let code = result.to_string();
        assert_eq!(code.trim(), "foo");
    }

    #[test]
    fn test_path_to_tokens_two_segments() {
        let result = path_to_tokens("foo::bar");
        let code = result.to_string();
        assert!(code.contains("foo"));
        assert!(code.contains("bar"));
    }

    #[test]
    fn test_path_to_tokens_three_segments() {
        let result = path_to_tokens("foo::bar::baz");
        let code = result.to_string();
        assert!(code.contains("foo"));
        assert!(code.contains("bar"));
        assert!(code.contains("baz"));
    }

    #[test]
    fn test_path_to_tokens_empty() {
        let result = path_to_tokens("");
        let code = result.to_string();
        assert!(code.is_empty());
    }

    // ==================== handle_single_import_item Tests ====================

    #[test]
    fn test_handle_single_import_item_named() {
        let path_tokens = path_to_tokens("foo::bar");
        let item = ImportItem::Named("baz".to_string());
        let result = handle_single_import_item(&path_tokens, "foo::bar", &item);
        let code = result.to_string();
        assert!(code.contains("baz"));
    }

    #[test]
    fn test_handle_single_import_item_suffix_match() {
        let path_tokens = path_to_tokens("foo::bar::baz");
        let item = ImportItem::Named("baz".to_string());
        let result = handle_single_import_item(&path_tokens, "foo::bar::baz", &item);
        let code = result.to_string();
        assert!(code.contains("foo"));
    }

    #[test]
    fn test_handle_single_import_item_aliased() {
        let path_tokens = path_to_tokens("foo::bar");
        let item = ImportItem::Aliased {
            name: "baz".to_string(),
            alias: "qux".to_string(),
        };
        let result = handle_single_import_item(&path_tokens, "foo::bar", &item);
        let code = result.to_string();
        assert!(code.contains("baz as qux"));
    }

    #[test]
    fn test_handle_single_import_item_wildcard() {
        let path_tokens = path_to_tokens("foo::bar");
        let item = ImportItem::Wildcard;
        let result = handle_single_import_item(&path_tokens, "foo::bar", &item);
        let code = result.to_string();
        assert!(code.contains("*"));
    }

    // ==================== handle_multiple_import_items Tests ====================

    #[test]
    fn test_handle_multiple_import_items_two() {
        let path_tokens = path_to_tokens("foo::bar");
        let items = vec![
            ImportItem::Named("a".to_string()),
            ImportItem::Named("b".to_string()),
        ];
        let result = handle_multiple_import_items(&path_tokens, &items);
        let code = result.to_string();
        assert!(code.contains("a"));
        assert!(code.contains("b"));
    }

    #[test]
    fn test_handle_multiple_import_items_mixed() {
        let path_tokens = path_to_tokens("foo::bar");
        let items = vec![
            ImportItem::Named("a".to_string()),
            ImportItem::Aliased {
                name: "b".to_string(),
                alias: "c".to_string(),
            },
        ];
        let result = handle_multiple_import_items(&path_tokens, &items);
        let code = result.to_string();
        assert!(code.contains("a"));
        assert!(code.contains("b as c"));
    }

    // ==================== process_import_items Tests ====================

    #[test]
    fn test_process_import_items_named() {
        let items = vec![ImportItem::Named("foo".to_string())];
        let result = process_import_items(&items);
        assert_eq!(result.len(), 1);
        assert!(result[0].to_string().contains("foo"));
    }

    #[test]
    fn test_process_import_items_aliased() {
        let items = vec![ImportItem::Aliased {
            name: "foo".to_string(),
            alias: "bar".to_string(),
        }];
        let result = process_import_items(&items);
        assert_eq!(result.len(), 1);
        assert!(result[0].to_string().contains("foo as bar"));
    }

    #[test]
    fn test_process_import_items_wildcard() {
        let items = vec![ImportItem::Wildcard];
        let result = process_import_items(&items);
        assert_eq!(result.len(), 1);
        assert!(result[0].to_string().contains("*"));
    }

    #[test]
    fn test_process_import_items_multiple() {
        let items = vec![
            ImportItem::Named("a".to_string()),
            ImportItem::Named("b".to_string()),
            ImportItem::Named("c".to_string()),
        ];
        let result = process_import_items(&items);
        assert_eq!(result.len(), 3);
    }
}
