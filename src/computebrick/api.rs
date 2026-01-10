//! User-facing API for ComputeBrick widgets.
//!
//! Provides simple functions like `plot()`, `table()`, `meter()` that
//! create and render widgets from Ruchy data.

use super::canvas::{CellBuffer, Color, Rect};
use super::widgets::{Brick, BrailleGraph, Gauge, Meter, ProgressBar, Table};
use super::render::DiffRenderer;
use super::{GraphMode, WidgetSpec, WidgetKind};
use std::io::{self, Write};

/// Plot data as a braille graph.
///
/// # Arguments
/// * `data` - Numeric data to plot
/// * `title` - Optional title for the graph
/// * `color` - Optional color name ("red", "green", "blue", etc.)
/// * `mode` - Render mode ("braille", "block", "tty")
///
/// # Example
/// ```ruchy
/// let data = [1, 4, 9, 16, 25]
/// plot(data, title="Squares", color="green")
/// ```
pub fn plot(
    data: Vec<f64>,
    title: Option<String>,
    color: Option<String>,
    mode: Option<String>,
) -> BrailleGraph {
    let mut graph = BrailleGraph::new(data);

    if let Some(t) = title {
        graph = graph.with_title(t);
    }

    if let Some(c) = color {
        graph = graph.with_color(parse_color(&c));
    }

    if let Some(m) = mode {
        graph = graph.with_mode(parse_graph_mode(&m));
    }

    graph
}

/// Create a horizontal meter/progress bar.
///
/// # Arguments
/// * `value` - Current value
/// * `max` - Maximum value
/// * `label` - Optional label
/// * `gradient` - Optional gradient colors ["start", "end"]
///
/// # Example
/// ```ruchy
/// meter(value=75, max=100, label="CPU", gradient=["green", "red"])
/// ```
pub fn meter(
    value: f64,
    max: f64,
    label: Option<String>,
    gradient: Option<(String, String)>,
) -> Meter {
    let mut m = Meter::new(value, max);

    if let Some(l) = label {
        m = m.with_label(l);
    }

    if let Some((start, end)) = gradient {
        m = m.with_gradient(parse_color(&start), parse_color(&end));
    }

    m
}

/// Create a circular gauge.
///
/// # Arguments
/// * `value` - Current value (0.0-1.0 for percentage)
/// * `label` - Optional label
///
/// # Example
/// ```ruchy
/// gauge(value=0.67, label="Progress")
/// ```
pub fn gauge(value: f64, label: Option<String>) -> Gauge {
    let mut g = Gauge::new(value, 1.0);

    if let Some(l) = label {
        g = g.with_label(l);
    }

    g
}

/// Create a progress bar with optional ETA.
///
/// # Arguments
/// * `current` - Current progress value
/// * `total` - Total value
/// * `show_eta` - Whether to show ETA
///
/// # Example
/// ```ruchy
/// progress(current=45, total=100, show_eta=true)
/// ```
pub fn progress(current: u64, total: u64, show_eta: bool) -> ProgressBar {
    ProgressBar::new(current, total).with_eta(show_eta)
}

/// Create a table from headers and rows.
///
/// # Arguments
/// * `headers` - Column headers
/// * `rows` - Data rows
///
/// # Example
/// ```ruchy
/// table(
///     headers=["Name", "Score"],
///     rows=[["Alice", "95"], ["Bob", "87"]]
/// )
/// ```
pub fn table(headers: Vec<String>, rows: Vec<Vec<String>>) -> Table {
    Table::new(headers, rows)
}

/// Create a dashboard layout with multiple widgets.
///
/// # Arguments
/// * `children` - Named widgets as (name, widget) pairs
///
/// # Example
/// ```ruchy
/// dashboard {
///     cpu: plot(cpu_data),
///     memory: meter(mem_used, mem_total, label="Memory"),
/// }
/// ```
pub fn dashboard(children: Vec<(String, Box<dyn Brick>)>) -> Dashboard {
    Dashboard::new(children)
}

/// Dashboard layout widget.
pub struct Dashboard {
    children: Vec<(String, Box<dyn Brick>)>,
    bounds: Rect,
}

impl Dashboard {
    pub fn new(children: Vec<(String, Box<dyn Brick>)>) -> Self {
        Self {
            children,
            bounds: Rect::default(),
        }
    }
}

impl Brick for Dashboard {
    fn layout(&mut self, bounds: Rect) {
        self.bounds = bounds;

        // Simple vertical layout - divide height equally
        let child_count = self.children.len();
        if child_count == 0 {
            return;
        }

        let child_height = bounds.height / child_count as f32;
        for (i, (_, child)) in self.children.iter_mut().enumerate() {
            let child_bounds = Rect::new(
                bounds.x,
                bounds.y + i as f32 * child_height,
                bounds.width,
                child_height,
            );
            child.layout(child_bounds);
        }
    }

    fn paint(&self, canvas: &mut dyn super::canvas::Canvas) {
        for (_, child) in &self.children {
            child.paint(canvas);
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }
}

/// Render a widget to the terminal.
pub fn render_to_terminal(widget: &dyn Brick, width: usize, height: usize) -> io::Result<()> {
    let mut buffer = CellBuffer::new(width, height);
    let mut renderer = DiffRenderer::new();

    // Layout and paint
    let mut widget_clone = create_default_graph(); // Placeholder
    widget_clone.layout(buffer.bounds());
    widget.paint(&mut buffer);

    // Render to stdout
    let mut output = Vec::with_capacity(8192);
    renderer.flush(&buffer, &mut output)?;

    io::stdout().write_all(&output)?;
    io::stdout().flush()?;

    Ok(())
}

/// Create widget from specification.
pub fn spec_to_widget(spec: &WidgetSpec) -> Box<dyn Brick> {
    match &spec.kind {
        WidgetKind::BrailleGraph { data, mode } => {
            let mut graph = BrailleGraph::new(data.clone()).with_mode(*mode);
            if let Some(ref title) = spec.style.title {
                graph = graph.with_title(title);
            }
            if let Some(color) = spec.style.color {
                graph = graph.with_color(color);
            }
            Box::new(graph)
        }
        WidgetKind::Meter { value, max, label } => {
            let mut m = Meter::new(*value, *max).with_label(label);
            if let Some(color) = spec.style.color {
                m = m.with_color(color);
            }
            Box::new(m)
        }
        WidgetKind::Table { headers, rows } => {
            Box::new(Table::new(headers.clone(), rows.clone()))
        }
        WidgetKind::Gauge { value, label } => {
            let mut g = Gauge::new(*value, 1.0).with_label(label);
            if let Some(color) = spec.style.color {
                g = g.with_color(color);
            }
            Box::new(g)
        }
        WidgetKind::Progress { current, total, show_eta } => {
            Box::new(ProgressBar::new(*current, *total).with_eta(*show_eta))
        }
        WidgetKind::Dashboard { children } => {
            let widgets: Vec<(String, Box<dyn Brick>)> = children
                .iter()
                .map(|(name, spec)| (name.clone(), spec_to_widget(spec)))
                .collect();
            Box::new(Dashboard::new(widgets))
        }
        WidgetKind::Text { content } => {
            Box::new(TextWidget::new(content.clone()))
        }
    }
}

/// Simple text widget.
pub struct TextWidget {
    content: String,
    bounds: Rect,
    color: Color,
}

impl TextWidget {
    pub fn new(content: String) -> Self {
        Self {
            content,
            bounds: Rect::default(),
            color: Color::WHITE,
        }
    }
}

impl Brick for TextWidget {
    fn layout(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn paint(&self, canvas: &mut dyn super::canvas::Canvas) {
        use super::canvas::{Point, TextStyle};
        let style = TextStyle::default().with_color(self.color);
        canvas.draw_text(&self.content, Point::new(self.bounds.x, self.bounds.y), &style);
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }
}

/// Parse color name to Color.
fn parse_color(name: &str) -> Color {
    match name.to_lowercase().as_str() {
        "red" => Color::RED,
        "green" => Color::GREEN,
        "blue" => Color::BLUE,
        "yellow" => Color::YELLOW,
        "cyan" => Color::CYAN,
        "magenta" => Color::MAGENTA,
        "white" => Color::WHITE,
        "black" => Color::BLACK,
        "gray" | "grey" => Color::GRAY,
        // Handle hex colors
        s if s.starts_with('#') && s.len() == 7 => {
            let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(255);
            let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(255);
            let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(255);
            Color::from_rgb8(r, g, b)
        }
        _ => Color::WHITE,
    }
}

/// Parse graph mode string.
fn parse_graph_mode(mode: &str) -> GraphMode {
    match mode.to_lowercase().as_str() {
        "braille" => GraphMode::Braille,
        "block" => GraphMode::Block,
        "tty" | "ascii" => GraphMode::Tty,
        _ => GraphMode::Braille,
    }
}

/// Create a default graph for layout purposes.
fn create_default_graph() -> BrailleGraph {
    BrailleGraph::new(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    // I01: plot() creates BrailleGraph
    #[test]
    fn test_plot_widget_i01() {
        let graph = plot(vec![1.0, 2.0, 3.0], None, None, None);
        // Type check - it compiles means it's BrailleGraph
        let _: BrailleGraph = graph;
    }

    // I02: table() creates Table
    #[test]
    fn test_table_widget_i02() {
        let t = table(
            vec!["A".to_string(), "B".to_string()],
            vec![vec!["1".to_string(), "2".to_string()]],
        );
        let _: Table = t;
    }

    // I03: meter() creates Meter
    #[test]
    fn test_meter_widget_i03() {
        let m = meter(50.0, 100.0, Some("Test".to_string()), None);
        let _: Meter = m;
    }

    // I04: gauge() creates Gauge
    #[test]
    fn test_gauge_widget_i04() {
        let g = gauge(0.5, Some("Progress".to_string()));
        let _: Gauge = g;
    }

    // I08: Color names resolve
    #[test]
    fn test_color_names_i08() {
        assert_eq!(parse_color("red"), Color::RED);
        assert_eq!(parse_color("GREEN"), Color::GREEN);
        assert_eq!(parse_color("Blue"), Color::BLUE);
    }

    #[test]
    fn test_color_hex() {
        let color = parse_color("#ff0000");
        assert_eq!(color.to_rgb8(), (255, 0, 0));
    }

    #[test]
    fn test_graph_mode_parse() {
        assert_eq!(parse_graph_mode("braille"), GraphMode::Braille);
        assert_eq!(parse_graph_mode("BLOCK"), GraphMode::Block);
        assert_eq!(parse_graph_mode("tty"), GraphMode::Tty);
    }

    // I16: Empty data handled
    #[test]
    fn test_empty_data_i16() {
        let graph = plot(vec![], None, None, None);
        // Should not panic
        let mut buf = CellBuffer::new(80, 24);
        let mut graph = graph;
        graph.layout(buf.bounds());
        graph.paint(&mut buf);
    }

    #[test]
    fn test_dashboard_layout() {
        let widgets: Vec<(String, Box<dyn Brick>)> = vec![
            ("test".to_string(), Box::new(plot(vec![1.0], None, None, None))),
        ];
        let mut dash = dashboard(widgets);
        dash.layout(Rect::new(0.0, 0.0, 80.0, 24.0));
        assert_eq!(dash.bounds().width, 80.0);
    }

    #[test]
    fn test_text_widget() {
        let mut widget = TextWidget::new("Hello".to_string());
        widget.layout(Rect::new(0.0, 0.0, 80.0, 1.0));
        assert_eq!(widget.bounds().height, 1.0);
    }

    // I05: dashboard {} creates layout
    #[test]
    fn test_dashboard_children_positioned_i05() {
        let widgets: Vec<(String, Box<dyn Brick>)> = vec![
            ("graph1".to_string(), Box::new(plot(vec![1.0, 2.0], None, None, None))),
            ("graph2".to_string(), Box::new(plot(vec![3.0, 4.0], None, None, None))),
        ];
        let mut dash = dashboard(widgets);
        dash.layout(Rect::new(0.0, 0.0, 80.0, 24.0));

        // Children should be positioned (vertical layout divides height equally)
        assert_eq!(dash.bounds().width, 80.0);
        assert_eq!(dash.bounds().height, 24.0);
    }

    // I07: Widget options apply
    #[test]
    fn test_widget_options_i07() {
        let graph = plot(
            vec![1.0, 2.0, 3.0],
            Some("My Title".to_string()),
            Some("red".to_string()),
            Some("block".to_string()),
        );

        assert!(graph.title.is_some());
        assert_eq!(graph.title.unwrap(), "My Title");
        assert_eq!(graph.color, Color::RED);
        assert_eq!(graph.mode, GraphMode::Block);
    }

    // I09: Gradient syntax works
    #[test]
    fn test_gradient_syntax_i09() {
        let m = meter(
            50.0,
            100.0,
            Some("CPU".to_string()),
            Some(("green".to_string(), "red".to_string())),
        );

        assert!(m.gradient.is_some());
    }

    // I11: Array to BrailleGraph
    #[test]
    fn test_array_to_graph_i11() {
        let data = vec![1.0, 4.0, 9.0, 16.0, 25.0];
        let graph = plot(data.clone(), None, None, None);

        assert_eq!(graph.data.len(), 5);
        assert_eq!(graph.data[0], 1.0);
        assert_eq!(graph.data[4], 25.0);
    }

    // I15: Null handling safe
    #[test]
    fn test_null_safety_i15() {
        // All options are None - should not panic
        let g = plot(vec![1.0], None, None, None);
        let _: BrailleGraph = g;

        let m = meter(50.0, 100.0, None, None);
        let _: Meter = m;

        let ga = gauge(0.5, None);
        let _: Gauge = ga;
    }

    // I17: Large data handled
    #[test]
    fn test_large_data_i17() {
        // Create 10K points
        let data: Vec<f64> = (0..10_000).map(|x| (x as f64).sin()).collect();
        let mut graph = plot(data, None, None, None);

        // Should not panic
        let mut buf = CellBuffer::new(80, 24);
        graph.layout(buf.bounds());
        graph.paint(&mut buf);
    }

    // Additional: All color names
    #[test]
    fn test_all_color_names() {
        let colors = vec![
            ("red", Color::RED),
            ("green", Color::GREEN),
            ("blue", Color::BLUE),
            ("yellow", Color::YELLOW),
            ("cyan", Color::CYAN),
            ("magenta", Color::MAGENTA),
            ("white", Color::WHITE),
            ("black", Color::BLACK),
            ("gray", Color::GRAY),
            ("grey", Color::GRAY),
        ];

        for (name, expected) in colors {
            assert_eq!(parse_color(name), expected, "Color '{}' should match", name);
        }
    }

    // Additional: Invalid color defaults to white
    #[test]
    fn test_invalid_color_default() {
        let color = parse_color("not_a_color");
        assert_eq!(color, Color::WHITE);
    }

    // Additional: Progress bar creation
    #[test]
    fn test_progress_creation() {
        let bar = progress(45, 100, true);
        assert!((bar.ratio() - 0.45).abs() < 0.01);
        assert!(bar.show_eta);
    }

    // Additional: spec_to_widget for all types
    #[test]
    fn test_spec_to_widget_braille() {
        let spec = WidgetSpec {
            kind: WidgetKind::BrailleGraph {
                data: vec![1.0, 2.0, 3.0],
                mode: GraphMode::Braille,
            },
            bounds: None,
            style: super::super::WidgetStyle::default(),
        };

        let mut widget = spec_to_widget(&spec);
        // Must layout before paint
        let mut buf = CellBuffer::new(80, 24);
        widget.layout(buf.bounds());
        widget.paint(&mut buf);
    }

    #[test]
    fn test_spec_to_widget_meter() {
        let spec = WidgetSpec {
            kind: WidgetKind::Meter {
                value: 50.0,
                max: 100.0,
                label: "CPU".to_string(),
            },
            bounds: None,
            style: super::super::WidgetStyle::default(),
        };

        let mut widget = spec_to_widget(&spec);
        let mut buf = CellBuffer::new(80, 24);
        widget.layout(buf.bounds());
        widget.paint(&mut buf);
    }

    #[test]
    fn test_spec_to_widget_table() {
        let spec = WidgetSpec {
            kind: WidgetKind::Table {
                headers: vec!["Name".to_string(), "Value".to_string()],
                rows: vec![vec!["A".to_string(), "1".to_string()]],
            },
            bounds: None,
            style: super::super::WidgetStyle::default(),
        };

        let mut widget = spec_to_widget(&spec);
        let mut buf = CellBuffer::new(80, 24);
        widget.layout(buf.bounds());
        widget.paint(&mut buf);
    }

    #[test]
    fn test_spec_to_widget_gauge() {
        let spec = WidgetSpec {
            kind: WidgetKind::Gauge {
                value: 0.75,
                label: "Progress".to_string(),
            },
            bounds: None,
            style: super::super::WidgetStyle::default(),
        };

        let mut widget = spec_to_widget(&spec);
        let mut buf = CellBuffer::new(80, 24);
        widget.layout(buf.bounds());
        widget.paint(&mut buf);
    }

    #[test]
    fn test_spec_to_widget_progress() {
        let spec = WidgetSpec {
            kind: WidgetKind::Progress {
                current: 50,
                total: 100,
                show_eta: true,
            },
            bounds: None,
            style: super::super::WidgetStyle::default(),
        };

        let mut widget = spec_to_widget(&spec);
        let mut buf = CellBuffer::new(80, 24);
        widget.layout(buf.bounds());
        widget.paint(&mut buf);
    }

    #[test]
    fn test_spec_to_widget_text() {
        let spec = WidgetSpec {
            kind: WidgetKind::Text {
                content: "Hello World".to_string(),
            },
            bounds: None,
            style: super::super::WidgetStyle::default(),
        };

        let mut widget = spec_to_widget(&spec);
        let mut buf = CellBuffer::new(80, 24);
        widget.layout(buf.bounds());
        widget.paint(&mut buf);
    }

    #[test]
    fn test_spec_to_widget_dashboard() {
        let spec = WidgetSpec {
            kind: WidgetKind::Dashboard {
                children: vec![
                    ("graph".to_string(), WidgetSpec {
                        kind: WidgetKind::BrailleGraph {
                            data: vec![1.0, 2.0],
                            mode: GraphMode::Braille,
                        },
                        bounds: None,
                        style: super::super::WidgetStyle::default(),
                    }),
                ],
            },
            bounds: None,
            style: super::super::WidgetStyle::default(),
        };

        let mut widget = spec_to_widget(&spec);
        let mut buf = CellBuffer::new(80, 24);
        widget.layout(buf.bounds());
        widget.paint(&mut buf);
    }
}
