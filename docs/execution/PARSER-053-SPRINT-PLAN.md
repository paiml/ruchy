# PARSER-053 Sprint Plan: Multi-line Comment Parsing Fix

**Sprint**: PARSER-053
**Version Target**: v3.106.0
**GitHub Issue**: [#45 - Multi-line Code Blocks with Inline Comments](https://github.com/paiml/ruchy/issues/45)
**Priority**: CRITICAL
**Impact**: 200+ broken examples in ruchy-book
**Created**: 2025-10-21
**Status**: PLANNED

---

## 🎯 Sprint Objective

Fix the parser to properly handle comments between continued lines in multi-line code blocks, unblocking 200+ examples in the ruchy-book documentation.

## 📋 Problem Statement

### Current Behavior (BROKEN)
```ruchy
let result = some_function()
    # This comment breaks parsing
    .method_call()
    .another_method()
```
**Error**: Parser fails to handle comments between chained method calls.

### Expected Behavior (TARGET)
```ruchy
let result = some_function()
    # This comment should be skipped
    .method_call()
    .another_method()
```
**Expected**: Parser skips comment and continues parsing the method chain.

### Root Cause
Parser doesn't properly handle comment tokens when parsing line continuations, method chains, and multi-line expressions.

---

## 🔧 Implementation Plan

### Phase 1: Investigation & Test Creation (1-2h)
**EXTREME TDD - RED Phase**

1. **Analyze Current Parser Behavior**
   ```bash
   # Create minimal reproduction case
   echo 'let x = 1
       # comment
       + 2' > test_comment.ruchy

   # Test current parser behavior
   ruchy check test_comment.ruchy  # Expected to fail
   ruchy ast test_comment.ruchy    # Expected to show parse error
   ```

2. **Write Failing Tests FIRST**
   - `tests/parser/test_parser_053_multiline_comments.rs`
   - Test cases:
     - Comments between arithmetic operations
     - Comments between method chains
     - Comments between function arguments
     - Comments in array/object literals
     - Nested comments in complex expressions

3. **Property Tests**
   ```rust
   proptest! {
       #[test]
       fn prop_comments_never_break_parsing(code in valid_ruchy_code()) {
           let with_comment = insert_comment_at_random_position(code);
           prop_assert!(parse_success(with_comment));
       }
   }
   ```

### Phase 2: Parser Implementation (2-3h)
**EXTREME TDD - GREEN Phase**

1. **Modify Tokenization Strategy**
   - Location: `src/frontend/lexer/mod.rs`
   - Change: Make comment tokens transparent to parser
   - Option A: Filter comments in token stream
   - Option B: Skip comments in parser peek/advance

2. **Update Parser Logic**
   - Location: `src/frontend/parser/mod.rs`, `src/frontend/parser/expressions.rs`
   - Key functions to modify:
     - `parse_expr()` - Handle comments between operators
     - `parse_postfix()` - Handle comments between method calls
     - `parse_call_args()` - Handle comments between arguments

3. **Implement Comment Skipping**
   ```rust
   impl Parser {
       fn skip_comments(&mut self) {
           while matches!(self.peek(), Some(Token::Comment(_))) {
               self.advance();
           }
       }

       fn advance_skip_comments(&mut self) {
           self.advance();
           self.skip_comments();
       }
   }
   ```

### Phase 3: Complexity & Quality Gates (1h)
**EXTREME TDD - REFACTOR Phase**

1. **Run PMAT Quality Gates**
   ```bash
   # Check complexity (MUST be ≤10)
   pmat analyze complexity src/frontend/parser/ --max-cyclomatic 10

   # Check TDG score (MUST be A-)
   pmat tdg src/frontend/parser/ --min-grade A-

   # Verify no SATD introduced
   pmat analyze satd src/frontend/parser/ --fail-on-violation
   ```

2. **Refactor If Needed**
   - Extract helper functions if complexity > 10
   - Apply Single Responsibility Principle
   - Document why comment skipping is safe

### Phase 4: Comprehensive Testing (2-3h)

1. **Unit Tests** (100% coverage of new code)
   ```bash
   cargo test test_parser_053 --features notebook -- --nocapture
   ```

2. **Property Tests** (10K+ random inputs)
   ```bash
   cargo test test_parser_053 property_tests -- --ignored --nocapture
   ```

3. **Mutation Tests** (≥75% coverage MANDATORY)
   ```bash
   cargo mutants --file src/frontend/parser/mod.rs --timeout 300
   # Target: ≥75% CAUGHT/(CAUGHT+MISSED)
   ```

4. **Integration Tests** (Real ruchy-book examples)
   ```bash
   # Run actual failing examples from ruchy-book
   cd ../ruchy-book
   ./validate-examples.sh --filter "multiline-comments"
   ```

5. **15-Tool Validation** (ALL tools must work)
   ```bash
   # Create test file with multiline comments
   cat > test_multiline.ruchy << 'EOF'
   let result = 10
       # Add two
       + 2
       # Multiply by three
       * 3
   EOF

   # Validate with all 15 tools (NO SKIPS)
   ruchy check test_multiline.ruchy
   ruchy transpile test_multiline.ruchy
   ruchy -e "$(cat test_multiline.ruchy)"
   ruchy lint test_multiline.ruchy
   ruchy compile test_multiline.ruchy
   ruchy run test_multiline.ruchy
   ruchy coverage test_multiline.ruchy
   ruchy runtime test_multiline.ruchy --bigo
   ruchy ast test_multiline.ruchy
   ruchy wasm test_multiline.ruchy
   ruchy provability test_multiline.ruchy
   ruchy property-tests test_multiline.ruchy
   ruchy mutations test_multiline.ruchy
   ruchy fuzz test_multiline.ruchy
   ruchy notebook test_multiline.ruchy
   ```

---

## 📊 Success Criteria (ALL MANDATORY)

### Code Quality Gates
- ✅ **Complexity**: All functions ≤10 cyclomatic complexity (PMAT enforced)
- ✅ **TDG Score**: A- minimum (≥85 points)
- ✅ **SATD**: Zero TODO/FIXME/HACK comments
- ✅ **Tests**: 100% line coverage of new code

### Test Coverage
- ✅ **Unit Tests**: 10+ test cases covering all comment scenarios
- ✅ **Property Tests**: 10K+ random inputs, 100% pass rate
- ✅ **Mutation Tests**: ≥75% mutation coverage (BLOCKING)
- ✅ **Integration Tests**: All 200+ affected examples pass

### Book Compatibility Impact
- **Before**: 65% (233/359 examples)
- **After Target**: 90%+ (324+/359 examples)
- **Improvement**: +25% book compatibility

### 15-Tool Validation
- ✅ ALL 15 native tools work with multiline comments
- ✅ NO tools skipped (zero exceptions)

---

## 🚨 Quality Gate Enforcement

### Pre-Commit Requirements (BLOCKING)
```bash
# These MUST pass before commit (enforced by pre-commit hook)
make lint                     # Clippy with -D warnings
cargo test --all-features     # All tests passing
pmat tdg . --min-grade A-     # TDG score ≥85
```

### Pre-Release Requirements (BLOCKING)
```bash
# These MUST pass before v3.106.0 release
cargo mutants --timeout 600   # ≥75% mutation coverage
make test-lang-comp           # 15-tool validation passing
../ruchy-book/validate-examples.sh  # Book compatibility ≥90%
```

---

## 📅 Timeline Estimate

- **Phase 1** (Investigation + Tests): 1-2 hours
- **Phase 2** (Implementation): 2-3 hours
- **Phase 3** (Quality Gates): 1 hour
- **Phase 4** (Testing): 2-3 hours
- **Total**: 6-9 hours (1 focused workday)

---

## 🎯 Definition of Done

1. ✅ All tests passing (unit + property + mutation ≥75%)
2. ✅ PMAT quality gates passing (complexity ≤10, TDG A-)
3. ✅ Book compatibility ≥90% (324+/359 examples)
4. ✅ 15-tool validation 100% passing
5. ✅ Documentation updated (CHANGELOG.md, roadmap.yaml)
6. ✅ Version bumped to v3.106.0
7. ✅ Published to crates.io
8. ✅ GitHub issue #45 closed with verification

---

## 🔗 Related Work

### Follow-up Sprints (Medium Priority)
After PARSER-053 completion, consider:

1. **STDLIB-007**: Add array.append() and string.format() (Issue #47)
   - Impact: ~5 examples
   - Estimated: 2-3 hours

2. **FEATURE-042**: Negative array indexing (Issue #46)
   - Impact: ~5 examples
   - Estimated: 3-4 hours

### Dependencies
- None - PARSER-053 is independent and can start immediately

---

## 📝 Notes

- **Toyota Way**: Stop the line for ANY parser regression
- **EXTREME TDD**: Write tests FIRST, then implementation
- **Quality Built-In**: No bypassing quality gates (--no-verify FORBIDDEN)
- **Scientific Method**: Measure before/after book compatibility with data

**This sprint is CRITICAL - it unblocks 200+ examples and demonstrates our commitment to quality over speed.**
