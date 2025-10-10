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
/// let result = path::join("/home/user", "documents").unwrap();
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
/// let result = path::join_many(&["/home", "user", "documents"]).unwrap();
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
/// let result = path::parent("/home/user/file.txt").unwrap();
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
/// let result = path::file_name("/home/user/file.txt").unwrap();
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
/// let result = path::file_stem("/home/user/file.txt").unwrap();
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
/// let result = path::extension("/home/user/file.txt").unwrap();
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
/// let result = path::with_extension("/home/user/file.txt", "md").unwrap();
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
/// let result = path::with_file_name("/home/user/old.txt", "new.txt").unwrap();
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
/// let components = path::components("/home/user/file.txt").unwrap();
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
/// let result = path::normalize("/home/user/../admin/./file.txt").unwrap();
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

    #[test]
    fn test_join_basic() {
        let result = join("/home", "user").unwrap();
        assert!(result.contains("user"));
    }

    #[test]
    fn test_is_absolute_true() {
        assert_eq!(is_absolute("/home/user"), true);
    }

    #[test]
    fn test_is_absolute_false() {
        assert_eq!(is_absolute("relative/path"), false);
    }
}
