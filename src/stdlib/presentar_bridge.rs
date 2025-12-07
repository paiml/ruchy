//! Presentar Bridge Module (Pillar 6: Interaction/Widgets)
//!
//! Thin wrappers around Presentar for Ruchy stdlib.
//! Per spec Section 5.6: WASM-first widgets for interactive notebooks and dashboards.
//!
//! # Design
//! - WASM-native UI components
//! - Declarative widget tree (Flutter-inspired)
//! - Reactive state management via Rust ownership
//! - No React/Vue/Angular required
//!
//! # References
//! - [33] Google LLC (2018). "Flutter: Beautiful native apps in record time"

// Re-export core types from presentar
pub use presentar::layout;
pub use presentar::widgets;
pub use presentar::yaml;

// Re-export browser components
pub use presentar::browser::{BrowserRouter, RouteMatch, RouteMatcher};

// Re-export notebook runtime for reactive cell execution
pub use presentar::browser::{Cell, CellGraph, CellId, CellOutput, NotebookRuntime};

// Re-export WebGPU types for visualization
pub use presentar::{commands_to_instances, GpuInstance, GpuUniforms, GpuVertex};

/// Widget color type for styling
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha component (0-255)
    pub a: u8,
}

impl Color {
    /// Create a new RGBA color
    #[must_use]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create an opaque RGB color
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Convert to normalized f32 values (0.0-1.0)
    #[must_use]
    pub fn to_f32(self) -> [f32; 4] {
        [
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
            f32::from(self.a) / 255.0,
        ]
    }

    /// Common colors
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

/// Parse a hex color string (e.g., "#FF0000" or "FF0000")
pub fn parse_hex_color(hex: &str) -> Result<Color, String> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 && hex.len() != 8 {
        return Err(format!("Invalid hex color length: {}", hex.len()));
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
    let a = if hex.len() == 8 {
        u8::from_str_radix(&hex[6..8], 16).map_err(|e| e.to_string())?
    } else {
        255
    };

    Ok(Color::rgba(r, g, b, a))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_rgb() {
        let c = Color::rgb(255, 128, 0);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_color_rgba() {
        let c = Color::rgba(100, 150, 200, 128);
        assert_eq!(c.r, 100);
        assert_eq!(c.g, 150);
        assert_eq!(c.b, 200);
        assert_eq!(c.a, 128);
    }

    #[test]
    fn test_color_to_f32() {
        let c = Color::rgb(255, 0, 128);
        let f = c.to_f32();
        assert!((f[0] - 1.0).abs() < 1e-5);
        assert!((f[1] - 0.0).abs() < 1e-5);
        assert!((f[2] - 0.501_960_8).abs() < 1e-5);
        assert!((f[3] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_parse_hex_color_6_digit() {
        let c = parse_hex_color("#FF8000").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_parse_hex_color_8_digit() {
        let c = parse_hex_color("FF800080").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 128);
    }

    #[test]
    fn test_parse_hex_color_invalid() {
        assert!(parse_hex_color("FFF").is_err());
        assert!(parse_hex_color("GGGGGG").is_err());
    }

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::WHITE.r, 255);
        assert_eq!(Color::BLACK.r, 0);
        assert_eq!(Color::RED.r, 255);
        assert_eq!(Color::RED.g, 0);
        assert_eq!(Color::TRANSPARENT.a, 0);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn prop_color_to_f32_bounded(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255, a in 0u8..=255) {
            let c = Color::rgba(r, g, b, a);
            let f = c.to_f32();
            prop_assert!(f[0] >= 0.0 && f[0] <= 1.0);
            prop_assert!(f[1] >= 0.0 && f[1] <= 1.0);
            prop_assert!(f[2] >= 0.0 && f[2] <= 1.0);
            prop_assert!(f[3] >= 0.0 && f[3] <= 1.0);
        }

        #[test]
        fn prop_hex_roundtrip(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255) {
            let hex = format!("{r:02X}{g:02X}{b:02X}");
            let c = parse_hex_color(&hex).unwrap();
            prop_assert_eq!(c.r, r);
            prop_assert_eq!(c.g, g);
            prop_assert_eq!(c.b, b);
        }
    }
}
