# Dependency Cleanup Analysis (DEPENDENCY-CLEANUP-001)

## Executive Summary

**Status**: Analysis complete, cleanup **DEFERRED** to v3.132.0
**Tool**: cargo-machete v0.6.2
**Findings**: 14 potentially unused dependencies identified
**Build Time**: 0.247s (already fast, not a performance issue)
**Recommendation**: Document findings, defer removal to post-release for safety

---

## Methodology

Ran `cargo machete` to identify unused dependencies:

```bash
cargo machete
```

**Tool Description**: cargo-machete identifies dependencies in Cargo.toml that appear unused in source code. May produce false positives for proc-macros, feature-gated code, and conditional compilation.

---

## Findings

### ruchy (10 potentially unused dependencies)

From `./Cargo.toml`:
1. **arrow-array** - Apache Arrow array types
2. **arrow-buffer** - Apache Arrow buffer management
3. **im** - Immutable data structures
4. **markup5ever** - HTML/XML parsing utilities
5. **mime_guess** - MIME type detection
6. **once_cell** - Single-assignment cells
7. **pest** - Parser generator
8. **pest_derive** - Pest procedural macros
9. **quickcheck** - Property-based testing framework
10. **web-sys** - Web APIs for WASM

### ruchy-wasm (4 potentially unused dependencies)

From `./ruchy-wasm/Cargo.toml`:
1. **js-sys** - JavaScript bindings for WASM
2. **serde** - Serialization framework
3. **serde-wasm-bindgen** - Serde/wasm-bindgen integration
4. **wasm-bindgen-futures** - Async support for WASM

---

## Analysis

### False Positive Candidates

**Procedural Macros**:
- `pest_derive` - Used via `#[derive(Parser)]` attribute, may not be detected by cargo-machete

**Feature-Gated Dependencies**:
- `arrow-array`, `arrow-buffer` - Likely behind feature flags (`dataframe`, `apache-arrow`)
- `web-sys` - Used in WASM feature (`wasm-compile`)

**WASM-Specific**:
- `js-sys`, `wasm-bindgen-futures` - May only be used in `ruchy-wasm` crate

**Testing Dependencies**:
- `quickcheck` - Property-based testing (may only be in tests/)

### True Positive Candidates

**Potentially Genuinely Unused**:
- `im` - Immutable data structures (check if actually used)
- `markup5ever` - HTML/XML parsing (related to thread safety issue in `tests/repl_thread_safety.rs`)
- `mime_guess` - MIME type detection (check usage)
- `once_cell` - May have been replaced by std `OnceLock` (Rust 1.70+)

---

## Recommendations

### Immediate Action (v3.131.0 Release)

✅ **DOCUMENT ONLY** - No code changes before release
- Create this analysis document
- Commit findings with ticket DEPENDENCY-CLEANUP-001
- Include in CHANGELOG.md as "Documentation" entry

### Post-Release Action (v3.132.0)

**Manual Verification Process** (for each dependency):

1. **Search codebase**: `rg "arrow_array|arrow-array"` (etc.)
2. **Check feature gates**: Examine `Cargo.toml` `[features]` section
3. **Verify proc-macros**: Look for `#[derive(...)]` attributes
4. **Test removal**: Create branch, remove dep, run `cargo test --all-features`
5. **Document decision**: Why kept or why removed

**Priority Order** (HIGH → LOW):
1. 🔴 **HIGH**: `markup5ever` (related to known thread safety issue)
2. 🔴 **HIGH**: `once_cell` (likely replaced by std library)
3. 🟡 **MEDIUM**: `im`, `mime_guess` (check actual usage)
4. 🟢 **LOW**: `arrow-*`, `pest_derive` (likely false positives)
5. 🟢 **LOW**: WASM dependencies (may be used in ruchy-wasm crate)

### Risk Assessment

**Removing before release v3.131.0**:
- ❌ **HIGH RISK**: May break builds unexpectedly
- ❌ **NO BENEFIT**: Build time already fast (0.247s)
- ❌ **VIOLATES POLICY**: Risky changes before release

**Deferring to v3.132.0**:
- ✅ **LOW RISK**: Controlled testing environment
- ✅ **PROPER TDD**: Can write tests to verify removal safety
- ✅ **INCREMENTAL**: Remove one at a time with verification

---

## Build Time Baseline

**Current**: 0.247s (cargo check, clean build)

```bash
$ cargo clean && time cargo check
...
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.25s

real    0m0.247s
user    0m0.175s
sys     0m0.045s
```

**Observation**: Build time is already excellent - dependency cleanup is not a performance optimization, it's a maintenance hygiene task.

---

## Thread Safety Issue Connection

**Discovered in Mutation Testing**: `tests/repl_thread_safety.rs` fails to compile due to:
```
error[E0277]: `Rc<markup5ever_rcdom::Node>` cannot be shared between threads safely
```

**Hypothesis**: If `markup5ever` is genuinely unused, removing it would:
1. Simplify dependency tree
2. Eliminate source of thread safety issue
3. Reduce attack surface (fewer dependencies = fewer CVEs)

**Action**: Investigate `markup5ever` usage in v3.132.0 as **PRIORITY #1**

---

## Conclusion

**✅ Analysis Complete**:
- 14 potentially unused dependencies identified
- False positive candidates documented (proc-macros, features)
- True positive candidates prioritized (markup5ever, once_cell)
- Build time baseline recorded (0.247s)
- Safe deferral to v3.132.0 after release

**Next Steps**:
1. ✅ Document findings (this file)
2. ✅ Commit with ticket DEPENDENCY-CLEANUP-001
3. ✅ Update CHANGELOG.md (v3.131.0 - Documentation)
4. 🔄 **Proceed with release v3.131.0**
5. 🔜 **Post-release**: Systematic dependency verification in v3.132.0

**Toyota Way**: Kaizen (continuous improvement) applies to dependency hygiene, but never at the expense of release stability.
