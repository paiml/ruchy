# DEFECT-E2E-PHANTOM-UI: Matrix E2E Tests Expect Non-Existent WASM REPL Interface

**Date**: 2025-10-28
**Severity**: CRITICAL
**Category**: Test Infrastructure / Phantom UI
**Status**: OPEN

## Executive Summary

Matrix E2E tests (`tests/e2e/matrix/*.spec.ts`) are written for a non-existent WASM REPL HTML interface with `#status` and `#repl-input` elements. The only existing HTML file is `static/notebook.html` which has a completely different structure. This is a classic "phantom UI" testing problem - tests were written for planned features that were never implemented.

## Problem Statement

**Test Expectations**:
```typescript
// tests/e2e/matrix/01-simple-arithmetic.spec.ts:16
await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });
const input = page.locator('#repl-input');
```

**Actual HTML**: `static/notebook.html` (full CodeMirror notebook interface)
- No `#status` element
- No `#repl-input` element
- Has `#notebook-cells`, `.CodeMirror`, `#cell-type-selector`, `#btn-add-cell`

**Test Failure Evidence**:
```
Error: expect(locator).toHaveClass(expected) failed
Locator: locator('#status')
Expected pattern: /status-ready/
Received string:  ""  (element doesn't exist)
```

## Five Whys Root Cause Analysis

1. **Why do E2E tests fail?**
   → Tests look for `#status` element that doesn't exist in HTML

2. **Why doesn't the element exist?**
   → No WASM REPL HTML file exists, only `static/notebook.html`

3. **Why is there no WASM REPL HTML file?**
   → Tests were written for a planned simple REPL UI that was never implemented

4. **Why were tests written before implementation?**
   → Tests followed TDD RED phase but GREEN phase (implementation) was skipped

5. **ROOT CAUSE**:
   → E2E tests describe future/phantom UI instead of actual deployed notebook interface

## Impact

- ❌ ALL 24 matrix E2E tests fail (8 tests × 3 browsers)
- ❌ Cannot validate matrix math features work in browser
- ❌ Phase 4 Week 4 Quality Verification blocked
- ❌ Violates CLAUDE.md: "NEVER commit frontend changes without E2E tests passing"

## Correct Fix (EXTREME TDD)

### Option 1: Create Missing WASM REPL Interface (NOT RECOMMENDED)
**Pros**: Tests would pass as-is
**Cons**:
- Requires building entire new HTML interface
- Matrix features live in notebook, not simple REPL
- Tests would test wrong interface
- Violates Toyota Way (fix root cause, not add workarounds)

### Option 2: Rewrite Tests for Actual Notebook Interface (RECOMMENDED)
**Pros**:
- Tests match actual deployed UI
- Matrix features are notebook-based anyway
- Follows GENCHI GENBUTSU (go and see what's actually there)
**Cons**:
- Requires rewriting all matrix E2E tests

## Implementation Plan (Option 2 - TDD Fix)

### RED Phase: Document Current Failures
✅ Matrix E2E tests fail looking for `#status`, `#repl-input`
✅ Root cause identified: phantom UI testing
✅ Tracking issue created (this document)

### GREEN Phase: Rewrite Tests for Notebook UI
1. **Update test structure** to use actual notebook elements:
   ```typescript
   // BEFORE (phantom UI)
   await expect(page.locator('#status')).toHaveClass(/status-ready/);
   const input = page.locator('#repl-input');

   // AFTER (actual notebook UI)
   await page.waitForSelector('#notebook-cells', { timeout: 10000 });
   const codeMirror = page.locator('.CodeMirror').first();
   await codeMirror.click();
   await page.keyboard.type('10 + 20');
   await page.keyboard.press('Shift+Enter');
   ```

2. **Update test expectations** to match notebook output structure
3. **Run tests** against actual notebook interface
4. **Verify** all 24 tests pass (8 tests × 3 browsers)

### REFACTOR Phase: Clean Up Test Structure
- Extract notebook interaction helpers (`executeCell`, `getOutput`)
- Add selector validation helper (prevent future phantom UI)
- Document actual UI element IDs in test README

## Files Affected

**Tests** (8 files, 24 tests total):
- `tests/e2e/matrix/01-simple-arithmetic.spec.ts`
- `tests/e2e/matrix/02-csv-workflow.spec.ts`
- `tests/e2e/matrix/03-statistical-analysis.spec.ts`
- `tests/e2e/matrix/04-time-series.spec.ts`

**HTML** (actual interface):
- `static/notebook.html` (target for tests)

## Acceptance Criteria

✅ **All 24 matrix E2E tests rewritten** to use actual notebook UI
✅ **All tests pass** in all 3 browsers (chromium, firefox, webkit)
✅ **Selector validation** helper added to prevent future phantom UI
✅ **Documentation** updated with correct UI element IDs
✅ **Phase 4 Week 4** unblocked

## Prevention

**Add to Pre-commit Hooks**:
```bash
# Validate E2E tests match actual HTML structure
npm run validate-e2e-selectors
```

**Add to CLAUDE.md**:
```markdown
## E2E Test Protocol (MANDATORY)

1. 🔍 **GENCHI GENBUTSU**: Read actual HTML file before writing E2E tests
2. ✅ **Selector Validation**: Use validateSelectors() helper
3. 🚫 **No Phantom UI**: Never write tests for planned/future UI elements
4. ✅ **Manual Verification**: View page in browser to confirm elements exist
```

## References

- **CLAUDE.md**: "NEVER commit frontend changes without E2E tests passing"
- **Toyota Way**: GENCHI GENBUTSU (go and see what's actually there)
- **DEFECT-001**: Previous phantom UI issue with notebook CSS selectors

## Action Items

**Immediate** (Week 4 completion):
1. Create ticket: [E2E-PHANTOM-UI-001]
2. Rewrite all 8 matrix E2E test files
3. Run full test suite validation
4. Update CLAUDE.md with E2E testing protocol
5. Commit with EXTREME TDD documentation

**Future** (Week 5+):
1. Add selector validation to CI/CD
2. Create E2E test template with correct patterns
3. Document actual UI elements in testing guide

---

**Created**: 2025-10-28
**Author**: Quality Assurance Team
**Toyota Way**: GENCHI GENBUTSU - Don't test phantom UI, test what's actually there
