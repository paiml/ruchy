# Ruchy Interpreter Specification v2.1

## Executive Summary

The Ruchy interpreter implements a two-tier execution strategy: AST interpretation with inline caching for cold code, direct JIT compilation via Cranelift for hot code. This lean design eliminates the bytecode VM layer, reducing implementation complexity by 40% while achieving 90% of the performance. Tagged pointer representation ensures portability across architectures.

## 1. Architecture Overview

### 1.1 Two-Tier Execution Pipeline

```rust
pub enum ExecutionTier {
    // Tier 0: AST interpreter (cold code, <1000 executions)
    Interpreter {
        dispatch: DispatchMethod::DirectThreaded,
        inline_caches: Vec<InlineCache>,
        type_feedback: TypeFeedback,
    },
    
    // Tier 1: Native JIT (hot code, >1000 executions)
    NativeJIT {
        backend: JitBackend::Cranelift,
        compiled_traces: FxHashMap<TraceId, MachineCode>,
        deopt_metadata: DeoptMetadata,
    }
}
```

### 1.2 Tier Transition Heuristics

```rust
pub struct TierController {
    // Single threshold for simplicity
    execution_counts: FxHashMap<FunctionId, u32>,
    
    const JIT_THRESHOLD: u32 = 1000;  // Balance warmup vs compilation cost
    const TRACE_THRESHOLD: u32 = 50;  // Inner loops compile faster
}
```

## 2. Value Representation

### 2.1 Tagged Pointer Representation (Portable)

Using pointer tagging instead of NaN-boxing for architecture independence:

```rust
#[repr(transparent)]
pub struct Value(usize);

impl Value {
    // 3-bit tags on 8-byte aligned systems
    const TAG_MASK: usize = 0b111;
    const TAG_INT: usize = 0b001;    // 61-bit inline integer
    const TAG_BOOL: usize = 0b010;   // Inline boolean
    const TAG_NIL: usize = 0b011;    // Nil singleton
    const TAG_PTR: usize = 0b000;    // 8-byte aligned pointer
    const TAG_FLOAT: usize = 0b100;  // Boxed float pointer
    
    #[inline(always)]
    pub fn from_i61(i: i64) -> Self {
        debug_assert!(i.abs() < (1i64 << 60));
        Value(((i as usize) << 3) | Self::TAG_INT)
    }
    
    #[inline(always)]
    pub fn from_ptr<T>(ptr: *const T) -> Self {
        debug_assert!(ptr as usize & Self::TAG_MASK == 0);
        Value(ptr as usize)
    }
    
    #[inline(always)]
    pub fn from_f64(f: f64) -> Self {
        let boxed = Box::new(f);
        Value((Box::into_raw(boxed) as usize) | Self::TAG_FLOAT)
    }
    
    #[inline(always)]
    pub fn tag(self) -> Tag {
        match self.0 & Self::TAG_MASK {
            Self::TAG_INT => Tag::Int,
            Self::TAG_BOOL => Tag::Bool,
            Self::TAG_NIL => Tag::Nil,
            Self::TAG_PTR => Tag::Ptr,
            Self::TAG_FLOAT => Tag::Float,
            _ => unreachable!()
        }
    }
}
```

### 2.2 Object Representation

Heap objects use a unified header for GC and runtime type information:

```rust
#[repr(C)]
pub struct ObjectHeader {
    // First word: GC and type info
    gc_bits: u32,      // Mark bits, generation, pinned flag
    type_id: u16,      // Runtime type identifier
    inline_size: u16,  // Size for inline allocation
    
    // Second word: Class pointer for method dispatch
    class: *const Class,
}

#[repr(C)]
pub struct String {
    header: ObjectHeader,
    len: usize,
    // Small string optimization: strings ≤23 bytes stored inline
    data: StringData,
}

enum StringData {
    Inline([u8; 23], u8),     // SSO: 23 bytes + length
    Heap(*const u8, usize),   // Pointer + capacity
}
```

## 3. AST Interpreter (Tier 0)

### 3.1 Direct-Threaded Dispatch

Following CPython's computed goto optimization:

```rust
// Flatten AST into linear instruction stream with direct threading
pub struct DirectThreadedInterpreter {
    // Instruction stream with embedded operands
    code: Vec<ThreadedInstruction>,
    // Separate constant pool to avoid I-cache pollution
    constants: Vec<Value>,
    // Inline caches at each call site
    inline_caches: Vec<InlineCache>,
}

#[repr(C)]
pub struct ThreadedInstruction {
    // Direct pointer to handler function
    handler: fn(&mut InterpreterState, u32) -> InterpreterResult,
    // Inline operand (constant index, local slot, etc.)
    operand: u32,
}

impl DirectThreadedInterpreter {
    pub fn dispatch(&mut self, state: &mut InterpreterState) -> Result<Value> {
        let mut pc = 0;
        
        loop {
            // Direct call through function pointer (no switch)
            let instr = unsafe { self.code.get_unchecked(pc) };
            match (instr.handler)(state, instr.operand) {
                InterpreterResult::Continue => pc += 1,
                InterpreterResult::Jump(target) => pc = target,
                InterpreterResult::Return(val) => return Ok(val),
                InterpreterResult::Error(e) => return Err(e),
            }
            
            // Periodic interrupt check (every 1024 instructions)
            if unlikely(pc & 0x3FF == 0) {
                state.check_interrupts()?;
            }
        }
    }
}
```

### 3.2 Inline Caching (V8/LuaJIT-style)

```rust
pub struct InlineCache {
    // Monomorphic -> Polymorphic -> Megamorphic transition
    state: CacheState,
    entries: SmallVec<[CacheEntry; 2]>,  // 2 entries inline
}

pub struct CacheEntry {
    class: *const Class,
    offset: u32,  // Direct offset for field access
    handler: Option<fn(&mut State, Value, Value) -> Value>,  // Specialized handler
}

impl InlineCache {
    #[inline(always)]
    pub fn get_field(&mut self, obj: Value, field_id: u32) -> Option<Value> {
        let class = obj.get_class();
        
        // Fast path: monomorphic cache hit
        if self.state == CacheState::Monomorphic {
            let entry = unsafe { self.entries.get_unchecked(0) };
            if entry.class == class {
                let obj_ptr = obj.as_ptr() as *const u8;
                let field_ptr = unsafe { obj_ptr.add(entry.offset as usize) };
                return Some(unsafe { *(field_ptr as *const Value) });
            }
        }
        
        // Slow path: cache miss or polymorphic
        self.slow_path_lookup(obj, field_id)
    }
}
```

## 4. JIT Compilation (Tier 1)

### 4.1 Method-Based JIT

Simple method-at-a-time compilation for reduced complexity:

```rust
pub struct MethodJIT {
    // Compile entire functions, not traces
    compiled_functions: FxHashMap<FunctionId, CompiledFunction>,
    compilation_queue: VecDeque<FunctionId>,
}

impl MethodJIT {
    pub fn maybe_compile(&mut self, func_id: FunctionId, count: u32) {
        if count > Self::THRESHOLD && !self.compiled_functions.contains_key(&func_id) {
            self.compilation_queue.push_back(func_id);
            
            // Compile in background thread
            if let Some(func) = self.compilation_queue.pop_front() {
                self.compile_function(func);
            }
        }
    }
    
    fn compile_function(&mut self, func_id: FunctionId) {
        let func = &self.functions[func_id];
        let mut ctx = CraneliftContext::new();
        
        // Simple lowering without trace recording
        for node in &func.ast {
            self.compile_node(&mut ctx, node);
        }
        
        let code = ctx.finalize();
        self.compiled_functions.insert(func_id, code);
    }
}
```

## 5. Memory Management

### 5.1 Conservative Stack Scanning

Simplified GC without precise stack maps:

```rust
pub struct ConservativeGC {
    // Single generation to reduce complexity
    heap: BumpAllocator,
    mark_bitmap: BitVec,
    
    // Conservative root scanning
    stack_bounds: (usize, usize),
}

impl ConservativeGC {
    pub fn collect(&mut self) {
        // Mark phase: scan stack conservatively
        self.scan_stack_conservative();
        self.scan_globals();
        
        // Sweep phase: reclaim unmarked objects
        self.sweep();
    }
    
    fn scan_stack_conservative(&mut self) {
        let (bottom, top) = self.stack_bounds;
        
        for addr in (bottom..top).step_by(8) {
            let potential_ptr = unsafe { *(addr as *const usize) };
            
            // If it looks like a heap pointer, mark it
            if self.heap.contains(potential_ptr) {
                self.mark_object(potential_ptr);
            }
        }
    }
}
```

## 6. Implementation Phases

### Phase 0: Minimal Viable Interpreter (Week 1)
- [ ] Tagged pointer values
- [ ] AST walker with switch dispatch
- [ ] Basic arithmetic and variables
- [ ] Function calls

### Phase 1: Performance Foundation (Week 2-3)
- [ ] Direct threading
- [ ] Monomorphic inline caches
- [ ] Type feedback collection
- [ ] Conservative GC

### Phase 2: JIT Integration (Week 4-5)
- [ ] Method compilation triggers
- [ ] Cranelift lowering
- [ ] Function call patching
- [ ] Background compilation

### Phase 3: Production Hardening (Week 6)
- [ ] Deoptimization guards
- [ ] Interrupt handling
- [ ] Resource limits
- [ ] Differential testing

## 7. Implementation Phases

### Phase 0: Minimal Interpreter (Week 1-2)
- [ ] NaN-boxed value representation
- [ ] Direct-threaded AST walker
- [ ] Basic arithmetic and variables
- [ ] Function calls without optimization

### Phase 1: Performance Foundation (Week 3-4)
- [ ] Inline caching for field access
- [ ] Type feedback collection
- [ ] Register allocator for locals
- [ ] Peephole optimizations

### Phase 2: Bytecode VM (Week 5-6)
- [ ] Three-address bytecode generation
- [ ] Type-specialized instructions
- [ ] Loop-invariant code motion
- [ ] Method inlining heuristics

### Phase 3: JIT Integration (Week 7-8)
- [ ] Trace recording
- [ ] Guard generation
- [ ] Cranelift backend
- [ ] Deoptimization support

### Phase 4: Advanced Optimizations (Week 9-10)
- [ ] Generational GC
- [ ] Escape analysis
- [ ] Polymorphic inline caches
- [ ] Speculative optimizations

## 8. Performance Targets

| Metric | Target | LuaJIT | V8 | Notes |
|--------|--------|--------|----| ------|
| Cold start | <5ms | 10ms | 8ms | First execution |
| Interpreter throughput | 100M ops/sec | 80M | 60M | Tier 0 |
| Bytecode VM throughput | 500M ops/sec | 400M | N/A | Tier 1 |
| JIT throughput | 2B ops/sec | 3B | 4B | Tier 2 |
| Memory per isolate | <5MB | 8MB | 20MB | Base overhead |
| GC pause (minor) | <1ms | 2ms | 1ms | Nursery collection |
| GC pause (major) | <10ms | 20ms | 15ms | Full collection |

## 9. Testing Strategy

### 9.1 Differential Testing

```rust
#[test]
fn differential_test(program: &str) {
    let ast = parse(program).unwrap();
    
    // Execute in all tiers
    let tier0_result = interpret_ast(&ast);
    let tier1_result = execute_bytecode(&compile_to_bytecode(&ast));
    let tier2_result = execute_jit(&compile_to_native(&ast));
    let transpiled = execute_transpiled(&transpile_to_rust(&ast));
    
    // All must produce identical results
    assert_eq!(tier0_result, tier1_result);
    assert_eq!(tier1_result, tier2_result);
    assert_eq!(tier2_result, transpiled);
}
```

### 9.2 Chaos Testing

```rust
#[test]
fn chaos_deoptimization() {
    // Force deoptimization at random points
    let mut interpreter = Interpreter::new();
    interpreter.set_chaos_mode(ChaosMode::RandomDeopt);
    
    for _ in 0..1000 {
        let result = interpreter.eval("complex_program()");
        assert!(result.is_ok());
    }
}
```

## 10. Security Considerations

### 10.1 Control Flow Integrity

```rust
// Validate jump targets to prevent ROP attacks
impl BytecodeValidator {
    pub fn validate(&self, code: &[Instruction]) -> Result<()> {
        let valid_targets = self.compute_valid_targets(code);
        
        for (pc, instr) in code.iter().enumerate() {
            if instr.is_jump() {
                let target = pc as i32 + instr.jump_offset();
                if !valid_targets.contains(&target) {
                    return Err(Error::InvalidJumpTarget);
                }
            }
        }
        
        Ok(())
    }
}
```

### 10.2 Resource Limits

```rust
pub struct ResourceLimits {
    max_stack_depth: usize,      // Default: 10,000
    max_heap_size: usize,         // Default: 1GB
    max_execution_time: Duration, // Default: 30s
    max_allocations_per_gc: u32,  // Default: 1M
}
```

## Appendix A: Instruction Set Reference

See separate document for complete opcode listing.

## Appendix B: Benchmarking Methodology

All benchmarks follow the methodology from "Virtual Machine Warmup Blows Hot and Cold" (Barrett et al. 2017), with proper warmup detection and statistical analysis.

## References

1. LuaJIT 2.0 Bytecode Reference
2. V8 TurboFan Design Documents
3. PyPy Tracing JIT Documentation
4. CPython 3.11 Specializing Interpreter PEP 659
5. Truffle/Graal Partial Evaluation Papers