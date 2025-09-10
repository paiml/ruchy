# WASM Lean Implementation Plan

## Executive Summary

**Current State**: WASM commands exist but return "not yet implemented"
**Root Cause**: Built analysis before emission (wrong sequence)
**Solution**: Build minimal emitter first, analysis later

## Critical Path (4 Weeks)

### Week 1: Module Structure
**Goal**: Emit valid empty WASM module

```rust
// src/backend/wasm/emitter.rs
use wasm_encoder::{Module, TypeSection, FunctionSection, CodeSection};

pub struct WasmEmitter {
    module: Module,
}

impl WasmEmitter {
    pub fn emit_program(ast: &Program) -> Result<Vec<u8>, String> {
        let mut module = Module::new();
        
        // Minimal valid module: just types section
        let mut types = TypeSection::new();
        types.function(vec![], vec![]); // void main()
        module.section(&types);
        
        Ok(module.finish())
    }
}
```

**Validation**: 
```bash
cargo test wasm_emit_empty
wasm-validate output.wasm
```

### Week 2: Expression Lowering
**Goal**: Compile arithmetic expressions

```rust
fn lower_expr(expr: &Expr) -> Vec<Instruction> {
    use wasm_encoder::Instruction::*;
    
    match &expr.kind {
        ExprKind::Literal(Value::Int(n)) => {
            vec![I32Const(*n as i32)]
        }
        ExprKind::Binary { op, left, right } => {
            let mut code = lower_expr(left);
            code.extend(lower_expr(right));
            code.push(match op {
                BinaryOp::Add => I32Add,
                BinaryOp::Sub => I32Sub,
                BinaryOp::Mul => I32Mul,
                BinaryOp::Div => I32DivS,
                _ => return vec![], // Skip unsupported
            });
            code
        }
        _ => vec![] // Skip complex expressions initially
    }
}
```

**Test Case**:
```rust
// Input: 2 + 3 * 4
// Expected WASM: [I32Const(2), I32Const(3), I32Const(4), I32Mul, I32Add]
```

### Week 3: Memory Model
**Goal**: Stack-based locals, no heap

```rust
pub struct MemoryLayout {
    stack_pointer: u32,
    locals: HashMap<String, u32>,
}

impl MemoryLayout {
    const STACK_SIZE: u32 = 65536; // 64KB
    
    fn allocate_local(&mut self, name: String, size: u32) -> u32 {
        let offset = self.stack_pointer;
        self.stack_pointer += size;
        self.locals.insert(name, offset);
        offset
    }
    
    fn emit_memory(&self) -> MemorySection {
        let mut memory = MemorySection::new();
        memory.memory(MemoryType {
            minimum: 1, // 1 page = 64KB
            maximum: None,
            memory64: false,
            shared: false,
        });
        memory
    }
}
```

### Week 4: Function Compilation
**Goal**: Compile simple functions

```rust
fn compile_function(func: &Function) -> Result<Vec<u8>, String> {
    let mut func_body = vec![];
    let mut locals = vec![];
    
    // Map parameters to locals
    for (i, param) in func.params.iter().enumerate() {
        locals.push((1, ValType::I32)); // All i32 initially
    }
    
    // Compile body
    for stmt in &func.body {
        func_body.extend(compile_statement(stmt)?);
    }
    
    // Return
    func_body.push(Instruction::End);
    
    Ok(encode_function(locals, func_body))
}
```

## Technical Debt Priority

### 1. Parser Complexity (BLOCKER)
**Current**: 27 functions with complexity >10
**Target**: All functions ≤10 complexity
**Approach**: Extract helper functions

```rust
// Before: parse_expression() with complexity 27
// After: 
fn parse_expression() {
    parse_primary_expr()
    parse_binary_expr()
    parse_call_expr()
    // Each helper <10 complexity
}
```

### 2. Type System (CRITICAL)
**Current**: Strings for types
**Target**: Proper type enum
**Timeline**: Week 2

```rust
enum WasmType {
    I32,
    I64,
    F32,
    F64,
    // No reference types initially
}
```

### 3. Error Handling (IMPORTANT)
**Current**: String errors
**Target**: Typed errors
**Timeline**: Week 3

```rust
enum WasmError {
    InvalidModule,
    TypeMismatch { expected: WasmType, found: WasmType },
    UnsupportedFeature(String),
}
```

## Validation Strategy

### Level 1: Binary Validity
```rust
#[test]
fn test_valid_wasm() {
    let wasm = emit_program(ast);
    assert!(wasmparser::validate(&wasm).is_ok());
}
```

### Level 2: Execution Success
```rust
#[test]
fn test_wasm_runs() {
    let wasm = emit_program(ast);
    let engine = wasmtime::Engine::default();
    assert!(wasmtime::Module::new(&engine, &wasm).is_ok());
}
```

### Level 3: Behavioral Correctness
```rust
#[test]
fn test_same_behavior() {
    let rust_result = interpret_rust(program);
    let wasm_result = execute_wasm(program);
    assert_eq!(rust_result, wasm_result);
}
```

## What We're NOT Doing (MUDA/Waste)

1. **NO streaming analysis** - Emit static modules first
2. **NO formal verification** - Basic correctness first  
3. **NO optimization** - Working code first
4. **NO security scanning** - Valid modules first
5. **NO hardware profiling** - Single target first
6. **NO intermediate representations** - Direct AST→WASM

## Success Metrics

### Week 1
- [ ] Empty module validates with wasmparser
- [ ] Module loads in wasmtime without error

### Week 2  
- [ ] Arithmetic expressions compile
- [ ] 10 integration tests pass

### Week 3
- [ ] Local variables work
- [ ] 25 integration tests pass

### Week 4
- [ ] Functions compile and execute
- [ ] 50 integration tests pass

## Implementation Commands

```bash
# Week 1: Setup
cargo add wasm-encoder wasmparser wasmtime
mkdir -p src/backend/wasm
touch src/backend/wasm/emitter.rs

# Week 2: Expression tests
cargo test --test wasm_expressions

# Week 3: Memory tests  
cargo test --test wasm_memory

# Week 4: Function tests
cargo test --test wasm_functions

# Validation
for f in tests/wasm/*.wasm; do
    wasm-validate "$f" || echo "FAIL: $f"
done
```

## Next Steps After MVP

Only after basic emission works:

1. **Optimization** (Week 5-6)
   - Register allocation
   - Instruction selection
   - Dead code elimination

2. **Advanced Features** (Week 7-8)
   - Closures
   - Heap allocation
   - Garbage collection

3. **Analysis Tools** (Week 9+)
   - Security scanner
   - Performance profiler
   - Formal verification

## Conclusion

The path is clear: **emission before analysis**, **correctness before optimization**, **working before perfect**.

Start with `cargo test wasm_emit_empty` and build up from there.