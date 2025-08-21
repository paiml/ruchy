# Unary Operator Specification

## Core Principle

Unary operators in Ruchy follow the **Principle of Least Surprise**: behavior matches Ruby/Elixir/Python programmer expectations exactly. No cognitive overhead for memory management. No hidden failure modes.

## Operator Set

### Prefix (Precedence: 8)

| Operator | Type Constraint | Semantics |
|----------|----------------|-----------|
| `-` | `Numeric a => a -> a` | Arithmetic negation |
| `!` | `Bool -> Bool` | Logical NOT |
| `await` | `Future<a> -> a` | Async force |

### Postfix (Precedence: 10)

| Operator | Type Constraint | Semantics |
|----------|----------------|-----------|
| `?` | `Option<a> -> Option<b>` | Optional chaining |

## Rejected Operators

**Memory operators** (`*`, `&`): Handled by escape analysis. Never surface syntax.

**Force unwrap** (`!`): Use `.expect()` or `.unwrap_or()`. Explicit failure modes only.

**Bitwise NOT** (`~`): Use `.bit_not()` method. Rare in scripting context.

## Grammar

```ebnf
unary   = ('-' | '!' | 'await')? postfix
postfix = primary (call | index | field | '?')*
```

## Operator Interaction

`await` binds tighter than `?`:
```rust
await get_optional_future()?
// Parses as: (await get_optional_future())?
// Type: Future<Option<T>> → Option<T> → T
```

## Transpilation Rules

### Negation
```rust
-x → -x  // Direct mapping
```

### Logical NOT
```rust
!expr → !expr  // Direct mapping
```

### Await
```rust
await expr → expr.await
// But often implicit:
fetch(url) → fetch(url).await  // In async context
```

### Optional Chaining
```rust
x?.field → x.and_then(|v| v.field)
x?.method() → x.and_then(|v| v.method())

// Compiler optimizes nested chains:
a?.b?.c → match a {
    Some(a) => match a.b {
        Some(b) => b.c,
        None => None
    },
    None => None
}
```

## Type Inference

Negation preserves numeric type:
```
Γ ⊢ e : τ where τ ∈ {i32, i64, f32, f64, ...}
─────────────────────────────────────────────
Γ ⊢ -e : τ
```

Optional chaining lifts into Option:
```
Γ ⊢ e : Option<τ>    Γ ⊢ τ.f : σ
───────────────────────────────────
Γ ⊢ e?.f : Option<σ>
```

## Error Quality

```
Error: Cannot negate string
  │
3 │ let x = -"hello"
  │         ^^^^^^^^ String is not Numeric
  │
  = help: convert first: -"123".parse::<i32>()?
```

No type theory in error messages. Show the fix.

## Implementation Priority

1. `-`, `!` - Day one. Non-negotiable.
2. `?` - Before v0.2. Eliminates None-checking boilerplate.
3. `await` - With async runtime. Can be implicit initially.

## Design Invariants

- Zero runtime cost after transpilation
- No operator performs direct allocation
- All operators have single, obvious semantics
- Error messages show working code, not theory
- Performance costs visible via LSP markers (implicit await)