# Issue #119: Global Mutable State Not Persisting Across Function Calls

## Status
**BLOCKER**: Blocks BENCH-002 (Matrix Multiplication)

## Reproduction
```ruby
let mut global_counter = 0

fun increment() {
    global_counter = global_counter + 1
}

increment()
println(global_counter)  // Expected: 1, Actual: 0
```

## Root Cause Analysis (Five Whys)

**Why does global_counter stay at 0?**
→ Because modifications inside the function don't propagate back to the parent scope

**Why don't modifications propagate back?**
→ Because the function works with a cloned copy of the environment

**Why does the function get a cloned copy?**
→ Because there are TWO clones happening:
1. **Function Definition** (src/runtime/eval_func.rs:31):
   ```rust
   env: Arc::new(current_env.clone())  // Captures environment snapshot
   ```
2. **Function Call** (src/runtime/eval_function.rs:274):
   ```rust
   let mut call_env = closure.captured_env.clone();  // Clones captured snapshot
   ```

**Why are we cloning instead of using references?**
→ Because the current architecture uses `HashMap<String, Value>` for environments, which requires ownership transfer

**Why not use shared mutable references?**
→ Because we didn't implement `Rc<RefCell<HashMap>>` for shared mutable access (standard pattern for tree-walking interpreters)

## Evidence (ruchydbg Trace)

```
TRACE: → increment()
TRACE: ← increment = 1: integer     ← Function computes correct value (1)
TRACE: → println(..., 0: integer)   ← But global_counter is still 0!
```

This proves:
- `increment()` successfully computes `global_counter + 1 = 1`
- The result is correct inside the function scope
- But the global scope never sees the modification
- **Hypothesis Confirmed: Scope Copy**

## Design Constraint: Lexical Scoping vs Global Mutability

**Lexical Scoping Requirement** (must preserve):
```ruby
let x = 10
fun outer() {
    let x = 20  // Shadows outer x
    fun inner() {
        println(x)  // Should capture x=20 from outer, not x=10
    }
    inner()
}
outer()  // Prints 20 (lexical scoping)
```

**Global Mutability Requirement** (currently broken):
```ruby
let mut global_state = []
fun accumulate(val) {
    global_state = global_state + [val]  // Should modify global, not local copy
}
accumulate(1)
println(len(global_state))  // Expected: 1, Actual: 0
```

## Solution Strategy

**Option A: Rc<RefCell<HashMap>>** (Standard Pattern)
- Replace `HashMap<String, Value>` with `Rc<RefCell<HashMap<String, Value>>>`
- Enables shared mutable access without cloning
- Used by most tree-walking interpreters (Crafting Interpreters, etc.)
- **Pros**: Clean architecture, industry-standard
- **Cons**: Requires refactoring ~500 lines (Interpreter struct, all env accessors)

**Option B: Global Variables Registry** (Hybrid Approach)
- Keep current lexical scoping with cloning
- Add separate `Arc<Mutex<HashMap<String, Value>>>` for global mutable variables
- Track which variables are "global mutable" at parse time
- **Pros**: Minimal changes, preserves lexical scoping
- **Cons**: Complexity in distinguishing global vs local, architectural debt

**Option C: Two-Tier Environment** (Recommended)
- Inner environment (local variables): Cloned per function call (lexical scoping)
- Outer environment (global state): Shared reference (mutability)
- Functions search local first, then global
- **Pros**: Balance between simplicity and correctness
- **Cons**: Moderate refactoring (~200 lines)

## Implementation Plan (Option C - Two-Tier)

### GREEN Phase: Minimal Fix (≤10 complexity)

1. **Add `global_env` field to Interpreter**:
   ```rust
   pub struct Interpreter {
       environment: Vec<HashMap<String, Value>>,  // Local scopes (lexical)
       global_env: Arc<Mutex<HashMap<String, Value>>>,  // Global mutable state
   }
   ```

2. **Track global variables at parse time**:
   - `let mut x = ...` at module level → global
   - `let mut x = ...` inside function → local

3. **Modify variable lookup**:
   - Check local scopes first (lexical scoping)
   - Fall back to global_env if not found

4. **Modify variable assignment**:
   - If variable exists in global_env → update there
   - Otherwise → update local scope

### Test Coverage
- 8/8 integration tests (Issue #119)
- Property tests: Lexical scoping still works, global mutations propagate
- Mutation tests: Verify correct scope lookup logic

## Related Issues
- BENCH-002 (Matrix Multiplication) - **BLOCKED**
- BENCH-006 (File Processing with global accumulator) - potential blocker

## References
- Crafting Interpreters: Chapter 8 (Environment with Rc<RefCell>)
- ruchydbg trace: 3ms execution, type-aware tracing enabled
- Toyota Way: Stop the Line - No workarounds, fix root cause
