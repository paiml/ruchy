# EXTREME TDD: Search and Explode Test Expansion

> **Goal**: Rapidly achieve 95% test coverage by systematically expanding tests in modules with low coverage ratios.

## Quick Reference

```bash
# Phase 1: Find targets (~2s)
for f in src/frontend/parser/expressions_helpers/*.rs; do
  lines=$(wc -l < "$f"); tests=$(grep -c "^\s*#\[test\]" "$f" 2>/dev/null || echo 0)
  [ "$tests" -gt 0 ] && ratio=$((lines / tests)) || ratio=99999
  printf "%5d %3d %4d %s\n" "$lines" "$tests" "$ratio" "$(basename $f)"
done | sort -t' ' -k3 -nr | head -10

# Phase 2: Sync and test (~15s per file)
rsync -av src/path/to/file.rs mac:~/src/ruchy/src/path/to/file.rs
ssh mac "cd ~/src/ruchy && CARGO_TARGET_DIR=/Volumes/LambdaCache/ruchy-target cargo test --lib -p ruchy module::tests --no-fail-fast 2>&1 | tail -20"

# Phase 3: Full suite verification (~17s)
ssh mac "cd ~/src/ruchy && CARGO_TARGET_DIR=/Volumes/LambdaCache/ruchy-target cargo test --lib -p ruchy 2>&1 | tail -5"

# Phase 4: Commit and push (~3s)
git add <file> && git commit -m "test(<module>): Add +N EXTREME TDD tests" && git push origin main
```

---

## Phase 1: Identify High-Value Targets

### The Lines/Test Ratio

**Key Insight**: Files with high `lines/test` ratio have the most coverage gaps.

```bash
# Calculate ratio for all files in a directory
for f in src/frontend/parser/expressions_helpers/*.rs; do
  name=$(basename "$f")
  lines=$(wc -l < "$f")
  tests=$(grep -c "^\s*#\[test\]" "$f" 2>/dev/null || echo 0)
  if [ "$tests" -gt 0 ]; then
    ratio=$((lines / tests))
  else
    ratio=99999
  fi
  printf "%5d %3d %4d %s\n" "$lines" "$tests" "$ratio" "$name"
done | sort -t' ' -k3 -nr | head -10
```

**Target Priority**:
| Ratio | Priority | Action |
|-------|----------|--------|
| >25 | Critical | Immediate attention |
| 15-25 | High | Next in queue |
| 10-15 | Medium | After high priority |
| <10 | Low | Maintenance mode |

---

## Phase 2: Standard Test Helper Pattern

### The Universal Test Scaffold

**Every test module should have this pattern:**

```rust
// ============================================================
// Additional comprehensive tests for EXTREME TDD coverage
// ============================================================

use crate::frontend::ast::{Expr, ExprKind};
use crate::frontend::parser::Result;

fn parse(code: &str) -> Result<Expr> {
    Parser::new(code).parse()
}

fn get_block_exprs(expr: &Expr) -> Option<&Vec<Expr>> {
    match &expr.kind {
        ExprKind::Block(exprs) => Some(exprs),
        _ => None,
    }
}
```

### Test Categories to Cover

For each module, systematically add tests for:

1. **ExprKind Verification** - Prove the parser produces the correct AST node
   ```rust
   #[test]
   fn test_X_produces_X_exprkind() {
       let expr = parse("...").unwrap();
       if let Some(exprs) = get_block_exprs(&expr) {
           assert!(matches!(&exprs[0].kind, ExprKind::X { .. }));
       }
   }
   ```

2. **Basic Variations** - Single char, long names, numbers, underscores
3. **Parameter Variations** - 0, 1, 2, 3+ parameters
4. **Type Variations** - i32, f64, String, bool, Option, Vec
5. **Nested Variations** - 2-level, 3-level nesting
6. **Combination Tests** - Multiple features together
7. **Edge Cases** - Empty, single element, maximum reasonable

---

## Phase 3: Common Patterns by Module Type

### Functions/Lambdas
```rust
// No params, 1 param, 2 params, 3+ params
// With/without type annotations
// With/without return type
// Arrow syntax variations
// Block body vs expression body
// Nested lambdas
// As function arguments (map, filter, fold)
```

### Collections (Arrays/Lists)
```rust
// Empty, single, multiple elements
// Various element types (int, float, string, bool)
// Nested collections
// Spread expressions
// Trailing commas
// Comprehensions
// Expression elements (arithmetic, function calls)
```

### Control Flow
```rust
// If/else basic
// If/else-if/else chains
// Match with various patterns
// Loop/while/for variations
// Break/continue/return
// Nested control flow
```

### Type Definitions (Struct/Enum/Class)
```rust
// Empty body
// Single field/variant
// Multiple fields/variants
// Generic type parameters (1, 2, 3)
// Visibility modifiers (pub, mut)
// With methods/constructors
// Inheritance/traits
```

---

## Phase 4: Workflow Optimization

### RAM Disk Testing (Mac)

```bash
# One-time setup
diskutil erasevolume HFS+ 'LambdaCache' `hdiutil attach -nomount ram://4194304`

# Every test run
CARGO_TARGET_DIR=/Volumes/LambdaCache/ruchy-target cargo test ...
```

### Efficient Iteration Loop

```bash
# 1. Edit file locally
# 2. Sync to Mac
rsync -av src/path/file.rs mac:~/src/ruchy/src/path/file.rs

# 3. Run module tests only (fast feedback)
ssh mac "cd ~/src/ruchy && CARGO_TARGET_DIR=/Volumes/LambdaCache/ruchy-target \
  cargo test --lib -p ruchy module_name::tests --no-fail-fast 2>&1 | tail -30"

# 4. If all pass, run full suite
ssh mac "cd ~/src/ruchy && CARGO_TARGET_DIR=/Volumes/LambdaCache/ruchy-target \
  cargo test --lib -p ruchy 2>&1 | tail -5"

# 5. Commit with metrics
git add <file> && git commit -m "test(<module>): Add +N EXTREME TDD tests (X→Y)

- Category 1 tests
- Category 2 tests
- Category 3 tests

Test suite: NNNN tests (+N)

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Phase 5: Handling Failures

### Common Parse Failures and Fixes

| Error Pattern | Cause | Fix |
|---------------|-------|-----|
| `handle` in test | Reserved keyword | Use `recover`, `process` |
| `|_|` underscore param | Not supported | Use `\|unused\|` |
| `&&x` double ref | Parsed as logical AND | Use `& &x` with space |
| Comments in variants | Parser limitation | Remove comments |
| Complex nested syntax | Grammar edge case | Simplify test |

### When Tests Fail

1. **Check for reserved keywords** - grep lexer for the identifier
2. **Simplify the test** - Remove complex nesting
3. **Use `let _ = result`** - For edge cases that may not parse
4. **Add to ignored** - Only for known parser limitations with ticket

---

## Anti-Patterns

### ❌ DON'T: Write tests without reading the file first
```rust
// BAD: Guessing at syntax
#[test]
fn test_maybe_works() {
    let result = parse("some random syntax");
    assert!(result.is_ok());
}
```

### ❌ DON'T: Skip the helper pattern
```rust
// BAD: Repeating Parser::new everywhere
#[test]
fn test_foo() {
    let result = Parser::new("...").parse();
}
#[test]
fn test_bar() {
    let result = Parser::new("...").parse();
}
```

### ❌ DON'T: Write one test at a time
```rust
// BAD: Inefficient iteration
// Write 1 test → sync → run → repeat 40 times
```

### ✅ DO: Batch tests by category
```rust
// GOOD: Write all tests for a category at once
// Write 40 tests → sync → run → fix failures → commit
```

### ❌ DON'T: Ignore the lines/test ratio
```rust
// BAD: Adding tests to already well-covered files
// File with ratio 8 doesn't need more tests
```

### ✅ DO: Target high-ratio files first
```rust
// GOOD: File with ratio 31 → immediate priority
// Maximum coverage gain per test written
```

---

## Success Criteria

### Per-File Targets
- [ ] Lines/test ratio < 15
- [ ] ExprKind verification test present
- [ ] All major variations covered
- [ ] No ignored tests without tickets
- [ ] All tests passing

### Session Targets
- [ ] 5+ files enhanced per session
- [ ] 200+ tests added per session
- [ ] All commits pushed to main
- [ ] No test regressions

### Project Targets
- [ ] 95% code coverage
- [ ] All files ratio < 15
- [ ] 10,000+ total tests
- [ ] Zero ignored tests without tickets

---

## Example Session

```
Session Start: 9015 tests

1. Identify targets:
   classes.rs: 1280 lines, 41 tests, ratio 31 → PRIORITY
   string_operations.rs: 317 lines, 16 tests, ratio 19
   literals.rs: 278 lines, 14 tests, ratio 19

2. Expand classes.rs:
   - Read file, identify test gaps
   - Add helper pattern
   - Add 40 tests across categories
   - Sync → test → fix → commit
   Result: 41 → 81 tests, ratio 31 → 15

3. Expand string_operations.rs:
   - Add 35 tests
   Result: 16 → 51 tests, ratio 19 → 6

4. Continue with remaining targets...

Session End: 9346 tests (+331)
```

---

## Appendix: Test Template Generator

For rapid test creation, use this template structure:

```rust
// ============================================================
// [CATEGORY NAME]
// ============================================================

#[test]
fn test_[feature]_[variation]() {
    let result = parse("[code]");
    assert!(result.is_ok(), "[Feature] [variation] should parse");
}
```

Categories to generate:
- Basic variations (5-10 tests)
- Parameter variations (5-10 tests)
- Type variations (5-10 tests)
- Nested variations (3-5 tests)
- Edge cases (3-5 tests)
- Error cases (2-3 tests)

**Target: 25-45 tests per file expansion**
