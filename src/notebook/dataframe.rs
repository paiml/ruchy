// NOTEBOOK-005: DataFrame HTML Rendering
// Phase 4: Notebook Excellence - Rich DataFrame Visualization
//
// This module provides DataFrame rendering as HTML tables:
// - Column type detection and styling
// - Row striping for readability
// - Sortable columns (future)
// - Pagination for large datasets (future)
//
// Quality Requirements:
// - Cyclomatic Complexity: ≤10 per function (Toyota Way)
// - Line Coverage: ≥85%
// - Branch Coverage: ≥90%

use crate::notebook::html::html_escape;

/// Column type for `DataFrame` rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    /// Integer numbers
    Integer,
    /// Floating point numbers
    Float,
    /// String/text data
    String,
    /// Boolean values
    Boolean,
    /// Unknown/mixed type
    Unknown,
}

/// A simple `DataFrame` representation for HTML rendering
///
/// # Examples
///
/// ```
/// use ruchy::notebook::dataframe::DataFrame;
///
/// let df = DataFrame::new(
///     vec!["Name".to_string(), "Age".to_string()],
///     vec![
///         vec!["Alice".to_string(), "30".to_string()],
///         vec!["Bob".to_string(), "25".to_string()],
///     ]
/// );
///
/// assert_eq!(df.row_count(), 2);
/// assert_eq!(df.column_count(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct DataFrame {
    /// Column headers
    columns: Vec<String>,
    /// Data rows
    rows: Vec<Vec<String>>,
    /// Column types (detected or specified)
    column_types: Vec<ColumnType>,
}

impl DataFrame {
    /// Create a new `DataFrame` with auto-detected column types
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::dataframe::DataFrame;
    ///
    /// let df = DataFrame::new(
    ///     vec!["ID".to_string(), "Value".to_string()],
    ///     vec![vec!["1".to_string(), "100".to_string()]]
    /// );
    ///
    /// assert_eq!(df.column_count(), 2);
    /// ```
    pub fn new(columns: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        let column_types = Self::detect_column_types(&rows, columns.len());
        Self {
            columns,
            rows,
            column_types,
        }
    }

    /// Create a `DataFrame` with explicit column types
    pub fn with_types(
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
        column_types: Vec<ColumnType>,
    ) -> Self {
        Self {
            columns,
            rows,
            column_types,
        }
    }

    /// Get the number of rows
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get the number of columns
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Get column headers
    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// Get rows
    pub fn rows(&self) -> &[Vec<String>] {
        &self.rows
    }

    /// Get column types
    pub fn column_types(&self) -> &[ColumnType] {
        &self.column_types
    }

    /// Check if `DataFrame` is empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Render `DataFrame` as HTML table
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::dataframe::DataFrame;
    ///
    /// let df = DataFrame::new(
    ///     vec!["Name".to_string()],
    ///     vec![vec!["Alice".to_string()]]
    /// );
    ///
    /// let html = df.to_html();
    /// assert!(html.contains("<table"));
    /// assert!(html.contains("Alice"));
    /// ```
    pub fn to_html(&self) -> String {
        let mut html = String::from(r#"<div class="dataframe-container">"#);
        html.push_str(r#"<table class="dataframe">"#);

        // Header row
        html.push_str("<thead><tr>");
        for (i, col) in self.columns.iter().enumerate() {
            let type_class = self.column_type_class(i);
            html.push_str(&format!(
                r#"<th class="{}">{}</th>"#,
                type_class,
                html_escape(col)
            ));
        }
        html.push_str("</tr></thead>");

        // Data rows
        html.push_str("<tbody>");
        for (row_idx, row) in self.rows.iter().enumerate() {
            let row_class = if row_idx % 2 == 0 {
                "even-row"
            } else {
                "odd-row"
            };
            html.push_str(&format!(r#"<tr class="{row_class}">"#));

            for (col_idx, cell) in row.iter().enumerate() {
                let type_class = self.column_type_class(col_idx);
                html.push_str(&format!(
                    r#"<td class="{}">{}</td>"#,
                    type_class,
                    html_escape(cell)
                ));
            }
            html.push_str("</tr>");
        }
        html.push_str("</tbody>");

        html.push_str("</table>");

        // Add summary info
        html.push_str(&format!(
            r#"<div class="dataframe-info">{} rows × {} columns</div>"#,
            self.row_count(),
            self.column_count()
        ));

        html.push_str("</div>");
        html
    }

    /// Detect column types from data
    fn detect_column_types(rows: &[Vec<String>], col_count: usize) -> Vec<ColumnType> {
        (0..col_count)
            .map(|col_idx| Self::detect_column_type(rows, col_idx))
            .collect()
    }

    /// Detect type of a single column
    fn detect_column_type(rows: &[Vec<String>], col_idx: usize) -> ColumnType {
        if rows.is_empty() {
            return ColumnType::Unknown;
        }

        let mut is_integer = true;
        let mut is_float = true;
        let mut is_boolean = true;

        for row in rows {
            if col_idx >= row.len() {
                continue;
            }

            let cell = &row[col_idx];

            // Check integer
            if is_integer && cell.parse::<i64>().is_err() {
                is_integer = false;
            }

            // Check float
            if is_float && cell.parse::<f64>().is_err() {
                is_float = false;
            }

            // Check boolean
            if is_boolean {
                let lower = cell.to_lowercase();
                if lower != "true" && lower != "false" {
                    is_boolean = false;
                }
            }
        }

        if is_integer {
            ColumnType::Integer
        } else if is_float {
            ColumnType::Float
        } else if is_boolean {
            ColumnType::Boolean
        } else {
            ColumnType::String
        }
    }

    /// Get CSS class for column type
    fn column_type_class(&self, col_idx: usize) -> &str {
        if col_idx >= self.column_types.len() {
            return "type-unknown";
        }

        match self.column_types[col_idx] {
            ColumnType::Integer => "type-integer",
            ColumnType::Float => "type-float",
            ColumnType::String => "type-string",
            ColumnType::Boolean => "type-boolean",
            ColumnType::Unknown => "type-unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write tests that define expected behavior

    #[test]
    fn test_notebook_005_dataframe_creation() {
        let df = DataFrame::new(
            vec!["Name".to_string(), "Age".to_string()],
            vec![vec!["Alice".to_string(), "30".to_string()]],
        );

        assert_eq!(df.row_count(), 1);
        assert_eq!(df.column_count(), 2);
        assert!(!df.is_empty());
    }

    #[test]
    fn test_notebook_005_empty_dataframe() {
        let df = DataFrame::new(vec!["A".to_string(), "B".to_string()], vec![]);

        assert_eq!(df.row_count(), 0);
        assert_eq!(df.column_count(), 2);
        assert!(df.is_empty());
    }

    #[test]
    fn test_notebook_005_column_type_detection_integer() {
        let df = DataFrame::new(
            vec!["ID".to_string()],
            vec![
                vec!["1".to_string()],
                vec!["2".to_string()],
                vec!["3".to_string()],
            ],
        );

        assert_eq!(df.column_types()[0], ColumnType::Integer);
    }

    #[test]
    fn test_notebook_005_column_type_detection_float() {
        let df = DataFrame::new(
            vec!["Price".to_string()],
            vec![
                vec!["1.5".to_string()],
                vec!["2.7".to_string()],
                vec!["3.9".to_string()],
            ],
        );

        assert_eq!(df.column_types()[0], ColumnType::Float);
    }

    #[test]
    fn test_notebook_005_column_type_detection_string() {
        let df = DataFrame::new(
            vec!["Name".to_string()],
            vec![
                vec!["Alice".to_string()],
                vec!["Bob".to_string()],
                vec!["Charlie".to_string()],
            ],
        );

        assert_eq!(df.column_types()[0], ColumnType::String);
    }

    #[test]
    fn test_notebook_005_column_type_detection_boolean() {
        let df = DataFrame::new(
            vec!["Active".to_string()],
            vec![
                vec!["true".to_string()],
                vec!["false".to_string()],
                vec!["true".to_string()],
            ],
        );

        assert_eq!(df.column_types()[0], ColumnType::Boolean);
    }

    #[test]
    fn test_notebook_005_to_html_basic() {
        let df = DataFrame::new(
            vec!["Name".to_string(), "Age".to_string()],
            vec![
                vec!["Alice".to_string(), "30".to_string()],
                vec!["Bob".to_string(), "25".to_string()],
            ],
        );

        let html = df.to_html();

        assert!(html.contains("<table"));
        assert!(html.contains("Name"));
        assert!(html.contains("Alice"));
        assert!(html.contains("30"));
        assert!(html.contains("2 rows × 2 columns"));
    }

    #[test]
    fn test_notebook_005_html_with_type_classes() {
        let df = DataFrame::new(
            vec!["ID".to_string(), "Name".to_string()],
            vec![vec!["1".to_string(), "Alice".to_string()]],
        );

        let html = df.to_html();

        assert!(html.contains("type-integer"));
        assert!(html.contains("type-string"));
    }

    #[test]
    fn test_notebook_005_html_row_striping() {
        let df = DataFrame::new(
            vec!["Value".to_string()],
            vec![
                vec!["A".to_string()],
                vec!["B".to_string()],
                vec!["C".to_string()],
            ],
        );

        let html = df.to_html();

        assert!(html.contains("even-row"));
        assert!(html.contains("odd-row"));
    }

    #[test]
    fn test_notebook_005_html_escapes_content() {
        let df = DataFrame::new(
            vec!["Data".to_string()],
            vec![vec!["<script>alert('xss')</script>".to_string()]],
        );

        let html = df.to_html();

        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_notebook_005_dataframe_with_explicit_types() {
        let df = DataFrame::with_types(
            vec!["Col1".to_string()],
            vec![vec!["data".to_string()]],
            vec![ColumnType::String],
        );

        assert_eq!(df.column_types()[0], ColumnType::String);
    }

    #[test]
    fn test_notebook_005_column_type_debug() {
        let types = vec![
            ColumnType::Integer,
            ColumnType::Float,
            ColumnType::String,
            ColumnType::Boolean,
            ColumnType::Unknown,
        ];

        for t in types {
            let debug_str = format!("{:?}", t);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_notebook_005_dataframe_clone() {
        let df = DataFrame::new(vec!["A".to_string()], vec![vec!["1".to_string()]]);

        let cloned = df.clone();

        assert_eq!(df.row_count(), cloned.row_count());
        assert_eq!(df.column_count(), cloned.column_count());
    }

    #[test]
    fn test_notebook_005_large_dataframe() {
        let mut rows = Vec::new();
        for i in 0..100 {
            rows.push(vec![i.to_string(), format!("Item {}", i)]);
        }

        let df = DataFrame::new(vec!["ID".to_string(), "Name".to_string()], rows);

        assert_eq!(df.row_count(), 100);
        assert!(df.to_html().contains("100 rows × 2 columns"));
    }

    #[test]
    fn test_notebook_005_unicode_in_dataframe() {
        let df = DataFrame::new(
            vec!["Language".to_string(), "Hello".to_string()],
            vec![
                vec!["English".to_string(), "Hello".to_string()],
                vec!["Japanese".to_string(), "こんにちは".to_string()],
                vec!["Greek".to_string(), "Γειά σου".to_string()],
            ],
        );

        let html = df.to_html();

        assert!(html.contains("こんにちは"));
        assert!(html.contains("Γειά σου"));
    }

    // Property-based tests for DataFrame rendering robustness
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_notebook_005_property_html_never_contains_unescaped_tags(
                columns in prop::collection::vec(any::<String>(), 1..5),
                rows in prop::collection::vec(
                    prop::collection::vec(any::<String>(), 1..5),
                    0..10
                )
            ) {
                let df = DataFrame::new(columns, rows);
                let html = df.to_html();

                // Should not contain raw < or > outside of HTML tags
                let content_between_tags = html.split('>').collect::<Vec<_>>();
                for content in content_between_tags {
                    if let Some(text) = content.split('<').next() {
                        // Text content should not contain unescaped < or >
                        if !text.is_empty() && !text.starts_with("div") && !text.starts_with("table")
                            && !text.starts_with("thead") && !text.starts_with("tbody")
                            && !text.starts_with("tr") && !text.starts_with("th")
                            && !text.starts_with("td") && !text.starts_with('/') {
                            prop_assert!(!text.contains("<script>"));
                            prop_assert!(!text.contains("</script>"));
                        }
                    }
                }
            }

            #[test]
            fn test_notebook_005_property_row_count_matches(
                columns in prop::collection::vec("[a-z]{3,8}".prop_map(|s| s.to_string()), 1..5),
                row_count in 0usize..20
            ) {
                let rows: Vec<Vec<String>> = (0..row_count)
                    .map(|i| {
                        columns.iter().map(|_| i.to_string()).collect()
                    })
                    .collect();

                let df = DataFrame::new(columns, rows);
                prop_assert_eq!(df.row_count(), row_count);

                let html = df.to_html();
                let expected = format!("{} rows", row_count);
                prop_assert!(html.contains(&expected));
            }

            #[test]
            fn test_notebook_005_property_column_count_matches(
                column_count in 1usize..10,
                row_count in 0usize..10
            ) {
                let columns: Vec<String> = (0..column_count)
                    .map(|i| format!("col{}", i))
                    .collect();

                let rows: Vec<Vec<String>> = (0..row_count)
                    .map(|i| {
                        (0..column_count).map(|j| format!("{}_{}", i, j)).collect()
                    })
                    .collect();

                let df = DataFrame::new(columns, rows);
                prop_assert_eq!(df.column_count(), column_count);

                let html = df.to_html();
                let expected = format!("× {} columns", column_count);
                prop_assert!(html.contains(&expected));
            }

            #[test]
            fn test_notebook_005_property_dangerous_strings_escaped(
                dangerous_input in prop::sample::select(vec![
                    "<script>alert('xss')</script>",
                    "<img src=x onerror=alert(1)>",
                    "'; DROP TABLE users; --",
                    "<iframe src='javascript:alert(1)'>",
                    "' OR '1'='1",
                ])
            ) {
                let df = DataFrame::new(
                    vec!["Data".to_string()],
                    vec![vec![dangerous_input.to_string()]],
                );

                let html = df.to_html();

                // Should contain escaped versions
                prop_assert!(html.contains("&lt;") || html.contains("&gt;") || html.contains("&quot;") || html.contains("&#39;"));

                // Should NOT contain unescaped dangerous content
                prop_assert!(!html.contains("<script>"));
                prop_assert!(!html.contains("<img"));
                prop_assert!(!html.contains("<iframe"));
            }

            #[test]
            fn test_notebook_005_property_integer_detection_consistent(
                integers in prop::collection::vec(-1000i64..1000i64, 1..20)
            ) {
                let rows: Vec<Vec<String>> = integers
                    .iter()
                    .map(|i| vec![i.to_string()])
                    .collect();

                let df = DataFrame::new(vec!["Numbers".to_string()], rows);

                prop_assert_eq!(df.column_types()[0], ColumnType::Integer);

                let html = df.to_html();
                prop_assert!(html.contains("type-integer"));
            }

            #[test]
            fn test_notebook_005_property_float_detection_consistent(
                floats in prop::collection::vec(-1000.0f64..1000.0, 1..20)
            ) {
                let rows: Vec<Vec<String>> = floats
                    .iter()
                    .map(|f| vec![f.to_string()])
                    .collect();

                let df = DataFrame::new(vec!["Floats".to_string()], rows);

                prop_assert_eq!(df.column_types()[0], ColumnType::Float);

                let html = df.to_html();
                prop_assert!(html.contains("type-float"));
            }

            #[test]
            fn test_notebook_005_property_boolean_detection_consistent(
                booleans in prop::collection::vec(any::<bool>(), 1..20)
            ) {
                let rows: Vec<Vec<String>> = booleans
                    .iter()
                    .map(|b| vec![b.to_string()])
                    .collect();

                let df = DataFrame::new(vec!["Flags".to_string()], rows);

                prop_assert_eq!(df.column_types()[0], ColumnType::Boolean);

                let html = df.to_html();
                prop_assert!(html.contains("type-boolean"));
            }

            #[test]
            fn test_notebook_005_property_row_striping_alternates(
                row_count in 2usize..50
            ) {
                let rows: Vec<Vec<String>> = (0..row_count)
                    .map(|i| vec![i.to_string()])
                    .collect();

                let df = DataFrame::new(vec!["Index".to_string()], rows);
                let html = df.to_html();

                // Should have both even and odd row classes (for row_count >= 2)
                prop_assert!(html.contains("even-row"));
                prop_assert!(html.contains("odd-row"));
            }

            #[test]
            fn test_notebook_005_property_html_structure_valid(
                columns in prop::collection::vec("[a-z]{3,8}".prop_map(|s| s.to_string()), 1..5),
                row_count in 0usize..10
            ) {
                let rows: Vec<Vec<String>> = (0..row_count)
                    .map(|i| {
                        columns.iter().map(|_| format!("val_{}", i)).collect()
                    })
                    .collect();

                let df = DataFrame::new(columns, rows);
                let html = df.to_html();

                // Verify basic HTML structure
                prop_assert!(html.contains("<div class=\"dataframe-container\">"));
                prop_assert!(html.contains("<table class=\"dataframe\">"));
                prop_assert!(html.contains("<thead>"));
                prop_assert!(html.contains("<tbody>"));
                prop_assert!(html.contains("</table>"));
                prop_assert!(html.contains("</div>"));
            }

            #[test]
            fn test_notebook_005_property_empty_dataframe_renders(
                columns in prop::collection::vec("[a-z]{3,8}".prop_map(|s| s.to_string()), 1..5)
            ) {
                let df = DataFrame::new(columns.clone(), vec![]);

                prop_assert!(df.is_empty());
                prop_assert_eq!(df.row_count(), 0);
                prop_assert_eq!(df.column_count(), columns.len());

                let html = df.to_html();
                prop_assert!(html.contains("0 rows"));
                prop_assert!(html.contains("<table"));
            }
        }
    }
}
