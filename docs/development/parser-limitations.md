# Parser Limitations Document

## Overview
This document catalogs parser limitations discovered during the coverage improvement sprint. These limitations prevent approximately 40% of transpiler functionality from being tested.

## Critical Parser Gaps

### 1. Pattern Matching Limitations

#### Pattern Guards
**Status**: ❌ Not Supported
```rust
// Parser fails on:
match x {
    n if n > 0 => "positive",
    n if n < 0 => "negative",
    _ => "zero"
}
```
**Impact**: Cannot test conditional pattern matching in transpiler

#### Or Patterns
**Status**: ❌ Not Supported
```rust
// Parser fails on:
match x {
    1 | 2 | 3 => "small",
    _ => "other"
}
```
**Impact**: Cannot test alternative pattern matching

#### Rest Patterns
**Status**: ❌ Not Supported
```rust
// Parser fails on:
match list {
    [first, ..rest] => process(first, rest),
    [] => empty()
}
```
**Impact**: Cannot test slice pattern matching

#### Struct Patterns
**Status**: ⚠️ Partially Supported
```rust
// Parser fails on complex destructuring:
match point {
    Point { x: 0, y: 0 } => "origin",
    Point { x, .. } => format!("x is {}", x)
}
```
**Impact**: Limited struct pattern testing

### 2. Type System Limitations

#### Complex Generic Types
**Status**: ⚠️ Partially Supported
```rust
// Parser struggles with:
fn process<T: Clone + Debug>(items: Vec<Option<T>>) -> Result<Vec<T>, Error>
```
**Impact**: Cannot test advanced generic transpilation

#### Associated Types
**Status**: ❌ Not Supported
```rust
// Parser fails on:
type Item = String;
impl Iterator for MyType {
    type Item = String;
}
```
**Impact**: Cannot test trait implementations

### 3. Expression Limitations

#### String Interpolation Edge Cases
**Status**: ⚠️ Partially Supported
```rust
// Parser fails on:
f"Complex: {obj.method()?.field[0]}"
f"Nested: {f"inner {x}"}"
```
**Impact**: Limited string interpolation testing

#### Advanced Closures
**Status**: ⚠️ Partially Supported
```rust
// Parser struggles with:
|x: &str| -> Result<i32, Error> { x.parse() }
move |x| async { x + 1 }
```
**Impact**: Cannot test complex lambda transpilation

### 4. Statement Limitations

#### Try Blocks
**Status**: ❌ Not Supported
```rust
// Parser fails on:
let result = try {
    let x = risky_op()?;
    x * 2
};
```
**Impact**: Cannot test try block transpilation

#### Async Blocks
**Status**: ⚠️ Partially Supported
```rust
// Parser struggles with:
async move {
    let data = fetch().await?;
    process(data)
}
```
**Impact**: Limited async testing

### 5. Module System Limitations

#### Visibility Modifiers
**Status**: ⚠️ Partially Supported
```rust
// Parser struggles with:
pub(crate) fn internal() {}
pub(super) mod parent_only {}
```
**Impact**: Limited visibility testing

#### Use Declarations
**Status**: ⚠️ Partially Supported
```rust
// Parser fails on:
use std::{
    collections::{HashMap, HashSet},
    io::{self, Read, Write},
};
```
**Impact**: Cannot test complex imports

## Test Impact Analysis

### Tests That Cannot Run Due to Parser
- `transpiler_patterns_comprehensive.rs`: 8/10 tests fail
- `transpiler_result_comprehensive.rs`: 10/10 tests fail  
- `transpiler_integration.rs`: 2/10 tests fail
- **Total**: ~40 test functions blocked

### Transpiler Features That Cannot Be Tested
1. Complex pattern matching (guards, or-patterns, rest)
2. Advanced type annotations and generics
3. String interpolation with nested expressions
4. Try blocks and error propagation patterns
5. Module visibility and complex imports

## Workaround Strategies

### 1. Direct AST Construction
Instead of parsing strings, build AST nodes directly:
```rust
let ast = Expr {
    kind: ExprKind::Match {
        expr: Box::new(var_expr("x")),
        arms: vec![/* manually constructed arms */]
    },
    span: Span::default(),
};
```

### 2. Simplified Test Cases
Test what the parser CAN handle:
- Simple patterns without guards
- Basic types without complex generics
- Simple string interpolation
- Basic async/await

### 3. Integration Tests
Use complete programs that work around limitations:
- Avoid problematic syntax
- Use supported alternatives
- Test end-to-end behavior

## Priority Fixes

### High Priority (Blocks Most Tests)
1. **Pattern Guards** - Enables conditional matching tests
2. **Or Patterns** - Enables alternative matching tests
3. **Complex String Interpolation** - Enables string processing tests

### Medium Priority (Blocks Some Tests)
1. **Rest Patterns** - Enables slice matching tests
2. **Try Blocks** - Enables error handling tests
3. **Complex Generics** - Enables type system tests

### Low Priority (Workarounds Available)
1. **Visibility Modifiers** - Can test with pub/private only
2. **Complex Use Statements** - Can use simple imports
3. **Associated Types** - Can avoid for now

## Implementation Recommendations

### Phase 1: Quick Wins (1-2 days)
- Add pattern guard support (if conditions in match arms)
- Add or-pattern support (| in patterns)
- Fix string interpolation edge cases

### Phase 2: Core Features (3-5 days)
- Add rest pattern support (..)
- Add try block support
- Improve generic type parsing

### Phase 3: Advanced Features (1 week)
- Add associated type support
- Add complex visibility modifiers
- Add nested use declarations

## Conclusion

Parser limitations are the primary blocker for achieving higher test coverage. Fixing these issues would immediately enable ~40 additional tests and allow coverage to reach the 70% target. The parser should be the top priority for the next development sprint.