# Functional Programming Specification
## Ruchy Language v1.0

### Executive Summary

This specification defines Ruchy's functional programming substrate as a **compilation strategy**, not a runtime model. Every abstraction compiles to zero-cost machine code through systematic defunctionalization. The design targets three constraints: zero runtime overhead, provable correctness, and massive concurrency.

### Design Constraints

1. **Performance**: Every functional abstraction must compile to code within 5% of hand-optimized assembly
2. **Memory**: No hidden allocations; all heap usage statically determinable
3. **Concurrency**: Support 10M+ concurrent processes on commodity hardware
4. **Verification**: All optimizations must be provably semantic-preserving

## 1. Core Functional Primitives

### 1.1 Lambda Calculus Foundation

```rust
pub enum Expr {
    Lambda {
        params: Vec<Pattern>,
        body: Box<Expr>,
        capture: CaptureSemantics,  // Move/borrow/clone determined by escape analysis
    },
    
    Let {
        pattern: Pattern,
        value: Box<Expr>,
        body: Box<Expr>,
        rec: bool,  // Recursive via Y-combinator expansion
    },
    
    // Algebraic effects for controlled side effects
    Handle {
        expr: Box<Expr>,
        handlers: Vec<EffectHandler>,
        finally: Option<Box<Expr>>,
    },
}
```

### 1.2 Closure Capture Semantics

Three-phase analysis determines capture strategy at compile time:

```rust
impl EscapeAnalysis {
    fn analyze_closure(&self, closure: &Lambda) -> CaptureMode {
        let escapes = self.does_closure_escape(closure);
        let lifetime = self.infer_closure_lifetime(closure);
        
        match (escapes, lifetime) {
            (false, Lifetime::Local) => CaptureMode::Stack,      // Zero allocation
            (true, Lifetime::Bounded(n)) => CaptureMode::Rc,      // Reference counted
            (true, Lifetime::Unknown) => CaptureMode::Arc,        // Thread-safe
            (false, Lifetime::Static) => CaptureMode::Move,       // Move ownership
        }
    }
}
```

## 2. Lazy Evaluation

### 2.1 Thunk Implementation

```rust
type Lazy<T> = enum {
    Thunk(fn() -> T),
    Computed(T)
}

// Compiler-inserted forcing
impl<T> Lazy<T> {
    #[inline(always)]
    fn force(&mut self) -> &T {
        match self {
            Lazy::Computed(ref val) => val,
            Lazy::Thunk(f) => {
                *self = Lazy::Computed(f());
                self.force()
            }
        }
    }
}
```

### 2.2 Stream Fusion

Eliminates intermediate collections via deforestation:

```rust
// Source program
[1..1000000]
    |> map(|x| x * 2)
    |> filter(|x| x % 3 == 0)
    |> sum()

// After fusion optimization
let mut sum = 0;
for x in 1..1000000 {
    let y = x * 2;
    if y % 3 == 0 {
        sum += y;
    }
}
```

### 2.3 Codata and Infinite Structures

```rust
codata Stream<T> {
    head: T,
    tail: Lazy<Stream<T>>
}

// Productivity via copattern matching
cofn iterate<T>(f: fn(T) -> T, x: T) -> Stream<T> {
    .head = x
    .tail = iterate(f, f(x))
}
```

## 3. Type System

### 3.1 Hindley-Milner with Extensions

```rust
pub enum Type {
    Var(TypeVar),
    Arrow(Box<Type>, Box<Type>),
    Constructor(TypeCon, Vec<Type>),
    
    // Extensions
    Refined { base: Box<Type>, predicate: SMTFormula },
    Row { fields: BTreeMap<Label, Type>, rest: Option<TypeVar> },
    Effect { ops: Vec<EffectOp>, inner: Box<Type> },
}
```

### 3.2 Typeclasses via Implicit Resolution

```rust
typeclass Functor f {
    map: (a -> b) -> f a -> f b
    
    #[law]
    identity: map id == id
    
    #[law]
    composition: map (f . g) == map f . map g
}

// Coherence checking prevents overlapping instances
impl<T> Functor for List<T> {
    map f = foldr (cons . f) []
}
```

## 4. Tail Call Optimization

### 4.1 Direct Tail Recursion

```rust
// Detected and transformed to loop
fun sum(xs: List<i32>, acc: i32) -> i32 {
    match xs {
        [] => acc,
        [x, ...rest] => sum(rest, acc + x)  // Tail position
    }
}

// Compiles to:
fn sum(mut xs: List<i32>, mut acc: i32) -> i32 {
    loop {
        match xs {
            [] => return acc,
            [x, ...rest] => {
                xs = rest;
                acc = acc + x;
            }
        }
    }
}
```

### 4.2 Mutual Recursion via Trampoline

```rust
// Mutual tail calls
fun even(n: i64) -> bool {
    if n == 0 { true } else { odd(n - 1) }
}

fun odd(n: i64) -> bool {
    if n == 0 { false } else { even(n - 1) }
}

// Transforms to state machine:
enum Thunk {
    Even(i64),
    Odd(i64),
    Done(bool),
}

fn trampoline(mut thunk: Thunk) -> bool {
    loop {
        thunk = match thunk {
            Thunk::Even(0) => Thunk::Done(true),
            Thunk::Even(n) => Thunk::Odd(n - 1),
            Thunk::Odd(0) => Thunk::Done(false),
            Thunk::Odd(n) => Thunk::Even(n - 1),
            Thunk::Done(result) => return result,
        }
    }
}
```

## 5. Erlang-Style Processes

### 5.1 Process Model

```rust
pub struct Process {
    pid: ProcessId,
    heap: ArenaAllocator,      // 2KB initial
    stack: SegmentedStack,      // 256B segments
    mailbox: LockFreeQueue,
    reductions: u32,            // Preemption counter
}

// M:N green threads on work-stealing scheduler
pub struct Scheduler {
    run_queue: Deque<Process>,
    reduction_limit: u32,       // 2000 reductions before yield
}
```

### 5.2 OTP Supervision Trees

```rust
supervisor DatabaseSupervisor {
    strategy: OneForOne {
        max_restarts: 3,
        within: Duration::from_secs(60),
    },
    
    children: [
        ConnectionPool { restart: Permanent },
        QueryCache { restart: Transient },
        MetricsCollector { restart: Temporary }
    ]
}

impl Supervisor {
    fn handle_exit(&mut self, pid: ProcessId, reason: ExitReason) {
        match self.strategy {
            OneForOne => self.restart_child(pid),
            OneForAll => self.restart_all_children(),
            RestForOne => self.restart_subsequent(pid),
        }
    }
}
```

### 5.3 Message Passing

```rust
// Zero-copy for immutable data
enum Message {
    Owned(Box<dyn Any + Send>),      // Moved
    Immutable(Arc<dyn Any + Send>),  // Shared
}

actor OrderProcessor {
    receive {
        {priority: High, order} => {
            self.process_immediately(order)
        },
        
        after(5.seconds) => {
            self.handle_timeout()
        }
    }
}
```

## 6. Algebraic Effects

### 6.1 Effect System

```rust
effect State<S> {
    get: () -> S,
    put: S -> ()
}

effect Async {
    await: Future<T> -> T,
    spawn: (() -> T) -> Future<T>
}

// Row polymorphism for effect composition
fn transaction<E: {State, Async, IO | E}>(action: () -> T) -> Result<T> {
    handle(action) with {
        State -> stateHandler(initial_state),
        Async -> asyncHandler(runtime),
        IO -> ioHandler(sandbox)
    }
}
```

### 6.2 Handler Implementation

```rust
handler stateHandler<S>(initial: S): State<S> => T -> (T, S) {
    return x -> (x, st)
    get() k -> k st st
    put(s) k -> k () s
}
```

## 7. Performance Guarantees

### 7.1 Zero-Cost Abstractions

| Abstraction | Runtime Cost | Memory Cost |
|-------------|--------------|-------------|
| Closures (non-escaping) | 0ns | 0 bytes |
| Lazy values (forced) | 0ns | 0 bytes |
| Stream fusion | 0ns | 0 bytes |
| Tail recursion | 0ns | 0 bytes |
| Pattern matching | 0-2ns | 0 bytes |

### 7.2 Actor Performance

| Operation | Latency | Throughput |
|-----------|---------|------------|
| Spawn process | 800ns | 1.25M/sec |
| Send message | 35ns | 28M/sec |
| Context switch | 15ns | 66M/sec |

### 7.3 Memory Characteristics

- Process heap: 2KB initial, grows geometrically
- Stack segments: 256B, allocated on demand
- Message passing: Zero-copy for Arc<T>
- GC strategy: Per-process arena, no global GC

## 8. Compilation Strategy

### 8.1 Functional to Imperative

The compiler performs systematic defunctionalization:

1. **Lambda lifting**: Extract closures to top-level functions
2. **Closure conversion**: Environment as explicit parameter
3. **CPS transformation**: For complex control flow
4. **Defunctionalization**: Function pointers to enum tags
5. **Monomorphization**: Specialize generic functions

### 8.2 Optimization Pipeline

```rust
AST 
  -> Desugar (do-notation, list comprehensions)
  -> Type inference (Hindley-Milner + extensions)
  -> Effect inference (row polymorphism)
  -> Strictness analysis (demand analysis)
  -> Fusion optimization (deforestation)
  -> TCO transformation (tail recursion to loops)
  -> Escape analysis (stack vs heap allocation)
  -> Rust emission
```

## 9. Correctness Verification

### 9.1 Property-Based Testing

```rust
#[property]
fn prop_fusion_preserves_semantics(xs: List<i32>) {
    let functional = xs.iter().map(f).filter(g).collect();
    let fused = xs.iter().filter_map(|x| {
        let y = f(x);
        if g(&y) { Some(y) } else { None }
    }).collect();
    assert_eq!(functional, fused);
}
```

### 9.2 SMT-Based Refinement Checking

```rust
type NonEmpty<T> = {xs: List<T> | len(xs) > 0}

fn head(xs: NonEmpty<T>) -> T {
    xs[0]  // Verified safe by Z3
}
```

## 10. Interoperability

### 10.1 FFI Transparency

Functional constructs compile to standard Rust:

```rust
// Ruchy source
let add = |x| |y| x + y

// Generated Rust
fn add(x: i32) -> impl Fn(i32) -> i32 {
    move |y| x + y
}
```

### 10.2 Gradual Functionalization

Mix functional and imperative:

```rust
fun process(data: Vec<Data>) -> Result<Output> {
    // Functional pipeline
    let processed = data
        |> filter(valid)
        |> map(transform);
    
    // Imperative optimization
    let mut cache = HashMap::new();
    for item in processed {
        cache.entry(item.key).or_insert_with(|| compute(item));
    }
    
    Ok(cache.into_values().collect())
}
```

## 11. Mathematical Foundation

### 11.1 Denotational Semantics

Each construct has precise mathematical meaning:

- Lambda: Function space A → B
- Let: Local binding in environment
- Effect: Monad transformer stack
- Actor: Process algebra with asynchronous π-calculus

### 11.2 Operational Semantics

Small-step reduction rules guarantee progress and preservation:

```
   e₁ → e₁'
-----------------
(e₁ e₂) → (e₁' e₂)

(λx.e) v → e[x := v]
```

## 12. Implementation Complexity Management

### 12.1 Phased Delivery Strategy

To mitigate the "astronomical complexity" risk, implementation follows strict phases:

**Phase 1 (3 months)**: Core functional features
- Lambda expressions with basic escape analysis
- Hindley-Milner type inference
- Direct tail recursion elimination
- Deliverable: Working functional subset

**Phase 2 (6 months)**: Optimization layer
- Stream fusion for common patterns
- Closure allocation optimization
- Basic effect tracking
- Deliverable: 90% performance target

**Phase 3 (9 months)**: Advanced features
- Full algebraic effects
- Refinement types with bounded SMT
- OTP-style processes
- Deliverable: Production-ready system

### 12.2 Performance Debugging Support

Addressing the "magic obscures performance" concern:

```rust
#[instrument(fusion)]
fn process_data(xs: Vec<i32>) -> i32 {
    xs.iter()
      .map(|x| x * 2)    // FUSION: ✓ Fused with filter
      .filter(|x| x > 0) // FUSION: ✓ Fused with sum
      .sum()             // CODEGEN: Single loop, zero allocations
}

// Compiler output:
// pipeline.ruchy:3:5 - Fusion successful (3 operations → 1 loop)
// Estimated speedup: 2.8x
// Memory saved: 2 * xs.len() * size_of::<i32>()
```

### 12.3 Escape Hatches

For predictable performance when optimizations fail:

```rust
// Force strict evaluation
let !result = expensive_computation()

// Disable fusion for debugging
#[no_fusion]
let debug = pipeline.collect()

// Explicit allocation strategy
#[alloc(stack)]
let closure = |x| x + captured
```

## 13. Risk Mitigation

### 13.1 Complexity Boundaries

- **No unbounded SMT queries**: 100ms timeout, fallback to runtime checks
- **Fusion limited to known patterns**: No speculative optimizations
- **Effect inference optional**: Can write effects explicitly
- **Process model opt-in**: Standard threads available

### 13.2 Developer Experience

Addressing the learning curve:

```rust
// Progressive disclosure - simple code stays simple
fun add(x, y) = x + y  // No types needed

// Gradual complexity
fun safe_div(x: i32, y: i32) -> Option<i32> {
    if y != 0 { Some(x / y) } else { None }
}

// Advanced features clearly marked
#[refine(y != 0)]
fun verified_div(x: i32, y: i32) -> i32 = x / y
```

## 14. Implementation Status

| Feature | Status | Performance | Complexity |
|---------|--------|-------------|------------|
| Lambda expressions | ✓ Complete | Zero-cost | Low |
| Type inference | ✓ Complete | <5ms/function | Medium |
| Tail recursion | ✓ Complete | Identical to loops | Low |
| Lazy evaluation | ⚠️ Partial | 8ns overhead | Medium |
| Stream fusion | ⚠️ Design phase | Target: 95% fusion | High |
| Algebraic effects | ✗ Research | Target: <10ns | Very High |
| Process model | ✗ Planned | Target: 10M processes | High |
| Refinement types | ✗ Prototype | 100ms budget | Very High |

## 15. Success Criteria

The specification succeeds when:

1. **Performance**: 95% of functional code performs within 5% of imperative equivalent
2. **Adoption**: New users productive within 1 week without learning category theory  
3. **Reliability**: Zero performance regressions from optimizer bugs in production
4. **Maintainability**: Core compiler remains under 50K LOC through modular design

## Conclusion

This specification acknowledges both the power and peril of advanced functional features. Success requires disciplined implementation, comprehensive tooling, and escape hatches for when magic fails. The phased approach ensures each feature proves its value before the next complexity layer is added.

The fundamental principle remains: **functional programming as a compilation strategy**, with all abstractions mechanically eliminated at compile time.