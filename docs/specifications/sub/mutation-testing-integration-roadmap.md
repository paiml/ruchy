# Sub-spec: Mutation Testing — Integration, Roadmap & Best Practices

**Parent:** [MUTATION_TESTING.md](../MUTATION_TESTING.md) Sections 9-16

---
## Roadmap to 90% Kill Rate

### Phase 1: Baseline (Week 1)

**Goal**: Establish baseline mutation kill rate

- [ ] Install cargo-mutants
- [ ] Run initial mutation tests on all modules
- [ ] Generate baseline report
- [ ] Categorize surviving mutants by priority
- [ ] Document baseline kill rate per module

**Deliverable**: `docs/execution/MUTATION_BASELINE_REPORT.md`

### Phase 2: Critical Modules (Week 2-3)

**Goal**: Achieve 95%+ kill rate on P0 modules

- [ ] Parser: Add arithmetic/comparison operator tests
- [ ] Parser: Add match arm coverage for all token types
- [ ] Evaluator: Add exact value tests for all operators
- [ ] Evaluator: Test all expression types
- [ ] Type Checker: Test all unification cases

**Target**: Parser (95%), Evaluator (95%), Type Checker (95%)

### Phase 3: High Priority Modules (Week 4)

**Goal**: Achieve 90%+ kill rate on P1 modules

- [ ] WASM: Test all instruction types
- [ ] WASM: Test type conversions
- [ ] REPL: Test all commands
- [ ] REPL: Test state management

**Target**: WASM (90%), REPL (90%)

### Phase 4: Medium Priority Modules (Week 5)

**Goal**: Achieve 85%+ kill rate on P2 modules

- [ ] Standard Library: Test all builtin functions
- [ ] Error Handling: Test all error types

**Target**: Stdlib (85%), Error (85%)

### Phase 5: CI/CD Integration (Week 6)

**Goal**: Prevent regressions

- [ ] Add mutation testing to pre-commit hooks
- [ ] Add mutation testing to CI/CD pipeline
- [ ] Set up mutation kill rate tracking
- [ ] Create mutation testing dashboard

**Target**: 90%+ overall kill rate, automated enforcement

## Metrics and Reporting

### Key Metrics

1. **Overall Kill Rate**: % of mutants caught by tests
   - **Target**: 90%+
   - **Formula**: (caught mutants / total mutants) * 100

2. **Module Kill Rate**: Per-module kill rates
   - **Target**: Varies by priority (P0: 95%, P1: 90%, P2: 85%)

3. **Mutation Category Kill Rate**: Kill rate by mutation type
   - **Target**: Arithmetic (95%), Boolean (95%), Comparison (95%)

4. **Test Efficiency**: Tests per mutant killed
   - **Target**: Minimize (reuse tests to kill multiple mutants)

### Report Format

```markdown
# Mutation Testing Report - YYYY-MM-DD

## Summary
- **Total Mutants**: 500
- **Caught**: 450
- **Missed**: 50
- **Kill Rate**: 90.0%

## By Module
| Module | Total | Caught | Missed | Kill Rate |
|--------|-------|--------|--------|-----------|
| Parser | 120 | 114 | 6 | 95.0% ✅ |
| Evaluator | 150 | 140 | 10 | 93.3% ✅ |
| Type Checker | 80 | 76 | 4 | 95.0% ✅ |
| WASM | 100 | 85 | 15 | 85.0% ⚠️ |
| REPL | 50 | 35 | 15 | 70.0% ❌ |

## By Mutation Type
| Type | Total | Caught | Missed | Kill Rate |
|------|-------|--------|--------|-----------|
| Arithmetic | 100 | 95 | 5 | 95.0% ✅ |
| Boolean | 80 | 76 | 4 | 95.0% ✅ |
| Comparison | 60 | 57 | 3 | 95.0% ✅ |
| Match Arms | 50 | 40 | 10 | 80.0% ⚠️ |
| Return Values | 100 | 70 | 30 | 70.0% ❌ |

## Action Items
1. REPL: Add command execution tests (15 mutants)
2. WASM: Add instruction tests (15 mutants)
3. All: Add Default::default() checks (30 mutants)
```

## Best Practices from pforge

Based on pforge's successful mutation testing:

1. **Target Critical Paths First**: Focus on schema generation, error handling, retry logic
2. **Use Exact Assertions**: `assert_eq!(actual, expected)` not `assert!(actual == expected)`
3. **Test Edge Cases**: Boundary conditions, empty inputs, all match arms
4. **Verify Behavior**: Test observable effects (logs, state changes, timeouts)
5. **Kill Arithmetic Mutants**: Test exact calculations, not "close enough"
6. **Kill Boolean Mutants**: Test both branches, verify operators (not just one path)

## References

- [cargo-mutants documentation](https://mutants.rs/)
- [Mutation Testing: A Comprehensive Survey](https://ieeexplore.ieee.org/document/5487526)
- [State of Mutation Testing at Google (2018)](https://research.google/pubs/pub46584/)
- [CLAUDE.md - TDD Methodology](../../CLAUDE.md)

## Appendix: Example Mutation Test Session

```bash
# 1. Run mutation tests on parser
$ cargo mutants --file src/frontend/parser/expressions.rs

Found 45 mutants in expressions.rs
Testing mutants...

❌ SURVIVED: line 234: replace + with - in parse_binary_expr
   Tests passed but mutant survived

# 2. Add test to kill the mutant
$ cat >> tests/parser_arithmetic_tdd.rs <<EOF
#[test]
fn test_addition_not_subtraction() {
    let ast = parse("2 + 3").unwrap();
    let result = eval_ast(&ast).unwrap();
    assert_eq!(result, Value::Integer(5));  // Would be -1 if + → -
}
EOF

# 3. Re-run mutation tests
$ cargo mutants --file src/frontend/parser/expressions.rs

Found 45 mutants in expressions.rs
Testing mutants...

✅ CAUGHT: line 234: replace + with - in parse_binary_expr
   test_addition_not_subtraction failed

Kill rate: 100% (45/45)
```

## Conclusion

Mutation testing is a powerful complement to traditional coverage metrics and PMAT quality enforcement. By achieving 90%+ mutation kill rate, we ensure that our tests don't just execute code—they actually verify correctness. This is critical for a compiler/interpreter where subtle bugs can have far-reaching consequences.

**Next Steps**:
1. Run baseline mutation tests
2. Document current kill rate
3. Implement Phase 1 (baseline)
4. Start Phase 2 (critical modules)
