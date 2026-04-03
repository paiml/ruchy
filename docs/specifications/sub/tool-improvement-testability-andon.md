# Sub-spec: Tool Improvement — Testability Review, Shrinking, and Andon Cord

**Parent:** [15-tool-improvement-spec.md](../15-tool-improvement-spec.md) Sections 5-8

---

## Testability Review Gate v2.0 (TICR Quantification)

### Problem: Subjective "Test Effort" Assessment

**v3.0**: "Test effort ≤2x implementation effort" (vague)  
**v4.0**: Quantified Test-to-Implementation Complexity Ratio (TICR)

---

### Test-to-Implementation Complexity Ratio (TICR)

**Definition**: `TICR = CP_test / CP_impl`

**Complexity Points (CP)** - Fibonacci scale:
- 1 = Trivial (simple function, <20 LOC)
- 2 = Simple (straightforward logic, 20-50 LOC)
- 3 = Moderate (some branching, 50-100 LOC)
- 5 = Complex (multiple branches, 100-200 LOC)
- 8 = Very Complex (intricate logic, >200 LOC)

**Test CP Includes**:
- Unit test writing (1-2 CP)
- Property test writing (1-2 CP)
- Mutation test iteration (0-1 CP)
- CLI test writing (0-1 CP)
- **Infrastructure** (if needed): AsyncTestHarness (5 CP), ASTGenerators (3 CP), etc.

---

### TICR Gate Criteria

| TICR | Status | Action |
|------|--------|--------|
| ≤ 1.0 | 🟢 GREEN | Proceed with implementation |
| 1.0-2.0 | 🟡 YELLOW | Proceed with tech lead sign-off |
| > 2.0 | 🔴 RED | **STOP** - Build infrastructure first |

---

### Example 1: Simple Tool (Proceed)

**Tool**: `ast` (AST pretty-printer)

**Implementation CP**:
- Parse AST → traverse → format: 3 CP (100 LOC, straightforward)

**Test CP**:
- Unit tests (3 tests): 1 CP
- Property tests (3 tests, AST generators exist): 1 CP
- CLI tests (3 tests, assert_cmd): 1 CP
- Mutation tests: 1 CP
- Total: 4 CP

**TICR**: 4 / 3 = **1.33** 🟡 YELLOW

**Decision**: Proceed with tech lead sign-off (slightly high test effort)

---

### Example 2: Complex Tool (STOP)

**Tool**: `provability` (formal verification)

**Implementation CP**:
- Z3 SMT integration + property translation + proof search: 8 CP (300 LOC, very complex)

**Test CP**:
- Unit tests: 2 CP
- Property tests: 2 CP
- CLI tests: 1 CP
- Mutation tests: 1 CP
- **Infrastructure**: Z3 SMT interface: 5 CP
- **Infrastructure**: Proof benchmark dataset: 3 CP
- Total: 14 CP

**TICR**: 14 / 8 = **1.75** 🟡 YELLOW

**Decision**: Close to red, but acceptable with sign-off

---

### Example 3: Infrastructure-Blocked (STOP)

**Tool**: `notebook` (interactive server)

**Implementation CP**:
- HTTP server + WebSocket + cell execution: 5 CP (200 LOC)

**Test CP**:
- Unit tests: 2 CP
- Property tests: 2 CP
- CLI tests (rexpect): 1 CP
- Mutation tests: 1 CP
- **Infrastructure**: AsyncTestHarness: 5 CP (MISSING)
- **Infrastructure**: Playwright E2E: 8 CP (MISSING)
- Total: 19 CP

**TICR**: 19 / 5 = **3.8** 🔴 RED

**Decision**: **STOP** - Build AsyncTestHarness + Playwright E2E first

---

### Testability Review Template

```markdown
## Testability Review: {Tool Name}

**Implementation Complexity**: {CP_impl} CP
**Test Complexity**: {CP_test} CP
**TICR**: {CP_test / CP_impl} = {TICR}

**Test Breakdown**:
- Unit tests: {X} CP
- Property tests: {X} CP
- CLI tests: {X} CP
- Mutation tests: {X} CP
- Infrastructure: {list} = {X} CP

**Infrastructure Dependencies**:
- [ ] {Dependency 1} ({X} CP, {status})
- [ ] {Dependency 2} ({X} CP, {status})

**Gate Status**: {GREEN/YELLOW/RED}

**Decision**: 
{PROCEED / PROCEED_WITH_SIGNOFF / STOP_BUILD_INFRASTRUCTURE}

**Sign-off** (if YELLOW): {Tech Lead Name, Date}
```

---

## Meta-Testing: Shrinking Mechanism (v4.0)

### Problem: Property Test Failures Hard to Debug

**Example**:
```
Property test failed on:
  Expr::Binary {
    op: Div,
    left: Binary { op: Mul, left: Int(i64::MAX), right: Int(2) },
    right: Binary { op: Sub, left: Int(1), right: Int(1) }
  }
```

**This is hard to debug** (deeply nested, multiple operations)

---

### Shrinking: Minimize Failing Case

**QuickCheck/Hypothesis** automatically shrink failing cases:

```
Original:  ((i64::MAX * 2) / (1 - 1))
Shrunk:    (1 / 0)  ← Division by zero, minimal
```

**This is easy to debug** (simple, root cause clear)

---

### Meta-Test: Validate Shrinker

**Add to Phase 1C** (property-tests meta-testing):

```rust
// tests/meta/property_tests_shrinking.rs

use proptest::prelude::*;

#[test]
fn meta_shrinking_preserves_failure() {
    // Generate a complex failing case
    proptest!(|(ast: ast::Expr)| {
        // Predicate that fails on division by zero
        let predicate = |a: &ast::Expr| {
            !matches!(a, ast::Expr::Binary {
                op: BinOp::Div,
                right: box ast::Expr::Int(0),
                ..
            })
        };
        
        // If we found a failing case
        if !predicate(&ast) {
            // Shrink it
            let shrunk = shrink_expr(&ast);
            
            // Assert two properties:
            // 1. Shrunk case still fails
            prop_assert!(!predicate(&shrunk),
                "Shrunk case must preserve failure");
            
            // 2. Shrunk case is simpler (smaller AST)
            prop_assert!(shrunk.node_count() <= ast.node_count(),
                "Shrunk case must be simpler");
            
            // 3. Shrunk case is minimal (can't shrink further)
            let double_shrunk = shrink_expr(&shrunk);
            prop_assert_eq!(shrunk, double_shrunk,
                "Shrunk case must be minimal (idempotent)");
        }
    });
}
```

**This validates**:
1. Failures are preserved during shrinking
2. Complexity decreases during shrinking
3. Shrinking is idempotent (minimal case found)

**Effort**: 2 hours (add to Phase 1C)

---

## Automated Issue Creation (Andon Cord)

### Problem: Manual Issue Creation (Toil)

**Current workflow**:
1. CI fails
2. Developer notices
3. Developer creates GitHub issue manually
4. Issue gets triaged
5. Work begins

**Waste**: Steps 2-4 (human latency: hours to days)

---

### Solution: Automated Andon Cord

**Toyota Way**: Pull the Andon cord → line stops → issue created **automatically**

```yaml
# .github/workflows/quality-gates.yml

name: Quality Gates

on: [push, pull_request]

jobs:
  quality-gates:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Run quality dashboard
        id: dashboard
        run: |
          cargo build --release
          ./target/release/ruchy quality-dashboard || echo "FAILED=true" >> $GITHUB_OUTPUT
      
      - name: Commit updated dashboard
        if: success()
        run: |
          git config user.name "Quality Bot"
          git config user.email "quality@ruchy.dev"
          git add QUALITY_GATES.md
          git diff --staged --quiet || git commit -m "chore: update quality gates [skip ci]"
          git push
      
      - name: Create Issue on Failure
        if: steps.dashboard.outputs.FAILED == 'true'
        uses: actions-ecosystem/action-create-issue@v1
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          title: "🔴 Quality Gate Failure: ${{ env.FAILED_GATE_NAME }}"
          labels: quality-gate, bug, p0
          assignees: ${{ github.actor }}
          body: |
            ## Quality Gate Failure
            
            **Commit**: `${{ github.sha }}`
            **Author**: @${{ github.actor }}
            **Date**: ${{ github.event.head_commit.timestamp }}
            
            ### Failed Gate
            `${{ env.FAILED_GATE_NAME }}`
            
            ### Details
            ```
            ${{ env.FAILED_GATE_DETAILS }}
            ```
            
            ### Action Required
            1. Review the CI run at: `${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}`
            2. Fix the root cause using Five Whys analysis
            3. Add regression test
            4. Close this issue when gate passes

            ### Quality Dashboard
            See quality gates documentation for full report.
```

**This closes the loop**:
1. CI fails → **0ms** (detection)
2. Issue created → **5s** (automation)
3. Developer assigned → **5s** (automation)
4. Work begins → **minutes** (not hours)

**Latency reduction**: Hours/days → **seconds**

---

