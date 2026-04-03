# Sub-spec: Optimization — JIT Architecture & LLVM Integration

**Parent:** [jit-llvm-julia-style-optimization.md](../jit-llvm-julia-style-optimization.md) Sections 3-5

---

## 3. Julia-Style JIT Architecture (Long-term)

### 3.1 How Julia Achieves Near-Native Performance

```
Julia Execution Flow:
─────────────────────

1. Parse & Lower to IR (one-time)
   source.jl → AST → Typed IR

2. Type Inference (runtime profiling)
   function add(a, b)      # Called with (5, 3)
   └→ Inferred: add(Int, Int) → Int

3. LLVM Code Generation (specialized)
   define i64 @add_Int_Int(i64 %a, i64 %b) {
     %result = add i64 %a, %b
     ret i64 %result
   }

4. LLVM Optimization & JIT Compile
   LLVM IR → Optimized IR → Native x86_64 assembly

5. Cache & Execute (subsequent calls)
   add(5, 3) → Lookup cache → Execute native code
```

### 3.2 Key Principles

1. **Lazy Compilation:** Only compile what's executed
2. **Type Specialization:** Generate different native code for different type combinations
3. **Method Cache:** Store compiled versions indexed by type signature
4. **Tiered Execution:**
   - Tier 0: Interpret (cold code, <10 calls)
   - Tier 1: Quick compile (warm code, 10-100 calls)
   - Tier 2: LLVM full optimization (hot code, 100+ calls)

---

## 4. Ruchy JIT+LLVM Design

### 4.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Ruchy v4.0 (Julia-Style)                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Tier 0: AST Interpreter (Cold Path)                          │  │
│  │ - First execution: Parse → AST → Interpret                   │  │
│  │ - Profile: Track call counts, type observations              │  │
│  │ - Decision: If hotness > threshold → promote to Tier 1       │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                              ↓                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Tier 1: Quick JIT (Warm Path)                                │  │
│  │ - Simple codegen: Direct x86_64 assembly (via Cranelift)     │  │
│  │ - No optimization: Fast compile, decent performance          │  │
│  │ - Continue profiling: Track types, inline candidates         │  │
│  │ - Decision: If hotness > threshold → promote to Tier 2       │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                              ↓                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Tier 2: LLVM Full Optimization (Hot Path)                    │  │
│  │ - Type specialization: Generate per-type-signature versions  │  │
│  │ - LLVM IR generation: From typed AST                         │  │
│  │ - Full optimization: -O3, inlining, vectorization, etc.      │  │
│  │ - Cache: Store in method table indexed by type signature     │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Method Cache (Global)                                         │  │
│  │ HashMap<(FunctionName, TypeSignature), CompiledCode>         │  │
│  │                                                               │  │
│  │ Example:                                                      │  │
│  │ ("is_prime", [i32]) → 0x7f8a4c0012a0 (Tier 2, native code)  │  │
│  │ ("add", [i32, i32]) → 0x7f8a4c001500 (Tier 2, native code)  │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.2 Core Components

#### 4.2.1 Execution Engine

```rust
pub struct RuchyExecutionEngine {
    /// AST interpreter for cold code
    interpreter: ASTInterpreter,

    /// Quick JIT compiler (Cranelift)
    quick_jit: CraneliftJIT,

    /// LLVM JIT compiler (inkwell)
    llvm_jit: LLVMJITEngine,

    /// Method cache: (function, type_sig) → compiled code
    method_cache: MethodCache,

    /// Profiler: Tracks hotness and type observations
    profiler: RuntimeProfiler,

    /// Configuration
    config: JITConfig,
}

pub struct JITConfig {
    /// Promote to Tier 1 after N calls
    tier1_threshold: usize,  // Default: 10

    /// Promote to Tier 2 after N calls
    tier2_threshold: usize,  // Default: 100

    /// Enable LLVM optimizations
    llvm_opt_level: OptLevel,  // Default: Aggressive

    /// Maximum cached methods
    max_cached_methods: usize,  // Default: 10000
}
```

#### 4.2.2 Type Specialization

```rust
/// Type signature for method specialization
#[derive(Hash, Eq, PartialEq, Clone)]
pub struct TypeSignature {
    params: Vec<ConcreteType>,
    return_type: ConcreteType,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum ConcreteType {
    Int32,
    Int64,
    Float64,
    Bool,
    String,
    Vec(Box<ConcreteType>),
    Function(Vec<ConcreteType>, Box<ConcreteType>),
}

/// Example specialization:
/// Ruchy function: fun add(a, b) { a + b }
///
/// Compiled versions:
/// - add(i32, i32) -> i32  (one LLVM function)
/// - add(f64, f64) -> f64  (different LLVM function)
/// - add(String, String) -> String  (different LLVM function)
```

#### 4.2.3 Method Cache

```rust
pub struct MethodCache {
    /// Cache of compiled methods
    cache: HashMap<MethodKey, CompiledMethod>,

    /// LRU eviction policy
    lru: LRUList,
}

#[derive(Hash, Eq, PartialEq)]
struct MethodKey {
    function_name: String,
    type_signature: TypeSignature,
}

struct CompiledMethod {
    /// Tier level (1 = Cranelift, 2 = LLVM)
    tier: u8,

    /// Function pointer to native code
    native_fn: *const (),

    /// Metadata for debugging
    metadata: MethodMetadata,
}
```

#### 4.2.4 Runtime Profiler

```rust
pub struct RuntimeProfiler {
    /// Call counts per function
    call_counts: HashMap<String, usize>,

    /// Observed type signatures per function
    type_observations: HashMap<String, Vec<TypeSignature>>,

    /// Execution time tracking
    execution_times: HashMap<MethodKey, Duration>,
}

impl RuntimeProfiler {
    /// Record a function call with observed types
    pub fn record_call(&mut self, func: &str, args: &[Value]) {
        // Increment call count
        *self.call_counts.entry(func.to_string()).or_insert(0) += 1;

        // Record type signature
        let sig = TypeSignature::from_values(args);
        self.type_observations
            .entry(func.to_string())
            .or_default()
            .push(sig);
    }

    /// Check if function should be promoted to next tier
    pub fn should_promote(&self, func: &str, current_tier: u8) -> bool {
        let count = self.call_counts.get(func).copied().unwrap_or(0);
        match current_tier {
            0 => count >= self.config.tier1_threshold,
            1 => count >= self.config.tier2_threshold,
            _ => false,
        }
    }
}
```

---

## 5. LLVM Integration (inkwell)

### 5.1 LLVM IR Generation

```rust
use inkwell::*;
use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;

pub struct LLVMCodegen<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> LLVMCodegen<'ctx> {
    /// Generate LLVM IR for a Ruchy function
    pub fn codegen_function(
        &self,
        ast: &Expr,
        type_sig: &TypeSignature,
    ) -> Result<FunctionValue<'ctx>> {
        match &ast.kind {
            ExprKind::Function { name, params, body, .. } => {
                // Create function signature
                let param_types: Vec<_> = type_sig.params.iter()
                    .map(|t| self.llvm_type(t))
                    .collect();
                let return_type = self.llvm_type(&type_sig.return_type);

                let fn_type = return_type.fn_type(&param_types, false);
                let function = self.module.add_function(name, fn_type, None);

                // Create entry basic block
                let entry = self.context.append_basic_block(function, "entry");
                self.builder.position_at_end(entry);

                // Generate IR for function body
                let return_value = self.codegen_expr(body, &type_sig)?;
                self.builder.build_return(Some(&return_value));

                Ok(function)
            }
            _ => bail!("Expected function expression"),
        }
    }

    /// Generate LLVM IR for an expression
    fn codegen_expr(
        &self,
        expr: &Expr,
        type_sig: &TypeSignature,
    ) -> Result<BasicValueEnum<'ctx>> {
        match &expr.kind {
            // Integer literal
            ExprKind::Literal(Literal::Integer(n)) => {
                Ok(self.context.i32_type().const_int(*n as u64, false).into())
            }

            // Binary operation (specialized!)
            ExprKind::Binary { op, left, right } => {
                let lhs = self.codegen_expr(left, type_sig)?;
                let rhs = self.codegen_expr(right, type_sig)?;

                match op {
                    BinaryOp::Add => {
                        // Type-specialized addition
                        match &type_sig.return_type {
                            ConcreteType::Int32 => {
                                let result = self.builder.build_int_add(
                                    lhs.into_int_value(),
                                    rhs.into_int_value(),
                                    "add"
                                );
                                Ok(result.into())
                            }
                            ConcreteType::Float64 => {
                                let result = self.builder.build_float_add(
                                    lhs.into_float_value(),
                                    rhs.into_float_value(),
                                    "fadd"
                                );
                                Ok(result.into())
                            }
                            _ => bail!("Unsupported add type"),
                        }
                    }
                    BinaryOp::Multiply => {
                        let result = self.builder.build_int_mul(
                            lhs.into_int_value(),
                            rhs.into_int_value(),
                            "mul"
                        );
                        Ok(result.into())
                    }
                    BinaryOp::Less => {
                        let result = self.builder.build_int_compare(
                            IntPredicate::SLT,
                            lhs.into_int_value(),
                            rhs.into_int_value(),
                            "lt"
                        );
                        Ok(result.into())
                    }
                    // ... other operators
                    _ => bail!("Unsupported operator: {:?}", op),
                }
            }

            // Variable reference
            ExprKind::Identifier(name) => {
                // Look up in local variables (would need proper scope tracking)
                self.lookup_variable(name)
            }

            // Function call
            ExprKind::Call { func, args } => {
                self.codegen_call(func, args, type_sig)
            }

            // While loop
            ExprKind::While { condition, body, .. } => {
                self.codegen_while_loop(condition, body, type_sig)
            }

            // If expression
            ExprKind::If { condition, then_branch, else_branch } => {
                self.codegen_if(condition, then_branch, else_branch.as_deref(), type_sig)
            }

            _ => bail!("Unsupported expression: {:?}", expr.kind),
        }
    }

    /// Generate optimized while loop
    fn codegen_while_loop(
        &self,
        condition: &Expr,
        body: &Expr,
        type_sig: &TypeSignature,
    ) -> Result<BasicValueEnum<'ctx>> {
        let current_fn = self.builder.get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();

        // Create basic blocks
        let loop_header = self.context.append_basic_block(current_fn, "loop");
        let loop_body = self.context.append_basic_block(current_fn, "loop.body");
        let loop_exit = self.context.append_basic_block(current_fn, "loop.exit");

        // Jump to loop header
        self.builder.build_unconditional_branch(loop_header);

        // Loop header: evaluate condition
        self.builder.position_at_end(loop_header);
        let cond_value = self.codegen_expr(condition, type_sig)?
            .into_int_value();
        self.builder.build_conditional_branch(cond_value, loop_body, loop_exit);

        // Loop body
        self.builder.position_at_end(loop_body);
        self.codegen_expr(body, type_sig)?;
        self.builder.build_unconditional_branch(loop_header);

        // Loop exit
        self.builder.position_at_end(loop_exit);

        // Return unit
        Ok(self.context.i32_type().const_int(0, false).into())
    }

    /// Map Ruchy type to LLVM type
    fn llvm_type(&self, ty: &ConcreteType) -> BasicTypeEnum<'ctx> {
        match ty {
            ConcreteType::Int32 => self.context.i32_type().into(),
            ConcreteType::Int64 => self.context.i64_type().into(),
            ConcreteType::Float64 => self.context.f64_type().into(),
            ConcreteType::Bool => self.context.bool_type().into(),
            ConcreteType::String => {
                // String as i8* (pointer to char array)
                self.context.i8_type().ptr_type(AddressSpace::default()).into()
            }
            ConcreteType::Vec(elem_ty) => {
                // Vec as struct { ptr: *T, len: i64, capacity: i64 }
                let elem_type = self.llvm_type(elem_ty);
                let ptr_type = elem_type.ptr_type(AddressSpace::default());
                let len_type = self.context.i64_type();
                self.context.struct_type(
                    &[ptr_type.into(), len_type.into(), len_type.into()],
                    false
                ).into()
            }
            _ => panic!("Unsupported type: {:?}", ty),
        }
    }
}
```

### 5.2 Optimized BENCH-008 Example

```rust
// Ruchy source
fun is_prime(n) {
    if n < 2 { return false }
    if n == 2 { return true }
    if n % 2 == 0 { return false }
    let mut i = 3
    while i * i <= n {
        if n % i == 0 { return false }
        i = i + 2
    }
    true
}

// After type specialization: is_prime(i32) -> bool
// Generated LLVM IR (simplified):

define i1 @is_prime_i32(i32 %n) {
entry:
  %cmp1 = icmp slt i32 %n, 2
  br i1 %cmp1, label %return_false, label %check2

check2:
  %cmp2 = icmp eq i32 %n, 2
  br i1 %cmp2, label %return_true, label %check_even

check_even:
  %rem = srem i32 %n, 2
  %is_even = icmp eq i32 %rem, 0
  br i1 %is_even, label %return_false, label %loop_init

loop_init:
  br label %loop

loop:
  %i = phi i32 [ 3, %loop_init ], [ %i_next, %loop_body ]
  %i_squared = mul i32 %i, %i
  %continue = icmp sle i32 %i_squared, %n
  br i1 %continue, label %loop_body, label %return_true

loop_body:
  %rem2 = srem i32 %n, %i
  %divides = icmp eq i32 %rem2, 0
  br i1 %divides, label %return_false, label %loop_continue

loop_continue:
  %i_next = add i32 %i, 2
  br label %loop

return_false:
  ret i1 false

return_true:
  ret i1 true
}

// After LLVM optimization (O3):
// - Loop unrolling for small i values
// - Strength reduction (i*i → incremental)
// - Dead code elimination
// - Register allocation
// Result: Near-native performance (~5ms vs 1,588ms)
```

---

