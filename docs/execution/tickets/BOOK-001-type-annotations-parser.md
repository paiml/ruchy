# BOOK-001: Type Annotations Parser Support (BUG-003)

**Priority**: P0 - CRITICAL  
**Impact**: Would fix ~100+ book examples  
**Duration**: 2-3 days  
**Coverage Target**: 80%  
**Complexity Target**: All functions < 10 (PMAT enforced)

## Problem Statement

Type annotations like `fn(i32) -> i32`, `: i32`, `: String` cause parser errors, blocking all typed examples in the book. This single issue blocks approximately 100+ examples from working.

## Root Cause Analysis (5 Whys)

1. **Why do type annotations fail?** Parser doesn't recognize type syntax
2. **Why doesn't parser recognize types?** Grammar lacks type annotation rules
3. **Why are type rules missing?** Initial implementation focused on type inference
4. **Why was inference prioritized?** To enable rapid prototyping (good choice)
5. **Why not added later?** No systematic book testing revealed the gap

## Solution Design

### Phase 1: Parse but Ignore (Day 1)
- Add type annotation parsing to grammar
- Store types in AST but don't validate
- Transpiler can initially ignore types
- **Goal**: Make examples parse without errors

### Phase 2: Basic Type Validation (Day 2)  
- Add simple type checking for primitives
- Validate function parameter types
- Check return type consistency
- **Goal**: Catch obvious type errors

### Phase 3: Testing & Coverage (Day 3)
- Add 50+ tests for type annotations
- Ensure 80% coverage of new code
- Test all book examples with types
- **Goal**: Prevent regression

## Test-Driven Development Plan

### RED Phase - Write Failing Tests
```rust
#[test]
fn test_parse_type_annotation_param() {
    let code = "fun add(x: i32, y: i32) { x + y }";
    let ast = Parser::new(code).parse();
    assert!(ast.is_ok());
}

#[test]
fn test_parse_type_annotation_return() {
    let code = "fun double(x: i32) -> i32 { x * 2 }";
    let ast = Parser::new(code).parse();
    assert!(ast.is_ok());
}

#[test]
fn test_parse_function_type() {
    let code = "fun apply(f: fn(i32) -> i32, x: i32) { f(x) }";
    let ast = Parser::new(code).parse();
    assert!(ast.is_ok());
}
```

### GREEN Phase - Implement Minimal Solution
```rust
// In parser.rs
fn parse_type(&mut self) -> Result<Type, ParseError> {
    match self.current_token() {
        Token::Identifier("i32") => Ok(Type::I32),
        Token::Identifier("String") => Ok(Type::String),
        Token::Identifier("bool") => Ok(Type::Bool),
        Token::Identifier("fn") => self.parse_function_type(),
        _ => Ok(Type::Inferred) // Default for now
    }
}

fn parse_param(&mut self) -> Result<Param, ParseError> {
    let name = self.expect_identifier()?;
    let ty = if self.match_token(Token::Colon) {
        self.parse_type()?
    } else {
        Type::Inferred
    };
    Ok(Param { name, ty })
}
```

### REFACTOR Phase - Ensure Quality
- Extract type parsing to separate module
- Ensure all functions have complexity < 10
- Add comprehensive error messages
- Document with examples

## Success Metrics

1. **Primary**: 100+ book examples now parse successfully
2. **Secondary**: 80% test coverage on type parsing code
3. **Tertiary**: All functions maintain complexity < 10
4. **Quaternary**: Zero regression in existing tests

## Risk Mitigation

- **Risk**: Breaking existing type inference
- **Mitigation**: Keep inference as fallback when no annotation

- **Risk**: Complex type syntax overwhelming parser
- **Mitigation**: Start with basic types, add complex types incrementally

## Quality Gates

- [ ] All new functions have complexity < 10 (PMAT enforced)
- [ ] 80% test coverage on new code
- [ ] All existing tests still pass
- [ ] 100+ book examples parse successfully
- [ ] Performance impact < 5% on parsing

## Example Code That Should Work After Fix

```ruchy
// Basic type annotations
fun add(x: i32, y: i32) -> i32 {
    x + y
}

// Function types
fun apply(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

// Complex types (future)
fun process(items: Vec<String>) -> Option<i32> {
    // ...
}
```

## Toyota Way Principles Applied

- **Jidoka**: Build quality in - types checked at parse time
- **Genchi Genbutsu**: Test with real book examples
- **Kaizen**: Incremental improvement - parse first, validate later
- **Respect for People**: Clear error messages help users
- **Long-term Philosophy**: Type system foundation for future