//! Filesystem builtin functions
//!
//! This module handles filesystem operations including read, write, exists,
//! directory operations, walking, globbing, searching, and hashing.

use crate::runtime::validation::validate_arg_count;
use crate::runtime::{InterpreterError, Value};

use std::collections::HashMap;
use std::sync::Arc;

/// Evaluate `fs_read()` builtin function
/// Reads file contents and returns as string
/// Complexity: 3 (within Toyota Way limits)
pub(crate) fn eval_fs_read(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_read", args, 1)?;

    match &args[0] {
        Value::String(path) => match std::fs::read_to_string(path.as_ref()) {
            Ok(content) => Ok(Value::EnumVariant {
                enum_name: "Result".to_string(),
                variant_name: "Ok".to_string(),
                data: Some(vec![Value::from_string(content)]),
            }),
            Err(e) => Ok(Value::EnumVariant {
                enum_name: "Result".to_string(),
                variant_name: "Err".to_string(),
                data: Some(vec![Value::from_string(e.to_string())]),
            }),
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_read() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate `read_file()` builtin function (ISSUE-121)
/// Unwrapping helper for benchmarks - returns plain string instead of Result enum
/// Complexity: 2 (within Toyota Way limits)
pub(crate) fn eval_read_file_unwrapped(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("read_file", args, 1)?;

    match &args[0] {
        Value::String(path) => std::fs::read_to_string(path.as_ref())
            .map(Value::from_string)
            .map_err(|e| {
                InterpreterError::RuntimeError(format!("Failed to read file '{path}': {e}"))
            }),
        _ => Err(InterpreterError::RuntimeError(
            "read_file() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate `fs_write()` builtin function
/// Writes content to file
/// Returns Result enum to match Rust API (RUNTIME-096)
/// Complexity: 4 (within Toyota Way limits)
pub(crate) fn eval_fs_write(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_write", args, 2)?;

    match (&args[0], &args[1]) {
        (Value::String(path), Value::String(content)) => {
            match std::fs::write(path.as_ref(), content.as_ref()) {
                Ok(()) => Ok(Value::EnumVariant {
                    enum_name: "Result".to_string(),
                    variant_name: "Ok".to_string(),
                    data: Some(vec![Value::Nil]),
                }),
                Err(e) => Ok(Value::EnumVariant {
                    enum_name: "Result".to_string(),
                    variant_name: "Err".to_string(),
                    data: Some(vec![Value::from_string(e.to_string())]),
                }),
            }
        }
        _ => Err(InterpreterError::RuntimeError(
            "fs_write() expects two string arguments".to_string(),
        )),
    }
}

/// Append content to file (creates if doesn't exist)
/// Wraps `std::fs::OpenOptions` with append(true) and create(true)
/// Complexity: 3 (within Toyota Way limits)
pub(crate) fn eval_append_file(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("append_file", args, 2)?;

    match (&args[0], &args[1]) {
        (Value::String(path), Value::String(content)) => {
            use std::fs::OpenOptions;
            use std::io::Write;

            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(path.as_ref())
            {
                Ok(mut file) => match file.write_all(content.as_bytes()) {
                    Ok(()) => Ok(Value::Nil),
                    Err(e) => Err(InterpreterError::RuntimeError(format!(
                        "Failed to append to file: {e}"
                    ))),
                },
                Err(e) => Err(InterpreterError::RuntimeError(format!(
                    "Failed to open file for append: {e}"
                ))),
            }
        }
        _ => Err(InterpreterError::RuntimeError(
            "append_file() expects two string arguments".to_string(),
        )),
    }
}

/// Evaluate `fs_exists()` builtin function
/// Checks if path exists
/// Complexity: 2 (within Toyota Way limits)
pub(crate) fn eval_fs_exists(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_exists", args, 1)?;

    match &args[0] {
        Value::String(path) => Ok(Value::Bool(std::path::Path::new(path.as_ref()).exists())),
        _ => Err(InterpreterError::RuntimeError(
            "fs_exists() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate `fs_create_dir()` builtin function
/// Creates directory (including parent directories)
/// Complexity: 4 (within Toyota Way limits)
pub(crate) fn eval_fs_create_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_create_dir", args, 1)?;

    match &args[0] {
        Value::String(path) => match std::fs::create_dir_all(path.as_ref()) {
            Ok(()) => Ok(Value::EnumVariant {
                enum_name: "Result".to_string(),
                variant_name: "Ok".to_string(),
                data: Some(vec![Value::Nil]),
            }),
            Err(e) => Ok(Value::EnumVariant {
                enum_name: "Result".to_string(),
                variant_name: "Err".to_string(),
                data: Some(vec![Value::from_string(e.to_string())]),
            }),
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_create_dir() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate `fs_remove_file()` builtin function
/// Removes a file. Idempotent: OK if already deleted.
/// Complexity: 5 (within Toyota Way limits)
pub(crate) fn eval_fs_remove_file(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_remove_file", args, 1)?;

    match &args[0] {
        Value::String(path) => match std::fs::remove_file(path.as_ref()) {
            Ok(()) => Ok(Value::EnumVariant {
                enum_name: "Result".to_string(),
                variant_name: "Ok".to_string(),
                data: Some(vec![Value::Nil]),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::EnumVariant {
                enum_name: "Result".to_string(),
                variant_name: "Ok".to_string(),
                data: Some(vec![Value::Nil]),
            }),
            Err(e) => Ok(Value::EnumVariant {
                enum_name: "Result".to_string(),
                variant_name: "Err".to_string(),
                data: Some(vec![Value::from_string(e.to_string())]),
            }),
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_remove_file() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate `fs_remove_dir()` builtin function
/// Complexity: 4 (within Toyota Way limits)
pub(crate) fn eval_fs_remove_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_remove_dir", args, 1)?;

    match &args[0] {
        Value::String(path) => match std::fs::remove_dir(path.as_ref()) {
            Ok(()) => Ok(Value::EnumVariant {
                enum_name: "Result".to_string(),
                variant_name: "Ok".to_string(),
                data: Some(vec![Value::Nil]),
            }),
            Err(e) => Ok(Value::EnumVariant {
                enum_name: "Result".to_string(),
                variant_name: "Err".to_string(),
                data: Some(vec![Value::from_string(e.to_string())]),
            }),
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_remove_dir() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate `walk()` builtin function (STDLIB-005)
/// Recursively walks a directory and returns array of `FileEntry` objects
/// Complexity: 8 (within Toyota Way limit of 10)
pub(crate) fn eval_walk(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("walk", args, 1)?;

    match &args[0] {
        Value::String(path) => {
            use walkdir::WalkDir;

            let entries: Vec<Value> = WalkDir::new(path.as_ref())
                .into_iter()
                .filter_map(std::result::Result::ok)
                .map(|entry| {
                    let mut fields = HashMap::new();
                    fields.insert(
                        "path".to_string(),
                        Value::String(entry.path().display().to_string().into()),
                    );
                    fields.insert(
                        "name".to_string(),
                        Value::String(entry.file_name().to_string_lossy().to_string().into()),
                    );
                    fields.insert(
                        "is_file".to_string(),
                        Value::from_bool(entry.file_type().is_file()),
                    );
                    fields.insert(
                        "is_dir".to_string(),
                        Value::from_bool(entry.file_type().is_dir()),
                    );
                    fields.insert(
                        "is_symlink".to_string(),
                        Value::from_bool(entry.file_type().is_symlink()),
                    );
                    let size = entry.metadata().map(|m| m.len() as i64).unwrap_or(0);
                    fields.insert("size".to_string(), Value::Integer(size));
                    fields.insert("depth".to_string(), Value::Integer(entry.depth() as i64));
                    Value::Object(Arc::new(fields))
                })
                .collect();

            Ok(Value::Array(entries.into()))
        }
        _ => Err(InterpreterError::RuntimeError(
            "walk() expects a string path".to_string(),
        )),
    }
}

/// Evaluate `glob()` builtin function (STDLIB-005)
/// Complexity: 4 (within Toyota Way limit of 10)
pub(crate) fn eval_glob(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("glob", args, 1)?;

    match &args[0] {
        Value::String(pattern) => {
            use glob::glob;

            match glob(pattern.as_ref()) {
                Ok(paths) => {
                    let results: Vec<Value> = paths
                        .filter_map(std::result::Result::ok)
                        .map(|path| Value::String(path.display().to_string().into()))
                        .collect();
                    Ok(Value::Array(results.into()))
                }
                Err(e) => Err(InterpreterError::RuntimeError(format!(
                    "glob() pattern error: {e}"
                ))),
            }
        }
        _ => Err(InterpreterError::RuntimeError(
            "glob() expects a string pattern".to_string(),
        )),
    }
}

pub(crate) fn parse_search_case_insensitive(args: &[Value]) -> bool {
    if args.len() != 3 {
        return false;
    }
    let Value::Object(opts) = &args[2] else {
        return false;
    };
    opts.get("case_insensitive")
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
        .unwrap_or(false)
}

pub(crate) fn search_file_for_matches(
    path: &std::path::Path,
    re: &regex::Regex,
    results: &mut Vec<Value>,
) {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return;
    };
    for (line_num, line) in contents.lines().enumerate() {
        if !re.is_match(line) {
            continue;
        }
        let mut fields = HashMap::new();
        fields.insert(
            "path".to_string(),
            Value::String(path.display().to_string().into()),
        );
        fields.insert(
            "line_num".to_string(),
            Value::Integer((line_num + 1) as i64),
        );
        fields.insert("line".to_string(), Value::String(line.to_string().into()));
        results.push(Value::Object(Arc::new(fields)));
    }
}

pub(crate) fn eval_search(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(InterpreterError::RuntimeError(
            "search() expects 2-3 arguments: (pattern, path, options?)".to_string(),
        ));
    }

    let (Value::String(pattern), Value::String(path)) = (&args[0], &args[1]) else {
        return Err(InterpreterError::RuntimeError(
            "search() expects (string pattern, string path, object? options)".to_string(),
        ));
    };

    use regex::RegexBuilder;
    use walkdir::WalkDir;

    let case_insensitive = parse_search_case_insensitive(args);
    let re = RegexBuilder::new(pattern.as_ref())
        .case_insensitive(case_insensitive)
        .build()
        .map_err(|e| InterpreterError::RuntimeError(format!("search() regex error: {e}")))?;

    let mut results = Vec::new();
    for entry in WalkDir::new(path.as_ref())
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        search_file_for_matches(entry.path(), &re, &mut results);
    }

    Ok(Value::Array(results.into()))
}

/// Evaluate `walk_with_options()` builtin function (STDLIB-005)
/// Complexity: 7 (within Toyota Way limit of 10)
pub(crate) fn eval_walk_with_options(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError(
            "walk_with_options() expects 2 arguments: (path, options)".to_string(),
        ));
    }

    match (&args[0], &args[1]) {
        (Value::String(path), Value::Object(opts)) => {
            use walkdir::WalkDir;

            let mut walker = WalkDir::new(path.as_ref());

            if let Some(Value::Integer(max)) = opts.get("max_depth") {
                walker = walker.max_depth(*max as usize);
            }
            if let Some(Value::Integer(min)) = opts.get("min_depth") {
                walker = walker.min_depth(*min as usize);
            }
            if let Some(Value::Bool(follow)) = opts.get("follow_links") {
                walker = walker.follow_links(*follow);
            }

            let mut results = Vec::new();

            for entry in walker.into_iter().filter_map(std::result::Result::ok) {
                let mut fields = HashMap::new();
                fields.insert(
                    "path".to_string(),
                    Value::String(entry.path().display().to_string().into()),
                );
                if let Some(name) = entry.file_name().to_str() {
                    fields.insert("name".to_string(), Value::String(name.to_string().into()));
                }
                let file_type = entry.file_type();
                fields.insert("is_file".to_string(), Value::Bool(file_type.is_file()));
                fields.insert("is_dir".to_string(), Value::Bool(file_type.is_dir()));
                fields.insert(
                    "is_symlink".to_string(),
                    Value::Bool(file_type.is_symlink()),
                );
                let size = if file_type.is_file() {
                    entry.metadata().ok().map_or(0, |m| m.len())
                } else {
                    0
                };
                fields.insert("size".to_string(), Value::Integer(size as i64));
                fields.insert("depth".to_string(), Value::Integer(entry.depth() as i64));
                results.push(Value::Object(Arc::new(fields)));
            }

            Ok(Value::Array(results.into()))
        }
        _ => Err(InterpreterError::RuntimeError(
            "walk_with_options() expects (string path, object options)".to_string(),
        )),
    }
}

/// Evaluate `walk_parallel()` builtin function (STDLIB-005)
/// Parallel directory walking using rayon for optimal I/O performance
/// Complexity: 8 (within Toyota Way limit of <=10)
pub(crate) fn eval_walk_parallel(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("walk_parallel", args, 1)?;

    match &args[0] {
        Value::String(path) => {
            use rayon::prelude::*;
            use walkdir::WalkDir;

            let entries: Vec<_> = WalkDir::new(path.as_ref())
                .into_iter()
                .filter_map(std::result::Result::ok)
                .par_bridge()
                .map(|entry: walkdir::DirEntry| {
                    let path_str = entry.path().display().to_string();
                    let name_str = entry.file_name().to_string_lossy().to_string();
                    let file_type = entry.file_type();
                    let is_file = file_type.is_file();
                    let is_dir = file_type.is_dir();
                    let is_symlink = file_type.is_symlink();
                    let size = entry
                        .metadata()
                        .ok()
                        .map_or(0, |m: std::fs::Metadata| m.len());
                    let depth = entry.depth();
                    (path_str, name_str, is_file, is_dir, is_symlink, size, depth)
                })
                .collect();

            let results: Vec<Value> = entries
                .into_iter()
                .map(
                    |(path_str, name_str, is_file, is_dir, is_symlink, size, depth)| {
                        let mut fields = HashMap::new();
                        fields.insert("path".to_string(), Value::String(path_str.into()));
                        fields.insert("name".to_string(), Value::String(name_str.into()));
                        fields.insert("is_file".to_string(), Value::Bool(is_file));
                        fields.insert("is_dir".to_string(), Value::Bool(is_dir));
                        fields.insert("is_symlink".to_string(), Value::Bool(is_symlink));
                        fields.insert("size".to_string(), Value::Integer(size as i64));
                        fields.insert("depth".to_string(), Value::Integer(depth as i64));
                        Value::Object(Arc::new(fields))
                    },
                )
                .collect();

            Ok(Value::Array(results.into()))
        }
        _ => Err(InterpreterError::RuntimeError(
            "walk_parallel() expects a string path argument".to_string(),
        )),
    }
}

/// Evaluate `compute_hash()` builtin function (STDLIB-005)
/// Computes MD5 hash of a file for duplicate detection
/// Complexity: 3 (within Toyota Way limit of <=10)
pub(crate) fn eval_compute_hash(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("compute_hash", args, 1)?;

    match &args[0] {
        Value::String(path) => {
            let content = std::fs::read(path.as_ref()).map_err(|e| {
                InterpreterError::RuntimeError(format!("Failed to read file '{path}': {e}"))
            })?;
            let digest = md5::compute(&content);
            let hash_string = format!("{digest:x}");
            Ok(Value::String(hash_string.into()))
        }
        _ => Err(InterpreterError::RuntimeError(
            "compute_hash() expects a string path argument".to_string(),
        )),
    }
}

/// Evaluate `fs_copy()` builtin function
/// Complexity: 3 (within Toyota Way limits)
pub(crate) fn eval_fs_copy(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_copy", args, 2)?;

    match (&args[0], &args[1]) {
        (Value::String(from), Value::String(to)) => match std::fs::copy(from.as_ref(), to.as_ref())
        {
            Ok(_) => Ok(Value::Nil),
            Err(e) => Err(InterpreterError::RuntimeError(format!(
                "Failed to copy file: {e}"
            ))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_copy() expects two string arguments".to_string(),
        )),
    }
}

/// Evaluate `fs_rename()` builtin function
/// Complexity: 3 (within Toyota Way limits)
pub(crate) fn eval_fs_rename(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_rename", args, 2)?;

    match (&args[0], &args[1]) {
        (Value::String(from), Value::String(to)) => {
            match std::fs::rename(from.as_ref(), to.as_ref()) {
                Ok(()) => Ok(Value::Nil),
                Err(e) => Err(InterpreterError::RuntimeError(format!(
                    "Failed to rename file: {e}"
                ))),
            }
        }
        _ => Err(InterpreterError::RuntimeError(
            "fs_rename() expects two string arguments".to_string(),
        )),
    }
}

/// Evaluate `fs_metadata()` builtin function
/// Complexity: 3 (within Toyota Way limits)
pub(crate) fn eval_fs_metadata(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_metadata", args, 1)?;

    match &args[0] {
        Value::String(path) => match std::fs::metadata(path.as_ref()) {
            Ok(meta) => {
                let mut map = HashMap::new();
                map.insert("size".to_string(), Value::Integer(meta.len() as i64));
                map.insert("is_dir".to_string(), Value::Bool(meta.is_dir()));
                map.insert("is_file".to_string(), Value::Bool(meta.is_file()));
                Ok(Value::Object(Arc::new(map)))
            }
            Err(e) => Err(InterpreterError::RuntimeError(format!(
                "Failed to get metadata: {e}"
            ))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_metadata() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate `fs_read_dir()` builtin function
/// Complexity: 3 (within Toyota Way limits)
pub(crate) fn eval_fs_read_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_read_dir", args, 1)?;

    match &args[0] {
        Value::String(path) => match std::fs::read_dir(path.as_ref()) {
            Ok(entries) => {
                let paths: Vec<Value> = entries
                    .filter_map(std::result::Result::ok)
                    .map(|e| Value::from_string(e.path().display().to_string()))
                    .collect();
                Ok(Value::Array(paths.into()))
            }
            Err(e) => Err(InterpreterError::RuntimeError(format!(
                "Failed to read directory: {e}"
            ))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_read_dir() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate `fs_canonicalize()` builtin function
/// Complexity: 3 (within Toyota Way limits)
pub(crate) fn eval_fs_canonicalize(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_canonicalize", args, 1)?;

    match &args[0] {
        Value::String(path) => match std::fs::canonicalize(path.as_ref()) {
            Ok(canonical) => Ok(Value::from_string(canonical.display().to_string())),
            Err(e) => Err(InterpreterError::RuntimeError(format!(
                "Failed to canonicalize path: {e}"
            ))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_canonicalize() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate `fs_is_file()` builtin function
/// Complexity: 2 (within Toyota Way limits)
pub(crate) fn eval_fs_is_file(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_is_file", args, 1)?;

    match &args[0] {
        Value::String(path) => Ok(Value::Bool(std::path::Path::new(path.as_ref()).is_file())),
        _ => Err(InterpreterError::RuntimeError(
            "fs_is_file() expects a string argument".to_string(),
        )),
    }
}

/// Dispatch file system functions - Part 1
pub(crate) fn try_eval_fs_part1(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_fs_read__" => Ok(Some(eval_fs_read(args)?)),
        "__builtin_fs_write__" => Ok(Some(eval_fs_write(args)?)),
        "__builtin_fs_exists__" => Ok(Some(eval_fs_exists(args)?)),
        "__builtin_fs_create_dir__" => Ok(Some(eval_fs_create_dir(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch file system functions - Part 2
pub(crate) fn try_eval_fs_part2(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_fs_remove_file__" => Ok(Some(eval_fs_remove_file(args)?)),
        "__builtin_fs_remove_dir__" => Ok(Some(eval_fs_remove_dir(args)?)),
        "__builtin_fs_copy__" => Ok(Some(eval_fs_copy(args)?)),
        "__builtin_fs_rename__" => Ok(Some(eval_fs_rename(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch file system functions - Part 3
pub(crate) fn try_eval_fs_part3(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_fs_metadata__" => Ok(Some(eval_fs_metadata(args)?)),
        "__builtin_fs_read_dir__" => Ok(Some(eval_fs_read_dir(args)?)),
        "__builtin_fs_canonicalize__" => Ok(Some(eval_fs_canonicalize(args)?)),
        "__builtin_fs_is_file__" => Ok(Some(eval_fs_is_file(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch STDLIB-003: User-friendly file I/O aliases
pub(crate) fn try_eval_stdlib003(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_read_file__" | "read_file" => Ok(Some(eval_read_file_unwrapped(args)?)),
        "__builtin_write_file__" | "write_file" => Ok(Some(eval_fs_write(args)?)),
        "__builtin_file_exists__" | "file_exists" => Ok(Some(eval_fs_exists(args)?)),
        "__builtin_delete_file__" | "delete_file" => Ok(Some(eval_fs_remove_file(args)?)),
        "__builtin_append_file__" | "append_file" => Ok(Some(eval_append_file(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch STDLIB-005: Multi-Threaded Directory Walking + Text Search + Hashing
pub(crate) fn try_eval_stdlib005(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_walk__" => Ok(Some(eval_walk(args)?)),
        "__builtin_glob__" => Ok(Some(eval_glob(args)?)),
        "__builtin_search__" => Ok(Some(eval_search(args)?)),
        "__builtin_walk_with_options__" => Ok(Some(eval_walk_with_options(args)?)),
        "__builtin_walk_parallel__" => Ok(Some(eval_walk_parallel(args)?)),
        "__builtin_compute_hash__" => Ok(Some(eval_compute_hash(args)?)),
        _ => Ok(None),
    }
}

/// Dispatcher for file system functions
pub(crate) fn try_eval_fs_function(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    if let Some(result) = try_eval_fs_part1(name, args)? {
        return Ok(Some(result));
    }
    if let Some(result) = try_eval_fs_part2(name, args)? {
        return Ok(Some(result));
    }
    if let Some(result) = try_eval_fs_part3(name, args)? {
        return Ok(Some(result));
    }
    try_eval_stdlib003(name, args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // ============================================================================
    // Coverage tests for search_file_for_matches (21 uncov lines, 0% coverage)
    // ============================================================================

    #[test]
    fn test_search_file_for_matches_basic() {
        let dir = std::env::temp_dir();
        let path = dir.join("ruchy_test_search_basic.txt");
        {
            let mut f = std::fs::File::create(&path).expect("create temp file");
            writeln!(f, "hello world").expect("write line 1");
            writeln!(f, "foo bar").expect("write line 2");
            writeln!(f, "hello again").expect("write line 3");
        }

        let re = regex::Regex::new("hello").expect("valid regex");
        let mut results = Vec::new();
        search_file_for_matches(&path, &re, &mut results);

        assert_eq!(results.len(), 2);

        // Check first match
        if let Value::Object(obj) = &results[0] {
            assert_eq!(
                obj.get("line_num"),
                Some(&Value::Integer(1))
            );
            assert_eq!(
                obj.get("line"),
                Some(&Value::String("hello world".to_string().into()))
            );
            // Path should be present
            assert!(obj.get("path").is_some());
        } else {
            panic!("Expected Object result");
        }

        // Check second match
        if let Value::Object(obj) = &results[1] {
            assert_eq!(
                obj.get("line_num"),
                Some(&Value::Integer(3))
            );
        } else {
            panic!("Expected Object result");
        }

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_search_file_for_matches_no_matches() {
        let dir = std::env::temp_dir();
        let path = dir.join("ruchy_test_search_no_match.txt");
        {
            let mut f = std::fs::File::create(&path).expect("create temp file");
            writeln!(f, "abc").expect("write");
            writeln!(f, "def").expect("write");
        }

        let re = regex::Regex::new("xyz").expect("valid regex");
        let mut results = Vec::new();
        search_file_for_matches(&path, &re, &mut results);

        assert!(results.is_empty());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_search_file_for_matches_nonexistent_file() {
        let path = std::path::Path::new("/tmp/ruchy_nonexistent_file_for_test_12345.txt");
        let re = regex::Regex::new("hello").expect("valid regex");
        let mut results = Vec::new();
        search_file_for_matches(path, &re, &mut results);

        // Should silently return with no results
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_file_for_matches_regex_pattern() {
        let dir = std::env::temp_dir();
        let path = dir.join("ruchy_test_search_regex.txt");
        {
            let mut f = std::fs::File::create(&path).expect("create temp file");
            writeln!(f, "error: something went wrong").expect("write");
            writeln!(f, "warning: be careful").expect("write");
            writeln!(f, "error: another issue").expect("write");
            writeln!(f, "info: all good").expect("write");
        }

        let re = regex::Regex::new(r"^error:").expect("valid regex");
        let mut results = Vec::new();
        search_file_for_matches(&path, &re, &mut results);

        assert_eq!(results.len(), 2);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_search_file_for_matches_all_lines_match() {
        let dir = std::env::temp_dir();
        let path = dir.join("ruchy_test_search_all.txt");
        {
            let mut f = std::fs::File::create(&path).expect("create temp file");
            writeln!(f, "test line 1").expect("write");
            writeln!(f, "test line 2").expect("write");
            writeln!(f, "test line 3").expect("write");
        }

        let re = regex::Regex::new("test").expect("valid regex");
        let mut results = Vec::new();
        search_file_for_matches(&path, &re, &mut results);

        assert_eq!(results.len(), 3);

        // Verify line numbers are 1-indexed
        for (i, result) in results.iter().enumerate() {
            if let Value::Object(obj) = result {
                assert_eq!(
                    obj.get("line_num"),
                    Some(&Value::Integer((i + 1) as i64))
                );
            }
        }

        let _ = std::fs::remove_file(&path);
    }
}
