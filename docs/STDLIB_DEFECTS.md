# Standard Library Defects - Accessibility Issues

## Critical: These are RUNTIME/TRANSPILER DEFECTS, not missing features

**Toyota Way Principle**: Stop the line when defects are found. Document and fix systematically.

---

## DEFECT-STDLIB-001: env Module Not Accessible from Ruchy Code

**Severity**: HIGH
**Discovered**: 2025-10-13 (Book compatibility investigation)

### Problem
```ruchy
let args = env::args()  // ❌ FAILS: "use of unresolved module env"
```

### Root Cause
- `env::args()` EXISTS in `src/stdlib/env.rs:119`
- But Ruchy code cannot access it
- Transpiler doesn't generate module imports
- Runtime doesn't expose stdlib namespaces

### Current Workaround
None - feature completely inaccessible

### Fix Required
1. Add stdlib module system to runtime (env::, fs::, etc.)
2. OR expose as builtins (global `env_args()` function)
3. OR generate proper Rust `use` statements in transpiled code

### Test File
`tests/stdlib_defect_001_env_args.rs` (RED phase complete)

---

## DEFECT-STDLIB-002: String.split() Returns Internal Rust Type

**Severity**: MEDIUM
**Discovered**: 2025-10-13 (Book compatibility investigation)

### Problem
```ruchy
let parts = "hello,world".split(",")
// Returns: Split(SplitInternal { ... }) instead of ["hello", "world"]
```

### Root Cause
- `.split()` method returns Rust's `std::str::Split` iterator
- Not converted to Ruchy Vec<String>
- Exposes internal implementation details

### Current Workaround
Cannot use `.split()` effectively in Ruchy

### Fix Implemented (2025-10-13)
**Solution**: Changed transpiler to collect iterator into Vec<String>
```rust
// Before (BROKEN):
"split" => Ok(quote! { #obj_tokens.split(#(#arg_tokens),*) }),

// After (FIXED):
"split" => Ok(quote! {
    #obj_tokens.split(#(#arg_tokens),*)
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}),
```

**Location**: `src/backend/transpiler/statements.rs:1440-1444`

**Test Results**: ✅ 8/8 tests passing
- ✅ Can call .len() on result
- ✅ Can index result (parts[0])
- ✅ Can iterate over result
- ✅ Works with various delimiters
- ✅ Works in run and compile modes

---

## DEFECT-STDLIB-003: Other stdlib Functions Inaccessible

**Severity**: HIGH
**Discovered**: 2025-10-13

### Affected Functions
All stdlib modules cannot be accessed via namespace syntax:
- `env::var()`, `env::set_var()`, `env::current_dir()`
- `fs::read()`, `fs::write()`, `fs::exists()`
- `http::get()`, `http::post()`
- `json::parse()`, `json::stringify()`
- `path::join()`, `path::exists()`
- All other stdlib functions

### Root Cause
Same as DEFECT-STDLIB-001 - no namespace support

### Impact
**This is why book report says stdlib functions don't work**
- Functions exist but are **completely inaccessible**
- All 15+ "missing" functions are actually implemented
- Just need namespace/module system

---

## Testing Protocol for Fixes

**EXTREME TDD Required**:
1. **RED**: Write failing test demonstrating defect
2. **GREEN**: Fix runtime/transpiler to pass test
3. **REFACTOR**: Ensure fix doesn't break existing tests
4. **VALIDATE**: Run all stdlib examples from book

---

## Priority Order

1. **DEFECT-STDLIB-001** (env module) - Blocks command-line argument use
2. **DEFECT-STDLIB-003** (all stdlib) - Blocks 100+ stdlib functions
3. **DEFECT-STDLIB-002** (split method) - Affects string processing

---

## Status

- [x] DEFECT-STDLIB-001: ✅ **GREEN PHASE COMPLETE** - env_args() implemented and tested (2025-10-13)
- [x] DEFECT-STDLIB-002: ✅ **GREEN PHASE COMPLETE** - .split() now returns Vec<String> (2025-10-13)
- [ ] DEFECT-STDLIB-003: ❌ **IDENTIFIED** - Needs comprehensive testing

**Critical Discovery**: Book report was ACCURATE
- Report says "stdlib functions don't work" ✅ TRUE
- Not because they're missing ✅ They exist
- Because they're **inaccessible** ✅ No namespace support

This explains ALL 15+ "missing" stdlib functions in the book report.

---

## Solution Architecture

### Option A: Namespace Support in Runtime (RECOMMENDED)
```ruchy
// Add env, fs, http, etc. as Value::Module variants
let args = env::args()  // Runtime resolves env module
```

**Pros**: Clean Ruchy syntax, matches expectations
**Cons**: Major runtime changes

### Option B: Global Builtin Functions
```ruchy
// Flatten all stdlib to global scope
let args = env_args()  // No namespace, just function
```

**Pros**: Easy to implement
**Cons**: Namespace pollution

### Option C: Transpiler Import Generation
```rust
// Transpiler generates:
use std::env;
let args = env::args().collect();
```

**Pros**: Works in transpiled code
**Cons**: Doesn't help interpreter mode

**Recommended**: Option A (namespace support) for consistency
