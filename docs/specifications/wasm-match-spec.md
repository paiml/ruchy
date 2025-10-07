# WASM Match Expression Support Specification

## Five Whys Root Cause Analysis

1. **Why does match expression WASM compilation fail?** → "type mismatch: expected i32 but nothing on stack"
2. **Why is nothing on the stack?** → Match expressions not handled in WASM backend at all
3. **Why weren't match expressions implemented?** → Incremental development, focused on simpler constructs first
4. **Why was this not caught earlier?** → 15-tool validation exposed the gap
5. **ROOT CAUSE**: Match expressions (`ExprKind::Match`) have NO implementation in `lower_expression()`

## Defect Classification

**DEFECT TYPE**: Missing language feature in WASM backend
**SEVERITY**: HIGH - Blocks control flow examples from compiling to WASM
**IMPACT**: Users cannot use match expressions when targeting WASM
**TOYOTA WAY RESPONSE**: STOP THE LINE - implement immediately per "NO DEFECT OUT OF SCOPE"

## Technical Analysis

### Match Expression Structure

```rust
pub enum ExprKind {
    Match {
        expr: Box<Expr>,      // Expression to match against
        arms: Vec<MatchArm>,  // Match arms with patterns and bodies
    },
    // ...
}

pub struct MatchArm {
    pattern: Pattern,         // Pattern to match (literal, wildcard, etc.)
    guard: Option<Box<Expr>>, // Optional guard condition
    body: Box<Expr>,          // Expression to execute if matched
    span: Span,
}
```

### Example Match Expression

```ruchy
let number = 2

let description = match number {
    1 => "one",
    2 => "two",
    3 => "three",
    _ => "other"
}

println(f"Number {number} is {description}")
```

### WASM Lowering Strategy

Match expressions in WASM can be implemented as cascading if-else statements:

```
match value {
    pattern1 => body1,
    pattern2 => body2,
    _ => default
}

↓ Lowers to:

if value == pattern1 {
    body1
} else if value == pattern2 {
    body2
} else {
    default
}

↓ WASM:

(local.get $value)
(i32.const pattern1_value)
(i32.eq)
(if (result i32)
  (then body1_instructions)
  (else
    (local.get $value)
    (i32.const pattern2_value)
    (i32.eq)
    (if (result i32)
      (then body2_instructions)
      (else default_instructions)
    )
  )
)
```

## Implementation Plan (EXTREME TDD)

### Phase 1: Simple Literal Match (RED→GREEN→REFACTOR)

**RED**: Test that fails
```rust
#[test]
fn test_wasm_match_simple_literal() {
    let code = r#"
        let x = 2
        match x {
            1 => 10,
            2 => 20,
            _ => 0
        }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).unwrap();
    assert!(wasmparser::validate(&wasm_bytes).is_ok());
}
```

**GREEN**: Implement `lower_match()` method
- Add `ExprKind::Match` case to `lower_expression()`
- Implement cascading if-else WASM instructions
- Handle literal patterns first (integers, strings)

**REFACTOR**: Ensure complexity ≤10, A- grade

### Phase 2: Wildcard Pattern (RED→GREEN→REFACTOR)

**RED**: Test with wildcard
```rust
#[test]
fn test_wasm_match_wildcard() {
    let code = r#"
        match 99 {
            1 => 10,
            _ => 20
        }
    "#;
    // Similar test structure
}
```

**GREEN**: Implement wildcard (`_`) pattern handling

**REFACTOR**: Ensure complexity ≤10

### Phase 3: Multiple Patterns (OR patterns)

**RED**: Test with multiple patterns per arm
```rust
#[test]
fn test_wasm_match_multiple_patterns() {
    let code = r#"
        let day = 6
        match day {
            1 | 2 | 3 | 4 | 5 => "weekday",
            6 | 7 => "weekend",
            _ => "invalid"
        }
    "#;
    // Similar test structure
}
```

**GREEN**: Implement OR pattern matching (logical OR for multiple checks)

**REFACTOR**: Ensure complexity ≤10

### Phase 4: Integration with F-Strings

**Test**: Full example from 02_match.ruchy
```rust
#[test]
fn test_wasm_match_with_fstring() {
    let code = include_str!("../examples/lang_comp/03-control-flow/02_match.ruchy");
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).unwrap();
    assert!(wasmparser::validate(&wasm_bytes).is_ok());
}
```

## Acceptance Criteria

✅ **Phase 1**: Simple literal match compiles to WASM
✅ **Phase 2**: Wildcard patterns work in WASM
✅ **Phase 3**: Multiple patterns (OR) work in WASM
✅ **Phase 4**: Full example (02_match.ruchy) compiles and validates
✅ **PMAT TDG**: A- minimum (≥85 points)
✅ **Complexity**: ≤10 per function
✅ **SATD**: Zero violations
✅ **Mutation Coverage**: ≥75% for new code

## Pattern Type Support

**Phase 1 (MVP)**:
- ✅ Literal integers
- ✅ Wildcard (`_`)
- ✅ OR patterns (`|`)

**Phase 2 (Future)**:
- ⏸️ Variable bindings
- ⏸️ Tuple destructuring
- ⏸️ Struct patterns
- ⏸️ Guards

## Quality Gates

- **PMAT TDG**: `pmat tdg src/backend/wasm/mod.rs --min-grade A-`
- **Complexity**: `pmat analyze complexity src/backend/wasm/mod.rs --max-cyclomatic 10`
- **SATD**: `pmat analyze satd src/backend/wasm/mod.rs --fail-on-violation`
- **Tests**: `cargo test test_wasm_match --lib`
- **Validation**: `cargo test test_langcomp_003_02_match_expression_example_file --test lang_comp_suite`

## Timeline

- **Phase 1**: 45 minutes (simple literal match)
- **Phase 2**: 15 minutes (wildcard pattern)
- **Phase 3**: 30 minutes (OR patterns)
- **Phase 4**: 15 minutes (integration testing)
- **Total**: ~2 hours

## Success Metrics

- ✅ 02_match.ruchy compiles to valid WASM
- ✅ Zero WASM validation errors
- ✅ A- TDG score maintained
- ✅ ≤10 complexity per function
- ✅ LANG-COMP-003 test passes
