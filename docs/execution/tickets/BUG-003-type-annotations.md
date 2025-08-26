# BUG-003: Type Annotations Break Parser

## Ticket Information
- **ID**: BUG-003
- **Priority**: P1 - HIGH
- **Severity**: HIGH
- **Component**: Parser
- **Reported**: 2025-08-26
- **Reporter**: REPL Demo Testing
- **Status**: OPEN

## Problem Statement
Rust-style type annotations cause parser errors, blocking all typed examples in documentation and REPL demos.

## Impact
- 48+ REPL demo files fail
- All typed function examples broken
- Cannot demonstrate type safety features

## Symptoms
```ruchy
// This fails:
fn add(x: i32, y: i32) -> i32 {
    x + y
}

// Error: Expected type
// Error: Unexpected token: ColonColon
```

## Reproduction Steps
1. Try to parse any code with type annotations:
```bash
$ ruchy -e 'fn add(x: i32, y: i32) -> i32 { x + y }'
Error: Expected type
```

## Expected Behavior
- Type annotations should be parsed (even if ignored at runtime)
- Or clear error message about unsupported feature

## Actual Behavior
- Parser fails with cryptic error
- No way to use typed examples

## Root Cause
- Type annotation syntax not implemented in parser
- Parser expects different syntax

## Acceptance Criteria
- [ ] Type annotations can be parsed (even if just ignored)
- [ ] OR: Clear error messages about unsupported features
- [ ] OR: Type inference without explicit annotations
- [ ] Documentation updated with supported syntax

## Solutions to Consider
1. **Option A**: Implement type annotation parsing (ignore at runtime)
2. **Option B**: Strip type annotations in preprocessor
3. **Option C**: Document Ruchy-specific type syntax
4. **Option D**: Full type system implementation

## Technical Notes
- Could parse and ignore type annotations for compatibility
- Consider gradual typing approach
- Update parser grammar to handle `:` and `->` in function signatures