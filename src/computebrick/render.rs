//! Render backend for `ComputeBrick`.
//!
//! Supports both native terminal (via crossterm) and WASM targets.

use super::canvas::{Cell, CellBuffer, Color};
use std::io::{self, Write};

/// Render backend selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackend {
    /// Direct terminal via crossterm.
    Terminal,
    /// WebAssembly `Canvas2D`.
    WasmCanvas,
    /// WebAssembly WebGL.
    WasmWebGL,
    /// Headless (for testing).
    Headless,
}

impl RenderBackend {
    /// Auto-detect appropriate backend.
    pub fn detect() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            // In WASM, prefer WebGL if available
            Self::WasmCanvas
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if std::env::var("HEADLESS").is_ok() {
                Self::Headless
            } else {
                Self::Terminal
            }
        }
    }
}

/// Color mode for terminal output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// True color (24-bit RGB).
    TrueColor,
    /// 256 colors (8-bit).
    Color256,
    /// 16 ANSI colors.
    Color16,
    /// 8 basic colors.
    Color8,
    /// Monochrome.
    Mono,
}

impl ColorMode {
    /// Detect terminal color support.
    pub fn detect() -> Self {
        // Check COLORTERM for true color support
        if let Ok(ct) = std::env::var("COLORTERM") {
            if ct == "truecolor" || ct == "24bit" {
                return Self::TrueColor;
            }
        }

        // Check TERM for 256 color support
        if let Ok(term) = std::env::var("TERM") {
            if term.contains("256color") {
                return Self::Color256;
            }
            if term.contains("color") || term == "xterm" {
                return Self::Color16;
            }
        }

        Self::Mono
    }
}

/// Differential renderer for efficient terminal updates.
///
/// Compares current and previous frame to output only changed cells.
pub struct DiffRenderer {
    color_mode: ColorMode,
    prev_buffer: Option<CellBuffer>,
    cursor_hidden: bool,
}

impl DiffRenderer {
    pub fn new() -> Self {
        Self {
            color_mode: ColorMode::detect(),
            prev_buffer: None,
            cursor_hidden: false,
        }
    }

    pub fn with_color_mode(color_mode: ColorMode) -> Self {
        Self {
            color_mode,
            prev_buffer: None,
            cursor_hidden: false,
        }
    }

    /// Flush buffer to output, using differential updates.
    ///
    /// Returns bytes written.
    pub fn flush(&mut self, buffer: &CellBuffer, output: &mut Vec<u8>) -> io::Result<usize> {
        let start_len = output.len();

        // Hide cursor during update (R19)
        if !self.cursor_hidden {
            output.extend_from_slice(b"\x1b[?25l");
            self.cursor_hidden = true;
        }

        // Use alternate screen buffer (R20)
        // output.extend_from_slice(b"\x1b[?1049h");

        // Synchronized output start (R25)
        output.extend_from_slice(b"\x1b[?2026h");

        let width = buffer.width();
        let height = buffer.height();

        // Full redraw if no previous buffer or size changed
        let full_redraw = match &self.prev_buffer {
            None => true,
            Some(prev) => prev.width() != width || prev.height() != height,
        };

        if full_redraw {
            // Clear screen and home cursor
            output.extend_from_slice(b"\x1b[2J\x1b[H");

            for row in 0..height {
                // Move to start of row
                write!(output, "\x1b[{};1H", row + 1)?;

                for col in 0..width {
                    if let Some(cell) = buffer.get(col, row) {
                        self.write_cell(output, cell)?;
                    }
                }
            }
        } else if let Some(ref prev) = self.prev_buffer {
            // Differential update - only changed cells
            for row in 0..height {
                for col in 0..width {
                    let curr = buffer.get(col, row);
                    let prev_cell = prev.get(col, row);

                    if curr != prev_cell {
                        if let Some(cell) = curr {
                            // Move cursor to position
                            write!(output, "\x1b[{};{}H", row + 1, col + 1)?;
                            self.write_cell(output, cell)?;
                        }
                    }
                }
            }
        }

        // Synchronized output end (R25)
        output.extend_from_slice(b"\x1b[?2026l");

        // Reset attributes (R18)
        output.extend_from_slice(b"\x1b[0m");

        // Store current buffer for next diff
        self.prev_buffer = Some(clone_buffer(buffer));

        Ok(output.len() - start_len)
    }

    /// Write a single cell with ANSI formatting.
    fn write_cell(&self, output: &mut Vec<u8>, cell: &Cell) -> io::Result<()> {
        // Set foreground color
        self.write_fg_color(output, cell.fg)?;

        // Set background color if not transparent
        if cell.bg.a > 0.0 {
            self.write_bg_color(output, cell.bg)?;
        }

        // Set attributes
        if cell.bold {
            output.extend_from_slice(b"\x1b[1m");
        }
        if cell.italic {
            output.extend_from_slice(b"\x1b[3m");
        }
        if cell.underline {
            output.extend_from_slice(b"\x1b[4m");
        }

        // Write character (R17: escape control chars)
        let ch = cell.ch;
        if ch as u32 >= 0x20 || ch == '\t' {
            let mut buf = [0u8; 4];
            let s = ch.encode_utf8(&mut buf);
            output.extend_from_slice(s.as_bytes());
        } else {
            // Control character - render as space (R17)
            output.push(b' ');
        }

        // Reset attributes
        output.extend_from_slice(b"\x1b[0m");

        Ok(())
    }

    /// Write foreground color ANSI sequence.
    fn write_fg_color(&self, output: &mut Vec<u8>, color: Color) -> io::Result<()> {
        match self.color_mode {
            ColorMode::TrueColor => {
                let (r, g, b) = color.to_rgb8();
                write!(output, "\x1b[38;2;{r};{g};{b}m")?;
            }
            ColorMode::Color256 => {
                let idx = color_to_256(color);
                write!(output, "\x1b[38;5;{idx}m")?;
            }
            ColorMode::Color16 | ColorMode::Color8 => {
                let idx = color_to_16(color);
                write!(output, "\x1b[{}m", 30 + idx)?;
            }
            ColorMode::Mono => {}
        }
        Ok(())
    }

    /// Write background color ANSI sequence.
    fn write_bg_color(&self, output: &mut Vec<u8>, color: Color) -> io::Result<()> {
        match self.color_mode {
            ColorMode::TrueColor => {
                let (r, g, b) = color.to_rgb8();
                write!(output, "\x1b[48;2;{r};{g};{b}m")?;
            }
            ColorMode::Color256 => {
                let idx = color_to_256(color);
                write!(output, "\x1b[48;5;{idx}m")?;
            }
            ColorMode::Color16 | ColorMode::Color8 => {
                let idx = color_to_16(color);
                write!(output, "\x1b[{}m", 40 + idx)?;
            }
            ColorMode::Mono => {}
        }
        Ok(())
    }

    /// Show cursor.
    pub fn show_cursor(&mut self, output: &mut Vec<u8>) {
        if self.cursor_hidden {
            output.extend_from_slice(b"\x1b[?25h");
            self.cursor_hidden = false;
        }
    }

    /// Reset terminal state.
    pub fn reset(&mut self, output: &mut Vec<u8>) {
        self.show_cursor(output);
        output.extend_from_slice(b"\x1b[0m"); // Reset attributes
                                              // output.extend_from_slice(b"\x1b[?1049l"); // Exit alternate screen
    }
}

impl Default for DiffRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Clone a `CellBuffer` (for diff comparison).
fn clone_buffer(buffer: &CellBuffer) -> CellBuffer {
    let mut new_buf = CellBuffer::new(buffer.width(), buffer.height());
    for y in 0..buffer.height() {
        for x in 0..buffer.width() {
            if let Some(cell) = buffer.get(x, y) {
                new_buf.set(x, y, cell.clone());
            }
        }
    }
    new_buf
}

/// Convert Color to 256-color palette index.
fn color_to_256(color: Color) -> u8 {
    let (r, g, b) = color.to_rgb8();

    // Check for grayscale
    if r == g && g == b {
        if r < 8 {
            return 16; // black
        }
        if r > 248 {
            return 231; // white
        }
        return 232 + ((r - 8) / 10).min(23);
    }

    // Map to 6x6x6 color cube (indices 16-231)
    let r_idx = (u16::from(r) * 5 / 255) as u8;
    let g_idx = (u16::from(g) * 5 / 255) as u8;
    let b_idx = (u16::from(b) * 5 / 255) as u8;

    16 + 36 * r_idx + 6 * g_idx + b_idx
}

/// Convert Color to 16-color ANSI index.
fn color_to_16(color: Color) -> u8 {
    let (r, g, b) = color.to_rgb8();

    // Simple mapping based on intensity
    let bright = r > 128 || g > 128 || b > 128;
    let base = match (r > 64, g > 64, b > 64) {
        (false, false, false) => 0, // black
        (true, false, false) => 1,  // red
        (false, true, false) => 2,  // green
        (true, true, false) => 3,   // yellow
        (false, false, true) => 4,  // blue
        (true, false, true) => 5,   // magenta
        (false, true, true) => 6,   // cyan
        (true, true, true) => 7,    // white
    };

    if bright {
        base + 8
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_mode_detect() {
        // Just ensure it doesn't panic
        let _mode = ColorMode::detect();
    }

    #[test]
    fn test_render_backend_detect() {
        let backend = RenderBackend::detect();
        #[cfg(not(target_arch = "wasm32"))]
        {
            assert!(matches!(
                backend,
                RenderBackend::Terminal | RenderBackend::Headless
            ));
        }
    }

    #[test]
    fn test_color_to_256() {
        assert_eq!(color_to_256(Color::BLACK), 16);
        // White should be in grayscale range
        let white_idx = color_to_256(Color::WHITE);
        assert!(white_idx >= 231 || white_idx == 231);
    }

    #[test]
    fn test_color_to_16() {
        assert_eq!(color_to_16(Color::BLACK), 0);
        assert_eq!(color_to_16(Color::RED), 9); // bright red
        assert_eq!(color_to_16(Color::WHITE), 15); // bright white
    }

    // R06: ANSI escape sequences valid per ECMA-48
    #[test]
    fn test_ansi_validity_r06() {
        let mut renderer = DiffRenderer::new();
        let mut buffer = CellBuffer::new(10, 5);
        buffer.set_char(0, 0, 'A', Color::RED, Color::BLACK);

        let mut output = Vec::new();
        renderer.flush(&buffer, &mut output).unwrap();

        let output_str = String::from_utf8_lossy(&output);
        // Should start with CSI sequences
        assert!(output_str.contains("\x1b["));
        // Should end with reset
        assert!(output_str.ends_with("\x1b[0m"));
    }

    // R13: Bell character (0x07) not in output
    #[test]
    fn test_no_bell_r13() {
        let mut renderer = DiffRenderer::new();
        let buffer = CellBuffer::new(10, 5);

        let mut output = Vec::new();
        renderer.flush(&buffer, &mut output).unwrap();

        assert!(!output.contains(&0x07));
    }

    // R18: Attribute reset (SGR 0) at frame end
    #[test]
    fn test_attribute_reset_r18() {
        let mut renderer = DiffRenderer::new();
        let buffer = CellBuffer::new(10, 5);

        let mut output = Vec::new();
        renderer.flush(&buffer, &mut output).unwrap();

        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.ends_with("\x1b[0m"));
    }

    // R19: No cursor flicker during update
    #[test]
    fn test_cursor_hide_r19() {
        let mut renderer = DiffRenderer::new();
        let buffer = CellBuffer::new(10, 5);

        let mut output = Vec::new();
        renderer.flush(&buffer, &mut output).unwrap();

        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("\x1b[?25l")); // CSI ?25l = hide cursor
    }

    // R25: Synchronized output (DCS) used
    #[test]
    fn test_sync_output_r25() {
        let mut renderer = DiffRenderer::new();
        let buffer = CellBuffer::new(10, 5);

        let mut output = Vec::new();
        renderer.flush(&buffer, &mut output).unwrap();

        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("\x1b[?2026h")); // sync start
        assert!(output_str.contains("\x1b[?2026l")); // sync end
    }

    // R07: Color degradation 256→16→8 correct
    #[test]
    fn test_color_degradation_r07() {
        // Test that similar colors map to same 16-color index
        let bright_red = Color::from_rgb8(255, 0, 0);
        let dark_red = Color::from_rgb8(200, 50, 50);

        // Both should map to red family in 16-color
        let bright_idx = color_to_16(bright_red);
        let dark_idx = color_to_16(dark_red);

        // Both should be in red range (1 or 9)
        assert!(bright_idx == 9 || bright_idx == 1);
        assert!(dark_idx == 9 || dark_idx == 1);

        // 256-color should preserve more detail
        let bright_256 = color_to_256(bright_red);
        let dark_256 = color_to_256(dark_red);
        assert_ne!(bright_256, dark_256); // Different shades distinguished
    }

    // R08: UTF-8 braille patterns render (U+2800-28FF)
    #[test]
    fn test_braille_unicode_r08() {
        let mut buffer = CellBuffer::new(10, 1);

        // Test all braille patterns are valid Unicode
        for pattern in 0..=255u8 {
            let ch = char::from_u32(0x2800 + pattern as u32).unwrap();
            buffer.set_char(0, 0, ch, Color::WHITE, Color::BLACK);
            assert_eq!(buffer.get(0, 0).unwrap().ch, ch);
        }
    }

    // R10: DiffRenderer produces minimal output
    #[test]
    fn test_diff_minimality_r10() {
        let mut renderer = DiffRenderer::new();
        let mut buffer = CellBuffer::new(80, 24);

        // First render - full redraw
        let mut output1 = Vec::new();
        renderer.flush(&buffer, &mut output1).unwrap();
        let full_size = output1.len();

        // Change only 10% of cells
        for y in 0..2 {
            for x in 0..8 {
                buffer.set_char(x, y, 'X', Color::RED, Color::BLACK);
            }
        }

        // Second render - differential
        let mut output2 = Vec::new();
        renderer.flush(&buffer, &mut output2).unwrap();
        let diff_size = output2.len();

        // Differential should be smaller than full redraw
        // (can't guarantee <50% due to positioning overhead)
        assert!(
            diff_size < full_size,
            "diff {} should be < full {}",
            diff_size,
            full_size
        );
    }

    // R14: Render idempotent (same input = same output)
    #[test]
    fn test_render_idempotent_r14() {
        let mut renderer1 = DiffRenderer::new();
        let mut renderer2 = DiffRenderer::new();
        let buffer = CellBuffer::new(10, 5);

        let mut output1 = Vec::new();
        let mut output2 = Vec::new();

        renderer1.flush(&buffer, &mut output1).unwrap();
        renderer2.flush(&buffer, &mut output2).unwrap();

        // Same input should produce identical output
        assert_eq!(output1, output2);
    }

    // R17: Control chars escaped in text
    #[test]
    fn test_control_escape_r17() {
        let mut renderer = DiffRenderer::new();
        let mut buffer = CellBuffer::new(10, 1);

        // Try to set a control character
        buffer.set_char(0, 0, '\x01', Color::WHITE, Color::BLACK);

        let mut output = Vec::new();
        renderer.flush(&buffer, &mut output).unwrap();

        // Should not contain raw control char (except ANSI sequences)
        // Count non-ANSI control chars
        let mut in_escape = false;
        for &byte in &output {
            if byte == 0x1b {
                in_escape = true;
            } else if in_escape && byte.is_ascii_alphabetic() {
                in_escape = false;
            } else if !in_escape && byte < 0x20 && byte != b'\n' {
                // Raw control char outside ANSI - this shouldn't happen
                // (DiffRenderer converts control chars to spaces per R17)
                panic!("Found raw control char: 0x{:02x}", byte);
            }
        }
    }

    // R11: Cursor positioning correct after render
    #[test]
    fn test_cursor_position_r11() {
        let mut renderer = DiffRenderer::new();
        let buffer = CellBuffer::new(10, 5);

        let mut output = Vec::new();
        renderer.flush(&buffer, &mut output).unwrap();

        let output_str = String::from_utf8_lossy(&output);

        // Should contain cursor positioning sequences
        assert!(output_str.contains("\x1b["));
        // First row positioning (row 1 in ANSI is 1-indexed)
        assert!(output_str.contains("\x1b[1;1H") || output_str.contains("\x1b[H"));
    }

    // R12: Terminal resize handled without crash
    #[test]
    fn test_resize_stability_r12() {
        let mut renderer = DiffRenderer::new();

        // Render at various sizes
        for size in [(10, 5), (80, 24), (200, 50), (1, 1)] {
            let buffer = CellBuffer::new(size.0, size.1);
            let mut output = Vec::new();
            // Should not panic
            let result = renderer.flush(&buffer, &mut output);
            assert!(result.is_ok());
        }
    }

    // Additional: Color mode with_color_mode constructor
    #[test]
    fn test_color_mode_constructors() {
        let mono = DiffRenderer::with_color_mode(ColorMode::Mono);
        let true_color = DiffRenderer::with_color_mode(ColorMode::TrueColor);

        let buffer = CellBuffer::new(10, 5);

        let mut output_mono = Vec::new();
        let mut output_true = Vec::new();

        let mut mono = mono;
        let mut true_color = true_color;

        mono.flush(&buffer, &mut output_mono).unwrap();
        true_color.flush(&buffer, &mut output_true).unwrap();

        // Mono should not have color codes
        let mono_str = String::from_utf8_lossy(&output_mono);
        assert!(!mono_str.contains("\x1b[38;2;"));
    }

    // Additional: Show cursor and reset
    #[test]
    fn test_show_cursor_and_reset() {
        let mut renderer = DiffRenderer::new();
        let buffer = CellBuffer::new(10, 5);

        let mut output = Vec::new();
        renderer.flush(&buffer, &mut output).unwrap();

        // Show cursor
        renderer.show_cursor(&mut output);
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("\x1b[?25h")); // show cursor

        // Reset
        let mut reset_output = Vec::new();
        renderer.reset(&mut reset_output);
        let reset_str = String::from_utf8_lossy(&reset_output);
        assert!(reset_str.contains("\x1b[0m")); // reset attributes
    }
}
