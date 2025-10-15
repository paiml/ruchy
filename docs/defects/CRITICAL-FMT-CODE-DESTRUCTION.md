# CRITICAL DEFECT: fmt Tool Destroys Code

**Status**: 🚨 CRITICAL - Code destroying bug
**Discovered**: 2025-10-15
**Severity**: P0 - Blocks all users, destroys code
**TICR Risk**: HIGH (fmt not in specification, untested)

## Problem Statement

The `ruchy fmt` command **destroys user code** by rewriting syntax instead of formatting it.

## Reproduction

```bash
# Create test file
cat > test.ruchy << 'EOF'
let x = 42
let y = x * 2
println(y)
EOF

# Run formatter
ruchy fmt test.ruchy

# Result: CODE DESTROYED
cat test.ruchy
{
    let x = 42 in ()
    let y = x Multiply 2 in ()
    println(y)
}
```

## Root Cause

File: `src/quality/formatter.rs`

### Problem 1: Debug Formatting for Operators (Line 77-79)
```rust
ExprKind::Binary { left, op, right } => {
    format!(
        "{} {:?} {}",  // ❌ Using Debug trait!
        self.format_expr(left, indent),
        op,  // This becomes "Multiply" instead of "*"
        self.format_expr(right, indent)
    )
}
```

**Issue**: Uses `{:?}` Debug trait which prints enum variant names (`Multiply`, `Add`, etc.) instead of actual operators (`*`, `+`).

### Problem 2: Functional Let Syntax (Line 68-73)
```rust
ExprKind::Let { name, value, body, .. } => {
    format!(
        "let {} = {} in {}",  // ❌ Functional style!
        name,
        self.format_expr(value, indent),
        self.format_expr(body, indent)
    )
}
```

**Issue**: Converts statement-style `let x = 42` to functional-style `let x = 42 in ()`, which breaks code.

### Problem 3: Block Wrapping
Parser treats multiple top-level statements as a Block, causing formatter to wrap entire file in `{ }`.

## Impact

- **ALL code formatted by `ruchy fmt` is BROKEN**
- Users lose working code
- CI/CD pipelines that use fmt will break builds
- Pre-commit hooks with fmt will block all commits

## Toyota Way Violation

- **Jidoka**: No quality built in - formatter wasn't tested
- **Poka-Yoke**: No error prevention - should validate output compiles
- **Genchi Genbutsu**: Formatter wasn't tested on real code

## Fix Required

### Immediate Actions (Stop the Line)

1. ❌ **DISABLE fmt tool** until fixed (document as broken)
2. ✅ Create CLI contract tests to prevent regression
3. ✅ Fix formatter to preserve original syntax
4. ✅ Add round-trip validation (parse → format → parse should be identical)

### Required Changes

#### Fix 1: Operator Display (MANDATORY)
```rust
ExprKind::Binary { left, op, right } => {
    format!(
        "{} {} {}",  // ✅ Use Display trait
        self.format_expr(left, indent),
        op,  // Must impl Display to show "*" not "Multiply"
        self.format_expr(right, indent)
    )
}
```

#### Fix 2: Statement-Style Let (MANDATORY)
```rust
ExprKind::Let { name, value, body, .. } => {
    // Check if body is Unit/empty - if so, use statement style
    if matches!(body.kind, ExprKind::Literal(Literal::Unit)) {
        format!("let {} = {}", name, self.format_expr(value, indent))
    } else {
        // Only use functional style when there's actually a body
        format!(
            "let {} = {} in {}",
            name,
            self.format_expr(value, indent),
            self.format_expr(body, indent)
        )
    }
}
```

#### Fix 3: Block Handling (MANDATORY)
Only wrap in braces if it's an actual block expression, not top-level statements.

### Test Requirements

```rust
#[test]
fn test_fmt_roundtrip() {
    let code = "let x = 42\nlet y = x * 2\nprintln(y)";
    let ast = parse(code).unwrap();
    let formatted = format(&ast).unwrap();
    let ast2 = parse(&formatted).unwrap();
    assert_eq!(ast, ast2, "Formatting must preserve AST semantics");
}

#[test]
fn test_fmt_preserves_operators() {
    let code = "x * y";
    let formatted = format_code(code);
    assert!(formatted.contains(" * "), "Must preserve * operator");
    assert!(!formatted.contains("Multiply"), "Must not use Debug trait");
}
```

## Specification Update Required

Add fmt as **16th tool** to:
- ✅ `docs/specifications/15-tool-improvement-spec.md` → rename to 16-tool
- ✅ `docs/testing/TICR-ANALYSIS.md` → add fmt TICR entry
- ✅ All roadmap documents mentioning "15 tools"

## Prevention

1. **CLI Contract Tests**: Create comprehensive fmt CLI tests
2. **Round-Trip Validation**: Format → Parse → Format must be idempotent
3. **Property Tests**: Random code → format → compile must succeed
4. **Mutation Tests**: Ensure tests catch operator changes

## References

- Toyota Way: Stop the line for defects
- CLAUDE.md: Zero tolerance for code-destroying bugs
- TICR Analysis: fmt tool missing from risk assessment

---

**Priority**: P0 CRITICAL
**Action**: STOP THE LINE - Fix before any other work
