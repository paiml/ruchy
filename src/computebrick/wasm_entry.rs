//! WebAssembly entry point for ComputeBrick.
//!
//! Provides WASM bindings for creating and rendering widgets in the browser.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use super::api;
use super::canvas::{CellBuffer, Color, Rect};
use super::widgets::Brick;
use super::{GraphMode, WidgetKind, WidgetSpec};

/// WASM-compatible widget container.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmWidget {
    inner: Box<dyn Brick>,
    buffer: CellBuffer,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmWidget {
    /// Create a new braille graph widget.
    #[wasm_bindgen(constructor)]
    pub fn new_graph(data: Vec<f64>, title: Option<String>, color: Option<String>) -> Self {
        let graph = api::plot(data, title, color, None);
        Self {
            inner: Box::new(graph),
            buffer: CellBuffer::new(80, 24),
        }
    }

    /// Create a new meter widget.
    #[wasm_bindgen]
    pub fn new_meter(value: f64, max: f64, label: Option<String>) -> Self {
        let meter = api::meter(value, max, label, None);
        Self {
            inner: Box::new(meter),
            buffer: CellBuffer::new(80, 1),
        }
    }

    /// Create a new gauge widget.
    #[wasm_bindgen]
    pub fn new_gauge(value: f64, label: Option<String>) -> Self {
        let gauge = api::gauge(value, label);
        Self {
            inner: Box::new(gauge),
            buffer: CellBuffer::new(40, 1),
        }
    }

    /// Render to a Canvas2D context.
    #[wasm_bindgen]
    pub fn render_to_canvas(&mut self, ctx: &CanvasRenderingContext2d, width: u32, height: u32) {
        use super::canvas::Canvas;

        // Resize buffer if needed
        let char_width = 8.0;
        let char_height = 16.0;
        let cols = (width as f32 / char_width) as usize;
        let rows = (height as f32 / char_height) as usize;

        if self.buffer.width() != cols || self.buffer.height() != rows {
            self.buffer.resize(cols, rows);
        }

        // Clear buffer
        self.buffer.clear();

        // Layout and paint
        self.inner.layout(self.buffer.bounds());
        self.inner.paint(&mut self.buffer);

        // Render to canvas
        ctx.set_font("16px monospace");

        for y in 0..self.buffer.height() {
            for x in 0..self.buffer.width() {
                if let Some(cell) = self.buffer.get(x, y) {
                    let (r, g, b) = cell.fg.to_rgb8();
                    ctx.set_fill_style_str(&format!("rgb({},{},{})", r, g, b));

                    let ch_str = cell.ch.to_string();
                    let _ = ctx.fill_text(
                        &ch_str,
                        x as f64 * char_width as f64,
                        (y + 1) as f64 * char_height as f64,
                    );
                }
            }
        }
    }

    /// Get widget as string (for terminal/text output).
    #[wasm_bindgen]
    pub fn to_string(&mut self) -> String {
        use super::canvas::Canvas;

        self.buffer.clear();
        self.inner.layout(self.buffer.bounds());
        self.inner.paint(&mut self.buffer);

        let mut output = String::new();
        for y in 0..self.buffer.height() {
            for x in 0..self.buffer.width() {
                if let Some(cell) = self.buffer.get(x, y) {
                    output.push(cell.ch);
                }
            }
            output.push('\n');
        }
        output
    }
}

/// Create widget from JSON specification.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn create_widget_from_json(json: &str) -> Result<WasmWidget, JsValue> {
    let spec: WidgetSpec = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("JSON parse error: {}", e)))?;

    let widget = api::spec_to_widget(&spec);

    Ok(WasmWidget {
        inner: widget,
        buffer: CellBuffer::new(80, 24),
    })
}

/// Initialize WASM module.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_init() {
    // Set panic hook for better error messages
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// Non-WASM stubs for testing
#[cfg(not(target_arch = "wasm32"))]
pub struct WasmWidget {
    inner: Box<dyn Brick>,
    buffer: CellBuffer,
}

#[cfg(not(target_arch = "wasm32"))]
impl WasmWidget {
    pub fn new_graph(data: Vec<f64>, title: Option<String>, color: Option<String>) -> Self {
        let graph = api::plot(data, title, color, None);
        Self {
            inner: Box::new(graph),
            buffer: CellBuffer::new(80, 24),
        }
    }

    pub fn new_meter(value: f64, max: f64, label: Option<String>) -> Self {
        let meter = api::meter(value, max, label, None);
        Self {
            inner: Box::new(meter),
            buffer: CellBuffer::new(80, 1),
        }
    }

    pub fn to_string(&mut self) -> String {
        use super::canvas::Canvas;

        self.buffer.clear();
        self.inner.layout(self.buffer.bounds());
        self.inner.paint(&mut self.buffer);

        let mut output = String::new();
        for y in 0..self.buffer.height() {
            for x in 0..self.buffer.width() {
                if let Some(cell) = self.buffer.get(x, y) {
                    output.push(cell.ch);
                }
            }
            output.push('\n');
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // WA01: WASM widget creation
    #[test]
    fn test_wasm_widget_creation_wa01() {
        let widget = WasmWidget::new_graph(vec![1.0, 2.0, 3.0], None, None);
        assert_eq!(widget.buffer.width(), 80);
    }

    // WA02: WASM meter widget
    #[test]
    fn test_wasm_meter_wa02() {
        let widget = WasmWidget::new_meter(50.0, 100.0, Some("CPU".to_string()));
        assert_eq!(widget.buffer.height(), 1);
    }

    // WA05: to_string produces output
    #[test]
    fn test_wasm_to_string_wa05() {
        let mut widget = WasmWidget::new_graph(vec![1.0, 2.0, 3.0], None, None);
        let output = widget.to_string();
        assert!(!output.is_empty());
    }

    // WA03: WASM graph widget with title
    #[test]
    fn test_wasm_graph_with_title_wa03() {
        let widget = WasmWidget::new_graph(
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            Some("Test Graph".to_string()),
            None,
        );
        assert_eq!(widget.buffer.width(), 80);
        assert_eq!(widget.buffer.height(), 24);
    }

    // WA04: WASM graph widget with color
    #[test]
    fn test_wasm_graph_with_color_wa04() {
        let widget = WasmWidget::new_graph(vec![10.0, 20.0, 30.0], None, Some("blue".to_string()));
        assert_eq!(widget.buffer.width(), 80);
    }

    // WA06: WASM graph widget with title and color
    #[test]
    fn test_wasm_graph_with_title_and_color_wa06() {
        let widget = WasmWidget::new_graph(
            vec![5.0, 10.0, 15.0, 20.0],
            Some("My Chart".to_string()),
            Some("green".to_string()),
        );
        assert_eq!(widget.buffer.width(), 80);
        assert_eq!(widget.buffer.height(), 24);
    }

    // WA07: WASM meter widget with no label
    #[test]
    fn test_wasm_meter_no_label_wa07() {
        let widget = WasmWidget::new_meter(75.0, 100.0, None);
        assert_eq!(widget.buffer.height(), 1);
        assert_eq!(widget.buffer.width(), 80);
    }

    // WA08: WASM meter widget at zero value
    #[test]
    fn test_wasm_meter_zero_value_wa08() {
        let widget = WasmWidget::new_meter(0.0, 100.0, Some("Empty".to_string()));
        assert_eq!(widget.buffer.height(), 1);
    }

    // WA09: WASM meter widget at max value
    #[test]
    fn test_wasm_meter_max_value_wa09() {
        let widget = WasmWidget::new_meter(100.0, 100.0, Some("Full".to_string()));
        assert_eq!(widget.buffer.height(), 1);
    }

    // WA10: WASM graph with empty data
    #[test]
    fn test_wasm_graph_empty_data_wa10() {
        let widget = WasmWidget::new_graph(vec![], None, None);
        assert_eq!(widget.buffer.width(), 80);
    }

    // WA11: WASM graph with single data point
    #[test]
    fn test_wasm_graph_single_point_wa11() {
        let widget = WasmWidget::new_graph(vec![42.0], None, None);
        assert_eq!(widget.buffer.width(), 80);
    }

    // WA12: WASM meter to_string produces output
    #[test]
    fn test_wasm_meter_to_string_wa12() {
        let mut widget = WasmWidget::new_meter(50.0, 100.0, Some("RAM".to_string()));
        let output = widget.to_string();
        assert!(!output.is_empty());
        assert!(output.contains('\n')); // Should have at least one newline
    }

    // WA13: WASM graph with large data set
    #[test]
    fn test_wasm_graph_large_data_wa13() {
        let data: Vec<f64> = (0..1000).map(|i| (i as f64).sin() * 100.0).collect();
        let widget = WasmWidget::new_graph(data, Some("Sine Wave".to_string()), None);
        assert_eq!(widget.buffer.width(), 80);
    }

    // WA14: WASM graph with negative values
    #[test]
    fn test_wasm_graph_negative_values_wa14() {
        let widget = WasmWidget::new_graph(vec![-10.0, -5.0, 0.0, 5.0, 10.0], None, None);
        assert_eq!(widget.buffer.width(), 80);
    }

    // WA15: WASM graph to_string produces lines
    #[test]
    fn test_wasm_graph_to_string_lines_wa15() {
        let mut widget = WasmWidget::new_graph(vec![1.0, 2.0, 3.0], None, None);
        let output = widget.to_string();
        let line_count = output.lines().count();
        assert!(line_count > 0);
    }

    // WA16: WASM meter with very large max
    #[test]
    fn test_wasm_meter_large_max_wa16() {
        let widget = WasmWidget::new_meter(50.0, 1_000_000.0, Some("Big".to_string()));
        assert_eq!(widget.buffer.height(), 1);
    }

    // WA17: WASM graph buffer dimensions correct
    #[test]
    fn test_wasm_graph_buffer_dimensions_wa17() {
        let widget = WasmWidget::new_graph(vec![1.0, 2.0], None, None);
        assert_eq!(widget.buffer.width(), 80);
        assert_eq!(widget.buffer.height(), 24);
    }
}
