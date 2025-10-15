# CRITICAL DEFECT: fmt Debug Fallback Corrupts Files

**Date**: 2025-10-15
**Severity**: 🚨 **P0 CRITICAL** - Silent data corruption
**Status**: 🛑 **ACTIVE** - Discovered post-v3.87.0 release
**Impact**: **DESTRUCTIVE** - Corrupts user files with AST Debug output

---

## Executive Summary

The `ruchy fmt` tool has a critical fallback that outputs AST Debug representation for unhandled expression types, completely corrupting formatted files. This is WORSE than the operator mangling bug (CRITICAL-FMT-CODE-DESTRUCTION) because it affects a wider range of code patterns.

**This bug was NOT caught by our CLI contract tests** because we only tested simple expressions.

---

## Discovery

**Source**: External bug report from ruchy-cli-tools-book project
**File**: BUG_VERIFICATION_v3.87.0.md
**Verification**: Reproduced with `examples/ruchy-head/head.ruchy`

### Bug Report Summary

```bash
$ ruchy fmt head.ruchy
✓ Formatted head.ruchy  # Claims success!

$ head -20 head.ruchy
{
    fun head_lines(file_path: Any, n: Any) {
        let content = fs_read(file_path) in {
            let result = ""
            let line_count = 0
            for i in range(0, content.len()) {
                let ch = IndexAccess { object: Expr { kind: Identifier("content"),
                    span: Span { start: 472, end: 479 }, attributes: [] },
                    index: Expr { kind: Identifier("i"), span: Span { start: 480,
                    end: 481 }, attributes: [] } } in {
```

**Problem**: File corrupted with AST Debug output instead of formatted Ruchy code!

---

## Root Cause Analysis (Five Whys)

### Why did fmt corrupt the file?
**Answer**: Formatter output AST Debug representation for `IndexAccess`, `Assign`, and `Return` expressions.

### Why did it output Debug representation?
**Answer**: Formatter has a catch-all pattern that falls back to `{:?}` format for unhandled ExprKind variants.

### Why does it have a catch-all Debug fallback?
**Answer**: Formatter is incomplete - only implements ~10 ExprKind variants, has `_` pattern for rest.

### Why is the formatter incomplete?
**Answer**: Incremental development - features added as needed, not all ExprKind variants implemented.

### Why wasn't this caught in testing?
**Answer**: CLI contract tests only used simple expressions (literals, binary ops, let, if). Real-world code uses array indexing, assignments, returns, etc.

---

## Source Code

**File**: `src/quality/formatter.rs`
**Lines**: 207-210

```rust
fn format_expr(&self, expr: &Expr, indent: usize) -> String {
    match &expr.kind {
        ExprKind::Literal(lit) => { /* ... */ },
        ExprKind::Identifier(name) => { /* ... */ },
        ExprKind::Let { /* ... */ } => { /* ... */ },
        ExprKind::Binary { /* ... */ } => { /* ... */ },
        ExprKind::Block(exprs) => { /* ... */ },
        ExprKind::Function { /* ... */ } => { /* ... */ },
        ExprKind::If { /* ... */ } => { /* ... */ },
        ExprKind::Call { /* ... */ } => { /* ... */ },
        ExprKind::MethodCall { /* ... */ } => { /* ... */ },
        ExprKind::For { /* ... */ } => { /* ... */ },
        _ => {
            format!("{:?}", expr.kind) // ⚠️ CRITICAL BUG: Debug fallback
        }
    }
}
```

---

## Missing ExprKind Variants

The formatter is missing support for these critical variants:

1. **IndexAccess** - Array/object indexing (`arr[i]`)
2. **Assign** - Assignment statements (`x = 42`)
3. **Return** - Return statements (`return value`)
4. **FieldAccess** - Object field access (`obj.field`)
5. **Match** - Pattern matching (`match x { ... }`)
6. **While** - While loops (`while condition { ... }`)
7. **Break** - Loop break (`break`)
8. **Continue** - Loop continue (`continue`)
9. **Array** - Array literals (`[1, 2, 3]`)
10. **Object** - Object literals (`{ x: 1, y: 2 }`)
11. **Tuple** - Tuple literals (`(1, 2, 3)`)
12. **Range** - Range expressions (`0..10`)
13. **Try** - Error handling (`try expr`)
14. **Await** - Async await (`await expr`)
15. **Closure** - Lambda expressions (`|x| x + 1`)
16. **Unary** - Unary operations (`-x`, `!x`)
17. **Cast** - Type casting (`x as i32`)
18. **Ref** - Reference (`&x`)
19. **Deref** - Dereference (`*x`)

**And potentially more...**

---

## Impact Assessment

### Severity: CRITICAL (P0)

**Why P0?**
1. **Silent data corruption**: Destructively modifies files without warning
2. **False success**: Claims "✓ Formatted" while corrupting
3. **Wide scope**: Affects any code using array indexing, assignments, or returns (VERY common!)
4. **Already released**: v3.87.0 shipped with this bug
5. **Trust violation**: Users expect fmt to be safe, not destructive

### Affected Code Patterns

**ANY Ruchy code using:**
- Array indexing: `arr[i]`
- Assignment: `x = value`
- Return statements: `return x`
- Object field access: `obj.field`
- Pattern matching: `match expr { ... }`
- While loops: `while cond { ... }`
- etc.

**Basically: ALL real-world Ruchy code** (not just toy examples).

### User Impact

**Catastrophic** for any user running `ruchy fmt` on production code:
- **Data loss**: Original code replaced with corrupted AST
- **Build breakage**: Corrupted files won't compile/run properly
- **Time loss**: Must restore from backup or rewrite
- **Trust loss**: Tool advertised as safe formatter destroys code

---

## Why This Wasn't Caught

### CLI Contract Tests Insufficiency

Our 23 fmt CLI tests (created in v3.87.0) only tested:
- ✅ Simple literals
- ✅ Binary operators
- ✅ Let expressions
- ✅ If statements
- ✅ Functions
- ❌ **Array indexing** (missing!)
- ❌ **Assignments** (missing!)
- ❌ **Return statements** (missing!)
- ❌ **Real-world code patterns** (missing!)

**Lesson**: Test coverage metrics (23 tests) != real-world coverage

---

## Fix Requirements

### Immediate (P0)

1. **Remove Debug fallback** - Make formatter fail loudly instead of corrupting
   ```rust
   _ => {
       panic!("Unsupported expression type: {:?}", expr.kind)
       // OR: return Err("Unsupported expression")
       // NEVER: Silent corruption with Debug output
   }
   ```

2. **Add CLI test for array indexing**
3. **Add CLI test for assignment**
4. **Add CLI test for return**

### Short-term (P1)

Implement all missing ExprKind variants:
- IndexAccess
- Assign
- Return
- FieldAccess
- Match
- While
- Break
- Continue
- etc.

### Long-term (P2)

- **Comprehensive test suite** with real-world Ruchy programs
- **Property test**: `format(parse(code))` should never contain `{:?}` output
- **Round-trip validation**: `format(format(x)) == format(x)` (already have test, but it passed!)

---

## Recommendations

### DO NOT USE `ruchy fmt` until fixed

**Critical Warning**: Using `ruchy fmt` on any non-trivial code will corrupt files!

**Safe code patterns** (won't trigger bug - very limited!):
- Pure expressions (no assignments, no array access)
- Function definitions without assignments/returns
- Simple if/else with expressions only

**Unsafe code patterns** (will corrupt):
- ANY code with assignments (`x = y`)
- ANY code with array indexing (`arr[i]`)
- ANY code with return statements
- **Basically all real code**

### For v3.88.0 Release

1. ✅ Remove Debug fallback (fail loudly)
2. ✅ Add array indexing support
3. ✅ Add assignment support
4. ✅ Add return support
5. ✅ Add comprehensive CLI tests with real-world patterns
6. ✅ Add property test for no-Debug-output invariant

---

## Prevention

### How to prevent similar bugs

1. **Test with real code**, not toy examples
2. **Property tests**: Invariants like "no `{:?}` in output"
3. **Fail loudly**: Never silently corrupt (panic/error > silent corruption)
4. **Code review**: All catch-all patterns should be reviewed
5. **Integration tests**: Real-world programs from examples/

---

## Toyota Way Principles Violated

1. **Jidoka** - Tool should stop/fail when encountering unhandled case, not corrupt
2. **Poka-Yoke** - No error-proofing: catch-all allows silent corruption
3. **Genchi Genbutsu** - We didn't "go and see" real-world usage patterns

### Toyota Way Response

1. **Stop the Line** - DO NOT USE fmt until fixed
2. **Root Cause** - Incomplete implementation + silent fallback
3. **Poka-Yoke** - Replace silent corruption with loud failure
4. **Kaizen** - Improve test suite to catch real-world patterns

---

## Status

**Current**: 🛑 **CRITICAL BUG ACTIVE** in v3.87.0
**Fix Status**: Not started (just discovered)
**Target**: v3.88.0
**Workaround**: Do not use `ruchy fmt` on any code with assignments/arrays/returns

---

## Related

- **CRITICAL-FMT-CODE-DESTRUCTION.md** - Operator mangling bug (FIXED in v3.87.0)
- **Bug #31** - External bug report (ruchy-cli-tools-book project)
- **tests/cli_contract_fmt.rs** - Insufficient test coverage (only simple cases)

---

**Date**: 2025-10-15
**Discovered By**: ruchy-cli-tools-book project verification
**Severity**: 🚨 P0 CRITICAL
**Action**: STOP USING fmt until v3.88.0 fix
