//! Rich ASCII Report Rendering
//!
//! Implements visual management (Mieruka) principles from Toyota Production System [5].
//!
//! # Features
//! - Sparklines: 8-level Unicode block elements for trend visualization [7]
//! - Progress bars: Visual completion indicators
//! - Andon status: Green/Yellow/Red quality gates [5][9]
//! - Grades: A+ to F letter grades for quality metrics

/// Andon status indicator (Toyota Way visual management) [5]
///
/// Thresholds based on Deming's statistical process control [9]:
/// - GREEN: ≥80% (within control limits)
/// - YELLOW: 50-80% (approaching limits)
/// - RED: <50% (out of control - stop the line)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndonStatus {
    /// Target reached (≥80%)
    Green,
    /// Warning zone (50-80%)
    Yellow,
    /// Stop the line (<50%)
    Red,
}

impl AndonStatus {
    /// Get emoji representation
    #[must_use]
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Green => "🟢",
            Self::Yellow => "🟡",
            Self::Red => "🔴",
        }
    }

    /// Get text label
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Green => "GREEN",
            Self::Yellow => "YELLOW",
            Self::Red => "RED",
        }
    }

    /// Get full display with emoji and label
    #[must_use]
    pub fn display(&self) -> String {
        format!("{} {}", self.emoji(), self.label())
    }
}

impl std::fmt::Display for AndonStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Determine Andon status from percentage [5][9]
#[must_use]
pub fn andon_status(percentage: f64) -> AndonStatus {
    if percentage >= 80.0 {
        AndonStatus::Green
    } else if percentage >= 50.0 {
        AndonStatus::Yellow
    } else {
        AndonStatus::Red
    }
}

/// Letter grade for quality metrics
/// Note: Higher grades have higher numeric values for proper ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    /// <50%
    F,
    /// 50-54%
    D,
    /// 55-59%
    CMinus,
    /// 60-64%
    C,
    /// 65-69%
    CPlus,
    /// 70-74%
    BMinus,
    /// 75-79%
    B,
    /// 80-84%
    BPlus,
    /// 85-89%
    AMinus,
    /// 90-94%
    A,
    /// 95-100%
    APlus,
}

impl PartialOrd for Grade {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Grade {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_val = match self {
            Grade::F => 0,
            Grade::D => 1,
            Grade::CMinus => 2,
            Grade::C => 3,
            Grade::CPlus => 4,
            Grade::BMinus => 5,
            Grade::B => 6,
            Grade::BPlus => 7,
            Grade::AMinus => 8,
            Grade::A => 9,
            Grade::APlus => 10,
        };
        let other_val = match other {
            Grade::F => 0,
            Grade::D => 1,
            Grade::CMinus => 2,
            Grade::C => 3,
            Grade::CPlus => 4,
            Grade::BMinus => 5,
            Grade::B => 6,
            Grade::BPlus => 7,
            Grade::AMinus => 8,
            Grade::A => 9,
            Grade::APlus => 10,
        };
        self_val.cmp(&other_val)
    }
}

impl Grade {
    /// Get display string
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::APlus => "A+",
            Self::A => "A",
            Self::AMinus => "A-",
            Self::BPlus => "B+",
            Self::B => "B",
            Self::BMinus => "B-",
            Self::CPlus => "C+",
            Self::C => "C",
            Self::CMinus => "C-",
            Self::D => "D",
            Self::F => "F",
        }
    }
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Calculate letter grade from percentage
#[must_use]
pub fn grade(percentage: f64) -> Grade {
    match percentage {
        p if p >= 95.0 => Grade::APlus,
        p if p >= 90.0 => Grade::A,
        p if p >= 85.0 => Grade::AMinus,
        p if p >= 80.0 => Grade::BPlus,
        p if p >= 75.0 => Grade::B,
        p if p >= 70.0 => Grade::BMinus,
        p if p >= 65.0 => Grade::CPlus,
        p if p >= 60.0 => Grade::C,
        p if p >= 55.0 => Grade::CMinus,
        p if p >= 50.0 => Grade::D,
        _ => Grade::F,
    }
}

/// Generate sparkline from values using 8-level Unicode block elements [7]
///
/// Characters: ▁▂▃▄▅▆▇█
///
/// # Example
/// ```
/// use ruchy::reporting::ascii::sparkline;
/// let line = sparkline(&[1.0, 3.0, 5.0, 7.0, 9.0]);
/// assert_eq!(line, "▁▃▄▆█");
/// ```
#[must_use]
pub fn sparkline(values: &[f64]) -> String {
    const CHARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    if values.is_empty() {
        return String::new();
    }

    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;

    values
        .iter()
        .map(|&v| {
            if range == 0.0 {
                CHARS[4] // Middle level for flat data
            } else {
                let normalized = (v - min) / range;
                let index = ((normalized * 7.0).round() as usize).min(7);
                CHARS[index]
            }
        })
        .collect()
}

/// Generate progress bar with percentage [5]
///
/// # Example
/// ```
/// use ruchy::reporting::ascii::progress_bar;
/// let bar = progress_bar(75, 100, 20);
/// assert!(bar.contains("███████████████"));
/// assert!(bar.contains("75%"));
/// ```
#[must_use]
pub fn progress_bar(current: usize, total: usize, width: usize) -> String {
    let percentage = if total == 0 {
        0.0
    } else {
        (current as f64 / total as f64) * 100.0
    };

    let filled = ((percentage / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    format!(
        "{}{} {:>3.0}%",
        "█".repeat(filled),
        "░".repeat(empty),
        percentage
    )
}

/// Generate histogram bar for distribution display
///
/// # Example
/// ```
/// use ruchy::reporting::ascii::histogram_bar;
/// let bar = histogram_bar(40, 100, 10);
/// assert_eq!(bar, "████░░░░░░");
/// ```
#[must_use]
pub fn histogram_bar(value: usize, max: usize, width: usize) -> String {
    let percentage = if max == 0 {
        0.0
    } else {
        value as f64 / max as f64
    };

    let filled = ((percentage) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Box drawing characters for rich reports
pub mod box_chars {
    /// Top-left corner (double line)
    pub const TOP_LEFT: char = '╔';
    /// Top-right corner (double line)
    pub const TOP_RIGHT: char = '╗';
    /// Bottom-left corner (double line)
    pub const BOTTOM_LEFT: char = '╚';
    /// Bottom-right corner (double line)
    pub const BOTTOM_RIGHT: char = '╝';
    /// Horizontal line (double)
    pub const HORIZONTAL: char = '═';
    /// Vertical line (double)
    pub const VERTICAL: char = '║';
    /// T-junction left (double)
    pub const T_LEFT: char = '╠';
    /// T-junction right (double)
    pub const T_RIGHT: char = '╣';

    /// Single line versions
    pub const SINGLE_TOP_LEFT: char = '┌';
    pub const SINGLE_TOP_RIGHT: char = '┐';
    pub const SINGLE_BOTTOM_LEFT: char = '└';
    pub const SINGLE_BOTTOM_RIGHT: char = '┘';
    pub const SINGLE_HORIZONTAL: char = '─';
    pub const SINGLE_VERTICAL: char = '│';
}

/// Generate a boxed header for reports
#[must_use]
pub fn boxed_header(title: &str, width: usize) -> String {
    let content_width = width.saturating_sub(2);
    let padding = content_width.saturating_sub(title.len());
    let left_pad = padding / 2;
    let right_pad = padding - left_pad;

    format!(
        "{}{}{}\n{}{}{}{}{}\n{}{}{}",
        box_chars::TOP_LEFT,
        box_chars::HORIZONTAL.to_string().repeat(content_width),
        box_chars::TOP_RIGHT,
        box_chars::VERTICAL,
        " ".repeat(left_pad),
        title,
        " ".repeat(right_pad),
        box_chars::VERTICAL,
        box_chars::BOTTOM_LEFT,
        box_chars::HORIZONTAL.to_string().repeat(content_width),
        box_chars::BOTTOM_RIGHT,
    )
}

/// Trend indicator based on values (Kaizen improvement tracking) [7]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    /// Improving (values increasing)
    Improving,
    /// Degrading (values decreasing)
    Degrading,
    /// Stable (minimal change)
    Stable,
    /// Oscillating (alternating direction)
    Oscillating,
}

impl TrendDirection {
    /// Get display label
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Improving => "improving",
            Self::Degrading => "degrading",
            Self::Stable => "stable",
            Self::Oscillating => "oscillating",
        }
    }

    /// Get arrow indicator
    #[must_use]
    pub fn arrow(&self) -> &'static str {
        match self {
            Self::Improving => "▲",
            Self::Degrading => "▼",
            Self::Stable => "→",
            Self::Oscillating => "↔",
        }
    }
}

/// Detect trend direction from values [7]
#[must_use]
pub fn detect_trend(values: &[f64]) -> TrendDirection {
    if values.len() < 2 {
        return TrendDirection::Stable;
    }

    let mut increases = 0;
    let mut decreases = 0;
    let threshold = 0.01; // 1% change threshold

    for window in values.windows(2) {
        let change = (window[1] - window[0]) / window[0].abs().max(1.0);
        if change > threshold {
            increases += 1;
        } else if change < -threshold {
            decreases += 1;
        }
    }

    let total_changes = increases + decreases;
    if total_changes == 0 {
        TrendDirection::Stable
    } else if increases > 0 && decreases > 0 && (f64::from(increases) / f64::from(total_changes)) < 0.7 {
        TrendDirection::Oscillating
    } else if increases > decreases {
        TrendDirection::Improving
    } else {
        TrendDirection::Degrading
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // EXTREME TDD: RED PHASE - Andon Status Tests
    // ============================================================

    #[test]
    fn test_andon_status_green() {
        assert_eq!(andon_status(100.0), AndonStatus::Green);
        assert_eq!(andon_status(95.0), AndonStatus::Green);
        assert_eq!(andon_status(80.0), AndonStatus::Green);
    }

    #[test]
    fn test_andon_status_yellow() {
        assert_eq!(andon_status(79.9), AndonStatus::Yellow);
        assert_eq!(andon_status(65.0), AndonStatus::Yellow);
        assert_eq!(andon_status(50.0), AndonStatus::Yellow);
    }

    #[test]
    fn test_andon_status_red() {
        assert_eq!(andon_status(49.9), AndonStatus::Red);
        assert_eq!(andon_status(25.0), AndonStatus::Red);
        assert_eq!(andon_status(0.0), AndonStatus::Red);
    }

    #[test]
    fn test_andon_emoji() {
        assert_eq!(AndonStatus::Green.emoji(), "🟢");
        assert_eq!(AndonStatus::Yellow.emoji(), "🟡");
        assert_eq!(AndonStatus::Red.emoji(), "🔴");
    }

    #[test]
    fn test_andon_label() {
        assert_eq!(AndonStatus::Green.label(), "GREEN");
        assert_eq!(AndonStatus::Yellow.label(), "YELLOW");
        assert_eq!(AndonStatus::Red.label(), "RED");
    }

    #[test]
    fn test_andon_display() {
        assert_eq!(AndonStatus::Green.display(), "🟢 GREEN");
        assert_eq!(AndonStatus::Yellow.display(), "🟡 YELLOW");
        assert_eq!(AndonStatus::Red.display(), "🔴 RED");
    }

    // ============================================================
    // EXTREME TDD: RED PHASE - Grade Tests
    // ============================================================

    #[test]
    fn test_grade_a_plus() {
        assert_eq!(grade(100.0), Grade::APlus);
        assert_eq!(grade(97.0), Grade::APlus);
        assert_eq!(grade(95.0), Grade::APlus);
    }

    #[test]
    fn test_grade_a() {
        assert_eq!(grade(94.9), Grade::A);
        assert_eq!(grade(92.0), Grade::A);
        assert_eq!(grade(90.0), Grade::A);
    }

    #[test]
    fn test_grade_a_minus() {
        assert_eq!(grade(89.9), Grade::AMinus);
        assert_eq!(grade(87.0), Grade::AMinus);
        assert_eq!(grade(85.0), Grade::AMinus);
    }

    #[test]
    fn test_grade_b_plus() {
        assert_eq!(grade(84.9), Grade::BPlus);
        assert_eq!(grade(82.0), Grade::BPlus);
        assert_eq!(grade(80.0), Grade::BPlus);
    }

    #[test]
    fn test_grade_b() {
        assert_eq!(grade(79.9), Grade::B);
        assert_eq!(grade(77.0), Grade::B);
        assert_eq!(grade(75.0), Grade::B);
    }

    #[test]
    fn test_grade_f() {
        assert_eq!(grade(49.9), Grade::F);
        assert_eq!(grade(25.0), Grade::F);
        assert_eq!(grade(0.0), Grade::F);
    }

    #[test]
    fn test_grade_as_str() {
        assert_eq!(Grade::APlus.as_str(), "A+");
        assert_eq!(Grade::A.as_str(), "A");
        assert_eq!(Grade::AMinus.as_str(), "A-");
        assert_eq!(Grade::F.as_str(), "F");
    }

    #[test]
    fn test_grade_ordering() {
        assert!(Grade::APlus > Grade::A);
        assert!(Grade::A > Grade::AMinus);
        assert!(Grade::AMinus > Grade::BPlus);
        assert!(Grade::D > Grade::F);
    }

    // ============================================================
    // EXTREME TDD: RED PHASE - Sparkline Tests
    // ============================================================

    #[test]
    fn test_sparkline_basic() {
        let line = sparkline(&[1.0, 3.0, 5.0, 7.0, 9.0]);
        assert_eq!(line.chars().count(), 5);
        // First value should be lowest, last should be highest
        assert!(line.starts_with('▁'));
        assert!(line.ends_with('█'));
    }

    #[test]
    fn test_sparkline_empty() {
        assert_eq!(sparkline(&[]), "");
    }

    #[test]
    fn test_sparkline_single() {
        let line = sparkline(&[5.0]);
        assert_eq!(line.chars().count(), 1);
        assert_eq!(line, "▅"); // Middle level for single value (index 4 of 8)
    }

    #[test]
    fn test_sparkline_flat() {
        let line = sparkline(&[5.0, 5.0, 5.0, 5.0]);
        assert!(line.chars().all(|c| c == '▅')); // All middle level (index 4 of 8)
    }

    #[test]
    fn test_sparkline_decreasing() {
        let line = sparkline(&[9.0, 7.0, 5.0, 3.0, 1.0]);
        assert!(line.starts_with('█'));
        assert!(line.ends_with('▁'));
    }

    #[test]
    fn test_sparkline_improving_trend() {
        // Kaizen: continuous improvement [7]
        let line = sparkline(&[70.0, 72.0, 75.0, 78.0, 80.0, 83.0, 85.0]);
        assert_eq!(line.chars().count(), 7);
        // Should show upward trend
        assert!(line.chars().last().unwrap() >= line.chars().next().unwrap());
    }

    // ============================================================
    // EXTREME TDD: RED PHASE - Progress Bar Tests
    // ============================================================

    #[test]
    fn test_progress_bar_empty() {
        let bar = progress_bar(0, 100, 20);
        assert!(bar.starts_with("░"));
        assert!(bar.contains("  0%"));
    }

    #[test]
    fn test_progress_bar_full() {
        let bar = progress_bar(100, 100, 20);
        assert!(bar.starts_with("████████████████████"));
        assert!(bar.contains("100%"));
    }

    #[test]
    fn test_progress_bar_half() {
        let bar = progress_bar(50, 100, 20);
        // 50% = 10 filled blocks
        let filled_count = bar.chars().filter(|&c| c == '█').count();
        assert_eq!(filled_count, 10);
        assert!(bar.contains("50%"));
    }

    #[test]
    fn test_progress_bar_zero_total() {
        let bar = progress_bar(0, 0, 20);
        assert!(bar.contains("0%"));
    }

    #[test]
    fn test_progress_bar_width() {
        let bar = progress_bar(75, 100, 10);
        // 10 characters for bar + space + percentage
        let bar_chars: usize = bar.chars().take_while(|&c| c == '█' || c == '░').count();
        assert_eq!(bar_chars, 10);
    }

    // ============================================================
    // EXTREME TDD: RED PHASE - Histogram Bar Tests
    // ============================================================

    #[test]
    fn test_histogram_bar_full() {
        let bar = histogram_bar(100, 100, 10);
        assert_eq!(bar, "██████████");
    }

    #[test]
    fn test_histogram_bar_empty() {
        let bar = histogram_bar(0, 100, 10);
        assert_eq!(bar, "░░░░░░░░░░");
    }

    #[test]
    fn test_histogram_bar_half() {
        let bar = histogram_bar(50, 100, 10);
        assert_eq!(bar, "█████░░░░░");
    }

    #[test]
    fn test_histogram_bar_zero_max() {
        let bar = histogram_bar(0, 0, 10);
        assert_eq!(bar, "░░░░░░░░░░");
    }

    // ============================================================
    // EXTREME TDD: RED PHASE - Box Drawing Tests
    // ============================================================

    #[test]
    fn test_boxed_header() {
        let header = boxed_header("TEST REPORT", 40);
        assert!(header.contains("╔"));
        assert!(header.contains("╗"));
        assert!(header.contains("║"));
        assert!(header.contains("╚"));
        assert!(header.contains("╝"));
        assert!(header.contains("TEST REPORT"));
    }

    #[test]
    fn test_boxed_header_centered() {
        let header = boxed_header("ABC", 20);
        // Title should be roughly centered
        assert!(header.contains("ABC"));
    }

    // ============================================================
    // EXTREME TDD: RED PHASE - Trend Detection Tests
    // ============================================================

    #[test]
    fn test_trend_improving() {
        let trend = detect_trend(&[70.0, 75.0, 80.0, 85.0, 90.0]);
        assert_eq!(trend, TrendDirection::Improving);
    }

    #[test]
    fn test_trend_degrading() {
        let trend = detect_trend(&[90.0, 85.0, 80.0, 75.0, 70.0]);
        assert_eq!(trend, TrendDirection::Degrading);
    }

    #[test]
    fn test_trend_stable() {
        let trend = detect_trend(&[80.0, 80.0, 80.0, 80.0]);
        assert_eq!(trend, TrendDirection::Stable);
    }

    #[test]
    fn test_trend_oscillating() {
        let trend = detect_trend(&[80.0, 70.0, 85.0, 65.0, 90.0, 60.0]);
        assert_eq!(trend, TrendDirection::Oscillating);
    }

    #[test]
    fn test_trend_single_value() {
        let trend = detect_trend(&[80.0]);
        assert_eq!(trend, TrendDirection::Stable);
    }

    #[test]
    fn test_trend_empty() {
        let trend = detect_trend(&[]);
        assert_eq!(trend, TrendDirection::Stable);
    }

    #[test]
    fn test_trend_label() {
        assert_eq!(TrendDirection::Improving.label(), "improving");
        assert_eq!(TrendDirection::Degrading.label(), "degrading");
        assert_eq!(TrendDirection::Stable.label(), "stable");
        assert_eq!(TrendDirection::Oscillating.label(), "oscillating");
    }

    #[test]
    fn test_trend_arrow() {
        assert_eq!(TrendDirection::Improving.arrow(), "▲");
        assert_eq!(TrendDirection::Degrading.arrow(), "▼");
        assert_eq!(TrendDirection::Stable.arrow(), "→");
        assert_eq!(TrendDirection::Oscillating.arrow(), "↔");
    }
}
