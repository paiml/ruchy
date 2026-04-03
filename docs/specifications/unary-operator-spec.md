# Unary Operator Specification

**Status**: Implemented (UnaryOp enum: Negate, Not, BitwiseNot, Reference, MutableReference, Deref in AST; parser + transpiler support)
**Date**: 2026-04-03 (status updated)

## Core Principle

Unary operators in Ruchy follow the **Principle of Least Surprise**: behavior matches Ruby/Elixir/Python programmer expectations exactly. No cognitive overhead for memory management. No hidden failure modes.

## Operator Set

### Prefix (Precedence: 8)

| Operator | UnaryOp Variant | Type Constraint | Semantics |
|----------|----------------|----------------|-----------|
| `-` | `Negate` | `Numeric a => a -> a` | Arithmetic negation |
| `!` | `Not` | `Bool -> Bool` | Logical NOT |
| `~` | `BitwiseNot` | `Integer a => a -> a` | Bitwise complement |
| `&` | `Reference` | `a -> &a` | Immutable reference |
| `&mut` | `MutableReference` | `a -> &mut a` | Mutable reference |
| `*` | `Deref` | `&a -> a` | Dereference |

### Separate Constructs (not in UnaryOp)

| Operator | AST Node | Type Constraint | Semantics |
|----------|----------|----------------|-----------|
| `await` | `ExprKind::Await` | `Future<a> -> a` | Async force |

### Not Yet Implemented

| Operator | Type Constraint | Semantics | Status |
|----------|----------------|-----------|--------|
| `?` (postfix) | `Option<a> -> Option<b>` | Optional chaining | Spec only |

## Design Notes

**Reference operators** (`*`, `&`, `&mut`): Exposed in syntax for Rust interop. The transpiler maps these directly to Rust reference/dereference operations.

**Force unwrap** (`!` suffix): Not planned. Use `.expect()` or `.unwrap_or()` for explicit failure modes.

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

## Implementation Status (as of v4.2.1, 2026-04-03)

| Feature | Status | Key File |
|---------|--------|----------|
| `-` (Negate) | Implemented | `ast.rs:989`, parser + transpiler |
| `!` (Not) | Implemented | `ast.rs:988`, parser + transpiler |
| `~` (BitwiseNot) | Implemented | `ast.rs:990`, parser + transpiler |
| `&` (Reference) | Implemented | `ast.rs:991`, parser + transpiler |
| `&mut` (MutableReference) | Implemented | `ast.rs:992`, PARSER-085 |
| `*` (Deref) | Implemented | `ast.rs:993`, parser + transpiler |
| `await` | Implemented | Separate `ExprKind::Await` node |
| `?` (optional chaining) | Not implemented | Spec only — no parser support |