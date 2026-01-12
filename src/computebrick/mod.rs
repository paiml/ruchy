//! `ComputeBrick` - Unified TUI/WASM Widget System
//!
//! Replaces the legacy WASM notebook system with a declarative widget API
//! that compiles to both native terminal and WebAssembly targets.
//!
//! # Architecture
//!
//! ```text
//! User API (plot, table, meter) → Widget Layer (Brick trait) → Render Backend
//! ```
//!
//! # Example
//!
//! ```ruchy
//! let data = [1, 4, 9, 16, 25]
//! plot(data, title="Squares")
//! ```

pub mod api;
pub mod canvas;
pub mod render;
pub mod widgets;

#[cfg(target_arch = "wasm32")]
pub mod wasm_entry;

use std::sync::Arc;

// Re-exports for convenience
pub use api::{dashboard, gauge, meter, plot, table};
pub use canvas::{Canvas, CellBuffer, Color, Point, Rect, TextStyle};
pub use render::{DiffRenderer, RenderBackend};
pub use widgets::{BrailleGraph, Brick, Gauge, Meter, ProgressBar, Table};

/// Event result from widget input handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult {
    /// Event was handled by the widget.
    Handled,
    /// Event was ignored, propagate to parent.
    Ignored,
}

/// Input event types for widgets.
#[derive(Debug, Clone)]
pub enum Event {
    /// Key press event.
    Key(KeyEvent),
    /// Mouse event.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize { width: u16, height: u16 },
    /// Focus gained/lost.
    Focus(bool),
}

/// Key event data.
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: Modifiers,
}

/// Key codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    Escape,
    F(u8),
}

/// Keyboard modifiers.
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

/// Mouse event data.
#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub column: u16,
    pub row: u16,
    pub modifiers: Modifiers,
}

/// Mouse event types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventKind {
    Down(MouseButton),
    Up(MouseButton),
    Drag(MouseButton),
    Moved,
    ScrollDown,
    ScrollUp,
}

/// Mouse buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Widget specification for serialization.
#[derive(Debug, Clone)]
pub struct WidgetSpec {
    pub kind: WidgetKind,
    pub bounds: Option<Rect>,
    pub style: WidgetStyle,
}

/// Widget types.
#[derive(Debug, Clone)]
pub enum WidgetKind {
    BrailleGraph {
        data: Vec<f64>,
        mode: GraphMode,
    },
    Meter {
        value: f64,
        max: f64,
        label: String,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Gauge {
        value: f64,
        label: String,
    },
    Progress {
        current: u64,
        total: u64,
        show_eta: bool,
    },
    Dashboard {
        children: Vec<(String, WidgetSpec)>,
    },
    Text {
        content: String,
    },
}

/// Graph rendering modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GraphMode {
    /// Braille patterns (2x4 dots per cell) - highest resolution.
    #[default]
    Braille,
    /// Block characters (half-blocks).
    Block,
    /// ASCII-only for TTY compatibility.
    Tty,
}

/// Widget styling options.
#[derive(Debug, Clone, Default)]
pub struct WidgetStyle {
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub title: Option<String>,
    pub border: BorderStyle,
}

/// Border styles for panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderStyle {
    /// No border.
    #[default]
    None,
    /// Sharp corners: ┌┐└┘
    Sharp,
    /// Rounded corners: ╭╮╰╯
    Rounded,
    /// Double lines: ╔╗╚╝
    Double,
    /// Heavy lines: ┏┓┗┛
    Heavy,
}

/// Gradient for color transitions.
#[derive(Debug, Clone)]
pub struct Gradient {
    stops: Vec<(f64, Color)>,
    cache: Option<Arc<[Color; 101]>>,
}

impl Gradient {
    /// Create gradient with color stops.
    pub fn new(stops: Vec<(f64, Color)>) -> Self {
        Self { stops, cache: None }
    }

    /// Two-color gradient.
    pub fn two(start: Color, end: Color) -> Self {
        Self::new(vec![(0.0, start), (1.0, end)])
    }

    /// Three-color gradient.
    pub fn three(start: Color, mid: Color, end: Color) -> Self {
        Self::new(vec![(0.0, start), (0.5, mid), (1.0, end)])
    }

    /// Precompute 101 colors for fast lookup.
    pub fn precompute(&mut self) {
        let mut colors = [Color::default(); 101];
        for (i, color) in colors.iter_mut().enumerate() {
            *color = self.sample(i as f64 / 100.0);
        }
        self.cache = Some(Arc::new(colors));
    }

    /// Sample color at position (0.0-1.0).
    #[inline]
    pub fn sample(&self, t: f64) -> Color {
        let t = t.clamp(0.0, 1.0);

        if self.stops.is_empty() {
            return Color::default();
        }
        if self.stops.len() == 1 {
            return self.stops[0].1;
        }

        // Find surrounding stops
        let mut prev = &self.stops[0];
        for stop in &self.stops {
            if stop.0 >= t {
                let range = stop.0 - prev.0;
                if range <= 0.0 {
                    return stop.1;
                }
                let local_t = (t - prev.0) / range;
                return Color::lerp(prev.1, stop.1, local_t as f32);
            }
            prev = stop;
        }

        self.stops.last().map(|s| s.1).unwrap_or_default()
    }

    /// Fast lookup by integer percentage (0-100).
    #[inline]
    pub fn at_percent(&self, pct: u8) -> Color {
        if let Some(ref cache) = self.cache {
            cache[pct.min(100) as usize]
        } else {
            self.sample(f64::from(pct) / 100.0)
        }
    }
}

/// Standard color palettes.
pub mod palettes {
    use super::Color;

    /// CPU usage gradient (green → yellow → red).
    pub const CPU: [(f64, Color); 3] = [
        (0.0, Color::new(0.3, 0.9, 0.5, 1.0)), // Green
        (0.7, Color::new(1.0, 0.9, 0.3, 1.0)), // Yellow
        (1.0, Color::new(1.0, 0.3, 0.3, 1.0)), // Red
    ];

    /// Temperature gradient (blue → white → red).
    pub const TEMP: [(f64, Color); 3] = [
        (0.0, Color::new(0.3, 0.5, 1.0, 1.0)), // Blue
        (0.5, Color::new(1.0, 1.0, 1.0, 1.0)), // White
        (1.0, Color::new(1.0, 0.3, 0.3, 1.0)), // Red
    ];

    /// Memory gradient (purple → yellow).
    pub const MEMORY: [(f64, Color); 2] = [
        (0.0, Color::new(0.6, 0.3, 0.9, 1.0)), // Purple
        (1.0, Color::new(1.0, 0.9, 0.3, 1.0)), // Yellow
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_two_color() {
        let g = Gradient::two(
            Color::new(0.0, 0.0, 0.0, 1.0),
            Color::new(1.0, 1.0, 1.0, 1.0),
        );

        let mid = g.sample(0.5);
        assert!((mid.r - 0.5).abs() < 0.01);
        assert!((mid.g - 0.5).abs() < 0.01);
        assert!((mid.b - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_gradient_precompute() {
        let mut g = Gradient::two(
            Color::new(0.0, 0.0, 0.0, 1.0),
            Color::new(1.0, 1.0, 1.0, 1.0),
        );
        g.precompute();

        let color = g.at_percent(50);
        assert!((color.r - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_event_result() {
        assert_ne!(EventResult::Handled, EventResult::Ignored);
    }

    #[test]
    fn test_graph_mode_default() {
        assert_eq!(GraphMode::default(), GraphMode::Braille);
    }

    #[test]
    fn test_border_style_default() {
        assert_eq!(BorderStyle::default(), BorderStyle::None);
    }
}
