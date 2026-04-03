# Sub-spec: Oracle — Training UX Design & Visual Feedback

**Parent:** [dynamic-mlops-training-ruchy-oracle-spec.md](../dynamic-mlops-training-ruchy-oracle-spec.md) Section 13.1-13.3

---

## 13. Unified Training Loop UX

### 13.1 Design Philosophy

**Core Principle**: The Oracle should improve **by default** with every transpilation - no special commands required.

**Toyota Way Alignment**:
- **Jidoka**: Visual feedback acts as Andon board - shows system health at a glance
- **Kaizen**: Every iteration is a small improvement
- **Genchi Genbutsu**: Real metrics from actual usage, not estimates

### 13.2 Default-On Behavior

The training loop activates automatically during normal transpilation:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    UNIFIED TRAINING LOOP (Default-On)                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  User runs: ruchy transpile foo.ruchy                                   │
│                     │                                                    │
│                     ▼                                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ 1. TRANSPILE                                                     │   │
│  │    • Generate Rust code                                          │   │
│  │    • Compile with rustc                                          │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                     │                                                    │
│                     ▼                                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ 2. COLLECT (Automatic)                                           │   │
│  │    • Parse rustc errors → corpus samples                         │   │
│  │    • Deduplicate by feature hash                                 │   │
│  │    • Store in ~/.ruchy/oracle/corpus.parquet                     │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                     │                                                    │
│                     ▼                                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ 3. EVALUATE (Background)                                         │   │
│  │    • Check drift status (ADWIN)                                  │   │
│  │    • Update running accuracy                                     │   │
│  │    • Trigger retrain if threshold met                            │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  No "apex hunt" required - learning happens transparently               │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 13.3 Visual Feedback Format

#### 13.3.1 Andon TUI (Toyota Way Visual Management)

**Primary Display** - The Andon board provides at-a-glance system health:

```
╔══════════════════════════════════════════════════════════════════════╗
║  Iteration: [████████████░░░░░░░░] 12/20 (60%)                      ║
║  Estimated Convergence: 83.2% → Target: 80.0%  ✓ ON TRACK           ║
║  Last Trained:    2025-12-08 20:22:15 UTC (3 min ago)               ║
║  Model Size:      503 KB (zstd compressed)                          ║
║  Accuracy:        ▁▂▃▄▅▆▇█ 85.3% (+2.1%)                           ║
║  Drift Status:    ● STABLE                                          ║
╚══════════════════════════════════════════════════════════════════════╝
```

**Toyota Way Principles Applied**:

| Principle | Visual Element | Purpose |
|-----------|----------------|---------|
| **Jidoka** (自働化) | Drift Status indicator (●) | Stop-the-line signal when RED |
| **Kaizen** (改善) | Accuracy sparkline (▁▂▃▄▅▆▇█) | Visual trend of continuous improvement |
| **Genchi Genbutsu** (現地現物) | Real metrics, not estimates | "Go and see" actual performance |
| **Andon** (行灯) | Color-coded status board | Visual factory floor signaling |

**Andon Color States**:

```
● GREEN  (STABLE)   - System healthy, no action needed
● YELLOW (WARNING)  - Attention required, monitor closely
● RED    (DRIFT)    - Stop the line! Immediate retraining required
```

#### 13.3.2 Detailed View (Verbose Mode)

**Iteration Display** (shown during active training or verbose mode):

```
┌─────────────────────────────────────────────────────────────────────────┐
│ 🔄 ORACLE TRAINING                                                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  iteration[12/50] ████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░ 24%           │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Model Stats                                                      │   │
│  ├─────────────────────────────────────────────────────────────────┤   │
│  │ Last trained:    2025-12-08 15:42:31 (3 hours ago)              │   │
│  │ Model size:      847 KB (.apr)                                   │   │
│  │ Corpus size:     2,847 samples                                   │   │
│  │ Trees:           100                                             │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Current Evaluation                                               │   │
│  ├─────────────────────────────────────────────────────────────────┤   │
│  │ Accuracy:        87.3% (target: 80%)  ✓                         │   │
│  │ Convergence:     ~3 iterations to 90%                           │   │
│  │ Drift status:    STABLE ●                                        │   │
│  │ Fix rate:        72% single-shot                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  Category Breakdown:                                                     │
│  TypeMismatch    ████████████████████ 94.2%  ↑                          │
│  BorrowChecker   ████████████████░░░░ 88.1%  →                          │
│  LifetimeError   ███████████████░░░░░ 76.3%  ↓  ⚠                       │
│  TraitBound      ████████████████░░░░ 82.5%  →                          │
│  MissingImport   ████████████████████ 91.8%  ↑                          │
│  MutabilityError █████████████████░░░ 85.7%  →                          │
│  SyntaxError     ████████████████████ 89.4%  ↑                          │
│  Other           █████████░░░░░░░░░░░ 45.2%  ↓  ⚠                       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 13.3.3 Compact Mode (Default)

**Compact Mode** (default during transpilation):

```
🔄 Oracle: iteration[12/50] 87.3% acc | 847KB | trained 3h ago | STABLE
```

#### 13.3.4 Andon TUI Implementation

**Rust Implementation**:

```rust
use std::io::Write;

/// Andon status (Toyota Way visual signaling)
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
    pub fn from_drift(drift: &DriftStatus) -> Self {
        match drift {
            DriftStatus::Stable => AndonStatus::Green,
            DriftStatus::Warning => AndonStatus::Yellow,
            DriftStatus::Drift => AndonStatus::Red,
        }
    }

    /// Get display string with color
    pub fn display(&self) -> &'static str {
        match self {
            AndonStatus::Green => "● STABLE",
            AndonStatus::Yellow => "● WARNING",
            AndonStatus::Red => "● DRIFT",
        }
    }

    /// Get ANSI color code
    pub fn color_code(&self) -> &'static str {
        match self {
            AndonStatus::Green => "\x1b[32m",   // Green
            AndonStatus::Yellow => "\x1b[33m",  // Yellow
            AndonStatus::Red => "\x1b[31m",     // Red
        }
    }
}

/// Sparkline for accuracy trend visualization (Kaizen principle)
pub fn render_sparkline(history: &[f64], width: usize) -> String {
    const CHARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    if history.is_empty() {
        return "─".repeat(width);
    }

    let min = history.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = history.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(0.01);  // Avoid division by zero

    history.iter()
        .take(width)
        .map(|&v| {
            let normalized = ((v - min) / range * 7.0).round() as usize;
            CHARS[normalized.min(7)]
        })
        .collect()
}

/// Render the Andon TUI board
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
        "█".repeat(progress),
        "░".repeat(20 - progress)
    );

    let on_track = if accuracy >= target { "✓ ON TRACK" } else { "⚠ BELOW TARGET" };
    let delta_sign = if accuracy_delta >= 0.0 { "+" } else { "" };
    let sparkline = render_sparkline(accuracy_history, 8);
    let andon = AndonStatus::from_drift(drift);

    format!(
        r#"╔══════════════════════════════════════════════════════════════════════╗
║  Iteration: {} {}/{} ({:.0}%){}║
║  Estimated Convergence: {:.1}% → Target: {:.1}%  {}{}║
║  Last Trained:    {}{}║
║  Model Size:      {} KB (zstd compressed){}║
║  Accuracy:        {} {:.1}% ({}{:.1}%){}║
║  Drift Status:    {}{}{}{}║
╚══════════════════════════════════════════════════════════════════════╝"#,
        progress_bar, iteration, max_iterations,
        (iteration as f64 / max_iterations as f64 * 100.0),
        " ".repeat(22 - progress_bar.len()),
        accuracy * 100.0, target * 100.0, on_track,
        " ".repeat(11 - on_track.len()),
        last_trained, " ".repeat(30 - last_trained.len()),
        model_size_kb, " ".repeat(40 - format!("{}", model_size_kb).len()),
        sparkline, accuracy * 100.0, delta_sign, accuracy_delta * 100.0,
        " ".repeat(30 - sparkline.len()),
        andon.color_code(), andon.display(), "\x1b[0m",
        " ".repeat(50 - andon.display().len())
    )
}

/// Render compact one-line status
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
        iteration, max_iterations,
        accuracy * 100.0,
        model_size_kb,
        last_trained_ago,
        andon.display()
    )
}
```

**Usage Example**:

```rust
// Verbose mode (--oracle-verbose)
let tui = render_andon_tui(
    12, 20,                          // iteration 12 of 20
    0.853, 0.80,                     // 85.3% accuracy, 80% target
    0.021,                           // +2.1% improvement
    "2025-12-08 20:22:15 UTC (3 min ago)",
    503,                             // 503 KB model
    &[0.72, 0.75, 0.78, 0.81, 0.83, 0.85],  // accuracy history
    &DriftStatus::Stable,
);
println!("{}", tui);

// Compact mode (default)
let compact = render_compact(12, 20, 0.853, 503, "3 min ago", &DriftStatus::Stable);
println!("{}", compact);
```

**Convergence Estimation Algorithm** [13]:

```rust
/// Estimate iterations to target accuracy using exponential smoothing
pub fn estimate_convergence(
    current_accuracy: f64,
    target_accuracy: f64,
    accuracy_history: &[f64],
    smoothing_factor: f64,  // α = 0.3 recommended
) -> Option<usize> {
    if accuracy_history.len() < 3 {
        return None;  // Need history for estimation
    }

    // Calculate smoothed improvement rate
    let improvements: Vec<f64> = accuracy_history
        .windows(2)
        .map(|w| w[1] - w[0])
        .collect();

    let smoothed_rate = improvements.iter()
        .rev()
        .enumerate()
        .fold(0.0, |acc, (i, &delta)| {
            acc + delta * smoothing_factor.powi(i as i32)
        });

    if smoothed_rate <= 0.0 {
        return None;  // Not converging
    }

    let gap = target_accuracy - current_accuracy;
    Some((gap / smoothed_rate).ceil() as usize)
}
```

