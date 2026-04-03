# Sub-spec: Classes & OOP — MCP Introspection, Provability, Disassembly & Metaprogramming

**Parent:** [ruchy_classes_spec.md](../ruchy_classes_spec.md) Sections 14-17

---

## 14. MCP-Native Introspection

### 14.1 AST Inspection

Every Ruchy construct exposes its AST via MCP tools:

```rust
#[mcp::tool("inspect_ast")]
fun show_ast(target: TypePath) -> AstNode {
    // Returns serialized AST for any type/function
    compiler::ast_for(target)
}

// Usage via MCP
> mcp.inspect_ast("Counter")
{
  "node": "Actor",
  "name": "Counter",
  "fields": [{"name": "count", "type": "i32", "default": 0}],
  "handlers": [
    {"name": "increment", "params": [], "body": {...}},
    {"name": "get", "returns": "i32", "body": {...}}
  ]
}
```

### 14.2 Type Graph Visualization

```rust
#[mcp::tool("type_graph")]  
fun show_relationships(root: TypePath) -> Graph {
    // Generates DOT/Mermaid graph of type relationships
    TypeGraph::from(root)
        .with_traits()
        .with_impls()
        .with_dependencies()
        .render()
}

// Output format
digraph {
    Counter -> CounterMessage [label="generates"];
    Counter -> mpsc::Sender [label="contains"];
    Counter -> Supervision [label="implements"];
}
```

## 15. Provability via PMAT

### 15.1 Property Specifications

```rust
#[pmat::spec]
impl Counter {
    // Invariant: count never negative
    #[invariant]
    fun count_non_negative(&self) -> bool {
        self.count >= 0
    }
    
    // Property: increment increases by exactly 1
    #[property]
    fun increment_adds_one(&self, initial: i32) -> bool {
        let before = self.get();
        self.increment();
        self.get() == before + 1
    }
}
```

### 15.2 SMT-Backed Verification

```rust
actor BankAccount {
    balance: i64 = 0,
    
    #[requires(amount > 0)]
    #[ensures(self.balance == old(self.balance) + amount)]
    receive deposit(amount: i64) {
        self.balance += amount;
    }
    
    #[requires(amount > 0 && amount <= self.balance)]
    #[ensures(self.balance == old(self.balance) - amount)]
    receive withdraw(amount: i64) -> Result<()> {
        self.balance -= amount;
        Ok(())
    }
}

// Generates Z3 constraints for verification
```

## 16. Disassembly Integration

### 16.1 Assembly Inspection

```rust
#[mcp::tool("show_assembly")]
fun disassemble(target: FunctionPath) -> Assembly {
    compiler::emit_asm(target, OptLevel::Release)
}

// Example output
> mcp.disassemble("Point::distance")
Point::distance:
    mulss  xmm0, xmm0      ; x²
    mulss  xmm1, xmm1      ; y²
    addss  xmm0, xmm1      ; x² + y²
    sqrtss xmm0, xmm0      ; √(x² + y²)
    ret
```

### 16.2 Optimization Verification

```rust
#[test]
#[pmat::verify_zero_cost]
fun test_actor_overhead() {
    // Verify actor message passing compiles to direct call
    let counter = Counter::spawn();
    counter.increment();  
    
    // PMAT verifies this produces identical assembly to:
    // counter_state.count += 1;
}
```

## 17. Metaprogramming

### 17.1 Compile-Time Reflection

```rust
// Type-level computation
meta fun generate_builder<T: Struct>() -> impl Builder<T> {
    let fields = T::fields();
    
    struct ${T}Builder {
        ${for field in fields {
            ${field.name}: Option<${field.type}>,
        }}
    }
    
    impl ${T}Builder {
        ${for field in fields {
            fun with_${field.name}(mut self, val: ${field.type}) -> Self {
                self.${field.name} = Some(val);
                self
            }
        }}
    }
}

// Usage
#[derive_builder]
struct Config { host: String, port: u16 }
```

### 17.2 Hygenic Macros

```rust
// Ruchy macros are hygenic and type-aware
macro define_enum_matcher($enum_type) {
    impl $enum_type {
        fun match_all<R>(&self, ${arms}) -> R {
            match self {
                ${for variant in $enum_type::variants() {
                    Self::${variant} => ${arms[variant]},
                }}
            }
        }
    }
}

// Expansion happens at Rust AST level, not text
```

### 17.3 Const Evaluation

```rust
// Compile-time execution
const TABLE: DataFrame = {
    let data = include_csv!("data.csv");
    data.filter(|row| row.valid)
        .select(["id", "name"])
        .collect()
};

// Const functions for metaprogramming
const fun type_size<T>() -> usize {
    std::mem::size_of::<T>()
}

// Used in type-level assertions
#[static_assert(type_size<Handle>() <= 8)]
struct Handle { ... }
```

### 17.4 Code Generation Hooks

```rust
// Pre-transpilation transformers
#[transformer(phase = "pre_typecheck")]
fun optimize_dataframe_ops(ast: &mut Ast) {
    // Rewrite df.filter().map().collect() to single pass
    ast.visit_mut(|node| {
        if let Chain(ops) = node {
            fuse_operations(ops);
        }
    });
}

// Post-transpilation Rust manipulation
#[transformer(phase = "post_transpile")]
fun add_telemetry(rust_ast: &mut syn::File) {
    // Inject performance counters
    for func in rust_ast.items.iter_mut() {
        inject_timer(func);
    }
}
```

