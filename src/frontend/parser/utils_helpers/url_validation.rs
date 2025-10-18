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
