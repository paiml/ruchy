# Ruchy Development Roadmap

**Single Source of Truth - Execution Focused**

## 📊 Current Status (2025-01-17)

### Build Status
- **`make lint`**: ❌ FAILING (134 clippy errors)
  - 18 duplicate match arms → `#[allow(clippy::match_single_binding)]`
  - 16 unwrap() on Result → Replace with `expect("context")`
  - 15 missing Error docs → Add `/// # Errors` sections
  - 11 unnecessary pass-by-value → Use `&str` not `String`
  - 10 expect() on Result → Add context messages
  - 9 unnecessary Result wrapping → Remove `Result<>` where unneeded
  - 20+ misc violations → Run `cargo clippy --fix`
- **`make test`**: ❌ FAILING (203/229 passing - 88.6%)
- **`cargo build`**: ✅ PASSING

### Key Metrics
- **Version**: v0.3.1
- **Tests**: 203/229 passing (26 failures with 3 root causes)
- **Lint Errors**: 134 clippy violations (30 min fix with --fix)
- **Actual Time to Resolution**: 11 hours (not 36)

## 🔴 Priority Zero: Get CI Green (30 minutes)

```bash
# Automated fixes
cargo clippy --fix --allow-dirty --allow-staged
cargo fmt

# Manual review and commit
git diff
git add -A
git commit -m "fix: Apply clippy auto-fixes"
```

## ✅ Actual Critical Path (11 Hours Total)

### Day 1 Morning (4 hours)
#### 1. Fix All Clippy Violations (30 min)
- Run `cargo clippy --fix`
- Add `#[allow()]` for intentional patterns
- Replace `unwrap()` with `expect("context")`

#### 2. Parser Struct Literal (30 min)
```rust
// In TokenStream - add this method
fn peek_nth_is_colon(&self, n: usize) -> bool {
    self.tokens.get(self.pos + n)
        .map(|t| matches!(t, Token::Colon))
        .unwrap_or(false)
}
```

#### 3. Semicolon Insertion (30 min)
- Location: `parser/core.rs:58-67`
- Change: Make semicolons optional for final expressions

#### 4. Remove Dead Code (2.5 hours)
- Delete `src/frontend/parser_old.rs` (1575 lines)
- Remove unused fuzz tests
- Consolidate duplicate test files

### Day 1 Afternoon (3 hours)

#### 5. Constraint Deferred Resolution (2 hours)
```rust
// In middleend/infer.rs
struct InferenceContext {
    constraints: Vec<(TyVar, TyVar)>,  // Add this
    // ... existing fields
}

impl InferenceContext {
    fn solve_constraints(&mut self) {
        while let Some((a, b)) = self.constraints.pop() {
            self.unify(a, b).ok();  // Ignore failures for now
        }
    }
}
```

#### 6. Fix Try/Catch (1 hour)
```rust
// Correct transpilation in backend/transpiler.rs
fn transpile_try_catch(&self, try_block: &Expr, catch_var: &str, catch_block: &Expr) -> TokenStream {
    quote! {
        match (|| -> Result<_, Box<dyn std::error::Error>> {
            Ok(#try_block)
        })() {
            Ok(val) => val,
            Err(e) => {
                let #catch_var = e;
                #catch_block
            }
        }
    }
}
```

### Day 2 (4 hours)

#### 7. Fix Remaining Transpilation (3 hours)
- Range operators: Add `..=` support
- Actor send/ask: Use existing message enum
- DataFrame: Trait abstraction, not direct Polars

#### 8. Update Test Assertions (1 hour)
- Fix spacing expectations
- Update for new transpilation output
- Add round-trip tests

## 📋 Root Cause Analysis

### 26 Test Failures = 3 Problems

1. **Parser Ambiguity** (6 tests)
   - Struct literal lookahead
   - Semicolon insertion
   - Time: 1 hour

2. **Missing Type Propagation** (4 tests)
   - Lambda scope issue
   - Constraint resolution
   - Time: 2 hours

3. **Incorrect Transpilation** (16 tests)
   - Try/catch pattern
   - Range operators
   - Actor messages
   - Time: 4 hours

## 🚫 Not Needed (Overengineering)

- ❌ Visitor pattern refactoring
- ❌ Separate semantic analysis phase
- ❌ Bidirectional type checking (for now)
- ❌ Incremental compilation
- ❌ 36-hour timeline

## 📐 Technical Decisions

### Immediate Simplifications
1. Use `cargo clippy --fix` for automatic repairs
2. Add `#[allow()]` pragmas for intentional patterns
3. Delete `parser_old.rs` entirely (1575 lines of debt)

### Documentation Consolidation
```
docs/
├── specification.md    # Merge all spec files
├── implementation.md    # Architecture + design
├── contributing.md      # Dev guide + quality gates
└── archive/            # Move 60+ files here
```

## 📊 Success Metrics

- **CI Status**: Green (priority zero)
- **Test Pass Rate**: 100%
- **Lint Violations**: 0
- **Build Time**: <30 seconds
- **Cyclomatic Complexity**: ≤10 enforced via test

## 🎯 Quality Gate Enforcement

```rust
#[test]
fn enforce_complexity_limit() {
    let max_complexity = 10;
    for function in project_functions() {
        assert!(
            function.cyclomatic_complexity() <= max_complexity,
            "{} exceeds complexity limit", 
            function.name
        );
    }
}
```

## 📅 Realistic Timeline

- **Hour 0-0.5**: Fix all clippy violations
- **Hour 0.5-1**: Parser fixes
- **Hour 1-3.5**: Remove dead code
- **Hour 3.5-5.5**: Type system fixes
- **Hour 5.5-6.5**: Try/catch transpilation
- **Hour 6.5-9.5**: Remaining transpilation
- **Hour 9.5-10.5**: Test assertions
- **Hour 10.5-11**: Final validation

**Total: 11 hours over 2 days**

## 📚 Documentation (To Be Consolidated)

Current: 65+ files across 8 directories
Target: 4 files + archive

Action: After code fixes, consolidate documentation in separate PR

---
*Last Updated: 2025-01-17*
*Revised Estimate: 11 hours (was 36)*
*Priority Zero: Fix lint violations first*