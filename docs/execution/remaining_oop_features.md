# Remaining OOP Features for 100% Coverage

## Current Status: 43.9% (29/66 tests passing)
- Structs: 20/24 (83.3%) - Fixed pub field visibility and pattern guards
- Classes: 5/25 (20%)
- Actors: 4/17 (23.5%)

## Critical Blockers for Remaining Tests

### 1. Type System Enhancements (Blocks 15+ tests)
- **Lifetime parameters** (`'a`, `'b`): Required for reference types in structs
- **Const generics** (`const N: usize`): Required for fixed-size arrays
- **Trait bounds** (`T: Display`): Required for generic constraints
- **Associated types**: Required for trait implementations

### 2. Visibility System (Blocks 5+ tests)
- **pub(crate)**: Crate-level visibility
- **protected**: Inheritance-based visibility
- **private**: Explicit private visibility
- Currently only `pub` works at struct level

### 3. Class Properties (Blocks 8+ tests)
- **Property syntax**: `property name: type { get => ..., set => ... }`
- **Getters/Setters**: Automatic accessor generation
- **Readonly properties**: Properties with only getters
- **Property validation**: Set-time validation logic

### 4. Actor Runtime (Blocks 22 tests)
- **Message passing**: Send/receive primitives
- **Actor spawning**: Runtime actor creation
- **Mailbox system**: Message queue per actor
- **Supervision**: Actor lifecycle management
- **Actor references**: Type-safe actor handles

### 5. Parser Bugs (Blocks 5+ tests)
- **Pattern guards with multiple fields**: Parser error "Expected RightBrace, found Let"
- **"exp" parameter in for loops**: Specific parsing failure
- **pub fields in structs**: Parser expects field name after pub

### 6. Advanced Class Features (Blocks 10+ tests)
- **Class constants**: `const NAME: type = value`
- **Abstract methods**: Method signatures without implementation
- **Interface implementation**: Explicit interface conformance
- **Method overriding**: Explicit override annotations
- **Decorators**: Attribute-based metaprogramming

## Implementation Effort Estimates

### Low Effort (Could add 5-10% coverage)
- Fix pub field parsing
- Fix pattern guard parsing bug
- Add basic const support in classes

### Medium Effort (Could add 10-20% coverage)
- Implement property syntax and basic getters/setters
- Add visibility modifiers (protected, private)
- Fix remaining parser bugs

### High Effort (Required for 100% coverage)
- Full lifetime system implementation
- Complete actor runtime with message passing
- Const generics and trait bounds
- Full property system with validation

## Recommended Path Forward

1. **Fix Parser Bugs** (Quick wins)
   - Pattern guard issue
   - pub field parsing
   - exp parameter bug

2. **Basic Property Support** (Medium complexity)
   - Parse property syntax
   - Generate getter/setter methods
   - Handle readonly properties

3. **Visibility System** (Medium complexity)
   - Add protected/private keywords
   - Implement visibility checking
   - Support pub(crate)

4. **Type System** (High complexity)
   - Lifetime parameters
   - Trait bounds
   - Const generics

5. **Actor System** (Very high complexity)
   - Design message passing protocol
   - Implement actor runtime
   - Add supervision tree

## Technical Debt to Address
- Error messages are often misleading (e.g., "found Let" when no let exists)
- TokenStream formatting adds spaces to attributes
- Some keywords cause context-dependent parsing issues

## Conclusion
Reaching 100% test coverage would require implementing several major language features. The current 32.9% coverage represents the "easy" features. The remaining 67.1% requires substantial parser, type system, and runtime work.

Priority should be given to fixing parser bugs and implementing the most commonly needed features (properties, better visibility) before tackling complex features like actors and lifetimes.