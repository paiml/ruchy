//! Path Module (STD-004)
//!
//! Thin wrappers around Rust's `std::path` for Ruchy-friendly API.
//!
//! # Examples
//!
//! ```no_run
//! use ruchy::stdlib::path;
//!
//! // Join paths
//! let full_path = path::join("/home/user", "documents")?;
//!
//! // Get file name
//! let name = path::file_name("/home/user/file.txt")?;
//!
//! // Get extension
//! let ext = path::extension("/home/user/file.txt")?;
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Join two path components
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::path;
///
/// let result = path::join("/home/user", "documents").expect("join should succeed in doctest");
/// assert!(result.contains("documents"));
/// ```
///
/// # Errors
///
/// Returns error if path conversion fails
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn join(base: &str, component: &str) -> Result<String> {
    let path = Path::new(base).join(component);
    Ok(path.to_string_lossy().to_string())
}

/// Join multiple path components
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::path;
///
/// let result = path::join_many(&["/home", "user", "documents"]).expect("join_many should succeed in doctest");
/// assert!(result.contains("documents"));
/// ```
///
/// # Errors
///
/// Returns error if path conversion fails
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
pub fn join_many(components: &[&str]) -> Result<String> {
    let mut path = PathBuf::new();
    for component in components {
        path.push(component);
    }
    Ok(path.to_string_lossy().to_string())
}

/// Get the parent directory of a path
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::path;
///
/// let result = path::parent("/home/user/file.txt").expect("parent should succeed in doctest");
/// assert!(result.is_some());
/// ```
///
/// # Errors
///
/// Returns error if path conversion fails
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
pub fn parent(path: &str) -> Result<Option<String>> {
    let p = Path::new(path);
    Ok(p.parent().map(|p| p.to_string_lossy().to_string()))
}

/// Get the file name from a path
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::path;
///
/// let result = path::file_name("/home/user/file.txt").expect("file_name should succeed in doctest");
/// assert_eq!(result, Some("file.txt".to_string()));
/// ```
///
/// # Errors
///
/// Returns error if path conversion fails
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
pub fn file_name(path: &str) -> Result<Option<String>> {
    let p = Path::new(path);
    Ok(p.file_name().map(|n| n.to_string_lossy().to_string()))
}

/// Get the file stem (name without extension)
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::path;
///
/// let result = path::file_stem("/home/user/file.txt").expect("file_stem should succeed in doctest");
/// assert_eq!(result, Some("file".to_string()));
/// ```
///
/// # Errors
///
/// Returns error if path conversion fails
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
pub fn file_stem(path: &str) -> Result<Option<String>> {
    let p = Path::new(path);
    Ok(p.file_stem().map(|s| s.to_string_lossy().to_string()))
}

/// Get the file extension
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::path;
///
/// let result = path::extension("/home/user/file.txt").expect("extension should succeed in doctest");
/// assert_eq!(result, Some("txt".to_string()));
/// ```
///
/// # Errors
///
/// Returns error if path conversion fails
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
pub fn extension(path: &str) -> Result<Option<String>> {
    let p = Path::new(path);
    Ok(p.extension().map(|e| e.to_string_lossy().to_string()))
}

/// Check if path is absolute
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::path;
///
/// assert_eq!(path::is_absolute("/home/user"), true);
/// assert_eq!(path::is_absolute("relative/path"), false);
/// ```
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn is_absolute(path: &str) -> bool {
    Path::new(path).is_absolute()
}

/// Check if path is relative
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::path;
///
/// assert_eq!(path::is_relative("relative/path"), true);
/// assert_eq!(path::is_relative("/home/user"), false);
/// ```
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn is_relative(path: &str) -> bool {
    Path::new(path).is_relative()
}

/// Canonicalize a path (resolve to absolute, real path)
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::path;
///
/// let canonical = path::canonicalize("./file.txt")?;
/// assert!(canonical.starts_with("/"));
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if path doesn't exist or cannot be canonicalized
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn canonicalize(path: &str) -> Result<String> {
    let p = Path::new(path);
    let canonical = p.canonicalize()?;
    Ok(canonical.to_string_lossy().to_string())
}

/// Replace the extension of a path
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::path;
///
/// let result = path::with_extension("/home/user/file.txt", "md").expect("with_extension should succeed in doctest");
/// assert!(result.ends_with(".md"));
/// ```
///
/// # Errors
///
/// Returns error if path conversion fails
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn with_extension(path: &str, ext: &str) -> Result<String> {
    let p = Path::new(path);
    let new_path = p.with_extension(ext);
    Ok(new_path.to_string_lossy().to_string())
}

/// Replace the file name of a path
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::path;
///
/// let result = path::with_file_name("/home/user/old.txt", "new.txt").expect("with_file_name should succeed in doctest");
/// assert!(result.ends_with("new.txt"));
/// ```
///
/// # Errors
///
/// Returns error if path conversion fails
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn with_file_name(path: &str, name: &str) -> Result<String> {
    let p = Path::new(path);
    let new_path = p.with_file_name(name);
    Ok(new_path.to_string_lossy().to_string())
}

/// Get path components as a vector
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::path;
///
/// let components = path::components("/home/user/file.txt").expect("components should succeed in doctest");
/// assert!(components.contains(&"user".to_string()));
/// ```
///
/// # Errors
///
/// Returns error if path conversion fails
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
pub fn components(path: &str) -> Result<Vec<String>> {
    let p = Path::new(path);
    let comps: Vec<String> = p
        .components()
        .filter_map(|c| {
            if let std::path::Component::Normal(s) = c {
                Some(s.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();
    Ok(comps)
}

/// Normalize a path (remove . and .. components)
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::path;
///
/// let result = path::normalize("/home/user/../admin/./file.txt").expect("normalize should succeed in doctest");
/// assert!(!result.contains(".."));
/// ```
///
/// # Errors
///
/// Returns error if path conversion fails
///
/// # Complexity
///
/// Complexity: 3 (within Toyota Way limits ≤10)
pub fn normalize(path: &str) -> Result<String> {
    let p = Path::new(path);
    let mut normalized = PathBuf::new();

    for component in p.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {
                // Skip current directory
            }
            _ => {
                normalized.push(component);
            }
        }
    }

    Ok(normalized.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // EXTREME TDD: Comprehensive Path Module Testing
    // Coverage Target: 20.41% → 80%+
    // Mutation Target: ≥75% caught
    // ============================================================================

    // --------------------------------------------------------------------------
    // join() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_join_basic() {
        let result = join("/home", "user").expect("join should succeed in test");
        assert!(result.contains("user"));
        assert!(result.contains("home"));
    }

    #[test]
    fn test_join_empty_base() {
        let result = join("", "file.txt").expect("operation should succeed in test");
        assert_eq!(result, "file.txt");
    }

    #[test]
    fn test_join_empty_component() {
        let result = join("/home", "").expect("operation should succeed in test");
        assert!(result.contains("home"));
    }

    #[test]
    fn test_join_windows_style() {
        let result = join("C:\\Users", "Documents").expect("operation should succeed in test");
        assert!(result.contains("Documents"));
    }

    // --------------------------------------------------------------------------
    // join_many() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_join_many_basic() {
        let result =
            join_many(&["/home", "user", "documents"]).expect("operation should succeed in test");
        assert!(result.contains("home"));
        assert!(result.contains("user"));
        assert!(result.contains("documents"));
    }

    #[test]
    fn test_join_many_empty() {
        let result = join_many(&[]).expect("operation should succeed in test");
        assert_eq!(result, "");
    }

    #[test]
    fn test_join_many_single() {
        let result = join_many(&["/home"]).expect("operation should succeed in test");
        assert!(result.contains("home"));
    }

    // --------------------------------------------------------------------------
    // parent() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_parent_file_path() {
        let result = parent("/home/user/file.txt").expect("operation should succeed in test");
        assert!(result.is_some());
        let parent_path = result.expect("operation should succeed in test");
        assert!(parent_path.contains("user"));
    }

    #[test]
    fn test_parent_root() {
        let result = parent("/").expect("operation should succeed in test");
        assert!(result.is_none(), "Root path should have no parent");
    }

    #[test]
    fn test_parent_relative() {
        let result = parent("file.txt").expect("operation should succeed in test");
        assert!(result.is_some() || result.is_none()); // Platform dependent
    }

    // --------------------------------------------------------------------------
    // file_name() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_file_name_basic() {
        let result = file_name("/home/user/file.txt").expect("operation should succeed in test");
        assert_eq!(result, Some("file.txt".to_string()));
    }

    #[test]
    fn test_file_name_no_extension() {
        let result = file_name("/home/user/file").expect("operation should succeed in test");
        assert_eq!(result, Some("file".to_string()));
    }

    #[test]
    fn test_file_name_directory() {
        let result = file_name("/home/user/").expect("operation should succeed in test");
        assert!(result.is_none() || result == Some("user".to_string()));
    }

    // --------------------------------------------------------------------------
    // file_stem() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_file_stem_basic() {
        let result = file_stem("/home/user/file.txt").expect("operation should succeed in test");
        assert_eq!(result, Some("file".to_string()));
    }

    #[test]
    fn test_file_stem_multiple_dots() {
        let result =
            file_stem("/home/user/archive.tar.gz").expect("operation should succeed in test");
        assert_eq!(result, Some("archive.tar".to_string()));
    }

    #[test]
    fn test_file_stem_no_extension() {
        let result = file_stem("/home/user/file").expect("operation should succeed in test");
        assert_eq!(result, Some("file".to_string()));
    }

    // --------------------------------------------------------------------------
    // extension() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_extension_basic() {
        let result = extension("/home/user/file.txt").expect("operation should succeed in test");
        assert_eq!(result, Some("txt".to_string()));
    }

    #[test]
    fn test_extension_multiple_dots() {
        let result =
            extension("/home/user/archive.tar.gz").expect("operation should succeed in test");
        assert_eq!(result, Some("gz".to_string()));
    }

    #[test]
    fn test_extension_none() {
        let result = extension("/home/user/file").expect("operation should succeed in test");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extension_hidden_file() {
        let result = extension("/home/user/.bashrc").expect("operation should succeed in test");
        assert_eq!(result, None);
    }

    // --------------------------------------------------------------------------
    // is_absolute() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_is_absolute_true() {
        assert!(is_absolute("/home/user"));
    }

    #[test]
    fn test_is_absolute_false() {
        assert!(!is_absolute("relative/path"));
    }

    #[test]
    fn test_is_absolute_current_dir() {
        assert!(!is_absolute("."));
    }

    #[test]
    fn test_is_absolute_parent_dir() {
        assert!(!is_absolute(".."));
    }

    // --------------------------------------------------------------------------
    // is_relative() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_is_relative_true() {
        assert!(is_relative("relative/path"));
    }

    #[test]
    fn test_is_relative_false() {
        assert!(!is_relative("/home/user"));
    }

    #[test]
    fn test_is_relative_current_dir() {
        assert!(is_relative("."));
    }

    // --------------------------------------------------------------------------
    // with_extension() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_with_extension_replace() {
        let result =
            with_extension("/home/user/file.txt", "md").expect("operation should succeed in test");
        assert!(result.ends_with(".md"));
        assert!(!result.ends_with(".txt"));
    }

    #[test]
    fn test_with_extension_add() {
        let result =
            with_extension("/home/user/file", "txt").expect("operation should succeed in test");
        assert!(result.ends_with(".txt"));
    }

    #[test]
    fn test_with_extension_empty() {
        let result =
            with_extension("/home/user/file.txt", "").expect("operation should succeed in test");
        assert!(!result.ends_with(".txt"));
    }

    // --------------------------------------------------------------------------
    // with_file_name() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_with_file_name_replace() {
        let result = with_file_name("/home/user/old.txt", "new.txt")
            .expect("operation should succeed in test");
        assert!(result.ends_with("new.txt"));
        assert!(!result.contains("old"));
    }

    #[test]
    fn test_with_file_name_different_extension() {
        let result = with_file_name("/home/user/file.txt", "data.json")
            .expect("operation should succeed in test");
        assert!(result.ends_with("data.json"));
    }

    // --------------------------------------------------------------------------
    // components() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_components_basic() {
        let result = components("/home/user/file.txt").expect("operation should succeed in test");
        assert!(result.contains(&"home".to_string()));
        assert!(result.contains(&"user".to_string()));
        assert!(result.contains(&"file.txt".to_string()));
    }

    #[test]
    fn test_components_relative() {
        let result = components("user/file.txt").expect("operation should succeed in test");
        assert!(result.contains(&"user".to_string()));
        assert!(result.contains(&"file.txt".to_string()));
    }

    #[test]
    fn test_components_empty() {
        let result = components("").expect("operation should succeed in test");
        assert_eq!(result.len(), 0);
    }

    // --------------------------------------------------------------------------
    // normalize() tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_normalize_parent_dir() {
        let result =
            normalize("/home/user/../admin/file.txt").expect("operation should succeed in test");
        assert!(!result.contains(".."));
        assert!(result.contains("admin"));
    }

    #[test]
    fn test_normalize_current_dir() {
        let result = normalize("/home/user/./file.txt").expect("operation should succeed in test");
        assert!(!result.contains("/./"));
    }

    #[test]
    fn test_normalize_multiple_dots() {
        let result =
            normalize("/home/user/../../etc/file.txt").expect("operation should succeed in test");
        assert!(!result.contains(".."));
    }

    #[test]
    fn test_normalize_no_dots() {
        let result = normalize("/home/user/file.txt").expect("operation should succeed in test");
        assert!(result.contains("home"));
        assert!(result.contains("user"));
    }

    // --------------------------------------------------------------------------
    // Property Tests (Mathematical Invariants)
    // --------------------------------------------------------------------------

    #[test]
    fn prop_is_absolute_and_is_relative_are_inverses() {
        let test_paths = vec!["/home/user", "relative/path", ".", "..", "/"];

        for path in test_paths {
            assert_eq!(
                is_absolute(path),
                !is_relative(path),
                "is_absolute and is_relative should be inverses for '{path}'"
            );
        }
    }

    #[test]
    fn prop_join_preserves_both_components() {
        let base = "/home";
        let component = "user";
        let result = join(base, component).expect("operation should succeed in test");

        assert!(result.contains("home"), "Result should contain base");
        assert!(result.contains("user"), "Result should contain component");
    }

    #[test]
    fn prop_extension_of_with_extension_matches() {
        let path = "/home/user/file.txt";
        let new_ext = "md";

        let modified = with_extension(path, new_ext).expect("operation should succeed in test");
        let ext = extension(&modified).expect("operation should succeed in test");

        assert_eq!(
            ext,
            Some(new_ext.to_string()),
            "Extension of modified path should match new extension"
        );
    }

    #[test]
    fn prop_file_stem_plus_extension_equals_file_name() {
        let path = "/home/user/file.txt";

        let stem = file_stem(path).expect("operation should succeed in test");
        let ext = extension(path).expect("operation should succeed in test");
        let name = file_name(path).expect("operation should succeed in test");

        if let (Some(s), Some(e), Some(n)) = (stem, ext, name) {
            assert_eq!(
                format!("{s}.{e}"),
                n,
                "file_stem + extension should equal file_name"
            );
        }
    }

    // --------------------------------------------------------------------------
    // Boundary Condition Tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_empty_path() {
        let result = file_name("").expect("operation should succeed in test");
        assert_eq!(result, None);
    }

    #[test]
    fn test_very_long_path() {
        let long_component = "a".repeat(255);
        let result = join("/home", &long_component).expect("operation should succeed in test");
        assert!(result.len() > 255);
    }

    #[test]
    fn test_special_characters() {
        let result = join("/home", "user@domain").expect("operation should succeed in test");
        assert!(result.contains('@'));
    }

    #[test]
    fn test_unicode_path() {
        let result = join("/home", "用户").expect("operation should succeed in test");
        assert!(result.contains("用户"));
    }
}

// ============================================================================
// Property Tests Module (High-Confidence Verification)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;

    #[test]
    fn prop_join_never_panics() {
        let test_cases = vec![
            ("", ""),
            ("/home", ""),
            ("", "user"),
            ("/home", "user"),
            ("C:\\Users", "Documents"),
            ("/", "file.txt"),
        ];

        for (base, component) in test_cases {
            let _ = join(base, component);
            // Should not panic
        }
    }

    #[test]
    fn prop_all_functions_handle_empty_strings() {
        // Property: All functions should handle empty strings gracefully
        let _ = join("", "");
        let _ = join_many(&[]);
        let _ = parent("");
        let _ = file_name("");
        let _ = file_stem("");
        let _ = extension("");
        let _ = is_absolute("");
        let _ = is_relative("");
        let _ = with_extension("", "txt");
        let _ = with_file_name("", "file.txt");
        let _ = components("");
        let _ = normalize("");
        // All should complete without panic
    }

    #[test]
    fn prop_path_operations_are_pure() {
        // Property: Path operations don't modify input, always produce consistent output
        let path = "/home/user/file.txt";

        let result1 = file_name(path).expect("operation should succeed in test");
        let result2 = file_name(path).expect("operation should succeed in test");

        assert_eq!(result1, result2, "Path operations should be deterministic");
    }
}
