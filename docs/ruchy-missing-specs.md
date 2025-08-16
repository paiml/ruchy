# Ruchy Missing Specifications - Critical Components

## Acknowledgments

This specification incorporates refinements based on architectural review feedback. The design prioritizes production readiness over theoretical elegance, focusing on components that directly impact developer experience and runtime performance.

## 1. Error Diagnostics Architecture

### 1.1 Diagnostic Rendering Pipeline

```rust
pub struct Diagnostic {
    level: Level,
    code: ErrorCode,
    primary: Label,
    secondary: Vec<Label>,
    notes: Vec<Note>,
    suggestions: Vec<CodeFix>,
}

pub struct Label {
    span: Span,
    message: String,
    style: LabelStyle, // Primary, Secondary, Tertiary
}

pub struct CodeFix {
    span: Span,
    replacement: String,
    applicability: Applicability, // MachineApplicable, MaybeIncorrect, HasPlaceholders
}

impl DiagnosticRenderer {
    fn render(&self, diag: &Diagnostic, source: &SourceMap) -> String {
        // 1. Extract source context (3 lines before/after)
        // 2. Apply syntax highlighting to source
        // 3. Underline error spans with ^^^^ or ----
        // 4. Number suggestions for --fix flag
        // 5. Generate terminal colors via termcolor crate
    }
}
```

### 1.2 Error Code System

```rust
// Error codes: XNNNN where X = category, NNNN = unique ID
pub enum ErrorCode {
    // Lexer errors: L0001-L0999
    L0001, // UnterminatedString
    L0002, // InvalidEscape
    L0003, // InvalidNumericLiteral
    
    // Parser errors: P1000-P1999
    P1000, // UnexpectedToken
    P1001, // MissingClosingDelimiter
    P1002, // InvalidOperatorChain
    
    // Type errors: T2000-T2999
    T2000, // TypeMismatch
    T2001, // UnresolvedType
    T2002, // InfiniteType
    
    // Borrow checker: B3000-B3999
    B3000, // UseAfterMove
    B3001, // MultipleOwnership
    B3002, // DanglingReference
    
    // Runtime errors: R4000-R4999
    R4000, // ActorPanic
    R4001, // StackOverflow
    R4002, // OutOfMemory
}

impl ErrorCode {
    fn url(&self) -> String {
        format!("https://ruchy.dev/errors/{}", self)
    }
    
    fn default_explanation(&self) -> &'static str {
        ERROR_EXPLANATIONS[*self as usize]
    }
}
```

### 1.3 Diagnostic Accumulator

```rust
pub struct DiagnosticBag {
    diagnostics: Vec<Diagnostic>,
    emitted: HashSet<(Span, ErrorCode)>, // Deduplication
    suppressed: Vec<Pattern>,             // Suppression rules
    fatals: Vec<Diagnostic>,              // Compilation-stopping errors
}

impl DiagnosticBag {
    fn emit(&mut self, diag: Diagnostic) {
        let key = (diag.primary.span, diag.code);
        if !self.emitted.contains(&key) {
            self.emitted.insert(key);
            
            // Group related diagnostics
            if let Some(parent) = self.find_related(&diag) {
                parent.add_note(diag.into_note());
            } else {
                self.diagnostics.push(diag);
            }
        }
    }
    
    fn report_phase(&mut self, phase: CompilerPhase) {
        // Sort by file, then line, then column
        self.diagnostics.sort_by_key(|d| d.primary.span);
        
        // Limit to 20 errors per phase
        for diag in self.diagnostics.drain(..).take(20) {
            eprintln!("{}", self.renderer.render(&diag));
        }
    }
}
```

## 2. Module System and Resolution

### 2.1 Module Dependency Graph

```rust
pub struct ModuleGraph {
    modules: HashMap<ModuleId, ModuleNode>,
    edges: petgraph::Graph<ModuleId, DependencyKind>,
    sccs: Vec<Vec<ModuleId>>, // Strongly connected components
}

pub struct ModuleNode {
    id: ModuleId,
    source: PathBuf,
    signature: ModuleSignature,
    dependencies: Vec<Dependency>,
    last_modified: SystemTime,
}

pub enum DependencyKind {
    Import,      // Normal import
    ReExport,    // pub use
    Circular,    // Within SCC
    Dev,         // Test-only
}

impl ModuleGraph {
    fn detect_cycles(&self) -> Vec<Vec<ModuleId>> {
        tarjan_scc(&self.edges)
    }
    
    fn compilation_order(&self) -> Result<Vec<ModuleId>> {
        // Topological sort with SCC handling
        let sccs = self.detect_cycles();
        for scc in &sccs {
            if scc.len() > 1 {
                return Err(CycleError(scc.clone()));
            }
        }
        Ok(toposort(&self.edges)?)
    }
}
```

### 2.2 Cross-Module Type Checking

```rust
pub struct ModuleSignature {
    exported_types: HashMap<Name, TypeScheme>,
    exported_values: HashMap<Name, TypeScheme>,
    exported_modules: HashMap<Name, ModuleId>,
    type_dependencies: HashSet<ModuleId>,
}

impl TypeChecker {
    fn check_cross_module(&mut self, import: &Import) -> Result<()> {
        let sig = self.module_cache.get_signature(import.module)?;
        
        // Validate imported names exist
        for name in &import.names {
            if !sig.exports(name) {
                return Err(UnresolvedImport(name));
            }
        }
        
        // Register types in local environment
        for (name, scheme) in sig.exported_types {
            self.env.insert_type(import.qualify(name), scheme);
        }
        
        Ok(())
    }
}
```

### 2.3 Symbol Visibility Rules

```rust
pub enum Visibility {
    Private,                    // Module-local
    Public,                     // Exported
    PublicIn(ModulePath),      // Restricted export
    Inherited,                  // From parent module
}

impl Resolver {
    fn resolve_visibility(&self, item: &Item, from: ModuleId) -> bool {
        match item.visibility {
            Private => from == item.defining_module,
            Public => true,
            PublicIn(ref path) => from.is_descendant_of(path),
            Inherited => self.resolve_visibility(item.parent?, from),
        }
    }
}
```

## 3. Memory Management Details

### 3.1 Region Inference Algorithm

```rust
pub struct RegionInference {
    constraints: Vec<RegionConstraint>,
    region_graph: DiGraph<RegionId, ConstraintKind>,
    solutions: HashMap<RegionId, Region>,
}

pub enum Region {
    Static,                    // 'static lifetime
    Stack(ScopeId),           // Stack-allocated
    Arena(ArenaId),          // Arena-allocated
    Heap(RefCount),           // Reference-counted
}

impl RegionInference {
    fn solve(&mut self) -> Result<RegionSolution> {
        // Algorithm:
        // 1. Build constraint graph from type annotations
        // 2. Find SCCs in constraint graph
        // 3. Solve SCCs bottom-up
        // 4. Propagate solutions
        
        loop {
            let changed = self.propagate_constraints()?;
            if !changed {
                break;
            }
        }
        
        self.validate_solutions()
    }
    
    fn propagate_constraints(&mut self) -> Result<bool> {
        let mut changed = false;
        
        for constraint in &self.constraints {
            match constraint {
                Outlives(r1, r2) => {
                    let s1 = self.solutions[r1];
                    let s2 = self.solutions[r2];
                    if !s1.outlives(s2) {
                        self.solutions[r1] = s1.join(s2);
                        changed = true;
                    }
                }
                Escapes(r, scope) => {
                    if self.solutions[r].escapes(scope) {
                        self.promote_to_heap(r)?;
                        changed = true;
                    }
                }
            }
        }
        
        Ok(changed)
    }
}
```

### 3.2 Lifetime Elision Rules

```rust
pub struct LifetimeElision {
    rules: Vec<ElisionRule>,
}

impl LifetimeElision {
    fn apply(&self, sig: &FunctionSignature) -> FunctionSignature {
        // Rule 1: Each input reference gets distinct lifetime
        // Rule 2: If one input lifetime, output gets same lifetime
        // Rule 3: If &self or &mut self, output gets self lifetime
        // Rule 4: In closures, capture lifetimes unify with body
        
        match (sig.inputs.len(), sig.has_self_param()) {
            (1, false) => {
                // Single input: output lifetime = input lifetime
                let lifetime = self.fresh_lifetime();
                sig.map_lifetimes(|_| lifetime)
            }
            (_, true) => {
                // Method: output lifetime = self lifetime
                sig.map_output_lifetime(sig.self_lifetime())
            }
            _ => {
                // Multiple inputs: each gets fresh lifetime
                sig.map_each_lifetime(|_| self.fresh_lifetime())
            }
        }
    }
}
```

### 3.3 Arena Allocation Boundaries

```rust
pub struct ArenaManager {
    arenas: Vec<Arena>,
    current: ArenaId,
    thresholds: ArenaThresholds,
}

pub struct ArenaThresholds {
    max_size: usize,        // 16MB default
    collection_ratio: f32,  // 0.8 = collect at 80% full
    promotion_count: u32,   // Promote after N collections
}

// Implementation note: Start with simple bump allocation
// Add generational collection only after proving necessity via benchmarks

impl ArenaManager {
    fn allocate(&mut self, size: usize, lifetime: Lifetime) -> *mut u8 {
        if self.current_arena().available() < size {
            self.trigger_collection();
        }
        
        if lifetime.escapes_arena() {
            // Promote to heap immediately
            return self.heap_allocate(size);
        }
        
        self.current_arena().bump_allocate(size)
    }
    
    fn trigger_collection(&mut self) {
        // Mark phase: trace from roots
        let reachable = self.mark_reachable();
        
        // Compact phase: move survivors to new arena
        let new_arena = Arena::new(self.thresholds.max_size);
        for obj in reachable {
            new_arena.evacuate(obj);
        }
        
        // Update references
        self.update_references(&new_arena);
        
        self.arenas.push(new_arena);
        self.current = self.arenas.len() - 1;
    }
}
```

## 4. FFI Specification

### 4.1 ABI Compatibility Layer

```rust
pub enum AbiKind {
    Rust,        // Rust calling convention
    C,           // System C ABI
    Stdcall,     // Windows stdcall
    Fastcall,    // x64 fastcall
    System,      // Platform default
}

pub struct FfiSignature {
    abi: AbiKind,
    params: Vec<FfiType>,
    returns: FfiType,
    variadic: bool,
}

impl FfiMarshaller {
    fn marshal_call(&self, sig: &FfiSignature, args: Vec<Value>) -> Result<Value> {
        // 1. Validate argument count (unless variadic)
        // 2. Convert Ruchy types to FFI types
        // 3. Allocate stack/registers per ABI
        // 4. Perform call
        // 5. Marshal return value back
        
        let mut marshalled = Vec::new();
        for (arg, ffi_type) in args.iter().zip(&sig.params) {
            marshalled.push(self.marshal_arg(arg, ffi_type)?);
        }
        
        unsafe {
            let result = match sig.abi {
                AbiKind::C => self.call_c(sig, marshalled),
                AbiKind::Rust => self.call_rust(sig, marshalled),
                _ => unimplemented!(),
            };
            
            self.unmarshal_return(result, &sig.returns)
        }
    }
}
```

### 4.2 Type Marshalling Rules

```rust
pub trait FfiCompatible {
    fn to_ffi(&self) -> FfiValue;
    fn from_ffi(val: FfiValue) -> Result<Self>;
}

impl FfiCompatible for String {
    fn to_ffi(&self) -> FfiValue {
        // Convert to null-terminated C string
        let c_str = CString::new(self.as_bytes()).unwrap();
        FfiValue::Pointer(c_str.into_raw() as *mut u8)
    }
    
    fn from_ffi(val: FfiValue) -> Result<Self> {
        match val {
            FfiValue::Pointer(ptr) => unsafe {
                let c_str = CStr::from_ptr(ptr as *const i8);
                Ok(c_str.to_string_lossy().into_owned())
            },
            _ => Err(FfiError::TypeMismatch),
        }
    }
}

// Complex type marshalling
impl FfiCompatible for Enum {
    fn to_ffi(&self) -> FfiValue {
        // Tagged union representation
        FfiValue::Struct(vec![
            FfiValue::U32(self.discriminant()),
            self.payload_to_ffi(),
        ])
    }
}
```

### 4.3 Ownership Transfer Protocol

```rust
pub enum OwnershipTransfer {
    Borrow,      // Caller retains ownership
    Move,        // Ownership transfers to callee
    Clone,       // Deep copy
    Shared,      // Reference counted
}

impl FfiFunction {
    fn transfer_ownership(&self, param: &Param) -> OwnershipTransfer {
        match param.annotation {
            #[ffi(borrow)] => Borrow,
            #[ffi(consume)] => Move,
            #[ffi(clone)] => Clone,
            _ => self.infer_transfer(param.ty),
        }
    }
    
    fn infer_transfer(&self, ty: &Type) -> OwnershipTransfer {
        match ty {
            Type::Ref(_) => Borrow,
            Type::Box(_) => Move,
            Type::Rc(_) => Shared,
            _ if ty.is_copy() => Clone,
            _ => Move,
        }
    }
}
```

## 5. Optimization Pipeline

### 5.1 MIR Specification

```rust
pub enum MirInst {
    // SSA operations
    Assign(Local, Rvalue),
    Load(Local, Place),
    Store(Place, Local),
    
    // Control flow
    Goto(Block),
    If(Local, Block, Block),
    Switch(Local, Vec<(Const, Block)>, Block),
    
    // Function calls
    Call(Local, Func, Vec<Local>),
    TailCall(Func, Vec<Local>),
    
    // Memory operations
    Alloc(Local, Size, Align),
    Free(Local),
    
    // Arithmetic (typed)
    BinOp(Local, BinOp, Local, Local),
    UnOp(Local, UnOp, Local),
    
    // Casts
    Cast(Local, Local, CastKind),
}

pub struct MirFunction {
    locals: Vec<LocalDecl>,
    blocks: Vec<BasicBlock>,
    dominators: DominatorTree,
    loops: LoopInfo,
}

impl MirBuilder {
    fn lower_ast(&mut self, ast: &TypedAst) -> MirFunction {
        // 1. Convert to SSA form
        // 2. Build CFG
        // 3. Compute dominators
        // 4. Identify loops
        // 5. Insert phi nodes
        
        let mut ssa = SsaBuilder::new();
        for stmt in ast.statements {
            self.lower_statement(&mut ssa, stmt);
        }
        ssa.seal_blocks();
        ssa.into_mir()
    }
}
```

### 5.2 Optimization Pass Ordering

```rust
pub struct OptimizationPipeline {
    passes: Vec<Box<dyn MirPass>>,
}

impl OptimizationPipeline {
    fn default() -> Self {
        Self {
            passes: vec![
                // Phase 1: Cleanup
                Box::new(SimplifyCfg),
                Box::new(RemoveDeadCode),
                
                // Phase 2: Analysis
                Box::new(ComputeAliasInfo),
                Box::new(ComputeLoopInfo),
                
                // Phase 3: High-level optimizations
                Box::new(InlineSmallFunctions),
                Box::new(ConstantPropagation),
                Box::new(CommonSubexprElimination),
                
                // Phase 4: Loop optimizations
                Box::new(LoopInvariantCodeMotion),
                Box::new(LoopUnrolling),
                Box::new(LoopVectorization),
                
                // Phase 5: Low-level optimizations
                Box::new(RegisterAllocation),
                Box::new(InstructionScheduling),
                
                // Phase 6: Final cleanup
                Box::new(SimplifyCfg),
                Box::new(RemoveDeadCode),
            ],
        }
    }
    
    fn optimize(&self, mir: &mut MirFunction, level: OptLevel) {
        for pass in &self.passes {
            if pass.should_run(level) {
                pass.run(mir);
                mir.verify_invariants();
            }
        }
    }
}
```

### 5.3 Inlining Heuristics

```rust
pub struct InliningDecision {
    cost_model: CostModel,
    thresholds: InlineThresholds,
}

pub struct InlineThresholds {
    max_size: usize,           // 100 MIR instructions
    max_depth: usize,          // 5 levels
    growth_factor: f32,        // 1.5x code size growth
    hotness_threshold: u32,    // 1000 calls
}

impl InliningDecision {
    fn should_inline(&self, caller: &MirFunction, callee: &MirFunction, 
                     call_site: &CallSite) -> bool {
        // Never inline recursive functions
        if self.is_recursive(callee) {
            return false;
        }
        
        let cost = self.cost_model.estimate(callee);
        let benefit = self.estimate_benefit(call_site);
        
        // Always inline trivial functions
        if cost < 5 {
            return true;
        }
        
        // Profile-guided decision
        if let Some(profile) = call_site.profile_data() {
            if profile.call_count > self.thresholds.hotness_threshold {
                return cost < self.thresholds.max_size * 2;
            }
        }
        
        // Heuristic: benefit/cost ratio
        benefit > cost * 1.5
    }
    
    fn estimate_benefit(&self, call_site: &CallSite) -> f32 {
        let mut benefit = 10.0; // Base call overhead
        
        // Constant arguments enable further optimization
        benefit += call_site.const_args().len() as f32 * 5.0;
        
        // Loop context multiplier
        if call_site.in_loop() {
            benefit *= 10.0;
        }
        
        benefit
    }
}
```

## 6. Runtime Components

### 6.1 Actor Mailbox Implementation

```rust
pub struct Mailbox {
    queue: crossbeam::deque::Worker<Message>,
    stealers: Vec<crossbeam::deque::Stealer<Message>>,
    backpressure: BackpressureStrategy,
    capacity: usize,
}

pub enum BackpressureStrategy {
    Drop,              // Drop new messages
    Block,             // Block sender
    Overflow(BoxId),   // Redirect to overflow actor
    Sampling(f32),     // Random sampling
}

impl Mailbox {
    fn send(&self, msg: Message) -> Result<()> {
        if self.queue.len() >= self.capacity {
            match self.backpressure {
                Drop => return Err(MailboxFull),
                Block => self.wait_for_space(),
                Overflow(actor) => return actor.send(msg),
                Sampling(rate) => {
                    if rand::random::<f32>() > rate {
                        return Err(MailboxFull);
                    }
                }
            }
        }
        
        self.queue.push(msg);
        Ok(())
    }
    
    fn receive(&self, timeout: Duration) -> Option<Message> {
        // Try local queue first
        if let Some(msg) = self.queue.pop() {
            return Some(msg);
        }
        
        // Work stealing from other actors
        for stealer in &self.stealers {
            if let Some(msg) = stealer.steal().success() {
                return Some(msg);
            }
        }
        
        // Block with timeout
        self.wait_for_message(timeout)
    }
}
```

### 6.2 Supervision Tree Protocol

```rust
pub enum SupervisorStrategy {
    OneForOne,      // Restart failed child only
    OneForAll,      // Restart all children
    RestForOne,     // Restart failed child and those started after
    Simple,         // No restart
}

pub struct Supervisor {
    strategy: SupervisorStrategy,
    children: Vec<ActorRef>,
    restart_intensity: (u32, Duration), // (max_restarts, within_period)
    restart_count: HashMap<ActorId, Vec<Instant>>,
}

impl Supervisor {
    fn handle_failure(&mut self, failed: ActorId, error: ActorError) {
        // Check restart intensity
        if !self.should_restart(failed) {
            self.escalate(error);
            return;
        }
        
        match self.strategy {
            OneForOne => {
                self.restart_child(failed);
            }
            OneForAll => {
                for child in &self.children {
                    self.stop_child(child.id);
                }
                for child in &self.children {
                    self.restart_child(child.id);
                }
            }
            RestForOne => {
                let failed_index = self.child_index(failed);
                for i in failed_index..self.children.len() {
                    self.restart_child(self.children[i].id);
                }
            }
            Simple => {
                self.remove_child(failed);
            }
        }
    }
    
    fn should_restart(&mut self, actor: ActorId) -> bool {
        let now = Instant::now();
        let restarts = self.restart_count.entry(actor).or_default();
        
        // Remove old restarts outside window
        let (max_restarts, period) = self.restart_intensity;
        restarts.retain(|&t| now.duration_since(t) < period);
        
        if restarts.len() >= max_restarts as usize {
            false
        } else {
            restarts.push(now);
            true
        }
    }
}
```

## 7. Build System Integration

### 7.1 Incremental Compilation Strategy

```rust
pub struct IncrementalCompiler {
    dependency_graph: HashMap<ModuleId, HashSet<ModuleId>>,
    fingerprints: HashMap<ModuleId, Fingerprint>,
    query_cache: QueryCache,
}

pub struct Fingerprint {
    source_hash: u64,
    interface_hash: u64,
    dependencies_hash: u64,
}

impl IncrementalCompiler {
    fn compile_incremental(&mut self, changed: Vec<ModuleId>) -> Result<()> {
        // 1. Compute reverse dependencies
        let mut to_recompile = HashSet::new();
        let mut queue = VecDeque::from(changed);
        
        while let Some(module) = queue.pop_front() {
            if to_recompile.insert(module) {
                // Add modules that depend on this one
                for dependent in self.reverse_deps(module) {
                    queue.push_back(dependent);
                }
            }
        }
        
        // 2. Check fingerprints to minimize recompilation
        to_recompile.retain(|&module| {
            let old_fp = self.fingerprints.get(&module);
            let new_fp = self.compute_fingerprint(module);
            
            // Only recompile if interface changed
            old_fp != Some(&new_fp) || new_fp.interface_changed()
        });
        
        // 3. Parallel compilation of independent modules
        let order = self.topological_sort(to_recompile)?;
        
        rayon::scope(|s| {
            for batch in order.batches() {
                for module in batch {
                    s.spawn(|_| self.compile_module(module));
                }
            }
        });
        
        Ok(())
    }
}
```

### 7.2 Build Cache Invalidation

```rust
pub struct BuildCache {
    artifacts: HashMap<CacheKey, CachedArtifact>,
    invalidation_rules: Vec<InvalidationRule>,
}

pub struct CacheKey {
    module: ModuleId,
    config: BuildConfig,
    dependencies: Vec<Fingerprint>,
}

pub enum InvalidationRule {
    SourceChanged,
    ConfigChanged,
    DependencyChanged,
    TimeBasedExpiry(Duration),
    SizeLimit(usize),
}

impl BuildCache {
    fn get(&self, key: &CacheKey) -> Option<&CachedArtifact> {
        let artifact = self.artifacts.get(key)?;
        
        // Check invalidation rules
        for rule in &self.invalidation_rules {
            if rule.should_invalidate(artifact) {
                return None;
            }
        }
        
        Some(artifact)
    }
    
    fn put(&mut self, key: CacheKey, artifact: CachedArtifact) {
        // Enforce size limit via LRU
        while self.total_size() > self.max_size {
            self.evict_lru();
        }
        
        self.artifacts.insert(key, artifact);
    }
}
```

## 8. Debugging Infrastructure

### 8.1 Debug Info Generation

```rust
pub struct DebugInfoBuilder {
    dwarf: gimli::write::Dwarf,
    source_map: SourceMap,
}

impl DebugInfoBuilder {
    fn generate(&mut self, mir: &MirFunction) -> DebugInfo {
        // Generate DWARF debug info
        let unit = self.dwarf.units.get_mut(self.unit_id);
        
        // Add function DIE
        let func_die = unit.root().add_child(gimli::DW_TAG_subprogram);
        func_die.set_name(mir.name.as_bytes());
        func_die.set_low_pc(mir.start_addr);
        func_die.set_high_pc(mir.end_addr);
        
        // Add variable DIEs
        for local in &mir.locals {
            let var_die = func_die.add_child(gimli::DW_TAG_variable);
            var_die.set_name(local.name.as_bytes());
            var_die.set_type(self.type_die(local.ty));
            var_die.set_location(self.location_expr(local));
        }
        
        // Line number program
        let line_program = &mut unit.line_program;
        for (addr, loc) in mir.source_locations() {
            line_program.add_row(addr, loc.line, loc.column);
        }
        
        self.dwarf.write()
    }
    
    fn location_expr(&self, local: &Local) -> Expression {
        // Generate DWARF expression for variable location
        match local.storage {
            Storage::Register(reg) => Expression::Register(reg),
            Storage::Stack(offset) => Expression::FrameOffset(offset),
            Storage::Memory(addr) => Expression::Address(addr),
        }
    }
}
```

### 8.2 Breakpoint Mapping

```rust
pub struct BreakpointMapper {
    ruchy_to_rust: HashMap<SourceLoc, SourceLoc>,
    rust_to_machine: HashMap<SourceLoc, Address>,
}

impl BreakpointMapper {
    fn set_breakpoint(&mut self, ruchy_loc: SourceLoc) -> Result<Breakpoint> {
        // Map Ruchy source location to generated Rust
        let rust_loc = self.ruchy_to_rust.get(&ruchy_loc)
            .ok_or(UnmappedLocation)?;
        
        // Map Rust location to machine code address
        let addr = self.rust_to_machine.get(&rust_loc)
            .ok_or(NoDebugInfo)?;
        
        // Insert INT3 instruction (x86) or equivalent
        unsafe {
            let old_byte = *(addr as *const u8);
            *(addr as *mut u8) = 0xCC; // INT3
            
            Ok(Breakpoint {
                id: BreakpointId::new(),
                address: addr,
                original: old_byte,
                ruchy_loc,
            })
        }
    }
    
    fn handle_breakpoint(&self, bp: &Breakpoint) -> DebugEvent {
        DebugEvent::Breakpoint {
            id: bp.id,
            location: bp.ruchy_loc,
            thread: current_thread_id(),
            stack_trace: self.capture_stack_trace(),
        }
    }
}
```

## 9. Standard Library Design

### 9.1 Collection Hierarchy

```rust
// Optimized for functional patterns
pub trait Collection<T> {
    type Iter: Iterator<Item = T>;
    
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }
    fn iter(&self) -> Self::Iter;
}

pub trait PersistentCollection<T>: Collection<T> {
    fn add(&self, item: T) -> Self;
    fn remove(&self, item: &T) -> Self;
}

// Persistent vector with RRB-tree
pub struct Vector<T> {
    root: Arc<Node<T>>,
    tail: Arc<[T; 32]>,
    len: usize,
    shift: u8,
}

impl<T: Clone> Vector<T> {
    pub fn push(&self, item: T) -> Self {
        if self.tail.len() < 32 {
            // Fast path: modify tail
            let mut new_tail = (*self.tail).clone();
            new_tail[self.tail.len()] = item;
            Vector {
                tail: Arc::new(new_tail),
                len: self.len + 1,
                ..*self
            }
        } else {
            // Slow path: rebalance tree
            self.push_tail(item)
        }
    }
}

// Persistent hash map with HAMT
pub struct HashMap<K, V> {
    root: Arc<HamtNode<K, V>>,
    len: usize,
}
```

### 9.2 I/O Abstraction Layer

```rust
pub trait AsyncRead {
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    
    async fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let mut pos = 0;
        while pos < buf.len() {
            match self.read(&mut buf[pos..]).await {
                Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
                Ok(n) => pos += n,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

pub struct BufferedReader<R> {
    inner: R,
    buffer: Vec<u8>,
    pos: usize,
    cap: usize,
}

impl<R: AsyncRead> BufferedReader<R> {
    async fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.pos >= self.cap {
            self.cap = self.inner.read(&mut self.buffer).await?;
            self.pos = 0;
        }
        Ok(&self.buffer[self.pos..self.cap])
    }
}
```

## 10. Formal Semantics

### 10.1 Operational Semantics

```
// Small-step operational semantics for core language

// Values
v ::= n | b | λx.e | ()

// Evaluation contexts
E ::= [] | E e | v E | if E then e else e

// Reduction rules
(λx.e) v → e[x ↦ v]                    (β-reduction)
if true then e₁ else e₂ → e₁           (if-true)
if false then e₁ else e₂ → e₂          (if-false)

// Context rule
e → e'
─────────────
E[e] → E[e']
```

### 10.2 Type Soundness

```
// Progress: Well-typed terms don't get stuck
Theorem (Progress):
  If ⊢ e : τ then either e is a value or ∃e'. e → e'

// Preservation: Types are preserved by reduction  
Theorem (Preservation):
  If ⊢ e : τ and e → e' then ⊢ e' : τ

// Type Safety = Progress + Preservation
Corollary (Type Safety):
  If ⊢ e : τ then either e →* v where ⊢ v : τ
  or e diverges
```

### 10.3 Memory Safety Invariants

```rust
// Formal invariants maintained by type system

pub enum Invariant {
    // No use after free
    NoUseAfterFree: ∀ ptr. ptr ∈ freed ⟹ ptr ∉ accessible,
    
    // No double free
    NoDoubleFree: ∀ ptr. free(ptr) called at most once,
    
    // No data races
    NoDataRaces: ∀ location. at_most_one_writer(location) ∧ 
                 (has_writer(location) ⟹ no_readers(location)),
    
    // Well-bracketed lifetimes
    WellBracketed: ∀ 'a. region_start('a) < region_end('a),
    
    // No dangling references
    NoDangling: ∀ ref. pointee(ref) ∈ live_objects,
}

// Proof sketch: Type system enforces invariants
impl TypeSystem {
    fn maintains_invariants(&self) -> Proof {
        // 1. Linear types prevent aliasing
        // 2. Lifetime bounds prevent dangling
        // 3. Borrow checker prevents data races
        // 4. RAII ensures cleanup
        proof::by_structural_induction()
    }
}
```

## 11. Language Server Protocol Details

### 11.1 Incremental Parsing Strategy

```rust
pub struct IncrementalParser {
    tree: rowan::SyntaxTree,
    cache: HashMap<NodeId, ParseResult>,
}

impl IncrementalParser {
    fn apply_edit(&mut self, edit: TextEdit) -> Vec<Diagnostic> {
        // Find minimal affected subtree
        let affected = self.find_affected_node(edit.range);
        
        // Reparse only affected portion
        let new_text = self.apply_edit_to_text(edit);
        let new_subtree = self.parse_node(new_text, affected.kind());
        
        // Splice new subtree into existing tree
        self.tree = self.tree.replace_node(affected, new_subtree);
        
        // Incremental type checking
        self.typecheck_incremental(affected)
    }
    
    fn find_affected_node(&self, range: Range) -> NodeId {
        // Walk tree to find smallest node containing range
        let mut current = self.tree.root();
        
        loop {
            let child = current.children()
                .find(|c| c.range().contains(range));
            
            match child {
                Some(c) => current = c,
                None => return current.id(),
            }
        }
    }
}
```

### 11.2 Semantic Token Classification

```rust
pub enum SemanticTokenType {
    // Types
    Type,
    Struct,
    Enum,
    Interface,
    TypeParameter,
    
    // Functions
    Function,
    Method,
    Macro,
    
    // Variables
    Variable,
    Parameter,
    Property,
    EnumMember,
    
    // Keywords
    Keyword,
    Modifier,
    
    // Literals
    Number,
    String,
    Regexp,
    
    // Comments
    Comment,
    Documentation,
}

impl SemanticAnalyzer {
    fn classify_token(&self, token: &Token) -> SemanticTokenType {
        match self.resolve_symbol(token) {
            Symbol::Type(_) => SemanticTokenType::Type,
            Symbol::Function(f) if f.is_macro => SemanticTokenType::Macro,
            Symbol::Function(_) => SemanticTokenType::Function,
            Symbol::Variable(v) if v.is_mutable => {
                SemanticTokenType::Variable | SemanticTokenModifier::Mutable
            }
            Symbol::Variable(_) => SemanticTokenType::Variable,
            _ => self.classify_by_syntax(token),
        }
    }
}
```

## 12. Macro System Design

### 12.1 Hygiene Rules

```rust
pub struct HygieneContext {
    scopes: Vec<SyntaxScope>,
    marks: HashMap<Ident, SyntaxMark>,
}

pub struct SyntaxScope {
    id: ScopeId,
    parent: Option<ScopeId>,
    bindings: HashSet<Ident>,
}

impl MacroExpander {
    fn expand_with_hygiene(&mut self, macro_call: &MacroCall) -> Expansion {
        // Create fresh scope for expansion
        let expansion_scope = self.hygiene.new_scope();
        
        // Mark all identifiers in expansion
        let expanded = self.expand_raw(macro_call);
        let marked = self.mark_identifiers(expanded, expansion_scope);
        
        // Resolve according to hygiene rules
        Expansion {
            ast: marked,
            scope: expansion_scope,
            rename_map: self.compute_renames(marked),
        }
    }
    
    fn resolve_ident(&self, ident: Ident) -> Resolution {
        // Hygienic resolution: respect lexical scope
        let mark = self.hygiene.marks[&ident];
        let scope = self.hygiene.scope_of_mark(mark);
        
        // Search from definition scope, not use scope
        self.resolve_in_scope(ident, scope)
    }
}
```

### 12.2 Compile-Time Evaluation

```rust
pub struct CompileTimeEvaluator {
    heap: Vec<ConstValue>,
    fuel: usize, // Prevent infinite loops
}

impl CompileTimeEvaluator {
    fn eval_const_expr(&mut self, expr: &Expr) -> Result<ConstValue> {
        if self.fuel == 0 {
            return Err(EvalError::OutOfFuel);
        }
        self.fuel -= 1;
        
        match expr {
            Expr::Literal(lit) => Ok(self.eval_literal(lit)),
            Expr::BinOp(op, left, right) => {
                let l = self.eval_const_expr(left)?;
                let r = self.eval_const_expr(right)?;
                self.eval_binop(op, l, r)
            }
            Expr::Call(f, args) if f.is_const_fn() => {
                let arg_vals = args.iter()
                    .map(|a| self.eval_const_expr(a))
                    .collect::<Result<Vec<_>>>()?;
                self.eval_const_fn(f, arg_vals)
            }
            _ => Err(EvalError::NotConstant),
        }
    }
}
```

## Implementation Priority

### Phase 0: Foundation (Weeks 1-4)
1. **Error diagnostics pipeline** - Block everything else
2. **Module dependency graph** - Enable parallel work
3. **Basic Rust FFI** - Validate interop strategy

### Phase 1: Core Runtime (Weeks 5-12)
1. **Simple arena allocator** - Start minimal, measure first
2. **Actor mailbox v1** - Single-threaded prototype
3. **Debug symbol generation** - Essential for development
4. **Incremental parsing** - LSP responsiveness

### Phase 2: Optimization (Weeks 13-20)  
1. **MIR + basic passes** - Const propagation, DCE only
2. **Region inference v1** - Stack vs heap decision only
3. **Supervision trees** - Fault tolerance
4. **Profile-guided inlining** - Measure before optimizing

### Phase 3: Polish (Weeks 21-28)
1. **Full optimization pipeline** - After benchmarking proves need
2. **Generational arenas** - Only if allocation profiles demand
3. **Macro hygiene** - Can ship 1.0 without
4. **Formal verification** - Post-1.0 research project

### Risk Mitigation Strategies

**Memory Management Complexity:**
- Start with reference counting fallback
- Add region inference incrementally
- Benchmark against RC baseline continuously
- Ship with "safe mode" flag for debugging

**Build Times:**
- Parallelize module compilation from day one
- Cache aggressively at every layer
- Profile-guided compilation ordering
- Lazy loading of unused subsystems

**Debugging Experience:**
- Source maps mandatory from v0.1
- REPL must always work, even partially
- Every optimization pass preserves debug info
- Gradual degradation over hard failures

Each component requires rigorous testing via property-based tests, fuzzing, and real-world validation before production deployment. 

## Technical Debt Budget

Acceptable shortcuts for v1.0:
- Macro system limited to simple substitution
- Region inference conservative (prefer heap)
- Actor scheduling non-optimal (round-robin)
- Missing some optimization passes

Unacceptable compromises:
- Error message quality
- Memory safety violations  
- Rust interop limitations
- REPL responsiveness

## Success Metrics

- Cold start: <10ms (measured, not theoretical)
- Parser throughput: >50MB/s sustained
- Memory overhead vs handwritten Rust: <10%
- Incremental recompilation: <100ms for single file
- Error message clarity: 90% first-time understanding
- Zero segfaults in 1M fuzzing hours