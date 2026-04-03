# Sub-spec: World-Class Formatter — Best Practices, Defect Analysis & Phase 1 Architecture

**Parent:** [world-class-formatter-spec.md](../world-class-formatter-spec.md)

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
