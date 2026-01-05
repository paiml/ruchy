//! JSON output format for machine-readable reports

use crate::reporting::TranspileReport;

use super::ReportFormatter;

/// JSON formatter
#[derive(Default)]
pub struct JsonFormatter {
    /// Pretty print output
    pub pretty: bool,
}

impl JsonFormatter {
    /// Create formatter with pretty printing enabled
    #[must_use]
    pub fn pretty() -> Self {
        Self { pretty: true }
    }
}

impl ReportFormatter for JsonFormatter {
    fn format(&self, report: &TranspileReport) -> String {
        let errors: Vec<_> = report
            .errors
            .iter()
            .map(|e| {
                let samples: Vec<_> = e
                    .samples
                    .iter()
                    .map(|s| format!("\"{}\"", escape_json(s)))
                    .collect();
                format!(
                    r#"{{"code":"{}","count":{},"samples":[{}]}}"#,
                    e.code,
                    e.count,
                    samples.join(",")
                )
            })
            .collect();

        let trend: Vec<_> = report.trend.iter().map(f64::to_string).collect();

        let json = format!(
            r#"{{"total":{},"passed":{},"failed":{},"success_rate":{:.2},"andon":"{}","grade":"{}","trend":[{}],"errors":[{}]}}"#,
            report.total,
            report.passed,
            report.failed,
            report.success_rate(),
            report.andon().label(),
            report.grade().as_str(),
            trend.join(","),
            errors.join(",")
        );

        if self.pretty {
            pretty_print_json(&json)
        } else {
            json
        }
    }
}

/// Escape special characters for JSON
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Simple pretty printer for JSON (no external deps)
fn pretty_print_json(json: &str) -> String {
    let mut result = String::new();
    let mut indent = 0;
    let mut in_string = false;
    let mut prev_char = ' ';

    for c in json.chars() {
        if c == '"' && prev_char != '\\' {
            in_string = !in_string;
        }

        if in_string {
            result.push(c);
        } else {
            match c {
                '{' | '[' => {
                    result.push(c);
                    indent += 2;
                    result.push('\n');
                    result.push_str(&" ".repeat(indent));
                }
                '}' | ']' => {
                    indent = indent.saturating_sub(2);
                    result.push('\n');
                    result.push_str(&" ".repeat(indent));
                    result.push(c);
                }
                ',' => {
                    result.push(c);
                    result.push('\n');
                    result.push_str(&" ".repeat(indent));
                }
                ':' => {
                    result.push(c);
                    result.push(' ');
                }
                ' ' => {} // Skip spaces outside strings
                _ => result.push(c),
            }
        }
        prev_char = c;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::ErrorEntry;

    #[test]
    fn test_json_formatter_basic() {
        let report = TranspileReport::new(100, 85, 15);
        let fmt = JsonFormatter::default();
        let output = fmt.format(&report);

        assert!(output.contains("\"total\":100"));
        assert!(output.contains("\"passed\":85"));
        assert!(output.contains("\"failed\":15"));
        assert!(output.contains("\"success_rate\":85.00"));
        assert!(output.contains("\"andon\":\"GREEN\""));
    }

    #[test]
    fn test_json_formatter_with_errors() {
        let mut report = TranspileReport::new(100, 85, 15);
        report.add_error(ErrorEntry::new("E0308", 5));

        let fmt = JsonFormatter::default();
        let output = fmt.format(&report);

        assert!(output.contains("\"code\":\"E0308\""));
        assert!(output.contains("\"count\":5"));
    }

    #[test]
    fn test_json_formatter_with_trend() {
        let report = TranspileReport::new(100, 85, 15).with_trend(vec![70.0, 80.0, 85.0]);

        let fmt = JsonFormatter::default();
        let output = fmt.format(&report);

        assert!(output.contains("\"trend\":[70,80,85]"));
    }

    #[test]
    fn test_json_formatter_pretty() {
        let report = TranspileReport::new(100, 85, 15);
        let fmt = JsonFormatter::pretty();
        let output = fmt.format(&report);

        assert!(output.contains('\n'));
        assert!(output.contains("  "));
    }

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_json("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_json("tab\there"), "tab\\there");
    }

    #[test]
    fn test_json_valid_structure() {
        let mut report = TranspileReport::new(100, 85, 15);
        report.add_error(ErrorEntry::new("E0308", 5).with_sample("mismatched types"));

        let fmt = JsonFormatter::default();
        let output = fmt.format(&report);

        // Basic JSON validation: starts and ends correctly
        assert!(output.starts_with('{'));
        assert!(output.ends_with('}'));

        // Contains all required fields
        assert!(output.contains("\"total\""));
        assert!(output.contains("\"passed\""));
        assert!(output.contains("\"failed\""));
        assert!(output.contains("\"errors\""));
    }

    // === EXTREME TDD Round 19 tests ===

    #[test]
    fn test_escape_json_backslash() {
        assert_eq!(escape_json("path\\to\\file"), "path\\\\to\\\\file");
    }

    #[test]
    fn test_escape_json_carriage_return() {
        assert_eq!(escape_json("line1\r\nline2"), "line1\\r\\nline2");
    }

    #[test]
    fn test_json_formatter_default_not_pretty() {
        let fmt = JsonFormatter::default();
        assert!(!fmt.pretty);
    }

    #[test]
    fn test_pretty_print_preserves_strings() {
        // Strings with special chars should be preserved
        let json = r#"{"message":"hello\nworld"}"#;
        let pretty = pretty_print_json(json);
        assert!(pretty.contains("hello\\nworld"));
    }

    #[test]
    fn test_json_formatter_empty_errors() {
        let report = TranspileReport::new(50, 50, 0);
        let fmt = JsonFormatter::default();
        let output = fmt.format(&report);
        assert!(output.contains("\"errors\":[]"));
    }
}
