# ADR-001: Pratt Parsing for Expression Handling

## Status

Accepted

## Date

2024-01-15

## Context

Ruchy needs a robust expression parser that can handle:
- Operator precedence (arithmetic, logical, comparison)
- Associativity (left, right, non-associative)
- Prefix and postfix operators
- Method chaining and field access
- Error recovery during parsing

Traditional recursive descent parsers require separate functions for each precedence level, leading to deeply nested call stacks and verbose code.

## Decision

We adopt **Pratt parsing** (also known as Top-Down Operator Precedence parsing) for all expression handling.

Key implementation details:
- Binding power pairs (left, right) determine precedence and associativity
- Single `parse_expression(min_bp)` function handles all precedence levels
- Prefix, infix, and postfix operators handled uniformly
- Error recovery via panic-mode with synchronization tokens

```rust
fn parse_expression(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
    let mut lhs = self.parse_prefix()?;
    while let Some((l_bp, r_bp)) = self.infix_binding_power() {
        if l_bp < min_bp { break; }
        lhs = self.parse_infix(lhs, r_bp)?;
    }
    Ok(lhs)
}
```

## Consequences

### Positive

- **Extensibility**: Adding new operators requires only defining binding power
- **Performance**: O(n) parsing with minimal stack depth
- **Maintainability**: Single unified parsing function vs N precedence functions
- **Correctness**: Binding power pairs mathematically guarantee correct precedence

### Negative

- **Learning curve**: Pratt parsing is less intuitive than recursive descent
- **Debugging**: Call stack doesn't directly reflect grammar structure

### Neutral

- Parser generator tools (lalrpop, pest) remain viable alternatives

## Alternatives Considered

### Recursive Descent with Precedence Climbing

Rejected: Requires separate functions per precedence level, leading to 15+ functions for Ruchy's operator set.

### Parser Generators (lalrpop, pest)

Rejected:
- External dependency increases build complexity
- Grammar file separate from code reduces locality
- Error messages harder to customize

### Shunting-Yard Algorithm

Rejected: Requires two-pass approach and explicit stack management, more complex than Pratt parsing.

## References

- Pratt, V. R. (1973). "Top Down Operator Precedence"
- Matklad's "Simple but Powerful Pratt Parsing" blog post
- Rust-analyzer parser implementation
