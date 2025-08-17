# Ruchy Development Roadmap

**Single Source of Truth - Execution Focused**

## 📊 Current Status (2025-01-17)

### Build Status 
- **`make lint`**: ✅ **PASSING** (0 clippy errors with `-D warnings`)
- **`make test`**: 🟡 IMPROVING (210/229 passing - 91.7%)
- **`cargo build`**: ✅ PASSING

### Key Metrics
- **Version**: v0.3.1
- **Tests**: 210/229 passing (19 failures remaining)
- **Lint Errors**: 0 (achieved full compliance)
- **Progress**: Major milestone achieved - zero lint errors

## ✅ COMPLETED: Zero Lint Errors Achieved

### What Was Fixed
- Removed all 262 clippy violations
- Fixed duplicate match arms by merging patterns
- Added missing `# Errors` documentation
- Fixed unnecessary Result wrapping
- Resolved pass-by-value issues with &str
- Fixed unused self/async warnings
- Handled cast truncation properly

### Test Improvements (210/229 passing)
- Fixed unit literal `()` parsing
- Fixed await expression parsing precedence
- Updated tests for tokenstream spacing
- Fixed empty list compilation

## 🎯 Next Priority: Fix Remaining 19 Tests (2-3 Hours)

### Test Categories to Fix
1. **Transpiler Tests** (8 failures)
   - try operator transpilation
   - list comprehension
   - dataframe operations
   - send/ask actor operations

2. **Type Inference** (4 failures)
   - Lambda type inference
   - Pattern matching types

3. **Parser Edge Cases** (7 failures)
   - Complex pattern matching
   - Actor system syntax
   - Trait implementations

## ✅ Completed Items

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

- **CI Status**: 🟡 Partially Green
- **Test Pass Rate**: 91.7% (210/229) 🎯 Target: 100%
- **Lint Violations**: 0 ✅ **ACHIEVED**
- **Build Time**: <30 seconds ✅
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

## 📅 Updated Timeline

### Completed (6 hours) ✅
- **Hour 0-4**: Fixed all 262 clippy violations → ZERO errors
- **Hour 4-5**: Fixed parser (unit literal, await precedence)  
- **Hour 5-6**: Improved test stability

### Remaining (2-3 hours)
- **Hour 6-7**: Fix transpiler tests
- **Hour 7-8**: Fix type inference
- **Hour 8-9**: Final validation

**Progress: 67% complete (6/9 hours)**

## 📚 Documentation (To Be Consolidated)

Current: 65+ files across 8 directories
Target: 4 files + archive

Action: After code fixes, consolidate documentation in separate PR

---
*Last Updated: 2025-01-17*
*Status: ZERO LINT ERRORS ✅ | Tests: 91.7% passing*
*Achievement: Full clippy compliance with -D warnings*
*Next Goal: 100% test coverage (19 tests remaining)*