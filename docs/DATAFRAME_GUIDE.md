# DataFrame User Guide - Ruchy v3.64.0

## Overview

DataFrames in Ruchy provide powerful data manipulation capabilities for data science and analytics workflows. All DataFrame operations maintain <10 cyclomatic complexity and are fully tested.

## Table of Contents

- [Creating DataFrames](#creating-dataframes)
- [Importing Data](#importing-data)
- [Inspecting DataFrames](#inspecting-dataframes)
- [Transforming Data](#transforming-data)
- [Sorting](#sorting)
- [Method Chaining](#method-chaining)
- [API Reference](#api-reference)

---

## Creating DataFrames

### Builder Pattern

The recommended way to create DataFrames programmatically:

```ruchy
let df = DataFrame::new()
    .column("name", ["Alice", "Bob", "Charlie"])
    .column("age", [25, 30, 35])
    .column("salary", [75000, 85000, 95000])
    .build();
```

### Empty DataFrame

```ruchy
let empty = DataFrame::new().build();
```

---

## Importing Data

### CSV Import

Import CSV data with automatic type inference:

```ruchy
let csv_data = "product,quantity,price
Widget,10,99.99
Gadget,5,149.99
Tool,15,79.99";

let df = DataFrame::from_csv_string(csv_data);
```

**Type Inference:**
- Integers: `42`, `-7`
- Floats: `3.14`, `-2.5`
- Strings: Everything else

### JSON Import

Import JSON arrays of objects:

```ruchy
let json_data = '[
    {"name": "Alice", "age": 25},
    {"name": "Bob", "age": 30}
]';

let df = DataFrame::from_json(json_data);
```

---

## Inspecting DataFrames

### Get Row Count

```ruchy
let count = df.rows();
println("Total rows:", count);
```

### Get Column Count

```ruchy
let num_cols = df.columns();
println("Total columns:", num_cols);
```

### Get Column Names

```ruchy
let names = df.column_names();
println("Columns:", names);
```

### Display DataFrame

```ruchy
println(df);
// Output:
// DataFrame with 3 columns:
//   name: 3 rows
//   age: 3 rows
//   salary: 3 rows
```

---

## Transforming Data

### Add Computed Column

Add a new column based on existing data:

```ruchy
// Using row object (access multiple columns)
let with_bonus = df.with_column("bonus", row => row["salary"] * 0.1);

// Using column name (direct access to single column)
let df2 = DataFrame::new()
    .column("x", [1, 2, 3])
    .build();
let doubled = df2.with_column("y", x => x * 2);
```

**Smart Closure Binding:**
- If parameter name matches a column: binds that column's value
- Otherwise: binds full row object

```ruchy
// Parameter "x" matches column "x" → direct value
df.with_column("y", x => x * 2)

// Parameter "row" doesn't match → full row object
df.with_column("total", row => row["price"] * row["qty"])
```

### Transform Existing Column

Modify a column in-place:

```ruchy
// Double all prices
let updated = df.transform("price", p => p * 2);

// Apply discount
let discounted = df.transform("price", p => p * 0.9);
```

### Filter Rows (Existing Feature)

Note: Filter uses `col()` syntax currently:

```ruchy
let expensive = df.filter(col("price") > 100);
```

---

## Sorting

### Sort Ascending

```ruchy
let sorted = df.sort_by("age");
```

### Sort Descending

```ruchy
let sorted_desc = df.sort_by("salary", true);
```

**Sort Features:**
- Index-based: Maintains row integrity across all columns
- Type support: Integers, floats, strings, booleans
- Stable: Preserves order of equal elements

---

## Method Chaining

Chain operations for powerful data pipelines:

```ruchy
let result = df
    .with_column("tax", row => row["price"] * 0.08)
    .with_column("total", row => row["price"] + row["tax"])
    .sort_by("total", true);
```

### Real-World Example: Sales Analysis

```ruchy
let sales_csv = "date,product,quantity,price
2024-01-15,Widget,10,99.99
2024-01-16,Gadget,5,149.99
2024-01-17,Widget,8,99.99
2024-01-18,Tool,12,79.99";

let analysis = DataFrame::from_csv_string(sales_csv)
    .with_column("revenue", row => row["quantity"] * row["price"])
    .with_column("category", row => {
        if row["price"] > 100 then "Premium" else "Standard"
    })
    .sort_by("revenue", true);

println("Top sales by revenue:");
println(analysis);
```

---

## API Reference

### Construction

| Method | Description | Example |
|--------|-------------|---------|
| `DataFrame::new()` | Start builder | `DataFrame::new()` |
| `.column(name, values)` | Add column | `.column("age", [25, 30])` |
| `.build()` | Finalize DataFrame | `.build()` |

### Import

| Method | Description | Return |
|--------|-------------|--------|
| `DataFrame::from_csv_string(csv)` | Parse CSV with headers | `DataFrame` |
| `DataFrame::from_json(json)` | Parse JSON array | `DataFrame` |

### Inspection

| Method | Description | Return |
|--------|-------------|--------|
| `.rows()` | Get row count | `Integer` |
| `.columns()` | Get column count | `Integer` |
| `.column_names()` | Get column names | `Array<String>` |

### Transformation

| Method | Description | Return |
|--------|-------------|--------|
| `.with_column(name, closure)` | Add computed column | `DataFrame` |
| `.transform(name, closure)` | Modify column | `DataFrame` |
| `.sort_by(column, [desc])` | Sort rows | `DataFrame` |

### Closure Parameters

```ruchy
// Column-based (parameter name = column name)
.with_column("doubled", x => x * 2)

// Row-based (parameter name ≠ column name)
.with_column("total", row => row["price"] * row["qty"])
```

---

## Performance Characteristics

- **Construction**: O(n) where n = total elements
- **CSV/JSON Import**: O(n) where n = data size
- **Transform**: O(n) where n = number of rows
- **Sort**: O(n log n) where n = number of rows
- **Memory**: Column-oriented storage

---

## Testing

All DataFrame features have comprehensive TDD tests:

- 39 tests passing (100% of implemented features)
- Property tests for edge cases
- All functions <10 cyclomatic complexity (Toyota Way)

Run tests:
```bash
cargo test --test dataframe_literal_tests
cargo test --test dataframe_constructor_tests
cargo test --test dataframe_io_tests
cargo test --test dataframe_transform_tests
```

---

## Coming Soon (v3.65.0+)

Future enhancements planned:

- **Advanced Aggregations**: Chained `.agg()` for custom aggregations
- **Statistics**: `.mean()`, `.std()`, `.percentile()`, rolling windows
- **Polars Integration**: Performance optimization for large datasets
- **Enhanced Filter**: Closure-based filtering like `with_column`

---

## Best Practices

1. **Use Builder Pattern**: More readable than literal syntax for code
2. **Chain Operations**: Build pipelines for clarity
3. **Name Closures Intuitively**: Use column names or "row"
4. **Type Inference**: Let CSV import infer types automatically
5. **Method Chaining**: Compose operations for readability

## Examples

See `tests/dataframe_*_tests.rs` for comprehensive examples of all features.

---

**Version**: 3.64.0
**Status**: Production-ready (60% of planned features)
**Quality**: All functions <10 complexity, 100% tested
