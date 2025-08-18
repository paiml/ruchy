# Ruchy REPL Testing & UX Specification v2.0

## Problem Statement

Current REPL fails basic reliability requirements. No recovery mechanism. No resource bounds. No testing infrastructure. This specification defines a production REPL with deterministic behavior and comprehensive testing.

## Core Architecture

### Minimal Defense Layers

```
┌─────────────────────────────────────┐
│  Bounded Evaluator (Resource Control)│
├─────────────────────────────────────┤
│  Transactional State (Checkpoints)   │
├─────────────────────────────────────┤
│  Test Harness (Verification)         │
└─────────────────────────────────────┘
```

## Resource-Bounded Evaluation

### Implementation

```rust
pub struct Evaluator {
    arena: ArenaAllocator,
    timeout: Duration,
    max_depth: usize,
}

impl Evaluator {
    pub fn eval(&mut self, input: &str) -> Result<Value> {
        // Pre-allocate fixed arena
        self.arena.reset();
        
        // Set hard timeout
        let deadline = Instant::now() + self.timeout;
        
        // Execute with bounds checking
        self.eval_bounded(input, deadline, 0)
    }
    
    fn eval_bounded(&mut self, expr: &str, deadline: Instant, depth: usize) -> Result<Value> {
        if Instant::now() > deadline {
            return Err(Error::Timeout);
        }
        if depth > self.max_depth {
            return Err(Error::StackOverflow);
        }
        // Actual evaluation with arena allocation
        self.eval_impl(expr, depth + 1)
    }
}
```

### Resource Limits

- Memory: 10MB arena, no heap allocation
- Time: 100ms hard limit via deadline
- Stack: 1000 frame maximum
- No I/O during evaluation

## Transactional State Machine

### State Definition

```rust
enum State {
    Ready(Env),
    Evaluating(Env, Checkpoint),
    Failed(Checkpoint),
}

struct Checkpoint {
    bindings: im::HashMap<String, Value>,  // Persistent data structure
    types: TypeEnv,
    pc: usize,  // Program counter for recovery
}

impl State {
    fn eval(self, input: &str) -> (State, Result<Value>) {
        match self {
            Ready(env) => {
                let checkpoint = env.checkpoint();
                match evaluate(&env, input) {
                    Ok(val) => {
                        let new_env = env.extend(val);
                        (Ready(new_env), Ok(val))
                    }
                    Err(e) => (Failed(checkpoint), Err(e))
                }
            }
            Failed(checkpoint) => {
                // Restore and retry
                (Ready(checkpoint.restore()), Err(Error::Recovered))
            }
            _ => (self, Err(Error::InvalidState))
        }
    }
}
```

### Checkpoint Strategy

Use persistent data structures (im crate) for O(1) checkpointing. No deep copying required.

## Testing Infrastructure

### Property Tests

```rust
#[quickcheck]
fn eval_preserves_type_safety(input: Expr) -> bool {
    let mut repl = Repl::new();
    match repl.eval(&input.to_string()) {
        Ok(val) => repl.typecheck(&val).is_ok(),
        Err(_) => true  // Errors are type-safe by definition
    }
}

#[quickcheck]
fn state_transitions_valid(ops: Vec<Operation>) -> bool {
    let mut state = State::Ready(Env::new());
    for op in ops {
        let prev_valid = state.is_valid();
        state = state.apply(op);
        if !prev_valid && state.is_valid() {
            return false;  // Invalid state became valid
        }
    }
    true
}
```

### Fuzz Testing

```rust
#[no_mangle]
pub extern "C" fn LLVMFuzzerTestOneInput(data: *const u8, size: usize) -> i32 {
    let input = unsafe { std::slice::from_raw_parts(data, size) };
    
    if let Ok(s) = std::str::from_utf8(input) {
        let mut repl = Repl::sandboxed();
        let _ = repl.eval(s);
        
        // Verify invariants hold
        assert!(repl.memory_used() < MAX_MEMORY);
        assert!(repl.bindings_valid());
    }
    
    0
}
```

### Differential Testing

```rust
#[test]
fn differential_test_against_reference() {
    let cases = include_str!("../test_cases.txt");
    
    for case in cases.lines() {
        let mut production = Repl::new();
        let mut reference = ReferenceImpl::new();
        
        let prod_result = production.eval(case);
        let ref_result = reference.eval(case);
        
        assert_eq!(
            prod_result.is_ok(), 
            ref_result.is_ok(),
            "Divergence on: {}", case
        );
    }
}
```

## User Experience

### Error Recovery

```
> let x = 
Error: Unexpected EOF at line 1:8
      │ let x = 
      │        ↑ expected expression

[Enter] Continue with empty binding
[Tab]   Show completion options
[Esc]   Discard line
```

### Progressive Modes

#### Standard Mode
```
> 1 + 2
3
```

#### Test Mode (Activated with #[test])
```
> #[test]
> assert 1 + 1 == 2
✓ Pass

> table_test!(
    (1, 2, 3),
    (0, 0, 0)
  ) |a, b, c| a + b == c
✓ 2/2 Pass
```

#### Debug Mode (Activated with #[debug])
```
> #[debug]
> let x = compute(42)
┌─ Trace ────────┐
│ parse:   0.1ms │
│ type:    0.0ms │
│ eval:    2.3ms │
│ alloc:   24B   │
└────────────────┘
x: Int = 84
```

### Performance Feedback

```
> fibonacci(40)
⚠ 847ms (timeout: 1000ms)
102334155
```

### Inline Tests

```rust
> fun add(a: Int, b: Int) -> Int {
    a + b
  } where tests {
    add(1, 2) == 3,
    add(0, 0) == 0
  }
✓ Function defined (2 tests pass)
```

## Advanced Features from Other Systems

### 1. Condition/Restart System (Common Lisp)

```rust
> process("invalid")
Error: Parse failed at "invalid"

Restarts:
  1. use_default() -> continue with default value
  2. retry_with(value) -> retry with new value
  3. abort() -> cancel operation

> 1
Continuing with default...
```

### 2. Type Providers (F#)

```rust
> #[provider(csv = "data.csv")]
Type 'CsvRow' generated with fields: name, age, salary

> let data = csv::load()
> data[0].name
"Alice"
```

### 3. Rich Introspection (PostgreSQL)

```rust
> :env           # List all bindings
> :type expr     # Show type of expression
> :ast expr      # Show AST
> :ir expr       # Show intermediate representation
> :asm expr      # Show generated assembly
```

### 4. Object Inspector (Smalltalk)

```rust
> :inspect value
┌─ Inspector ────────────────┐
│ Type: HashMap<String, Int> │
│ Size: 1024 entries         │
│ Memory: 32KB               │
│                            │
│ [Enter] Browse entries     │
│ [S] Statistics             │
│ [M] Memory layout          │
└────────────────────────────┘
```

### 5. Dynamic Manipulation (Wolfram)

```rust
> @manipulate n:[1..100] plot(|x| sin(n * x), 0..2π)
[Interactive slider appears in supporting frontend]
```

## Testing Requirements

### Coverage Targets
- Line coverage: ≥95%
- Branch coverage: ≥90%  
- Mutation score: ≥85%

### Performance Targets
- Simple eval: <1ms
- With 1000 bindings: <5ms
- Checkpoint: <100μs (using persistent structures)
- Recovery: <1ms

### Reliability Targets
- Crash rate: <0.001%
- Recovery success: >99.9%
- Memory stability: 0 bytes leaked over 24h

### Continuous Validation

```rust
#[test]
#[ignore] // Run only in CI
fn stability_test() {
    let mut repl = Repl::new();
    let deadline = Instant::now() + Duration::from_hours(24);
    
    while Instant::now() < deadline {
        for _ in 0..1000 {
            let input = generate_random_input();
            let _ = repl.eval(&input);
            
            assert!(repl.memory_used() < MAX_MEMORY);
            assert!(repl.can_accept_input());
        }
    }
}
```

## Implementation Strategy

### Phase 1: Core (Week 1)
- Bounded evaluator
- Basic checkpointing
- Minimal test suite

### Phase 2: Reliability (Week 2)
- State machine formalization
- Recovery mechanisms
- Property tests

### Phase 3: Testing (Week 3)
- Fuzzing infrastructure
- Differential testing
- Performance benchmarks

### Phase 4: UX (Week 4)
- Error recovery UI
- Progressive modes
- Introspection commands

## Trade-offs

### Accepted
- Checkpoint overhead (~100μs per eval)
- Fixed memory arena (no unbounded allocation)
- Synchronous evaluation only

### Rejected
- Perfect sandboxing (use OS process isolation if needed)
- Zero-overhead abstraction (safety > performance)
- Arbitrary code timeout (use deadline approach)

## Success Metrics

1. 1M random inputs without crash
2. 99.9% recovery rate from induced failures
3. <10ms response for standard operations
4. Zero memory growth over 24h operation
5. 95% code coverage with passing tests

## Non-Goals

- Distributed REPL
- GUI interface  
- Natural language input
- Concurrent evaluation
- JIT compilation (use ahead-of-time)

## References

- SQLite: Comprehensive testing methodology
- Erlang/OTP: Supervisor patterns
- Common Lisp: Condition system
- F# Interactive: Type providers
- PostgreSQL: Meta-commands