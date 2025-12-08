//! Human-readable terminal output (Mieruka visual management) [5]

use crate::reporting::ascii::{boxed_header, detect_trend, histogram_bar, progress_bar, sparkline};
use crate::reporting::TranspileReport;

use super::ReportFormatter;

/// Human-readable terminal formatter
pub struct HumanFormatter {
    /// Report width in characters
    pub width: usize,
}

impl Default for HumanFormatter {
    fn default() -> Self {
        Self { width: 70 }
    }
}

impl HumanFormatter {
    /// Create new formatter with custom width
    #[must_use]
    pub fn with_width(width: usize) -> Self {
        Self { width }
    }

    /// Format executive summary section
    fn format_summary(&self, report: &TranspileReport) -> String {
        let rate = report.success_rate();
        let andon = report.andon();
        let grade = report.grade();

        format!(
            r"EXECUTIVE SUMMARY
{}
  Total Files:       {}
  Compiles (PASS):   {}
  Fails:             {}
  Single-Shot Rate:  {:.1}%

⚑ Andon Status: {}
  Grade:         {}",
            "═".repeat(self.width),
            report.total,
            report.passed,
            report.failed,
            rate,
            andon,
            grade
        )
    }

    /// Format progress section
    fn format_progress(&self, report: &TranspileReport) -> String {
        let bar = progress_bar(report.passed, report.total, 40);
        format!(
            "Transpilation Progress: {} ({}/{})",
            bar, report.passed, report.total
        )
    }

    /// Format trend section if trend data available
    fn format_trend(&self, report: &TranspileReport) -> Option<String> {
        if report.trend.is_empty() {
            return None;
        }

        let line = sparkline(&report.trend);
        let direction = detect_trend(&report.trend);

        Some(format!(
            "Error Trend (7 days):   {} ({})",
            line,
            direction.label()
        ))
    }

    /// Format error taxonomy section
    fn format_errors(&self, report: &TranspileReport) -> String {
        if report.errors.is_empty() {
            return "No errors to report.".to_string();
        }

        let max_count = report.errors.iter().map(|e| e.count).max().unwrap_or(1);
        let mut lines = vec![format!(
            "ERROR TAXONOMY (Prioritized by Impact)\n{}",
            "═".repeat(self.width)
        )];

        for entry in &report.errors {
            let bar = histogram_bar(entry.count, max_count, 10);
            let percentage = if report.failed > 0 {
                (entry.count as f64 / report.failed as f64) * 100.0
            } else {
                0.0
            };

            lines.push(format!(
                "  {} {} ({}) - {:>5.1}%",
                entry.code, bar, entry.count, percentage
            ));

            // Add sample if available
            if let Some(sample) = entry.samples.first() {
                lines.push(format!("    → {}", truncate(sample, 50)));
            }
        }

        lines.join("\n")
    }
}

impl ReportFormatter for HumanFormatter {
    fn format(&self, report: &TranspileReport) -> String {
        let mut sections = vec![
            boxed_header("RUCHY TRANSPILATION REPORT", self.width),
            String::new(),
            self.format_progress(report),
        ];

        if let Some(trend) = self.format_trend(report) {
            sections.push(trend);
        }

        sections.push(String::new());
        sections.push(self.format_summary(report));
        sections.push(String::new());
        sections.push(self.format_errors(report));

        sections.join("\n")
    }
}

/// Truncate string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::ErrorEntry;

    #[test]
    fn test_human_formatter_default() {
        let fmt = HumanFormatter::default();
        assert_eq!(fmt.width, 70);
    }

    #[test]
    fn test_human_formatter_with_width() {
        let fmt = HumanFormatter::with_width(100);
        assert_eq!(fmt.width, 100);
    }

    #[test]
    fn test_human_format_basic_report() {
        let report = TranspileReport::new(100, 85, 15);
        let fmt = HumanFormatter::default();
        let output = fmt.format(&report);

        assert!(output.contains("RUCHY TRANSPILATION REPORT"));
        assert!(output.contains("Total Files:"));
        assert!(output.contains("100"));
        assert!(output.contains("85"));
        assert!(output.contains("Andon Status"));
    }

    #[test]
    fn test_human_format_with_errors() {
        let mut report = TranspileReport::new(100, 85, 15);
        report.add_error(
            ErrorEntry::new("E0308", 8).with_sample("mismatched types: expected i32, found String"),
        );
        report.add_error(ErrorEntry::new("E0382", 4).with_sample("borrow of moved value"));

        let fmt = HumanFormatter::default();
        let output = fmt.format(&report);

        assert!(output.contains("E0308"));
        assert!(output.contains("E0382"));
        assert!(output.contains("ERROR TAXONOMY"));
    }

    #[test]
    fn test_human_format_with_trend() {
        let report = TranspileReport::new(100, 85, 15)
            .with_trend(vec![70.0, 72.0, 75.0, 78.0, 80.0, 83.0, 85.0]);

        let fmt = HumanFormatter::default();
        let output = fmt.format(&report);

        assert!(output.contains("Error Trend (7 days)"));
        assert!(output.contains("improving"));
    }

    #[test]
    fn test_human_format_andon_green() {
        let report = TranspileReport::new(100, 85, 15);
        let fmt = HumanFormatter::default();
        let output = fmt.format(&report);

        assert!(output.contains("🟢 GREEN"));
    }

    #[test]
    fn test_human_format_andon_red() {
        let report = TranspileReport::new(100, 40, 60);
        let fmt = HumanFormatter::default();
        let output = fmt.format(&report);

        assert!(output.contains("🔴 RED"));
    }

    #[test]
    fn test_truncate_short() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long() {
        assert_eq!(truncate("hello world this is long", 10), "hello w...");
    }
}
