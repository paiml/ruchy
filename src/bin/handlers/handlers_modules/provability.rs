//! `ruchy provability` — scan a directory, report §14.2 tier distribution.
//!
//! Measures the raw input to falsifier F1 (`% of `fun` defs with
//! non-trivial contracts`) from `docs/specifications/ruchy-5.0-
//! sovereign-platform.md` §14.5.
//!
//! Ticket: PROVABILITY-001

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use ruchy::provability::{tier_of_function, Tier};
use ruchy::{ExprKind, Parser};

/// A single classified function.
#[derive(Debug, Clone)]
pub struct ClassifiedFunction {
    pub file: PathBuf,
    pub name: String,
    pub tier: Tier,
}

/// Aggregate tier counts for a directory scan.
#[derive(Debug, Default, Clone)]
pub struct ProvabilityReport {
    pub files_scanned: usize,
    pub functions_total: usize,
    pub bronze: usize,
    pub silver: usize,
    pub gold: usize,
    pub platinum: usize,
    pub parse_errors: usize,
    /// Per-function classifications (populated when caller needs detail).
    pub functions: Vec<ClassifiedFunction>,
}

impl ProvabilityReport {
    fn record_tier(&mut self, tier: Tier) {
        self.functions_total += 1;
        match tier {
            Tier::Bronze => self.bronze += 1,
            Tier::Silver => self.silver += 1,
            Tier::Gold => self.gold += 1,
            Tier::Platinum => self.platinum += 1,
        }
    }

    /// Percentage of functions at Silver tier or above.
    #[must_use]
    pub fn non_bronze_pct(&self) -> f64 {
        if self.functions_total == 0 {
            return 0.0;
        }
        let non_bronze = self.silver + self.gold + self.platinum;
        (non_bronze as f64 / self.functions_total as f64) * 100.0
    }

    /// Human-readable summary.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "files: {}\nfunctions: {}\n  bronze:   {} ({:.1}%)\n  silver:   {} ({:.1}%)\n  gold:     {} ({:.1}%)\n  platinum: {} ({:.1}%)\nnon-bronze: {:.1}%\nparse errors: {}",
            self.files_scanned,
            self.functions_total,
            self.bronze,
            pct(self.bronze, self.functions_total),
            self.silver,
            pct(self.silver, self.functions_total),
            self.gold,
            pct(self.gold, self.functions_total),
            self.platinum,
            pct(self.platinum, self.functions_total),
            self.non_bronze_pct(),
            self.parse_errors,
        )
    }
}

fn pct(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        (numerator as f64 / denominator as f64) * 100.0
    }
}

fn collect_ruchy_files(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("ruchy") {
        out.push(path.to_path_buf());
        return;
    }
    if !path.is_dir() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_ruchy_files(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("ruchy") {
            out.push(p);
        }
    }
}

fn classify_source(src: &str, file: &Path, report: &mut ProvabilityReport) {
    let mut parser = Parser::new(src);
    let Ok(expr) = parser.parse() else {
        report.parse_errors += 1;
        return;
    };
    visit_expr(&expr, file, report);
}

fn function_name(expr: &ruchy::Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Function { name, .. } => Some(name.as_str()),
        _ => None,
    }
}

fn visit_expr(expr: &ruchy::Expr, file: &Path, report: &mut ProvabilityReport) {
    if matches!(expr.kind, ExprKind::Function { .. }) {
        if let Some(tier) = tier_of_function(expr) {
            report.record_tier(tier);
            if let Some(name) = function_name(expr) {
                report.functions.push(ClassifiedFunction {
                    file: file.to_path_buf(),
                    name: name.to_string(),
                    tier,
                });
            }
        }
    }
    if let ExprKind::Block(exprs) = &expr.kind {
        for e in exprs {
            visit_expr(e, file, report);
        }
    }
}

/// Scan `path` (file or directory) and build a [`ProvabilityReport`].
pub fn scan(path: &Path) -> Result<ProvabilityReport> {
    let mut files = Vec::new();
    collect_ruchy_files(path, &mut files);
    let mut report = ProvabilityReport::default();
    for file in &files {
        let src = std::fs::read_to_string(file)
            .with_context(|| format!("reading {}", file.display()))?;
        report.files_scanned += 1;
        classify_source(&src, file, &mut report);
    }
    Ok(report)
}

/// CLI entry point for `ruchy tier <path>`.
pub fn handle_provability_command(
    path: &Path,
    json: bool,
    list: bool,
    fail_under: Option<f64>,
) -> Result<()> {
    let report = scan(path)?;
    if json {
        println!(
            "{{\"files\":{},\"functions\":{},\"bronze\":{},\"silver\":{},\"gold\":{},\"platinum\":{},\"non_bronze_pct\":{:.2},\"parse_errors\":{}}}",
            report.files_scanned,
            report.functions_total,
            report.bronze,
            report.silver,
            report.gold,
            report.platinum,
            report.non_bronze_pct(),
            report.parse_errors,
        );
    } else {
        println!("Provability tier scan: {}", path.display());
        println!("{}", report.summary());
        if list {
            println!("\nfunctions:");
            for f in &report.functions {
                println!("  {:<10} {} ({})", f.tier.label(), f.name, f.file.display());
            }
        }
    }
    // Apply --fail-under gate (F1 CI enforcement).
    if let Some(threshold) = fail_under {
        let actual = report.non_bronze_pct();
        if actual < threshold {
            anyhow::bail!(
                "non-bronze-pct {:.2}% is below threshold {:.2}% (F1 falsifier breach)",
                actual,
                threshold
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_empty_report_has_zero_fns() {
        let r = ProvabilityReport::default();
        assert_eq!(r.functions_total, 0);
        assert_eq!(r.non_bronze_pct(), 0.0);
    }

    #[test]
    fn test_record_tier_updates_counts() {
        let mut r = ProvabilityReport::default();
        r.record_tier(Tier::Bronze);
        r.record_tier(Tier::Silver);
        r.record_tier(Tier::Silver);
        r.record_tier(Tier::Gold);
        assert_eq!(r.functions_total, 4);
        assert_eq!(r.bronze, 1);
        assert_eq!(r.silver, 2);
        assert_eq!(r.gold, 1);
        assert_eq!(r.platinum, 0);
        assert_eq!(r.non_bronze_pct(), 75.0);
    }

    #[test]
    fn test_classify_source_bare_fn_is_bronze() {
        let mut r = ProvabilityReport::default();
        classify_source("fun f() { 1 }", Path::new("test.ruchy"), &mut r);
        assert_eq!(r.functions_total, 1);
        assert_eq!(r.bronze, 1);
    }

    #[test]
    fn test_classify_source_multiple_fns_counts_each() {
        let mut r = ProvabilityReport::default();
        classify_source(
            "fun a() { 1 }\nfun b() { 2 }\n#[bronze]\nfun c() { 3 }",
            Path::new("test.ruchy"),
            &mut r,
        );
        assert_eq!(r.functions_total, 3);
        assert_eq!(r.bronze, 3);
    }

    #[test]
    fn test_classify_source_unparseable_increments_errors() {
        let mut r = ProvabilityReport::default();
        classify_source("this is not valid ruchy @#$%", Path::new("test.ruchy"), &mut r);
        assert_eq!(r.parse_errors, 1);
        assert_eq!(r.functions_total, 0);
    }

    #[test]
    fn test_scan_directory_of_ruchy_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("a.ruchy"), "fun a() { 1 }").unwrap();
        fs::write(tmp.path().join("b.ruchy"), "fun b() { 2 }\nfun c() { 3 }").unwrap();
        // Non-.ruchy file should be ignored
        fs::write(tmp.path().join("notes.txt"), "skip me").unwrap();

        let report = scan(tmp.path()).unwrap();
        assert_eq!(report.files_scanned, 2);
        assert_eq!(report.functions_total, 3);
        assert_eq!(report.bronze, 3);
    }

    #[test]
    fn test_scan_single_file_path() {
        let tmp = TempDir::new().unwrap();
        let f = tmp.path().join("x.ruchy");
        fs::write(&f, "fun x() { 42 }").unwrap();
        let report = scan(&f).unwrap();
        assert_eq!(report.files_scanned, 1);
        assert_eq!(report.functions_total, 1);
    }

    #[test]
    fn test_scan_recurses_subdirectories() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("nested");
        fs::create_dir_all(&sub).unwrap();
        fs::write(tmp.path().join("top.ruchy"), "fun t() { 0 }").unwrap();
        fs::write(sub.join("deep.ruchy"), "fun d() { 0 }").unwrap();
        let report = scan(tmp.path()).unwrap();
        assert_eq!(report.files_scanned, 2);
        assert_eq!(report.functions_total, 2);
    }

    #[test]
    fn test_summary_is_non_empty() {
        let mut r = ProvabilityReport::default();
        r.record_tier(Tier::Silver);
        let s = r.summary();
        assert!(s.contains("functions: 1"));
        assert!(s.contains("silver:"));
    }
}
