# ADR-005: Error Recovery in Parser

## Status

Accepted

## Date

2024-03-15

## Context

Users need meaningful error messages when their code has syntax errors. A parser that fails on the first error provides poor developer experience. We need a strategy for:

- Continuing parsing after encountering errors
- Collecting multiple errors in a single pass
- Providing accurate error locations
- Suggesting potential fixes

## Decision

We implement **panic-mode error recovery** with synchronization tokens.

Key mechanisms:
1. **Synchronization points**: Statement boundaries, block ends, semicolons
2. **Error accumulation**: Collect all errors, don't stop at first
3. **AST placeholders**: Insert `Error` nodes for unparseable regions
4. **Span tracking**: Every AST node tracks its source location

```rust
fn synchronize(&mut self) {
    self.advance();
    while !self.is_at_end() {
        if self.previous().kind == TokenKind::Semicolon {
            return;
        }
        match self.peek().kind {
            TokenKind::Fun | TokenKind::Let | TokenKind::If |
            TokenKind::While | TokenKind::Return => return,
            _ => self.advance(),
        }
    }
}
```

## Consequences

### Positive

- Multiple errors reported per compilation
- Better IDE integration (partial AST available)
- Improved user experience

### Negative

- Cascading errors possible (error begets error)
- Parser complexity increases
- Some error messages may be confusing

## References

- Crafting Interpreters, Chapter 16: "Parsing Expressions"
- GCC error recovery documentation
