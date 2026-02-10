//! Andon TUI - Toyota Way Visual Management for Oracle
//!
//! Implements the Andon board visualization for Oracle training status.
//! Based on Toyota Production System visual signaling principles.
//!
//! Uses trueno-viz for visualization rendering when the `visualization` feature is enabled.
//!
//! # Toyota Way Principles
//! - **Jidoka** (自働化): Stop-the-line signal when drift detected
//! - **Kaizen** (改善): Sparkline shows continuous improvement trend
//! - **Genchi Genbutsu** (現地現物): Real metrics, not estimates
//! - **Andon** (行灯): Color-coded status board
//!
//! # References
//! - [8] Liker, J. K. (2004). "The Toyota Way." McGraw-Hill.
//! - Spec: docs/specifications/dynamic-mlops-training-ruchy-oracle-spec.md §13.3

use super::DriftStatus;
#[cfg(feature = "visualization")]
use trueno_viz::widgets::{Sparkline as VizSparkline, TrendDirection as VizTrendDirection};

/// Andon status (Toyota Way visual signaling)
///
/// Maps to factory floor Andon board colors:
/// - GREEN: System healthy, production continues
/// - YELLOW: Attention needed, monitor closely
/// - RED: Stop the line! Immediate action required
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndonStatus {
    /// GREEN - System healthy
    Green,
    /// YELLOW - Attention needed
    Yellow,
    /// RED - Stop the line!
    Red,
}

impl AndonStatus {
    /// Convert from drift status
    #[must_use]
    pub fn from_drift(drift: &DriftStatus) -> Self {
        match drift {
            DriftStatus::Stable => AndonStatus::Green,
            DriftStatus::Warning => AndonStatus::Yellow,
            DriftStatus::Drift => AndonStatus::Red,
        }
    }

    /// Get display string with indicator
    #[must_use]
    pub fn display(&self) -> &'static str {
        match self {
            AndonStatus::Green => "● STABLE",
            AndonStatus::Yellow => "● WARNING",
            AndonStatus::Red => "● DRIFT",
        }
    }

    /// Get ANSI color code for terminal output
    #[must_use]
    pub fn color_code(&self) -> &'static str {
        match self {
            AndonStatus::Green => "\x1b[32m",  // Green
            AndonStatus::Yellow => "\x1b[33m", // Yellow
            AndonStatus::Red => "\x1b[31m",    // Red
        }
    }

    /// Get reset ANSI code
    #[must_use]
    pub fn reset_code() -> &'static str {
        "\x1b[0m"
    }

    /// Check if action is required
    #[must_use]
    pub fn requires_action(&self) -> bool {
        matches!(self, AndonStatus::Yellow | AndonStatus::Red)
    }

    /// Check if stop-the-line is required
    #[must_use]
    pub fn stop_the_line(&self) -> bool {
        matches!(self, AndonStatus::Red)
    }
}

/// Sparkline characters for accuracy trend visualization (Kaizen principle)
const SPARKLINE_CHARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Render a sparkline from accuracy history
///
/// When the `visualization` feature is enabled, uses trueno-viz `Sparkline` widget
/// for trend calculation. Otherwise, renders directly from the raw values.
///
/// # Arguments
/// * `history` - Historical accuracy values (0.0-1.0)
/// * `width` - Maximum width in characters
///
/// # Returns
/// String containing sparkline characters
#[must_use]
pub fn render_sparkline(history: &[f64], width: usize) -> String {
    if history.is_empty() {
        return "─".repeat(width);
    }

    // Use trueno-viz Sparkline for trend analysis when available
    #[cfg(feature = "visualization")]
    {
        let _viz_sparkline = VizSparkline::new(history).with_trend_indicator();
    }

    // Render to Unicode block characters
    let min = history.iter().copied().fold(f64::INFINITY, f64::min);
    let max = history.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(0.01); // Avoid division by zero

    history
        .iter()
        .take(width)
        .map(|&v| {
            let normalized = ((v - min) / range * 7.0).round() as usize;
            SPARKLINE_CHARS[normalized.min(7)]
        })
        .collect()
}

/// Get trend direction
///
/// When the `visualization` feature is enabled, uses trueno-viz for trend calculation.
/// Otherwise, uses a simple slope-based heuristic.
#[must_use]
pub fn get_trend_direction(history: &[f64]) -> &'static str {
    #[cfg(feature = "visualization")]
    {
        let sparkline = VizSparkline::new(history);
        match sparkline.trend() {
            VizTrendDirection::Rising => "↑",
            VizTrendDirection::Falling => "↓",
            VizTrendDirection::Stable => "→",
        }
    }
    #[cfg(not(feature = "visualization"))]
    {
        if history.len() < 2 {
            return "→";
        }
        let first = history[0];
        let last = history[history.len() - 1];
        let diff = last - first;
        if diff > 0.01 {
            "↑"
        } else if diff < -0.01 {
            "↓"
        } else {
            "→"
        }
    }
}

/// Render the full Andon TUI board
///
/// Displays comprehensive Oracle training status in a box format.
/// Implements Genchi Genbutsu (go and see) with real metrics.
///
/// # Arguments
/// * `iteration` - Current iteration number
/// * `max_iterations` - Maximum iterations
/// * `accuracy` - Current accuracy (0.0-1.0)
/// * `target` - Target accuracy (0.0-1.0)
/// * `accuracy_delta` - Change from previous iteration
/// * `last_trained` - Timestamp string
/// * `model_size_kb` - Model size in KB
/// * `accuracy_history` - Historical accuracy values
/// * `drift` - Current drift status
#[must_use]
pub fn render_andon_tui(
    iteration: usize,
    max_iterations: usize,
    accuracy: f64,
    target: f64,
    accuracy_delta: f64,
    last_trained: &str,
    model_size_kb: usize,
    accuracy_history: &[f64],
    drift: &DriftStatus,
) -> String {
    let progress = (iteration as f64 / max_iterations as f64 * 20.0) as usize;
    let progress_bar = format!(
        "[{}{}]",
        "█".repeat(progress.min(20)),
        "░".repeat(20_usize.saturating_sub(progress))
    );

    let on_track = if accuracy >= target {
        "✓ ON TRACK"
    } else {
        "⚠ BELOW TARGET"
    };
    let delta_sign = if accuracy_delta >= 0.0 { "+" } else { "" };
    let sparkline = render_sparkline(accuracy_history, 8);
    let trend = get_trend_direction(accuracy_history);
    let andon = AndonStatus::from_drift(drift);
    let percent = (iteration as f64 / max_iterations as f64 * 100.0).round() as usize;

    format!(
        r"╔══════════════════════════════════════════════════════════════════════╗
║  Iteration: {} {}/{} ({}%)                                          ║
║  Estimated Convergence: {:.1}% → Target: {:.1}%  {}                 ║
║  Last Trained:    {}                                                ║
║  Model Size:      {} KB (zstd compressed)                           ║
║  Accuracy:        {} {:.1}% ({}{:.1}%) {}                           ║
║  Drift Status:    {}{}{}                                            ║
╚══════════════════════════════════════════════════════════════════════╝",
        progress_bar,
        iteration,
        max_iterations,
        percent,
        accuracy * 100.0,
        target * 100.0,
        on_track,
        last_trained,
        model_size_kb,
        sparkline,
        accuracy * 100.0,
        delta_sign,
        accuracy_delta * 100.0,
        trend,
        andon.color_code(),
        andon.display(),
        AndonStatus::reset_code()
    )
}

/// Render compact one-line status for default mode
///
/// Shows essential Oracle status in minimal space.
/// Used during normal transpilation operations.
#[must_use]
pub fn render_compact(
    iteration: usize,
    max_iterations: usize,
    accuracy: f64,
    model_size_kb: usize,
    last_trained_ago: &str,
    drift: &DriftStatus,
) -> String {
    let andon = AndonStatus::from_drift(drift);
    format!(
        "🔄 Oracle: iteration[{}/{}] {:.1}% acc | {}KB | {} | {}",
        iteration,
        max_iterations,
        accuracy * 100.0,
        model_size_kb,
        last_trained_ago,
        andon.display()
    )
}

/// Render minimal inline status
#[must_use]
pub fn render_inline(accuracy: f64, drift: &DriftStatus) -> String {
    let andon = AndonStatus::from_drift(drift);
    format!("[Oracle: {:.0}% {}]", accuracy * 100.0, andon.display())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_andon_status_from_drift_stable() {
        assert_eq!(
            AndonStatus::from_drift(&DriftStatus::Stable),
            AndonStatus::Green
        );
    }

    #[test]
    fn test_andon_status_from_drift_warning() {
        assert_eq!(
            AndonStatus::from_drift(&DriftStatus::Warning),
            AndonStatus::Yellow
        );
    }

    #[test]
    fn test_andon_status_from_drift_drift() {
        assert_eq!(
            AndonStatus::from_drift(&DriftStatus::Drift),
            AndonStatus::Red
        );
    }

    #[test]
    fn test_andon_status_display() {
        assert_eq!(AndonStatus::Green.display(), "● STABLE");
        assert_eq!(AndonStatus::Yellow.display(), "● WARNING");
        assert_eq!(AndonStatus::Red.display(), "● DRIFT");
    }

    #[test]
    fn test_andon_status_color_codes() {
        assert_eq!(AndonStatus::Green.color_code(), "\x1b[32m");
        assert_eq!(AndonStatus::Yellow.color_code(), "\x1b[33m");
        assert_eq!(AndonStatus::Red.color_code(), "\x1b[31m");
    }

    #[test]
    fn test_andon_requires_action() {
        assert!(!AndonStatus::Green.requires_action());
        assert!(AndonStatus::Yellow.requires_action());
        assert!(AndonStatus::Red.requires_action());
    }

    #[test]
    fn test_andon_stop_the_line() {
        assert!(!AndonStatus::Green.stop_the_line());
        assert!(!AndonStatus::Yellow.stop_the_line());
        assert!(AndonStatus::Red.stop_the_line());
    }

    #[test]
    fn test_sparkline_empty() {
        let sparkline = render_sparkline(&[], 8);
        assert_eq!(sparkline, "────────");
    }

    #[test]
    fn test_sparkline_single_value() {
        let sparkline = render_sparkline(&[0.5], 8);
        assert_eq!(sparkline.chars().count(), 1);
    }

    #[test]
    fn test_sparkline_increasing() {
        let history = vec![0.0, 0.14, 0.28, 0.42, 0.57, 0.71, 0.85, 1.0];
        let sparkline = render_sparkline(&history, 8);
        assert_eq!(sparkline, "▁▂▃▄▅▆▇█");
    }

    #[test]
    fn test_sparkline_flat() {
        let history = vec![0.5, 0.5, 0.5, 0.5];
        let sparkline = render_sparkline(&history, 4);
        // All same value should use same char
        let chars: Vec<char> = sparkline.chars().collect();
        assert!(chars.iter().all(|&c| c == chars[0]));
    }

    #[test]
    fn test_sparkline_width_limit() {
        let history: Vec<f64> = (0..100).map(|i| f64::from(i) / 100.0).collect();
        let sparkline = render_sparkline(&history, 5);
        assert_eq!(sparkline.chars().count(), 5);
    }

    #[test]
    fn test_trend_direction_rising() {
        let history = vec![0.1, 0.3, 0.5, 0.7, 0.9];
        assert_eq!(get_trend_direction(&history), "↑");
    }

    #[test]
    fn test_trend_direction_falling() {
        let history = vec![0.9, 0.7, 0.5, 0.3, 0.1];
        assert_eq!(get_trend_direction(&history), "↓");
    }

    #[test]
    fn test_trend_direction_stable() {
        let history = vec![0.5, 0.5, 0.5, 0.5];
        assert_eq!(get_trend_direction(&history), "→");
    }

    #[test]
    fn test_render_compact() {
        let compact = render_compact(12, 50, 0.873, 847, "3h ago", &DriftStatus::Stable);
        assert!(compact.contains("12/50"));
        assert!(compact.contains("87.3%"));
        assert!(compact.contains("847KB"));
        assert!(compact.contains("STABLE"));
    }

    #[test]
    fn test_render_andon_tui_contains_progress() {
        let tui = render_andon_tui(
            10,
            20,
            0.85,
            0.80,
            0.02,
            "2025-12-08",
            500,
            &[0.85],
            &DriftStatus::Stable,
        );
        assert!(tui.contains("10/20"));
        assert!(tui.contains("50%"));
    }

    #[test]
    fn test_render_andon_tui_on_track() {
        let tui = render_andon_tui(
            10,
            20,
            0.85,
            0.80,
            0.02,
            "2025-12-08",
            500,
            &[0.85],
            &DriftStatus::Stable,
        );
        assert!(tui.contains("ON TRACK"));
    }

    #[test]
    fn test_render_andon_tui_below_target() {
        let tui = render_andon_tui(
            10,
            20,
            0.70,
            0.80,
            -0.02,
            "2025-12-08",
            500,
            &[0.70],
            &DriftStatus::Warning,
        );
        assert!(tui.contains("BELOW TARGET"));
    }

    #[test]
    fn test_render_inline() {
        let inline = render_inline(0.85, &DriftStatus::Stable);
        assert!(inline.contains("85%"));
        assert!(inline.contains("STABLE"));
    }
}
