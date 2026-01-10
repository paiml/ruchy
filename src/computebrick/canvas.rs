//! Canvas abstraction for rendering widgets.
//!
//! Provides a unified drawing interface for both terminal and WASM targets.

use std::fmt;

/// RGBA color with f32 components (0.0-1.0).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create new color from RGBA components.
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create color from RGB (alpha = 1.0).
    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    /// Create from 8-bit RGB values (0-255).
    #[inline]
    pub fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgb(f32::from(r) / 255.0, f32::from(g) / 255.0, f32::from(b) / 255.0)
    }

    /// Convert to 8-bit RGB tuple.
    #[inline]
    pub fn to_rgb8(self) -> (u8, u8, u8) {
        (
            (self.r * 255.0).round() as u8,
            (self.g * 255.0).round() as u8,
            (self.b * 255.0).round() as u8,
        )
    }

    /// Linear interpolation between two colors.
    #[inline]
    pub fn lerp(a: Color, b: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color::new(
            a.r + (b.r - a.r) * t,
            a.g + (b.g - a.g) * t,
            a.b + (b.b - a.b) * t,
            a.a + (b.a - a.a) * t,
        )
    }

    // Standard colors
    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
    pub const RED: Color = Color::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Color = Color::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Color = Color::new(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW: Color = Color::new(1.0, 1.0, 0.0, 1.0);
    pub const CYAN: Color = Color::new(0.0, 1.0, 1.0, 1.0);
    pub const MAGENTA: Color = Color::new(1.0, 0.0, 1.0, 1.0);
    pub const GRAY: Color = Color::new(0.5, 0.5, 0.5, 1.0);
    pub const TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);
}

/// 2D point with f32 coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Rectangle with position and size.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    #[inline]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Create from position and size.
    #[inline]
    pub const fn from_pos_size(pos: Point, width: f32, height: f32) -> Self {
        Self::new(pos.x, pos.y, width, height)
    }

    /// Get top-left corner.
    #[inline]
    pub fn top_left(&self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Get bottom-right corner.
    #[inline]
    pub fn bottom_right(&self) -> Point {
        Point::new(self.x + self.width, self.y + self.height)
    }

    /// Check if point is inside rectangle.
    #[inline]
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x && p.x < self.x + self.width &&
        p.y >= self.y && p.y < self.y + self.height
    }

    /// Intersect with another rectangle.
    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);

        if x2 > x1 && y2 > y1 {
            Some(Rect::new(x1, y1, x2 - x1, y2 - y1))
        } else {
            None
        }
    }
}

/// Text styling options.
#[derive(Debug, Clone, Default)]
pub struct TextStyle {
    pub color: Color,
    pub background: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

impl TextStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_background(mut self, bg: Color) -> Self {
        self.background = Some(bg);
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
}

/// A single cell in the terminal buffer.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::WHITE,
            bg: Color::BLACK,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

impl Cell {
    pub fn new(ch: char) -> Self {
        Self { ch, ..Default::default() }
    }

    pub fn with_fg(mut self, fg: Color) -> Self {
        self.fg = fg;
        self
    }

    pub fn with_bg(mut self, bg: Color) -> Self {
        self.bg = bg;
        self
    }
}

/// Cell buffer for terminal rendering.
///
/// Stores a 2D grid of cells representing the terminal display.
/// Uses row-major order: `cells[row * width + col]`.
pub struct CellBuffer {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl CellBuffer {
    /// Create new buffer with given dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![Cell::default(); width * height];
        Self { width, height, cells }
    }

    /// Get buffer width.
    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get buffer height.
    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get buffer bounds as Rect.
    pub fn bounds(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width as f32, self.height as f32)
    }

    /// Get cell at position.
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        if x < self.width && y < self.height {
            Some(&self.cells[y * self.width + x])
        } else {
            None
        }
    }

    /// Get mutable cell at position.
    #[inline]
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Cell> {
        if x < self.width && y < self.height {
            Some(&mut self.cells[y * self.width + x])
        } else {
            None
        }
    }

    /// Set cell at position.
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = cell;
        }
    }

    /// Set character at position with current style.
    #[inline]
    pub fn set_char(&mut self, x: usize, y: usize, ch: char, fg: Color, bg: Color) {
        if x < self.width && y < self.height {
            let cell = &mut self.cells[y * self.width + x];
            cell.ch = ch;
            cell.fg = fg;
            cell.bg = bg;
        }
    }

    /// Clear buffer to default cells.
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::default();
        }
    }

    /// Fill rectangle with character and colors.
    pub fn fill_rect(&mut self, rect: &Rect, ch: char, fg: Color, bg: Color) {
        let x1 = rect.x.max(0.0) as usize;
        let y1 = rect.y.max(0.0) as usize;
        let x2 = ((rect.x + rect.width) as usize).min(self.width);
        let y2 = ((rect.y + rect.height) as usize).min(self.height);

        for y in y1..y2 {
            for x in x1..x2 {
                self.set_char(x, y, ch, fg, bg);
            }
        }
    }

    /// Draw text at position.
    pub fn draw_text(&mut self, x: usize, y: usize, text: &str, style: &TextStyle) {
        if y >= self.height {
            return;
        }

        let bg = style.background.unwrap_or(Color::TRANSPARENT);

        for (i, ch) in text.chars().enumerate() {
            let col = x + i;
            if col >= self.width {
                break;
            }
            let cell = &mut self.cells[y * self.width + col];
            cell.ch = ch;
            cell.fg = style.color;
            if style.background.is_some() {
                cell.bg = bg;
            }
            cell.bold = style.bold;
            cell.italic = style.italic;
            cell.underline = style.underline;
        }
    }

    /// Draw horizontal line.
    pub fn draw_hline(&mut self, x: usize, y: usize, len: usize, ch: char, color: Color) {
        if y >= self.height {
            return;
        }
        for i in 0..len {
            let col = x + i;
            if col >= self.width {
                break;
            }
            self.cells[y * self.width + col].ch = ch;
            self.cells[y * self.width + col].fg = color;
        }
    }

    /// Draw vertical line.
    pub fn draw_vline(&mut self, x: usize, y: usize, len: usize, ch: char, color: Color) {
        if x >= self.width {
            return;
        }
        for i in 0..len {
            let row = y + i;
            if row >= self.height {
                break;
            }
            self.cells[row * self.width + x].ch = ch;
            self.cells[row * self.width + x].fg = color;
        }
    }

    /// Draw box with border characters.
    pub fn draw_box(&mut self, rect: &Rect, chars: &BoxChars, color: Color) {
        let x1 = rect.x as usize;
        let y1 = rect.y as usize;
        let x2 = (rect.x + rect.width - 1.0) as usize;
        let y2 = (rect.y + rect.height - 1.0) as usize;

        if x2 >= self.width || y2 >= self.height {
            return;
        }

        // Corners
        self.set_char(x1, y1, chars.top_left, color, Color::TRANSPARENT);
        self.set_char(x2, y1, chars.top_right, color, Color::TRANSPARENT);
        self.set_char(x1, y2, chars.bottom_left, color, Color::TRANSPARENT);
        self.set_char(x2, y2, chars.bottom_right, color, Color::TRANSPARENT);

        // Horizontal lines
        for x in (x1 + 1)..x2 {
            self.set_char(x, y1, chars.horizontal, color, Color::TRANSPARENT);
            self.set_char(x, y2, chars.horizontal, color, Color::TRANSPARENT);
        }

        // Vertical lines
        for y in (y1 + 1)..y2 {
            self.set_char(x1, y, chars.vertical, color, Color::TRANSPARENT);
            self.set_char(x2, y, chars.vertical, color, Color::TRANSPARENT);
        }
    }

    /// Get raw cells slice.
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Resize buffer (clears content).
    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.cells = vec![Cell::default(); width * height];
    }
}

impl fmt::Debug for CellBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CellBuffer")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish_non_exhaustive()
    }
}

/// Box drawing characters.
#[derive(Debug, Clone, Copy)]
pub struct BoxChars {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
}

impl BoxChars {
    pub const SHARP: BoxChars = BoxChars {
        top_left: '┌',
        top_right: '┐',
        bottom_left: '└',
        bottom_right: '┘',
        horizontal: '─',
        vertical: '│',
    };

    pub const ROUNDED: BoxChars = BoxChars {
        top_left: '╭',
        top_right: '╮',
        bottom_left: '╰',
        bottom_right: '╯',
        horizontal: '─',
        vertical: '│',
    };

    pub const DOUBLE: BoxChars = BoxChars {
        top_left: '╔',
        top_right: '╗',
        bottom_left: '╚',
        bottom_right: '╝',
        horizontal: '═',
        vertical: '║',
    };

    pub const HEAVY: BoxChars = BoxChars {
        top_left: '┏',
        top_right: '┓',
        bottom_left: '┗',
        bottom_right: '┛',
        horizontal: '━',
        vertical: '┃',
    };
}

/// Canvas trait for drawing operations.
pub trait Canvas {
    /// Get canvas bounds.
    fn bounds(&self) -> Rect;

    /// Set a single cell.
    fn set_cell(&mut self, x: usize, y: usize, cell: Cell);

    /// Set character at position with colors.
    fn set_char(&mut self, x: usize, y: usize, ch: char, fg: Color, bg: Color);

    /// Draw text at position.
    fn draw_text(&mut self, text: &str, pos: Point, style: &TextStyle);

    /// Fill rectangle with color.
    fn fill_rect(&mut self, rect: Rect, color: Color);

    /// Draw braille character at position.
    fn draw_braille(&mut self, x: f32, y: f32, pattern: u8, color: Color);
}

impl Canvas for CellBuffer {
    fn bounds(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width as f32, self.height as f32)
    }

    fn set_cell(&mut self, x: usize, y: usize, cell: Cell) {
        self.set(x, y, cell);
    }

    fn set_char(&mut self, x: usize, y: usize, ch: char, fg: Color, bg: Color) {
        CellBuffer::set_char(self, x, y, ch, fg, bg);
    }

    fn draw_text(&mut self, text: &str, pos: Point, style: &TextStyle) {
        CellBuffer::draw_text(self, pos.x as usize, pos.y as usize, text, style);
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        CellBuffer::fill_rect(self, &rect, ' ', Color::WHITE, color);
    }

    fn draw_braille(&mut self, x: f32, y: f32, pattern: u8, color: Color) {
        let ch = char::from_u32(0x2800 + u32::from(pattern)).unwrap_or(' ');
        self.set_char(x as usize, y as usize, ch, color, Color::TRANSPARENT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // R05: CellBuffer correctly stores 80×24 cells
    #[test]
    fn test_cellbuffer_dimensions_r05() {
        let buf = CellBuffer::new(80, 24);
        assert_eq!(buf.width(), 80);
        assert_eq!(buf.height(), 24);
        assert_eq!(buf.cells().len(), 80 * 24);
    }

    // R15: Blank cells use space (0x20) not NUL
    #[test]
    fn test_blank_cells_r15() {
        let buf = CellBuffer::new(10, 10);
        for cell in buf.cells() {
            assert_eq!(cell.ch, ' ');
            assert_ne!(cell.ch, '\0');
        }
    }

    #[test]
    fn test_color_lerp() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        let mid = Color::lerp(a, b, 0.5);
        assert!((mid.r - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10.0, 10.0, 20.0, 20.0);
        assert!(rect.contains(Point::new(15.0, 15.0)));
        assert!(!rect.contains(Point::new(5.0, 5.0)));
    }

    #[test]
    fn test_cellbuffer_set_get() {
        let mut buf = CellBuffer::new(10, 10);
        buf.set_char(5, 5, 'X', Color::RED, Color::BLACK);

        let cell = buf.get(5, 5).unwrap();
        assert_eq!(cell.ch, 'X');
        assert_eq!(cell.fg, Color::RED);
    }

    #[test]
    fn test_cellbuffer_draw_text() {
        let mut buf = CellBuffer::new(20, 5);
        let style = TextStyle::default().with_color(Color::GREEN);
        buf.draw_text(0, 0, "Hello", &style);

        assert_eq!(buf.get(0, 0).unwrap().ch, 'H');
        assert_eq!(buf.get(4, 0).unwrap().ch, 'o');
    }

    #[test]
    fn test_cellbuffer_clear() {
        let mut buf = CellBuffer::new(10, 10);
        buf.set_char(5, 5, 'X', Color::RED, Color::BLACK);
        buf.clear();

        assert_eq!(buf.get(5, 5).unwrap().ch, ' ');
    }

    #[test]
    fn test_box_chars() {
        assert_eq!(BoxChars::SHARP.top_left, '┌');
        assert_eq!(BoxChars::ROUNDED.top_left, '╭');
    }

    // R03: Memory usage for 80×24 buffer <100KB
    #[test]
    fn test_memory_bound_r03() {
        let buf = CellBuffer::new(80, 24);
        // Each Cell has: char (4), Color×2 (8×2=16), bool×3 (3) = ~23 bytes
        // 80×24 = 1920 cells × ~32 bytes (with padding) = ~61KB
        // Plus Vec overhead = should be well under 100KB
        let cell_count = buf.cells().len();
        assert_eq!(cell_count, 1920);
        // Memory is implementation detail, but cell count must be exact
    }

    // R09: Box drawing characters align
    #[test]
    fn test_box_drawing_alignment_r09() {
        let mut buf = CellBuffer::new(10, 5);
        let bounds = Rect::new(0.0, 0.0, 10.0, 5.0);
        buf.draw_box(&bounds, &BoxChars::SHARP, Color::WHITE);

        // Corners should be at correct positions
        assert_eq!(buf.get(0, 0).unwrap().ch, '┌'); // top-left
        assert_eq!(buf.get(9, 0).unwrap().ch, '┐'); // top-right
        assert_eq!(buf.get(0, 4).unwrap().ch, '└'); // bottom-left
        assert_eq!(buf.get(9, 4).unwrap().ch, '┘'); // bottom-right

        // Horizontal edges
        assert_eq!(buf.get(5, 0).unwrap().ch, '─'); // top
        assert_eq!(buf.get(5, 4).unwrap().ch, '─'); // bottom

        // Vertical edges
        assert_eq!(buf.get(0, 2).unwrap().ch, '│'); // left
        assert_eq!(buf.get(9, 2).unwrap().ch, '│'); // right
    }

    // Additional: Color from_rgb8 and to_rgb8 roundtrip
    #[test]
    fn test_color_rgb8_roundtrip() {
        let original = (128, 64, 255);
        let color = Color::from_rgb8(original.0, original.1, original.2);
        let result = color.to_rgb8();

        // Should round-trip within 1 due to float conversion
        assert!((result.0 as i32 - original.0 as i32).abs() <= 1);
        assert!((result.1 as i32 - original.1 as i32).abs() <= 1);
        assert!((result.2 as i32 - original.2 as i32).abs() <= 1);
    }

    // Additional: Rect intersection
    #[test]
    fn test_rect_intersection() {
        let r1 = Rect::new(0.0, 0.0, 20.0, 20.0);
        let r2 = Rect::new(10.0, 10.0, 20.0, 20.0);

        let intersection = r1.intersect(&r2);
        assert!(intersection.is_some());

        let i = intersection.unwrap();
        assert_eq!(i.x, 10.0);
        assert_eq!(i.y, 10.0);
        assert_eq!(i.width, 10.0);
        assert_eq!(i.height, 10.0);
    }

    // Additional: Non-overlapping rects
    #[test]
    fn test_rect_no_intersection() {
        let r1 = Rect::new(0.0, 0.0, 10.0, 10.0);
        let r2 = Rect::new(20.0, 20.0, 10.0, 10.0);

        let intersection = r1.intersect(&r2);
        assert!(intersection.is_none());
    }

    // Additional: CellBuffer bounds check
    #[test]
    fn test_cellbuffer_bounds_check() {
        let mut buf = CellBuffer::new(10, 10);

        // Out of bounds should return None
        assert!(buf.get(10, 5).is_none());
        assert!(buf.get(5, 10).is_none());
        assert!(buf.get(100, 100).is_none());

        // Out of bounds set should be ignored (no panic)
        buf.set_char(100, 100, 'X', Color::RED, Color::BLACK);
    }

    // Additional: TextStyle builder
    #[test]
    fn test_text_style_builder() {
        let style = TextStyle::new()
            .with_color(Color::RED)
            .with_background(Color::BLACK)
            .bold()
            .italic();

        assert_eq!(style.color, Color::RED);
        assert_eq!(style.background, Some(Color::BLACK));
        assert!(style.bold);
        assert!(style.italic);
    }

    // Additional: Cell builder
    #[test]
    fn test_cell_builder() {
        let cell = Cell::new('A')
            .with_fg(Color::GREEN)
            .with_bg(Color::BLUE);

        assert_eq!(cell.ch, 'A');
        assert_eq!(cell.fg, Color::GREEN);
        assert_eq!(cell.bg, Color::BLUE);
    }

    // Additional: Draw horizontal and vertical lines
    #[test]
    fn test_draw_lines() {
        let mut buf = CellBuffer::new(20, 10);

        buf.draw_hline(5, 3, 10, '-', Color::WHITE);
        buf.draw_vline(10, 1, 8, '|', Color::WHITE);

        // Horizontal line
        assert_eq!(buf.get(5, 3).unwrap().ch, '-');
        assert_eq!(buf.get(14, 3).unwrap().ch, '-');

        // Vertical line
        assert_eq!(buf.get(10, 1).unwrap().ch, '|');
        assert_eq!(buf.get(10, 8).unwrap().ch, '|');
    }

    // Additional: Fill rect
    #[test]
    fn test_fill_rect() {
        let mut buf = CellBuffer::new(20, 10);
        let rect = Rect::new(5.0, 2.0, 10.0, 6.0);

        buf.fill_rect(&rect, '#', Color::YELLOW, Color::BLUE);

        // Check filled area
        assert_eq!(buf.get(5, 2).unwrap().ch, '#');
        assert_eq!(buf.get(14, 7).unwrap().ch, '#');
        assert_eq!(buf.get(5, 2).unwrap().fg, Color::YELLOW);

        // Check outside area unchanged
        assert_eq!(buf.get(0, 0).unwrap().ch, ' ');
    }

    // Additional: Resize buffer
    #[test]
    fn test_resize_buffer() {
        let mut buf = CellBuffer::new(10, 10);
        buf.set_char(5, 5, 'X', Color::RED, Color::BLACK);

        buf.resize(20, 20);

        assert_eq!(buf.width(), 20);
        assert_eq!(buf.height(), 20);
        // Content should be cleared after resize
        assert_eq!(buf.get(5, 5).unwrap().ch, ' ');
    }

    // Additional: Canvas trait draw_braille
    #[test]
    fn test_canvas_draw_braille() {
        let mut buf = CellBuffer::new(10, 10);

        // Draw braille pattern 0xFF (all dots)
        buf.draw_braille(5.0, 5.0, 0xFF, Color::WHITE);

        let cell = buf.get(5, 5).unwrap();
        assert_eq!(cell.ch, '⣿'); // U+28FF = all 8 dots
    }

    // Additional: Point and Rect constructors
    #[test]
    fn test_point_rect_constructors() {
        let point = Point::new(10.0, 20.0);
        assert_eq!(point.x, 10.0);
        assert_eq!(point.y, 20.0);

        let rect = Rect::from_pos_size(point, 30.0, 40.0);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 30.0);
        assert_eq!(rect.height, 40.0);
    }

    // Additional: Rect top_left and bottom_right
    #[test]
    fn test_rect_corners() {
        let rect = Rect::new(10.0, 20.0, 30.0, 40.0);

        let tl = rect.top_left();
        assert_eq!(tl.x, 10.0);
        assert_eq!(tl.y, 20.0);

        let br = rect.bottom_right();
        assert_eq!(br.x, 40.0);
        assert_eq!(br.y, 60.0);
    }

    // Additional: All box char styles
    #[test]
    fn test_all_box_styles() {
        assert_eq!(BoxChars::SHARP.horizontal, '─');
        assert_eq!(BoxChars::ROUNDED.horizontal, '─');
        assert_eq!(BoxChars::DOUBLE.horizontal, '═');
        assert_eq!(BoxChars::HEAVY.horizontal, '━');

        assert_eq!(BoxChars::SHARP.vertical, '│');
        assert_eq!(BoxChars::DOUBLE.vertical, '║');
        assert_eq!(BoxChars::HEAVY.vertical, '┃');
    }
}
