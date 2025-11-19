//! Formatter configuration system
//!
//! This module provides configuration options for the Ruchy code formatter,
//! allowing users to customize formatting behavior via .ruchy-fmt.toml

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration for the Ruchy formatter
///
/// # Examples
///
/// ```
/// use ruchy::quality::FormatterConfig;
///
/// let config = FormatterConfig::default();
/// assert_eq!(config.indent_width, 4);
/// assert!(!config.use_tabs);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatterConfig {
    /// Number of spaces per indentation level (default: 4)
    pub indent_width: usize,

    /// Use tabs instead of spaces for indentation (default: false)
    pub use_tabs: bool,

    /// Maximum line length before wrapping (default: 100)
    pub max_line_length: usize,

    /// Preserve existing newlines between items (default: true)
    pub preserve_newlines: bool,

    /// Add trailing commas in multi-line lists (default: true)
    pub trailing_commas: bool,

    /// Space after colon in type annotations (default: true)
    pub space_after_colon: bool,

    /// Space before opening brace (default: true)
    pub space_before_brace: bool,

    /// Format strings (normalize quotes, escapes) (default: false)
    pub format_strings: bool,

    /// Format comments (normalize spacing) (default: false)
    pub format_comments: bool,

    /// Ignore files matching these patterns (default: empty)
    pub ignore_patterns: Vec<String>,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            indent_width: 4,
            use_tabs: false,
            max_line_length: 100,
            preserve_newlines: true,
            trailing_commas: true,
            space_after_colon: true,
            space_before_brace: true,
            format_strings: false,
            format_comments: false,
            ignore_patterns: vec![],
        }
    }
}

impl FormatterConfig {
    /// Create a new configuration with default values
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::quality::FormatterConfig;
    ///
    /// let config = FormatterConfig::new();
    /// assert_eq!(config.indent_width, 4);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a TOML file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The TOML is invalid
    /// - Required fields are missing
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ruchy::quality::FormatterConfig;
    ///
    /// let config = FormatterConfig::from_file(".ruchy-fmt.toml").unwrap();
    /// println!("Indent width: {}", config.indent_width);
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let contents = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read config file: {e}"))?;

        Self::from_toml(&contents)
    }

    /// Load configuration from TOML string
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::quality::FormatterConfig;
    ///
    /// let toml = r#"
    /// indent_width = 2
    /// use_tabs = false
    /// "#;
    ///
    /// let config = FormatterConfig::from_toml(toml).unwrap();
    /// assert_eq!(config.indent_width, 2);
    /// ```
    pub fn from_toml(toml_str: &str) -> Result<Self, String> {
        toml::from_str(toml_str).map_err(|e| format!("Failed to parse config TOML: {e}"))
    }

    /// Save configuration to a TOML file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ruchy::quality::FormatterConfig;
    ///
    /// let config = FormatterConfig::default();
    /// config.to_file(".ruchy-fmt.toml").unwrap();
    /// ```
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let toml_str = self.to_toml()?;
        std::fs::write(path.as_ref(), toml_str)
            .map_err(|e| format!("Failed to write config file: {e}"))
    }

    /// Convert configuration to TOML string
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::quality::FormatterConfig;
    ///
    /// let config = FormatterConfig::default();
    /// let toml = config.to_toml().unwrap();
    /// assert!(toml.contains("indent_width"));
    /// ```
    pub fn to_toml(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|e| format!("Failed to serialize config: {e}"))
    }

    /// Check if a file path should be ignored based on patterns
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::quality::FormatterConfig;
    ///
    /// let mut config = FormatterConfig::default();
    /// config.ignore_patterns = vec!["**/test/**".to_string()];
    ///
    /// assert!(config.should_ignore("src/test/example.ruchy"));
    /// assert!(!config.should_ignore("src/main.ruchy"));
    /// ```
    pub fn should_ignore(&self, path: &str) -> bool {
        for pattern in &self.ignore_patterns {
            if path.contains(pattern.trim_start_matches("**/").trim_end_matches("/**")) {
                return true;
            }
        }
        false
    }

    /// Merge with another configuration, preferring non-default values
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::quality::FormatterConfig;
    ///
    /// let mut base = FormatterConfig::default();
    /// let mut override_config = FormatterConfig::default();
    /// override_config.indent_width = 2;
    ///
    /// base.merge(override_config);
    /// assert_eq!(base.indent_width, 2);
    /// ```
    pub fn merge(&mut self, other: Self) {
        // Only override if the value differs from default
        let default = Self::default();

        if other.indent_width != default.indent_width {
            self.indent_width = other.indent_width;
        }
        if other.use_tabs != default.use_tabs {
            self.use_tabs = other.use_tabs;
        }
        if other.max_line_length != default.max_line_length {
            self.max_line_length = other.max_line_length;
        }
        if other.preserve_newlines != default.preserve_newlines {
            self.preserve_newlines = other.preserve_newlines;
        }
        if other.trailing_commas != default.trailing_commas {
            self.trailing_commas = other.trailing_commas;
        }
        if other.space_after_colon != default.space_after_colon {
            self.space_after_colon = other.space_after_colon;
        }
        if other.space_before_brace != default.space_before_brace {
            self.space_before_brace = other.space_before_brace;
        }
        if other.format_strings != default.format_strings {
            self.format_strings = other.format_strings;
        }
        if other.format_comments != default.format_comments {
            self.format_comments = other.format_comments;
        }
        if !other.ignore_patterns.is_empty() {
            self.ignore_patterns.extend(other.ignore_patterns);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FormatterConfig::default();
        assert_eq!(config.indent_width, 4);
        assert!(!config.use_tabs);
        assert_eq!(config.max_line_length, 100);
        assert!(config.preserve_newlines);
        assert!(config.trailing_commas);
    }

    #[test]
    fn test_from_toml() {
        let toml = r#"
        indent_width = 2
        use_tabs = true
        max_line_length = 80
        preserve_newlines = false
        trailing_commas = false
        space_after_colon = false
        space_before_brace = false
        format_strings = true
        format_comments = true
        ignore_patterns = ["*.test.rs"]
        "#;

        let config = FormatterConfig::from_toml(toml).unwrap();
        assert_eq!(config.indent_width, 2);
        assert!(config.use_tabs);
        assert_eq!(config.max_line_length, 80);
        assert!(!config.preserve_newlines);
        assert!(config.format_strings);
    }

    #[test]
    fn test_to_toml() {
        let config = FormatterConfig::default();
        let toml = config.to_toml().unwrap();

        assert!(toml.contains("indent_width = 4"));
        assert!(toml.contains("use_tabs = false"));
        assert!(toml.contains("max_line_length = 100"));
    }

    #[test]
    fn test_should_ignore() {
        let mut config = FormatterConfig::default();
        config.ignore_patterns = vec!["**/target/**".to_string(), "**/test/**".to_string()];

        assert!(config.should_ignore("src/target/debug/file.ruchy"));
        assert!(config.should_ignore("src/test/integration.ruchy"));
        assert!(!config.should_ignore("src/main.ruchy"));
    }

    #[test]
    fn test_merge() {
        let mut base = FormatterConfig::default();
        let mut override_config = FormatterConfig::default();
        override_config.indent_width = 2;
        override_config.use_tabs = true;

        base.merge(override_config);

        assert_eq!(base.indent_width, 2);
        assert!(base.use_tabs);
        assert_eq!(base.max_line_length, 100); // unchanged
    }

    #[test]
    fn test_config_round_trip() {
        let original = FormatterConfig {
            indent_width: 2,
            use_tabs: true,
            max_line_length: 120,
            ..Default::default()
        };

        let toml = original.to_toml().unwrap();
        let loaded = FormatterConfig::from_toml(&toml).unwrap();

        assert_eq!(loaded.indent_width, original.indent_width);
        assert_eq!(loaded.use_tabs, original.use_tabs);
        assert_eq!(loaded.max_line_length, original.max_line_length);
    }
}
