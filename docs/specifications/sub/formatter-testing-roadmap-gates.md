# Sub-spec: World-Class Formatter — Testing Strategy, Roadmap & Quality Gates

**Parent:** [world-class-formatter-spec.md](../world-class-formatter-spec.md)

---

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
