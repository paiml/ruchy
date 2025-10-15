# DEFECT-FMT-002: Formatter Strips All Comments

**Date**: 2025-10-15
**Severity**: 🔴 **P1 HIGH** - Loses documentation, not silent corruption
**Status**: 🛑 **ACTIVE** - Discovered in v3.88.0 verification
**Impact**: **DESTRUCTIVE** - All comments stripped, documentation lost

---

## Executive Summary

The `ruchy fmt` tool strips ALL comments from code, resulting in complete loss of documentation. While v3.88.0 fixed the P0 corruption bug (AST Debug output), the formatter still destroys valuable information.

**Status**: Code is functional but documentation is lost.

---

## Discovery

**Source**: External verification report from ruchy-cli-tools-book project
**File**: BUG_VERIFICATION_v3.88.0.md
**Date**: 2025-10-15 (post v3.88.0 release)

### Example

**Before Formatting**:
```ruchy
// ruchy-head: Output the first n lines of a file
// Sprint 4 - Chapter 4 example from Ruchy CLI Tools Book

// Returns the first n lines from a file.
// If n is greater than the number of lines, returns all lines.
// If n is 0, returns empty string.
// Algorithm: O(n) single pass through file content.
fun head_lines(file_path, n) {
    let content = fs_read(file_path)
    ...
}
```

**After Formatting**:
```ruchy
{
    fun head_lines(file_path: Any, n: Any) {
        let content = fs_read(file_path) in {
            ...
        }
    }
}
```

**All 6 comment lines GONE!**

---

## Root Cause Analysis (Five Whys)

### Why does fmt strip comments?
**Answer**: Formatter reconstructs code from AST, doesn't preserve comments.

### Why doesn't AST preserve comments?
**Answer**: Ruchy AST doesn't store comments (treats them as whitespace during parsing).

### Why doesn't parser store comments?
**Answer**: Parser discards comments during tokenization phase (standard practice).

### Why is this a problem now?
**Answer**: v3.88.0 fixed corruption bug, so people will actually use fmt now!

### Why is this high priority?
**Answer**: Comments are critical for code documentation, learning, and maintenance.

---

## Impact Assessment

### Severity: P1 HIGH (not P0)

**Why P1 not P0?**
- Code is functional (not corrupted)
- Users can avoid using fmt if they need comments
- Not silent corruption (users will notice comments are gone)

**Why High Priority?**
1. **Documentation Loss**: All inline documentation destroyed
2. **Learning Impact**: Tutorial code loses explanatory comments
3. **Maintenance Cost**: Future developers lose context
4. **User Trust**: People won't use fmt if it destroys documentation

### Affected Users

**ALL users** who:
- Write documented code (best practice)
- Create tutorial/example code
- Maintain production codebases
- Value code readability

**Example**: The ruchy-cli-tools-book project uses extensive comments for teaching.

---

## Additional Formatter Issues

### Issue 2: Significant Style Changes

**Changes Made**:
1. Wraps entire file in block braces `{ ... }`
2. Adds type annotations `: Any` everywhere
3. Changes `let x = value` to `let x = value in { ... }`
4. Modifies nesting and indentation significantly
5. Changes control flow structure

**Impact**: Code looks dramatically different, harder to recognize.

### Issue 3: Newline Display Problem

**Original**: `if ch == "\n"`
**After fmt**: `if ch == "` (literal newline in source)

**Impact**: Visual formatting issue, but functionally correct.

---

## Technical Solution Options

### Option 1: Preserve Comments in AST (PROPER FIX)

**Approach**: Store comments in AST during parsing
- Add `comments: Vec<Comment>` to AST nodes
- Associate comments with nearest AST node
- Emit comments during formatting

**Pros**:
- Proper solution
- Full comment preservation
- Industry standard approach

**Cons**:
- Significant parser changes required
- AST structure changes (breaking change)
- Complexity increase

**Effort**: HIGH (2-3 days)

### Option 2: Two-Pass Formatting (HYBRID)

**Approach**: Parse comments separately, inject during formatting
- First pass: Extract all comments with positions
- Second pass: Format code
- Third pass: Inject comments at appropriate locations

**Pros**:
- Doesn't require AST changes
- Can preserve most comments
- Incremental improvement

**Cons**:
- Complex position tracking
- May misplace some comments
- Not 100% reliable

**Effort**: MEDIUM (1-2 days)

### Option 3: Preserve Original with Patches (MINIMAL)

**Approach**: Keep original code, only apply minimal formatting changes
- Parse both original and formatted
- Generate diff patches
- Apply only necessary changes (indentation, spacing)

**Pros**:
- Preserves everything (comments, style, etc.)
- Minimal changes to user code
- Safer approach

**Cons**:
- Not a "true" formatter
- Limited formatting capabilities
- May not fix all style issues

**Effort**: LOW (1 day)

### Option 4: Document "fmt --strip-comments" Behavior (DEFER)

**Approach**: Document current behavior, defer fix
- Add warning to fmt help text
- Update documentation
- Recommend manual formatting for commented code

**Pros**:
- Zero implementation cost
- Users can make informed choice
- Buy time for proper fix

**Cons**:
- Doesn't fix the problem
- Users won't use fmt
- Looks unprofessional

**Effort**: MINIMAL (< 1 hour)

---

## Recommended Solution

### Phase 1: Document Behavior (IMMEDIATE - < 1 hour)
Add warning to fmt:
```
⚠️  Warning: fmt strips all comments. Back up files before formatting.
```

Update help text and documentation.

### Phase 2: Preserve Comments in AST (PROPER FIX - 2-3 days)
Implement proper comment preservation:
1. Add comment tracking to parser
2. Store comments in AST
3. Emit comments during formatting
4. Add comprehensive tests

### Phase 3: Fix Style Issues (FOLLOW-UP - 1-2 days)
Address other formatting issues:
- Remove unnecessary block wrapping
- Make type annotations optional
- Preserve original let syntax when possible
- Fix newline display

---

## Testing Requirements

### CLI Tests (Add to tests/cli_contract_fmt.rs)

1. **test_fmt_preserves_comments** - Verify comments are kept
2. **test_fmt_preserves_multiline_comments** - Block comments
3. **test_fmt_preserves_inline_comments** - End-of-line comments
4. **test_fmt_preserves_doc_comments** - Documentation comments
5. **test_fmt_comment_position_correct** - Comments stay in right place

### Integration Tests

1. Test with real-world documented code (head.ruchy)
2. Test with tutorial code (multiple comment styles)
3. Verify comment content unchanged
4. Verify comment positions reasonable

---

## Success Criteria

### Must Have (P1)
- ✅ Comments are preserved
- ✅ Comment positions are reasonable
- ✅ Documentation is not lost

### Should Have
- ✅ Minimal style changes
- ✅ Preserves user's formatting choices
- ✅ No unnecessary block wrapping

### Nice to Have
- ✅ Configurable formatting rules
- ✅ Type annotation control
- ✅ Perfect comment positioning

---

## External Feedback

From ruchy-cli-tools-book verification report:

> **Recommendation**: ✅ **Use with Caution**
>
> The formatter can now be used without corrupting files, but:
> - ⚠️ **Back up files first** (comments will be lost)
> - ⚠️ **Review changes carefully** (style changes significantly)
> - ⚠️ **Consider manual formatting** if you need to preserve comments
> - ✅ **Safe for code-only files** (no comments to lose)

---

## Status

**Current**: 🛑 **P1 HIGH** - Known issue, documented
**Fix Status**: Not started (proper fix requires parser changes)
**Workaround**: Use fmt only on code without important comments
**Target**: v3.89.0 or v3.90.0 (proper fix takes 2-3 days)

---

## Related

- **CRITICAL-FMT-DEBUG-FALLBACK.md** - P0 corruption bug (FIXED in v3.88.0)
- **CRITICAL-FMT-CODE-DESTRUCTION.md** - Operator mangling bug (FIXED in v3.87.0)
- **BUG_VERIFICATION_v3.88.0.md** - External verification report

---

**Date**: 2025-10-15
**Discovered By**: ruchy-cli-tools-book project verification
**Severity**: 🔴 P1 HIGH (documentation loss)
**Action**: Document behavior immediately, fix properly in future release
