//! Trueno-Viz Bridge Module (Pillar 5: Visualization)
//!
//! Thin wrappers around trueno-viz for Ruchy stdlib.
//! Per spec Section 5.5: GPU/WASM-accelerated charts that render identically everywhere.
//!
//! # Design
//! - Declarative grammar of graphics (Vega-Lite inspired)
//! - WebGPU for browser rendering
//! - SVG/PNG export for static output
//!
//! # Chart Embedding (Notebook Integration)
//!
//! Charts can be rendered to multiple output formats:
//! - SVG: Vector graphics for web embedding
//! - PNG: Raster graphics for static output
//! - Terminal: ASCII/Unicode rendering for CLI
//!
//! # References
//! - [32] Satyanarayan et al. (2017). "Vega-Lite: A Grammar of Interactive Graphics"
//! - [35] Wickham (2010). "A Layered Grammar of Graphics"

#[cfg(feature = "visualization")]
mod inner {
    // Re-export public types from trueno_viz
    pub use trueno_viz::plots::{BoxPlot, Heatmap, Histogram, LineChart, ScatterPlot};
    pub use trueno_viz::scale::Scale;

    // Re-export geometry for point creation
    pub use trueno_viz::geometry::Point;

    // Re-export output encoders for chart embedding
    pub use trueno_viz::output::{
        HtmlExporter, PngEncoder, SvgElement, SvgEncoder, TerminalEncoder, TerminalMode, TextAnchor,
    };

    /// Create a Point from x/y coordinates.
    #[must_use]
    pub fn point(x: f32, y: f32) -> Point {
        Point::new(x, y)
    }

    /// Convert f64 coordinates to a Point (f32).
    #[must_use]
    pub fn point_f64(x: f64, y: f64) -> Point {
        Point::new(x as f32, y as f32)
    }

    /// Create a vector of Points from parallel x/y slices.
    #[must_use]
    pub fn points_from_slices(x: &[f64], y: &[f64]) -> Vec<Point> {
        x.iter()
            .zip(y.iter())
            .map(|(&px, &py)| Point::new(px as f32, py as f32))
            .collect()
    }
}

#[cfg(feature = "visualization")]
pub use inner::*;

#[cfg(test)]
#[cfg(feature = "visualization")]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let p = point(1.0, 2.0);
        assert!((p.x - 1.0).abs() < 1e-5);
        assert!((p.y - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_point_f64_creation() {
        let p = point_f64(1.5, 2.5);
        assert!((p.x - 1.5).abs() < 1e-5);
        assert!((p.y - 2.5).abs() < 1e-5);
    }

    #[test]
    fn test_points_from_slices() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![4.0, 5.0, 6.0];
        let pts = points_from_slices(&x, &y);
        assert_eq!(pts.len(), 3);
        assert!((pts[0].x - 1.0).abs() < 1e-5);
        assert!((pts[0].y - 4.0).abs() < 1e-5);
    }
}

#[cfg(test)]
#[cfg(feature = "visualization")]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_points_preserve_length(
            x in prop::collection::vec(-1000.0..1000.0f64, 1..100),
        ) {
            let y = x.clone();
            let pts = points_from_slices(&x, &y);
            prop_assert_eq!(pts.len(), x.len());
        }

        #[test]
        fn prop_point_coordinates_preserved(
            x in -1000.0..1000.0f32,
            y in -1000.0..1000.0f32,
        ) {
            let p = point(x, y);
            prop_assert!((p.x - x).abs() < 1e-5);
            prop_assert!((p.y - y).abs() < 1e-5);
        }
    }
}

// ===== EXTREME TDD Round 154 - Viz Bridge Tests =====

#[cfg(test)]
#[cfg(feature = "visualization")]
mod extreme_tdd_tests {
    use super::*;

    #[test]
    fn test_point_zero() {
        let p = point(0.0, 0.0);
        assert!((p.x - 0.0).abs() < 1e-5);
        assert!((p.y - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_point_negative() {
        let p = point(-10.0, -20.0);
        assert!((p.x - (-10.0)).abs() < 1e-5);
        assert!((p.y - (-20.0)).abs() < 1e-5);
    }

    #[test]
    fn test_point_large_values() {
        let p = point(1e6, 1e6);
        assert!((p.x - 1e6).abs() < 1.0);
        assert!((p.y - 1e6).abs() < 1.0);
    }

    #[test]
    fn test_point_f64_zero() {
        let p = point_f64(0.0, 0.0);
        assert!((p.x - 0.0).abs() < 1e-5);
        assert!((p.y - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_point_f64_negative() {
        let p = point_f64(-100.0, -200.0);
        assert!((p.x - (-100.0)).abs() < 1e-3);
        assert!((p.y - (-200.0)).abs() < 1e-3);
    }

    #[test]
    fn test_point_f64_precision() {
        let p = point_f64(0.123456789, 0.987654321);
        assert!((p.x - 0.123456789_f32).abs() < 1e-5);
        assert!((p.y - 0.987654321_f32).abs() < 1e-5);
    }

    #[test]
    fn test_points_from_slices_empty() {
        let x: Vec<f64> = vec![];
        let y: Vec<f64> = vec![];
        let pts = points_from_slices(&x, &y);
        assert!(pts.is_empty());
    }

    #[test]
    fn test_points_from_slices_single() {
        let x = vec![42.0];
        let y = vec![84.0];
        let pts = points_from_slices(&x, &y);
        assert_eq!(pts.len(), 1);
        assert!((pts[0].x - 42.0).abs() < 1e-3);
        assert!((pts[0].y - 84.0).abs() < 1e-3);
    }

    #[test]
    fn test_points_from_slices_mismatched_shorter_y() {
        let x = vec![1.0, 2.0, 3.0, 4.0];
        let y = vec![10.0, 20.0];
        let pts = points_from_slices(&x, &y);
        // Only pairs up to shortest length
        assert_eq!(pts.len(), 2);
    }

    #[test]
    fn test_points_from_slices_negative_values() {
        let x = vec![-1.0, -2.0, -3.0];
        let y = vec![-4.0, -5.0, -6.0];
        let pts = points_from_slices(&x, &y);
        assert_eq!(pts.len(), 3);
        assert!((pts[0].x - (-1.0)).abs() < 1e-3);
        assert!((pts[2].y - (-6.0)).abs() < 1e-3);
    }
}
