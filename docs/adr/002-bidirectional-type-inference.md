# ADR-002: Bidirectional Type Inference

## Status

Accepted

## Date

2024-02-01

## Context

Ruchy aims to provide Python-like syntax with Rust-level type safety. This requires:
- Minimal type annotations (ergonomic like Python)
- Full type inference (no runtime type errors)
- Informative error messages when types don't match
- Support for generics and higher-kinded types

Pure Hindley-Milner inference is powerful but produces poor error messages. Full annotation (like Java pre-10) is ergonomically unacceptable.

## Decision

We implement **bidirectional type checking** combining:
- **Inference mode**: Bottom-up type synthesis from expressions
- **Checking mode**: Top-down type propagation from context

```rust
enum TypeCheck {
    Infer,                    // Synthesize type from expression
    Check(Type),              // Verify expression has expected type
}

fn typecheck(expr: &Expr, mode: TypeCheck) -> Result<Type, TypeError> {
    match mode {
        TypeCheck::Infer => synthesize(expr),
        TypeCheck::Check(expected) => {
            let actual = synthesize(expr)?;
            unify(expected, actual)
        }
    }
}
```

Key design choices:
1. Function parameters and return types require annotations
2. Local variables inferred from initialization
3. Generics use unification with occurs check
4. Error messages include both expected and actual types

## Consequences

### Positive

- **Error quality**: "Expected String, found i32" vs "type mismatch"
- **Annotation balance**: ~90% of types inferred, key points annotated
- **Predictability**: Type annotations flow downward predictably
- **Performance**: Linear-time type checking in practice

### Negative

- **Complexity**: Two modes adds implementation complexity
- **Higher-rank types**: Limited support compared to full HM

### Neutral

- IDE integration straightforward via type synthesis

## Alternatives Considered

### Full Hindley-Milner (Algorithm W)

Rejected:
- Poor error messages ("cannot unify 'a with 'b")
- Global inference makes errors non-local
- Let-polymorphism complexity not needed for Ruchy

### Fully Annotated (Java-style)

Rejected:
- Verbose, un-Pythonic syntax
- Users expect type inference in modern languages

### Local Type Inference (Scala 2 style)

Considered but extended:
- Good baseline but limited propagation
- Bidirectional adds checking mode for better errors

## References

- Pierce & Turner (2000). "Local Type Inference"
- Dunfield & Krishnaswami (2021). "Bidirectional Typing"
- TypeScript design documentation
