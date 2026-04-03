# Sub-spec: World-Class Formatter — TDD Comment Tests & ExprKind Coverage

**Parent:** [world-class-formatter-spec.md](../world-class-formatter-spec.md)

---

#### Step 1.5: Extreme TDD for Comment Preservation

**Test Suite**: `tests/cli_contract_fmt_comments.rs`

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_fmt_preserves_line_comments() {
    // RED: Write failing test FIRST
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("comments.ruchy");

    let original = "// This is a comment\nlet x = 42";
    std::fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Must preserve comment
    assert!(formatted.contains("// This is a comment"),
            "Line comment was stripped!");
}

#[test]
fn test_fmt_preserves_block_comments() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("block_comments.ruchy");

    let original = "/* Block comment */\nlet x = 42";
    std::fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(formatted.contains("/* Block comment */"),
            "Block comment was stripped!");
}

#[test]
fn test_fmt_preserves_doc_comments() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("doc_comments.ruchy");

    let original = "/// Returns the sum\nfun add(a, b) { a + b }";
    std::fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(formatted.contains("/// Returns the sum"),
            "Doc comment was stripped!");
}

#[test]
fn test_fmt_preserves_trailing_comments() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("trailing.ruchy");

    let original = "let x = 42  // Important value";
    std::fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(formatted.contains("// Important value"),
            "Trailing comment was stripped!");
}

#[test]
fn test_fmt_preserves_multiple_comments() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("multiple.ruchy");

    let original = r#"
// Comment 1
// Comment 2
let x = 42  // Trailing

/* Block comment */
let y = x * 2
"#;
    std::fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // All comments must be present
    assert!(formatted.contains("// Comment 1"));
    assert!(formatted.contains("// Comment 2"));
    assert!(formatted.contains("// Trailing"));
    assert!(formatted.contains("/* Block comment */"));
}

#[test]
fn test_fmt_preserves_comment_positions() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("positions.ruchy");

    let original = r#"
// Before function
fun add(a, b) {
    // Inside function
    a + b  // End of line
}
// After function
"#;
    std::fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = formatted.lines().collect();

    // Verify comment order preserved
    let before_idx = lines.iter().position(|l| l.contains("Before function")).unwrap();
    let inside_idx = lines.iter().position(|l| l.contains("Inside function")).unwrap();
    let after_idx = lines.iter().position(|l| l.contains("After function")).unwrap();

    assert!(before_idx < inside_idx && inside_idx < after_idx,
            "Comment order not preserved!");
}
```

---

### Phase 2: Complete ExprKind Coverage (v3.90.0)

**Goal**: Implement ALL remaining ~58 ExprKind variants

#### Current Status (v3.88.0)
Implemented (27/85 variants):
- Literal, Identifier, Let, Binary, Block
- Function, If, Call, MethodCall, For
- IndexAccess, Assign, Return, FieldAccess
- While, Break, Continue, Range
- Unary, List, Tuple, Match
- CompoundAssign

Missing (~58 variants):
- Array, Object, TupleStruct
- Lambda, Closure, Async, Await
- Try, Throw, Catch
- Import, Export, Module
- Class, Trait, Impl
- Enum, Struct, Type
- Cast, As, Is
- Ref, Deref, Borrow
- Macro, Attribute
- And many more...

#### Implementation Strategy

**Pattern**: Systematic variant implementation with tests

```rust
// File: src/quality/formatter.rs

impl Formatter {
    fn format_expr_kind(&self, kind: &ExprKind, indent: usize) -> String {
        match kind {
            // Existing variants (27)...

            // NEW: Array literal
            ExprKind::Array { elements } => {
                if elements.is_empty() {
                    "[]".to_string()
                } else if self.fits_on_line(elements) {
                    format!(
                        "[{}]",
                        elements.iter()
                            .map(|e| self.format_expr(e, indent))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                } else {
                    format!(
                        "[\n{}\n{}]",
                        elements.iter()
                            .map(|e| format!("{}    {}", self.indent_string(indent), self.format_expr(e, indent + 1)))
                            .collect::<Vec<_>>()
                            .join(",\n"),
                        self.indent_string(indent)
                    )
                }
            }

            // NEW: Object literal
            ExprKind::Object { fields } => {
                if fields.is_empty() {
                    "{}".to_string()
                } else {
                    format!(
                        "{{\n{}\n{}}}",
                        fields.iter()
                            .map(|(k, v)| format!(
                                "{}    {}: {}",
                                self.indent_string(indent),
                                k,
                                self.format_expr(v, indent + 1)
                            ))
                            .collect::<Vec<_>>()
                            .join(",\n"),
                        self.indent_string(indent)
                    )
                }
            }

            // NEW: Lambda expression
            ExprKind::Lambda { params, body } => {
                format!(
                    "|{}| {}",
                    params.join(", "),
                    self.format_expr(body, indent)
                )
            }

            // NEW: Async expression
            ExprKind::Async { body } => {
                format!(
                    "async {}",
                    self.format_expr(body, indent)
                )
            }

            // NEW: Await expression
            ExprKind::Await { expr } => {
                format!(
                    "await {}",
                    self.format_expr(expr, indent)
                )
            }

            // NEW: Try expression
            ExprKind::Try { expr } => {
                format!(
                    "try {}",
                    self.format_expr(expr, indent)
                )
            }

            // NEW: Cast expression
            ExprKind::Cast { expr, ty } => {
                format!(
                    "{} as {}",
                    self.format_expr(expr, indent),
                    self.format_type(ty)
                )
            }

            // ... implement ALL remaining variants

            // NO FALLBACK - fail loudly if variant missing
            _ => panic!(
                "FATAL: Unimplemented ExprKind variant in formatter: {:?}\n\
                 This is a BUG. Please report at: https://github.com/paiml/ruchy/issues",
                kind
            ),
        }
    }
}
```

#### Extreme TDD for Each Variant

**Test Pattern** (repeat for ALL 58 variants):

```rust
#[test]
fn test_fmt_array_literal_empty() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("array_empty.ruchy");
    std::fs::write(&test_file, "let x = []").unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();
    assert!(formatted.contains("[]"));
}

#[test]
fn test_fmt_array_literal_single() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("array_single.ruchy");
    std::fs::write(&test_file, "let x = [1]").unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();
    assert!(formatted.contains("[1]"));
}

#[test]
fn test_fmt_array_literal_multiple() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("array_multiple.ruchy");
    std::fs::write(&test_file, "let x = [1, 2, 3, 4, 5]").unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();
    assert!(formatted.contains("[1, 2, 3, 4, 5]"));
}

#[test]
fn test_fmt_array_literal_multiline() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("array_multiline.ruchy");

    let original = "let x = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]";
    std::fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Should wrap to multiple lines if exceeds line width
    let line_count = formatted.lines().count();
    assert!(line_count > 1, "Long array should wrap to multiple lines");
}
```

