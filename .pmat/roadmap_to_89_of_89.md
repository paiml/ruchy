# Roadmap to 89/89 (100% Grammar Implementation)

## Current State

**Grammar: 85/89 (95%, Grade A+ 🏆)**
**Commits**: c97b5ad4 (effect_decl), d15a8a95 (continuation doc)

## 🚨 CRITICAL: Update grammar.yaml First!

**DEFECT**: effect_decl shows `implemented: false` in grammar.yaml but was fully implemented in commit c97b5ad4.

**Fix Required**:
```yaml
# In grammar/ruchy-grammar.yaml line 551:
effect_decl:
  rule: "visibility? 'effect' identifier generic_params? '{' effect_operations '}'"
  implemented: true  # ← Change from false to true
  test_coverage: 100  # ← Update from 0
  reason: "Implemented in c97b5ad4 - transpiles to Rust traits"
```

After fix: **86/89 (97%, Grade A+ 🏆)**

## Remaining 3 Features to Reach 89/89

### 1. SPEC-001-J: handler_expr (Effect Handlers)
- **Status**: Not implemented
- **Complexity**: 4-6 hours
- **Syntax**: `handle expr with { operation => handler }`
- **Dependencies**: ✅ effect_decl complete (c97b5ad4)
- **Impact**: 87/89 (98%)

**Implementation Plan**:
- **Phase 1 RED**: Create failing test
- **Phase 2 GREEN**: Implement parser, transpiler, interpreter
- **Phase 3 REFACTOR**: Quality gates (≤10 complexity, zero clippy errors)
- **Phase 4 VALIDATE**: All 3 modes pass

**Files to modify**:
- `grammar/ruchy-grammar.yaml` - Mark implemented
- `src/frontend/parser/effects.rs` - Add handler parsing
- `src/backend/transpiler/effects.rs` - Add handler transpilation
- `src/runtime/interpreter.rs` - Add Handler case
- `tests/spec_001_three_mode_validation.rs` - Add test

### 2. actor_ask operator (<?): Ask Operation
- **Status**: Not implemented
- **Complexity**: 2-3 hours
- **Syntax**: `actor <? message` (query with response)
- **Symbol**: `<?`
- **Dependencies**: None (actors already implemented)
- **Impact**: 88/89 (99%)

**Implementation Plan**:
- Similar to existing actor_send (`<-`) operator
- Add `ActorAsk` token to lexer
- Add binary operator case in parser
- Transpile to async ask/await pattern
- Interpreter: Simulate ask-response

**Files to modify**:
- `grammar/ruchy-grammar.yaml` - Mark implemented
- `src/frontend/lexer.rs` - Add ActorAsk token
- `src/frontend/parser/operators.rs` - Add <? parsing
- `src/backend/transpiler/operators.rs` - Transpile to ask
- `src/runtime/interpreter.rs` - Evaluate ask

### 3. expression_roundtrip: Property Test
- **Status**: Not implemented
- **Complexity**: 3-4 hours
- **Description**: "Parse then pretty-print preserves semantics"
- **Category**: Quality property test (not runtime feature)
- **Dependencies**: None
- **Impact**: 89/89 (100% 🏆)

**Implementation Plan**:
- Create property test using proptest
- Generate random AST expressions
- Format to string (pretty-print)
- Parse back to AST
- Assert semantic equivalence
- Run 10K+ test cases

**Files to modify**:
- `grammar/ruchy-grammar.yaml` - Mark implemented
- `tests/property_roundtrip.rs` - NEW file
- Add to CI pipeline

## Excluded Features (Not Counted in 89)

### actor_send operator (<-): Already Implemented!
- **Status**: Shows `implemented: false` but exists in codebase!
- **Evidence**: `ActorSend` token exists, parser handles it
- **Action**: Mark as `implemented: true` in grammar.yaml

**Verification**:
```bash
grep -r "ActorSend" src/
# Shows: lexer.rs, parser/mod.rs, transpiler/dispatcher.rs, etc.
```

## Implementation Priority (Recommended Order)

### Sprint 1: Fix Grammar Metadata (15 minutes)
1. Update grammar.yaml: mark effect_decl as implemented
2. Update grammar.yaml: mark actor_send as implemented (if verified)
3. Commit: `[GRAMMAR] Fix metadata: effect_decl + actor_send already implemented`
4. **Result**: 86/89 or 87/89 (97-98%)

### Sprint 2: handler_expr (4-6 hours)
1. Follow EXTREME TDD (RED→GREEN→REFACTOR→VALIDATE)
2. All 3 modes working (interpreter, transpile, compile)
3. Commit: `[SPEC-001-J] Implement handler_expr - All 3 modes working`
4. **Result**: 87/89 or 88/89 (98-99%)

### Sprint 3: actor_ask (2-3 hours)
1. Follow actor_send pattern (already exists)
2. Add <? operator to parser
3. Transpile + interpret
4. Commit: `[ACTOR-ASK] Implement <? operator - All 3 modes working`
5. **Result**: 88/89 or 89/89 (99-100%)

### Sprint 4: expression_roundtrip (3-4 hours)
1. Create property test with proptest
2. Generate arbitrary expressions
3. Test parse→format→parse roundtrip
4. Commit: `[PROPERTY] Implement expression roundtrip property test`
5. **Result**: 89/89 (100% 🏆)

## Time Estimate

- **Sprint 1 (Metadata fix)**: 15 minutes
- **Sprint 2 (handler_expr)**: 4-6 hours
- **Sprint 3 (actor_ask)**: 2-3 hours
- **Sprint 4 (roundtrip)**: 3-4 hours

**Total**: **10-14 hours to reach 89/89 (100% 🏆)**

## Next Steps

1. **Immediate**: Verify actor_send implementation status
2. **Fix grammar.yaml metadata** (Sprint 1)
3. **Implement handler_expr** (Sprint 2) - continuation doc already exists
4. **Implement actor_ask** (Sprint 3)
5. **Implement roundtrip property test** (Sprint 4)

## Success Criteria

✅ All 89 grammar features marked `implemented: true`
✅ All features pass 3-mode validation (interpreter, transpile, compile)
✅ Zero clippy errors, A+ quality standards
✅ Property tests pass with 10K+ cases
✅ Grammar: 89/89 (100% 🏆)

## Reference

- Current continuation: `.pmat/continue_remaining_features.md`
- Grammar file: `grammar/ruchy-grammar.yaml`
- Recent commits: c97b5ad4 (effect_decl), d15a8a95 (continuation)
