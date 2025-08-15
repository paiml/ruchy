# Ruchy-Depyler Integration Specification v1.0
## Python Migration and Hybrid Execution Architecture

### Table of Contents
1. [Integration Architecture](#1-integration-architecture)
2. [Type System Mapping](#2-type-system-mapping)
3. [AST Transformation Pipeline](#3-ast-transformation-pipeline)
4. [Runtime Interoperability](#4-runtime-interoperability)
5. [Memory Model Unification](#5-memory-model-unification)
6. [Performance Optimization](#6-performance-optimization)
7. [Quality Enforcement](#7-quality-enforcement)
8. [Migration Strategies](#8-migration-strategies)

---

## 1. Integration Architecture

### 1.1 Component Interaction Model

```rust
pub struct RuchyDepylerBridge {
    // Depyler transpilation engine
    depyler: DepylerCore,
    
    // Ruchy type elaborator
    elaborator: RuchyElaborator,
    
    // Shared HIR representation
    hir_cache: Arc<DashMap<ModuleId, HirModule>>,
    
    // Type inference unification
    type_unifier: UnifiedTypeSystem,
    
    // Performance profiler
    profiler: MigrationProfiler,
}

impl RuchyDepylerBridge {
    pub fn transpile_python(&self, source: &str) -> Result<RuchyAst> {
        // Phase 1: Python → Depyler HIR
        let python_ast = depyler::parse_python(source)?;
        let hir = self.depyler.to_hir(python_ast)?;
        
        // Phase 2: Type inference with cross-language constraints
        let typed_hir = self.type_unifier.infer_types(&hir)?;
        
        // Phase 3: HIR → Ruchy AST with elaboration
        let ruchy_ast = self.elaborate_to_ruchy(typed_hir)?;
        
        // Phase 4: Optimization passes
        let optimized = self.optimize_migration(ruchy_ast)?;
        
        Ok(optimized)
    }
}
```

### 1.2 Shared Intermediate Representation

```rust
// Unified HIR supporting both Python and Ruchy semantics
pub enum UnifiedHir {
    // Python-specific constructs
    PythonComprehension {
        target: Pattern,
        iter: Box<UnifiedHir>,
        filters: Vec<UnifiedHir>,
        body: Box<UnifiedHir>,
    },
    
    // Ruchy-specific constructs
    Pipeline {
        source: Box<UnifiedHir>,
        operations: Vec<PipelineOp>,
    },
    
    // Shared constructs
    Function {
        name: Symbol,
        params: Vec<Parameter>,
        body: Box<UnifiedHir>,
        effects: EffectSet,
        purity: PurityLevel,
    },
    
    // Type-directed elaboration hints
    Elaborated {
        original: Box<UnifiedHir>,
        elaboration: ElaborationStrategy,
        performance_hint: PerformanceHint,
    },
}

// Performance-guided elaboration
pub enum ElaborationStrategy {
    // Python list → Ruchy DataFrame for analytics workloads
    ListToDataFrame {
        expected_ops: Vec<DataFrameOp>,
        cardinality_estimate: usize,
    },
    
    // Python dict → Ruchy HashMap with custom hasher
    DictToHashMap {
        key_distribution: Distribution,
        access_pattern: AccessPattern,
    },
    
    // Python generator → Ruchy lazy iterator
    GeneratorToStream {
        bounded: bool,
        parallelizable: bool,
    },
}
```

## 2. Type System Mapping

### 2.1 Bidirectional Type Translation

```rust
pub struct TypeSystemBridge {
    // Python → Ruchy type mapping
    python_to_ruchy: BiMap<PythonType, RuchyType>,
    
    // Constraint solver for gradual typing boundaries
    constraint_solver: Z3Solver,
    
    // Refinement type synthesis
    refinement_synthesizer: RefinementSynthesizer,
}

impl TypeSystemBridge {
    pub fn translate_type(&self, py_type: &PythonType) -> Result<RuchyType> {
        match py_type {
            // Primitive mappings
            PythonType::Int => Ok(RuchyType::I64), // Default to i64
            PythonType::Float => Ok(RuchyType::F64),
            PythonType::Str => Ok(RuchyType::String),
            
            // Collection type inference
            PythonType::List(inner) => {
                let inner_ruchy = self.translate_type(inner)?;
                
                // Analyze usage patterns for optimal representation
                if self.is_numeric_computation(inner) {
                    Ok(RuchyType::Series(Box::new(inner_ruchy)))
                } else if self.is_tabular_data(inner) {
                    Ok(RuchyType::DataFrame)
                } else {
                    Ok(RuchyType::Vec(Box::new(inner_ruchy)))
                }
            },
            
            // Function types with effect tracking
            PythonType::Callable(params, ret) => {
                let ruchy_params = params.iter()
                    .map(|p| self.translate_type(p))
                    .collect::<Result<Vec<_>>>()?;
                    
                let ruchy_ret = self.translate_type(ret)?;
                
                // Infer effects from function body analysis
                let effects = self.infer_effects(py_type)?;
                
                Ok(RuchyType::Function {
                    params: ruchy_params,
                    returns: Box::new(ruchy_ret),
                    effects,
                })
            },
            
            // Gradual typing boundary
            PythonType::Any => Ok(RuchyType::Dynamic(TypeId::fresh())),
            
            // Union types → Ruchy enums
            PythonType::Union(variants) => {
                self.synthesize_enum(variants)
            },
        }
    }
    
    pub fn synthesize_refinement(&self, 
                                  py_type: &PythonType, 
                                  usage: &UsageAnalysis) -> Option<RefinedType> {
        // Extract invariants from usage patterns
        let invariants = usage.extract_invariants();
        
        // Synthesize refinement predicates
        if let Some(predicate) = self.refinement_synthesizer.synthesize(&invariants) {
            Some(RefinedType {
                base: self.translate_type(py_type).ok()?,
                predicate,
                proof: self.generate_proof(&predicate),
            })
        } else {
            None
        }
    }
}
```

### 2.2 Ownership Inference

```rust
pub struct OwnershipInferencer {
    // Escape analysis for Python values
    escape_analyzer: EscapeAnalyzer,
    
    // Lifetime inference engine
    lifetime_inferencer: LifetimeInferencer,
    
    // Sharing analysis
    alias_analyzer: AliasAnalyzer,
}

impl OwnershipInferencer {
    pub fn infer_ownership(&self, hir: &UnifiedHir) -> OwnershipAnnotated {
        // Phase 1: Escape analysis
        let escapes = self.escape_analyzer.analyze(hir);
        
        // Phase 2: Sharing pattern detection
        let aliases = self.alias_analyzer.find_aliases(hir);
        
        // Phase 3: Lifetime inference
        let lifetimes = self.lifetime_inferencer.infer(hir, &escapes, &aliases);
        
        // Phase 4: Generate ownership annotations
        self.annotate_ownership(hir, escapes, aliases, lifetimes)
    }
    
    fn annotate_ownership(&self, 
                          hir: &UnifiedHir,
                          escapes: EscapeSet,
                          aliases: AliasSet,
                          lifetimes: LifetimeMap) -> OwnershipAnnotated {
        OwnershipAnnotated {
            hir: hir.clone(),
            annotations: hir.traverse_with(|node| {
                match self.classify_ownership(node, &escapes, &aliases) {
                    OwnershipClass::Owned => Annotation::Move,
                    OwnershipClass::Borrowed => {
                        Annotation::Borrow(lifetimes.get(node))
                    },
                    OwnershipClass::Shared => Annotation::Rc,
                    OwnershipClass::Mutable => {
                        Annotation::RefCell(lifetimes.get(node))
                    },
                }
            }),
        }
    }
}
```

## 3. AST Transformation Pipeline

### 3.1 Pattern-Based Transformation

```rust
pub struct AstTransformer {
    // Pattern matching engine
    pattern_engine: PatternMatcher,
    
    // Transformation rules database
    rules: TransformationRules,
    
    // Cost model for selecting transformations
    cost_model: CostModel,
}

impl AstTransformer {
    pub fn transform(&self, python_ast: &PythonAst) -> RuchyAst {
        // Apply transformations in optimal order
        let mut ast = self.initial_transform(python_ast);
        
        // Iterative improvement with cost-guided search
        while let Some(improvement) = self.find_best_improvement(&ast) {
            ast = self.apply_transformation(ast, improvement);
        }
        
        ast
    }
    
    fn transformation_rules(&self) -> Vec<TransformationRule> {
        vec![
            // List comprehension → Pipeline
            TransformationRule {
                pattern: pattern! {
                    [#expr for #var in #iter if #cond]
                },
                transform: |captures| {
                    quote! {
                        #iter |> filter(|#var| #cond) |> map(|#var| #expr)
                    }
                },
                cost_reduction: 0.3, // 30% performance improvement
            },
            
            // Nested loops → DataFrame operations
            TransformationRule {
                pattern: pattern! {
                    for #i in #range1:
                        for #j in #range2:
                            #body
                },
                transform: |captures| {
                    if self.is_matrix_operation(&captures) {
                        quote! {
                            DataFrame::from_product(#range1, #range2)
                                |> apply(|row| #body)
                        }
                    } else {
                        // Keep nested loops
                        captures.original()
                    }
                },
                cost_reduction: 0.6, // 60% for matrix operations
            },
            
            // Exception → Result
            TransformationRule {
                pattern: pattern! {
                    try:
                        #body
                    except #exc_type as #var:
                        #handler
                },
                transform: |captures| {
                    quote! {
                        match (|| -> Result<_, #exc_type> { #body })() {
                            Ok(val) => val,
                            Err(#var) => #handler,
                        }
                    }
                },
                cost_reduction: 0.1, // 10% from removed runtime checks
            },
        ]
    }
}
```

### 3.2 Semantic-Preserving Optimizations

```rust
pub struct SemanticOptimizer {
    // Dataflow analysis
    dataflow: DataflowAnalyzer,
    
    // Constant propagation
    const_prop: ConstantPropagator,
    
    // Dead code elimination
    dce: DeadCodeEliminator,
}

impl SemanticOptimizer {
    pub fn optimize(&self, ast: RuchyAst) -> RuchyAst {
        // Build dataflow graph
        let cfg = self.build_cfg(&ast);
        let dataflow = self.dataflow.analyze(&cfg);
        
        // Apply optimizations in specific order
        let mut optimized = ast;
        
        // 1. Constant propagation
        optimized = self.const_prop.propagate(optimized, &dataflow);
        
        // 2. Common subexpression elimination
        optimized = self.eliminate_common_subexpressions(optimized, &dataflow);
        
        // 3. Loop-invariant code motion
        optimized = self.hoist_loop_invariants(optimized, &cfg);
        
        // 4. Dead code elimination
        optimized = self.dce.eliminate(optimized, &dataflow);
        
        // 5. Python-specific optimizations
        optimized = self.optimize_python_patterns(optimized);
        
        optimized
    }
    
    fn optimize_python_patterns(&self, ast: RuchyAst) -> RuchyAst {
        ast.transform_bottom_up(|node| {
            match node {
                // String concatenation → StringBuilder
                RuchyAst::BinOp { 
                    op: Op::Add, 
                    left: String(_), 
                    right: String(_) 
                } => {
                    RuchyAst::MethodCall {
                        receiver: quote! { StringBuilder::new() },
                        method: "push_str",
                        args: vec![left, right],
                    }
                },
                
                // Range iteration → iterator
                RuchyAst::ForLoop { 
                    var, 
                    iter: Range(start, end), 
                    body 
                } => {
                    RuchyAst::Iterator {
                        iter: quote! { (#start..#end) },
                        var,
                        body,
                    }
                },
                
                _ => node,
            }
        })
    }
}
```

## 4. Runtime Interoperability

### 4.1 Hybrid Execution Model

```rust
pub struct HybridRuntime {
    // Python interpreter for gradual migration
    python_vm: PyO3Runtime,
    
    // Ruchy execution engine
    ruchy_engine: RuchyEngine,
    
    // Cross-language FFI
    ffi_bridge: FfiBridge,
    
    // Shared memory manager
    memory_manager: UnifiedMemoryManager,
}

impl HybridRuntime {
    pub fn execute_hybrid(&self, module: HybridModule) -> Result<Value> {
        // Partition module into Python and Ruchy components
        let (python_parts, ruchy_parts) = self.partition_module(module);
        
        // Compile Ruchy parts to native code
        let ruchy_lib = self.compile_ruchy(ruchy_parts)?;
        
        // Set up FFI bindings
        let bindings = self.ffi_bridge.create_bindings(&python_parts, &ruchy_lib)?;
        
        // Execute with automatic marshalling
        self.execute_with_marshalling(python_parts, ruchy_lib, bindings)
    }
    
    fn execute_with_marshalling(&self,
                                python: PythonModule,
                                ruchy: NativeLib,
                                bindings: FfiBindings) -> Result<Value> {
        // Install cross-language call trampolines
        self.install_trampolines(&bindings);
        
        // Execute Python with Ruchy functions available
        let py_result = self.python_vm.execute_with_extensions(
            python,
            |name| {
                // Intercept calls to migrated functions
                if let Some(ruchy_fn) = ruchy.get_function(name) {
                    Some(self.create_python_wrapper(ruchy_fn))
                } else {
                    None
                }
            }
        )?;
        
        Ok(self.marshal_result(py_result))
    }
}
```

### 4.2 Zero-Copy Data Exchange

```rust
pub struct ZeroCopyBridge {
    // Memory arena for shared data
    arena: MemoryArena,
    
    // Type layout calculator
    layout_calc: LayoutCalculator,
    
    // Reference tracking
    ref_tracker: ReferenceTracker,
}

impl ZeroCopyBridge {
    pub fn share_data<T>(&self, data: T) -> SharedRef<T> 
    where 
        T: SafeToShare 
    {
        // Allocate in shared arena
        let ptr = self.arena.alloc(data);
        
        // Track reference for GC coordination
        self.ref_tracker.track(ptr);
        
        SharedRef {
            ptr,
            phantom: PhantomData,
        }
    }
    
    pub fn numpy_to_polars(&self, array: PyArray) -> DataFrame {
        unsafe {
            // Zero-copy conversion via Arrow format
            let arrow_array = self.numpy_to_arrow_unchecked(array);
            DataFrame::from_arrow(arrow_array)
        }
    }
    
    unsafe fn numpy_to_arrow_unchecked(&self, array: PyArray) -> ArrowArray {
        // Extract raw pointer and metadata
        let ptr = array.as_ptr();
        let shape = array.shape();
        let dtype = array.dtype();
        
        // Map NumPy dtype to Arrow type
        let arrow_type = self.map_dtype_to_arrow(dtype);
        
        // Create Arrow array without copying
        ArrowArray::from_raw_parts(
            ptr as *const u8,
            shape[0],
            arrow_type,
            self.calculate_null_bitmap(&array),
        )
    }
}
```

## 5. Memory Model Unification

### 5.1 Garbage Collection Bridge

```rust
pub struct GcBridge {
    // Python reference counting
    python_gc: PyGc,
    
    // Ruchy ownership tracking
    ownership_tracker: OwnershipTracker,
    
    // Cycle detector
    cycle_detector: CycleDetector,
}

impl GcBridge {
    pub fn manage_hybrid_object(&self, obj: HybridObject) -> ManagedRef {
        match obj.origin() {
            Origin::Python => {
                // Python object: use reference counting
                self.python_gc.incref(&obj);
                ManagedRef::Python(obj.as_py())
            },
            Origin::Ruchy => {
                // Ruchy object: use ownership rules
                let ownership = self.ownership_tracker.analyze(&obj);
                match ownership {
                    Ownership::Unique => ManagedRef::Owned(obj),
                    Ownership::Shared => ManagedRef::Rc(Rc::new(obj)),
                    Ownership::Borrowed(lt) => ManagedRef::Borrowed(obj, lt),
                }
            },
            Origin::Shared => {
                // Shared object: coordinate both systems
                self.coordinate_gc(obj)
            },
        }
    }
    
    fn coordinate_gc(&self, obj: HybridObject) -> ManagedRef {
        // Register with both GC systems
        let py_ref = self.python_gc.track(&obj);
        let ruchy_ref = self.ownership_tracker.track(&obj);
        
        // Set up finalizer coordination
        self.setup_finalizer_coordination(py_ref, ruchy_ref);
        
        // Detect cycles across language boundary
        if self.cycle_detector.has_cross_language_cycle(&obj) {
            self.break_cycle(&obj);
        }
        
        ManagedRef::Coordinated {
            python: py_ref,
            ruchy: ruchy_ref,
        }
    }
}
```

### 5.2 Memory Layout Optimization

```rust
pub struct LayoutOptimizer {
    // Cache line analyzer
    cache_analyzer: CacheLineAnalyzer,
    
    // SIMD alignment calculator
    simd_aligner: SimdAligner,
    
    // Memory pool allocator
    pool_allocator: PoolAllocator,
}

impl LayoutOptimizer {
    pub fn optimize_struct_layout(&self, py_class: &PyClass) -> RuchyStruct {
        // Analyze field access patterns
        let access_patterns = self.analyze_access_patterns(py_class);
        
        // Group frequently accessed fields
        let field_groups = self.group_by_access_frequency(&access_patterns);
        
        // Optimize for cache lines
        let optimized_layout = field_groups.iter()
            .map(|group| self.pack_for_cache_line(group))
            .collect();
            
        RuchyStruct {
            name: py_class.name.to_ruchy(),
            fields: optimized_layout,
            alignment: self.calculate_optimal_alignment(&optimized_layout),
        }
    }
    
    fn pack_for_cache_line(&self, fields: &[Field]) -> PackedFields {
        // Sort by size for optimal packing
        let mut sorted = fields.to_vec();
        sorted.sort_by_key(|f| std::cmp::Reverse(f.size()));
        
        // Pack into 64-byte cache lines
        let mut packed = PackedFields::new();
        let mut current_line = CacheLine::new();
        
        for field in sorted {
            if current_line.can_fit(&field) {
                current_line.add(field);
            } else {
                packed.push(current_line);
                current_line = CacheLine::new();
                current_line.add(field);
            }
        }
        
        if !current_line.is_empty() {
            packed.push(current_line);
        }
        
        packed
    }
}
```

## 6. Performance Optimization

### 6.1 JIT Compilation Strategy

```rust
pub struct JitStrategy {
    // Profiling-guided JIT
    profiler: RuntimeProfiler,
    
    // Cranelift JIT compiler
    jit_compiler: CraneliftJit,
    
    // Optimization threshold
    threshold: HotPathThreshold,
}

impl JitStrategy {
    pub fn optimize_hot_paths(&self, module: &HybridModule) -> OptimizedModule {
        // Profile execution
        let profile = self.profiler.profile_execution(module, Duration::from_secs(1));
        
        // Identify hot paths
        let hot_paths = profile.paths()
            .filter(|p| p.execution_count() > self.threshold.count())
            .filter(|p| p.time_percentage() > self.threshold.time_pct());
            
        // JIT compile hot Python functions
        let mut optimized = module.clone();
        for path in hot_paths {
            if let Some(py_func) = path.as_python_function() {
                // Transpile to Ruchy for JIT
                let ruchy_func = self.transpile_for_jit(py_func);
                
                // JIT compile
                let native_code = self.jit_compiler.compile(ruchy_func);
                
                // Replace with native version
                optimized.replace_function(py_func.name(), native_code);
            }
        }
        
        optimized
    }
    
    fn transpile_for_jit(&self, py_func: &PyFunction) -> RuchyFunction {
        // Fast-path transpilation for JIT
        let mut transpiler = FastTranspiler::new();
        
        // Skip complex type inference
        transpiler.use_dynamic_types();
        
        // Inline all small functions
        transpiler.set_inline_threshold(100);
        
        // Generate SIMD operations where possible
        transpiler.enable_auto_vectorization();
        
        transpiler.transpile(py_func)
    }
}
```

### 6.2 Vectorization and Parallelization

```rust
pub struct VectorizationOptimizer {
    // Loop analyzer
    loop_analyzer: LoopAnalyzer,
    
    // SIMD code generator
    simd_gen: SimdGenerator,
    
    // Parallelization engine
    parallel_engine: RayonIntegration,
}

impl VectorizationOptimizer {
    pub fn vectorize_numeric_code(&self, ast: &RuchyAst) -> RuchyAst {
        ast.transform_bottom_up(|node| {
            match node {
                // NumPy-style operations → SIMD
                RuchyAst::NumpyOp { op, arrays } => {
                    self.generate_simd_operation(op, arrays)
                },
                
                // Pandas operations → Polars lazy evaluation
                RuchyAst::PandasOp { df, operations } => {
                    self.convert_to_polars_lazy(df, operations)
                },
                
                // Loop vectorization
                RuchyAst::ForLoop { var, iter, body } => {
                    if self.loop_analyzer.is_vectorizable(&body) {
                        self.simd_gen.vectorize_loop(var, iter, body)
                    } else if self.loop_analyzer.is_parallelizable(&body) {
                        self.parallel_engine.parallelize_loop(var, iter, body)
                    } else {
                        node
                    }
                },
                
                _ => node,
            }
        })
    }
    
    fn generate_simd_operation(&self, op: NumpyOp, arrays: Vec<Array>) -> RuchyAst {
        // Determine SIMD width
        let width = self.detect_simd_width();
        
        // Generate SIMD intrinsics
        match op {
            NumpyOp::Add => quote! {
                arrays[0].chunks_exact(#width)
                    .zip(arrays[1].chunks_exact(#width))
                    .map(|(a, b)| {
                        let va = f32x#width::from_slice(a);
                        let vb = f32x#width::from_slice(b);
                        va + vb
                    })
                    .flatten()
                    .collect()
            },
            // ... other operations
        }
    }
}
```

## 7. Quality Enforcement

### 7.1 Migration Quality Gates

```rust
pub struct MigrationQualityGates {
    // PMAT integration
    pmat_client: PmatClient,
    
    // Type coverage analyzer
    type_coverage: TypeCoverageAnalyzer,
    
    // Performance regression detector
    perf_detector: PerformanceRegressionDetector,
}

impl MigrationQualityGates {
    pub fn validate_migration(&self, 
                              python_code: &str, 
                              ruchy_code: &str) -> QualityReport {
        let mut report = QualityReport::new();
        
        // Check semantic equivalence
        let sem_equiv = self.check_semantic_equivalence(python_code, ruchy_code);
        report.add_check("semantic_equivalence", sem_equiv);
        
        // Type coverage must increase
        let py_coverage = self.type_coverage.analyze_python(python_code);
        let ruchy_coverage = self.type_coverage.analyze_ruchy(ruchy_code);
        report.add_check("type_coverage", 
                        ruchy_coverage >= py_coverage * 1.2); // 20% improvement
        
        // Complexity must not increase
        let py_complexity = self.pmat_client.analyze_complexity(python_code);
        let ruchy_complexity = self.pmat_client.analyze_complexity(ruchy_code);
        report.add_check("complexity", 
                        ruchy_complexity.max <= py_complexity.max);
        
        // Performance must improve
        let perf_improvement = self.perf_detector.measure_improvement(
            python_code, 
            ruchy_code
        );
        report.add_check("performance", perf_improvement >= 1.5); // 50% faster
        
        // No new technical debt
        let satd = self.pmat_client.analyze_satd(ruchy_code);
        report.add_check("zero_satd", satd.count == 0);
        
        report
    }
}
```

### 7.2 Property Testing for Migration

```rust
pub struct MigrationPropertyTests {
    // QuickCheck generator
    generator: ArbitraryPythonCode,
    
    // Property verifier
    verifier: PropertyVerifier,
}

impl MigrationPropertyTests {
    pub fn test_migration_properties(&self) {
        // Property 1: Type inference is sound
        quickcheck! {
            fn type_inference_sound(py_code: PythonCode) -> bool {
                let ruchy_code = transpile(py_code);
                let inferred_types = infer_types(&ruchy_code);
                
                // All inferred types must be correct
                verify_types(&ruchy_code, &inferred_types)
            }
        }
        
        // Property 2: Ownership inference is safe
        quickcheck! {
            fn ownership_safe(py_code: PythonCode) -> bool {
                let ruchy_code = transpile_with_ownership(py_code);
                
                // No use-after-free or data races
                !has_memory_safety_issues(&ruchy_code)
            }
        }
        
        // Property 3: Performance never degrades
        quickcheck! {
            fn performance_improves(py_code: PythonCode) -> bool {
                let ruchy_code = transpile_optimized(py_code);
                
                let py_time = benchmark_python(&py_code);
                let ruchy_time = benchmark_ruchy(&ruchy_code);
                
                ruchy_time <= py_time
            }
        }
        
        // Property 4: Semantic preservation
        quickcheck! {
            fn semantics_preserved(py_code: PythonCode, input: TestInput) -> bool {
                let ruchy_code = transpile(py_code);
                
                let py_result = execute_python(&py_code, &input);
                let ruchy_result = execute_ruchy(&ruchy_code, &input);
                
                py_result == ruchy_result
            }
        }
    }
}
```

## 8. Migration Strategies

### 8.1 Incremental Migration Path

```rust
pub struct IncrementalMigrator {
    // Module dependency analyzer
    dep_analyzer: DependencyAnalyzer,
    
    // Migration planner
    planner: MigrationPlanner,
    
    // Risk assessor
    risk_assessor: RiskAssessor,
}

impl IncrementalMigrator {
    pub fn plan_migration(&self, project: &PythonProject) -> MigrationPlan {
        // Analyze dependencies
        let dep_graph = self.dep_analyzer.build_graph(project);
        
        // Identify migration candidates
        let candidates = self.identify_candidates(&dep_graph);
        
        // Order by risk and benefit
        let ordered = self.order_by_roi(candidates);
        
        MigrationPlan {
            phases: ordered.into_iter().map(|module| {
                MigrationPhase {
                    module: module.name,
                    strategy: self.select_strategy(&module),
                    estimated_effort: self.estimate_effort(&module),
                    risk_level: self.risk_assessor.assess(&module),
                    fallback_plan: self.create_fallback(&module),
                }
            }).collect(),
        }
    }
    
    fn select_strategy(&self, module: &Module) -> MigrationStrategy {
        match (module.complexity(), module.criticality()) {
            (Low, Low) => MigrationStrategy::FullAutomation,
            (Low, High) => MigrationStrategy::AutomatedWithReview,
            (High, Low) => MigrationStrategy::SemiAutomated,
            (High, High) => MigrationStrategy::ManualWithAssist,
        }
    }
}
```

### 8.2 Performance Profiling Integration

```rust
pub struct MigrationProfiler {
    // Python profiler
    py_profiler: PyFlame,
    
    // Ruchy profiler
    ruchy_profiler: Perf,
    
    // Comparison engine
    comparator: ProfileComparator,
}

impl MigrationProfiler {
    pub fn profile_migration(&self, 
                             py_module: &PyModule,
                             ruchy_module: &RuchyModule,
                             workload: &Workload) -> ProfileComparison {
        // Profile Python execution
        let py_profile = self.py_profiler.profile(py_module, workload);
        
        // Profile Ruchy execution
        let ruchy_profile = self.ruchy_profiler.profile(ruchy_module, workload);
        
        // Generate comparison
        ProfileComparison {
            speedup: ruchy_profile.total_time / py_profile.total_time,
            memory_reduction: py_profile.peak_memory / ruchy_profile.peak_memory,
            
            hot_path_improvements: self.compare_hot_paths(
                &py_profile.hot_paths,
                &ruchy_profile.hot_paths
            ),
            
            bottleneck_analysis: self.identify_remaining_bottlenecks(
                &ruchy_profile
            ),
            
            optimization_suggestions: self.suggest_optimizations(
                &py_profile,
                &ruchy_profile
            ),
        }
    }
}
```

## Implementation Timeline

### Phase 1: Foundation (Weeks 1-2)
- [ ] Unified HIR design
- [ ] Basic type translation
- [ ] Simple transpilation

### Phase 2: Type System (Weeks 3-4)
- [ ] Bidirectional type mapping
- [ ] Ownership inference
- [ ] Refinement synthesis

### Phase 3: Optimization (Weeks 5-6)
- [ ] Pattern-based transformation
- [ ] Vectorization
- [ ] JIT integration

### Phase 4: Runtime (Weeks 7-8)
- [ ] Hybrid execution
- [ ] Zero-copy bridge
- [ ] GC coordination

### Phase 5: Quality (Weeks 9-10)
- [ ] Migration validation
- [ ] Property testing
- [ ] Performance profiling

## Performance Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| Transpilation Speed | >100K LOC/sec | CI/CD feasibility |
| Type Inference Coverage | >80% | Minimize manual annotation |
| Performance Improvement | >2x vs CPython | Justify migration effort |
| Memory Reduction | >50% | Ownership elimination |
| Migration Accuracy | >99% | Semantic preservation |
| Zero-Copy Overhead | <100ns | Native interop speed |

## Summary

This specification defines a comprehensive integration between Depyler and Ruchy, enabling seamless Python-to-Ruchy migration with guaranteed performance improvements and type safety. The architecture prioritizes incremental migration, zero-copy interoperability, and automatic optimization while maintaining semantic equivalence through property-based testing.