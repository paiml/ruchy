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

#[cfg(test)]
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
