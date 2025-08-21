// Display formatting helpers for Value - extracted to reduce complexity

use super::Value;
use colored::Colorize;
use std::fmt;

impl Value {
    pub(super) fn format_dataframe(
        f: &mut fmt::Formatter<'_>,
        columns: &[super::DataFrameColumn],
    ) -> fmt::Result {
        if columns.is_empty() {
            return write!(f, "Empty DataFrame");
        }

        let col_widths = calculate_column_widths(columns);
        let num_rows = get_max_rows(columns);

        write_table_top_border(f, &col_widths)?;
        write_table_headers(f, columns, &col_widths)?;
        write_table_header_separator(f, &col_widths)?;
        write_table_rows(f, columns, &col_widths, num_rows)?;
        write_table_bottom_border(f, &col_widths)?;
        write!(f, "\n{num_rows} rows × {} columns", columns.len())
    }

    pub(super) fn fmt_list(f: &mut fmt::Formatter<'_>, items: &[Value]) -> fmt::Result {
        write!(f, "[")?;
        write_comma_separated(f, items)?;
        write!(f, "]")
    }

    pub(super) fn fmt_tuple(f: &mut fmt::Formatter<'_>, items: &[Value]) -> fmt::Result {
        write!(f, "(")?;
        write_comma_separated(f, items)?;
        write!(f, ")")
    }

    pub(super) fn fmt_object(
        f: &mut fmt::Formatter<'_>,
        map: &std::collections::HashMap<String, Value>,
    ) -> fmt::Result {
        write!(f, "{{")?;
        for (i, (key, value)) in map.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "\"{key}\": {value}")?;
        }
        write!(f, "}}")
    }

    pub(super) fn fmt_enum_variant(
        f: &mut fmt::Formatter<'_>,
        enum_name: &str,
        variant_name: &str,
        data: Option<&[Value]>,
    ) -> fmt::Result {
        write!(f, "{enum_name}::{variant_name}")?;
        if let Some(values) = data {
            write!(f, "(")?;
            write_comma_separated(f, values)?;
            write!(f, ")")
        } else {
            Ok(())
        }
    }
}

// Helper functions

fn calculate_column_widths(columns: &[super::DataFrameColumn]) -> Vec<usize> {
    columns
        .iter()
        .map(|col| {
            let header_width = col.name.len();
            let max_value_width = col
                .values
                .iter()
                .map(|v| format!("{v}").len())
                .max()
                .unwrap_or(0);
            header_width.max(max_value_width).max(4)
        })
        .collect()
}

fn get_max_rows(columns: &[super::DataFrameColumn]) -> usize {
    columns.iter().map(|c| c.values.len()).max().unwrap_or(0)
}

fn write_table_top_border(f: &mut fmt::Formatter<'_>, col_widths: &[usize]) -> fmt::Result {
    write!(f, "┌")?;
    for (i, width) in col_widths.iter().enumerate() {
        if i > 0 {
            write!(f, "┬")?;
        }
        write!(f, "{}", "─".repeat(width + 2))?;
    }
    writeln!(f, "┐")
}

fn write_table_headers(
    f: &mut fmt::Formatter<'_>,
    columns: &[super::DataFrameColumn],
    col_widths: &[usize],
) -> fmt::Result {
    write!(f, "│")?;
    for (i, (col, width)) in columns.iter().zip(col_widths).enumerate() {
        if i > 0 {
            write!(f, "│")?;
        }
        write!(
            f,
            " {:width$} ",
            col.name.bright_cyan().bold(),
            width = width
        )?;
    }
    writeln!(f, "│")
}

fn write_table_header_separator(f: &mut fmt::Formatter<'_>, col_widths: &[usize]) -> fmt::Result {
    write!(f, "├")?;
    for (i, width) in col_widths.iter().enumerate() {
        if i > 0 {
            write!(f, "┼")?;
        }
        write!(f, "{}", "─".repeat(width + 2))?;
    }
    writeln!(f, "┤")
}

fn write_table_rows(
    f: &mut fmt::Formatter<'_>,
    columns: &[super::DataFrameColumn],
    col_widths: &[usize],
    num_rows: usize,
) -> fmt::Result {
    for row_idx in 0..num_rows {
        write_table_row(f, columns, col_widths, row_idx)?;
    }
    Ok(())
}

fn write_table_row(
    f: &mut fmt::Formatter<'_>,
    columns: &[super::DataFrameColumn],
    col_widths: &[usize],
    row_idx: usize,
) -> fmt::Result {
    write!(f, "│")?;
    for (col_idx, (col, width)) in columns.iter().zip(col_widths).enumerate() {
        if col_idx > 0 {
            write!(f, "│")?;
        }
        write_table_cell(f, col, *width, row_idx)?;
    }
    writeln!(f, "│")
}

fn write_table_cell(
    f: &mut fmt::Formatter<'_>,
    col: &super::DataFrameColumn,
    width: usize,
    row_idx: usize,
) -> fmt::Result {
    if row_idx < col.values.len() {
        let formatted = format_cell_value(&col.values[row_idx]);
        write!(f, " {formatted:width$} ")
    } else {
        write!(f, " {:<width$} ", "")
    }
}

fn format_cell_value(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{s}\"").bright_green().to_string(),
        Value::Int(n) => n.to_string().bright_blue().to_string(),
        Value::Float(n) => n.to_string().bright_blue().to_string(),
        Value::Bool(b) => b.to_string().bright_yellow().to_string(),
        other => format!("{other}"),
    }
}

fn write_table_bottom_border(f: &mut fmt::Formatter<'_>, col_widths: &[usize]) -> fmt::Result {
    write!(f, "└")?;
    for (i, width) in col_widths.iter().enumerate() {
        if i > 0 {
            write!(f, "┴")?;
        }
        write!(f, "{}", "─".repeat(width + 2))?;
    }
    write!(f, "┘")
}

fn write_comma_separated(f: &mut fmt::Formatter<'_>, items: &[Value]) -> fmt::Result {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{item}")?;
    }
    Ok(())
}
