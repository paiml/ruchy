//! `ruchy provability` — scan a directory, report §14.2 tier distribution.
//!
//! Measures the raw input to falsifier F1 (`% of `fun` defs with
//! non-trivial contracts`) from `docs/specifications/ruchy-5.0-
//! sovereign-platform.md` §14.5.
//!
//! Ticket: PROVABILITY-001

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use ruchy::frontend::ast::{ContractClause, Literal};
use ruchy::provability::{tier_of_function, Tier, Totality};
use ruchy::{ExprKind, Parser};

/// A single classified function.
#[derive(Debug, Clone)]
pub struct ClassifiedFunction {
    pub file: PathBuf,
    pub name: String,
    pub tier: Tier,
    pub totality: Totality,
    /// Whether the function has at least one non-trivial contract clause
    /// (§14.5 F1 approximation: "SMT-non-trivial" → at minimum, not `true`).
    pub has_non_trivial_contract: bool,
    /// Whether the function was declared `pub` (part of the public API).
    pub is_pub: bool,
}

impl ClassifiedFunction {
    /// Per §14.10.6 Gold/Platinum require `@total`. Return true if this
    /// classification violates the requirement.
    #[must_use]
    pub fn violates_totality_rule(&self) -> bool {
        matches!(self.tier, Tier::Gold | Tier::Platinum)
            && !matches!(self.totality, Totality::Total | Totality::Corecursive(_))
    }
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
    /// Totality-marker counts (from @total/@partial decorators).
    pub total_marked: usize,
    pub partial_marked: usize,
    pub totality_unmarked: usize,
    /// §14.5 F1 approximation: functions with at least one non-trivial clause.
    pub non_trivial_contracts: usize,
    /// Functions where EVERY requires/ensures clause is `true` (tautology).
    pub trivial_contracts: usize,
    /// §14.5 F2: count of `#[contract_exempt]` attributes encountered.
    pub contract_exempt_count: usize,
    /// §14.5 F11: count of `#[diff_exempt]` attributes encountered.
    /// Tracks escape hatches from the (future) §14.10.4 differential gate.
    pub diff_exempt_count: usize,
    /// Lines of code scanned (for F2/F11 density computation).
    pub total_loc: usize,
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

    fn record_contract_triviality(&mut self, has_non_trivial: bool, has_any_contract: bool) {
        if has_any_contract {
            if has_non_trivial {
                self.non_trivial_contracts += 1;
            } else {
                self.trivial_contracts += 1;
            }
        }
    }

    /// §14.5 F1 metric: % of contract-bearing functions with a non-trivial clause.
    #[must_use]
    pub fn non_trivial_pct(&self) -> f64 {
        let with_contracts = self.non_trivial_contracts + self.trivial_contracts;
        if with_contracts == 0 {
            return 0.0;
        }
        (self.non_trivial_contracts as f64 / with_contracts as f64) * 100.0
    }

    /// §14.5 F2 metric: `#[contract_exempt]` attributes per 1000 LoC.
    /// Target ≤ 0.5; falsifies if > 5.
    #[must_use]
    pub fn exempt_density_per_kloc(&self) -> f64 {
        if self.total_loc == 0 {
            return 0.0;
        }
        (self.contract_exempt_count as f64 * 1000.0) / self.total_loc as f64
    }

    /// §14.5 F11 metric: `#[diff_exempt]` attributes per 1000 LoC.
    /// Parallels F2 but tracks escape hatches from §14.10.4 differential
    /// execution gate. Shipping F11 reporter now so baseline density is
    /// observable before the §14.10.4 gate itself lands.
    #[must_use]
    pub fn diff_exempt_density_per_kloc(&self) -> f64 {
        if self.total_loc == 0 {
            return 0.0;
        }
        (self.diff_exempt_count as f64 * 1000.0) / self.total_loc as f64
    }

    /// Emit the full report as a single-line JSON object for dashboards.
    #[must_use]
    pub fn to_json(&self) -> String {
        format!(
            "{{\
\"files\":{},\
\"loc\":{},\
\"functions\":{},\
\"bronze\":{},\
\"silver\":{},\
\"gold\":{},\
\"platinum\":{},\
\"non_bronze_pct\":{:.2},\
\"non_trivial_contracts\":{},\
\"trivial_contracts\":{},\
\"non_trivial_pct\":{:.2},\
\"contract_exempt\":{},\
\"exempt_density_per_kloc\":{:.2},\
\"diff_exempt\":{},\
\"diff_exempt_density_per_kloc\":{:.2},\
\"total_marked\":{},\
\"partial_marked\":{},\
\"totality_unmarked\":{},\
\"totality_violations\":{},\
\"pub_bronze\":{},\
\"parse_errors\":{}\
}}",
            self.files_scanned,
            self.total_loc,
            self.functions_total,
            self.bronze,
            self.silver,
            self.gold,
            self.platinum,
            self.non_bronze_pct(),
            self.non_trivial_contracts,
            self.trivial_contracts,
            self.non_trivial_pct(),
            self.contract_exempt_count,
            self.exempt_density_per_kloc(),
            self.diff_exempt_count,
            self.diff_exempt_density_per_kloc(),
            self.total_marked,
            self.partial_marked,
            self.totality_unmarked,
            self.totality_violations().len(),
            self.pub_bronze_count(),
            self.parse_errors,
        )
    }

    fn record_totality(&mut self, totality: Totality) {
        match totality {
            Totality::Total => self.total_marked += 1,
            Totality::Partial => self.partial_marked += 1,
            Totality::Corecursive(_) => self.total_marked += 1, // counts as "proved to not hang"
            Totality::Unknown => self.totality_unmarked += 1,
        }
    }

    /// Returns functions that violate §14.10.6 (Gold/Platinum without @total).
    #[must_use]
    pub fn totality_violations(&self) -> Vec<&ClassifiedFunction> {
        self.functions
            .iter()
            .filter(|f| f.violates_totality_rule())
            .collect()
    }

    /// §14.5 F4 proxy: count of Bronze-tier `pub` functions.
    /// After release 5.2, stdlib `pub` functions must not be Bronze.
    #[must_use]
    pub fn pub_bronze_count(&self) -> usize {
        self.functions
            .iter()
            .filter(|f| f.is_pub && matches!(f.tier, Tier::Bronze))
            .count()
    }

    /// Derive a new report containing ONLY `pub` functions. Aggregate
    /// counts and totals are recomputed from the filtered set.
    #[must_use]
    pub fn filter_to_pub(&self) -> Self {
        let mut out = Self {
            files_scanned: self.files_scanned,
            total_loc: self.total_loc,
            parse_errors: self.parse_errors,
            // §14.5 F2/F11: exemptions are file-level, not fn-scoped, so
            // we keep the absolute counts for density computation.
            contract_exempt_count: self.contract_exempt_count,
            diff_exempt_count: self.diff_exempt_count,
            ..Self::default()
        };
        for f in &self.functions {
            if !f.is_pub {
                continue;
            }
            out.record_tier(f.tier);
            out.record_totality(f.totality);
            if f.has_non_trivial_contract {
                out.non_trivial_contracts += 1;
            }
            out.functions.push(f.clone());
        }
        out
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
            "files: {}\nloc: {}\nfunctions: {}\n  bronze:   {} ({:.1}%)\n  silver:   {} ({:.1}%)\n  gold:     {} ({:.1}%)\n  platinum: {} ({:.1}%)\nnon-bronze: {:.1}%\ncontract triviality (F1):\n  non-trivial: {}\n  trivial:     {}\n  non-trivial %: {:.1}%\nexemptions (F2):\n  #[contract_exempt]: {}\n  density / KLoC:     {:.2}\ndiff exemptions (F11):\n  #[diff_exempt]: {}\n  density / KLoC: {:.2}\npublic API (F4 proxy):\n  pub Bronze: {}\ntotality:\n  @total:    {}\n  @partial:  {}\n  unmarked:  {}\nparse errors: {}",
            self.files_scanned,
            self.total_loc,
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
            self.non_trivial_contracts,
            self.trivial_contracts,
            self.non_trivial_pct(),
            self.contract_exempt_count,
            self.exempt_density_per_kloc(),
            self.diff_exempt_count,
            self.diff_exempt_density_per_kloc(),
            self.pub_bronze_count(),
            self.total_marked,
            self.partial_marked,
            self.totality_unmarked,
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

fn function_is_pub(expr: &ruchy::Expr) -> bool {
    match &expr.kind {
        ExprKind::Function { is_pub, .. } => *is_pub,
        _ => false,
    }
}

/// A contract clause is "trivially true" if it is the literal `true`.
/// This is the §14.5 F1 approximation — a cheap syntactic check that
/// catches `requires true` / `ensures true`. Genuine SMT-based tautology
/// detection is a future sprint.
fn clause_is_trivially_true(clause: &ContractClause) -> bool {
    let body = match clause {
        ContractClause::Requires(e) => e,
        ContractClause::Ensures(e) => e,
        // Invariant/Decreases aren't part of the F1 requires/ensures
        // triviality check.
        ContractClause::Invariant(_) | ContractClause::Decreases(_) => return false,
    };
    matches!(body.kind, ExprKind::Literal(Literal::Bool(true)))
}

/// Returns (has_any_contract, has_non_trivial) for a function's contract list.
fn analyze_contract_triviality(contracts: &[ContractClause]) -> (bool, bool) {
    let req_ens: Vec<&ContractClause> = contracts
        .iter()
        .filter(|c| matches!(c, ContractClause::Requires(_) | ContractClause::Ensures(_)))
        .collect();
    if req_ens.is_empty() {
        return (false, false);
    }
    let has_non_trivial = req_ens.iter().any(|c| !clause_is_trivially_true(c));
    (true, has_non_trivial)
}

fn function_totality(expr: &ruchy::Expr) -> Totality {
    // Scan attributes for @total, @partial. @corecursive requires a
    // justification argument and is intentionally not detected by
    // bare-name lookup (Totality::from_decorator returns None for it).
    for attr in &expr.attributes {
        if let Some(t) = Totality::from_decorator(&attr.name) {
            return t;
        }
    }
    Totality::Unknown
}

fn has_attribute(expr: &ruchy::Expr, name: &str) -> bool {
    expr.attributes.iter().any(|a| a.name == name)
}

fn has_contract_exempt(expr: &ruchy::Expr) -> bool {
    has_attribute(expr, "contract_exempt")
}

fn has_diff_exempt(expr: &ruchy::Expr) -> bool {
    has_attribute(expr, "diff_exempt")
}

fn visit_expr(expr: &ruchy::Expr, file: &Path, report: &mut ProvabilityReport) {
    // Detect escape-hatch attributes on any expression.
    if has_contract_exempt(expr) {
        report.contract_exempt_count += 1;
    }
    if has_diff_exempt(expr) {
        report.diff_exempt_count += 1;
    }
    if matches!(expr.kind, ExprKind::Function { .. }) {
        if let Some(tier) = tier_of_function(expr) {
            report.record_tier(tier);
            let totality = function_totality(expr);
            report.record_totality(totality);
            let (has_contract, has_non_trivial) = analyze_contract_triviality(&expr.contracts);
            report.record_contract_triviality(has_non_trivial, has_contract);
            if let Some(name) = function_name(expr) {
                report.functions.push(ClassifiedFunction {
                    file: file.to_path_buf(),
                    name: name.to_string(),
                    tier,
                    totality,
                    has_non_trivial_contract: has_non_trivial,
                    is_pub: function_is_pub(expr),
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
        report.total_loc += src.lines().count();
        classify_source(&src, file, &mut report);
    }
    Ok(report)
}

/// CLI entry point for `ruchy tier <path>`.
#[allow(clippy::too_many_arguments)]
pub fn handle_provability_command(
    path: &Path,
    json: bool,
    list: bool,
    fail_under: Option<f64>,
    fail_on_totality_violation: bool,
    fail_under_f1: Option<f64>,
    fail_exempt_density_above: Option<f64>,
    public_only: bool,
    fail_pub_bronze_above: Option<usize>,
) -> Result<()> {
    let raw = scan(path)?;
    let report = if public_only { raw.filter_to_pub() } else { raw };
    if json {
        println!("{}", report.to_json());
    } else {
        println!("Provability tier scan: {}", path.display());
        println!("{}", report.summary());
        if list {
            println!("\nfunctions:");
            for f in &report.functions {
                println!(
                    "  {:<10} {:<10} {:<4} {} ({})",
                    f.tier.label(),
                    f.totality.label(),
                    if f.is_pub { "pub" } else { "" },
                    f.name,
                    f.file.display()
                );
            }
        }
        // §14.10.6 totality rule enforcement: Gold/Platinum MUST be @total.
        let violations = report.totality_violations();
        if !violations.is_empty() {
            eprintln!(
                "\n§14.10.6 violations: {} Gold/Platinum function(s) lack @total:",
                violations.len()
            );
            for f in &violations {
                eprintln!(
                    "  {} ({}) is {} but has {}",
                    f.name,
                    f.file.display(),
                    f.tier.label(),
                    f.totality.label()
                );
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
    // Apply --fail-on-totality-violation gate (§14.10.6 CI enforcement).
    if fail_on_totality_violation {
        let violations = report.totality_violations();
        if !violations.is_empty() {
            anyhow::bail!(
                "{} Gold/Platinum function(s) lack @total (§14.10.6 breach)",
                violations.len()
            );
        }
    }
    // Apply --fail-under-f1 gate (§14.5 F1 CI enforcement).
    if let Some(threshold) = fail_under_f1 {
        // F1 is only meaningful when at least one function has a contract.
        let with_contracts = report.non_trivial_contracts + report.trivial_contracts;
        if with_contracts > 0 {
            let actual = report.non_trivial_pct();
            if actual < threshold {
                anyhow::bail!(
                    "non-trivial contract pct {:.2}% is below threshold {:.2}% (§14.5 F1 breach)",
                    actual,
                    threshold
                );
            }
        }
    }
    // Apply --fail-exempt-density-above gate (§14.5 F2 CI enforcement).
    if let Some(ceiling) = fail_exempt_density_above {
        // F2 is only meaningful when at least some LoC has been scanned.
        if report.total_loc > 0 {
            let actual = report.exempt_density_per_kloc();
            if actual > ceiling {
                anyhow::bail!(
                    "#[contract_exempt] density {:.2}/KLoC exceeds ceiling {:.2}/KLoC (§14.5 F2 breach)",
                    actual,
                    ceiling
                );
            }
        }
    }
    // Apply --fail-pub-bronze-above gate (§14.5 F4 proxy CI enforcement).
    if let Some(ceiling) = fail_pub_bronze_above {
        let actual = report.pub_bronze_count();
        if actual > ceiling {
            anyhow::bail!(
                "pub Bronze count {} exceeds ceiling {} (§14.5 F4 breach)",
                actual,
                ceiling
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
    fn test_classify_source_records_total_marker() {
        let mut r = ProvabilityReport::default();
        classify_source(
            "#[total]\nfun f() requires true { 1 }",
            Path::new("test.ruchy"),
            &mut r,
        );
        assert_eq!(r.total_marked, 1);
        assert_eq!(r.totality_unmarked, 0);
    }

    #[test]
    fn test_classify_source_records_partial_marker() {
        let mut r = ProvabilityReport::default();
        classify_source(
            "#[partial]\nfun f() { 1 }",
            Path::new("test.ruchy"),
            &mut r,
        );
        assert_eq!(r.partial_marked, 1);
        assert_eq!(r.total_marked, 0);
    }

    #[test]
    fn test_classify_source_counts_unmarked() {
        let mut r = ProvabilityReport::default();
        classify_source("fun f() { 1 }", Path::new("test.ruchy"), &mut r);
        assert_eq!(r.total_marked, 0);
        assert_eq!(r.partial_marked, 0);
        assert_eq!(r.totality_unmarked, 1);
    }

    #[test]
    fn test_violates_totality_rule_gold_without_total() {
        let cf = ClassifiedFunction {
            file: PathBuf::from("x.ruchy"),
            name: "f".to_string(),
            tier: Tier::Gold,
            totality: Totality::Unknown,
            has_non_trivial_contract: true,
            is_pub: false,
        };
        assert!(cf.violates_totality_rule());
    }

    #[test]
    fn test_violates_totality_rule_gold_with_total_ok() {
        let cf = ClassifiedFunction {
            file: PathBuf::from("x.ruchy"),
            name: "f".to_string(),
            tier: Tier::Gold,
            totality: Totality::Total,
            has_non_trivial_contract: true,
            is_pub: false,
        };
        assert!(!cf.violates_totality_rule());
    }

    #[test]
    fn test_violates_totality_rule_silver_unaffected() {
        // Silver may be @partial — no violation.
        let cf = ClassifiedFunction {
            file: PathBuf::from("x.ruchy"),
            name: "f".to_string(),
            tier: Tier::Silver,
            totality: Totality::Partial,
            has_non_trivial_contract: true,
            is_pub: false,
        };
        assert!(!cf.violates_totality_rule());
    }

    #[test]
    fn test_violates_totality_rule_gold_with_corecursive_ok() {
        let cf = ClassifiedFunction {
            file: PathBuf::from("server.ruchy"),
            name: "event_loop".to_string(),
            tier: Tier::Gold,
            totality: Totality::Corecursive("server event loop"),
            has_non_trivial_contract: true,
            is_pub: false,
        };
        assert!(!cf.violates_totality_rule());
    }

    #[test]
    fn test_trivial_contract_requires_true_detected() {
        let mut r = ProvabilityReport::default();
        classify_source(
            "fun f() requires true ensures true { 1 }",
            Path::new("t.ruchy"),
            &mut r,
        );
        assert_eq!(r.trivial_contracts, 1);
        assert_eq!(r.non_trivial_contracts, 0);
    }

    #[test]
    fn test_non_trivial_contract_detected() {
        let mut r = ProvabilityReport::default();
        classify_source(
            "fun f(x: i32) requires x > 0 ensures x > 0 { x }",
            Path::new("t.ruchy"),
            &mut r,
        );
        assert_eq!(r.non_trivial_contracts, 1);
        assert_eq!(r.trivial_contracts, 0);
    }

    #[test]
    fn test_mixed_trivial_and_nontrivial_is_nontrivial() {
        // At least one non-trivial clause → classifies as non-trivial.
        let mut r = ProvabilityReport::default();
        classify_source(
            "fun f(x: i32) requires true ensures x > 0 { x }",
            Path::new("t.ruchy"),
            &mut r,
        );
        assert_eq!(r.non_trivial_contracts, 1);
        assert_eq!(r.trivial_contracts, 0);
    }

    #[test]
    fn test_no_contract_is_neither_trivial_nor_nontrivial() {
        let mut r = ProvabilityReport::default();
        classify_source("fun f() { 1 }", Path::new("t.ruchy"), &mut r);
        assert_eq!(r.trivial_contracts, 0);
        assert_eq!(r.non_trivial_contracts, 0);
    }

    #[test]
    fn test_non_trivial_pct_calculation() {
        let mut r = ProvabilityReport::default();
        r.non_trivial_contracts = 3;
        r.trivial_contracts = 1;
        // 3 of 4 contract-bearing functions have a non-trivial clause.
        assert_eq!(r.non_trivial_pct(), 75.0);
    }

    #[test]
    fn test_non_trivial_pct_zero_contracts_returns_zero() {
        let r = ProvabilityReport::default();
        assert_eq!(r.non_trivial_pct(), 0.0);
    }

    #[test]
    fn test_exempt_density_empty_loc_is_zero() {
        let r = ProvabilityReport::default();
        assert_eq!(r.exempt_density_per_kloc(), 0.0);
    }

    #[test]
    fn test_exempt_density_calculation() {
        let mut r = ProvabilityReport::default();
        r.contract_exempt_count = 2;
        r.total_loc = 1000;
        // 2 exemptions per 1000 LoC = 2.00 / KLoC.
        assert_eq!(r.exempt_density_per_kloc(), 2.0);
    }

    #[test]
    fn test_exempt_density_sub_kloc() {
        let mut r = ProvabilityReport::default();
        r.contract_exempt_count = 1;
        r.total_loc = 500;
        // 1 exemption in 500 LoC = 2.00 / KLoC.
        assert_eq!(r.exempt_density_per_kloc(), 2.0);
    }

    #[test]
    fn test_exempt_density_zero_exemptions() {
        let mut r = ProvabilityReport::default();
        r.contract_exempt_count = 0;
        r.total_loc = 1000;
        assert_eq!(r.exempt_density_per_kloc(), 0.0);
    }

    #[test]
    fn test_contract_exempt_detected_on_function() {
        let mut r = ProvabilityReport::default();
        classify_source(
            "#[contract_exempt]\nfun bypass() { 1 }",
            Path::new("t.ruchy"),
            &mut r,
        );
        assert_eq!(r.contract_exempt_count, 1);
    }

    #[test]
    fn test_contract_exempt_not_counted_for_other_attrs() {
        let mut r = ProvabilityReport::default();
        classify_source(
            "#[bronze]\nfun f() { 1 }",
            Path::new("t.ruchy"),
            &mut r,
        );
        assert_eq!(r.contract_exempt_count, 0);
    }

    #[test]
    fn test_diff_exempt_detected() {
        let mut r = ProvabilityReport::default();
        classify_source(
            "#[diff_exempt]\nfun divergent() { 1 }",
            Path::new("t.ruchy"),
            &mut r,
        );
        assert_eq!(r.diff_exempt_count, 1);
        assert_eq!(r.contract_exempt_count, 0);
    }

    #[test]
    fn test_diff_exempt_and_contract_exempt_independent() {
        // Both attributes on the same function should count in both buckets.
        let mut r = ProvabilityReport::default();
        classify_source(
            "#[contract_exempt]\n#[diff_exempt]\nfun exempt() { 1 }",
            Path::new("t.ruchy"),
            &mut r,
        );
        assert_eq!(r.contract_exempt_count, 1);
        assert_eq!(r.diff_exempt_count, 1);
    }

    #[test]
    fn test_diff_exempt_density_zero_loc_is_zero() {
        let r = ProvabilityReport::default();
        assert_eq!(r.diff_exempt_density_per_kloc(), 0.0);
    }

    #[test]
    fn test_diff_exempt_density_calculation() {
        let mut r = ProvabilityReport::default();
        r.diff_exempt_count = 3;
        r.total_loc = 1000;
        assert_eq!(r.diff_exempt_density_per_kloc(), 3.0);
    }

    #[test]
    fn test_is_pub_detected_on_function() {
        let mut r = ProvabilityReport::default();
        classify_source("pub fun f() { 1 }", Path::new("t.ruchy"), &mut r);
        assert_eq!(r.functions.len(), 1);
        assert!(r.functions[0].is_pub);
    }

    #[test]
    fn test_is_pub_false_on_private_function() {
        let mut r = ProvabilityReport::default();
        classify_source("fun f() { 1 }", Path::new("t.ruchy"), &mut r);
        assert_eq!(r.functions.len(), 1);
        assert!(!r.functions[0].is_pub);
    }

    #[test]
    fn test_pub_bronze_count_sees_only_pub_bronze() {
        let mut r = ProvabilityReport::default();
        classify_source(
            "pub fun a() { 1 }\nfun b() { 2 }\npub fun c() requires true { 3 }",
            Path::new("t.ruchy"),
            &mut r,
        );
        // a: pub Bronze ✓
        // b: private Bronze ✗ (not pub)
        // c: pub Silver ✗ (not Bronze)
        assert_eq!(r.pub_bronze_count(), 1);
    }

    #[test]
    fn test_filter_to_pub_keeps_only_pub_functions() {
        let mut r = ProvabilityReport::default();
        classify_source(
            "pub fun a() { 1 }\nfun b() { 2 }\npub fun c() { 3 }",
            Path::new("t.ruchy"),
            &mut r,
        );
        let pub_only = r.filter_to_pub();
        assert_eq!(pub_only.functions.len(), 2);
        assert!(pub_only.functions.iter().all(|f| f.is_pub));
        assert_eq!(pub_only.functions_total, 2);
        assert_eq!(pub_only.bronze, 2);
    }

    #[test]
    fn test_filter_to_pub_preserves_file_and_loc_totals() {
        let mut r = ProvabilityReport::default();
        r.files_scanned = 5;
        r.total_loc = 1000;
        classify_source("pub fun a() { 1 }", Path::new("t.ruchy"), &mut r);
        let pub_only = r.filter_to_pub();
        // File/LoC counts are preserved (they're file-level, not fn-scoped).
        assert_eq!(pub_only.files_scanned, 5);
        assert_eq!(pub_only.total_loc, 1000);
    }

    #[test]
    fn test_to_json_contains_all_metric_keys() {
        let mut r = ProvabilityReport::default();
        r.record_tier(Tier::Silver);
        r.record_tier(Tier::Gold);
        r.record_totality(Totality::Total);
        r.total_loc = 500;
        r.contract_exempt_count = 1;
        r.non_trivial_contracts = 1;
        r.trivial_contracts = 1;
        let j = r.to_json();
        // §14.5 metric keys
        for key in [
            "files",
            "loc",
            "functions",
            "bronze",
            "silver",
            "gold",
            "platinum",
            "non_bronze_pct",
            "non_trivial_contracts",
            "trivial_contracts",
            "non_trivial_pct",
            "contract_exempt",
            "exempt_density_per_kloc",
            "diff_exempt",
            "diff_exempt_density_per_kloc",
            "total_marked",
            "partial_marked",
            "totality_unmarked",
            "totality_violations",
            "pub_bronze",
            "parse_errors",
        ] {
            assert!(j.contains(key), "JSON missing key `{key}`: {j}");
        }
    }

    #[test]
    fn test_to_json_is_single_line() {
        let r = ProvabilityReport::default();
        let j = r.to_json();
        assert!(!j.contains('\n'), "JSON must be single-line: {j}");
    }

    #[test]
    fn test_to_json_includes_correct_values() {
        let mut r = ProvabilityReport::default();
        r.record_tier(Tier::Silver);
        r.record_tier(Tier::Silver);
        r.record_tier(Tier::Bronze);
        r.total_loc = 100;
        let j = r.to_json();
        assert!(j.contains("\"silver\":2"));
        assert!(j.contains("\"bronze\":1"));
        assert!(j.contains("\"functions\":3"));
        assert!(j.contains("\"loc\":100"));
    }

    #[test]
    fn test_to_json_includes_pub_bronze_value() {
        let mut r = ProvabilityReport::default();
        classify_source(
            "pub fun a() { 1 }\nfun b() { 2 }\npub fun c() requires x > 0 { 3 }",
            Path::new("t.ruchy"),
            &mut r,
        );
        // 1 pub Bronze (a), 1 private Bronze (b, not counted), 1 pub Silver (c, not Bronze)
        let j = r.to_json();
        assert!(j.contains("\"pub_bronze\":1"), "JSON: {j}");
    }

    #[test]
    fn test_summary_is_non_empty() {
        let mut r = ProvabilityReport::default();
        r.record_tier(Tier::Silver);
        let s = r.summary();
        assert!(s.contains("functions: 1"));
        assert!(s.contains("silver:"));
    }

    #[test]
    fn test_summary_contains_pub_bronze_section() {
        let mut r = ProvabilityReport::default();
        classify_source("pub fun a() { 1 }", Path::new("t.ruchy"), &mut r);
        let s = r.summary();
        assert!(s.contains("public API (F4 proxy)"));
        assert!(s.contains("pub Bronze: 1"));
    }
}
