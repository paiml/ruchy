# How to Submit PMAT Bug Report to Upstream

## Bug Report Location

**File**: `docs/execution/PMAT_BUG_REPORT.md`

## Upstream Project Information

- **Project**: paiml-mcp-agent-toolkit
- **Repository**: https://github.com/paiml/paiml-mcp-agent-toolkit
- **Homepage**: https://paiml.com
- **Documentation**: https://docs.rs/pmat/2.111.0
- **Issue Tracker**: https://github.com/paiml/paiml-mcp-agent-toolkit/issues

## Submission Steps

### 1. Check Existing Issues

Before filing, search for existing related issues:

```bash
# Navigate to issues
open https://github.com/paiml/paiml-mcp-agent-toolkit/issues
# Or: xdg-open on Linux, start on Windows
```

Search terms to try:
- "TDG structural score"
- "Rust complexity analysis"
- "TDG 0.0/25"
- "structural complexity static"

### 2. Prepare Supporting Files

Create a minimal reproducible example:

```bash
# Create tarball with relevant files
cd /home/noah/src/ruchy
tar -czf pmat-bug-reproduction.tar.gz \
  src/backend/wasm/mod.rs \
  docs/execution/WASM_QUALITY_ANALYSIS.md \
  docs/execution/PMAT_BUG_REPORT.md
```

### 3. File GitHub Issue

Navigate to: https://github.com/paiml/paiml-mcp-agent-toolkit/issues/new

**Title**:
```
TDG Structural Score remains 0.0/25 despite significant Rust refactoring (2.111.0)
```

**Body Template**:
```markdown
## Summary

TDG Structural Complexity score remains static at 0.0/25 despite systematic refactoring that reduced function complexity by 80-90% and extracted 24 helper functions in Rust code.

## Environment

- **PMAT Version**: 2.111.0
- **OS**: Linux 6.8.0-83-generic
- **Language**: Rust
- **File Size**: 1,267 lines

## Expected vs Actual Behavior

**Expected**: TDG Structural score should increase when large functions are decomposed into smaller ones (all <10 complexity)

**Actual**: Score remains 0.0/25 across 4 refactoring phases despite:
- 80-90% function size reductions
- 24 helper functions extracted (all <10 complexity)
- 100% test pass rate maintained

## Evidence

| Phase | Functions Extracted | TDG Overall | TDG Structural |
|-------|---------------------|-------------|----------------|
| Baseline | 0 | 75.7/100 | **0.0/25** |
| After Phase 1-4 | 24 | 76.1/100 | **0.0/25** |

**Function Reductions:**
- `emit()`: 128 lines → 26 lines (80% reduction, complexity 15-20 → 4)
- `lower_expression()`: ~240 lines → 24 lines (90% reduction, complexity 55-60 → 4)

## Reproduction Steps

```bash
# Clone repository
git clone https://github.com/cognitivetech/ruchy
cd ruchy

# Check TDG score before refactoring
git checkout f55fb1fa  # Before refactoring
pmat tdg src/backend/wasm/mod.rs --include-components

# Check after each refactoring phase
git checkout 162570b7  # Phase 1
git checkout e3030a7f  # Phase 2
git checkout e9834ce9  # Phase 3
git checkout ec0e784b  # Phase 4

# All show Structural: 0.0/25 despite dramatic improvements
```

## Hypotheses

1. **File size threshold**: 1,267 lines may exceed TDG threshold regardless of internal structure
2. **Rust analysis limitation**: `pmat analyze complexity` returns 0 functions for Rust files
3. **Hidden metrics**: Structural score may include factors beyond cyclomatic complexity

## Additional Context

Full analysis: [Attach PMAT_BUG_REPORT.md]

This refactoring followed Toyota Way principles and achieved measurable quality improvements, but TDG metric doesn't reflect the changes. This affects confidence in PMAT's Rust analysis capabilities.

## Request

Please investigate why TDG Structural score remains static for Rust code. Is this:
1. A bug in the scoring algorithm?
2. A limitation of Rust analysis?
3. Expected behavior that should be documented?

Happy to provide additional information or test fixes!
```

### 4. Attach Supporting Documentation

- Upload `docs/execution/PMAT_BUG_REPORT.md` (full analysis)
- Optionally: Upload `docs/execution/WASM_QUALITY_ANALYSIS.md` (before/after)
- Include link to public repository if available

### 5. Add Labels (if possible)

Suggested labels:
- `bug` (if structural score should reflect changes)
- `documentation` (if behavior is expected but not documented)
- `rust` (language-specific issue)
- `tdg` (TDG component)
- `metrics` (metric accuracy)

## Follow-up Actions

After filing:

1. **Monitor issue**: Check for maintainer responses
2. **Provide clarifications**: Answer any questions from maintainers
3. **Test proposed fixes**: If maintainer provides diagnostic commands
4. **Update our docs**: Once resolved, document findings

## Expected Outcomes

Possible resolutions:

1. **Bug confirmed**: PMAT team fixes Rust structural analysis
2. **Documentation needed**: Clarification on what Structural score measures
3. **Feature request**: Add function-level complexity breakdown to TDG
4. **Known limitation**: Document that file size dominates Rust structural score

## Local Workaround

Until resolved, continue using manual verification:

```bash
# Our internal quality gate (already in place)
# All functions manually verified <10 complexity via code review
# Test coverage maintained at 100% (26/26 passing)
# Toyota Way principles applied regardless of TDG score
```

## Contact

For questions about this bug report:
- Repository: https://github.com/cognitivetech/ruchy
- File: `docs/execution/PMAT_BUG_REPORT.md`

---

**Status**: ✅ **FILED** (2025-10-03)
**Issue**: https://github.com/paiml/paiml-mcp-agent-toolkit/issues/62
**Priority**: Medium (affects metric confidence, not code quality)
