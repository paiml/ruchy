//! Spectrum-Based Fault Localization (SBFL)
//!
//! Implements fault localization algorithms from academic research:
//! - [1] Jones et al. (2002). Tarantula algorithm. ICSE '02.
//! - [2] Abreu et al. (2007). Ochiai formula. TAICPART-MUTATION '07.
//! - [3] Wong et al. (2014). D* method. IEEE TSE.

/// SBFL formula selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SbflFormula {
    /// Tarantula algorithm [1]
    /// suspiciousness = (`failed_cover/total_failed`) /
    ///                  ((`failed_cover/total_failed`) + (`passed_cover/total_passed`))
    #[default]
    Tarantula,

    /// Ochiai formula [2] - outperforms Tarantula in 75% of cases
    /// suspiciousness = `failed_cover` / `sqrt(total_failed` * (`failed_cover` + `passed_cover`))
    Ochiai,

    /// Jaccard similarity coefficient
    /// suspiciousness = `failed_cover` / (`total_failed` + `passed_cover`)
    Jaccard,

    /// Wong-II formula
    /// suspiciousness = `failed_cover` - `passed_cover`
    WongII,

    /// D* method with exponential weighting [3]
    /// suspiciousness = (`failed_cover`)^* / (`passed_cover` + (`total_failed` - `failed_cover`))
    DStar(u32),
}

impl SbflFormula {
    /// Get display name
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Tarantula => "Tarantula",
            Self::Ochiai => "Ochiai",
            Self::Jaccard => "Jaccard",
            Self::WongII => "Wong-II",
            Self::DStar(_) => "D*",
        }
    }

    /// Get academic reference
    #[must_use]
    pub fn reference(&self) -> &'static str {
        match self {
            Self::Tarantula => "[1] Jones et al., ICSE 2002",
            Self::Ochiai => "[2] Abreu et al., TAICPART-MUTATION 2007",
            Self::Jaccard => "Jaccard similarity coefficient",
            Self::WongII => "Wong et al.",
            Self::DStar(_) => "[3] Wong et al., IEEE TSE 2014",
        }
    }
}

/// Spectrum coverage data for a code location
#[derive(Debug, Clone, Default)]
pub struct SpectrumData {
    /// Number of passing tests that cover this location
    pub passed_covering: usize,
    /// Number of failing tests that cover this location
    pub failed_covering: usize,
    /// Number of passing tests that don't cover this location
    pub passed_not_covering: usize,
    /// Number of failing tests that don't cover this location
    pub failed_not_covering: usize,
}

impl SpectrumData {
    /// Create new spectrum data
    #[must_use]
    pub fn new(
        passed_covering: usize,
        failed_covering: usize,
        passed_not_covering: usize,
        failed_not_covering: usize,
    ) -> Self {
        Self {
            passed_covering,
            failed_covering,
            passed_not_covering,
            failed_not_covering,
        }
    }

    /// Total passing tests
    #[must_use]
    pub fn total_passed(&self) -> usize {
        self.passed_covering + self.passed_not_covering
    }

    /// Total failing tests
    #[must_use]
    pub fn total_failed(&self) -> usize {
        self.failed_covering + self.failed_not_covering
    }

    /// Calculate suspiciousness using given formula
    #[must_use]
    pub fn suspiciousness(&self, formula: SbflFormula) -> f64 {
        let ef = self.failed_covering as f64;
        let ep = self.passed_covering as f64;
        let nf = self.failed_not_covering as f64;
        let _np = self.passed_not_covering as f64;

        let total_failed = ef + nf;
        let total_passed = ep + self.passed_not_covering as f64;

        match formula {
            SbflFormula::Tarantula => {
                if total_failed == 0.0 || (ef == 0.0 && ep == 0.0) {
                    return 0.0;
                }
                let failed_ratio = ef / total_failed;
                let passed_ratio = if total_passed == 0.0 {
                    0.0
                } else {
                    ep / total_passed
                };
                if failed_ratio + passed_ratio == 0.0 {
                    0.0
                } else {
                    failed_ratio / (failed_ratio + passed_ratio)
                }
            }

            SbflFormula::Ochiai => {
                let denominator = (total_failed * (ef + ep)).sqrt();
                if denominator == 0.0 {
                    0.0
                } else {
                    ef / denominator
                }
            }

            SbflFormula::Jaccard => {
                let denominator = total_failed + ep;
                if denominator == 0.0 {
                    0.0
                } else {
                    ef / denominator
                }
            }

            SbflFormula::WongII => {
                // Can be negative, normalize to 0-1 range later
                ef - ep
            }

            SbflFormula::DStar(star) => {
                let numerator = ef.powi(star as i32);
                let denominator = ep + nf;
                if denominator == 0.0 {
                    if numerator > 0.0 {
                        f64::INFINITY
                    } else {
                        0.0
                    }
                } else {
                    numerator / denominator
                }
            }
        }
    }
}

/// Suspiciousness score with location
#[derive(Debug, Clone)]
pub struct SuspiciousnessScore {
    /// Location identifier (<file:line> or expression ID)
    pub location: String,
    /// Suspiciousness score (0.0 to 1.0 for most formulas)
    pub score: f64,
    /// Rank (1 = most suspicious)
    pub rank: usize,
}

impl SuspiciousnessScore {
    /// Check if highly suspicious (>0.7)
    #[must_use]
    pub fn is_high(&self) -> bool {
        self.score > 0.7
    }

    /// Check if moderately suspicious (0.3-0.7)
    #[must_use]
    pub fn is_moderate(&self) -> bool {
        self.score > 0.3 && self.score <= 0.7
    }

    /// Check if low suspicion (≤0.3)
    #[must_use]
    pub fn is_low(&self) -> bool {
        self.score <= 0.3
    }
}

/// SBFL ranking result
#[derive(Debug, Clone)]
pub struct SbflRanking {
    /// Formula used
    pub formula: SbflFormula,
    /// Ranked scores (most suspicious first)
    pub scores: Vec<SuspiciousnessScore>,
}

impl SbflRanking {
    /// Create ranking from spectrum data
    #[must_use]
    pub fn rank(locations: &[(String, SpectrumData)], formula: SbflFormula) -> Self {
        let mut scores: Vec<SuspiciousnessScore> = locations
            .iter()
            .map(|(loc, data)| SuspiciousnessScore {
                location: loc.clone(),
                score: data.suspiciousness(formula),
                rank: 0,
            })
            .collect();

        // Sort by score descending
        scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Assign ranks
        for (i, score) in scores.iter_mut().enumerate() {
            score.rank = i + 1;
        }

        Self { formula, scores }
    }

    /// Get top N suspicious locations
    #[must_use]
    pub fn top_n(&self, n: usize) -> &[SuspiciousnessScore] {
        &self.scores[..n.min(self.scores.len())]
    }

    /// Get highly suspicious locations
    #[must_use]
    pub fn high_suspicion(&self) -> Vec<&SuspiciousnessScore> {
        self.scores.iter().filter(|s| s.is_high()).collect()
    }

    /// Get moderate suspicion locations
    #[must_use]
    pub fn moderate_suspicion(&self) -> Vec<&SuspiciousnessScore> {
        self.scores.iter().filter(|s| s.is_moderate()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // EXTREME TDD: RED PHASE - SbflFormula Tests
    // ============================================================

    #[test]
    fn test_formula_default() {
        assert_eq!(SbflFormula::default(), SbflFormula::Tarantula);
    }

    #[test]
    fn test_formula_names() {
        assert_eq!(SbflFormula::Tarantula.name(), "Tarantula");
        assert_eq!(SbflFormula::Ochiai.name(), "Ochiai");
        assert_eq!(SbflFormula::Jaccard.name(), "Jaccard");
        assert_eq!(SbflFormula::WongII.name(), "Wong-II");
        assert_eq!(SbflFormula::DStar(2).name(), "D*");
    }

    #[test]
    fn test_formula_references() {
        assert!(SbflFormula::Tarantula.reference().contains("Jones"));
        assert!(SbflFormula::Ochiai.reference().contains("Abreu"));
        assert!(SbflFormula::DStar(2).reference().contains("Wong"));
    }

    // ============================================================
    // EXTREME TDD: RED PHASE - SpectrumData Tests
    // ============================================================

    #[test]
    fn test_spectrum_data_new() {
        let data = SpectrumData::new(10, 5, 90, 5);
        assert_eq!(data.passed_covering, 10);
        assert_eq!(data.failed_covering, 5);
        assert_eq!(data.total_passed(), 100);
        assert_eq!(data.total_failed(), 10);
    }

    #[test]
    fn test_tarantula_basic() {
        // Location covered by 5 failing, 10 passing
        // Total: 10 failing, 100 passing
        let data = SpectrumData::new(10, 5, 90, 5);
        let score = data.suspiciousness(SbflFormula::Tarantula);

        // failed_ratio = 5/10 = 0.5
        // passed_ratio = 10/100 = 0.1
        // suspiciousness = 0.5 / (0.5 + 0.1) = 0.833...
        assert!((score - 0.833).abs() < 0.01);
    }

    #[test]
    fn test_tarantula_only_failing() {
        // Location only covered by failing tests
        let data = SpectrumData::new(0, 10, 100, 0);
        let score = data.suspiciousness(SbflFormula::Tarantula);
        assert!((score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_tarantula_only_passing() {
        // Location only covered by passing tests
        let data = SpectrumData::new(100, 0, 0, 10);
        let score = data.suspiciousness(SbflFormula::Tarantula);
        assert!((score - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_ochiai_basic() {
        let data = SpectrumData::new(10, 5, 90, 5);
        let score = data.suspiciousness(SbflFormula::Ochiai);

        // ef = 5, ep = 10, total_failed = 10
        // sqrt(10 * (5 + 10)) = sqrt(150) = 12.25
        // score = 5 / 12.25 = 0.408
        assert!((score - 0.408).abs() < 0.01);
    }

    #[test]
    fn test_ochiai_perfect_fault() {
        // Only covered by failing tests
        let data = SpectrumData::new(0, 10, 100, 0);
        let score = data.suspiciousness(SbflFormula::Ochiai);
        assert!((score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_jaccard_basic() {
        let data = SpectrumData::new(10, 5, 90, 5);
        let score = data.suspiciousness(SbflFormula::Jaccard);

        // ef / (total_failed + ep) = 5 / (10 + 10) = 0.25
        assert!((score - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_wong_ii_basic() {
        let data = SpectrumData::new(10, 5, 90, 5);
        let score = data.suspiciousness(SbflFormula::WongII);

        // ef - ep = 5 - 10 = -5
        assert!((score - (-5.0)).abs() < 0.01);
    }

    #[test]
    fn test_dstar_basic() {
        let data = SpectrumData::new(10, 5, 90, 5);
        let score = data.suspiciousness(SbflFormula::DStar(2));

        // ef^2 / (ep + nf) = 25 / (10 + 5) = 1.67
        assert!((score - 1.67).abs() < 0.01);
    }

    #[test]
    fn test_spectrum_zero_coverage() {
        let data = SpectrumData::default();
        assert!((data.suspiciousness(SbflFormula::Tarantula) - 0.0).abs() < 0.01);
        assert!((data.suspiciousness(SbflFormula::Ochiai) - 0.0).abs() < 0.01);
    }

    // ============================================================
    // EXTREME TDD: RED PHASE - SuspiciousnessScore Tests
    // ============================================================

    #[test]
    fn test_score_high() {
        let score = SuspiciousnessScore {
            location: "test.rs:10".to_string(),
            score: 0.85,
            rank: 1,
        };
        assert!(score.is_high());
        assert!(!score.is_moderate());
        assert!(!score.is_low());
    }

    #[test]
    fn test_score_moderate() {
        let score = SuspiciousnessScore {
            location: "test.rs:20".to_string(),
            score: 0.5,
            rank: 2,
        };
        assert!(!score.is_high());
        assert!(score.is_moderate());
        assert!(!score.is_low());
    }

    #[test]
    fn test_score_low() {
        let score = SuspiciousnessScore {
            location: "test.rs:30".to_string(),
            score: 0.2,
            rank: 3,
        };
        assert!(!score.is_high());
        assert!(!score.is_moderate());
        assert!(score.is_low());
    }

    // ============================================================
    // EXTREME TDD: RED PHASE - SbflRanking Tests
    // ============================================================

    #[test]
    fn test_ranking_basic() {
        let locations = vec![
            ("loc1".to_string(), SpectrumData::new(10, 2, 90, 8)),
            ("loc2".to_string(), SpectrumData::new(0, 8, 100, 2)),
            ("loc3".to_string(), SpectrumData::new(50, 1, 50, 9)),
        ];

        let ranking = SbflRanking::rank(&locations, SbflFormula::Tarantula);

        assert_eq!(ranking.formula, SbflFormula::Tarantula);
        assert_eq!(ranking.scores.len(), 3);
        assert_eq!(ranking.scores[0].rank, 1);
        assert_eq!(ranking.scores[1].rank, 2);
        assert_eq!(ranking.scores[2].rank, 3);
    }

    #[test]
    fn test_ranking_order() {
        let locations = vec![
            ("low".to_string(), SpectrumData::new(100, 0, 0, 10)),  // Low suspicion
            ("high".to_string(), SpectrumData::new(0, 10, 100, 0)), // High suspicion
        ];

        let ranking = SbflRanking::rank(&locations, SbflFormula::Tarantula);

        // "high" should be ranked first
        assert_eq!(ranking.scores[0].location, "high");
        assert_eq!(ranking.scores[1].location, "low");
    }

    #[test]
    fn test_ranking_top_n() {
        let locations = vec![
            ("a".to_string(), SpectrumData::new(0, 10, 100, 0)),
            ("b".to_string(), SpectrumData::new(10, 5, 90, 5)),
            ("c".to_string(), SpectrumData::new(50, 2, 50, 8)),
            ("d".to_string(), SpectrumData::new(80, 1, 20, 9)),
        ];

        let ranking = SbflRanking::rank(&locations, SbflFormula::Ochiai);
        let top2 = ranking.top_n(2);

        assert_eq!(top2.len(), 2);
        assert_eq!(top2[0].rank, 1);
        assert_eq!(top2[1].rank, 2);
    }

    #[test]
    fn test_ranking_high_suspicion() {
        let locations = vec![
            ("high1".to_string(), SpectrumData::new(0, 10, 100, 0)),
            ("low1".to_string(), SpectrumData::new(100, 0, 0, 10)),
        ];

        let ranking = SbflRanking::rank(&locations, SbflFormula::Tarantula);
        let high = ranking.high_suspicion();

        assert_eq!(high.len(), 1);
        assert_eq!(high[0].location, "high1");
    }

    #[test]
    fn test_ranking_empty() {
        let locations: Vec<(String, SpectrumData)> = Vec::new();
        let ranking = SbflRanking::rank(&locations, SbflFormula::Ochiai);

        assert!(ranking.scores.is_empty());
        assert!(ranking.top_n(5).is_empty());
    }
}
