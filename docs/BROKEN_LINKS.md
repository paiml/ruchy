# Broken Documentation Links Report

**Generated**: 2025-10-13
**Tool**: `pmat validate-docs`
**Total Broken Links**: 91 (7 fixed)

## Summary

This document tracks known broken documentation links that need to be fixed over time.

## Status

- ✅ **Fixed (7/98)**: GitHub repository URLs (noahgift → paiml)
- ❌ **Remaining (91/98)**: See categories below

## Categories of Broken Links

### 1. Missing Files (High Priority)

**Missing Root Files:**
- `CONTRIBUTING.md` - Referenced in README.md (2 occurrences)
- `SPECIFICATION.md` - Referenced in examples/README.md

**Missing Directories:**
- `docs/architecture/` - Referenced in README.md
- `docs/guides/` - Multiple references (testing.md, code-quality.md, formal-methods.md, etc.)
- `examples/scoring/`, `examples/testing/`, `examples/linting/` - Referenced in command docs

### 2. Missing Tool Documentation (Medium Priority)

**docs/tools/ missing:**
- `ast.md` - AST Analysis guide
- `test.md` - Test Framework guide
- `fmt.md` - Code Formatter guide
- `lint.md` - Linter guide

### 3. Notebook Book Links (Low Priority)

**Pattern**: docs/notebook/book/src/ references to tests/lang_comp/ that use wrong relative paths
- 60+ broken links in notebook book documentation
- These are cross-references from docs/ to tests/ that need path fixes

### 4. External Repository Links (Fixed ✅)

All GitHub URLs updated from `noahgift` to `paiml` organization.

## Enforcement

- **Pre-commit Hook**: Added `pmat validate-docs` as WARNING (non-blocking)
- **Why Non-blocking**: 91 broken links exist, need gradual fixing
- **Future**: When count drops below 10, make it BLOCKING

## How to Fix

```bash
# Run validation to see current status
pmat validate-docs

# Fix broken links incrementally
# Priority order:
# 1. Create missing root files (CONTRIBUTING.md, SPECIFICATION.md)
# 2. Create missing docs/guides/ directory with stub files
# 3. Fix notebook book relative paths
# 4. Create missing examples/ subdirectories
```

## Progress Tracking

| Date | Broken Links | Fixed This Sprint | Notes |
|------|--------------|-------------------|-------|
| 2025-10-13 | 98 | 0 | Baseline established |
| 2025-10-13 | 91 | 7 | Fixed GitHub URLs (noahgift→paiml) |

## Next Steps

1. Create `CONTRIBUTING.md` with contribution guidelines
2. Move `docs/SPECIFICATION.md` to root or create stub
3. Create `docs/guides/` directory with essential guides
4. Fix notebook book relative path references
5. Re-run `pmat validate-docs` and update this document

---

**Note**: This document should be updated after each sprint as links are fixed.
