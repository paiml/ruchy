# Stdlib Function Accessibility Plan

## Context

**Problem**: 82 stdlib functions across 10 modules exist but are inaccessible from Ruchy code
**Root Cause**: No namespace/module system (env::, fs::, http::, etc.)
**Proven Solution**: Global builtin functions (env_args() SUCCESS)

## Scope Analysis

### Total Functions by Module
| Module | Functions | Priority | Reason |
|--------|-----------|----------|--------|
| env | 8 | HIGH | System interaction, already started |
| fs | 12 | HIGH | File operations critical |
| path | 13 | HIGH | Path manipulation critical |
| json | 10 | MEDIUM | Data interchange common |
| http | 4 | MEDIUM | Network requests |
| regex | 10 | MEDIUM | Text processing |
| time | 6 | MEDIUM | Timing operations |
| logging | 8 | LOW | Development/debugging |
| dataframe | 9 | LOW | Specialized use case |
| process | 2 | LOW | Process management |
| **TOTAL** | **82** | | |

## Recommended Approach

### Option A: Continue Global Builtins (RECOMMENDED)
**Pros**:
- ✅ Proven to work (env_args() successful)
- ✅ Minimal code changes
- ✅ Fast implementation (<1 hour per function)
- ✅ Works in interpreter AND transpiler modes

**Cons**:
- ⚠️ 82 global functions = namespace pollution
- ⚠️ No logical grouping (env_var vs fs_read vs http_get)

**Mitigation**: Use clear naming convention: `module_function`
- `env_var()`, `env_set_var()`, `env_current_dir()`
- `fs_read()`, `fs_write()`, `fs_exists()`
- `http_get()`, `http_post()`

### Option B: Implement Namespace System
**Pros**:
- ✅ Clean syntax: `env::var()`, `fs::read()`
- ✅ Matches Rust conventions
- ✅ No namespace pollution

**Cons**:
- ❌ Major runtime changes (weeks of work)
- ❌ Parser changes needed
- ❌ High risk of breaking existing code
- ❌ Unknown complexity

## Decision: Option A (Global Builtins with Naming Convention)

**Rationale**:
1. **Proven Pattern**: env_args() demonstrates this works
2. **Fast Delivery**: Can implement 10-15 functions per day
3. **Low Risk**: Minimal code changes, isolated to known areas
4. **Incremental**: Can implement highest-priority functions first
5. **Future-proof**: Can migrate to namespaces later without breaking user code

## Implementation Plan

### Phase 1: Environment Functions (HIGH PRIORITY) - 2 hours
Already done: `env_args()` ✅

Remaining 7 functions:
1. `env_var(key: String) -> Result<String>`
2. `env_set_var(key: String, value: String) -> Result<()>`
3. `env_remove_var(key: String) -> Result<()>`
4. `env_vars() -> Result<HashMap<String, String>>`
5. `env_current_dir() -> Result<String>`
6. `env_set_current_dir(path: String) -> Result<()>`
7. `env_temp_dir() -> Result<String>`

### Phase 2: File System Functions (HIGH PRIORITY) - 3 hours
12 functions:
1. `fs_read(path: String) -> Result<String>`
2. `fs_write(path: String, content: String) -> Result<()>`
3. `fs_exists(path: String) -> Bool`
4. `fs_create_dir(path: String) -> Result<()>`
5. `fs_remove_file(path: String) -> Result<()>`
6. `fs_remove_dir(path: String) -> Result<()>`
7. `fs_copy(from: String, to: String) -> Result<()>`
8. `fs_rename(from: String, to: String) -> Result<()>`
9. `fs_metadata(path: String) -> Result<Metadata>`
10. `fs_read_dir(path: String) -> Result<Vec<String>>`
11. `fs_canonicalize(path: String) -> Result<String>`
12. `fs_is_file(path: String) -> Bool`

### Phase 3: Path Functions (HIGH PRIORITY) - 3 hours
13 functions (path manipulation)

### Phase 4-7: Medium Priority Modules - 8 hours
- json (10 functions)
- http (4 functions)
- regex (10 functions)
- time (6 functions)

### Phase 8-10: Low Priority Modules - 6 hours
- logging, dataframe, process

**Total Estimate**: 20-25 hours for all 82 functions

## Implementation Pattern (From env_args Success)

For each function, apply three-layer pattern:

### Layer 1: Runtime Builtin Registration
**File**: `src/runtime/builtins.rs`
```rust
// In register_all()
self.register("env_var", builtin_env_var);

// Implementation
fn builtin_env_var(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "env_var() expects 1 argument".to_string(),
        ));
    }

    let key = args[0].as_string()?;
    match std::env::var(key.as_ref()) {
        Ok(val) => Ok(Value::from_string(val)),
        Err(_) => Err(InterpreterError::RuntimeError(
            format!("Environment variable {} not found", key)
        )),
    }
}
```

### Layer 2: Transpiler Support
**File**: `src/backend/transpiler/statements.rs`
```rust
// In try_transpile_environment_function()
"env_var" => {
    if args.len() != 1 {
        anyhow::bail!("env_var() expects 1 argument");
    }
    let key = &args[0];
    Ok(Some(quote! {
        std::env::var(#key).expect("Environment variable not found")
    }))
}
```

### Layer 3: Global Environment Registration
**File**: `src/runtime/builtin_init.rs`
```rust
// In add_environment_functions()
global_env.insert(
    "env_var".to_string(),
    Value::from_string("__builtin_env_var__".to_string()),
);
```

**File**: `src/runtime/eval_builtin.rs`
```rust
// In try_eval_environment_function()
"__builtin_env_var__" => Ok(Some(eval_env_var(args)?)),

// Implementation
fn eval_env_var(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("env_var", args, 1)?;
    let key = args[0].as_string()?;
    match std::env::var(key.as_ref()) {
        Ok(val) => Ok(Value::from_string(val)),
        Err(_) => Err(InterpreterError::RuntimeError(
            format!("Environment variable {} not found", key)
        )),
    }
}
```

## Quality Requirements

For EVERY function:
- ✅ EXTREME TDD (RED → GREEN → REFACTOR)
- ✅ Complexity ≤2 (thin wrappers only)
- ✅ Unit tests (100% coverage)
- ✅ Property tests where applicable
- ✅ Documentation with examples
- ✅ Works in interpreter AND transpiler modes

## Success Criteria

- ✅ All 82 functions accessible via global builtins
- ✅ All functions tested (RED → GREEN)
- ✅ Book compatibility improves significantly
- ✅ Zero SATD in implementation
- ✅ Complexity ≤2 per function

## Future Migration Path

If namespace system is implemented later:
1. Keep global builtins as aliases
2. Add namespace support: `env::var()` calls same implementation as `env_var()`
3. Deprecate global builtins gradually
4. No breaking changes to user code

## Next Steps

1. ✅ Complete STDLIB-DEFECT-001 (env_args) - DONE
2. ✅ Complete STDLIB-DEFECT-002 (.split) - DONE
3. ⏭️ Implement remaining 7 env functions (Phase 1)
4. ⏭️ Implement 12 fs functions (Phase 2)
5. ⏭️ Continue with path, json, http, etc.

## Notes

- This plan is pragmatic and deliverable
- Follows proven pattern from env_args()
- Low risk, high value
- Can complete highest-priority functions in 1-2 days
- Total completion: 1-2 weeks for all 82 functions
