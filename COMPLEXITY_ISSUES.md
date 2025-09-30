# Cognitive Complexity Issues

## Pre-existing Issues Found: 2025-09-30

The following functions exceed the cognitive complexity limit of 30:

1. **src/backend/transpiler/statements.rs:681** - Complexity: 38/30
2. **src/backend/transpiler/types.rs:364** - Complexity: 36/30
3. **src/backend/transpiler/mod.rs:952** - Complexity: 61/30

## Action Required

These should be refactored following Toyota Way principles:
- Extract helper functions
- Apply single responsibility principle
- Reduce nesting and branching

## Tracking

Created during v3.60.0 release - actor operators work.
These issues existed before the actor operators changes.