//! URL validation helpers for import statements

use super::super::{bail, Result};

/// Validate URL imports for safe operation
pub(in crate::frontend::parser) fn validate_url_import(url: &str) -> Result<()> {
    validate_url_scheme(url)?;
    validate_url_extension(url)?;
    validate_url_path_safety(url)?;
    validate_url_no_suspicious_patterns(url)?;
    Ok(())
}

/// Validate URL uses HTTPS (except for localhost)
/// Extracted to reduce complexity
pub fn validate_url_scheme(url: &str) -> Result<()> {
    if is_valid_url_scheme(url) {
        Ok(())
    } else {
        bail!("URL imports must use HTTPS for security (except for localhost). Got: {url}")
    }
}

/// Check if URL has valid scheme
pub fn is_valid_url_scheme(url: &str) -> bool {
    url.starts_with("https://")
        || url.starts_with("http://localhost")
        || url.starts_with("http://127.0.0.1")
}

/// Validate URL has correct file extension
pub fn validate_url_extension(url: &str) -> Result<()> {
    if url.ends_with(".ruchy") || url.ends_with(".rchy") {
        Ok(())
    } else {
        bail!("URL imports must reference .ruchy or .rchy files. Got: {url}")
    }
}

/// Validate URL doesn't contain path traversal
pub fn validate_url_path_safety(url: &str) -> Result<()> {
    if url.contains("..") || url.contains("/.") {
        bail!("URL imports cannot contain path traversal sequences (.. or /.): {url}")
    }
    Ok(())
}

/// Validate URL doesn't contain suspicious patterns
pub fn validate_url_no_suspicious_patterns(url: &str) -> Result<()> {
    const SUSPICIOUS_PATTERNS: &[&str] = &["javascript:", "data:", "file:"];
    for pattern in SUSPICIOUS_PATTERNS {
        if url.contains(pattern) {
            bail!("Invalid URL scheme for import");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_import_valid_https() {
        assert!(validate_url_import("https://example.com/module.ruchy").is_ok());
        assert!(validate_url_import("https://cdn.example.com/lib.rchy").is_ok());
    }

    #[test]
    fn test_validate_url_import_valid_localhost() {
        assert!(validate_url_import("http://localhost/test.ruchy").is_ok());
        assert!(validate_url_import("http://127.0.0.1/module.ruchy").is_ok());
    }

    #[test]
    fn test_validate_url_scheme_https() {
        assert!(validate_url_scheme("https://example.com/test.ruchy").is_ok());
    }

    #[test]
    fn test_validate_url_scheme_http_rejected() {
        assert!(validate_url_scheme("http://example.com/test.ruchy").is_err());
    }

    #[test]
    fn test_validate_url_scheme_localhost() {
        assert!(validate_url_scheme("http://localhost/test.ruchy").is_ok());
        assert!(validate_url_scheme("http://127.0.0.1/test.ruchy").is_ok());
    }

    #[test]
    fn test_is_valid_url_scheme() {
        assert!(is_valid_url_scheme("https://example.com"));
        assert!(is_valid_url_scheme("http://localhost"));
        assert!(is_valid_url_scheme("http://127.0.0.1"));
        assert!(!is_valid_url_scheme("http://evil.com"));
        assert!(!is_valid_url_scheme("ftp://example.com"));
    }

    #[test]
    fn test_validate_url_extension_valid() {
        assert!(validate_url_extension("https://example.com/mod.ruchy").is_ok());
        assert!(validate_url_extension("https://example.com/mod.rchy").is_ok());
    }

    #[test]
    fn test_validate_url_extension_invalid() {
        assert!(validate_url_extension("https://example.com/mod.js").is_err());
        assert!(validate_url_extension("https://example.com/mod.py").is_err());
        assert!(validate_url_extension("https://example.com/mod").is_err());
    }

    #[test]
    fn test_validate_url_path_safety_valid() {
        assert!(validate_url_path_safety("https://example.com/modules/test.ruchy").is_ok());
    }

    #[test]
    fn test_validate_url_path_safety_traversal() {
        assert!(validate_url_path_safety("https://example.com/../etc/passwd.ruchy").is_err());
        assert!(validate_url_path_safety("https://example.com/.hidden/test.ruchy").is_err());
    }

    #[test]
    fn test_validate_url_no_suspicious_patterns_valid() {
        assert!(validate_url_no_suspicious_patterns("https://example.com/test.ruchy").is_ok());
    }

    #[test]
    fn test_validate_url_no_suspicious_patterns_javascript() {
        assert!(validate_url_no_suspicious_patterns("javascript:alert(1)").is_err());
    }

    #[test]
    fn test_validate_url_no_suspicious_patterns_data() {
        assert!(validate_url_no_suspicious_patterns("data:text/html,<script>").is_err());
    }

    #[test]
    fn test_validate_url_no_suspicious_patterns_file() {
        assert!(validate_url_no_suspicious_patterns("file:///etc/passwd").is_err());
    }
}
