# Issue #119: Option A Implementation Plan - Rc<RefCell<HashMap>> Refactoring

## Status
**PLANNED** - Ready for execution in dedicated session

## Prerequisites
- ✅ RED phase complete (8/8 tests failing)
- ✅ Root cause documented (double-clone at function def + call)
- ✅ Architecture analysis complete

## Refactoring Scope

**Estimated changes**: 600-800 lines across 6+ files
**Estimated time**: 2-3 hours (dedicated session)
**Risk level**: HIGH - Touches core interpreter architecture

## Step-by-Step Implementation

### Phase 1: Update Value::Closure Definition

**File**: `src/runtime/interpreter.rs`
**Location**: Lines 79-84

**Current code**:
```rust
Closure {
    params: Vec<String>,
    body: Arc<Expr>,
    env: Arc<HashMap<String, Value>>, // ← BUG: Stores snapshot
},
```

**New code**:
```rust
Closure {
    params: Vec<String>,
    body: Arc<Expr>,
    env: Rc<RefCell<HashMap<String, Value>>>, // ← FIX: Shared mutable reference
},
```

**Impact**: Every pattern match on `Value::Closure { params, body, env }` must be updated

### Phase 2: Update Interpreter::env_stack

**File**: `src/runtime/interpreter.rs`
**Location**: Line 219

**Current code**:
```rust
env_stack: Vec<HashMap<std::string::String, Value>>,
```

**New code**:
```rust
env_stack: Vec<Rc<RefCell<HashMap<std::string::String, Value>>>>,
```

**Impact**: ALL 29 access sites in interpreter.rs must be updated

### Phase 3: Update Environment Access Patterns

#### 3.1: Reading from environment (11 sites)

**Current pattern**:
```rust
if let Some(global_env) = self.env_stack.first() {
    // Access global_env directly
}
```

**New pattern**:
```rust
if let Some(global_env_ref) = self.env_stack.first() {
    let global_env = global_env_ref.borrow();
    // Access global_env via borrow
}
```

**Locations**:
- Line 1164, 1225, 1483, 1507, 2431, 2508, 2521, 3483, 7587

#### 3.2: Writing to environment (9 sites)

**Current pattern**:
```rust
if let Some(global_env) = self.env_stack.first_mut() {
    global_env.insert(key, value);
}
```

**New pattern**:
```rust
if let Some(global_env_ref) = self.env_stack.first() {
    let mut global_env = global_env_ref.borrow_mut();
    global_env.insert(key, value);
}
```

**Locations**:
- Line 1200, 1252, 1288, 2560, 3492, 3499

#### 3.3: Push/Pop operations (9 sites)

**Current pattern**:
```rust
self.env_stack.push(new_env.clone());
// ...
self.env_stack.pop();
```

**New pattern**:
```rust
self.env_stack.push(Rc::new(RefCell::new(new_env)));
// ...
self.env_stack.pop();
```

**Locations**:
- Line 1273/1275, 2578, 2583, 5251/5253, 5394/5396, 5461/5463, 6284/6302, 6411/6417, 6747/6753

### Phase 4: Update Function Definition (Closure Capture)

**File**: `src/runtime/eval_func.rs`
**Location**: Line 31

**Current code**:
```rust
let closure = Value::Closure {
    params: param_names,
    body: Arc::new(body.clone()),
    env: Arc::new(current_env.clone()), // ← BUG: First clone
};
```

**New code**:
```rust
let closure = Value::Closure {
    params: param_names,
    body: Arc::new(body.clone()),
    env: current_env_ref.clone(), // ← FIX: Rc::clone (shallow copy of Rc pointer)
};
```

**Note**: `current_env` parameter type changes to `&Rc<RefCell<HashMap<String, Value>>>`

### Phase 5: Update Function Call (Environment Handling)

**File**: `src/runtime/eval_function.rs`
**Location**: Line 274

**Current code**:
```rust
let mut call_env = closure.captured_env.clone(); // ← BUG: Second clone
```

**New code**:
```rust
let mut call_env = closure.captured_env.borrow_mut(); // ← FIX: Borrow instead of clone
```

**Impact**: All subsequent `call_env` operations must use `&mut RefMut`

### Phase 6: Update eval_function.rs Closure Struct

**File**: `src/runtime/eval_function.rs`
**Location**: Lines 82-87

**Current code**:
```rust
pub struct Closure {
    pub params: Vec<Pattern>,
    pub body: Expr,
    pub captured_env: HashMap<String, Value>, // ← BUG: Owns copy
    pub name: Option<String>,
}
```

**New code**:
```rust
pub struct Closure {
    pub params: Vec<Pattern>,
    pub body: Expr,
    pub captured_env: Rc<RefCell<HashMap<String, Value>>>, // ← FIX: Shared reference
    pub name: Option<String>,
}
```

### Phase 7: Update All Pattern Matches (50+ locations)

**Files to update**:
- `src/runtime/eval_function.rs` - 15+ match sites
- `src/runtime/bytecode/vm.rs` - 2 sites (lines 476, 748)
- `src/runtime/bytecode/compiler.rs` - 1 site (line 592)
- `src/runtime/eval_display.rs` - 1 site (line 27)
- `src/runtime/eval_array.rs` - Multiple sites
- `src/runtime/magic.rs` - 1 site (line 390)

**Pattern**:
```rust
// Before: Direct access
match function {
    Value::Closure { params, body, env } => {
        // Access env directly
    }
}

// After: Borrow when needed
match function {
    Value::Closure { params, body, env } => {
        let env_borrowed = env.borrow();
        // Access env_borrowed
    }
}
```

### Phase 8: Update Interpreter::new() Initialization

**File**: `src/runtime/interpreter.rs`
**Location**: ~Line 300 (in `new()` method)

**Current pattern**:
```rust
env_stack: vec![HashMap::new()],
```

**New pattern**:
```rust
env_stack: vec![Rc::new(RefCell::new(HashMap::new()))],
```

## Testing Strategy

### Compilation Verification (Incremental)
```bash
# After each phase, verify compilation
cargo check --lib

# Expected: Compilation errors guide next fixes
# Target: Zero errors after Phase 8
```

### Test Execution (After Phase 8)
```bash
# Run Issue #119 tests
cargo test --test issue_119_global_mutable_state --release

# Expected: 8/8 passing (was 0/8)
```

### Regression Testing
```bash
# Verify no regressions in existing tests
cargo test --lib --release

# Expected: All 4038 tests still passing
```

## Rollback Plan

**If refactoring fails**:
1. Revert all changes: `git checkout src/runtime/`
2. Re-run tests to verify clean state
3. Document blocking issues in this file

## Success Criteria

- ✅ Compilation: Zero errors, zero warnings
- ✅ Issue #119 tests: 8/8 passing (global mutations work)
- ✅ Regression tests: 4038/4038 passing (no breakage)
- ✅ PMAT TDG: All files maintain ≥A- grade
- ✅ Complexity: All functions ≤10 cyclomatic complexity

## Estimated Timeline

**Phase 1-2**: 20 minutes (Value enum + Interpreter struct)
**Phase 3**: 30 minutes (29 env_stack access sites)
**Phase 4-5**: 15 minutes (Function def + call)
**Phase 6**: 10 minutes (eval_function.rs Closure)
**Phase 7**: 60 minutes (50+ pattern matches)
**Phase 8**: 10 minutes (Interpreter::new)
**Testing**: 30 minutes (compilation + tests + PMAT)

**Total**: ~2.5 hours in dedicated session

## Notes for Next Session

1. **Import statements**: Add `use std::rc::Rc; use std::cell::RefCell;` to relevant files
2. **Clone semantics**: `Rc::clone(&env)` is shallow (just increments reference count)
3. **Borrow rules**: Cannot have mutable + immutable borrows simultaneously
4. **Testing order**: Fix compilation first, then verify tests
5. **PMAT validation**: Run `pmat tdg . --min-grade A-` before commit

## References

- **Crafting Interpreters**: Chapter 8 (Environments with Rc<RefCell>)
- **Rust Book**: Chapter 15.5 (Rc<T> and RefCell<T> patterns)
- **Issue #119**: `/home/noah/src/ruchy/docs/issues/ISSUE-119-GLOBAL-MUTABLE-STATE.md`
- **Tests**: `/home/noah/src/ruchy/tests/issue_119_global_mutable_state.rs`
