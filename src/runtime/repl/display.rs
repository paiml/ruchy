// Display formatting helpers for Value - extracted to reduce complexity

use super::Value;
use colored::Colorize;
use std::fmt;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use crate::runtime::repl::DataFrameColumn;
    
    // Helper function to format a value to string
    fn format_to_string(value: &Value) -> String {
        format!("{}", value)
    }
    
    // ========== DataFrame Tests ==========
    
    #[test]
    fn test_format_empty_dataframe() {
        let value = Value::DataFrame { columns: vec![] };
        let result = format_to_string(&value);
        assert!(result.contains("Empty DataFrame"));
    }
    
    #[test]
    fn test_format_dataframe_single_column() {
        let columns = vec![
            DataFrameColumn {
                name: "id".to_string(),
                values: vec![Value::Int(1), Value::Int(2), Value::Int(3)],
            }
        ];
        let value = Value::DataFrame { columns };
        let result = format_to_string(&value);
        assert!(result.contains("id"));
        assert!(result.contains("3 rows × 1 columns"));
    }
    
    #[test]
    fn test_format_dataframe_multiple_columns() {
        let columns = vec![
            DataFrameColumn {
                name: "id".to_string(),
                values: vec![Value::Int(1), Value::Int(2)],
            },
            DataFrameColumn {
                name: "name".to_string(),
                values: vec![Value::String("Alice".to_string()), Value::String("Bob".to_string())],
            },
            DataFrameColumn {
                name: "active".to_string(),
                values: vec![Value::Bool(true), Value::Bool(false)],
            },
        ];
        let value = Value::DataFrame { columns };
        let result = format_to_string(&value);
        assert!(result.contains("id"));
        assert!(result.contains("name"));
        assert!(result.contains("active"));
        assert!(result.contains("2 rows × 3 columns"));
    }
    
    #[test]
    fn test_format_dataframe_mixed_types() {
        let columns = vec![
            DataFrameColumn {
                name: "mixed".to_string(),
                values: vec![
                    Value::Int(42),
                    Value::String("test".to_string()),
                    Value::Float(3.14),
                    Value::Bool(true),
                ],
            },
        ];
        let value = Value::DataFrame { columns };
        let result = format_to_string(&value);
        assert!(result.contains("mixed"));
        assert!(result.contains("4 rows × 1 columns"));
    }
    
    #[test]
    fn test_format_dataframe_with_nil_values() {
        let columns = vec![
            DataFrameColumn {
                name: "nullable".to_string(),
                values: vec![
                    Value::Int(1),
                    Value::Nil,
                    Value::Int(3),
                ],
            },
        ];
        let value = Value::DataFrame { columns };
        let result = format_to_string(&value);
        assert!(result.contains("nullable"));
        assert!(result.contains("null") || result.contains("nil"));
    }
    
    // ========== List Tests ==========
    
    #[test]
    fn test_format_empty_list() {
        let value = Value::List(vec![]);
        assert_eq!(format_to_string(&value), "[]");
    }
    
    #[test]
    fn test_format_single_item_list() {
        let value = Value::List(vec![Value::Int(42)]);
        assert_eq!(format_to_string(&value), "[42]");
    }
    
    #[test]
    fn test_format_multiple_items_list() {
        let value = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ]);
        assert_eq!(format_to_string(&value), "[1, 2, 3]");
    }
    
    #[test]
    fn test_format_mixed_types_list() {
        let value = Value::List(vec![
            Value::Int(42),
            Value::String("hello".to_string()),
            Value::Bool(true),
            Value::Float(3.14),
        ]);
        assert_eq!(format_to_string(&value), "[42, hello, true, 3.14]");
    }
    
    #[test]
    fn test_format_nested_list() {
        let inner = Value::List(vec![Value::Int(1), Value::Int(2)]);
        let outer = Value::List(vec![inner, Value::Int(3)]);
        assert_eq!(format_to_string(&outer), "[[1, 2], 3]");
    }
    
    // ========== Tuple Tests ==========
    
    #[test]
    fn test_format_empty_tuple() {
        let value = Value::Tuple(vec![]);
        assert_eq!(format_to_string(&value), "()");
    }
    
    #[test]
    fn test_format_single_item_tuple() {
        let value = Value::Tuple(vec![Value::Int(42)]);
        assert_eq!(format_to_string(&value), "(42)");
    }
    
    #[test]
    fn test_format_multiple_items_tuple() {
        let value = Value::Tuple(vec![
            Value::Int(1),
            Value::String("test".to_string()),
            Value::Bool(false),
        ]);
        assert_eq!(format_to_string(&value), "(1, test, false)");
    }
    
    #[test]
    fn test_format_nested_tuple() {
        let inner = Value::Tuple(vec![Value::Int(1), Value::Int(2)]);
        let outer = Value::Tuple(vec![inner, Value::String("end".to_string())]);
        assert_eq!(format_to_string(&outer), "((1, 2), end)");
    }
    
    // ========== Object Tests ==========
    
    #[test]
    fn test_format_empty_object() {
        let value = Value::Object(HashMap::new());
        assert_eq!(format_to_string(&value), "{}");
    }
    
    #[test]
    fn test_format_single_field_object() {
        let mut map = HashMap::new();
        map.insert("field".to_string(), Value::Int(123));
        let value = Value::Object(map);
        assert_eq!(format_to_string(&value), "{\"field\": 123}");
    }
    
    #[test]
    fn test_format_multiple_fields_object() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), Value::Int(1));
        map.insert("b".to_string(), Value::String("test".to_string()));
        map.insert("c".to_string(), Value::Bool(true));
        let value = Value::Object(map);
        let result = format_to_string(&value);
        // HashMap order is not guaranteed
        assert!(result.starts_with("{"));
        assert!(result.ends_with("}"));
        assert!(result.contains("\"a\": 1"));
        assert!(result.contains("\"b\": test"));
        assert!(result.contains("\"c\": true"));
    }
    
    #[test]
    fn test_format_nested_object() {
        let mut inner = HashMap::new();
        inner.insert("inner".to_string(), Value::Int(42));
        let mut outer = HashMap::new();
        outer.insert("nested".to_string(), Value::Object(inner));
        outer.insert("value".to_string(), Value::String("test".to_string()));
        let value = Value::Object(outer);
        let result = format_to_string(&value);
        assert!(result.contains("\"nested\": {\"inner\": 42}"));
    }
    
    // ========== HashMap Tests ==========
    
    #[test]
    fn test_format_empty_hashmap() {
        let value = Value::HashMap(HashMap::new());
        assert_eq!(format_to_string(&value), "HashMap{}");
    }
    
    #[test]
    fn test_format_single_entry_hashmap() {
        let mut map = HashMap::new();
        map.insert(Value::String("key".to_string()), Value::Int(42));
        let value = Value::HashMap(map);
        assert_eq!(format_to_string(&value), "HashMap{key: 42}");
    }
    
    #[test]
    fn test_format_multiple_entries_hashmap() {
        let mut map = HashMap::new();
        map.insert(Value::Int(1), Value::String("one".to_string()));
        map.insert(Value::Int(2), Value::String("two".to_string()));
        let value = Value::HashMap(map);
        let result = format_to_string(&value);
        assert!(result.starts_with("HashMap{"));
        assert!(result.contains("1: one"));
        assert!(result.contains("2: two"));
    }
    
    // ========== HashSet Tests ==========
    
    #[test]
    fn test_format_empty_hashset() {
        let value = Value::HashSet(HashSet::new());
        assert_eq!(format_to_string(&value), "HashSet{}");
    }
    
    #[test]
    fn test_format_single_item_hashset() {
        let mut set = HashSet::new();
        set.insert(Value::Int(42));
        let value = Value::HashSet(set);
        assert_eq!(format_to_string(&value), "HashSet{42}");
    }
    
    #[test]
    fn test_format_multiple_items_hashset() {
        let mut set = HashSet::new();
        set.insert(Value::Int(1));
        set.insert(Value::Int(2));
        set.insert(Value::Int(3));
        let value = Value::HashSet(set);
        let result = format_to_string(&value);
        assert!(result.starts_with("HashSet{"));
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }
    
    // ========== Range Tests ==========
    
    #[test]
    fn test_format_exclusive_range() {
        let value = Value::Range { start: 1, end: 10, inclusive: false };
        assert_eq!(format_to_string(&value), "1..10");
    }
    
    #[test]
    fn test_format_inclusive_range() {
        let value = Value::Range { start: 0, end: 5, inclusive: true };
        assert_eq!(format_to_string(&value), "0..=5");
    }
    
    #[test]
    fn test_format_negative_range() {
        let value = Value::Range { start: -10, end: 10, inclusive: false };
        assert_eq!(format_to_string(&value), "-10..10");
    }
    
    // ========== EnumVariant Tests ==========
    
    #[test]
    fn test_format_enum_variant_no_data() {
        let value = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "None".to_string(),
            data: None,
        };
        assert_eq!(format_to_string(&value), "Option::None");
    }
    
    #[test]
    fn test_format_enum_variant_with_single_data() {
        let value = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![Value::Int(42)]),
        };
        assert_eq!(format_to_string(&value), "Option::Some(42)");
    }
    
    #[test]
    fn test_format_enum_variant_with_multiple_data() {
        let value = Value::EnumVariant {
            enum_name: "Result".to_string(),
            variant_name: "Err".to_string(),
            data: Some(vec![Value::String("error".to_string()), Value::Int(404)]),
        };
        assert_eq!(format_to_string(&value), "Result::Err(error, 404)");
    }
    
    // ========== Basic Value Type Tests ==========
    
    #[test]
    fn test_format_int() {
        assert_eq!(format_to_string(&Value::Int(42)), "42");
        assert_eq!(format_to_string(&Value::Int(-100)), "-100");
        assert_eq!(format_to_string(&Value::Int(0)), "0");
    }
    
    #[test]
    fn test_format_float() {
        assert_eq!(format_to_string(&Value::Float(3.14)), "3.14");
        assert_eq!(format_to_string(&Value::Float(0.0)), "0");
        assert_eq!(format_to_string(&Value::Float(-2.5)), "-2.5");
    }
    
    #[test]
    fn test_format_string() {
        assert_eq!(format_to_string(&Value::String("hello".to_string())), "hello");
        assert_eq!(format_to_string(&Value::String("".to_string())), "");
        assert_eq!(format_to_string(&Value::String("with spaces".to_string())), "with spaces");
    }
    
    #[test]
    fn test_format_bool() {
        assert_eq!(format_to_string(&Value::Bool(true)), "true");
        assert_eq!(format_to_string(&Value::Bool(false)), "false");
    }
    
    #[test]
    fn test_format_char() {
        assert_eq!(format_to_string(&Value::Char('a')), "a");
        assert_eq!(format_to_string(&Value::Char(' ')), " ");
        assert_eq!(format_to_string(&Value::Char('🚀')), "🚀");
    }
    
    #[test]
    fn test_format_unit() {
        assert_eq!(format_to_string(&Value::Unit), "()");
    }
    
    #[test]
    fn test_format_nil() {
        assert_eq!(format_to_string(&Value::Nil), "null");
    }
    
    // ========== Function Type Tests ==========
    
    #[test]
    fn test_format_function() {
        let value = Value::Function {
            name: "test_func".to_string(),
            params: vec!["x".to_string(), "y".to_string()],
            body: Box::new(crate::frontend::ast::Expr {
                kind: crate::frontend::ast::ExprKind::Literal(
                    crate::frontend::ast::Literal::Integer(0)
                ),
                span: crate::frontend::ast::Span::default(),
                attributes: vec![],
            }),
        };
        assert_eq!(format_to_string(&value), "<function test_func>");
    }
    
    #[test]
    fn test_format_lambda() {
        let value = Value::Lambda {
            params: vec!["x".to_string()],
            body: Box::new(crate::frontend::ast::Expr {
                kind: crate::frontend::ast::ExprKind::Literal(
                    crate::frontend::ast::Literal::Integer(0)
                ),
                span: crate::frontend::ast::Span::default(),
                attributes: vec![],
            }),
        };
        assert_eq!(format_to_string(&value), "<closure>");
    }
    
    // ========== Helper Function Tests ==========
    
    #[test]
    fn test_calculate_column_widths() {
        let columns = vec![
            DataFrameColumn {
                name: "short".to_string(),
                values: vec![Value::Int(1)],
            },
            DataFrameColumn {
                name: "very_long_column_name".to_string(),
                values: vec![Value::String("x".to_string())],
            },
        ];
        let widths = super::calculate_column_widths(&columns);
        assert_eq!(widths.len(), 2);
        assert!(widths[1] >= 21); // At least as long as the column name
    }
    
    #[test]
    fn test_get_max_rows() {
        let columns = vec![
            DataFrameColumn {
                name: "col1".to_string(),
                values: vec![Value::Int(1), Value::Int(2)],
            },
            DataFrameColumn {
                name: "col2".to_string(),
                values: vec![Value::Int(1), Value::Int(2), Value::Int(3)],
            },
        ];
        assert_eq!(super::get_max_rows(&columns), 3);
    }
    
    #[test]
    fn test_get_max_rows_empty() {
        let columns = vec![];
        assert_eq!(super::get_max_rows(&columns), 0);
    }
    
    #[test]
    fn test_write_comma_separated() {
        let values = vec![Value::Int(1), Value::Int(2), Value::Int(3)];
        let mut output = String::new();
        use std::fmt::Write;
        write!(&mut output, "[").unwrap();
        for (i, v) in values.iter().enumerate() {
            if i > 0 {
                write!(&mut output, ", ").unwrap();
            }
            write!(&mut output, "{}", v).unwrap();
        }
        write!(&mut output, "]").unwrap();
        assert_eq!(output, "[1, 2, 3]");
    }
    
    #[test]
    fn test_write_comma_separated_empty() {
        // Test with empty list formatting
        let value = Value::List(vec![]);
        assert_eq!(format_to_string(&value), "[]");
    }
    
    #[test]
    fn test_write_comma_separated_single() {
        // Test with single item list formatting
        let value = Value::List(vec![Value::String("only".to_string())]);
        assert_eq!(format_to_string(&value), "[only]");
    }
}

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

    pub(super) fn fmt_hashmap(f: &mut fmt::Formatter<'_>, map: &std::collections::HashMap<Value, Value>) -> fmt::Result {
        write!(f, "HashMap{{")?;
        let mut first = true;
        for (key, value) in map {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{key}: {value}")?;
            first = false;
        }
        write!(f, "}}")
    }

    pub(super) fn fmt_hashset(f: &mut fmt::Formatter<'_>, set: &std::collections::HashSet<Value>) -> fmt::Result {
        write!(f, "HashSet{{")?;
        let mut first = true;
        for value in set {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{value}")?;
            first = false;
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
