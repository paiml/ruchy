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

impl BaselineSnapshot {
    /// Compare `current` against `self` (baseline) and return all
    /// metrics that regressed. "Regression" semantics:
    /// - count-like metrics regress when they INCREASE (more bad, more errors)
    /// - percentage-like metrics regress when they DECREASE (less good)
    #[must_use]
    pub fn regressions_vs(&self, current: &BaselineSnapshot) -> Vec<Regression> {
        let mut out = Vec::new();
        // Counts that should not increase.
        for (name, base, cur) in &[
            ("bronze", self.bronze, current.bronze),
            ("pub_bronze", self.pub_bronze, current.pub_bronze),
            ("contract_exempt", self.contract_exempt, current.contract_exempt),
            ("diff_exempt", self.diff_exempt, current.diff_exempt),
            ("parse_errors", self.parse_errors, current.parse_errors),
            ("parse_timeouts", self.parse_timeouts, current.parse_timeouts),
            (
                "totality_violations",
                self.totality_violations,
                current.totality_violations,
            ),
        ] {
            if cur > base {
                out.push(Regression {
                    metric: name,
                    baseline: base.to_string(),
                    current: cur.to_string(),
                });
            }
        }
        // Percentages that should not decrease.
        for (name, base, cur) in &[
            ("non_trivial_pct", self.non_trivial_pct, current.non_trivial_pct),
            ("non_bronze_pct", self.non_bronze_pct, current.non_bronze_pct),
        ] {
            // Tolerate 0.01% rounding noise.
            if *cur + 0.01 < *base {
                out.push(Regression {
                    metric: name,
                    baseline: format!("{base:.2}"),
                    current: format!("{cur:.2}"),
                });
            }
        }
        // Densities that should not increase. Tolerate 0.01 noise.
        for (name, base, cur) in &[
            (
                "exempt_density_per_kloc",
                self.exempt_density_per_kloc,
                current.exempt_density_per_kloc,
            ),
            (
                "diff_exempt_density_per_kloc",
                self.diff_exempt_density_per_kloc,
                current.diff_exempt_density_per_kloc,
            ),
        ] {
            if *cur > *base + 0.01 {
                out.push(Regression {
                    metric: name,
                    baseline: format!("{base:.2}"),
                    current: format!("{cur:.2}"),
                });
            }
        }
        out
    }
}

impl ClassifiedFunction {
    /// Per §14.10.6 Gold/Platinum require `@total`. Return true if this
    /// classification violates the requirement.
    #[must_use]
    pub fn violates_totality_rule(&self) -> bool {
        matches!(self.tier, Tier::Gold | Tier::Platinum)
            && !matches!(self.totality, Totality::Total | Totality::Corecursive(_))
    }

    /// Emit this function as a single-line JSON object. Filenames are
    /// JSON-escaped minimally (backslash, quote, control chars).
    #[must_use]
    pub fn to_json(&self) -> String {
        format!(
            "{{\"name\":\"{}\",\"file\":\"{}\",\"tier\":\"{}\",\"totality\":\"{}\",\"pub\":{},\"non_trivial_contract\":{}}}",
            escape_json(&self.name),
            escape_json(&self.file.to_string_lossy()),
            self.tier.label(),
            self.totality.label(),
            self.is_pub,
            self.has_non_trivial_contract,
        )
    }
}

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
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
    /// Count of files where the parser exceeded the per-file timeout.
    /// Indicates a parser bug (infinite loop / pathological input).
    pub parse_timeouts: usize,
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

/// Snapshot of the metrics that matter for baseline comparison.
/// Intentionally a subset of `ProvabilityReport` — only the numeric
/// metrics we regression-check in CI.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BaselineSnapshot {
    pub bronze: usize,
    pub silver: usize,
    pub gold: usize,
    pub platinum: usize,
    pub pub_bronze: usize,
    pub non_trivial_pct: f64,
    pub non_bronze_pct: f64,
    pub exempt_density_per_kloc: f64,
    pub diff_exempt_density_per_kloc: f64,
    pub contract_exempt: usize,
    pub diff_exempt: usize,
    pub parse_errors: usize,
    pub parse_timeouts: usize,
    pub totality_violations: usize,
}

/// A single metric regression: name, baseline value, current value.
#[derive(Debug, Clone)]
pub struct Regression {
    pub metric: &'static str,
    pub baseline: String,
    pub current: String,
}

/// §14.5 falsifier status for a single metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FalsifierStatus {
    /// Metric meets the 5.2 target.
    Ok,
    /// Metric misses the target but is not in falsifier range.
    Warn,
    /// Metric is in falsifier range (refutes the claim).
    Fail,
    /// Metric cannot be evaluated (e.g., no contracts → no F1).
    NotApplicable,
}

impl FalsifierStatus {
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
            Self::NotApplicable => "N/A",
        }
    }

    /// Numeric severity for comparison. Higher = worse.
    /// OK/N/A = 0 (acceptable), WARN = 1, FAIL = 2.
    #[must_use]
    pub fn severity(&self) -> u8 {
        match self {
            Self::Ok | Self::NotApplicable => 0,
            Self::Warn => 1,
            Self::Fail => 2,
        }
    }

    /// Parse a user-supplied level string (case-insensitive).
    /// Accepts: "warn"/"warning" → Warn; "fail"/"error" → Fail.
    #[must_use]
    pub fn parse_level(s: &str) -> Option<FalsifierStatus> {
        match s.to_ascii_lowercase().as_str() {
            "warn" | "warning" => Some(Self::Warn),
            "fail" | "error" => Some(Self::Fail),
            _ => None,
        }
    }
}

/// §14.5 falsifier scorecard across the four tracked metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FalsifierScorecard {
    pub f1: FalsifierStatus,
    pub f2: FalsifierStatus,
    pub f4: FalsifierStatus,
    pub f11: FalsifierStatus,
}

impl FalsifierScorecard {
    /// True if any metric is in `Fail` state (falsifier range).
    #[must_use]
    pub fn any_fail(&self) -> bool {
        self.breaches_at(FalsifierStatus::Fail)
    }

    /// True if any metric is at-or-above `level` (severity-wise).
    /// `level=Warn` matches Warn+Fail. `level=Fail` matches Fail only.
    #[must_use]
    pub fn breaches_at(&self, level: FalsifierStatus) -> bool {
        [self.f1, self.f2, self.f4, self.f11]
            .iter()
            .any(|s| s.severity() >= level.severity())
    }

    /// Return (label, status) for each breached metric at-or-above `level`.
    #[must_use]
    pub fn breached_metrics(
        &self,
        level: FalsifierStatus,
    ) -> Vec<(&'static str, FalsifierStatus)> {
        let pairs = [
            ("F1", self.f1),
            ("F2", self.f2),
            ("F4", self.f4),
            ("F11", self.f11),
        ];
        pairs
            .into_iter()
            .filter(|(_, s)| s.severity() >= level.severity())
            .collect()
    }
}

/// Per-file tier breakdown, sortable by worst-tier distribution.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FileTierCounts {
    pub bronze: usize,
    pub silver: usize,
    pub gold: usize,
    pub platinum: usize,
}

impl FileTierCounts {
    #[must_use]
    pub fn total(&self) -> usize {
        self.bronze + self.silver + self.gold + self.platinum
    }

    fn bump(&mut self, tier: Tier) {
        match tier {
            Tier::Bronze => self.bronze += 1,
            Tier::Silver => self.silver += 1,
            Tier::Gold => self.gold += 1,
            Tier::Platinum => self.platinum += 1,
        }
    }
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

    /// Per-file tier counts. Returns a vector sorted by file path
    /// (ascending) for deterministic output.
    #[must_use]
    pub fn by_file(&self) -> Vec<(PathBuf, FileTierCounts)> {
        use std::collections::BTreeMap;
        let mut map: BTreeMap<PathBuf, FileTierCounts> = BTreeMap::new();
        for f in &self.functions {
            map.entry(f.file.clone()).or_default().bump(f.tier);
        }
        map.into_iter().collect()
    }

    /// Per-file tier counts, sorted by the given column.
    /// `sort_by` is one of: `file` (ascending), `bronze`/`silver`/`gold`/
    /// `platinum`/`total` (descending, ties broken by file path). Unknown
    /// values fall back to `file` (for user typos — not a hard error).
    /// `top` limits the output to N entries after sorting.
    #[must_use]
    pub fn by_file_sorted(
        &self,
        sort_by: &str,
        top: Option<usize>,
    ) -> Vec<(PathBuf, FileTierCounts)> {
        let mut rows = self.by_file();
        // Tie-breaker: file path ascending (already BTreeMap-ordered).
        match sort_by {
            "bronze" => rows.sort_by(|a, b| b.1.bronze.cmp(&a.1.bronze).then(a.0.cmp(&b.0))),
            "silver" => rows.sort_by(|a, b| b.1.silver.cmp(&a.1.silver).then(a.0.cmp(&b.0))),
            "gold" => rows.sort_by(|a, b| b.1.gold.cmp(&a.1.gold).then(a.0.cmp(&b.0))),
            "platinum" => rows.sort_by(|a, b| b.1.platinum.cmp(&a.1.platinum).then(a.0.cmp(&b.0))),
            "total" => rows.sort_by(|a, b| b.1.total().cmp(&a.1.total()).then(a.0.cmp(&b.0))),
            _ => {} // default: already sorted by path
        }
        if let Some(n) = top {
            rows.truncate(n);
        }
        rows
    }

    /// Emit a GitHub-flavored markdown report suitable for PR summaries.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let sc = self.falsifier_scorecard();
        let badge = |s: FalsifierStatus| -> &'static str {
            match s {
                FalsifierStatus::Ok => "🟢 OK",
                FalsifierStatus::Warn => "🟡 WARN",
                FalsifierStatus::Fail => "🔴 FAIL",
                FalsifierStatus::NotApplicable => "⚪ N/A",
            }
        };
        let mut out = String::new();
        out.push_str("## §14.5 Provability Tier Report\n\n");
        out.push_str(&format!(
            "**Files scanned:** {} ({} LoC, {} functions)\n\n",
            self.files_scanned, self.total_loc, self.functions_total
        ));
        // Tier distribution
        out.push_str("### Tier Distribution\n\n");
        out.push_str("| Tier | Count | % |\n|------|------:|--:|\n");
        let pct_of = |n: usize| -> f64 {
            if self.functions_total == 0 {
                0.0
            } else {
                (n as f64 / self.functions_total as f64) * 100.0
            }
        };
        out.push_str(&format!(
            "| Bronze   | {} | {:.1}% |\n",
            self.bronze,
            pct_of(self.bronze)
        ));
        out.push_str(&format!(
            "| Silver   | {} | {:.1}% |\n",
            self.silver,
            pct_of(self.silver)
        ));
        out.push_str(&format!(
            "| Gold     | {} | {:.1}% |\n",
            self.gold,
            pct_of(self.gold)
        ));
        out.push_str(&format!(
            "| Platinum | {} | {:.1}% |\n",
            self.platinum,
            pct_of(self.platinum)
        ));
        out.push_str(&format!("| **non-Bronze** | | **{:.1}%** |\n\n", self.non_bronze_pct()));
        // Falsifier scorecard
        out.push_str("### §14.5 Falsifier Scorecard\n\n");
        out.push_str("| Metric | Value | Status |\n|--------|------:|:------:|\n");
        out.push_str(&format!(
            "| F1 non-trivial % | {:.1}% | {} |\n",
            self.non_trivial_pct(),
            badge(sc.f1)
        ));
        out.push_str(&format!(
            "| F2 exempt / KLoC | {:.2} | {} |\n",
            self.exempt_density_per_kloc(),
            badge(sc.f2)
        ));
        out.push_str(&format!(
            "| F4 pub Bronze | {} | {} |\n",
            self.pub_bronze_count(),
            badge(sc.f4)
        ));
        out.push_str(&format!(
            "| F11 diff_exempt / KLoC | {:.2} | {} |\n\n",
            self.diff_exempt_density_per_kloc(),
            badge(sc.f11)
        ));
        // Parse health
        if self.parse_errors > 0 || self.parse_timeouts > 0 {
            out.push_str("### Parser Health\n\n");
            out.push_str(&format!("- Parse errors: {}\n", self.parse_errors));
            out.push_str(&format!("- Parse timeouts: {}\n\n", self.parse_timeouts));
        }
        out
    }

    /// Extract a baseline snapshot for regression tracking.
    #[must_use]
    pub fn baseline_snapshot(&self) -> BaselineSnapshot {
        BaselineSnapshot {
            bronze: self.bronze,
            silver: self.silver,
            gold: self.gold,
            platinum: self.platinum,
            pub_bronze: self.pub_bronze_count(),
            non_trivial_pct: self.non_trivial_pct(),
            non_bronze_pct: self.non_bronze_pct(),
            exempt_density_per_kloc: self.exempt_density_per_kloc(),
            diff_exempt_density_per_kloc: self.diff_exempt_density_per_kloc(),
            contract_exempt: self.contract_exempt_count,
            diff_exempt: self.diff_exempt_count,
            parse_errors: self.parse_errors,
            parse_timeouts: self.parse_timeouts,
            totality_violations: self.totality_violations().len(),
        }
    }

    /// §14.5 falsifier scorecard: OK / WARN / FAIL per metric.
    /// - OK: metric meets the 5.2 target
    /// - WARN: metric misses the target but is not in falsifier range
    /// - FAIL: metric is in falsifier range (refutes the claim)
    /// - N/A: metric cannot be evaluated (e.g., no contracts → no F1)
    #[must_use]
    pub fn falsifier_scorecard(&self) -> FalsifierScorecard {
        // F1: non_trivial_pct. Target ≥95% at 5.2, falsifies <50%.
        let f1 = if self.non_trivial_contracts + self.trivial_contracts == 0 {
            FalsifierStatus::NotApplicable
        } else {
            let pct = self.non_trivial_pct();
            if pct < 50.0 {
                FalsifierStatus::Fail
            } else if pct < 95.0 {
                FalsifierStatus::Warn
            } else {
                FalsifierStatus::Ok
            }
        };
        // F2: contract_exempt density/KLoC. Target ≤0.5, falsifies >5.0.
        let f2 = if self.total_loc == 0 {
            FalsifierStatus::NotApplicable
        } else {
            let d = self.exempt_density_per_kloc();
            if d > 5.0 {
                FalsifierStatus::Fail
            } else if d > 0.5 {
                FalsifierStatus::Warn
            } else {
                FalsifierStatus::Ok
            }
        };
        // F4: pub_bronze. Target 0 after 5.2.
        let f4 = match self.pub_bronze_count() {
            0 => FalsifierStatus::Ok,
            _ => FalsifierStatus::Warn, // Not Fail until 5.2
        };
        // F11: diff_exempt density/KLoC. Target 0, any > 0 is Warn.
        let f11 = if self.total_loc == 0 {
            FalsifierStatus::NotApplicable
        } else if self.diff_exempt_count == 0 {
            FalsifierStatus::Ok
        } else {
            FalsifierStatus::Warn
        };
        FalsifierScorecard { f1, f2, f4, f11 }
    }

    /// Emit per-file breakdown as a single-line JSON array of objects.
    #[must_use]
    pub fn by_file_to_json(&self) -> String {
        self.by_file_to_json_sorted("file", None)
    }

    /// Emit per-file breakdown as a single-line JSON array of objects,
    /// sorted and limited per `sort_by`/`top`.
    #[must_use]
    pub fn by_file_to_json_sorted(&self, sort_by: &str, top: Option<usize>) -> String {
        let mut out = String::from("[");
        for (i, (path, counts)) in self.by_file_sorted(sort_by, top).iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"file\":\"{}\",\"bronze\":{},\"silver\":{},\"gold\":{},\"platinum\":{},\"total\":{}}}",
                escape_json(&path.to_string_lossy()),
                counts.bronze,
                counts.silver,
                counts.gold,
                counts.platinum,
                counts.total(),
            ));
        }
        out.push(']');
        out
    }

    /// Emit the list of classified functions as a single-line JSON array.
    /// Intended for `--json --list`; dashboards can consume per-function
    /// records alongside the aggregate report.
    #[must_use]
    pub fn functions_to_json(&self) -> String {
        let mut out = String::from("[");
        for (i, f) in self.functions.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&f.to_json());
        }
        out.push(']');
        out
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
\"scorecard\":{{\"f1\":\"{}\",\"f2\":\"{}\",\"f4\":\"{}\",\"f11\":\"{}\"}},\
\"parse_errors\":{},\
\"parse_timeouts\":{}\
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
            self.falsifier_scorecard().f1.label(),
            self.falsifier_scorecard().f2.label(),
            self.falsifier_scorecard().f4.label(),
            self.falsifier_scorecard().f11.label(),
            self.parse_errors,
            self.parse_timeouts,
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
            parse_timeouts: self.parse_timeouts,
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
            "files: {}\nloc: {}\nfunctions: {}\n  bronze:   {} ({:.1}%)\n  silver:   {} ({:.1}%)\n  gold:     {} ({:.1}%)\n  platinum: {} ({:.1}%)\nnon-bronze: {:.1}%\ncontract triviality (F1):\n  non-trivial: {}\n  trivial:     {}\n  non-trivial %: {:.1}%\nexemptions (F2):\n  #[contract_exempt]: {}\n  density / KLoC:     {:.2}\ndiff exemptions (F11):\n  #[diff_exempt]: {}\n  density / KLoC: {:.2}\npublic API (F4 proxy):\n  pub Bronze: {}\ntotality:\n  @total:    {}\n  @partial:  {}\n  unmarked:  {}\nparse errors: {}\nparse timeouts: {}",
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
            self.parse_timeouts,
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

/// Parse `src` in a worker thread with a wall-clock timeout. Returns
/// `Some(expr)` on success, `None` if the parser errored, `Err(())` if
/// the parser exceeded `timeout_ms` milliseconds (treated as a hang /
/// parser infinite loop).
fn parse_with_timeout(
    src: &str,
    timeout_ms: u64,
) -> Result<Option<ruchy::Expr>, ()> {
    use std::sync::mpsc;
    use std::time::Duration;
    let (tx, rx) = mpsc::sync_channel::<Option<ruchy::Expr>>(1);
    // Clone src into owned String for the worker thread.
    let owned = src.to_string();
    std::thread::spawn(move || {
        let mut parser = Parser::new(&owned);
        let result = parser.parse().ok();
        // If receiver has hung up (timed out), this send fails silently.
        let _ = tx.send(result);
    });
    match rx.recv_timeout(Duration::from_millis(timeout_ms)) {
        Ok(maybe_expr) => Ok(maybe_expr),
        Err(_) => Err(()),
    }
}

fn classify_source_with_timeout(
    src: &str,
    file: &Path,
    report: &mut ProvabilityReport,
    timeout_ms: u64,
) {
    match parse_with_timeout(src, timeout_ms) {
        Ok(Some(expr)) => visit_expr(&expr, file, report),
        Ok(None) => report.parse_errors += 1,
        Err(()) => report.parse_timeouts += 1,
    }
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
/// Default per-file parse timeout is 5 seconds; see [`scan_with_timeout`].
pub fn scan(path: &Path) -> Result<ProvabilityReport> {
    scan_with_timeout(path, 5000)
}

/// Scan `path` (file or directory) with a per-file parse timeout in ms.
/// Files whose parser exceeds the timeout are counted in `parse_timeouts`
/// and skipped. Default via `scan()` is 5000ms.
pub fn scan_with_timeout(path: &Path, timeout_ms: u64) -> Result<ProvabilityReport> {
    let mut files = Vec::new();
    collect_ruchy_files(path, &mut files);
    let mut report = ProvabilityReport::default();
    for file in &files {
        let src = std::fs::read_to_string(file)
            .with_context(|| format!("reading {}", file.display()))?;
        report.files_scanned += 1;
        report.total_loc += src.lines().count();
        classify_source_with_timeout(&src, file, &mut report, timeout_ms);
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
    fail_diff_exempt_density_above: Option<f64>,
    by_file: bool,
    sort_by: &str,
    top: Option<usize>,
    parse_timeout_ms: u64,
    baseline: Option<&Path>,
    markdown: bool,
    fail_on_scorecard: Option<&str>,
) -> Result<()> {
    let raw = scan_with_timeout(path, parse_timeout_ms)?;
    let report = if public_only { raw.filter_to_pub() } else { raw };
    if markdown {
        print!("{}", report.to_markdown());
    } else if json {
        println!("{}", report.to_json());
        if list {
            println!("{}", report.functions_to_json());
        }
        if by_file {
            println!("{}", report.by_file_to_json_sorted(sort_by, top));
        }
    } else {
        println!("Provability tier scan: {}", path.display());
        println!("{}", report.summary());
        let sc = report.falsifier_scorecard();
        println!(
            "§14.5 scorecard: F1:{} F2:{} F4:{} F11:{}",
            sc.f1.label(),
            sc.f2.label(),
            sc.f4.label(),
            sc.f11.label()
        );
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
        if by_file {
            println!("\nper-file tier breakdown:");
            println!(
                "  {:<6} {:<6} {:<6} {:<8} {:<6}  {}",
                "bronze", "silver", "gold", "platinum", "total", "file"
            );
            for (path, c) in report.by_file_sorted(sort_by, top) {
                println!(
                    "  {:<6} {:<6} {:<6} {:<8} {:<6}  {}",
                    c.bronze,
                    c.silver,
                    c.gold,
                    c.platinum,
                    c.total(),
                    path.display()
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
    // Apply --fail-diff-exempt-density-above gate (§14.5 F11 CI enforcement).
    if let Some(ceiling) = fail_diff_exempt_density_above {
        if report.total_loc > 0 {
            let actual = report.diff_exempt_density_per_kloc();
            if actual > ceiling {
                anyhow::bail!(
                    "#[diff_exempt] density {:.2}/KLoC exceeds ceiling {:.2}/KLoC (§14.5 F11 breach)",
                    actual,
                    ceiling
                );
            }
        }
    }
    // Apply --fail-on-scorecard gate (§14.5 scorecard CI enforcement).
    if let Some(level_str) = fail_on_scorecard {
        let level = FalsifierStatus::parse_level(level_str).ok_or_else(|| {
            anyhow::anyhow!(
                "invalid --fail-on-scorecard level `{}`: expected `warn` or `fail`",
                level_str
            )
        })?;
        let sc = report.falsifier_scorecard();
        let breaches = sc.breached_metrics(level);
        if !breaches.is_empty() {
            let summary: Vec<String> = breaches
                .iter()
                .map(|(k, s)| format!("{k}:{}", s.label()))
                .collect();
            anyhow::bail!(
                "§14.5 scorecard breach at level={}: {}",
                level.label(),
                summary.join(" ")
            );
        }
    }
    // Baseline comparison (§14.5 regression gate).
    if let Some(baseline_path) = baseline {
        let current_snap = report.baseline_snapshot();
        if baseline_path.exists() {
            let raw = std::fs::read_to_string(baseline_path)
                .with_context(|| format!("reading baseline {}", baseline_path.display()))?;
            let baseline_snap: BaselineSnapshot = serde_json::from_str(&raw)
                .with_context(|| format!("parsing baseline {}", baseline_path.display()))?;
            let regs = baseline_snap.regressions_vs(&current_snap);
            if regs.is_empty() {
                eprintln!("baseline OK: no regressions vs {}", baseline_path.display());
            } else {
                eprintln!("\nbaseline regressions vs {}:", baseline_path.display());
                for r in &regs {
                    eprintln!("  {} : {} → {}", r.metric, r.baseline, r.current);
                }
                anyhow::bail!("{} baseline regression(s) detected", regs.len());
            }
        } else {
            // First run: capture baseline.
            let pretty = serde_json::to_string_pretty(&current_snap)
                .with_context(|| "serializing baseline")?;
            std::fs::write(baseline_path, pretty)
                .with_context(|| format!("writing baseline {}", baseline_path.display()))?;
            eprintln!("baseline captured: {}", baseline_path.display());
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
    fn test_falsifier_scorecard_empty_report() {
        let r = ProvabilityReport::default();
        let sc = r.falsifier_scorecard();
        // No contracts → F1 N/A. No LoC → F2/F11 N/A. No pub Bronze → F4 OK.
        assert_eq!(sc.f1, FalsifierStatus::NotApplicable);
        assert_eq!(sc.f2, FalsifierStatus::NotApplicable);
        assert_eq!(sc.f4, FalsifierStatus::Ok);
        assert_eq!(sc.f11, FalsifierStatus::NotApplicable);
        assert!(!sc.any_fail());
    }

    #[test]
    fn test_falsifier_scorecard_f1_fail_below_50pct() {
        let mut r = ProvabilityReport::default();
        // 1 non-trivial + 9 trivial = 10% non-trivial → F1 FAIL
        r.non_trivial_contracts = 1;
        r.trivial_contracts = 9;
        let sc = r.falsifier_scorecard();
        assert_eq!(sc.f1, FalsifierStatus::Fail);
        assert!(sc.any_fail());
    }

    #[test]
    fn test_falsifier_scorecard_f1_warn_between_50_and_95() {
        let mut r = ProvabilityReport::default();
        r.non_trivial_contracts = 7;
        r.trivial_contracts = 3; // 70%
        let sc = r.falsifier_scorecard();
        assert_eq!(sc.f1, FalsifierStatus::Warn);
        assert!(!sc.any_fail());
    }

    #[test]
    fn test_falsifier_scorecard_f1_ok_at_95pct_or_above() {
        let mut r = ProvabilityReport::default();
        r.non_trivial_contracts = 19;
        r.trivial_contracts = 1; // 95%
        let sc = r.falsifier_scorecard();
        assert_eq!(sc.f1, FalsifierStatus::Ok);
    }

    #[test]
    fn test_falsifier_scorecard_f2_fail_above_5_per_kloc() {
        let mut r = ProvabilityReport::default();
        r.total_loc = 100;
        r.contract_exempt_count = 1; // 10/KLoC
        let sc = r.falsifier_scorecard();
        assert_eq!(sc.f2, FalsifierStatus::Fail);
    }

    #[test]
    fn test_falsifier_scorecard_f2_warn_between_half_and_5() {
        let mut r = ProvabilityReport::default();
        r.total_loc = 1000;
        r.contract_exempt_count = 1; // 1.0/KLoC
        let sc = r.falsifier_scorecard();
        assert_eq!(sc.f2, FalsifierStatus::Warn);
    }

    #[test]
    fn test_falsifier_scorecard_f4_warn_on_pub_bronze() {
        let mut r = ProvabilityReport::default();
        classify_source("pub fun a() { 1 }", Path::new("t.ruchy"), &mut r);
        let sc = r.falsifier_scorecard();
        assert_eq!(sc.f4, FalsifierStatus::Warn);
    }

    #[test]
    fn test_falsifier_scorecard_f11_warn_on_any_diff_exempt() {
        let mut r = ProvabilityReport::default();
        r.total_loc = 1000;
        r.diff_exempt_count = 1;
        let sc = r.falsifier_scorecard();
        assert_eq!(sc.f11, FalsifierStatus::Warn);
    }

    #[test]
    fn test_falsifier_status_labels() {
        assert_eq!(FalsifierStatus::Ok.label(), "OK");
        assert_eq!(FalsifierStatus::Warn.label(), "WARN");
        assert_eq!(FalsifierStatus::Fail.label(), "FAIL");
        assert_eq!(FalsifierStatus::NotApplicable.label(), "N/A");
    }

    #[test]
    fn test_to_json_includes_scorecard() {
        let r = ProvabilityReport::default();
        let j = r.to_json();
        assert!(j.contains("\"scorecard\""));
        assert!(j.contains("\"f1\":"));
        assert!(j.contains("\"f2\":"));
        assert!(j.contains("\"f4\":"));
        assert!(j.contains("\"f11\":"));
    }

    #[test]
    fn test_falsifier_status_severity_ordering() {
        assert!(FalsifierStatus::Fail.severity() > FalsifierStatus::Warn.severity());
        assert!(FalsifierStatus::Warn.severity() > FalsifierStatus::Ok.severity());
        assert_eq!(
            FalsifierStatus::NotApplicable.severity(),
            FalsifierStatus::Ok.severity()
        );
    }

    #[test]
    fn test_parse_level_accepts_common_spellings() {
        assert_eq!(
            FalsifierStatus::parse_level("warn"),
            Some(FalsifierStatus::Warn)
        );
        assert_eq!(
            FalsifierStatus::parse_level("WARN"),
            Some(FalsifierStatus::Warn)
        );
        assert_eq!(
            FalsifierStatus::parse_level("warning"),
            Some(FalsifierStatus::Warn)
        );
        assert_eq!(
            FalsifierStatus::parse_level("fail"),
            Some(FalsifierStatus::Fail)
        );
        assert_eq!(
            FalsifierStatus::parse_level("error"),
            Some(FalsifierStatus::Fail)
        );
        assert_eq!(FalsifierStatus::parse_level("bogus"), None);
        assert_eq!(FalsifierStatus::parse_level(""), None);
    }

    #[test]
    fn test_scorecard_breaches_at_warn_catches_warn_and_fail() {
        let sc = FalsifierScorecard {
            f1: FalsifierStatus::Ok,
            f2: FalsifierStatus::Warn,
            f4: FalsifierStatus::Ok,
            f11: FalsifierStatus::Ok,
        };
        assert!(sc.breaches_at(FalsifierStatus::Warn));
        assert!(!sc.breaches_at(FalsifierStatus::Fail));
    }

    #[test]
    fn test_scorecard_breaches_at_fail_ignores_warn() {
        let sc = FalsifierScorecard {
            f1: FalsifierStatus::Warn,
            f2: FalsifierStatus::Warn,
            f4: FalsifierStatus::Ok,
            f11: FalsifierStatus::Ok,
        };
        // No metric at Fail → gate at Fail passes.
        assert!(!sc.breaches_at(FalsifierStatus::Fail));
    }

    #[test]
    fn test_scorecard_breached_metrics_returns_only_breaches() {
        let sc = FalsifierScorecard {
            f1: FalsifierStatus::Fail,
            f2: FalsifierStatus::Warn,
            f4: FalsifierStatus::Ok,
            f11: FalsifierStatus::Ok,
        };
        let w = sc.breached_metrics(FalsifierStatus::Warn);
        assert_eq!(w.len(), 2); // F1+F2
        let f = sc.breached_metrics(FalsifierStatus::Fail);
        assert_eq!(f.len(), 1); // F1 only
        assert_eq!(f[0].0, "F1");
    }

    #[test]
    fn test_scorecard_breached_metrics_clean_scorecard_empty() {
        let sc = FalsifierScorecard {
            f1: FalsifierStatus::Ok,
            f2: FalsifierStatus::NotApplicable,
            f4: FalsifierStatus::Ok,
            f11: FalsifierStatus::Ok,
        };
        assert!(sc.breached_metrics(FalsifierStatus::Warn).is_empty());
    }

    #[test]
    fn test_markdown_contains_header_and_tables() {
        let mut r = ProvabilityReport::default();
        classify_source(
            "pub fun a() { 1 }\nfun b() requires x > 0 { 2 }",
            Path::new("t.ruchy"),
            &mut r,
        );
        r.total_loc = 100;
        let md = r.to_markdown();
        assert!(md.contains("## §14.5 Provability Tier Report"));
        assert!(md.contains("### Tier Distribution"));
        assert!(md.contains("### §14.5 Falsifier Scorecard"));
        assert!(md.contains("| Bronze"));
        assert!(md.contains("| F1 non-trivial %"));
    }

    #[test]
    fn test_markdown_uses_status_badges() {
        let mut r = ProvabilityReport::default();
        classify_source("pub fun a() { 1 }", Path::new("t.ruchy"), &mut r);
        r.total_loc = 100;
        let md = r.to_markdown();
        // 1 pub Bronze → F4 should be WARN (yellow circle)
        assert!(md.contains("🟡 WARN"));
        // F2/F11 should be OK (green)
        assert!(md.contains("🟢 OK"));
    }

    #[test]
    fn test_markdown_shows_parser_health_section_only_when_nonzero() {
        let r_clean = ProvabilityReport::default();
        let md_clean = r_clean.to_markdown();
        assert!(!md_clean.contains("### Parser Health"));

        let r_bad = ProvabilityReport {
            parse_errors: 1,
            ..Default::default()
        };
        let md_bad = r_bad.to_markdown();
        assert!(md_bad.contains("### Parser Health"));
        assert!(md_bad.contains("Parse errors: 1"));
    }

    #[test]
    fn test_baseline_snapshot_captures_current_metrics() {
        let mut r = ProvabilityReport::default();
        classify_source("pub fun a() { 1 }", Path::new("t.ruchy"), &mut r);
        r.total_loc = 100;
        r.contract_exempt_count = 2;
        let s = r.baseline_snapshot();
        assert_eq!(s.bronze, 1);
        assert_eq!(s.pub_bronze, 1);
        assert_eq!(s.contract_exempt, 2);
    }

    #[test]
    fn test_regressions_vs_empty_when_identical() {
        let s = BaselineSnapshot {
            bronze: 3,
            silver: 2,
            gold: 0,
            platinum: 0,
            pub_bronze: 1,
            non_trivial_pct: 95.0,
            non_bronze_pct: 40.0,
            exempt_density_per_kloc: 0.2,
            diff_exempt_density_per_kloc: 0.0,
            contract_exempt: 1,
            diff_exempt: 0,
            parse_errors: 0,
            parse_timeouts: 0,
            totality_violations: 0,
        };
        assert!(s.regressions_vs(&s).is_empty());
    }

    #[test]
    fn test_regressions_vs_detects_bronze_increase() {
        let base = BaselineSnapshot {
            bronze: 3,
            silver: 0, gold: 0, platinum: 0, pub_bronze: 0,
            non_trivial_pct: 0.0, non_bronze_pct: 0.0,
            exempt_density_per_kloc: 0.0, diff_exempt_density_per_kloc: 0.0,
            contract_exempt: 0, diff_exempt: 0,
            parse_errors: 0, parse_timeouts: 0, totality_violations: 0,
        };
        let cur = BaselineSnapshot { bronze: 5, ..base };
        let regs = base.regressions_vs(&cur);
        assert_eq!(regs.len(), 1);
        assert_eq!(regs[0].metric, "bronze");
        assert_eq!(regs[0].baseline, "3");
        assert_eq!(regs[0].current, "5");
    }

    #[test]
    fn test_regressions_vs_detects_pct_decrease() {
        let base = BaselineSnapshot {
            bronze: 0, silver: 0, gold: 0, platinum: 0, pub_bronze: 0,
            non_trivial_pct: 95.0, non_bronze_pct: 0.0,
            exempt_density_per_kloc: 0.0, diff_exempt_density_per_kloc: 0.0,
            contract_exempt: 0, diff_exempt: 0,
            parse_errors: 0, parse_timeouts: 0, totality_violations: 0,
        };
        let cur = BaselineSnapshot { non_trivial_pct: 80.0, ..base };
        let regs = base.regressions_vs(&cur);
        assert_eq!(regs.len(), 1);
        assert_eq!(regs[0].metric, "non_trivial_pct");
    }

    #[test]
    fn test_regressions_vs_ignores_tiny_pct_noise() {
        let base = BaselineSnapshot {
            bronze: 0, silver: 0, gold: 0, platinum: 0, pub_bronze: 0,
            non_trivial_pct: 95.0, non_bronze_pct: 0.0,
            exempt_density_per_kloc: 0.0, diff_exempt_density_per_kloc: 0.0,
            contract_exempt: 0, diff_exempt: 0,
            parse_errors: 0, parse_timeouts: 0, totality_violations: 0,
        };
        // Within 0.01 tolerance → no regression.
        let cur = BaselineSnapshot { non_trivial_pct: 94.995, ..base };
        assert!(base.regressions_vs(&cur).is_empty());
    }

    #[test]
    fn test_regressions_vs_improvement_not_regression() {
        let base = BaselineSnapshot {
            bronze: 10, silver: 0, gold: 0, platinum: 0, pub_bronze: 3,
            non_trivial_pct: 50.0, non_bronze_pct: 20.0,
            exempt_density_per_kloc: 2.0, diff_exempt_density_per_kloc: 0.0,
            contract_exempt: 5, diff_exempt: 0,
            parse_errors: 2, parse_timeouts: 0, totality_violations: 1,
        };
        // Everything is improved.
        let cur = BaselineSnapshot {
            bronze: 5, pub_bronze: 0, non_trivial_pct: 90.0,
            non_bronze_pct: 60.0, exempt_density_per_kloc: 0.5,
            contract_exempt: 1, parse_errors: 0, totality_violations: 0,
            ..base
        };
        assert!(base.regressions_vs(&cur).is_empty());
    }

    #[test]
    fn test_baseline_snapshot_roundtrips_via_serde() {
        let s = BaselineSnapshot {
            bronze: 5, silver: 3, gold: 1, platinum: 0, pub_bronze: 2,
            non_trivial_pct: 87.5, non_bronze_pct: 44.4,
            exempt_density_per_kloc: 1.2, diff_exempt_density_per_kloc: 0.0,
            contract_exempt: 3, diff_exempt: 0,
            parse_errors: 1, parse_timeouts: 0, totality_violations: 0,
        };
        let json = serde_json::to_string(&s).unwrap();
        let parsed: BaselineSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(s, parsed);
    }

    #[test]
    fn test_parse_with_timeout_succeeds_on_valid_source() {
        let r = parse_with_timeout("fun a() { 1 }", 5000);
        assert!(matches!(r, Ok(Some(_))));
    }

    #[test]
    fn test_parse_with_timeout_none_on_invalid_source() {
        let r = parse_with_timeout("fun (", 5000);
        assert!(matches!(r, Ok(None)));
    }

    #[test]
    fn test_scan_single_file_with_default_timeout_works() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("t.ruchy");
        fs::write(&p, "fun a() { 1 }").unwrap();
        let r = scan(&p).unwrap();
        assert_eq!(r.parse_timeouts, 0);
        assert_eq!(r.functions_total, 1);
    }

    #[test]
    fn test_to_json_includes_parse_timeouts_key() {
        let r = ProvabilityReport::default();
        let j = r.to_json();
        assert!(j.contains("\"parse_timeouts\":0"));
    }

    #[test]
    fn test_summary_includes_parse_timeouts() {
        let r = ProvabilityReport::default();
        let s = r.summary();
        assert!(s.contains("parse timeouts:"));
    }

    #[test]
    fn test_by_file_groups_functions_by_path() {
        let mut r = ProvabilityReport::default();
        classify_source("pub fun a() { 1 }\nfun b() { 2 }", Path::new("x.ruchy"), &mut r);
        classify_source("fun c() requires x > 0 { 3 }", Path::new("y.ruchy"), &mut r);
        let bf = r.by_file();
        assert_eq!(bf.len(), 2);
        // Sorted by path: x.ruchy before y.ruchy
        assert_eq!(bf[0].0, PathBuf::from("x.ruchy"));
        assert_eq!(bf[0].1.bronze, 2);
        assert_eq!(bf[0].1.silver, 0);
        assert_eq!(bf[0].1.total(), 2);
        assert_eq!(bf[1].0, PathBuf::from("y.ruchy"));
        assert_eq!(bf[1].1.silver, 1);
        assert_eq!(bf[1].1.total(), 1);
    }

    #[test]
    fn test_by_file_empty_returns_empty_vec() {
        let r = ProvabilityReport::default();
        assert!(r.by_file().is_empty());
    }

    #[test]
    fn test_by_file_to_json_produces_valid_array() {
        let mut r = ProvabilityReport::default();
        classify_source("fun a() { 1 }", Path::new("t.ruchy"), &mut r);
        let j = r.by_file_to_json();
        assert!(j.starts_with('['));
        assert!(j.ends_with(']'));
        assert!(j.contains("\"file\":\"t.ruchy\""));
        assert!(j.contains("\"bronze\":1"));
        assert!(j.contains("\"total\":1"));
    }

    #[test]
    fn test_by_file_sorted_by_bronze_descending() {
        let mut r = ProvabilityReport::default();
        // a.ruchy: 1 Bronze, b.ruchy: 3 Bronze
        classify_source("fun x() { 1 }", Path::new("a.ruchy"), &mut r);
        classify_source(
            "fun p() { 1 }\nfun q() { 2 }\nfun r() { 3 }",
            Path::new("b.ruchy"),
            &mut r,
        );
        let sorted = r.by_file_sorted("bronze", None);
        assert_eq!(sorted[0].0, PathBuf::from("b.ruchy"));
        assert_eq!(sorted[0].1.bronze, 3);
        assert_eq!(sorted[1].0, PathBuf::from("a.ruchy"));
    }

    #[test]
    fn test_by_file_sorted_top_n_truncates() {
        let mut r = ProvabilityReport::default();
        classify_source("fun x() { 1 }", Path::new("a.ruchy"), &mut r);
        classify_source("fun y() { 2 }", Path::new("b.ruchy"), &mut r);
        classify_source("fun z() { 3 }", Path::new("c.ruchy"), &mut r);
        let sorted = r.by_file_sorted("file", Some(2));
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].0, PathBuf::from("a.ruchy"));
        assert_eq!(sorted[1].0, PathBuf::from("b.ruchy"));
    }

    #[test]
    fn test_by_file_sorted_unknown_key_falls_back_to_file() {
        let mut r = ProvabilityReport::default();
        classify_source("fun y() { 1 }", Path::new("z.ruchy"), &mut r);
        classify_source("fun x() { 1 }", Path::new("a.ruchy"), &mut r);
        let sorted = r.by_file_sorted("typo_column", None);
        // Falls back to default = file ascending
        assert_eq!(sorted[0].0, PathBuf::from("a.ruchy"));
        assert_eq!(sorted[1].0, PathBuf::from("z.ruchy"));
    }

    #[test]
    fn test_by_file_sorted_ties_broken_by_file_path() {
        let mut r = ProvabilityReport::default();
        // Both have 1 Bronze; tie-breaker is path ascending
        classify_source("fun x() { 1 }", Path::new("z.ruchy"), &mut r);
        classify_source("fun y() { 1 }", Path::new("a.ruchy"), &mut r);
        let sorted = r.by_file_sorted("bronze", None);
        assert_eq!(sorted[0].0, PathBuf::from("a.ruchy"));
        assert_eq!(sorted[1].0, PathBuf::from("z.ruchy"));
    }

    #[test]
    fn test_by_file_to_json_empty_is_empty_array() {
        let r = ProvabilityReport::default();
        assert_eq!(r.by_file_to_json(), "[]");
    }

    #[test]
    fn test_file_tier_counts_bump_and_total() {
        let mut c = FileTierCounts::default();
        c.bump(Tier::Bronze);
        c.bump(Tier::Silver);
        c.bump(Tier::Silver);
        c.bump(Tier::Gold);
        assert_eq!(c.bronze, 1);
        assert_eq!(c.silver, 2);
        assert_eq!(c.gold, 1);
        assert_eq!(c.platinum, 0);
        assert_eq!(c.total(), 4);
    }

    #[test]
    fn test_classified_function_to_json_has_all_keys() {
        let cf = ClassifiedFunction {
            file: PathBuf::from("src/a.ruchy"),
            name: "foo".into(),
            tier: Tier::Silver,
            totality: Totality::Total,
            has_non_trivial_contract: true,
            is_pub: true,
        };
        let j = cf.to_json();
        for key in ["name", "file", "tier", "totality", "pub", "non_trivial_contract"] {
            assert!(j.contains(key), "missing key `{key}`: {j}");
        }
        assert!(j.contains("\"pub\":true"));
        assert!(j.contains("\"non_trivial_contract\":true"));
        assert!(j.contains("\"tier\":\"silver\""));
    }

    #[test]
    fn test_classified_function_to_json_escapes_quotes() {
        let cf = ClassifiedFunction {
            file: PathBuf::from("src/with\"quote.ruchy"),
            name: "bad\"name".into(),
            tier: Tier::Bronze,
            totality: Totality::Unknown,
            has_non_trivial_contract: false,
            is_pub: false,
        };
        let j = cf.to_json();
        // Quotes must be escaped.
        assert!(j.contains("bad\\\"name"), "json: {j}");
        assert!(j.contains("with\\\"quote"), "json: {j}");
    }

    #[test]
    fn test_functions_to_json_empty_is_empty_array() {
        let r = ProvabilityReport::default();
        assert_eq!(r.functions_to_json(), "[]");
    }

    #[test]
    fn test_functions_to_json_commas_between_entries() {
        let mut r = ProvabilityReport::default();
        classify_source(
            "pub fun a() { 1 }\nfun b() { 2 }",
            Path::new("t.ruchy"),
            &mut r,
        );
        let j = r.functions_to_json();
        assert!(j.starts_with('['));
        assert!(j.ends_with(']'));
        // Two function entries → one comma separator.
        assert_eq!(j.matches("},{").count(), 1);
        assert!(j.contains("\"name\":\"a\""));
        assert!(j.contains("\"name\":\"b\""));
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
