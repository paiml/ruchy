# Apex Hunt: Zero Defects Autonomous Sweep

Autonomous defect hunting loop for the Ruchy compiler. Uses Toyota Way principles to systematically find and fix ALL defects.

## Philosophy

**Toyota Way Principles:**
- **Jidoka (自働化)**: Stop the line on ANY defect
- **Genchi Genbutsu (現地現物)**: Go and see - investigate root cause
- **Kaizen (改善)**: Continuous incremental improvement
- **Hansei (反省)**: Reflection - fix before adding

## Pre-Flight: Ensure Clean State

```bash
cd /home/noah/src/ruchy

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo "Uncommitted changes detected"
    cargo test --quiet && git add -A && git commit -m "wip: Pre-hunt state"
fi

# Ensure release build exists
cargo build --release
```

## Main Loop

```
for iteration in 1..∞:
    1. MEASURE: Run all quality gates
    2. IDENTIFY: Find highest-impact defect
    3. INVESTIGATE: Root cause analysis (Five Whys)
    4. FIX: Extreme TDD (RED→GREEN→REFACTOR)
    5. VALIDATE: All gates must pass
    6. COMMIT: Atomic commit with defect reference
    7. REPEAT until zero defects
```

## Phase 1: Quality Gate Sweep

Run these in order. STOP on first failure:

```bash
# Gate 1: Clippy (lint)
cargo clippy --all-targets --all-features -- -D warnings

# Gate 2: Tests
cargo test

# Gate 3: Examples validation
for example in examples/*.ruchy; do
    timeout 10 ruchy check "$example" || echo "FAIL: $example"
done

# Gate 4: Book validation (if exists)
cd ../ruchy-book && make validate 2>/dev/null || true

# Gate 5: SATD scan (technical debt)
grep -rn "TODO\|FIXME\|HACK\|XXX" src/ --include="*.rs" | head -20

# Gate 6: Property tests
cargo test --test '*property*' -- --ignored 2>/dev/null || true
```

## Phase 2: Identify Defect Pattern

**Priority Order:**
1. Clippy errors (blocking)
2. Test failures (blocking)
3. Example failures (high impact)
4. Property test failures (edge cases)
5. SATD comments (tech debt)

**Categorize by impact:**
```bash
# Count test failures by file
cargo test 2>&1 | grep "FAILED" | sort | uniq -c | sort -rn

# Count clippy warnings by type
cargo clippy 2>&1 | grep "^error\|^warning" | sed 's/::.*//' | sort | uniq -c | sort -rn
```

## Phase 3: Root Cause Analysis (Five Whys)

Before fixing, understand WHY:

1. **What** is the exact error message?
2. **Where** in the code does it occur?
3. **When** was it introduced? (`git bisect`)
4. **Why** does this code path exist?
5. **Why** wasn't this caught earlier?

**Use debugging tools:**
```bash
# Parser issues
ruchydbg tokenize /tmp/test.ruchy
ruchydbg trace /tmp/test.ruchy --analyze

# Runtime issues
renacer -- ruchy run /tmp/test.ruchy
```

## Phase 4: Extreme TDD Fix

**RED Phase:**
```bash
# Write failing test FIRST
# Test must fail before fix
cargo test <test_name> -- --nocapture
# Verify: FAIL
```

**GREEN Phase:**
```bash
# Minimal fix to pass test
# Change ONE thing at a time
cargo test <test_name>
# Verify: PASS
```

**REFACTOR Phase:**
```bash
# Clean up without changing behavior
# Check complexity: pmat tdg <file>
# Ensure: complexity ≤10, A- grade
```

## Phase 5: Validate Fix

ALL gates must pass:

```bash
# Full validation suite
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test && \
cargo build --release
```

**If validation fails:**
```bash
git checkout -- .  # Rollback
# Try narrower fix approach
```

## Phase 6: Commit

```bash
git add -A
git commit -m "[DEFECT-XXX] Brief description

- Root cause: <why it happened>
- Fix: <what was changed>
- Tests: <what tests added/fixed>

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

## Error Pattern Cookbook

| Error Pattern | Location | Fix Approach |
|--------------|----------|--------------|
| `clippy::*` | Various | Follow clippy suggestion |
| `#[ignore]` without reason | tests/ | Add reason string |
| `uninlined_format_args` | `format!()` calls | Use `{var}` syntax |
| `Token::Label` not handled | parser | Add match arm |
| Property test keyword collision | tests/ | Add keyword filter |
| Missing keyword in filter | tests/*property* | Update KEYWORDS const |

## Stuck Detection

Track attempts per defect:
```
if same defect fails 3x:
    - Document in roadmap.yaml as known issue
    - Add #[ignore = "reason"] if test
    - Move to next defect
```

## Exit Conditions

| Condition | Action |
|-----------|--------|
| Zero defects | SUCCESS - report victory |
| All gates pass | SUCCESS - clean slate |
| Stuck 5 iterations | PAUSE - reassess strategy |
| Same error 3x | SKIP - document and move on |

## Recovery

All progress committed. To resume:
```bash
git log --oneline -5  # See last fixes
cargo clippy 2>&1 | head -20  # See current state
cargo test 2>&1 | grep FAILED  # See remaining failures
```

## Final Report

When hunt complete:
```bash
echo "=== Apex Hunt Report ==="
echo "Clippy: $(cargo clippy 2>&1 | grep -c 'error\|warning') issues"
echo "Tests: $(cargo test 2>&1 | grep -E 'passed|failed')"
echo "SATD: $(grep -rn 'TODO\|FIXME' src/ | wc -l) items"
echo "Commits this session: $(git log --oneline --since='4 hours ago' | wc -l)"
```
