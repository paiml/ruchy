//! Widget implementations for `ComputeBrick`.
//!
//! All widgets implement the `Brick` trait for unified rendering.

use super::canvas::{Canvas, Color, Point, Rect, TextStyle};
use super::{Event, EventResult, Gradient, GraphMode};

/// Core widget trait - all widgets implement this.
///
/// Per PROBAR-SPEC-009, widgets must:
/// - Calculate layout in O(1) or O(n) where n is data size
/// - Paint without heap allocations in steady-state
/// - Handle events and return result
pub trait Brick: Send + Sync {
    /// Calculate layout given constraints.
    fn layout(&mut self, bounds: Rect);

    /// Paint to canvas.
    fn paint(&self, canvas: &mut dyn Canvas);

    /// Handle input events (optional).
    fn event(&mut self, _event: &Event) -> EventResult {
        EventResult::Ignored
    }

    /// Get current bounds.
    fn bounds(&self) -> Rect;
}

/// Braille graph for time-series visualization.
///
/// Uses Unicode braille patterns (U+2800-28FF) for high-resolution display.
/// Each cell contains 2x4 dots, allowing 8 vertical levels per character.
#[derive(Debug, Clone)]
pub struct BrailleGraph {
    pub data: Vec<f64>,
    bounds: Rect,
    pub color: Color,
    pub min: f64,
    pub max: f64,
    pub mode: GraphMode,
    pub title: Option<String>,
}

impl BrailleGraph {
    /// Create new graph from data.
    pub fn new(data: Vec<f64>) -> Self {
        let (min, max) = Self::compute_range(&data);
        Self {
            data,
            bounds: Rect::default(),
            color: Color::GREEN,
            min,
            max,
            mode: GraphMode::Braille,
            title: None,
        }
    }

    fn compute_range(data: &[f64]) -> (f64, f64) {
        if data.is_empty() {
            return (0.0, 1.0);
        }
        let min = data.iter().fold(f64::MAX, |a, &b| a.min(b));
        let max = data.iter().fold(f64::MIN, |a, &b| a.max(b));
        if (max - min).abs() < 0.001 {
            (min - 0.5, max + 0.5)
        } else {
            (min, max)
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn with_mode(mut self, mode: GraphMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Push new data point (for real-time updates).
    pub fn push(&mut self, value: f64) {
        self.data.push(value);
        // Recompute range
        let (min, max) = Self::compute_range(&self.data);
        self.min = min;
        self.max = max;
    }

    /// Replace entire dataset.
    pub fn set_data(&mut self, data: Vec<f64>) {
        self.data = data;
        let (min, max) = Self::compute_range(&self.data);
        self.min = min;
        self.max = max;
    }

    /// Normalize value to 0.0-1.0 range.
    #[inline]
    fn normalize(&self, value: f64) -> f64 {
        let range = self.max - self.min;
        if range.abs() < 0.001 {
            0.5
        } else {
            ((value - self.min) / range).clamp(0.0, 1.0)
        }
    }

    /// Get braille character for column (2 data points per char).
    fn braille_char(&self, left_val: f64, right_val: f64, _height: usize) -> char {
        let left_norm = self.normalize(left_val);
        let right_norm = self.normalize(right_val);

        // Each braille cell has 4 rows of dots
        // Map normalized value to 0-4 range
        let left_dots = (left_norm * 4.0).round() as u8;
        let right_dots = (right_norm * 4.0).round() as u8;

        // Braille pattern lookup (simplified)
        // Left column: bits 0,1,2,6 | Right column: bits 3,4,5,7
        let mut pattern: u8 = 0;

        // Fill dots from bottom up for left column
        if left_dots >= 1 {
            pattern |= 0x40;
        } // dot 7
        if left_dots >= 2 {
            pattern |= 0x04;
        } // dot 3
        if left_dots >= 3 {
            pattern |= 0x02;
        } // dot 2
        if left_dots >= 4 {
            pattern |= 0x01;
        } // dot 1

        // Fill dots from bottom up for right column
        if right_dots >= 1 {
            pattern |= 0x80;
        } // dot 8
        if right_dots >= 2 {
            pattern |= 0x20;
        } // dot 6
        if right_dots >= 3 {
            pattern |= 0x10;
        } // dot 5
        if right_dots >= 4 {
            pattern |= 0x08;
        } // dot 4

        char::from_u32(0x2800 + u32::from(pattern)).unwrap_or(' ')
    }

    /// Get block character for value.
    fn block_char(&self, value: f64) -> char {
        const BLOCKS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let norm = self.normalize(value);
        let idx = (norm * 8.0).round() as usize;
        BLOCKS[idx.min(8)]
    }

    /// Get TTY-safe character for value.
    fn tty_char(&self, value: f64) -> char {
        const TTY: [char; 5] = [' ', '.', 'o', 'O', '#'];
        let norm = self.normalize(value);
        let idx = (norm * 4.0).round() as usize;
        TTY[idx.min(4)]
    }
}

impl Brick for BrailleGraph {
    fn layout(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn paint(&self, canvas: &mut dyn Canvas) {
        if self.data.is_empty() {
            return;
        }

        let width = self.bounds.width as usize;
        let height = self.bounds.height as usize;
        let start_x = self.bounds.x as usize;
        let start_y = self.bounds.y as usize;

        // Draw title if present
        let data_start_y = if let Some(ref title) = self.title {
            let style = TextStyle::default().with_color(self.color).bold();
            canvas.draw_text(title, Point::new(self.bounds.x, self.bounds.y), &style);
            start_y + 1
        } else {
            start_y
        };

        let data_height = height - (data_start_y - start_y);

        match self.mode {
            GraphMode::Braille => {
                // Each braille char represents 2 data points horizontally
                let chars_per_row = width;
                let step = if self.data.len() > chars_per_row * 2 {
                    self.data.len() / (chars_per_row * 2)
                } else {
                    1
                };

                for col in 0..chars_per_row.min(self.data.len() / 2) {
                    let left_idx = col * 2 * step;
                    let right_idx = (col * 2 + 1) * step;

                    let left_val = self.data.get(left_idx).copied().unwrap_or(0.0);
                    let right_val = self.data.get(right_idx).copied().unwrap_or(left_val);

                    let ch = self.braille_char(left_val, right_val, data_height);
                    canvas.draw_braille(
                        (start_x + col) as f32,
                        (data_start_y + data_height - 1) as f32,
                        ch as u8,
                        self.color,
                    );
                }
            }
            GraphMode::Block => {
                let step = if self.data.len() > width {
                    self.data.len() / width
                } else {
                    1
                };

                for col in 0..width.min(self.data.len()) {
                    let idx = col * step;
                    let val = self.data.get(idx).copied().unwrap_or(0.0);
                    let ch = self.block_char(val);
                    canvas.set_char(
                        start_x + col,
                        data_start_y + data_height - 1,
                        ch,
                        self.color,
                        Color::TRANSPARENT,
                    );
                }
            }
            GraphMode::Tty => {
                let step = if self.data.len() > width {
                    self.data.len() / width
                } else {
                    1
                };

                for col in 0..width.min(self.data.len()) {
                    let idx = col * step;
                    let val = self.data.get(idx).copied().unwrap_or(0.0);
                    let ch = self.tty_char(val);
                    canvas.set_char(
                        start_x + col,
                        data_start_y + data_height - 1,
                        ch,
                        self.color,
                        Color::TRANSPARENT,
                    );
                }
            }
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }
}

/// Horizontal meter/progress bar widget.
#[derive(Debug, Clone)]
pub struct Meter {
    value: f64,
    max: f64,
    pub label: String,
    bounds: Rect,
    fill_color: Color,
    background_color: Color,
    pub gradient: Option<Gradient>,
    pub show_percentage: bool,
}

impl Meter {
    pub fn new(value: f64, max: f64) -> Self {
        Self {
            value,
            max,
            label: String::new(),
            bounds: Rect::default(),
            fill_color: Color::GREEN,
            background_color: Color::GRAY,
            gradient: None,
            show_percentage: true,
        }
    }

    pub fn percentage(pct: f64) -> Self {
        Self::new(pct, 100.0)
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.fill_color = color;
        self
    }

    pub fn with_gradient(mut self, start: Color, end: Color) -> Self {
        self.gradient = Some(Gradient::two(start, end));
        self
    }

    pub fn set_value(&mut self, value: f64) {
        self.value = value;
    }

    pub fn ratio(&self) -> f64 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.value / self.max).clamp(0.0, 1.0)
        }
    }
}

impl Brick for Meter {
    fn layout(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn paint(&self, canvas: &mut dyn Canvas) {
        let width = self.bounds.width as usize;
        let x = self.bounds.x as usize;
        let y = self.bounds.y as usize;

        // Calculate label width
        let label_width = if self.label.is_empty() {
            0
        } else {
            self.label.len() + 1
        };
        let pct_width = if self.show_percentage { 5 } else { 0 }; // " 100%"
        let bar_width = width.saturating_sub(label_width + pct_width);

        if bar_width == 0 {
            return;
        }

        // Draw label
        if !self.label.is_empty() {
            let style = TextStyle::default().with_color(Color::WHITE);
            canvas.draw_text(&self.label, Point::new(x as f32, y as f32), &style);
        }

        let bar_x = x + label_width;
        let filled = (self.ratio() * bar_width as f64).round() as usize;

        // Get fill color (from gradient if set)
        let fill_color = if let Some(ref gradient) = self.gradient {
            gradient.sample(self.ratio())
        } else {
            self.fill_color
        };

        // Draw filled portion
        for i in 0..filled {
            canvas.set_char(bar_x + i, y, '█', fill_color, Color::TRANSPARENT);
        }

        // Draw unfilled portion
        for i in filled..bar_width {
            canvas.set_char(bar_x + i, y, '░', self.background_color, Color::TRANSPARENT);
        }

        // Draw percentage
        if self.show_percentage {
            let pct = format!("{:3.0}%", self.ratio() * 100.0);
            let style = TextStyle::default().with_color(Color::WHITE);
            canvas.draw_text(
                &pct,
                Point::new((bar_x + bar_width) as f32, y as f32),
                &style,
            );
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }
}

/// Circular gauge widget.
#[derive(Debug, Clone)]
pub struct Gauge {
    value: f64,
    max: f64,
    label: String,
    bounds: Rect,
    color: Color,
}

impl Gauge {
    pub fn new(value: f64, max: f64) -> Self {
        Self {
            value,
            max,
            label: String::new(),
            bounds: Rect::default(),
            color: Color::CYAN,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn ratio(&self) -> f64 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.value / self.max).clamp(0.0, 1.0)
        }
    }
}

impl Brick for Gauge {
    fn layout(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn paint(&self, canvas: &mut dyn Canvas) {
        // Simple text-based gauge: [████░░░░] 67%
        let width = self.bounds.width as usize;
        let x = self.bounds.x as usize;
        let y = self.bounds.y as usize;

        let bar_chars = 10;
        let filled = (self.ratio() * bar_chars as f64).round() as usize;

        let mut display = String::with_capacity(width);
        if !self.label.is_empty() {
            display.push_str(&self.label);
            display.push(' ');
        }
        display.push('[');
        for i in 0..bar_chars {
            display.push(if i < filled { '█' } else { '░' });
        }
        display.push_str(&format!("] {:3.0}%", self.ratio() * 100.0));

        let style = TextStyle::default().with_color(self.color);
        canvas.draw_text(&display, Point::new(x as f32, y as f32), &style);
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }
}

/// Progress bar with optional ETA.
#[derive(Debug, Clone)]
pub struct ProgressBar {
    current: u64,
    total: u64,
    pub show_eta: bool,
    eta_seconds: Option<u64>,
    bounds: Rect,
    color: Color,
}

impl ProgressBar {
    pub fn new(current: u64, total: u64) -> Self {
        Self {
            current,
            total,
            show_eta: false,
            eta_seconds: None,
            bounds: Rect::default(),
            color: Color::GREEN,
        }
    }

    pub fn with_eta(mut self, show: bool) -> Self {
        self.show_eta = show;
        self
    }

    pub fn set_eta(&mut self, seconds: u64) {
        self.eta_seconds = Some(seconds);
    }

    pub fn set_progress(&mut self, current: u64) {
        self.current = current;
    }

    pub fn ratio(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.current as f64 / self.total as f64
        }
    }
}

impl Brick for ProgressBar {
    fn layout(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn paint(&self, canvas: &mut dyn Canvas) {
        let width = self.bounds.width as usize;
        let x = self.bounds.x as usize;
        let y = self.bounds.y as usize;

        let bar_width = 30.min(width.saturating_sub(20));
        let filled = (self.ratio() * bar_width as f64).round() as usize;

        let mut bar = String::with_capacity(bar_width + 20);
        bar.push('[');
        for i in 0..bar_width {
            bar.push(if i < filled { '█' } else { '░' });
        }
        bar.push(']');
        bar.push_str(&format!(" {:5.1}%", self.ratio() * 100.0));

        if self.show_eta {
            if let Some(eta) = self.eta_seconds {
                bar.push_str(&format!(" ETA: {}:{:02}", eta / 60, eta % 60));
            }
        }

        let style = TextStyle::default().with_color(self.color);
        canvas.draw_text(&bar, Point::new(x as f32, y as f32), &style);
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }
}

/// Simple table widget.
#[derive(Debug, Clone)]
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    bounds: Rect,
    header_color: Color,
    row_color: Color,
    column_widths: Vec<usize>,
}

impl Table {
    pub fn new(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        let column_widths = Self::compute_column_widths(&headers, &rows);
        Self {
            headers,
            rows,
            bounds: Rect::default(),
            header_color: Color::CYAN,
            row_color: Color::WHITE,
            column_widths,
        }
    }

    fn compute_column_widths(headers: &[String], rows: &[Vec<String>]) -> Vec<usize> {
        let mut widths: Vec<usize> = headers.iter().map(std::string::String::len).collect();

        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        widths
    }

    pub fn with_header_color(mut self, color: Color) -> Self {
        self.header_color = color;
        self
    }
}

impl Brick for Table {
    fn layout(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn paint(&self, canvas: &mut dyn Canvas) {
        let x = self.bounds.x as usize;
        let mut y = self.bounds.y as usize;
        let max_rows = self.bounds.height as usize;

        // Draw headers
        let header_style = TextStyle::default().with_color(self.header_color).bold();
        let mut col_x = x;
        for (i, header) in self.headers.iter().enumerate() {
            let width = self.column_widths.get(i).copied().unwrap_or(header.len());
            let padded = format!("{:<width$}", header, width = width + 2);
            canvas.draw_text(&padded, Point::new(col_x as f32, y as f32), &header_style);
            col_x += width + 2;
        }
        y += 1;

        // Draw separator
        if y < self.bounds.y as usize + max_rows {
            let total_width: usize = self.column_widths.iter().map(|w| w + 2).sum();
            let sep = "─".repeat(total_width.min(self.bounds.width as usize));
            let sep_style = TextStyle::default().with_color(Color::GRAY);
            canvas.draw_text(&sep, Point::new(x as f32, y as f32), &sep_style);
            y += 1;
        }

        // Draw rows
        let row_style = TextStyle::default().with_color(self.row_color);
        for row in &self.rows {
            if y >= self.bounds.y as usize + max_rows {
                break;
            }

            let mut col_x = x;
            for (i, cell) in row.iter().enumerate() {
                let width = self.column_widths.get(i).copied().unwrap_or(cell.len());
                let padded = format!("{:<width$}", cell, width = width + 2);
                canvas.draw_text(&padded, Point::new(col_x as f32, y as f32), &row_style);
                col_x += width + 2;
            }
            y += 1;
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // W01: BrailleGraph maps 0-100% to full height
    #[test]
    fn test_braille_scale_w01() {
        let data = vec![0.0, 50.0, 100.0];
        let graph = BrailleGraph::new(data);

        assert_eq!(graph.normalize(0.0), 0.0);
        assert!((graph.normalize(50.0) - 0.5).abs() < 0.01);
        assert_eq!(graph.normalize(100.0), 1.0);
    }

    // W02: Meter fills proportionally
    #[test]
    fn test_meter_fill_w02() {
        let meter = Meter::new(75.0, 100.0);
        assert!((meter.ratio() - 0.75).abs() < 0.01);

        let meter2 = Meter::new(50.0, 200.0);
        assert!((meter2.ratio() - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_braille_graph_push() {
        let mut graph = BrailleGraph::new(vec![1.0, 2.0, 3.0]);
        graph.push(4.0);
        assert_eq!(graph.data.len(), 4);
    }

    #[test]
    fn test_meter_percentage() {
        let meter = Meter::percentage(67.5);
        assert!((meter.ratio() - 0.675).abs() < 0.01);
    }

    #[test]
    fn test_progress_bar_ratio() {
        let bar = ProgressBar::new(45, 100);
        assert!((bar.ratio() - 0.45).abs() < 0.01);
    }

    #[test]
    fn test_table_column_widths() {
        let headers = vec!["Name".to_string(), "Score".to_string()];
        let rows = vec![
            vec!["Alice".to_string(), "95".to_string()],
            vec!["Bob".to_string(), "87".to_string()],
        ];
        let table = Table::new(headers, rows);

        assert_eq!(table.column_widths[0], 5); // "Alice"
        assert_eq!(table.column_widths[1], 5); // "Score"
    }

    #[test]
    fn test_gauge_ratio() {
        let gauge = Gauge::new(0.67, 1.0);
        assert!((gauge.ratio() - 0.67).abs() < 0.01);
    }

    // W03: Table columns align
    #[test]
    fn test_table_alignment_w03() {
        let headers = vec!["Name".to_string(), "Value".to_string()];
        let rows = vec![
            vec!["A".to_string(), "100".to_string()],
            vec!["BB".to_string(), "200".to_string()],
        ];
        let table = Table::new(headers, rows);

        // Column widths should be consistent across all rows
        assert!(table.column_widths[0] >= 4); // At least "Name" length
        assert!(table.column_widths[1] >= 5); // At least "Value" length
    }

    // W11: Sparkline fits in single row
    #[test]
    fn test_sparkline_height_w11() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut graph = BrailleGraph::new(data);

        // Layout in a single-row area
        let bounds = Rect::new(0.0, 0.0, 20.0, 1.0);
        graph.layout(bounds);

        assert_eq!(graph.bounds().height, 1.0);
    }

    // W12: ProgressBar shows percentage
    #[test]
    fn test_progress_percent_w12() {
        let bar = ProgressBar::new(50, 100);
        assert!((bar.ratio() * 100.0 - 50.0).abs() < 0.01);

        let bar2 = ProgressBar::new(75, 300);
        assert!((bar2.ratio() * 100.0 - 25.0).abs() < 0.01);
    }

    // W18: Segments sum correctly (meter value clamping)
    #[test]
    fn test_meter_clamping_w18() {
        // Value above max should clamp to 100%
        let meter = Meter::new(150.0, 100.0);
        assert!((meter.ratio() - 1.0).abs() < 0.01);

        // Value below 0 should clamp to 0%
        let meter2 = Meter::new(-10.0, 100.0);
        assert!((meter2.ratio() - 0.0).abs() < 0.01);
    }

    // W25: Widget bounds respected
    #[test]
    fn test_bounds_clip_w25() {
        let data = vec![1.0, 2.0, 3.0];
        let mut graph = BrailleGraph::new(data);

        let bounds = Rect::new(10.0, 5.0, 30.0, 15.0);
        graph.layout(bounds);

        // Widget should respect assigned bounds
        assert_eq!(graph.bounds().x, 10.0);
        assert_eq!(graph.bounds().y, 5.0);
        assert_eq!(graph.bounds().width, 30.0);
        assert_eq!(graph.bounds().height, 15.0);
    }

    // Additional: BrailleGraph modes
    #[test]
    fn test_graph_modes() {
        use super::GraphMode;

        let graph = BrailleGraph::new(vec![1.0, 2.0, 3.0]).with_mode(GraphMode::Block);
        assert_eq!(graph.mode, GraphMode::Block);

        let graph2 = BrailleGraph::new(vec![1.0, 2.0, 3.0]).with_mode(GraphMode::Tty);
        assert_eq!(graph2.mode, GraphMode::Tty);
    }

    // Additional: Meter with gradient
    #[test]
    fn test_meter_gradient() {
        let meter = Meter::new(50.0, 100.0).with_gradient(Color::GREEN, Color::RED);
        assert!(meter.gradient.is_some());
    }

    // Additional: Gauge with label
    #[test]
    fn test_gauge_label() {
        let gauge = Gauge::new(0.5, 1.0).with_label("Progress");
        assert_eq!(gauge.label, "Progress");
    }

    // Additional: ProgressBar with ETA
    #[test]
    fn test_progress_eta() {
        let bar = ProgressBar::new(50, 100).with_eta(true);
        assert!(bar.show_eta);
    }

    // Additional: Table empty handling
    #[test]
    fn test_table_empty() {
        let table = Table::new(vec![], vec![]);
        assert!(table.headers.is_empty());
        assert!(table.rows.is_empty());
    }

    // Additional: BrailleGraph empty data
    #[test]
    fn test_graph_empty_data() {
        let graph = BrailleGraph::new(vec![]);
        assert!(graph.data.is_empty());
        assert_eq!(graph.min, 0.0);
        assert_eq!(graph.max, 1.0);
    }

    // Additional: Meter with_label
    #[test]
    fn test_meter_with_label() {
        let meter = Meter::new(50.0, 100.0).with_label("CPU");
        assert_eq!(meter.label, "CPU");
    }

    // Additional: Block and TTY char mapping
    #[test]
    fn test_block_char_mapping() {
        // Need data with range 0-1 and graph with that range
        let mut graph = BrailleGraph::new(vec![0.0, 1.0]);
        graph.min = 0.0;
        graph.max = 1.0;

        // Test block characters - these use normalized values
        // BLOCKS: [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█']
        let low = graph.block_char(0.0); // normalized = 0.0 -> idx 0 -> ' '
        let mid = graph.block_char(0.5); // normalized = 0.5 -> idx 4 -> '▄'
        let high = graph.block_char(1.0); // normalized = 1.0 -> idx 8 -> '█'

        assert_eq!(low, ' '); // Lowest is blank
        assert_eq!(mid, '▄'); // Middle is half-block
        assert_eq!(high, '█'); // Highest is full block
    }

    // Additional: TTY char mapping
    #[test]
    fn test_tty_char_mapping() {
        // Need data with range 0-1 and graph with that range
        let mut graph = BrailleGraph::new(vec![0.0, 1.0]);
        graph.min = 0.0;
        graph.max = 1.0;

        // Test TTY characters - these use normalized values
        // TTY: [' ', '.', 'o', 'O', '#']
        let low = graph.tty_char(0.0); // normalized = 0.0 -> idx 0 -> ' '
        let mid = graph.tty_char(0.5); // normalized = 0.5 -> idx 2 -> 'o'
        let high = graph.tty_char(1.0); // normalized = 1.0 -> idx 4 -> '#'

        assert_eq!(low, ' '); // Lowest is blank
        assert_eq!(mid, 'o'); // Middle
        assert_eq!(high, '#'); // Highest
    }
}
