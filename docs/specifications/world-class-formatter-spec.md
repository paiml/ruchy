# World-Class Formatter Specification v1.0

**Status**: 🎯 **ACTIVE** - Specification for Perfect Formatter
**Created**: 2025-10-15
**Target**: v3.89.0 - v3.91.0 (3-release roadmap)
**Goal**: Make `ruchy fmt` PERFECT - industry-leading formatter

---

## Executive Summary

Create a **world-class formatter** for Ruchy that matches or exceeds the quality of industry leaders (rustfmt, Deno fmt, Ruff). The formatter must be **PERFECT** - preserving comments, doctests, annotations, and user intent while applying consistent formatting.

**Previous Status**: v3.88.0 - P0 corruption fixed, but formatter had P1 issues:
- ❌ Stripped ALL comments (documentation loss)
- ❌ Significant unwanted style changes
- ❌ Only 27/85 ExprKind variants implemented (~32%)
- ❌ Newline display issues

**Current Status**: v3.89.0 - Configuration + Ignore Directives COMPLETE (2025-10-15):
- ✅ Configuration system with TOML support (11 tests passing)
- ✅ Ignore directives fully functional (10/10 tests passing)
- ✅ Parser fixes: Line continuations + multiple comments (9 parser tests)
- ✅ 10 critical bugs fixed with Extreme TDD methodology
- ✅ Property tests: 6 tests with 10K+ random inputs
- 🎯 Released to crates.io: https://crates.io/crates/ruchy/3.89.0

**Target Status**: v3.91.0 - Perfect formatter:
- ✅ 100% comment preservation
- ✅ 100% ExprKind coverage (85/85 variants)
- ✅ Minimal, intentional style changes only
- ✅ Configurable formatting options
- ✅ Round-trip validation
- ✅ Industry-leading quality

---

## Industry Best Practices (Learned from Leaders)

### 1. rustfmt (Rust - Gold Standard)

**Comment Preservation Strategy**:
- Comments stored as tokens, not in AST
- Associated with nearest AST node via position tracking
- Preserved during formatting with intelligent placement
- Configuration: `normalize_comments`, `wrap_comments`

**Key Insights**:
- ✅ Comments are first-class citizens (not discarded)
- ✅ Position-based association (span tracking)
- ✅ Configurable comment behavior
- ⚠️ Known issue: Comments in unusual positions may be lost

**Implementation Pattern**:
```rust
// Comments stored separately from AST
struct CommentTracker {
    comments: Vec<Comment>,
    comment_map: HashMap<Span, Vec<usize>>, // AST span → comment indices
}

struct Comment {
    kind: CommentKind,      // Line, Block, Doc
    text: String,
    span: Span,
    association: NodeId,     // Associated AST node
}
```

### 2. Deno fmt (TypeScript - dprint-based)

**Comment Preservation Strategy**:
- Uses dprint as formatter engine
- Preserves all comments (line and block)
- Special directives: `// deno-fmt-ignore`, `// deno-fmt-ignore-file`
- Maintains comment context while reformatting code

**Key Insights**:
- ✅ Ignore directives for user control
- ✅ All comments preserved by default
- ✅ Context-aware comment placement
- ✅ Works with complex TypeScript AST

**Implementation Pattern**:
```typescript
// Ignore directives control formatting
// deno-fmt-ignore
export const identity = [
    1, 0, 0,  // User formatting preserved
    0, 1, 0,
    0, 0, 1,
];
```

### 3. Ruff (Python - Rust-based, Black-compatible)

**Comment Preservation Strategy**:
- Comments associated with entire expressions
- AST + Tokens + Formatter IR (Intermediate Representation)
- 99.9% Black compatibility (industry standard)
- Structured approach: FormatNodeRule for each AST node

**Key Insights**:
- ✅ Multi-layer representation (AST + Tokens + IR)
- ✅ Comments preserved in complex boolean expressions
- ✅ Systematic node formatting (FormatNodeRule pattern)
- ✅ >30x faster than Black, 100x faster than YAPF

**Implementation Pattern**:
```rust
// Implement FormatNodeRule for each AST node
impl FormatNodeRule<ast::ExprBoolOp> for FormatExprBoolOp {
    fn fmt_fields(&self, item: &ast::ExprBoolOp, f: &mut Formatter) -> FormatResult<()> {
        // Destructure to ensure all fields handled
        let ast::ExprBoolOp { range: _, op, values } = item;
        // Format with comment preservation
    }
}
```

---

## Defect Analysis (Learning from Our Mistakes)

### CRITICAL-FMT-CODE-DESTRUCTION (v3.86.0 - FIXED v3.87.0)

**Problem**: Operator mangling, let rewriting
**Root Cause**: Used Debug trait instead of Display, forced functional syntax
**Lesson**: Always use Display for user-facing output, respect original syntax

### CRITICAL-FMT-DEBUG-FALLBACK (v3.87.0 - FIXED v3.88.0)

**Problem**: AST Debug output for 70+ unhandled ExprKind variants
**Root Cause**: Catch-all pattern with Debug formatting
**Lesson**: No silent fallbacks, implement ALL variants or fail loudly

### DEFECT-FMT-002-COMMENT-STRIPPING (v3.88.0 - ACTIVE)

**Problem**: All comments stripped, documentation lost
**Root Cause**: Parser discards comments, AST doesn't store them
**Lesson**: Comments are data, not whitespace - must be preserved

### Additional Issues Discovered (External Verification)

**Problem**: Significant unwanted style changes
- Wraps files in `{ }` blocks
- Adds `: Any` type annotations everywhere
- Changes `let` syntax unnecessarily
- Newline display issues in strings

**Root Cause**: Formatter reconstructs code from AST without preserving original style choices
**Lesson**: Preserve user intent, only format what's necessary

---

## Perfect Formatter Requirements

### Must Have (P0 - Blocking)

1. **100% Comment Preservation**
   - Line comments: `//`
   - Block comments: `/* */`
   - Doc comments: `///`, `/**`
   - Maintain position and context
   - Never lose a single comment

2. **100% ExprKind Coverage**
   - All 85+ ExprKind variants formatted
   - No fallback cases
   - Handles all valid Ruchy syntax

3. **Semantic Preservation**
   - Format → Parse → Format = idempotent
   - Original semantics unchanged
   - Code still compiles and runs

4. **Style Preservation**
   - Minimal changes only
   - No unnecessary block wrapping
   - Preserve user's structural choices
   - Only format whitespace/indentation

### Should Have (P1 - Important)

5. **Configurable Formatting**
   - Line width (default: 80)
   - Indent size (default: 4 spaces)
   - Tab vs spaces
   - Type annotation control

6. **Ignore Directives**
   - `// ruchy-fmt-ignore` - ignore next statement
   - `// ruchy-fmt-ignore-file` - ignore entire file
   - `// ruchy-fmt-ignore-block` - ignore next block

7. **Round-Trip Validation**
   - Verify formatted code is valid
   - Verify semantics unchanged
   - Verify idempotency

8. **Comprehensive Testing**
   - Unit tests for each ExprKind variant
   - CLI contract tests
   - Property tests (random code generation)
   - Real-world code validation

### Nice to Have (P2 - Future)

9. **Advanced Features**
   - Import sorting
   - Trailing comma insertion
   - Alignment options
   - Custom style profiles

10. **IDE Integration**
    - Format on save
    - Format selection
    - Real-time validation

---

## Implementation Architecture

### Phase 1: Comment Preservation (v3.89.0)

**Goal**: Store and preserve ALL comments

#### Step 1.1: Extend Lexer to Track Comments
```rust
// File: src/frontend/lexer.rs

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ... existing tokens
    Comment(CommentKind),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommentKind {
    Line(String),           // // comment
    Block(String),          // /* comment */
    Doc(String),            // /// doc comment
    BlockDoc(String),       // /** doc comment */
}

pub struct Lexer {
    // ... existing fields
    comments: Vec<Comment>, // Track ALL comments
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub kind: CommentKind,
    pub text: String,
    pub span: Span,
    pub position: CommentPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommentPosition {
    Leading,      // Before associated code
    Trailing,     // End of line
    Standalone,   // Own line
}
```

#### Step 1.2: Store Comments in AST
```rust
// File: src/frontend/ast.rs

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
    pub attributes: Vec<Attribute>,
    pub leading_comments: Vec<Comment>,   // NEW: Comments before expr
    pub trailing_comment: Option<Comment>, // NEW: End-of-line comment
}

impl Expr {
    pub fn new_with_comments(
        kind: ExprKind,
        span: Span,
        leading: Vec<Comment>,
        trailing: Option<Comment>,
    ) -> Self {
        Self {
            kind,
            span,
            attributes: vec![],
            leading_comments: leading,
            trailing_comment: trailing,
        }
    }
}
```

#### Step 1.3: Parser Associates Comments with AST Nodes
```rust
// File: src/frontend/parser/core.rs

impl Parser {
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        // Collect leading comments
        let leading_comments = self.consume_leading_comments();

        // Parse expression
        let expr = self.parse_expr_inner()?;

        // Check for trailing comment
        let trailing_comment = self.consume_trailing_comment();

        Ok(Expr::new_with_comments(
            expr.kind,
            expr.span,
            leading_comments,
            trailing_comment,
        ))
    }

    fn consume_leading_comments(&mut self) -> Vec<Comment> {
        let mut comments = Vec::new();
        while matches!(self.peek(), Some(Token { kind: TokenKind::Comment(_), .. })) {
            if let Some(token) = self.advance() {
                comments.push(Comment::from_token(token, CommentPosition::Leading));
            }
        }
        comments
    }

    fn consume_trailing_comment(&mut self) -> Option<Comment> {
        if matches!(self.peek(), Some(Token { kind: TokenKind::Comment(_), .. })) {
            self.advance()
                .map(|token| Comment::from_token(token, CommentPosition::Trailing))
        } else {
            None
        }
    }
}
```

#### Step 1.4: Formatter Emits Comments
```rust
// File: src/quality/formatter.rs

impl Formatter {
    fn format_expr(&self, expr: &Expr, indent: usize) -> String {
        let mut result = String::new();

        // Emit leading comments
        for comment in &expr.leading_comments {
            result.push_str(&self.format_comment(comment, indent));
            result.push('\n');
            result.push_str(&self.indent_string(indent));
        }

        // Emit expression
        result.push_str(&self.format_expr_kind(&expr.kind, indent));

        // Emit trailing comment
        if let Some(comment) = &expr.trailing_comment {
            result.push_str("  ");
            result.push_str(&comment.text);
        }

        result
    }

    fn format_comment(&self, comment: &Comment, indent: usize) -> String {
        match &comment.kind {
            CommentKind::Line(text) => format!("// {}", text),
            CommentKind::Block(text) => format!("/* {} */", text),
            CommentKind::Doc(text) => format!("/// {}", text),
            CommentKind::BlockDoc(text) => format!("/** {} */", text),
        }
    }
}
```

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

---

### Phase 3: Style Preservation & Configuration (v3.91.0)

**Goal**: Minimal style changes, configurable options

#### Step 3.1: Configuration File Support

```toml
# .ruchy-fmt.toml or ruchy.toml [fmt] section

[fmt]
# Line width (default: 80)
line_width = 100

# Indent size (default: 4)
indent_size = 2

# Use tabs instead of spaces (default: false)
use_tabs = false

# Preserve let syntax (default: true)
preserve_let_syntax = true

# Add type annotations (default: false)
add_type_annotations = false

# Wrap top-level in block (default: false)
wrap_top_level = false

# Normalize newlines (default: true)
normalize_newlines = true

# Trailing commas (default: "preserve")
trailing_commas = "always"  # "always", "never", "preserve"
```

#### Step 3.2: Implement Configuration

```rust
// File: src/quality/formatter_config.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatterConfig {
    pub line_width: usize,
    pub indent_size: usize,
    pub use_tabs: bool,
    pub preserve_let_syntax: bool,
    pub add_type_annotations: bool,
    pub wrap_top_level: bool,
    pub normalize_newlines: bool,
    pub trailing_commas: TrailingCommas,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrailingCommas {
    Always,
    Never,
    Preserve,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            line_width: 80,
            indent_size: 4,
            use_tabs: false,
            preserve_let_syntax: true,
            add_type_annotations: false,
            wrap_top_level: false,
            normalize_newlines: true,
            trailing_commas: TrailingCommas::Preserve,
        }
    }
}

impl FormatterConfig {
    pub fn load() -> Result<Self, ConfigError> {
        // Try .ruchy-fmt.toml first
        if let Ok(config) = Self::load_from(".ruchy-fmt.toml") {
            return Ok(config);
        }

        // Try ruchy.toml [fmt] section
        if let Ok(config) = Self::load_from_section("ruchy.toml", "fmt") {
            return Ok(config);
        }

        // Use defaults
        Ok(Self::default())
    }
}
```

#### Step 3.3: Fix Unwanted Style Changes

```rust
impl Formatter {
    fn format_let(&self, name: &str, value: &Expr, body: &Expr, indent: usize) -> String {
        // Check if body is Unit (meaning statement-style)
        if matches!(body.kind, ExprKind::Literal(Literal::Unit)) {
            // Statement style (PRESERVE USER'S CHOICE)
            format!(
                "let {} = {}",
                name,
                self.format_expr(value, indent)
            )
        } else {
            // Functional style (only when user wrote it that way)
            format!(
                "let {} = {} in {}",
                name,
                self.format_expr(value, indent),
                self.format_expr(body, indent)
            )
        }
    }

    fn format_top_level(&self, exprs: &[Expr]) -> String {
        // NEVER wrap in { } unless user wrote it that way
        if self.config.wrap_top_level {
            format!(
                "{{\n{}\n}}",
                exprs.iter()
                    .map(|e| format!("    {}", self.format_expr(e, 1)))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            // Just format each expression
            exprs.iter()
                .map(|e| self.format_expr(e, 0))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    fn should_add_type_annotation(&self, name: &str) -> bool {
        // Only add type annotations if configured
        self.config.add_type_annotations
    }
}
```

#### Step 3.4: Ignore Directives

```rust
impl Formatter {
    fn should_ignore(&self, expr: &Expr) -> bool {
        // Check for // ruchy-fmt-ignore in leading comments
        expr.leading_comments.iter().any(|c| {
            c.text.trim() == "ruchy-fmt-ignore"
        })
    }

    fn format_expr(&self, expr: &Expr, indent: usize) -> String {
        if self.should_ignore(expr) {
            // Return original source text (preserve exactly)
            return self.get_original_text(expr.span);
        }

        // Normal formatting
        self.format_expr_with_comments(expr, indent)
    }
}
```

---

## Testing Strategy (Extreme TDD + CLI Extreme Testing)

### Test Pyramid

```
                /\
               /  \
              /E2E \              10 tests - Real-world programs
             /------\
            /Property\             50 tests - Random generation
           /----------\
          / CLI Tests  \          200+ tests - All variants + comments
         /--------------\
        /  Unit Tests    \        500+ tests - Each function
       /------------------\
```

### Test Organization

```
tests/
├── cli_contract_fmt_comments.rs       # Comment preservation (50 tests)
├── cli_contract_fmt_variants.rs       # All ExprKind variants (85 tests)
├── cli_contract_fmt_style.rs          # Style preservation (30 tests)
├── cli_contract_fmt_config.rs         # Configuration (25 tests)
├── cli_contract_fmt_ignore.rs         # Ignore directives (10 tests)
├── cli_contract_fmt_round_trip.rs     # Round-trip validation (20 tests)
├── cli_contract_fmt_real_world.rs     # Real Ruchy programs (10 tests)
├── property_fmt_random.rs             # Property-based (10 tests)
└── e2e_fmt_integration.rs             # End-to-end (5 tests)
```

### Property Tests (Random Code Generation)

```rust
// File: tests/property_fmt_random.rs

use proptest::prelude::*;

proptest! {
    #[test]
    fn test_fmt_never_corrupts_code(code in any_valid_ruchy_program()) {
        // Format the code
        let formatted = format_code(&code);

        // Verify it's still valid
        assert!(parse_code(&formatted).is_ok(), "Formatted code must parse");
    }

    #[test]
    fn test_fmt_is_idempotent(code in any_valid_ruchy_program()) {
        // Format once
        let formatted1 = format_code(&code);

        // Format again
        let formatted2 = format_code(&formatted1);

        // Must be identical
        assert_eq!(formatted1, formatted2, "Formatting must be idempotent");
    }

    #[test]
    fn test_fmt_preserves_all_comments(code in any_ruchy_program_with_comments()) {
        // Extract original comments
        let original_comments = extract_comments(&code);

        // Format the code
        let formatted = format_code(&code);

        // Extract formatted comments
        let formatted_comments = extract_comments(&formatted);

        // All comments must be preserved
        assert_eq!(
            original_comments.len(),
            formatted_comments.len(),
            "Comment count must be preserved"
        );

        for (orig, fmt) in original_comments.iter().zip(formatted_comments.iter()) {
            assert_eq!(orig.text, fmt.text, "Comment text must be preserved");
        }
    }
}

fn any_valid_ruchy_program() -> impl Strategy<Value = String> {
    // Generate random valid Ruchy programs
    prop::collection::vec(any_ruchy_statement(), 1..20)
        .prop_map(|stmts| stmts.join("\n"))
}

fn any_ruchy_program_with_comments() -> impl Strategy<Value = String> {
    // Generate programs with random comments
    any_valid_ruchy_program()
        .prop_flat_map(|code| {
            prop::collection::vec(any_comment(), 0..10)
                .prop_map(move |comments| inject_comments(&code, &comments))
        })
}
```

### Real-World Test Suite

```rust
// File: tests/cli_contract_fmt_real_world.rs

#[test]
fn test_fmt_ruchy_head_example() {
    // Use actual head.ruchy from examples
    let output = ruchy_cmd()
        .arg("fmt")
        .arg("examples/ruchy-head/head.ruchy")
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Must preserve all comments
    assert!(formatted.contains("ruchy-head:"));
    assert!(formatted.contains("Returns the first n lines"));
    assert!(formatted.contains("Algorithm: O(n)"));

    // Must be valid and functional
    let check = ruchy_cmd()
        .arg("check")
        .arg("examples/ruchy-head/head.ruchy")
        .assert()
        .success();
}

#[test]
fn test_fmt_all_examples_directory() {
    // Format ALL files in examples/ directory
    let examples_dir = Path::new("examples");

    for entry in WalkDir::new(examples_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension() == Some(OsStr::new("ruchy")))
    {
        let file_path = entry.path();

        // Format the file
        let output = ruchy_cmd()
            .arg("fmt")
            .arg(file_path)
            .arg("--stdout")
            .output()
            .expect("Failed to run fmt");

        assert!(
            output.status.success(),
            "fmt failed on example: {}",
            file_path.display()
        );

        let formatted = String::from_utf8(output.stdout).unwrap();

        // Verify it's valid Ruchy
        let temp_dir = TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("formatted.ruchy");
        std::fs::write(&temp_file, formatted).unwrap();

        ruchy_cmd()
            .arg("check")
            .arg(&temp_file)
            .assert()
            .success()
            .stderr(predicate::str::contains("Error").not());
    }
}
```

---

## Implementation Roadmap

### v3.89.0 - Comment Preservation (Sprint 1: 2-3 days)

**Tickets**:
- [FMT-PERFECT-001] Extend lexer to track comments as tokens
- [FMT-PERFECT-002] Store comments in AST (leading + trailing)
- [FMT-PERFECT-003] Parser associates comments with AST nodes
- [FMT-PERFECT-004] Formatter emits comments with expressions
- [FMT-PERFECT-005] Add 50 CLI tests for comment preservation
- [FMT-PERFECT-006] Verify all examples preserve comments
- [FMT-PERFECT-007] External validation (ruchy-cli-tools-book)

**Success Criteria**:
- ✅ 0 comments lost (100% preservation)
- ✅ 50/50 comment tests passing
- ✅ External verification: "Comments preserved perfectly"
- ✅ head.ruchy retains all documentation

### v3.90.0 - Complete ExprKind Coverage (Sprint 2: 3-5 days)

**Tickets**:
- [FMT-PERFECT-008] Audit all 85+ ExprKind variants
- [FMT-PERFECT-009] Implement Array, Object, TupleStruct (3 variants)
- [FMT-PERFECT-010] Implement Lambda, Closure, Async, Await (4 variants)
- [FMT-PERFECT-011] Implement Try, Throw, Catch (3 variants)
- [FMT-PERFECT-012] Implement Import, Export, Module (3 variants)
- [FMT-PERFECT-013] Implement Class, Trait, Impl (3 variants)
- [FMT-PERFECT-014] Implement Enum, Struct, Type (3 variants)
- [FMT-PERFECT-015] Implement Cast, As, Is (3 variants)
- [FMT-PERFECT-016] Implement Ref, Deref, Borrow (3 variants)
- [FMT-PERFECT-017] Implement Macro, Attribute (2 variants)
- [FMT-PERFECT-018] Implement remaining ~40 variants (grouped)
- [FMT-PERFECT-019] Remove fallback panic (all variants covered)
- [FMT-PERFECT-020] Add 85 CLI tests (one per variant)

**Success Criteria**:
- ✅ 85/85 ExprKind variants implemented (100%)
- ✅ No fallback case remains
- ✅ 85/85 variant tests passing
- ✅ Handles all possible Ruchy syntax

### v3.91.0 - Style Preservation & Configuration (Sprint 3: 2-3 days)

**Tickets**:
- [FMT-PERFECT-021] Create FormatterConfig with defaults
- [FMT-PERFECT-022] Load config from .ruchy-fmt.toml
- [FMT-PERFECT-023] Fix: Don't wrap top-level in blocks (unless user wants)
- [FMT-PERFECT-024] Fix: Preserve let syntax (statement vs functional)
- [FMT-PERFECT-025] Fix: Make type annotations optional
- [FMT-PERFECT-026] Fix: Newline display in strings
- [FMT-PERFECT-027] Implement ignore directives (ruchy-fmt-ignore)
- [FMT-PERFECT-028] Add 25 configuration tests
- [FMT-PERFECT-029] Add 10 ignore directive tests
- [FMT-PERFECT-030] Add 20 round-trip validation tests
- [FMT-PERFECT-031] Add 10 property tests (random generation)
- [FMT-PERFECT-032] Final external validation

**Success Criteria**:
- ✅ Minimal style changes only
- ✅ User has full control via config
- ✅ Ignore directives work perfectly
- ✅ Round-trip: format(format(x)) == format(x)
- ✅ External verification: "Formatter is PERFECT"

---

## Quality Gates (Must Pass Before Release)

### Gate 1: Test Coverage
- ✅ 200+ CLI tests passing (100%)
- ✅ 500+ unit tests passing (100%)
- ✅ 50+ property tests passing (100%)
- ✅ 10+ real-world programs formatted successfully

### Gate 2: Comment Preservation
- ✅ 0 comments lost across all test cases
- ✅ head.ruchy preserves all documentation
- ✅ External verification confirms preservation

### Gate 3: ExprKind Coverage
- ✅ 85/85 variants implemented (100%)
- ✅ No fallback case
- ✅ All Ruchy syntax handled

### Gate 4: Style Preservation
- ✅ No unwanted block wrapping
- ✅ Let syntax preserved
- ✅ Type annotations optional
- ✅ Minimal changes only

### Gate 5: Idempotency
- ✅ format(format(x)) == format(x) for all test cases
- ✅ Formatted code is valid Ruchy
- ✅ Formatted code runs correctly

### Gate 6: Configuration
- ✅ Config file loading works
- ✅ All options respected
- ✅ Ignore directives functional

### Gate 7: Performance
- ✅ Formats 1000-line file in < 1 second
- ✅ Faster than or equal to rustfmt
- ✅ Memory usage reasonable

### Gate 8: External Validation
- ✅ ruchy-cli-tools-book verification: "PERFECT"
- ✅ All known issues resolved
- ✅ Zero bug reports on GitHub

---

## Success Metrics

### Before (v3.88.0)
- ❌ Comments: 0% preserved (all stripped)
- ⚠️ ExprKind coverage: 32% (27/85)
- ❌ Style changes: Significant unwanted changes
- ⚠️ User satisfaction: "Use with caution"
- ❌ Rating: 4/10 (functional but limited)

### After (v3.91.0 Target)
- ✅ Comments: 100% preserved
- ✅ ExprKind coverage: 100% (85/85)
- ✅ Style changes: Minimal, intentional only
- ✅ User satisfaction: "Production-ready, perfect"
- ✅ Rating: 10/10 (world-class)

---

## Toyota Way Principles Applied

### Jidoka (Built-in Quality)
- Comments preserved by design (not as afterthought)
- All variants implemented (no fallbacks)
- Tests written FIRST (TDD)
- Quality gates block bad releases

### Genchi Genbutsu (Go and See)
- Learned from external bug reports
- Studied industry leaders (rustfmt, Deno, Ruff)
- Tested with real-world code
- External validation required

### Poka-Yoke (Error Proofing)
- No fallback cases (panic if variant missing)
- Round-trip validation catches corruption
- Property tests catch edge cases
- Ignore directives give users control

### Kaizen (Continuous Improvement)
- Systematic 3-release roadmap
- Each release builds on previous
- Defect reports drive improvements
- Never satisfied with "good enough"

### Respect for People
- Preserve user's documentation (comments)
- Preserve user's style choices
- Give users control (configuration)
- Never lose user's work

---

## Appendix: Related Documents

- **CRITICAL-FMT-CODE-DESTRUCTION.md** - Operator mangling (FIXED v3.87.0)
- **CRITICAL-FMT-DEBUG-FALLBACK.md** - AST corruption (FIXED v3.88.0)
- **DEFECT-FMT-002-COMMENT-STRIPPING.md** - Comment loss (ACTIVE v3.88.0)
- **BUG_VERIFICATION_v3.88.0.md** - External validation report
- **15-tool-improvement-spec.md** - Tool quality standards
- **TICR-ANALYSIS.md** - Test complexity analysis

---

**Created**: 2025-10-15
**Target Completion**: v3.91.0 (3 sprints, ~8-11 days total)
**Status**: 🎯 **READY TO IMPLEMENT**
**Goal**: Make `ruchy fmt` **PERFECT** - worthy of industry leaders
