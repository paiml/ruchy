# REPL Mutability Concepts Specification
## Semantic Consistency Between Interactive and Compiled Modes

### Executive Summary

This specification enforces identical mutability semantics across REPL and compiled execution contexts. The design eliminates semantic drift while maintaining zero-cost abstractions and data science ergonomics.

## Core Invariant

**Any valid REPL expression must produce identical semantics when transpiled to Rust.**

```rust
// REPL session
>>> let x = 42
>>> x = 43        // ERROR: cannot assign to immutable binding
>>> let x = 43    // OK: shadowing creates new binding
```

```rust
// Transpiled Rust (identical semantics)
let x = 42;
x = 43;           // ERROR: cannot assign twice to immutable variable
let x = 43;       // OK: shadowing
```

## Mutability Model

### Binding Types

| Declaration | Reassignable | Shadowable | Memory Strategy |
|------------|--------------|------------|-----------------|
| `let x = v` | No | Yes | Stack/Register |
| `var x = v` | Yes | Yes | Stack with mut flag |

Note: `const` is omitted as a language feature. Compile-time constants are an optimization detail, not a semantic distinction requiring user annotation.

### REPL State Machine

```rust
pub struct ReplEnvironment {
    bindings: HashMap<Ident, Binding>,
    generation: u64,  // Monotonic counter for shadowing
}

pub struct Binding {
    value: Value,
    mutable: bool,
    generation: u64,
    location: SourceLoc,  // For error reporting
}
```

### Assignment Resolution

```rust
impl ReplEnvironment {
    pub fn resolve_assignment(&mut self, ident: &str, value: Value) -> Result<()> {
        match self.bindings.get(ident) {
            Some(binding) if binding.mutable => {
                self.bindings.get_mut(ident).unwrap().value = value;
                Ok(())
            }
            Some(binding) => {
                Err(Error::ImmutableAssignment {
                    ident: ident.to_string(),
                    location: binding.location,
                })
            }
            None => Err(Error::UndefinedVariable(ident.to_string()))
        }
    }
    
    pub fn create_binding(&mut self, mutable: bool, ident: &str, value: Value) -> Result<()> {
        self.generation += 1;
        self.bindings.insert(ident.to_string(), Binding {
            value,
            mutable,
            generation: self.generation,
            location: self.current_location(),
        });
        Ok(())
    }
}
```

## Data Science Workflows

### Functional Transformation Pattern

DataFrames use immutable transformation chains:

```rust
>>> let df = read_csv("data.csv")
>>> let df = df.filter(col("x") > 10)  // Shadow for pipeline
>>> let df = df.select(["x", "y"])     // Progressive refinement
```

### Explicit Mutation Pattern

When accumulation is required:

```rust
>>> var results = Vec::new()
>>> for model in models {
...     results.push(model.evaluate())
... }
```

### Method Naming Convention

- Immutable methods: `sort()`, `filter()`, `map()`
- Mutating methods: `sort_in_place()`, `push()`, `clear()`

## Error Messages

### Immutable Assignment

```
Error: Cannot assign to immutable binding 'x'
  --> REPL:2:1
  |
1 | let x = 42
  |     - binding declared here as immutable
2 | x = 43
  | ^^^^^^ cannot assign twice to immutable variable
  |
  = help: consider making this binding mutable: `var x = 42`
  = help: or shadow the binding: `let x = 43`
```

### Undefined Variable

```
Error: Cannot assign to undefined variable 'y'
  --> REPL:1:1
  |
1 | y = 10
  | ^ variable not found in scope
  |
  = help: declare the variable first: `let y = 10` or `var y = 10`
```

## Implementation Strategy

### Phase 1: Parser (Week 1)
- Reject bare assignments without `let`/`var`
- Implement error recovery with suggested fixes
- Parse both binding forms correctly

### Phase 2: State Tracking (Week 1)
- Implement `Binding` with mutability flag
- Track generation counter for shadowing
- Preserve source locations

### Phase 3: Transpiler (Week 2)
- Map `let` → `let`
- Map `var` → `let mut`
- Preserve shadowing semantics

### Phase 4: Optimization (Week 2)
- Inline immutable scalar values
- Dead binding elimination
- Copy-on-write for collections

## Test Requirements

```rust
#[test]
fn immutable_binding_prevents_reassignment() {
    let mut repl = Repl::new();
    repl.eval("let x = 1").unwrap();
    assert!(repl.eval("x = 2").is_err());
    assert_eq!(repl.eval("x").unwrap(), Value::Int(1));
}

#[test]
fn mutable_binding_allows_reassignment() {
    let mut repl = Repl::new();
    repl.eval("var x = 1").unwrap();
    repl.eval("x = 2").unwrap();
    assert_eq!(repl.eval("x").unwrap(), Value::Int(2));
}

#[test]
fn shadowing_creates_new_binding() {
    let mut repl = Repl::new();
    repl.eval("let x = 1").unwrap();
    repl.eval("let x = 2").unwrap();
    assert_eq!(repl.eval("x").unwrap(), Value::Int(2));
}
```

### Property Tests

```rust
#[quickcheck]
fn repl_transpiler_semantic_equivalence(program: ValidProgram) {
    let repl_result = evaluate_in_repl(&program);
    let rust_result = transpile_and_run(&program);
    assert_eq!(repl_result, rust_result);
}
```

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Binding lookup | O(1) | HashMap |
| Shadow creation | O(1) | Insert with generation |
| Mutation check | O(1) | Single bool check |
| Scope management | O(1) | Not needed for flat REPL |

## Shadowing Semantics

### Design Rationale

Shadowing enables functional transformation pipelines without namespace pollution:

```rust
// Without shadowing (awkward)
let df1 = read_csv("data.csv")
let df2 = df1.filter(x > 0)
let df3 = df2.normalize()
let df4 = df3.sort("timestamp")

// With shadowing (elegant)
let df = read_csv("data.csv")
let df = df.filter(x > 0)
let df = df.normalize()
let df = df.sort("timestamp")
```

### Shadow Tracking

Each binding carries a generation number. Newer generations shadow older ones:

```rust
>>> let x = 1    // generation: 1
>>> let x = 2    // generation: 2, shadows previous
>>> x            // resolves to generation: 2
2
```

## Comparison with Alternatives

| Language | Default | Mutable | Shadowing | REPL=Compiled |
|----------|---------|---------|-----------|---------------|
| **Ruchy** | Immutable | `var` | Yes | Yes |
| Python | Mutable | N/A | Yes | Yes |
| Rust | Immutable | `mut` | Yes | N/A |
| Julia | Mutable | N/A | No | No* |
| Scala | Immutable | `var` | Yes | Yes |

*Julia REPL has different scoping rules than scripts

## Zero-Cost Proof

Mutability is purely compile-time metadata:

```rust
// Ruchy
let x = compute()
var y = compute()
y = update(y)

// Generated Rust
let x = compute();
let mut y = compute();
y = update(y);

// Assembly (identical)
call compute
mov  rax, rdi
call compute  
mov  rbx, rdi
call update
mov  rbx, rax
```

## Design Trade-offs

### Accepted Costs
1. **Verbosity for mutation**: Explicit `var` required
2. **Shadowing complexity**: Users must track which binding is active
3. **No bare assignments**: Every binding needs `let`/`var`

### Rejected Alternatives
1. **Mutable by default**: Sacrifices safety for convenience
2. **No shadowing**: Forces awkward variable naming
3. **Different REPL semantics**: Creates deployment surprises
4. **Three binding types**: `const` adds complexity without clear value

## Conclusion

This design delivers:
- **Semantic consistency** across all execution contexts
- **Zero runtime cost** through compile-time tracking  
- **Safety by default** with explicit mutation
- **Ergonomic workflows** via shadowing
- **Clear error messages** with actionable fixes

The specification eliminates an entire class of semantic drift bugs while maintaining the iterative nature essential for data science.