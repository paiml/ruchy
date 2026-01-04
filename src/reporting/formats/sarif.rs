//! SARIF 2.1.0 output format for IDE integration [10]
//!
//! Static Analysis Results Interchange Format enables VS Code, `IntelliJ`,
//! and other IDEs to display transpilation errors inline.

use crate::reporting::TranspileReport;

use super::ReportFormatter;

/// SARIF 2.1.0 formatter
#[derive(Default)]
pub struct SarifFormatter;

impl SarifFormatter {
    /// Get the SARIF schema URL
    const SCHEMA: &'static str =
        "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json";

    /// Get the SARIF version
    const VERSION: &'static str = "2.1.0";

    /// Map Rust error code to severity level
    fn error_severity(code: &str) -> &'static str {
        // E0XXX errors are typically errors, warnings are W0XXX
        if code.starts_with('E') {
            "error"
        } else if code.starts_with('W') {
            "warning"
        } else {
            "note"
        }
    }

    /// Map error code to rule description
    fn rule_description(code: &str) -> &'static str {
        match code {
            "E0308" => "mismatched types",
            "E0382" => "borrow of moved value",
            "E0412" => "cannot find type in this scope",
            "E0425" => "cannot find value in this scope",
            "E0282" => "type annotations needed",
            "E0277" => "trait bound not satisfied",
            "E0502" | "E0503" | "E0505" => "borrow checker error",
            "E0106" | "E0621" => "lifetime error",
            _ => "transpilation error",
        }
    }
}

impl ReportFormatter for SarifFormatter {
    fn format(&self, report: &TranspileReport) -> String {
        // Build rules array
        let rules: Vec<String> = report
            .errors
            .iter()
            .map(|e| {
                format!(
                    r#"{{"id":"{}","shortDescription":{{"text":"{}"}},"helpUri":"https://doc.rust-lang.org/error-index.html#{}"}}"#,
                    e.code,
                    Self::rule_description(&e.code),
                    e.code
                )
            })
            .collect();

        // Build results array
        let results: Vec<String> = report
            .errors
            .iter()
            .flat_map(|e| {
                e.samples.iter().map(move |sample| {
                    format!(
                        r#"{{"ruleId":"{}","level":"{}","message":{{"text":"{}"}}}}"#,
                        e.code,
                        Self::error_severity(&e.code),
                        escape_sarif(sample)
                    )
                })
            })
            .collect();

        format!(
            r#"{{"$schema":"{}","version":"{}","runs":[{{"tool":{{"driver":{{"name":"ruchy","version":"1.0.0","informationUri":"https://github.com/ruchy-lang/ruchy","rules":[{}]}}}},"results":[{}]}}]}}"#,
            Self::SCHEMA,
            Self::VERSION,
            rules.join(","),
            results.join(",")
        )
    }
}

/// Escape special characters for SARIF JSON
fn escape_sarif(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::ErrorEntry;

    #[test]
    fn test_sarif_schema_version() {
        let report = TranspileReport::new(100, 85, 15);
        let fmt = SarifFormatter;
        let output = fmt.format(&report);

        assert!(output.contains("sarif-schema-2.1.0"));
        assert!(output.contains("\"version\":\"2.1.0\""));
    }

    #[test]
    fn test_sarif_tool_info() {
        let report = TranspileReport::new(100, 85, 15);
        let fmt = SarifFormatter;
        let output = fmt.format(&report);

        assert!(output.contains("\"name\":\"ruchy\""));
        assert!(output.contains("\"version\":\"1.0.0\""));
    }

    #[test]
    fn test_sarif_with_errors() {
        let mut report = TranspileReport::new(100, 85, 15);
        report.add_error(ErrorEntry::new("E0308", 2).with_sample("mismatched types: expected i32"));

        let fmt = SarifFormatter;
        let output = fmt.format(&report);

        assert!(output.contains("\"ruleId\":\"E0308\""));
        assert!(output.contains("\"level\":\"error\""));
        assert!(output.contains("mismatched types"));
    }

    #[test]
    fn test_sarif_rules_with_help_uri() {
        let mut report = TranspileReport::new(100, 85, 15);
        report.add_error(ErrorEntry::new("E0382", 1));

        let fmt = SarifFormatter;
        let output = fmt.format(&report);

        assert!(output.contains("\"helpUri\":\"https://doc.rust-lang.org/error-index.html#E0382\""));
    }

    #[test]
    fn test_sarif_error_severity() {
        assert_eq!(SarifFormatter::error_severity("E0308"), "error");
        assert_eq!(SarifFormatter::error_severity("W0001"), "warning");
        assert_eq!(SarifFormatter::error_severity("N0001"), "note");
    }

    #[test]
    fn test_sarif_rule_description() {
        assert_eq!(
            SarifFormatter::rule_description("E0308"),
            "mismatched types"
        );
        assert_eq!(
            SarifFormatter::rule_description("E0382"),
            "borrow of moved value"
        );
        assert_eq!(
            SarifFormatter::rule_description("E9999"),
            "transpilation error"
        );
    }

    #[test]
    fn test_escape_sarif() {
        assert_eq!(escape_sarif("hello"), "hello");
        assert_eq!(escape_sarif("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_sarif("line\nbreak"), "line\\nbreak");
    }

    #[test]
    fn test_sarif_valid_json_structure() {
        let report = TranspileReport::new(100, 85, 15);
        let fmt = SarifFormatter;
        let output = fmt.format(&report);

        // Basic JSON validation
        assert!(output.starts_with('{'));
        assert!(output.ends_with('}'));
        assert!(output.contains("\"$schema\""));
        assert!(output.contains("\"runs\""));
        assert!(output.contains("\"tool\""));
        assert!(output.contains("\"results\""));
    }
}
