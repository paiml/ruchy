//! Multi-Format Output Support
//!
//! Supports JSON, SARIF 2.1.0, and Markdown output formats [10].

mod human;
mod json;
mod markdown;
mod sarif;

pub use human::HumanFormatter;
pub use json::JsonFormatter;
pub use markdown::MarkdownFormatter;
pub use sarif::SarifFormatter;

use crate::reporting::TranspileReport;

/// Output format selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Human-readable terminal output with colors and Unicode
    #[default]
    Human,
    /// Machine-readable JSON
    Json,
    /// SARIF 2.1.0 for IDE integration (VS Code, `IntelliJ`)
    Sarif,
    /// `CommonMark` Markdown for documentation
    Markdown,
}

impl OutputFormat {
    /// Parse from string
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "human" | "text" | "terminal" => Some(Self::Human),
            "json" => Some(Self::Json),
            "sarif" => Some(Self::Sarif),
            "md" | "markdown" => Some(Self::Markdown),
            _ => None,
        }
    }

    /// Get file extension for this format
    #[must_use]
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Human => "txt",
            Self::Json => "json",
            Self::Sarif => "sarif",
            Self::Markdown => "md",
        }
    }
}

/// Report formatter trait
pub trait ReportFormatter {
    /// Format the report as string
    fn format(&self, report: &TranspileReport) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(OutputFormat::from_str("human"), Some(OutputFormat::Human));
        assert_eq!(OutputFormat::from_str("json"), Some(OutputFormat::Json));
        assert_eq!(OutputFormat::from_str("sarif"), Some(OutputFormat::Sarif));
        assert_eq!(
            OutputFormat::from_str("markdown"),
            Some(OutputFormat::Markdown)
        );
        assert_eq!(OutputFormat::from_str("md"), Some(OutputFormat::Markdown));
        assert_eq!(OutputFormat::from_str("text"), Some(OutputFormat::Human));
        assert_eq!(OutputFormat::from_str("unknown"), None);
    }

    #[test]
    fn test_output_format_extension() {
        assert_eq!(OutputFormat::Human.extension(), "txt");
        assert_eq!(OutputFormat::Json.extension(), "json");
        assert_eq!(OutputFormat::Sarif.extension(), "sarif");
        assert_eq!(OutputFormat::Markdown.extension(), "md");
    }

    #[test]
    fn test_output_format_default() {
        assert_eq!(OutputFormat::default(), OutputFormat::Human);
    }

    // === EXTREME TDD Round 20 tests ===

    #[test]
    fn test_output_format_debug() {
        let format = OutputFormat::Json;
        let debug_str = format!("{:?}", format);
        assert_eq!(debug_str, "Json");
    }

    #[test]
    fn test_output_format_clone() {
        let format1 = OutputFormat::Sarif;
        let format2 = format1;
        assert_eq!(format1, format2);
    }

    #[test]
    fn test_output_format_from_str_case_insensitive() {
        assert_eq!(OutputFormat::from_str("JSON"), Some(OutputFormat::Json));
        assert_eq!(OutputFormat::from_str("SARIF"), Some(OutputFormat::Sarif));
        assert_eq!(OutputFormat::from_str("HUMAN"), Some(OutputFormat::Human));
        assert_eq!(OutputFormat::from_str("MD"), Some(OutputFormat::Markdown));
    }

    #[test]
    fn test_output_format_terminal_alias() {
        assert_eq!(
            OutputFormat::from_str("terminal"),
            Some(OutputFormat::Human)
        );
    }
}
