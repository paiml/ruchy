# Continue: Implement Remaining Grammar Features (EXTREME TDD)

## Current State (Post SPEC-001-D)

**Grammar Implementation: 80/89 (89%, Grade B ⭐)**

**Just Completed:**
- ✅ SPEC-001-D: lazy_expr implemented - All 3 modes working
- ✅ Commit: b126c432

**Test Status:**
- All 20 three-mode validation tests: PASS (100%)
- lazy_expr: interpreter ✅, transpile ✅, compile ✅

## Next Task: Implement Remaining 6 Features (Priority Order)

### TIER 1 - High Value, Low Complexity (START HERE)
These features are already partially implemented in the codebase:

**1. async_block - Async expressions (async { ... })**
- **Rationale**: AsyncBlock already exists in AST and has parser support
- **Status**: Parser implemented (async_expressions.rs), needs transpiler + interpreter
- **Complexity**: 2-3 hours
- **Impact**: Enables async/await programming model
- **Test**: Create test_spec_001_async_block_three_modes()

### TIER 2 - Complex Features (Next Sprint)

**2. actor_decl - Actor system declarations**
- **Complexity**: 4-6 hours
- **Dependencies**: Requires message passing infrastructure
- **Impact**: Enables actor model concurrency

**3. macro_call - Macro system (name!(args))**
- **Complexity**: 6-8 hours
- **Dependencies**: Requires macro expansion infrastructure
- **Impact**: Enables metaprogramming

### TIER 3 - Research Features (Future)

**4. refined_type - Refinement types (type with constraints)**
- **Complexity**: 8-12 hours
- **Dependencies**: Requires constraint solver
- **Impact**: Enables dependent types

**5. effect_decl - Effect system declarations**
- **Complexity**: 8-16 hours
- **Dependencies**: Requires effect handler infrastructure
- **Impact**: Enables algebraic effects

**6. handler_expr - Effect handler expressions**
- **Complexity**: 8-16 hours
- **Dependencies**: Requires effect_decl + runtime support
- **Impact**: Enables effect handling

## Recommended Next Step: async_block (TIER 1)

### EXTREME TDD Workflow for async_block:

**Phase 1 - RED (Verify Failure)**
1. Create `/tmp/test_async_block.ruchy`:
```ruchy
fun main() {
    let result = async {
        println("Running async")
        42
    }
    println(result.to_string())
}
```

2. Test all three modes (should FAIL):
```bash
timeout 10 target/debug/ruchy run /tmp/test_async_block.ruchy
timeout 10 target/debug/ruchy transpile /tmp/test_async_block.ruchy
timeout 10 target/debug/ruchy compile /tmp/test_async_block.ruchy
```

3. Add test to `tests/spec_001_three_mode_validation.rs`:
```rust
#[test]
fn test_spec_001_async_block_three_modes() {
    let code = r#"
fun main() {
    let result = async {
        println("Async block")
        42
    }
    println(result.to_string())
}
"#;
    let result = validate_three_modes(code, "async_block");
    assert!(result.all_pass(), "{}", result.failure_report());
}
```

4. Verify test FAILS: `cargo test test_spec_001_async_block_three_modes`

**Phase 2 - GREEN (Minimal Implementation)**
1. Check existing parser support in `src/frontend/parser/expressions_helpers/async_expressions.rs`
2. Add transpiler support in `src/backend/transpiler/dispatcher.rs`
3. Add interpreter support in `src/runtime/interpreter.rs`
4. Add formatter support in `src/quality/formatter.rs`
5. Verify test PASSES

**Phase 3 - REFACTOR (Quality)**
1. Run clippy: `cargo clippy --all-targets -- -D warnings`
2. Check complexity: `pmat analyze complexity` (≤10)
3. Verify TDG score: `pmat tdg src/backend/transpiler/dispatcher.rs` (≥A-)

**Phase 4 - VALIDATE (Three Modes)**
1. Run all three-mode tests: `cargo test --test spec_001_three_mode_validation`
2. Update `grammar/ruchy-grammar.yaml` (mark async_block implemented: true)
3. Run grammar analysis: `bash /tmp/analyze_grammar.sh`
4. Commit with EXTREME TDD workflow documentation

**Expected Outcome:**
- Grammar: 81/89 (91%, Grade A- 🏆)
- All 21 three-mode tests pass
- Commit: [SPEC-001-E] EXTREME TDD: Implement async_block

## Toyota Way Principles

**STOP THE LINE**: If you discover ANY bug during implementation, halt work and fix it using EXTREME TDD

**GENCHI GENBUTSU (Go and See)**: Always examine actual code before making changes
- Read existing async_expressions.rs parser code
- Understand current AST structure
- Check how similar features are transpiled

**KAIZEN (Continuous Improvement)**: Each feature improves grammar implementation percentage

**JIDOKA (Built-in Quality)**: Quality gates enforce standards automatically

## Time Budget

- async_block: 2-3 hours (TIER 1)
- Remaining features: 30-50 hours total
- Goal: 100% grammar implementation (89/89 features)

## Notes

- All work uses EXTREME TDD methodology (RED → GREEN → REFACTOR → VALIDATE)
- Tests written BEFORE implementation (no exceptions)
- Quality gates enforced via pre-commit hooks
- Atomic commits with comprehensive documentation
