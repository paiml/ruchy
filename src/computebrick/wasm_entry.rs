//! WebAssembly entry point for ComputeBrick.
//!
//! Provides WASM bindings for creating and rendering widgets in the browser.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use super::canvas::{CellBuffer, Color, Rect};
use super::widgets::Brick;
use super::api;
use super::{WidgetSpec, WidgetKind, GraphMode};

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
}
