# DEFECT-ENUM-OK-RESERVED: Parser Rejects 'Ok' as Enum Variant Inside Functions

**Ticket**: DEFECT-ENUM-OK-RESERVED
**Created**: 2025-10-09
**Severity**: P0 - Advertised feature completely broken
**Status**: 🔴 OPEN

---

## Problem Statement

**What**: Parser fails when `Ok` is used as an enum variant name inside a function
**Where**: `examples/lang_comp/15-enums/04_enum_discriminants.ruchy`
**Impact**: Advertised language feature doesn't work - violates "if it's advertised, it MUST work" principle

---

## Reproduction

### Failing Example
```ruchy
fn main() {
    enum HttpStatus {
        Ok = 200,
        NotFound = 404,
        ServerError = 500
    }
    let status = HttpStatus::Ok
    println(status)
}
```

**Error**: `✗ Syntax error: Expected RightBrace, found Enum`

### Working Example (Top Level)
```ruchy
enum HttpStatus {
    Ok = 200,
    NotFound = 404,
    ServerError = 500
}

fn main() {
    let status = HttpStatus::Ok
    println(status)
}
```

**Result**: ✅ Parses successfully

---

## Root Cause Analysis (Five Whys)

1. **Why does parsing fail?**
   → Parser expects `}` after function body starts but finds `enum` keyword

2. **Why does it expect `}`?**
   → The `Ok` token is being interpreted as something other than an identifier

3. **Why is `Ok` interpreted differently inside functions?**
   → `Ok` is a reserved token (Token::Ok) used for Result<T,E> pattern matching

4. **Why does it work at top level but not inside functions?**
   → Different parsing contexts - top level allows reserved keywords as variant names

5. **Why aren't reserved keywords allowed as variants inside functions?**
   → **BUG**: `parse_variant_name()` correctly handles `Token::Ok`, but function body parser may be consuming it first

---

## Investigation Findings

### Parser Code Analysis

**File**: `src/frontend/parser/expressions.rs`
**Lines**: 4371-4396

```rust
fn parse_variant_name(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            Ok(name)
        }
        Some((Token::Some, _)) => {
            state.tokens.advance();
            Ok("Some".to_string())
        }
        Some((Token::None, _)) => {
            state.tokens.advance();
            Ok("None".to_string())
        }
        Some((Token::Ok, _)) => {  // ✅ This DOES handle Ok!
            state.tokens.advance();
            Ok("Ok".to_string())
        }
        Some((Token::Err, _)) => {  // ✅ This DOES handle Err!
            state.tokens.advance();
            Ok("Err".to_string())
        }
        _ => bail!("Expected variant name in enum"),
    }
}
```

**Code is Correct** - The variant name parser handles `Ok` and `Err` tokens.

### Hypothesis

The issue is likely in the **function body parser** consuming the `Ok` token before the enum parser sees it. The context difference between top-level and function-level parsing causes the bug.

---

## Test Plan (EXTREME TDD)

### Unit Tests (RED → GREEN → REFACTOR)

1. **test_enum_variant_ok_inside_function** - Parse enum with Ok variant inside function
2. **test_enum_variant_err_inside_function** - Parse enum with Err variant inside function
3. **test_enum_variant_some_inside_function** - Parse enum with Some variant inside function
4. **test_enum_variant_none_inside_function** - Parse enum with None variant inside function
5. **test_enum_all_reserved_variants** - Parse enum with all reserved keywords as variants

### Property Tests (10,000 cases)

1. **prop_reserved_keywords_as_variants_never_panic** - All reserved keywords parse without panic
2. **prop_enum_inside_function_parses** - Random enum definitions inside functions parse correctly

### Mutation Tests

- Target: `parse_variant_name` function (lines 4371-4396)
- Goal: ≥75% mutation kill rate
- Pattern focus: Match arm deletions (reserved keyword cases)

---

## Fix Strategy

### Step 1: Write Failing Tests (RED)
Create test file: `tests/defect_enum_ok_reserved.rs`

### Step 2: Investigate Function Parser
- Find where function bodies are parsed
- Check if `Token::Ok` is being consumed before enum parsing
- Identify conflicting context

### Step 3: Implement Minimal Fix (GREEN)
- Ensure enum parser has priority for reserved keywords inside enum definitions
- May need to adjust lookahead or context tracking

### Step 4: Refactor (REFACTOR)
- Ensure cyclomatic complexity ≤10
- Ensure cognitive complexity ≤10
- Run PMAT quality gates

### Step 5: Comprehensive Testing
- Run all P0 tests (must pass 15/15)
- Run property tests (10K cases)
- Run mutation tests (≥75% kill)

---

## Success Criteria

- ✅ All 5 unit tests pass
- ✅ Property tests pass (10K cases each)
- ✅ Mutation test kill rate ≥75%
- ✅ PMAT TDG score ≥ A- (85+)
- ✅ Cyclomatic complexity ≤10
- ✅ Cognitive complexity ≤10
- ✅ P0 tests 15/15 passing (zero regressions)
- ✅ `examples/lang_comp/15-enums/04_enum_discriminants.ruchy` passes check/run/wasm

---

## Toyota Way Principles

**Jidoka (Stop the Line)**: HALT all other work until this P0 defect is fixed
**Genchi Genbutsu (Go and See)**: Reproduced bug empirically, identified exact failing code
**Kaizen (Continuous Improvement)**: This is not a "missing feature" - it's a BUG in advertised functionality
**Respect for People**: Users expect advertised features to work - this violates trust

---

## Next Steps

1. Create failing test (RED)
2. Fix minimal implementation (GREEN)
3. Refactor to quality standards (REFACTOR)
4. Add property tests (10K+ cases)
5. Run mutation tests (≥75% kill)
6. Verify PMAT gates pass
7. Commit with ticket reference

---

**Generated**: 2025-10-09
**Ticket**: DEFECT-ENUM-OK-RESERVED
**Priority**: P0
**Estimate**: 2-3 hours (TDD + property + mutation + PMAT)
