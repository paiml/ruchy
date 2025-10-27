# ERRATUM: Ruchy v3.138.0 Release

**Date**: 2025-10-27
**Severity**: HIGH
**Status**: PARSER-081 fix INCOMPLETE

---

## Summary

The v3.138.0 release claimed to fix PARSER-081 (array literals after sequential let statements), but the fix is **incomplete** and only works for small files.

## What Was Claimed ❌

Release notes stated:
> "Fixed array literals after sequential let statements (PARSER-081)"
> "Impact: Arrays with identifiers now work correctly"

## What Actually Works ✅

- Files **<100 LOC**: Array literals after let statements work correctly
- Test coverage: 10 tests (all small examples) passing
- Small code snippets: No issues

## What Still Fails ❌

- Files **~300-450 LOC**: Parser reports spurious brace errors
- Pattern: "Expected RightBrace, found Let" on perfectly balanced code
- Evidence: dead_code_detector.ruchy (343 LOC, 73:73 brace balance)

## Root Cause Analysis

**Original Fix** (incomplete):
- Modified `src/frontend/parser/mod.rs:395-403`
- Prevented array indexing after literals/struct literals
- **Only addressed ONE manifestation of the bug**

**Actual Problem** (unfixed):
- Parser has size-dependent bug or state corruption
- Manifests differently in large files
- May be stack/buffer overflow, state accumulation, or memory issue

## Impact Assessment

### Blocked Work
- **QUALITY-002**: Cannot implement (~250+ LOC required)
- **QUALITY-001**: Cannot expand (already at limits with stubs)
- **CYCLE 6**: Quality tooling development blocked

### User Impact
- Small examples: Work fine
- Production code: May fail with cryptic errors
- Large files: Unreliable parsing

## Evidence

### File Size Testing
| Size (LOC) | Status | Notes |
|------------|--------|-------|
| <100 | ✅ Works | All tests passing |
| 100-200 | ❓ Unknown | Not tested |
| 300-450 | ❌ Fails | Brace errors |

### Specific Failure
```
File: dead_code_detector.ruchy
Size: 343 lines
Braces: 73 opening, 73 closing (verified balanced)
Error: Expected RightBrace, found Let (line 343)
Line 343: }  // Just a closing brace
```

## Recommended Actions

### Immediate
1. ✅ Update Issue #65 with size limitation discovery
2. ✅ Add erratum to GitHub release notes
3. ✅ Create this ERRATUM document
4. Warn users about large file limitations

### Investigation Needed
1. **Binary search file size threshold**: Find exact LOC limit
2. **Parser state inspection**: Check for state accumulation
3. **Memory profiling**: Look for leaks or buffer issues
4. **Stack depth analysis**: Check recursion limits
5. **Tokenizer testing**: Verify token stream correctness

### Fix Strategy
1. Create minimal reproduction (200-300 LOC range)
2. Use RuchyRuchy debugger (once enhanced) for state inspection
3. Add large file tests to prevent regression
4. Consider parser refactoring if architectural issue

## Lessons Learned

### Testing Gaps
- ❌ No tests for files >100 LOC
- ❌ No stress testing for parser
- ❌ No size boundary exploration
- ✅ Good coverage for small examples

### Investigation Errors
- Initial hypothesis was partially correct
- Fix addressed one symptom, not root cause
- Need better large-file testing strategy

### Quality Process
- TDD methodology worked for small cases
- Need property tests with varying file sizes
- Should have tested with real-world file sizes

## References

- **GitHub Issue**: https://github.com/paiml/ruchy/issues/65
- **Release**: https://github.com/paiml/ruchy/releases/tag/v3.138.0
- **Original Fix**: Commits 528e474d, d1424f16
- **Erratum Added**: 2025-10-27

---

## User Advisory

**If you encounter parser errors on large Ruchy files:**

1. **Workaround**: Split into multiple smaller files (<100 LOC each)
2. **Report**: Add details to Issue #65 with file size
3. **Help**: Share reproduction case if possible

**This issue is being actively investigated and will be fully resolved in a future release.**

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
