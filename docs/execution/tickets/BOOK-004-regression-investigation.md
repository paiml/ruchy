# BOOK-004: v1.17 Regression Root Cause Analysis (BUG-001)  

**Priority**: P1 - HIGH (Understanding needed)  
**Impact**: Understanding why 299 examples broke  
**Duration**: 1 day  
**Coverage Target**: N/A (investigation)  
**Complexity Target**: N/A (investigation)

## Problem Statement

Between v1.16 and v1.17, the number of failing book examples jumped from ~100 to 299. This regression needs investigation to understand root cause and determine if fixes or reverts are needed.

## Investigation Plan

### Phase 1: Git Bisect (2 hours)
```bash
# Find the breaking commit
git bisect start
git bisect bad v1.17.0
git bisect good v1.16.0

# Run book tests at each commit
while git bisect; do
    cargo build --release
    cd ../ruchy-book
    make test | grep "passed:"
done
```

### Phase 2: Change Analysis (2 hours)
- Review commits between versions
- Identify quality/validation changes
- Document stricter checking added
- List new error types introduced

### Phase 3: Categorize Failures (4 hours)
```rust
// Group failures by error type
- Type checking failures: X examples
- Parser strictness: Y examples  
- Missing features: Z examples
- Runtime validation: W examples
```

### Phase 4: Impact Assessment (2 hours)
- Determine if regression is "good" (catching real bugs)
- Identify false positives from overly strict checks
- Recommend fixes vs reverts
- Create action plan

## Expected Findings

Based on roadmap notes, likely causes:
1. **Stricter Type Validation** - Quality sprint added checks
2. **Parser Enhancements** - More syntax validation
3. **Runtime Safety** - Additional runtime checks
4. **Linting Integration** - Code quality requirements

## Investigation Queries

### Query 1: Error Message Analysis
```bash
# Collect all unique error messages
cd ../ruchy-book
for f in tests/**/*.ruchy; do
    ruchy compile "$f" 2>&1 | grep "Error:"
done | sort -u > error_types.txt
```

### Query 2: Working vs Broken Diff
```bash
# Compare v1.16 vs v1.17 outputs
for f in tests/**/*.ruchy; do
    echo "=== $f ==="
    ruchy-v1.16 compile "$f" 2>&1 > old.txt
    ruchy-v1.17 compile "$f" 2>&1 > new.txt
    diff old.txt new.txt
done
```

### Query 3: Commit Impact
```bash
# Test each significant commit
for commit in $(git log --oneline v1.16.0..v1.17.0 | awk '{print $1}'); do
    git checkout $commit
    cargo build --release
    echo "$commit: $(test_book_examples | grep passed)"
done
```

## Root Cause Categories (Hypothesis)

### Category 1: Type System Strictness
- Likely: Implicit conversions blocked
- Impact: ~100 examples
- Fix: Add type coercion rules

### Category 2: Parser Validation  
- Likely: Syntax patterns rejected
- Impact: ~50 examples
- Fix: Relax specific patterns

### Category 3: Missing Stdlib
- Likely: Methods assumed to exist
- Impact: ~80 examples
- Fix: Implement methods (BOOK-002)

### Category 4: Quality Gates
- Likely: Linting/formatting required
- Impact: ~70 examples
- Fix: Disable for examples or fix formatting

## Success Metrics

1. **Primary**: Understand exact cause of 299 failures
2. **Secondary**: Categorize failures by fix difficulty
3. **Tertiary**: Identify quick wins vs long-term fixes
4. **Quaternary**: Create prioritized fix list

## Deliverables

### 1. Regression Report
```markdown
# v1.17 Regression Analysis

## Summary
- Breaking commit: <hash>
- Primary cause: <description>
- Examples affected: 299
- Fixable quickly: X
- Require major work: Y

## Breakdown by Error Type
1. Type errors: X examples (Y%)
2. Parser errors: X examples (Y%)
3. Missing features: X examples (Y%)
4. Runtime errors: X examples (Y%)

## Recommendations
1. Quick fixes (1 day): ...
2. Medium fixes (1 week): ...
3. Long-term fixes (1 month): ...
```

### 2. Fix Priority Matrix
| Error Type | Count | Impact | Effort | Priority |
|-----------|-------|--------|---------|----------|
| Type annotations | 100 | High | Medium | P0 |
| Stdlib methods | 80 | High | Low | P0 |
| Parser strict | 50 | Medium | Low | P1 |
| Quality gates | 70 | Low | Low | P2 |

### 3. Revert Recommendations
- Commits safe to keep: [list]
- Commits to consider reverting: [list]
- Partial reverts needed: [list]

## Investigation Tools

### Script 1: Bulk Test Runner
```bash
#!/bin/bash
# test_all_examples.sh
count_pass=0
count_fail=0
for file in tests/**/*.ruchy; do
    if ruchy compile "$file" &>/dev/null; then
        ((count_pass++))
    else
        ((count_fail++))
        echo "FAIL: $file"
    fi
done
echo "Passed: $count_pass, Failed: $count_fail"
```

### Script 2: Error Categorizer
```python
#!/usr/bin/env python3
import subprocess
import re
from collections import Counter

errors = Counter()
for file in glob.glob("tests/**/*.ruchy"):
    result = subprocess.run(["ruchy", "compile", file], 
                          capture_output=True, text=True)
    if result.returncode != 0:
        error_type = extract_error_type(result.stderr)
        errors[error_type] += 1

for error, count in errors.most_common():
    print(f"{count}: {error}")
```

## Risk Mitigation

- **Risk**: Reverting breaks other improvements
- **Mitigation**: Selective reverts, not wholesale

- **Risk**: Investigation takes too long
- **Mitigation**: Time-box to 1 day, document findings

## Toyota Way Principles Applied

- **Genchi Genbutsu**: Go to the actual failing examples
- **5 Whys**: Deep dive into root causes
- **Kaizen**: Learn from regression for future prevention
- **Respect**: Document clearly for team understanding
- **Long-term**: Use findings to improve testing